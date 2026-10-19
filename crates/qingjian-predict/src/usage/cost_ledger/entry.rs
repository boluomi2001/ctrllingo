//! 账本中的单次请求记录。

use serde::{Deserialize, Serialize};

use super::state::AttemptState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct LedgerEntry {
    pub id: String,

    pub day: String,

    pub started_unix_secs: u64,

    pub provider_host: String,

    pub model: String,

    pub price_version: Option<String>,

    pub state: AttemptState,

    pub input_tokens: Option<u64>,

    pub output_tokens: Option<u64>,

    pub cached_input_tokens: Option<u64>,

    pub cache_detail_known: bool,

    pub lower_nano_cny: Option<u64>,

    pub upper_nano_cny: Option<u64>,
}
