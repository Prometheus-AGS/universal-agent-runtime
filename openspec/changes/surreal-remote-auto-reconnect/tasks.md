# Tasks — surreal-remote-auto-reconnect

Fixes GitHub issue #441. Verification is one BDD scenario run after implementation is complete and the operator opens the gate. Nothing runs in GitHub Actions.

## 1. Implementation

- [x] 1.1 Add `src/server/lifecycle_auto_reconnect.rs`: supervised driver that polls the retained transport state and calls `RuntimeHandle::reconnect` with capped doubling backoff (250 ms → 30 s) until the handler succeeds, the transport is released, or shutdown cancels its scope.
- [x] 1.2 Start the driver for `persistence` (dependencies `persistence`, `realtime`) and `memory` (dependencies `memory`, `memory-realtime`) only when `start_server` received no host control.
- [x] 1.3 `readiness_handler`: report `checks.persistence` with the backend, keep `checks.postgres` only for PostgreSQL, log the failed check's cause.
- [x] 1.4 Update the `/readyz` OpenAPI description.

## 2. Behavior scenario (authored now, run at the gate)

- [x] 2.1 Host fixture `scripts/acceptance/fixtures/remote-surreal-runtime.mjs`: pinned SurrealDB image with a persistent volume, a dedicated UAR and identity proxy; generated root credentials stay inside the fixture.
- [x] 2.2 `scripts/behaviors/features/remote-surreal-restart.feature` with steps and cucumber config.
- [x] 2.3 Run the scenario once; record evidence under `.prometheus/verification/remote-surreal-restart/`.

## 3. Validation

- [x] 3.1 `openspec validate surreal-remote-auto-reconnect`.
- [x] 3.2 Build `universal-agent-runtime` with `server-full`.
