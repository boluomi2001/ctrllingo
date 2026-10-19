//! 按真实 HTTP 尝试保存云 API token 与费用估算；未知用量和进行中请求保持显式状态。

mod context;
mod data;
mod entry;
mod ledger;
mod pricing;
mod state;
mod summary;

pub use context::AttemptContext;
pub use ledger::{COST_LEDGER_FILE, CostLedger};
pub use summary::CostLedgerSummary;

const PRICE_VERSION: &str = "deepseek-flash-cny-2026-10-19";

#[cfg(test)]
mod tests;
