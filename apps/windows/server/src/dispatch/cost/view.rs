//! 菜单与费用详情共用的文字，缺失记录不伪装成零费用。

use std::path::Path;

use qingjian_predict::CostLedger;

use super::beijing_now;
use super::state::MonitorState;

pub(super) fn ledger(path: &Path) -> (String, String) {
    let day = beijing_now().date().to_string();
    if !path.exists() {
        return (
            "今日尚无新记录".to_owned(),
            format!("{day}（北京时间）\n新版本从首次模型请求开始记账，不补算升级前费用。"),
        );
    }
    let summary = match CostLedger::load(path).and_then(|ledger| ledger.summary(&day)) {
        Ok(summary) => summary,
        Err(_) => {
            return (
                "账本暂不可读".to_owned(),
                "费用账本暂不可读；原文件保留，费用未知。".to_owned(),
            );
        }
    };
    let amount = match summary.upper_nano_cny {
        Some(upper) if upper == summary.lower_nano_cny => format!("¥{:.6}", upper as f64 / 1e9),
        Some(upper) => format!(
            "¥{:.6}–{:.6}",
            summary.lower_nano_cny as f64 / 1e9,
            upper as f64 / 1e9
        ),
        None => format!(
            "已知≥¥{:.6}（另有未知费用）",
            summary.lower_nano_cny as f64 / 1e9
        ),
    };
    let cached = if summary.unknown > 0 {
        "（部分缓存或用量未知）"
    } else {
        ""
    };
    let since = jiff::Timestamp::from_second(summary.since_unix_secs as i64)
        .ok()
        .map(|timestamp| {
            timestamp
                .to_zoned(jiff::tz::Offset::from_hours(8).unwrap().to_time_zone())
                .strftime("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "未知".to_owned());
    let prices = if summary.price_versions.is_empty() {
        "无可用价格".to_owned()
    } else {
        summary.price_versions.join("、")
    };
    let details = format!(
        "本输入法今日估算：{amount}\n日期：{day}（北京时间）\n实际模型请求：{} 次；未完成：{} 次\n输入 token：{}（其中已知缓存命中：{}）\n输出 token：{}\n含未知明细的请求：{} 次 {cached}\n首次记账：{since}\n\n价格版本：{prices}\n缓存或法定节假日信息不全时显示区间。\n缺用量或未知模型的费用保持未知。\n强制退出前未完成的请求不能视为免费。\n费用账本不保存输入原文或密钥。",
        summary.requests,
        summary.pending,
        summary.input_tokens,
        summary.cached_input_tokens,
        summary.output_tokens,
        summary.unknown
    );
    (format!("今日 {amount} · {} 次", summary.requests), details)
}

pub(super) fn balance(state: &MonitorState) -> String {
    match &state.balance {
        Some(snapshot) => {
            let age = snapshot.updated.elapsed().unwrap_or_default().as_secs();
            let status = if state.in_flight {
                "，更新中"
            } else if state.error.is_some() {
                "，更新失败"
            } else {
                ""
            };
            format!("{} · {}秒前{status}", snapshot.text, age)
        }
        None if state.in_flight => "查询中…".to_owned(),
        None => state
            .error
            .clone()
            .unwrap_or_else(|| "尚未查询，打开菜单后按需更新".to_owned()),
    }
}
