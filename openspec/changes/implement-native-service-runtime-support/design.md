## Context

Service supervisors provide explicit environment files and lifecycle controls. UAR already has graceful cancellation and provider catalog enrichment; this change connects those existing mechanisms rather than adding parallel systems.

## Decisions

1. Load a selected env file before configuration and telemetry; any selected file error terminates startup.
2. `UAR_LOG_FILE` selects a file-backed tracing writer while preserving current stdout behavior when absent.
3. Windows SCM service status and controls live behind `cfg(windows)` and an exact target dependency.
4. Add a crate-private `start_server_with_shutdown` entrypoint accepting a process-scoped `CancellationToken`. SCM Stop/Shutdown cancels that token; it does not terminate the process directly. The existing `start_server_with_listener(..., http_shutdown)` token remains HTTP-only for embedded callers.
5. Resolve the gRPC socket from `server.host` and its configured gRPC port.
6. Call the existing provider enrichment function before YAML provider persistence.

## Risks

- A different Windows shutdown path would split cleanup semantics; the adapter must call the existing cancellation seam.
- A log file open failure cannot be silently ignored in a service process.
- IPv4/IPv6 host parsing must not restore a wildcard fallback.
