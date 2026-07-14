## Why

Change 4 shipped `UarError` and a minimal `IntoResponse` implementation that captures `SpanTrace` at render time and logs it. However, the release-readiness assessment identified **382 `unwrap()`/`expect()` calls** on production hot paths and only sporadic `tracing::span!` coverage at the API boundary. Without a deliberate sweep, errors lack request identifiers and panics can still escape from production code.

The operator's 2026-07-13 analysis selected `tracing-error 0.2` for span chains and `sentry-sdk` behind a feature flag. Change 5 finishes the observability-error integration started in Change 4: it drives down hot-path `unwrap()`/`expect()`, enforces the reduction with `clippy.toml`, and completes the tracing/sentry wiring deferred from Change 4.

## What Changes

- Add `clippy.toml` with `#![deny(clippy::unwrap_used, clippy::expect_used)]` scoped to `src/uar/api/`, `src/uar/server.rs`, and `src/uar/runtime/`.
- Audit `src/uar/` and eliminate production hot-path `unwrap()`/`expect()`, targeting **382 → < 50** remaining.
- Complete the `tracing-error` integration: ensure every route handler enters a `tracing::span!` carrying `request_id`, `agent_id`, and `run_id` so `SpanTrace` captured by `UarError::into_response` is non-empty.
- Wire Sentry reporting into `release.yml` when built with `--features sentry`; keep the feature off by default.
- Add `docs/observability.md` describing the tracing/sentry setup and the error-code surface for operators.
- Run full-workspace `cargo fmt --all -- --check` and `cargo clippy --features server-full --no-deps` as part of this change (deferred from Change 4).

## Capabilities

### New Capabilities

- `uar-observability-errors`: clippy-guarded hot-path error handling, span propagation through `UarError`, and Sentry integration in release builds.

## Impact

- **No public API break.** `UarError` shape is unchanged; this change only improves how errors are captured and reported.
- **Panics on hot paths are drastically reduced.** Remaining `unwrap()`/`expect()` are either init-time, test-only, or explicitly annotated with `#[expect]`.
- **Production traces are actionable.** Every error logged through `UarError` carries `request_id`, `agent_id`, `run_id` via the current span.
- **Sentry is build-time gated.** The `sentry` feature must be enabled at compile time; default builds are unaffected.
- **CI is stricter.** `clippy.toml` means future PRs introducing `unwrap()`/`expect()` in the scoped paths will fail checks.

## Out of scope

- Removing **every** `unwrap()`/`expect()` in the repository. Some are in CLI/init code or tests and are not production hot-path risks.
- Configuring a real Sentry DSN or project. The feature flag and release wiring are in place; operator-provided DSN setup is separate.
- Replacing `anyhow!()` calls deeper than `src/uar/api/` trait boundaries. That broader sweep is tracked as candidate future work from Change 4.
