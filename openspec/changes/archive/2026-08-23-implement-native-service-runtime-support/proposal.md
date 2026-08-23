## Why

The current runtime cannot consume a dedicated service environment file, cannot direct tracing to a service log file, lacks Windows SCM control integration, binds A2A gRPC independently of `server.host`, and skips embedded-catalog enrichment for YAML-defined providers.

## What Changes

- Add fail-closed `--env-file`/`UAR_ENV_FILE` and `UAR_LOG_FILE` support.
- Add a Windows-only native SCM service command pinned to `windows-service` 0.8.1.
- Route SCM stop/shutdown through existing graceful cancellation.
- Make A2A gRPC inherit `server.host`.
- Enrich YAML provider definitions before first registration.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: runtime inputs, logging, Windows lifecycle, and provider bootstrap behavior.
- `a2a-grpc`: bind-address inheritance.
- `graceful-shutdown`: external supervisor cancellation on Windows.
- `provider-model-settings-certification`: catalog enrichment on YAML seed.

## Impact

- Changes server-full startup behavior and Windows-only CLI surface.
- Adds one exact Windows target dependency; no new public Rust API.
