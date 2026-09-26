## Why

The supervised UAR sidecar always binds an opaque operating-system-assigned port. The Boss therefore cannot show a predictable endpoint or let an operator resolve a local port conflict before restarting the runtime.

## What Changes

- Start the sidecar at port 1906 by default.
- When that port is occupied, advance one port at a time and atomically retain the first listener that binds.
- Honor the existing `--port`/`PORT` input as the preferred starting port while preserving loopback-only binding and the launch-token guard.
- Continue reporting the actual listening port through the existing `READY:{port}` contract.

## Capabilities

**Modified Capabilities**
- `sidecar-launch-security` — supervised launch now includes deterministic preferred-port selection without weakening loopback or token admission.

## Impact

- `src/bin/uar-sidecar.rs`
- The Boss can persist a preferred port and display the effective port returned at readiness.
