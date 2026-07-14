## Why

UAR has 130 `anyhow!()` calls in `src/uar/` and 382 `unwrap()/expect()`
calls — many on production hot paths — with no central error
taxonomy. Each public route in `src/uar/api/*` returns a different
error shape (some use `(StatusCode, String)`, some `ApiError`,
some `AppError`). The 2026-07-13 release-readiness assessment
flagged this as **the largest single error-handling gap**; the
SDK work in `sdk-{rust,python,typescript}-1.0` cannot land
typed-error support without a central `UarError` to mirror.

The operator's 2026-07-13 analysis selected **`thiserror 2.0` for
library code**, **`anyhow 1.0` for application code only**, plus
`error-stack` for context attachment and `tracing-error` for
span chains. The `sentry-sdk` integration is behind a feature
flag (default off).

## What Changes

- New `src/uar/error.rs` with `pub enum UarError` (#[non_exhaustive]).
- Variants grouped by domain: `Config`, `Auth`, `Rag`, `Memory`,
  `Mcp`, `A2a`, `Llm`, `Internal`, with stable error codes for
  each leaf variant (e.g. `E_CONFIG_MISSING_FIELD`).
- Every existing public `*Error` enum in each submodule wrapped
  as a variant.
- 130 `anyhow!()` in public-API boundary code converted to
  `UarError` variants.
- `tracing-error` integrated so every `UarError` carries the
  current span trace (request_id, agent_id, run_id).
- `sentry-sdk` integration behind `--features sentry`; default off.
- Stable error codes (string constants) consumable by the SDKs
  in subsequent changes.

## Capabilities

### New Capabilities

- `uar-error-model`: the central `UarError` enum + stable error
  codes + tracing-error integration.

## Impact

- **API surface change:** HTTP error responses change shape.
  Backward-compat shim: every existing error code (string) MUST
  be preserved as a stable string in the new `UarError::code()`
  method.
- **All public routes** in `src/uar/api/*` MUST use a single
  `IntoResponse` impl for `UarError` to consolidate the error
  surface.
- **130 `anyhow!()` in public-API boundary code** converted to
  `UarError` variants. The remaining `anyhow!()` calls in
  `main.rs` / CLI code are unaffected (application code, per the
  thiserror-vs-anyhow rule).
- **Tracing context** now flows through errors; observability
  tooling can use the span trace for root-cause analysis.
- **No dependency removal.** `anyhow` and `thiserror` are
  already in the dependency tree; `error-stack` and
  `tracing-error` are new additions; `sentry-sdk` is behind a
  feature flag.

## Out of scope

- **Reducing 382 `unwrap()/expect()` to < 50.** Tracked as the
  separate change `unwrap-sweep-tracing-error-chains` in the same
  Order 3 of the grade-A plan.
- **Removing the `(StatusCode, String)` error shape in routes.**
  This change introduces the new shape; the cleanup of legacy
  shapes is a separate refactor in the SDK-coordination change.
- **The Sentry dashboard configuration.** The feature flag is
  wired; the actual Sentry project + DSN setup is operator work.
