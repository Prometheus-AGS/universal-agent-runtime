## Context

`uar-sidecar` is not a public server deployment. It binds `127.0.0.1:0`, publishes the chosen port over a private stdout pipe, and exits when the supervising parent's stdin pipe closes. BossFang authenticates its operator routes before forwarding requests to this child.

The standalone UAR default `security.jwt_required = true` nevertheless applies to the child. That yields a contradictory state: `/readyz` is green, but the first real client request to `/api/openapi.json` is rejected with 401.

## Decision

When neither `UAR_SECURITY__JWT_REQUIRED` nor the legacy `JWT_REQUIRED` variable is explicitly present, the sidecar entry point sets `UAR_SECURITY__JWT_REQUIRED=false` before loading configuration. Explicit environment configuration remains authoritative.

This belongs in the sidecar executable rather than each parent integration: loopback binding and the private parent contract are properties the executable itself owns, and every supervisor needs the same usable default.

The entry point uses a synchronous bootstrap to load dotenv, apply all environment
overrides, reserve the listener, and publish its selected port into configuration before
constructing Tokio's multithread runtime. Rust 2024 makes process-environment mutation
unsafe because concurrent readers can observe invalid state; `#[tokio::main]` would create
worker threads before the function body and therefore cannot truthfully support the
sidecar's former "still single-threaded" safety assumption.

## Security

- The sidecar continues to force `127.0.0.1` and an ephemeral port.
- Standalone `universal-agent-runtime` behavior is unchanged.
- Parents remain responsible for authenticating their externally reachable control surfaces.
- Operators that require defense against other local processes can explicitly enable UAR JWT enforcement and provide a compatible token exchange.
- Process-global configuration is complete before any application worker threads exist.
