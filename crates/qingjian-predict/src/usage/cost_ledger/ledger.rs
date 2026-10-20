//! 按请求开始时间归档的跨进程费用账本。

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use qingjian_core::storage::write_atomic_str;
use serde_json::Value;

use super::context::AttemptContext;
use super::data::LedgerData;
use super::entry::LedgerEntry;
use super::pricing::{self, cost_bounds, supported_model, usage};
use super::state::AttemptState;
use super::{CostLedgerSummary, PRICE_VERSION};

/// 费用账本文件名，和原有 token 统计文件并存。
pub const COST_LEDGER_FILE: &str = "api-cost-ledger.json";

#[derive(Debug, Clone)]
pub struct CostLedger {
    path: PathBuf,
}

impl CostLedger {
    /// 读取账本；文件损坏时返回错误并保留原文件。
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_owned();
        read_data(&path)?;
        Ok(Self { path })
    }

    /// 发送一次真实请求前调用。重试必须为每次 HTTP 尝试分别调用。
    pub fn record_attempt(&self, provider_host: &str, model: &str) -> io::Result<AttemptContext> {
        self.record_attempt_for(provider_host, model, "other")
    }

    /// 按用途记录一次真实请求；未知用途归入 `other`。
    pub fn record_attempt_for(
        &self,
        provider_host: &str,
        model: &str,
        purpose: &str,
    ) -> io::Result<AttemptContext> {
        let (day, started_unix_secs) = pricing::request_clock();
        let id = uuid::Uuid::new_v4().to_string();
        let is_priced = supported_model(provider_host, model);
        let purpose = match purpose {
            "translate" | "compose" => purpose,
            _ => "other",
        };
        let entry = LedgerEntry {
            id: id.clone(),
            day: day.clone(),
            started_unix_secs,
            provider_host: provider_host.to_ascii_lowercase(),
            model: model.to_owned(),
            purpose: purpose.to_owned(),
            price_version: is_priced.then(|| PRICE_VERSION.to_owned()),
            state: AttemptState::Pending,
            input_tokens: None,
            output_tokens: None,
            cached_input_tokens: None,
            cache_detail_known: false,
            lower_nano_cny: None,
            upper_nano_cny: None,
        };
        with_data(&self.path, |data| {
            if data.since_unix_secs == 0 {
                data.since_unix_secs = started_unix_secs;
            }
            data.attempts.push(entry);
            Ok(())
        })?;
        Ok(AttemptContext { id, day })
    }

    /// 记录响应或失败。没有 usage 时不把 token 数或费用伪装成零。
    pub fn record_result(
        &self,
        context: &AttemptContext,
        response: Option<&Value>,
    ) -> io::Result<()> {
        with_data(&self.path, |data| {
            let entry = data
                .attempts
                .iter_mut()
                .find(|entry| entry.id == context.id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "ledger attempt missing"))?;
            if entry.state != AttemptState::Pending {
                return Ok(());
            }
            entry.state = AttemptState::Finished;
            if let Some(tokens) = usage(response) {
                entry.input_tokens = Some(tokens.input);
                entry.output_tokens = Some(tokens.output);
                entry.cached_input_tokens = Some(tokens.cached_input);
                entry.cache_detail_known = tokens.cache_detail_known;
                if entry.price_version.is_some() {
                    let (lower, upper) = cost_bounds(
                        tokens.input,
                        tokens.output,
                        tokens.cache_detail_known.then_some(tokens.cached_input),
                        is_peak_uncertain(entry.started_unix_secs),
                    );
                    entry.lower_nano_cny = Some(lower);
                    entry.upper_nano_cny = Some(upper);
                }
            }
            Ok(())
        })
    }

    /// 汇总传入的北京时间日期，格式为 `YYYY-MM-DD`。
    pub fn summary(&self, day: &str) -> io::Result<CostLedgerSummary> {
        let data = read_data(&self.path)?;
        summarize(
            data.attempts.iter().filter(|attempt| attempt.day == day),
            day,
            data.since_unix_secs,
        )
    }

    /// 汇总账本中的所有日期。
    pub fn total(&self) -> io::Result<CostLedgerSummary> {
        let data = read_data(&self.path)?;
        summarize(data.attempts.iter(), "全部", data.since_unix_secs)
    }

    /// 今日按北京时间汇总。
    pub fn today(&self) -> io::Result<CostLedgerSummary> {
        self.summary(&pricing::today())
    }
}

fn summarize<'a>(
    attempts: impl Iterator<Item = &'a LedgerEntry>,
    day: &str,
    since_unix_secs: u64,
) -> io::Result<CostLedgerSummary> {
    let attempts: Vec<_> = attempts.collect();
    let mut summary = CostLedgerSummary {
        day: day.to_owned(),
        price_versions: attempts
            .iter()
            .filter_map(|attempt| attempt.price_version.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        requests: attempts.len() as u64,
        translation_requests: attempts.iter().filter(|a| a.purpose == "translate").count() as u64,
        prediction_requests: attempts.iter().filter(|a| a.purpose == "compose").count() as u64,
        other_requests: attempts
            .iter()
            .filter(|a| a.purpose != "translate" && a.purpose != "compose")
            .count() as u64,
        input_tokens: 0,
        output_tokens: 0,
        cached_input_tokens: 0,
        unknown: 0,
        unknown_token_requests: attempts
            .iter()
            .filter(|attempt| attempt.input_tokens.is_none() || attempt.output_tokens.is_none())
            .count() as u64,
        pending: 0,
        lower_nano_cny: 0,
        upper_nano_cny: Some(0),
        since_unix_secs,
    };
    for attempt in attempts {
        if attempt.state == AttemptState::Pending {
            summary.pending += 1;
            summary.upper_nano_cny = None;
            continue;
        }
        if let Some(input) = attempt.input_tokens {
            summary.input_tokens = summary.input_tokens.saturating_add(input);
        }
        if let Some(output) = attempt.output_tokens {
            summary.output_tokens = summary.output_tokens.saturating_add(output);
        }
        if let Some(cached) = attempt.cached_input_tokens {
            summary.cached_input_tokens = summary.cached_input_tokens.saturating_add(cached);
        }
        if !attempt.cache_detail_known {
            summary.unknown += 1;
        }
        match (attempt.lower_nano_cny, attempt.upper_nano_cny) {
            (Some(lower), Some(upper)) => {
                summary.lower_nano_cny = summary.lower_nano_cny.saturating_add(lower);
                if let Some(total) = &mut summary.upper_nano_cny {
                    *total = total.saturating_add(upper);
                }
            }
            _ => {
                if attempt.cache_detail_known {
                    summary.unknown += 1;
                }
                summary.upper_nano_cny = None;
            }
        }
    }
    Ok(summary)
}

fn with_data(
    path: &Path,
    update: impl FnOnce(&mut LedgerData) -> io::Result<()>,
) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path.with_extension("lock"))?;
    lock.lock()?;
    let mut data = read_data(path)?;
    update(&mut data)?;
    let text = serde_json::to_string_pretty(&data).map_err(io::Error::other)?;
    write_atomic_str(path, &text)
}

fn read_data(path: &Path) -> io::Result<LedgerData> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(LedgerData::default()),
        Err(error) => Err(error),
    }
}

fn is_peak_uncertain(started_unix_secs: u64) -> bool {
    let zoned = pricing::beijing_time(started_unix_secs);
    let peak_time = pricing::is_peak_hour(zoned.hour());
    let weekday = !matches!(
        zoned.weekday(),
        jiff::civil::Weekday::Saturday | jiff::civil::Weekday::Sunday
    );
    weekday && peak_time
}
