//! 某一北京时间自然日的账本汇总。

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostLedgerSummary {
    pub day: String,

    pub price_versions: Vec<String>,

    pub requests: u64,

    pub translation_requests: u64,

    pub prediction_requests: u64,

    pub other_requests: u64,

    pub input_tokens: u64,

    pub output_tokens: u64,

    pub cached_input_tokens: u64,

    pub unknown: u64,

    pub unknown_token_requests: u64,

    pub pending: u64,

    pub lower_nano_cny: u64,

    pub upper_nano_cny: Option<u64>,

    pub since_unix_secs: u64,
}
