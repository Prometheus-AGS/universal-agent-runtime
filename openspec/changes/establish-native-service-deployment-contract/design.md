## Context

Native services have different supervisors and filesystem conventions but share one UAR binary and configuration model. The contract must prevent three recurring errors: exposing a secondary listener, overwriting operator-owned state, and claiming a platform was deployed when only its template compiled.

## Decisions

1. macOS uses a user LaunchAgent, Linux uses systemd, and Windows uses native SCM.
2. Installed listeners are loopback-only and every network transport inherits `server.host`.
3. Immutable program files and mutable state use platform conventions; operational output lives below an external `.prometheus/logs` directory.
4. Existing configuration and databases remain authoritative. Installers back up and merge only missing phase-owned entries.
5. macOS is runtime-verified locally. Linux and Windows receive separate structure/compile evidence only.

## Risks

- A superficially local install could expose gRPC if any listener bypasses `server.host`; listener inspection is therefore a completion gate.
- YAML merge logic could destroy user configuration; replacement of an existing file is forbidden.
- Retained evidence could leak credentials; commands and artifacts record names and redacted presence only.
