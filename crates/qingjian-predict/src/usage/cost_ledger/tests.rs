//! 费用边界、未知用量与持久化状态测试。

use std::path::PathBuf;

use serde_json::json;

use super::ledger::CostLedger;
use super::pricing::{cost_bounds, is_peak_hour, supported_model};

fn scratch() -> PathBuf {
    std::env::temp_dir()
        .join(format!("qingjian-cost-ledger-{}", uuid::Uuid::new_v4()))
        .join("api-cost-ledger.json")
}

#[test]
fn computes_cached_and_cache_unknown_bounds_in_integer_nanocny() {
    assert_eq!(cost_bounds(100, 2, Some(40), false), (68_800, 68_800));
    assert_eq!(cost_bounds(100, 2, Some(40), true), (68_800, 137_600));
    assert_eq!(cost_bounds(100, 2, None, false), (10_000, 108_000));
    assert_eq!(cost_bounds(100, 2, None, true), (10_000, 216_000));
}

#[test]
fn prices_only_the_exact_official_host_and_supported_model_names() {
    assert!(supported_model("api.deepseek.com", "deepseek-flash"));
    assert!(supported_model("api.deepseek.com", "deepseek-v4-flash"));
    assert!(supported_model(
        "api.deepseek.com",
        "deepseek-v4-flash-vision-exp"
    ));
    assert!(!supported_model("proxy.deepseek.com", "deepseek-flash"));
    assert!(!supported_model(
        "api.deepseek.com.evil.test",
        "deepseek-flash"
    ));
    assert!(!supported_model("api.deepseek.com", "deepseek-chat"));
}

#[test]
fn peak_windows_include_start_and_exclude_end() {
    assert!(!is_peak_hour(8));
    assert!(is_peak_hour(9));
    assert!(is_peak_hour(11));
    assert!(!is_peak_hour(12));
    assert!(!is_peak_hour(13));
    assert!(is_peak_hour(14));
    assert!(is_peak_hour(17));
    assert!(!is_peak_hour(18));
}

#[test]
fn pending_and_unknown_attempts_stay_distinct_and_survive_reload() {
    let path = scratch();
    let ledger = CostLedger::load(&path).unwrap();
    let pending = ledger
        .record_attempt("api.deepseek.com", "deepseek-flash")
        .unwrap();
    let failed = ledger
        .record_attempt("api.deepseek.com", "deepseek-flash")
        .unwrap();
    ledger.record_result(&failed, None).unwrap();
    let summary = CostLedger::load(&path)
        .unwrap()
        .summary(&pending.day)
        .unwrap();
    assert_eq!(summary.requests, 2);
    assert_eq!(summary.pending, 1);
    assert_eq!(summary.unknown, 1);
    assert_eq!(summary.upper_nano_cny, None);
}

#[test]
fn malformed_ledger_is_preserved_and_never_reset() {
    let path = scratch();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, b"{broken").unwrap();
    assert!(CostLedger::load(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
}

#[test]
fn extracts_deepseek_and_openai_cache_detail_without_double_counting_input() {
    let deepseek = super::super::tokens::UsageTokens::from_response(&json!({
        "usage": {"prompt_tokens": 100, "completion_tokens": 5,
            "prompt_cache_hit_tokens": 30, "prompt_cache_miss_tokens": 70}
    }))
    .unwrap();
    assert_eq!(deepseek.input, 100);
    assert_eq!(deepseek.cached_input, 30);
    assert!(deepseek.cache_detail_known);

    let openai = super::super::tokens::UsageTokens::from_response(&json!({
        "usage": {"prompt_tokens": 100, "completion_tokens": 5,
            "prompt_tokens_details": {"cached_tokens": 60}}
    }))
    .unwrap();
    assert_eq!(openai.input, 100);
    assert_eq!(openai.cached_input, 60);
    assert!(openai.cache_detail_known);

    let unspecified = super::super::tokens::UsageTokens::from_response(&json!({
        "usage": {"prompt_tokens": 100, "completion_tokens": 5}
    }))
    .unwrap();
    assert_eq!(unspecified.cached_input, 0);
    assert!(!unspecified.cache_detail_known);
}

#[test]
fn response_remains_on_the_request_start_day_and_duplicate_results_are_ignored() {
    let path = scratch();
    let ledger = CostLedger::load(&path).unwrap();
    let request = ledger
        .record_attempt("api.deepseek.com", "deepseek-flash")
        .unwrap();
    let mut data: super::data::LedgerData =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    data.attempts[0].day = "2026-10-18".to_owned();
    data.attempts[0].started_unix_secs = "2026-10-18T15:59:59Z"
        .parse::<jiff::Timestamp>()
        .unwrap()
        .as_second() as u64;
    std::fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
    let response =
        json!({"usage": {"prompt_tokens":100,"completion_tokens":2,"prompt_cache_hit_tokens":40}});
    ledger.record_result(&request, Some(&response)).unwrap();
    ledger.record_result(&request, Some(&response)).unwrap();
    let summary = ledger.summary("2026-10-18").unwrap();
    assert_eq!(summary.requests, 1);
    assert_eq!(summary.input_tokens, 100);
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.lower_nano_cny, 68_800);
    assert_eq!(summary.upper_nano_cny, Some(68_800));
    assert_eq!(ledger.summary("2026-10-19").unwrap().requests, 0);
}

#[test]
fn parallel_writers_preserve_all_attempts_and_usage() {
    let path = scratch();
    let threads: Vec<_> = (0..4).map(|_| {
        let path = path.clone();
        std::thread::spawn(move || {
            for _ in 0..4 {
                let ledger = CostLedger::load(&path).unwrap();
                let request = ledger.record_attempt("api.deepseek.com", "deepseek-flash").unwrap();
                ledger.record_result(&request, Some(&json!({"usage": {"prompt_tokens":100,"completion_tokens":2,"prompt_cache_hit_tokens":40}}))).unwrap();
            }
        })
    }).collect();
    for thread in threads {
        thread.join().unwrap();
    }
    let summary = CostLedger::load(&path).unwrap().today().unwrap();
    assert_eq!(summary.requests, 16);
    assert_eq!(summary.input_tokens, 1600);
    assert_eq!(summary.cached_input_tokens, 640);
    assert_eq!(summary.output_tokens, 32);
    assert_eq!(summary.pending, 0);
    assert_eq!(summary.unknown, 0);
    assert_eq!(summary.lower_nano_cny, 68_800 * 16);
}
