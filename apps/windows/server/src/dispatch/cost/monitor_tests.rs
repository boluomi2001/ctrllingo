//! 本机 HTTP 模拟验证余额异步查询、全局节流与失败后的缓存保留。

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use qingjian_core::Engine;
use qingjian_dictionary::Dictionary;
use qingjian_platform::Config;
use qingjian_platform::protocol::{ClientMessage, IndicatorCommand, SessionId};
use serde_json::json;

use super::super::snapshot::BalanceSnapshot;
use super::CostMonitor;
use crate::dispatch::{DataDirs, Router, RouterConfig};

fn temporary_dir(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "qingjian-cost-{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn mock_server(
    responses: Vec<(u16, String, Duration)>,
) -> (String, Arc<AtomicUsize>, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(AtomicUsize::new(0));
    let server_requests = requests.clone();
    let server = std::thread::spawn(move || {
        for (status, body, delay) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let _ = stream.read(&mut request).unwrap();
            server_requests.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(delay);
            let reason = if status == 200 {
                "OK"
            } else {
                "Internal Server Error"
            };
            let response = format!(
                concat!(
                    "HTTP/1.1 {status} {reason}\r\n",
                    "Content-Type: application/json\r\n",
                    "Content-Length: {}\r\n",
                    "Connection: close\r\n\r\n{body}"
                ),
                body.len(),
                status = status,
                reason = reason,
                body = body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (endpoint, requests, server)
}

fn wait_for_idle(monitor: &CostMonitor) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !monitor.state.lock().unwrap().in_flight {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("余额请求在 3 秒内没有结束");
}

fn success_body() -> String {
    json!({"balance_infos": [{"currency": "USD", "total_balance": "12.34"}]}).to_string()
}

#[test]
fn balance_query_returns_while_local_server_is_delayed() {
    let (endpoint, requests, server) =
        mock_server(vec![(200, success_body(), Duration::from_millis(800))]);
    let dir = temporary_dir("async");
    let monitor = CostMonitor::new(&Default::default(), &dir);

    let started = Instant::now();
    monitor.start_balance_query(endpoint, "test-key".to_owned());
    let elapsed = started.elapsed();

    assert!(
        elapsed < Duration::from_millis(400),
        "请求调用阻塞了 {elapsed:?}"
    );
    wait_for_idle(&monitor);
    server.join().unwrap();
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert!(monitor.summaries().1.contains("USD 12.34"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn concurrent_and_failed_requests_share_the_sixty_second_throttle() {
    let (endpoint, requests, server) =
        mock_server(vec![(500, "{}".to_owned(), Duration::from_millis(250))]);
    let dir = temporary_dir("throttle");
    let monitor = Arc::new(CostMonitor::new(&Default::default(), &dir));
    let callers = 8;
    let start = Arc::new(Barrier::new(callers + 1));
    let threads: Vec<_> = (0..callers)
        .map(|_| {
            let monitor = monitor.clone();
            let endpoint = endpoint.clone();
            let start = start.clone();
            std::thread::spawn(move || {
                start.wait();
                monitor.start_balance_query(endpoint, "test-key".to_owned());
            })
        })
        .collect();
    start.wait();
    for thread in threads {
        thread.join().unwrap();
    }
    wait_for_idle(&monitor);
    let failed_attempt = monitor.state.lock().unwrap().last_attempt;
    monitor.start_balance_query(endpoint, "test-key".to_owned());
    server.join().unwrap();

    assert_eq!(requests.load(Ordering::SeqCst), 1);
    assert_eq!(monitor.state.lock().unwrap().last_attempt, failed_attempt);
    assert!(monitor.summaries().1.contains("HTTP 500"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn failed_refresh_keeps_the_previous_balance_and_success_time() {
    let (endpoint, _, server) = mock_server(vec![(500, "{}".to_owned(), Duration::ZERO)]);
    let dir = temporary_dir("retain");
    let monitor = CostMonitor::new(&Default::default(), &dir);
    let updated = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    {
        let mut state = monitor.state.lock().unwrap();
        state.balance = Some(BalanceSnapshot {
            text: "CNY 8.00 / USD 1.00".to_owned(),
            updated,
        });
        state.last_attempt = Some(Instant::now() - Duration::from_secs(61));
    }

    monitor.start_balance_query(endpoint, "test-key".to_owned());
    wait_for_idle(&monitor);
    server.join().unwrap();

    let state = monitor.state.lock().unwrap();
    let balance = state.balance.as_ref().expect("旧余额应保留");
    assert_eq!(balance.text, "CNY 8.00 / USD 1.00");
    assert_eq!(balance.updated, updated);
    assert!(state.error.as_deref().unwrap().contains("HTTP 500"));
    drop(state);
    let summary = monitor.summaries().1;
    assert!(summary.contains("CNY 8.00 / USD 1.00"));
    assert!(summary.contains("更新失败"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn different_sessions_use_the_routers_single_cost_monitor() {
    let dir = temporary_dir("sessions");
    let mut config = Config::default();
    config.predict.api_key = Some("test-key".to_owned());
    let mut router = Router::new(Engine::new(Dictionary::default()), RouterConfig::default());
    router.watch_config(
        &config,
        dir.join("config.toml"),
        dir.clone(),
        DataDirs {
            user_root: Some(dir.clone()),
            ..DataDirs::default()
        },
    );
    let last_attempt = Instant::now();
    router
        .costs
        .as_ref()
        .unwrap()
        .state
        .lock()
        .unwrap()
        .last_attempt = Some(last_attempt);

    for session in [SessionId(1), SessionId(2)] {
        let _ = router.handle(ClientMessage::Indicator {
            session,
            command: IndicatorCommand::RefreshBalance,
        });
    }

    assert_eq!(
        router
            .costs
            .as_ref()
            .unwrap()
            .state
            .lock()
            .unwrap()
            .last_attempt,
        Some(last_attempt)
    );
    drop(router);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn balance_response_requires_fields_finite_amounts_and_supported_currencies() {
    assert!(BalanceSnapshot::parse(json!({})).is_err());
    assert!(BalanceSnapshot::parse(json!({"balance_infos": [{}]})).is_err());
    assert!(
        BalanceSnapshot::parse(json!({
            "balance_infos": [{"currency": "USD"}]
        }))
        .is_err()
    );
    for amount in ["NaN", "inf", "-Infinity"] {
        let response = json!({
            "balance_infos": [{"currency": "USD", "total_balance": amount}]
        });
        assert!(BalanceSnapshot::parse(response).is_err());
    }

    let parsed = BalanceSnapshot::parse(json!({
        "balance_infos": [
            {"currency": "CNY", "total_balance": "3.20"},
            {"currency": "USD", "total_balance": "1.50"},
            {"currency": "CNY\nUSD 999", "total_balance": "999"},
            {"currency": "EUR", "total_balance": "0"}
        ]
    }))
    .unwrap();
    assert_eq!(parsed.text, "CNY 3.20 / USD 1.50");
    assert!(
        BalanceSnapshot::parse(json!({
            "balance_infos": [{"currency": "EUR", "total_balance": "999"}]
        }))
        .is_err()
    );
}

#[test]
fn unknown_provider_does_not_report_a_zero_balance_or_make_a_request() {
    let dir = temporary_dir("provider");
    let config = qingjian_predict::PredictConfig {
        base_url: "https://api.example.com/v1".to_owned(),
        api_key: None,
        api_key_env: format!("QINGJIAN_COST_TEST_NO_KEY_{}", std::process::id()),
        ..Default::default()
    };
    assert!(std::env::var_os(&config.api_key_env).is_none());
    let monitor = CostMonitor::new(&config, &dir);

    monitor.refresh_balance();
    let balance = monitor.summaries().1;

    assert!(balance.contains("当前服务不支持"));
    assert!(!balance.contains("0.00"));
    assert!(!balance.contains("USD 0"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn local_cost_summary_refreshes_after_recording_a_response() {
    let dir = temporary_dir("summary");
    let monitor = CostMonitor::new(&Default::default(), &dir);
    monitor.summaries();
    let wait_for_read = || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while monitor.state.lock().unwrap().reading {
            assert!(Instant::now() < deadline, "费用账本读取未结束");
            std::thread::sleep(Duration::from_millis(10));
        }
    };
    wait_for_read();
    let ledger =
        qingjian_predict::CostLedger::load(dir.join(qingjian_predict::COST_LEDGER_FILE)).unwrap();
    let request = ledger.record_attempt("local-test", "local-test").unwrap();
    ledger.record_result(&request, Some(&json!({"usage":{"prompt_tokens":100,"completion_tokens":20,"prompt_cache_hit_tokens":40}}))).unwrap();
    monitor.state.lock().unwrap().last_read = Some(Instant::now() - Duration::from_secs(2));
    monitor.summaries();
    wait_for_read();
    assert!(monitor.summaries().0.contains("1 次"));
    assert!(monitor.details().contains("未知费用"));
    drop(monitor);
    std::fs::remove_dir_all(dir).unwrap();
}
