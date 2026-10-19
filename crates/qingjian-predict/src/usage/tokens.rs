//! 从 OpenAI 兼容响应提取实际用量；缓存输入属于输入总量，不重复累加。

use serde_json::Value;

pub(super) struct UsageTokens {
    pub input: u64,

    pub output: u64,

    pub total: u64,

    pub cached_input: u64,

    pub cache_detail_known: bool,
}

impl UsageTokens {
    pub fn from_response(response: &Value) -> Option<Self> {
        let usage = response.get("usage")?;
        let input = usage.get("prompt_tokens")?.as_u64()?;
        let output = usage.get("completion_tokens")?.as_u64()?;
        let total = usage
            .get("total_tokens")
            .and_then(Value::as_u64)
            .or_else(|| input.checked_add(output))?;
        let cached_input = usage
            .get("prompt_cache_hit_tokens")
            .and_then(Value::as_u64)
            .or_else(|| {
                usage
                    .pointer("/prompt_tokens_details/cached_tokens")
                    .and_then(Value::as_u64)
            })
            .or_else(|| {
                usage
                    .get("prompt_cache_miss_tokens")
                    .and_then(Value::as_u64)
                    .and_then(|miss| input.checked_sub(miss))
            });
        let cache_detail_known = cached_input.is_some_and(|cached| cached <= input);
        Some(Self {
            input,
            output,
            total,
            cached_input: cached_input.filter(|cached| *cached <= input).unwrap_or(0),
            cache_detail_known,
        })
    }
}
