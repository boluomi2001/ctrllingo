//! 按需查询官方余额；所有应用共用同一个 Server 实例的在途请求。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use qingjian_predict::{COST_LEDGER_FILE, PredictConfig};

use super::beijing_now;
use super::panel_view::CostPanelView;
use super::snapshot::BalanceSnapshot;
use super::state::MonitorState;
use super::view;

pub(in crate::dispatch) struct CostMonitor {
    config: PredictConfig,

    ledger_path: PathBuf,

    state: Arc<Mutex<MonitorState>>,
}

impl CostMonitor {
    pub(in crate::dispatch) fn new(config: &PredictConfig, dir: &Path) -> Self {
        Self {
            config: config.clone(),
            ledger_path: dir.join(COST_LEDGER_FILE),
            state: Arc::default(),
        }
    }

    pub(in crate::dispatch) fn refresh_balance(&self) {
        let Ok(url) = reqwest::Url::parse(&self.config.base_url) else {
            return;
        };
        if url.scheme() != "https" || url.host_str() != Some("api.deepseek.com") {
            self.state.lock().unwrap().error =
                Some("当前服务不支持 DeepSeek 官方余额查询".to_owned());
            return;
        }
        let Some(key) = self.config.resolve_api_key() else {
            self.state.lock().unwrap().error = Some("请先在云服务设置中配置密钥".to_owned());
            return;
        };
        self.start_balance_query("https://api.deepseek.com/user/balance".to_owned(), key);
    }

    pub(super) fn start_balance_query(&self, endpoint: String, key: String) {
        let mut state = self.state.lock().unwrap();
        if state.in_flight {
            return;
        }
        state.last_attempt = Some(Instant::now());
        state.in_flight = true;
        state.error = None;
        drop(state);
        let shared = self.state.clone();
        let spawned = std::thread::Builder::new()
            .name("qingjian-balance".to_owned())
            .spawn(move || {
                let result = query_balance(&endpoint, &key);
                let mut state = shared.lock().unwrap();
                state.in_flight = false;
                match result {
                    Ok(snapshot) => {
                        state.balance = Some(snapshot);
                        state.error = None;
                    }
                    Err(error) => state.error = Some(error),
                }
            });
        if spawned.is_err() {
            let mut state = self.state.lock().unwrap();
            state.in_flight = false;
            state.error = Some("余额查询暂时无法启动".to_owned());
        }
    }

    pub(in crate::dispatch) fn summaries(&self) -> (String, String) {
        let mut state = self.state.lock().unwrap();
        if state
            .last_read
            .is_none_or(|last| last.elapsed() >= Duration::from_secs(1))
            && !state.reading
        {
            state.reading = true;
            let shared = self.state.clone();
            let path = self.ledger_path.clone();
            // 账本随历史增长，磁盘读取和解析不能卡住按键与模式同步。
            let spawned = std::thread::Builder::new()
                .name("qingjian-cost-read".to_owned())
                .spawn(move || {
                    let metadata = std::fs::metadata(&path).ok();
                    let stamp = (
                        beijing_now().date().to_string(),
                        metadata.as_ref().and_then(|m| m.modified().ok()),
                        metadata.as_ref().map_or(0, |m| m.len()),
                    );
                    let changed = shared.lock().unwrap().ledger_stamp.as_ref() != Some(&stamp);
                    let update = changed.then(|| view::ledger(&path));
                    let mut state = shared.lock().unwrap();
                    if let Some((today, total)) = update {
                        state.today_panel = Some(today);
                        state.total_panel = Some(total);
                        state.ledger_stamp = Some(stamp);
                    }
                    state.last_read = Some(Instant::now());
                    state.reading = false;
                });
            if spawned.is_err() {
                state.reading = false;
                state.last_read = Some(Instant::now());
            }
        }
        let summary = state
            .today_panel
            .as_ref()
            .map(|panel| panel.amount.clone())
            .unwrap_or_else(|| "统计读取中…".to_owned());
        (summary, view::balance(&state))
    }

    pub(in crate::dispatch) fn panel(
        &self,
        total: bool,
        language: qingjian_core::Language,
    ) -> CostPanelView {
        self.summaries();
        let state = self.state.lock().unwrap();
        let mut panel = if total {
            state.total_panel.clone()
        } else {
            state.today_panel.clone()
        }
        .unwrap_or_else(|| CostPanelView {
            amount: "统计读取中…".to_owned(),
            details: "费用统计读取中…".to_owned(),
            ..CostPanelView::default()
        });
        panel.balance = view::balance(&state);
        panel.updated = view::updated(&state);
        panel.details.push_str(&format!(
            "\n\nDeepSeek 账户余额：{}\n{}\n账户余额包含该账户的其他使用和充值，不等于本输入法消费。\n每次打开费用卡片都会自动查询余额。\n估算使用人民币公开价格，以平台账单为准。",
            panel.balance, panel.updated
        ));
        view::with_language(panel, language)
    }
}

fn query_balance(endpoint: &str, key: &str) -> Result<BalanceSnapshot, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| "余额查询无法启动".to_owned())?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "余额查询无法启动")?;
        let response = client
            .get(endpoint)
            .bearer_auth(key)
            .send()
            .await
            .map_err(|_| "余额查询网络失败，保留上次结果")?;
        if !response.status().is_success() {
            return Err(format!(
                "余额查询失败（HTTP {}），保留上次结果",
                response.status().as_u16()
            ));
        }
        let value = response.json().await.map_err(|_| "余额响应格式不正确")?;
        BalanceSnapshot::parse(value)
    })
}

#[cfg(test)]
#[path = "monitor_tests.rs"]
mod tests;
