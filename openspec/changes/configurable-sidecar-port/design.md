## Context

The sidecar currently binds `127.0.0.1:0` before configuration is loaded. This prevents conflicts but makes the endpoint unpredictable and impossible to administer from the supervising desktop application.

## Decision

Treat the configured port as a preferred starting port. Bind directly to that loopback address and, only for `AddrInUse`, advance until a listener is acquired. The acquired listener remains owned through Axum startup, preserving the existing protection against a probe-to-bind race. The default preferred port is 1906.

The sidecar reports the effective port through `READY:{port}`. It does not expose the launch token or weaken authority validation. Other bind failures remain terminal and visible to the supervisor.

After binding, the effective port replaces the preferred CLI value before application configuration is loaded. Runtime administration therefore reports the same port as the retained listener when fallback occurs.

## Rollback

Revert to binding `127.0.0.1:0`; The Boss must then remove preferred-port administration because the requested port would have no effect.
