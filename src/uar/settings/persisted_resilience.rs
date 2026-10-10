//! Resolve the persisted next-turn resilience policy for every runtime entrypoint.

use super::SettingsManager;
use super::resilience_policy::{PolicySource, ResiliencePolicy, resolve_effective_policy};

async fn load_global_resilience_policy(
    baseline: &ResiliencePolicy,
    settings: Option<&SettingsManager>,
) -> ResiliencePolicy {
    let mut policy = baseline.clone();

    let Some(mgr) = settings else {
        return policy;
    };

    macro_rules! apply_typed {
        ($key:literal, $ty:ty, $field:ident) => {
            if let Ok(Some(v)) = mgr.get_typed::<$ty>($key).await {
                policy.$field = v;
            }
        };
    }

    apply_typed!("resilience.rate_limit_enabled", bool, rate_limit_enabled);
    apply_typed!("resilience.requests_per_second", f32, requests_per_second);
    apply_typed!("resilience.burst_size", f32, burst_size);
    apply_typed!("resilience.request_timeout_ms", u64, request_timeout_ms);
    apply_typed!(
        "resilience.stream_start_timeout_ms",
        u64,
        stream_start_timeout_ms
    );
    apply_typed!(
        "resilience.stream_idle_timeout_ms",
        u64,
        stream_idle_timeout_ms
    );
    apply_typed!("resilience.retries_enabled", bool, retries_enabled);
    apply_typed!("resilience.retry_max_attempts", u32, retry_max_attempts);
    apply_typed!("resilience.retry_base_delay_ms", u64, retry_base_delay_ms);
    apply_typed!(
        "resilience.retry_backoff_multiplier",
        f32,
        retry_backoff_multiplier
    );
    apply_typed!("resilience.retry_max_delay_ms", u64, retry_max_delay_ms);
    apply_typed!("resilience.retry_jitter_mode", String, retry_jitter_mode);
    apply_typed!(
        "resilience.retry_respect_retry_after",
        bool,
        retry_respect_retry_after
    );
    apply_typed!(
        "resilience.retryable_http_statuses",
        Vec<u16>,
        retryable_http_statuses
    );
    apply_typed!(
        "resilience.retryable_transport_errors",
        bool,
        retryable_transport_errors
    );
    apply_typed!("resilience.retry_budget_ms", u64, retry_budget_ms);

    if let Err(err) = policy.validate() {
        tracing::warn!(
            error = %err,
            "Invalid global resilience settings detected; falling back to config defaults"
        );
        return baseline.clone();
    }

    policy
}

pub(crate) async fn resolve_persisted_resilience_policy(
    baseline: &ResiliencePolicy,
    settings: Option<&SettingsManager>,
    agent_id: &str,
) -> (ResiliencePolicy, PolicySource) {
    let global = load_global_resilience_policy(baseline, settings).await;
    let Some(mgr) = settings else {
        return (global, PolicySource::Global);
    };

    let mut lookup_keys = vec![format!("agent_config.{agent_id}")];
    if agent_id == "default-agent" {
        lookup_keys.push("agent_config.orchestrated".to_string());
    }

    for key in lookup_keys {
        if let Some(agent_cfg) = mgr.get_value(&key).await {
            match resolve_effective_policy(&global, Some(&agent_cfg)) {
                Ok(resolved) => return resolved,
                Err(err) => {
                    tracing::warn!(
                        setting_key = %key,
                        error = %err,
                        "Invalid per-agent resilience override; using global policy"
                    );
                }
            }
        }
    }

    (global, PolicySource::Global)
}

