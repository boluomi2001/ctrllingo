//! 菜单与费用卡片共用的文字，未知用量和费用保持显式。

use std::path::Path;

use qingjian_core::Language;
use qingjian_predict::{CostLedger, CostLedgerSummary};

use super::beijing_now;
use super::panel_view::CostPanelView;
use super::state::MonitorState;

pub(super) fn ledger(path: &Path) -> (CostPanelView, CostPanelView) {
    let today = beijing_now().date().to_string();
    let (today_summary, total_summary) = if !path.exists() {
        return (
            empty_panel(&format!("今日（{today}，北京时间）")),
            empty_panel("累计"),
        );
    } else {
        match CostLedger::load(path)
            .and_then(|ledger| Ok((ledger.summary(&today)?, ledger.total()?)))
        {
            Ok(summaries) => summaries,
            Err(_) => {
                let mut panel = empty_panel(&format!("今日（{today}，北京时间）"));
                panel.tokens = "未知".to_owned();
                panel.amount = "费用未知（账本暂不可读）".to_owned();
                panel.details = "费用账本暂不可读；原文件保留，费用未知。".to_owned();
                let mut total = empty_panel("累计");
                total.tokens = panel.tokens.clone();
                total.amount = panel.amount.clone();
                total.details = panel.details.clone();
                return (panel, total);
            }
        }
    };
    (
        panel(&today_summary, &format!("今日（{today}，北京时间）")),
        panel(&total_summary, "累计"),
    )
}

pub(super) fn panel(summary: &CostLedgerSummary, period: &str) -> CostPanelView {
    let amount = amount(summary);
    let first_record = jiff::Timestamp::from_second(summary.since_unix_secs as i64)
        .ok()
        .map(|timestamp| {
            timestamp
                .to_zoned(jiff::tz::Offset::from_hours(8).unwrap().to_time_zone())
                .strftime("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "未知".to_owned());
    let history = summary.requests > 0;
    let mut request_parts = Vec::new();
    if summary.translation_requests > 0 || history {
        request_parts.push(format!("翻译{}次", summary.translation_requests));
    }
    if summary.prediction_requests > 0 || history {
        request_parts.push(format!("云联想{}次", summary.prediction_requests));
    }
    if summary.other_requests > 0 || history {
        request_parts.push(format!("其他/旧记录{}次", summary.other_requests));
    }
    let requests = if request_parts.is_empty() {
        "暂无请求记录".to_owned()
    } else {
        request_parts.join("、")
    };
    let mut flags = Vec::new();
    if summary.pending > 0 {
        flags.push(format!("未完成 {} 次", summary.pending));
    }
    if summary.unknown > 0 {
        flags.push(format!("用量或费用未知 {} 次", summary.unknown));
    }
    let flags = if flags.is_empty() {
        "无未完成或未知记录".to_owned()
    } else {
        flags.join("；")
    };
    let prices = if summary.price_versions.is_empty() {
        "无可用价格".to_owned()
    } else {
        summary.price_versions.join("、")
    };
    let tokens = summary.input_tokens.saturating_add(summary.output_tokens);
    CostPanelView {
        tokens: if summary.unknown_token_requests > 0 {
            format!("已知 {tokens}")
        } else {
            tokens.to_string()
        },
        token_detail: format!(
            "输入 {} · 输出 {}\n其中已知缓存命中 {}（包含在输入中）",
            summary.input_tokens, summary.output_tokens, summary.cached_input_tokens
        ),
        amount: amount.clone(),
        balance: String::new(),
        updated: String::new(),
        requests: requests.clone(),
        target_language: String::new(),
        details: format!(
            "{period}估算：{amount}\n实际模型请求：{} 次\n{}\n输入 token：{}（其中缓存命中：{}）\n输出 token：{}\n{flags}\n首次记账：{first_record}\n\n价格版本：{prices}\n缓存或法定节假日信息不全时显示区间。\n缺用量或未知模型的费用保持未知。\n强制退出前未完成的请求不能视为免费。\n费用账本不保存输入原文或密钥。",
            summary.requests,
            requests,
            summary.input_tokens,
            summary.cached_input_tokens,
            summary.output_tokens,
        ),
    }
}

pub(super) fn with_language(mut panel: CostPanelView, language: Language) -> CostPanelView {
    panel.target_language = match language {
        Language::Chinese => "中文",
        Language::English => "英语",
        Language::Japanese => "日语",
        Language::Spanish => "西班牙语",
        Language::Vietnamese => "越南语",
        Language::German => "德语",
    }
    .to_owned();
    panel
}

pub(super) fn balance(state: &MonitorState) -> String {
    state
        .balance
        .as_ref()
        .map(|snapshot| snapshot.text.clone())
        .or_else(|| state.error.clone())
        .unwrap_or_else(|| "尚未查询".to_owned())
}

pub(super) fn updated(state: &MonitorState) -> String {
    if state.in_flight {
        return state.balance.as_ref().map_or_else(
            || "更新中…".to_owned(),
            |snapshot| format!("更新中…上次成功：{}", beijing_timestamp(snapshot.updated)),
        );
    }
    match (&state.balance, &state.error) {
        (Some(snapshot), Some(error)) => {
            format!("{}；更新失败：{error}", beijing_timestamp(snapshot.updated))
        }
        (Some(snapshot), None) => format!("最后成功：{}", beijing_timestamp(snapshot.updated)),
        (None, Some(error)) => format!("查询失败：{error}"),
        (None, None) => "尚未查询".to_owned(),
    }
}

fn empty_panel(period: &str) -> CostPanelView {
    CostPanelView {
        tokens: "0".to_owned(),
        token_detail: "输入 0 · 输出 0\n尚无新请求记录".to_owned(),
        requests: "暂无请求记录".to_owned(),
        amount: "暂无新记录".to_owned(),
        details: format!("{period}尚无新记录。新版本从首次模型请求开始记账，不补算升级前费用。"),
        ..CostPanelView::default()
    }
}

fn amount(summary: &CostLedgerSummary) -> String {
    match summary.upper_nano_cny {
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
    }
}

fn beijing_timestamp(updated: std::time::SystemTime) -> String {
    let Ok(duration) = updated.duration_since(std::time::UNIX_EPOCH) else {
        return "未知".to_owned();
    };
    jiff::Timestamp::from_second(duration.as_secs() as i64)
        .ok()
        .map(|timestamp| {
            timestamp
                .to_zoned(jiff::tz::Offset::from_hours(8).unwrap().to_time_zone())
                .strftime("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "未知".to_owned())
}
