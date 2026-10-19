//! 账本文件的根数据结构。

use serde::{Deserialize, Serialize};

use super::entry::LedgerEntry;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct LedgerData {
    pub since_unix_secs: u64,

    pub attempts: Vec<LedgerEntry>,
}
