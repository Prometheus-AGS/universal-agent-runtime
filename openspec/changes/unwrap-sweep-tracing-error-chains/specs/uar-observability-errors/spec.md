# UAR observability and error chains

## Purpose

Complete the error-observability work started in Change 4 by enforcing a hard floor on production hot-path panics, ensuring every `UarError` carries the active tracing span context, and wiring Sentry reporting into release builds.

## ADDED Requirements

### Requirement: unwrap/expect ban on production hot paths
The following paths MUST be free of `unwrap()` and `expect()` except for call sites annotated with an explicit `#[expect]`:

- `src/uar/api/`
- `src/uar/server.rs`
- `src/uar/runtime/`

CI MUST fail if a new `unwrap()` or `expect()` is introduced in these paths without an `#[expect]` annotation.

#### Scenario: A new handler is added
- **WHEN** a new route handler is added under `src/uar/api/`
- **THEN** it MUST not contain `unwrap()` or `expect()` unless the call site is init-time or test-only and annotated with `#[expect]` and a reason
- **AND** `cargo clippy --features server-full --no-deps` MUST pass

### Requirement: Panic reduction target
Across all production hot paths in `src/uar/`, the count of `unwrap()`/`expect()` call sites MUST be reduced from the Change 4 baseline (~382) to fewer than 50. The remaining call sites MUST be documented with `#[expect]` and a one-line reason.

#### Scenario: Production code parses a configuration value
- **WHEN** production code reads a configuration value that might be missing or invalid
- **THEN** it MUST return a `UarError` or propagate an `Err` instead of calling `unwrap()` or `expect()`

### Requirement: Tracing span context on every error
Every route handler in `src/uar/api/` MUST execute inside a `tracing` span that carries at least the fields `request_id`, `agent_id`, and `run_id`. When `UarError::into_response` captures `tracing_error::SpanTrace`, the trace MUST be non-empty for normal requests and MUST contain the three IDs.

#### Scenario: A request fails in a route handler
- **WHEN** a route handler returns `Err(UarError::...)`
- **THEN** the emitted `tracing::error!` event MUST include a `SpanTrace` containing `request_id`, `agent_id`, and `run_id`
- **AND** the public HTTP response body MUST still contain only `code` and `message`

### Requirement: Sentry release wiring
Release builds MAY be produced with the `sentry` feature enabled. When the feature is enabled, `UarError::into_response` reports the error to Sentry via `sentry::capture_error`. The release workflow MUST include a build job that passes `--features sentry`.

#### Scenario: A release is published with Sentry enabled
- **WHEN** CI builds with `--features sentry`
- **AND** the `SENTRY_DSN` environment variable is configured at runtime
- **THEN** errors rendered through `UarError` are reported to Sentry

#### Scenario: A default release is published
- **WHEN** CI builds without the `sentry` feature
- **THEN** no Sentry code is compiled in and no DSN is required

### Requirement: Operator documentation
`docs/observability.md` MUST exist and explain:
- The structure of request spans (`request_id`, `agent_id`, `run_id`)
- How `UarError` captures and logs span traces
- How to enable Sentry in a release build
- The clippy unwrap/expect policy and how to request an exception

#### Scenario: A new operator wants to enable Sentry
- **WHEN** an operator reads `docs/observability.md`
- **THEN** they can find the required Cargo feature, env var, and release workflow step
