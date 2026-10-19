//! DeepSeek Flash 公开价格的纳元换算与峰谷时段边界。
//! 价格核对日期 2026-10-19：https://api-docs.deepseek.com/zh-cn/quick_start/pricing/

use jiff::{Timestamp, Zoned, tz::TimeZone};
use serde_json::Value;

use super::super::tokens::UsageTokens;

const CACHED_NANO_PER_TOKEN: u64 = 20;
const MISSED_NANO_PER_TOKEN: u64 = 1_000;
const OUTPUT_NANO_PER_TOKEN: u64 = 4_000;

pub(super) fn request_clock() -> (String, u64) {
    let now = Timestamp::now();
    let zoned = now.to_zoned(beijing_timezone());
    let date = zoned.date();
    (date.to_string(), now.as_second() as u64)
}

pub(super) fn supported_model(host: &str, model: &str) -> bool {
    host == "api.deepseek.com"
        && matches!(
            model,
            "deepseek-flash" | "deepseek-v4-flash" | "deepseek-v4-flash-vision-exp"
        )
}

pub(super) fn usage(response: Option<&Value>) -> Option<UsageTokens> {
    UsageTokens::from_response(response?)
}

pub(super) fn cost_bounds(
    input: u64,
    output: u64,
    cached: Option<u64>,
    peak_uncertain: bool,
) -> (u64, u64) {
    let (low_cached, low_missed, high_cached, high_missed) = match cached {
        Some(cached) if cached <= input => (cached, input - cached, cached, input - cached),
        _ => (input, 0, 0, input),
    };
    let low_off_peak = low_cached
        .saturating_mul(CACHED_NANO_PER_TOKEN)
        .saturating_add(low_missed.saturating_mul(MISSED_NANO_PER_TOKEN))
        .saturating_add(output.saturating_mul(OUTPUT_NANO_PER_TOKEN));
    let high_off_peak = high_cached
        .saturating_mul(CACHED_NANO_PER_TOKEN)
        .saturating_add(high_missed.saturating_mul(MISSED_NANO_PER_TOKEN))
        .saturating_add(output.saturating_mul(OUTPUT_NANO_PER_TOKEN));
    if peak_uncertain {
        let high_peak = high_off_peak.saturating_mul(2);
        (low_off_peak, high_peak)
    } else {
        (low_off_peak, high_off_peak)
    }
}

pub(super) fn is_peak_hour(hour: i8) -> bool {
    (9..12).contains(&hour) || (14..18).contains(&hour)
}

pub(super) fn today() -> String {
    Timestamp::now()
        .to_zoned(beijing_timezone())
        .date()
        .to_string()
}

pub(super) fn beijing_time(unix_secs: u64) -> Zoned {
    Timestamp::from_second(unix_secs as i64)
        .expect("valid request timestamp")
        .to_zoned(beijing_timezone())
}

fn beijing_timezone() -> TimeZone {
    TimeZone::fixed(jiff::tz::offset(8))
}
