## Why

GitHub issue #441: on 2026-10-26 a UAR 1.3.0 Kubernetes deployment with `persistence.provider = surreal` and a remote SurrealDB server stayed not-ready for about 2h20m after the SurrealDB container restarted (exit 137, ready again 1 s later, data intact). `/readyz` returned 503 the whole time, `/healthz` kept passing, and only `kubectl rollout restart deploy/uar` recovered it.

Source reading shows why. The patched SurrealDB 3.3.0 SDK (`vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/engine/remote/ws/native.rs:228-279`) publishes `TransportState::Unavailable` when the socket drops and then rejects every operation with "remote transport unavailable; operation was not admitted" until the host calls `Surreal::reconnect(expected_generation)`. That reconnect replays signin and `use_ns`/`use_db` for each session. UAR registers reconnect handlers for `persistence`, `realtime`, `memory` and `memory-realtime` (`src/server.rs`), but only `ServerLifecycleControl::execute` invokes them, and only the `uar-sidecar` stdin control pipe drives that. The standalone `universal-agent-runtime` binary that the Docker image runs starts with no control, so it never attempts a reconnect.

The readiness probe hid the cause: `readiness_handler` discarded the `list_skills()` error and reported a Surreal backend under the key `postgres`.

## What Changes

- A standalone server (no host control attached) drives its own recovery. For `persistence` and `memory`, a supervised driver watches the retained transport state and, on loss, calls the existing `RuntimeHandle::reconnect` with the current generation. It retries with a delay that starts at 250 ms, doubles, and is capped at 30 s, until the registered handler succeeds or shutdown begins. Each loss, failed attempt and recovery is logged.
- A sidecar host keeps reconnect authority. When control is attached, no driver starts.
- `/readyz` reports `checks.persistence = {"backend": <provider>, "status": ok|failed|not_configured}`. The `postgres` key remains only for a PostgreSQL backend. A failed persistence check logs its cause (`readiness.persistence_failed`); the response never carries it.
- `/healthz` is unchanged and stays dependency-free.

## Capabilities

### New Capabilities
- `remote-persistence-recovery`: a standalone server recovers its remote SurrealDB transports after a loss without a process restart, while a sidecar host keeps that authority.

### Modified Capabilities
- `deep-health-probes`: the readiness probe names the configured persistence backend and logs the cause of a failed check.

## Impact

- **Code:** new `src/server/lifecycle_auto_reconnect.rs`; `src/server.rs` (driver registration for `persistence` and `memory`, `readiness_handler`); `src/uar/api/openapi.rs` (readiness description).
- **APIs:** `/readyz` body gains `checks.persistence`; Surreal-backed deployments no longer report a `postgres` key. HTTP status semantics are unchanged.
- **Operations:** a remote SurrealDB restart no longer needs a UAR pod restart. Logs carry `persistence.transport.lost`, `persistence.transport.reconnect_failed` and `persistence.transport.recovered`.
- **Known limit:** if the server rejects a replayed signin (for example, rotated credentials), the SDK poisons that session permanently. The driver keeps retrying and logging, and readiness stays 503 with the cause in the log; a process restart is still required in that case.
- **Not changed:** the sidecar and The Boss reconnect path. A sidecar host that never sends `Reconnect` has the same exposure.
- **Dependencies:** none.
