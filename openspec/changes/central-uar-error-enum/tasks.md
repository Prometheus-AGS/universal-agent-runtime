## 1. Design the central enum
- [x] 1.1 Define `pub enum UarError` in `src/uar/error.rs` with `#[non_exhaustive]`.
- [x] 1.2 Variants: `Config { code, message }`, `Auth { code, message }`, `Rag { code, message }`, `Memory { code, message }`, `Mcp { code, message }`, `A2a { code, message }`, `Llm { code, message }`, `Internal(#[from] anyhow::Error)`. Struct-variant `{code, message}` shape used instead of `Config(ConfigError)` etc. — see task 2 for why.
- [x] 1.3 Implement `Display`/`Error` via `#[derive(thiserror::Error)]`, and `code(&self) -> &'static str`.
- [x] 1.4 Define `pub type Result<T> = std::result::Result<T, UarError>;` in `src/uar/error.rs`, re-exported at crate root (`pub use uar::error::{Result, UarError};` in `src/lib.rs`) so `crate::Result<T>` works everywhere.

## 2. Wrap existing module errors
- [x] 2.1-2.7 **Scope correction**: audited every listed module (`settings`, `memory`, `rag`, `governance`, `api::a2a`, `llm`, `mcp`) — only `src/uar/compiler/error.rs` has a pre-existing dedicated typed `*Error` enum (`CompileError`); the rest use ad hoc `anyhow::Error`/`String`/`(StatusCode, String)` with no typed error type to wrap. The proposal's assumption that these all already have `*Error` enums to wrap was incorrect. Shipped the `{code, message}` struct-variant shape instead (task 1.2), which each domain's call sites can adopt without requiring a typed error to exist first. Migrating individual submodules to dedicated typed errors is follow-up work, disclosed in the spec's "Variants grouped by domain" adoption note.
- [x] 2.8 `UarError::Internal(#[from] anyhow::Error)` implemented for cross-cutting/unclassified failures; composes with `?` at any call site already returning `anyhow::Result`.

## 3. Public-API surface
- [x] 3.1 `impl IntoResponse for UarError` implemented in `src/uar/error.rs` (co-located with the type, not `api/mod.rs` — more conventional Rust module layout; `api/mod.rs` is a thin router-assembly file with no existing error-type home).
- [x] 3.2 `status_code()` maps `Auth` → 401, all other domain variants → 400, `Internal` → 500.
- [x] 3.3 `code()` string included in the JSON error response body.
- [x] 3.4 `message` included in the JSON error response body.
- [x] 3.5 Full error + `SpanTrace` logged via `tracing::error!` server-side; public response body carries only `{code, message}`.

## 4. Convert public-API anyhow!() to UarError
- [ ] 4.1-4.2 **Scope correction, not done this pass**: audited actual `anyhow!()` distribution — 127 total in `src/uar/` (not the proposal's estimated 130 "in public-API boundary code"). Only 8 are in `src/uar/api/` at all (`knowledge.rs`, `a2a/client.rs`, `a2a/registry.rs`), and every one of those 8 is inside an internal trait-implementation method (`RetrievalBackend::search_one`, the A2A outbound HTTP client, `InMemoryAgentRegistry`) several layers removed from an axum handler's return type — converting them requires changing those traits' signatures (`RetrievalBackend`, `AgentRegistry`), which is a wider-blast-radius change than this proposal's stated scope. Deferred to follow-up work; disclosed in the spec's "thiserror for the central enum" scope note with the corrected counts.
- [x] 4.3 Application code (`main.rs`, CLI) untouched — still `anyhow::Result<()>` per the thiserror-vs-anyhow rule (no change needed since nothing there was touched).

## 5. Tracing-error integration
- [x] 5.1 Added `tracing-error = "0.2"` to `Cargo.toml` (resolved to 0.2.1).
- [x] 5.2 Implemented via `tracing_error::SpanTrace::capture()` inside `IntoResponse::into_response`, logged alongside the error — simpler and more idiomatic than a `From<SpanTrace> for UarError` conversion, and matches the spec's "captured and logged, not carried in the type" requirement.
- [ ] 5.3 **Deferred**: auditing/adding `tracing::span!` to every route handler so span traces are always non-empty is a broad, low-risk-but-wide sweep across all of `src/uar/api/`; out of this change's bounded scope, tracked as follow-up (existing request-level tracing spans already exist via middleware in `src/server.rs` in most cases, so this is a completeness pass, not a from-scratch build).

## 6. Sentry integration (feature-flagged)
- [x] 6.1 Added `sentry = { version = "0.48", optional = true, default-features = false, features = [...] }` (started at the proposal's placeholder `0.x`, verified 0.34 was outdated vs. the resolver's own "available: v0.48.4" notice, bumped to 0.48 — confirmed both with/without the feature compile clean).
- [x] 6.2 Sentry reporting gated on `#[cfg(feature = "sentry")]` inside `IntoResponse::into_response` (`sentry::capture_error(&self)`).
- [ ] 6.3 **Deferred**: adding `--features sentry` to `release.yml`'s build job is a release-pipeline change; out of scope for this implementation-focused change, and premature before an actual Sentry DSN/project exists (see 6.4).
- [ ] 6.4 **Deferred**: `docs/observability.md` Sentry setup doc — DSN/project configuration is explicitly operator work per this change's own proposal ("Out of scope: The Sentry dashboard configuration"); documenting a setup process for infrastructure that doesn't exist yet would be premature.

## 7. Verification
- [x] 7.1 Unit tests for every variant's `code()` and `Display` (`src/uar/error.rs`'s `#[cfg(test)] mod tests`) — one test per domain variant plus `Internal`.
- [x] 7.2 Integration-style test confirming the JSON body shape: `into_response_renders_the_documented_json_shape` calls `UarError::config(...).into_response()` directly and asserts the JSON body via axum's `to_bytes` — validates the documented `{code, message}` shape without requiring the error type to be wired into a live route (per task 3's design note, existing handlers are unaffected/optional adopters).
- [x] 7.3 `cargo test --locked --no-default-features --features server-full --lib uar::error::` green (9/9 tests pass).
- [ ] 7.4 **Deferred to the phase's consolidated validation pass**: full-workspace `cargo fmt --all -- --check` and `cargo clippy --features server-full --no-deps`, per the KBD implementation-first policy (static inspection + `cargo check` during implementation). `cargo check --no-default-features --features server-full` and `--features server-full,sentry` both verified clean this pass.
