//! 经过边界校验的账户余额，保留币种与成功时间。

use std::time::SystemTime;

use serde_json::Value;

#[derive(Clone)]
pub(super) struct BalanceSnapshot {
    pub(super) text: String,

    pub(super) updated: SystemTime,
}

impl BalanceSnapshot {
    pub(super) fn parse(value: Value) -> Result<Self, String> {
        let balances = value["balance_infos"]
            .as_array()
            .ok_or("余额响应格式不正确")?;
        let mut parts = Vec::new();
        for balance in balances {
            let currency = balance["currency"].as_str().ok_or("余额缺少币种")?;
            if !matches!(currency, "CNY" | "USD") {
                continue;
            }
            let total = balance["total_balance"].as_str().ok_or("余额缺少金额")?;
            if total.len() > 32
                || !total
                    .parse::<f64>()
                    .is_ok_and(|n| n.is_finite() && n >= 0.0)
            {
                return Err("余额金额格式不正确".to_owned());
            }
            parts.push(format!("{currency} {total}"));
        }
        if parts.is_empty() {
            return Err("账户未返回支持的币种".to_owned());
        }
        Ok(Self {
            text: parts.join(" / "),
            updated: SystemTime::now(),
        })
    }
}
