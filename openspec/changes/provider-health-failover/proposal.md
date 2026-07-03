# CH-03 provider-health-failover

## Why

The router (CH-09) already scores candidates by cost/context/benchmark, and
`FailoverConfig`/`with_failover` existed on `Orchestrator` — but nothing ever
called `with_failover`, and neither `ModelRouter::route` nor
`ProviderRegistry::resolve_to_llm_config` had any notion of provider health.
A provider having a bad day (timeouts, 5xx) kept winning routing ties against
healthy alternatives, and the previously-dead `health_check_secs` /
`cooldown_secs` / `error_threshold` config knobs did nothing.

## What changed

- `ProviderHealthMonitor` (`src/llm/health.rs`, new): per-provider
  consecutive-failure tracking. `error_threshold` consecutive failures put a
  provider into cooldown for `cooldown_secs`; `is_available` reports whether
  a provider is currently outside its cooldown window.
- `ProviderRegistry` now owns a shared `Arc<ProviderHealthMonitor>`
  (`health()` accessor). `resolve_to_llm_config` skips providers currently in
  cooldown, same as it already skipped disabled providers.
- `ModelRouter::route` filters candidates to healthy providers before the
  existing cost/context/benchmark sort — an unhealthy provider is excluded
  entirely, not just deprioritized.
- `Orchestrator` gained `with_health_monitor` and a `build_fallback_driver`
  helper; every primary/fallback driver success or failure records against
  the shared monitor, so a failure observed on one run affects routing for
  the very next request.
- `RunManager` gained `failover_config` + `with_failover_config`; the per-run
  `Orchestrator::new` call site attaches a fallback driver (built from
  `fallback_models.first()`, i.e. `FailoverStrategy::Priority`) when
  `failover.enabled`, plus the health monitor unconditionally when a
  provider registry is configured.
- `src/server.rs` spawns a periodic health-sweep loop
  (`ProviderHealthMonitor::spawn_monitor_loop`) using `health_check_secs`
  (default 30s), sharing the existing run-cancellation shutdown token.
- New `uar_provider_health` gauge metric and `GET
  /api/uar/providers/health` endpoint for Runtime Console surfacing.

## Scope notes

- Only `FailoverStrategy::Priority` (first configured fallback) is wired;
  `RoundRobin`/`CostOptimized` remain declared-but-unused enum variants —
  no deployment currently sets them, and multi-fallback chains are a
  reasonable follow-up rather than a blocker for this pass.
- Health is purely reactive (driven by observed call failures), not an
  active probe hitting each provider's health-check endpoint on a timer —
  the periodic loop only sweeps/expires existing cooldown state and refreshes
  the metric. An active-probe mode is a natural follow-up if reactive-only
  proves too slow to detect an outage with low traffic.
