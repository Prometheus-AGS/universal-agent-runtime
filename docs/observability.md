# UAR observability and error handling

This document describes how UAR captures request context and reports errors so operators can diagnose production issues.

## Request spans

Every HTTP request enters a `tracing` span named `http_request` before it reaches a route handler. The span carries three identifiers:

| Field | Source | Example |
|---|---|---|
| `request_id` | `X-Request-ID` header, or a generated UUID if absent | `req-123` |
| `agent_id` | `"none"` by default; handlers with an explicit agent record it | `agent-456` |
| `run_id` | `"none"` by default; handlers with an explicit run record it | `run-789` |

The middleware lives in [`src/server.rs`](../src/server.rs) as `request_span_layer`. It is applied as the outermost layer around the Axum router so the span remains active for the entire request.

### Recording agent/run IDs in handlers

When a handler knows the agent or run identifier, it can record it into the current span:

```rust
tracing::Span::current().record("agent_id", &agent_id);
tracing::Span::current().record("run_id", &run_id);
```

After recording, any `UarError` rendered by that handler will include the updated values in its captured `SpanTrace`.

## Error reporting

`UarError` is the central error type for the public API boundary. When a handler returns `Err(UarError)`:

1. `UarError::into_response` captures the current `tracing_error::SpanTrace`.
2. It emits a `tracing::error!` event containing the error, its stable `code`, and the span trace.
3. The public HTTP response body contains only `{ "code": "...", "message": "..." }`; no internal trace is leaked.

Stable error codes follow the format `E_<DOMAIN>_<SPECIFIC>` (e.g. `E_CONFIG_MISSING_FIELD`, `E_RAG_NO_KB`). See [`src/uar/error.rs`](../src/uar/error.rs) for the full set of domain variants.

## Sentry integration

Sentry error reporting is behind the `sentry` Cargo feature and is **off by default**.

### Enable Sentry in a release build

```bash
cargo build --release --no-default-features --features server-full,sentry
```

At runtime, set the DSN via environment variable:

```bash
export SENTRY_DSN="https://public@example.sentry.io/1"
```

When the feature is enabled and a `UarError` is rendered, `sentry::capture_error` is called. When the feature is disabled, no Sentry code is compiled in and no DSN is required.

Local release verification builds the Sentry-enabled bundle and records the
result before publication.

## Clippy unwrap/expect policy

The following paths are guarded by `#![deny(clippy::unwrap_used, clippy::expect_used)]`:

- `src/uar/api/`
- `src/uar/runtime/`
- `src/server.rs`

New `unwrap()` or `expect()` calls in these paths fail local Clippy verification.
Legitimate exceptions (init-time, static-parse, or test-only) must be annotated
with `#[expect(clippy::unwrap_used, reason = "...")]` or
`#[expect(clippy::expect_used, reason = "...")]`.

## Troubleshooting

### Missing request context in logs

If `SpanTrace` in the error log is empty, check that:

1. The request reached the `request_span_layer` middleware (it is applied before `TraceLayer`).
2. The handler is not explicitly dropping into an uninstrumented future.

### Sentry feature build fails

Ensure you are using the same feature set as CI:

```bash
cargo check --no-default-features --features server-full,sentry
```

If the failure is in a dependency, verify the `sentry` version in `Cargo.toml` matches the documented feature flags.
