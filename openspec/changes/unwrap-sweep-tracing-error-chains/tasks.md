## 1. Hot-path unwrap/expect audit
- [x] 1.1 Count current `unwrap()`/`expect()` in `src/uar/` (baseline ~382) and categorize by file.
- [x] 1.2 Identify production hot paths: `src/uar/api/`, `src/uar/server.rs`, `src/uar/runtime/`, and any `tokio::spawn` or request-handler entry points.
- [x] 1.3 Refactor hot-path call sites to `match`, `if let`, or `UarError`/`anyhow` propagation.
- [x] 1.4 Annotate legitimate init-time/test-only call sites with `#[expect(clippy::unwrap_used)]` or `#[expect(clippy::expect_used)]` and a one-line reason.
- [x] 1.5 Verify remaining count is `< 50` in production hot paths; record final count in `tasks.md`.
  - Final production hot-path count: 0 (src/uar/api/ 0, src/uar/server.rs 0, src/uar/runtime/ 0). Total remaining in target scopes: 92 (tests + init-time annotated).

## 2. Clippy guard
- [x] 2.1 Added `#![deny(clippy::unwrap_used, clippy::expect_used)]` to `src/uar/api/mod.rs`, `src/uar/runtime/mod.rs`, and `src/server.rs`.
- [x] 2.2 N/A — module-level attributes satisfy the scoped guard without a workspace `clippy.toml`.
- [x] 2.3 The existing `.github/workflows/ci.yml` already runs clippy; the new deny attributes make the scoped paths fail CI if new unwrap/expect are introduced.

## 3. Tracing-error completeness
- [x] 3.1 Added `request_span_layer` middleware in `src/server.rs` that enters a `tracing` span carrying `request_id`, `agent_id`, and `run_id` for every request.
- [x] 3.2 Middleware is applied as the outermost layer so the span is active for the entire request lifecycle.
- [x] 3.3 Added unit test `into_response_captures_span_trace_with_request_ids` in `src/uar/error.rs`.
- [x] 3.4 Verified `cargo test --locked --no-default-features --features server-full --lib uar::error::` passes (10/10 tests).

## 4. Sentry release wiring
- [x] 4.1 Added `Build Sentry-enabled release bundle` step to `.github/workflows/release.yml` that passes `--features server-full,sentry`.
- [x] 4.2 Documented the feature flag in `docs/observability.md` and in `Cargo.toml` feature comment.
- [x] 4.3 Verified `cargo check --locked --no-default-features --features server-full,sentry` compiles clean.

## 5. Documentation
- [x] 5.1 Created `docs/observability.md` with span structure, `SpanTrace` capture, Sentry enablement, and clippy policy.
- [x] 5.2 Updated `TESTING.md` with an error-handling/observability section and a quality-gate entry.

## 6. Final validation
- [x] 6.1 `cargo fmt --all -- --check` passed.
- [x] 6.2 `cargo clippy --no-default-features --features server-full --no-deps` has no unwrap/expect lints in target scopes (pre-existing unrelated warnings remain outside scope).
- [x] 6.3 `cargo test --locked --no-default-features --features server-full --lib uar::error::` passed (10/10).
- [x] 6.4 Run `openspec validate --strict` and confirm the new change is valid.
- [ ] 6.5 Mark Change 5 implementation complete in `progress.json` and update `current-waypoint.json`.
