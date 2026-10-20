//! Server 全局的余额请求状态与本地面板缓存。

use std::time::{Instant, SystemTime};

use super::panel_view::CostPanelView;
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

    pub(super) today_panel: Option<CostPanelView>,

    pub(super) total_panel: Option<CostPanelView>,
}
