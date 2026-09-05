use crate::config::ResilienceConfig;
use anyhow::{Result, anyhow};
use backon::{BackoffBuilder, ExponentialBackoff, ExponentialBuilder};
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct ResiliencePolicy {
    pub rate_limit_enabled: bool,
    pub requests_per_second: f32,
    pub burst_size: f32,
    pub request_timeout_ms: u64,
    pub stream_start_timeout_ms: u64,
    pub stream_idle_timeout_ms: u64,
    pub retries_enabled: bool,
    pub retry_max_attempts: u32,
    pub retry_base_delay_ms: u64,
    pub retry_backoff_multiplier: f32,
    pub retry_max_delay_ms: u64,
    pub retry_jitter_mode: String,
    pub retry_respect_retry_after: bool,
    pub retryable_http_statuses: Vec<u16>,
    pub retryable_transport_errors: bool,
    pub retry_budget_ms: u64,
}

impl Default for ResiliencePolicy {
    fn default() -> Self {
        Self {
            rate_limit_enabled: true,
            requests_per_second: 10.0,
            burst_size: 20.0,
            request_timeout_ms: 30_000,
            stream_start_timeout_ms: 15_000,
            stream_idle_timeout_ms: 30_000,
            retries_enabled: true,
            retry_max_attempts: 3,
            retry_base_delay_ms: 1_000,
            retry_backoff_multiplier: 2.0,
            retry_max_delay_ms: 10_000,
            retry_jitter_mode: "full".to_string(),
            retry_respect_retry_after: true,
            retryable_http_statuses: vec![408, 425, 429, 500, 502, 503, 504],
            retryable_transport_errors: true,
            retry_budget_ms: 20_000,
        }
    }
}

impl From<&ResilienceConfig> for ResiliencePolicy {
    fn from(value: &ResilienceConfig) -> Self {
        Self {
            rate_limit_enabled: value.rate_limit_enabled,
            requests_per_second: value.requests_per_second,
            burst_size: value.burst_size,
            request_timeout_ms: value.request_timeout_ms,
            stream_start_timeout_ms: value.stream_start_timeout_ms,
            stream_idle_timeout_ms: value.stream_idle_timeout_ms,
            retries_enabled: value.retries_enabled,
            retry_max_attempts: value.retry_max_attempts,
            retry_base_delay_ms: value.retry_base_delay_ms,
            retry_backoff_multiplier: value.retry_backoff_multiplier,
            retry_max_delay_ms: value.retry_max_delay_ms,
            retry_jitter_mode: value.retry_jitter_mode.clone(),
            retry_respect_retry_after: value.retry_respect_retry_after,
            retryable_http_statuses: value.retryable_http_statuses.clone(),
            retryable_transport_errors: value.retryable_transport_errors,
            retry_budget_ms: value.retry_budget_ms,
        }
    }
}

impl ResiliencePolicy {
    /// Build the retry schedule consumed by the model-call boundary.
    #[must_use]
    pub fn retry_backoff_builder(&self) -> RetryBackoffBuilder {
        RetryBackoffBuilder {
            min_delay: Duration::from_millis(self.retry_base_delay_ms),
            max_delay: Duration::from_millis(self.retry_max_delay_ms),
            factor: self.retry_backoff_multiplier,
            max_times: self.retry_max_attempts.saturating_sub(1) as usize,
            total_delay: Duration::from_millis(self.retry_budget_ms),
            jitter_mode: RetryJitterMode::from_config(&self.retry_jitter_mode),
            jitter_seed: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.requests_per_second < 0.1 {
            return Err(anyhow!("requests_per_second must be >= 0.1"));
        }
        if self.burst_size < 1.0 {
            return Err(anyhow!("burst_size must be >= 1"));
        }
        if self.request_timeout_ms < 1_000 {
            return Err(anyhow!("request_timeout_ms must be >= 1000"));
        }
        if self.stream_start_timeout_ms < 1_000 {
            return Err(anyhow!("stream_start_timeout_ms must be >= 1000"));
        }
        if self.stream_idle_timeout_ms < 1_000 {
            return Err(anyhow!("stream_idle_timeout_ms must be >= 1000"));
        }
        if self.retry_max_attempts > 10 {
            return Err(anyhow!("retry_max_attempts must be <= 10"));
        }
        if self.retry_base_delay_ms < 100 {
            return Err(anyhow!("retry_base_delay_ms must be >= 100"));
        }
        if self.retry_backoff_multiplier < 1.1 || self.retry_backoff_multiplier > 5.0 {
            return Err(anyhow!("retry_backoff_multiplier must be in [1.1, 5.0]"));
        }
        if self.retry_max_delay_ms < 100 {
            return Err(anyhow!("retry_max_delay_ms must be >= 100"));
        }
        match self.retry_jitter_mode.as_str() {
            "none" | "full" | "equal" | "decorrelated" => {}
            _ => return Err(anyhow!("retry_jitter_mode has an invalid value")),
        }
        if self.retryable_http_statuses.is_empty() {
            return Err(anyhow!("retryable_http_statuses must not be empty"));
        }
        if self
            .retryable_http_statuses
            .iter()
            .any(|status| *status < 100 || *status > 599)
        {
            return Err(anyhow!(
                "retryable_http_statuses entries must be within [100, 599]"
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetryJitterMode {
    None,
    Full,
    Equal,
    Decorrelated,
}

impl RetryJitterMode {
    fn from_config(value: &str) -> Self {
        match value {
            "full" => Self::Full,
            "equal" => Self::Equal,
            "decorrelated" => Self::Decorrelated,
            _ => Self::None,
        }
    }
}

/// Backoff builder derived entirely from one immutable resilience policy.
#[derive(Debug, Clone, Copy)]
pub struct RetryBackoffBuilder {
    min_delay: Duration,
    max_delay: Duration,
    factor: f32,
    max_times: usize,
    total_delay: Duration,
    jitter_mode: RetryJitterMode,
    jitter_seed: Option<u64>,
}

impl RetryBackoffBuilder {
    /// Use a deterministic entropy stream for reproducible schedule tests.
    #[must_use]
    pub const fn with_jitter_seed(mut self, seed: u64) -> Self {
        self.jitter_seed = Some(seed);
        self
    }
}

impl BackoffBuilder for RetryBackoffBuilder {
    type Backoff = RetryBackoff;

    fn build(self) -> Self::Backoff {
        let base = ExponentialBuilder::new()
            .with_factor(self.factor)
            .with_min_delay(self.min_delay)
            .with_max_delay(self.max_delay)
            .with_max_times(self.max_times)
            .with_total_delay(Some(self.total_delay))
            .build();
        let entropy_builder = ExponentialBuilder::new()
            .with_jitter()
            .with_factor(self.factor)
            .with_min_delay(self.min_delay)
            .with_max_delay(self.max_delay)
            .with_max_times(self.max_times);
        let entropy = self.jitter_seed.map_or_else(
            || entropy_builder.build(),
            |seed| entropy_builder.with_jitter_seed(seed).build(),
        );

        RetryBackoff {
            base,
            entropy,
            min_delay: self.min_delay,
            max_delay: self.max_delay,
            total_delay: self.total_delay,
            cumulative_delay: Duration::ZERO,
            previous_delay: self.min_delay,
            jitter_mode: self.jitter_mode,
        }
    }
}

/// Iterator of bounded retry delays.
#[derive(Debug)]
pub struct RetryBackoff {
    base: ExponentialBackoff,
    entropy: ExponentialBackoff,
    min_delay: Duration,
    max_delay: Duration,
    total_delay: Duration,
    cumulative_delay: Duration,
    previous_delay: Duration,
    jitter_mode: RetryJitterMode,
}

impl Iterator for RetryBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let base = self.base.next()?;
        let entropy = self.entropy.next().unwrap_or(base);
        let random_part = entropy.saturating_sub(base);
        let delay = match self.jitter_mode {
            RetryJitterMode::None => base,
            RetryJitterMode::Full => nonzero_duration(random_part.min(base)),
            RetryJitterMode::Equal => base / 2 + random_part.min(base) / 2,
            RetryJitterMode::Decorrelated => {
                let upper = self.previous_delay.saturating_mul(3).min(self.max_delay);
                sample_duration(self.min_delay.min(upper), upper, random_part, base)
            }
        }
        .min(self.max_delay);

        if self.cumulative_delay.saturating_add(delay) > self.total_delay {
            return None;
        }
        self.cumulative_delay = self.cumulative_delay.saturating_add(delay);
        self.previous_delay = delay;
        Some(delay)
    }
}

fn nonzero_duration(value: Duration) -> Duration {
    if value.is_zero() {
        Duration::from_nanos(1)
    } else {
        value
    }
}

fn sample_duration(
    lower: Duration,
    upper: Duration,
    random_part: Duration,
    base: Duration,
) -> Duration {
    let lower_nanos = lower.as_nanos();
    let span_nanos = upper.saturating_sub(lower).as_nanos();
    let denominator = base.as_nanos().max(1);
    let sampled_nanos =
        lower_nanos.saturating_add(span_nanos.saturating_mul(random_part.as_nanos()) / denominator);
    Duration::from_nanos(u64::try_from(sampled_nanos).unwrap_or(u64::MAX))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicySource {
    Global,
    AgentOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResilienceOverrideMode {
    #[default]
    Inherit,
    Override,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AgentResilienceOverride {
    #[serde(default)]
    pub mode: ResilienceOverrideMode,
    pub rate_limit_enabled: Option<bool>,
    pub requests_per_second: Option<f32>,
    pub burst_size: Option<f32>,
    pub request_timeout_ms: Option<u64>,
    pub stream_start_timeout_ms: Option<u64>,
    pub stream_idle_timeout_ms: Option<u64>,
    pub retries_enabled: Option<bool>,
    pub retry_max_attempts: Option<u32>,
    pub retry_base_delay_ms: Option<u64>,
    pub retry_backoff_multiplier: Option<f32>,
    pub retry_max_delay_ms: Option<u64>,
    pub retry_jitter_mode: Option<String>,
    pub retry_respect_retry_after: Option<bool>,
    pub retryable_http_statuses: Option<Vec<u16>>,
    pub retryable_transport_errors: Option<bool>,
    pub retry_budget_ms: Option<u64>,
}

pub fn resolve_effective_policy(
    global: &ResiliencePolicy,
    agent_settings_value: Option<&Value>,
) -> Result<(ResiliencePolicy, PolicySource)> {
    let Some(agent_settings) = agent_settings_value else {
        return Ok((global.clone(), PolicySource::Global));
    };
    let Some(resilience) = agent_settings.get("resilience") else {
        return Ok((global.clone(), PolicySource::Global));
    };
    if !resilience.is_object() {
        return Ok((global.clone(), PolicySource::Global));
    }

    let override_cfg: AgentResilienceOverride = serde_json::from_value(resilience.clone())?;
    if override_cfg.mode == ResilienceOverrideMode::Inherit {
        return Ok((global.clone(), PolicySource::Global));
    }

    let mut merged = global.clone();
    if let Some(v) = override_cfg.rate_limit_enabled {
        merged.rate_limit_enabled = v;
    }
    if let Some(v) = override_cfg.requests_per_second {
        merged.requests_per_second = v;
    }
    if let Some(v) = override_cfg.burst_size {
        merged.burst_size = v;
    }
    if let Some(v) = override_cfg.request_timeout_ms {
        merged.request_timeout_ms = v;
    }
    if let Some(v) = override_cfg.stream_start_timeout_ms {
        merged.stream_start_timeout_ms = v;
    }
    if let Some(v) = override_cfg.stream_idle_timeout_ms {
        merged.stream_idle_timeout_ms = v;
    }
    if let Some(v) = override_cfg.retries_enabled {
        merged.retries_enabled = v;
    }
    if let Some(v) = override_cfg.retry_max_attempts {
        merged.retry_max_attempts = v;
    }
    if let Some(v) = override_cfg.retry_base_delay_ms {
        merged.retry_base_delay_ms = v;
    }
    if let Some(v) = override_cfg.retry_backoff_multiplier {
        merged.retry_backoff_multiplier = v;
    }
    if let Some(v) = override_cfg.retry_max_delay_ms {
        merged.retry_max_delay_ms = v;
    }
    if let Some(v) = override_cfg.retry_jitter_mode {
        merged.retry_jitter_mode = v;
    }
    if let Some(v) = override_cfg.retry_respect_retry_after {
        merged.retry_respect_retry_after = v;
    }
    if let Some(v) = override_cfg.retryable_http_statuses {
        merged.retryable_http_statuses = v;
    }
    if let Some(v) = override_cfg.retryable_transport_errors {
        merged.retryable_transport_errors = v;
    }
    if let Some(v) = override_cfg.retry_budget_ms {
        merged.retry_budget_ms = v;
    }

    merged.validate()?;
    Ok((merged, PolicySource::AgentOverride))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolves_global_policy_when_agent_inherits() {
        let global = ResiliencePolicy::default();
        let agent = json!({
            "resilience": { "mode": "inherit", "retry_max_attempts": 9 }
        });

        let (effective, source) = resolve_effective_policy(&global, Some(&agent)).unwrap();
        assert_eq!(source, PolicySource::Global);
        assert_eq!(effective, global);
    }

    #[test]
    fn resolves_overridden_policy_for_agent() {
        let global = ResiliencePolicy::default();
        let agent = json!({
            "resilience": {
                "mode": "override",
                "retry_max_attempts": 5,
                "request_timeout_ms": 45000
            }
        });

        let (effective, source) = resolve_effective_policy(&global, Some(&agent)).unwrap();
        assert_eq!(source, PolicySource::AgentOverride);
        assert_eq!(effective.retry_max_attempts, 5);
        assert_eq!(effective.request_timeout_ms, 45_000);
        assert_eq!(
            effective.retryable_http_statuses,
            global.retryable_http_statuses
        );
    }

    #[test]
    fn rejects_invalid_retryable_statuses() {
        let global = ResiliencePolicy::default();
        let agent = json!({
            "resilience": {
                "mode": "override",
                "retryable_http_statuses": [99, 700]
            }
        });

        let err = resolve_effective_policy(&global, Some(&agent)).unwrap_err();
        assert!(
            err.to_string().contains("retryable_http_statuses"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn rejects_invalid_jitter_mode() {
        let global = ResiliencePolicy::default();
        let agent = json!({
            "resilience": {
                "mode": "override",
                "retry_jitter_mode": "burst"
            }
        });

        let err = resolve_effective_policy(&global, Some(&agent)).unwrap_err();
        assert!(
            err.to_string().contains("retry_jitter_mode"),
            "unexpected error: {err}"
        );
    }
}
