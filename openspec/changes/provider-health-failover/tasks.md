## 1. Health tracking

- [x] 1.1 `ProviderHealthMonitor` (`src/llm/health.rs`): consecutive-failure
      cooldown tracking, `is_available`, `snapshot`, periodic sweep loop.
- [x] 1.2 Unit tests: unknown-provider-available, threshold trips cooldown,
      success clears cooldown, snapshot reports cooldown remaining.

## 2. Wire into routing

- [x] 2.1 `ProviderRegistry` owns the shared monitor; `resolve_to_llm_config`
      skips providers in cooldown.
- [x] 2.2 `ModelRouter::route` filters unhealthy providers before the
      cost/context/benchmark sort.

## 3. Wire into failover

- [x] 3.1 `Orchestrator::with_health_monitor` + `build_fallback_driver`
      helper (reuses the base `LlmConfig`, overrides model/api_key/base_url
      from `FallbackModel`).
- [x] 3.2 Every primary/fallback driver success/failure records against the
      monitor from within the existing tool-loop retry block
      (`src/llm/orchestrator.rs`).
- [x] 3.3 `RunManager::with_failover_config` + per-run wiring: attaches a
      fallback driver from `fallback_models.first()` when enabled, health
      monitor unconditionally when a provider registry is present.

## 4. Background loop + surfacing

- [x] 4.1 `src/server.rs` spawns the health-sweep loop using
      `health_check_secs` (default 30s), sharing the run-cancellation token.
- [x] 4.2 `uar_provider_health` gauge metric.
- [x] 4.3 `GET /api/uar/providers/health` endpoint.

## 5. Verify

- [x] 5.1 `cargo check --lib` green.
- [x] 5.2 `cargo test --lib` — 318/318 lib tests green (full-suite batch
      checkpoint covering this + CH-04/CH-06/CH-08 together).
