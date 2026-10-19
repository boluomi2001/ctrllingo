//! Server 全局的余额节流状态与本地摘要缓存。

use std::time::{Instant, SystemTime};

use super::snapshot::BalanceSnapshot;

#[derive(Default)]
pub(super) struct MonitorState {
    pub(super) last_attempt: Option<Instant>,

    pub(super) in_flight: bool,

    pub(super) balance: Option<BalanceSnapshot>,

    pub(super) error: Option<String>,

    pub(super) last_read: Option<Instant>,

    pub(super) reading: bool,

    pub(super) ledger_stamp: Option<(String, Option<SystemTime>, u64)>,

    pub(super) cost_summary: String,

    pub(super) cost_details: String,
}
