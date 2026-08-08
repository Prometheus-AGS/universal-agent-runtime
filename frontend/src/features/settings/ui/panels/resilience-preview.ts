export const RESILIENCE_RECOMMENDED_DEFAULTS = {
  rate_limit_enabled: true,
  requests_per_second: 10,
  burst_size: 20,
  request_timeout_ms: 30000,
  stream_start_timeout_ms: 15000,
  retries_enabled: true,
  retry_max_attempts: 3,
  retry_base_delay_ms: 1000,
  retry_backoff_multiplier: 2,
  retry_max_delay_ms: 10000,
  retry_jitter_mode: "full",
  retry_respect_retry_after: true,
  retryable_http_statuses: [408, 425, 429, 500, 502, 503, 504],
  retryable_transport_errors: true,
  retry_budget_ms: 20000,
} as const;

export function buildGlobalResiliencePreview(
  values: Record<string, unknown>,
): Record<string, unknown> {
  const global = { ...RESILIENCE_RECOMMENDED_DEFAULTS } as Record<
    string,
    unknown
  >;
  Object.keys(global).forEach((key) => {
    const candidate = values[`resilience.${key}`];
    if (candidate !== undefined) {
      global[key] = candidate;
    }
  });
  return global;
}

export function mergeAgentResiliencePreview(
  global: Record<string, unknown>,
  override: Record<string, unknown>,
): Record<string, unknown> {
  if ((override.mode as string) !== "override") {
    return global;
  }
  const merged = { ...global };
  Object.entries(override).forEach(([key, value]) => {
    if (key === "mode") return;
    if (value !== undefined && value !== null) {
      merged[key] = value;
    }
  });
  return merged;
}
