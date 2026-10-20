//! 费用卡片的显示数据；只包含可显示文字，不包含凭据。

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CostPanelView {
    pub tokens: String,

    pub token_detail: String,

    pub amount: String,

    pub balance: String,

    pub updated: String,

    pub requests: String,

    pub target_language: String,

    pub details: String,
}
