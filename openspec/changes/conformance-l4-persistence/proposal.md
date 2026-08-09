# conformance-l4-persistence

Phase: `uar-spec-conformance-2026-08` (change C-05)

## Why

The matrix cannot produce a single L4 result. `boot_test_server` mints a fresh
temp SurrealKV path per boot (`harness.rs:143-149`, `unique_temp_path`), so
write→reboot→read is inexpressible.

That matters most exactly where it is missing. **C-12 is persistence and C-13 is
sessions** — the two capabilities whose defining property *is* surviving a
restart. A shape-only pass on C-12 asserts that a persistence-config endpoint
returns well-formed JSON. It does not assert that anything persists. Adversarial
review called a pass of this kind vacuous, and it is: the property that defines
the capability is the one not tested.

## What Changes

Expose the shutdown that already exists, let the harness reuse a DB path, and
write the round-trip.

**Graceful shutdown is already implemented.** Verified by reading
`src/server.rs` on 2026-08-09:

| Line | What is there |
|---|---|
| 1386 | `let http_shutdown = tokio_util::sync::CancellationToken::new()` |
| 1388-1420 | signal-handler task: SIGINT/SIGTERM → drain ingestion pool → `http_shutdown.cancel()` |
| 1425-1438 | `shutdown_future` awaits the token, then drains in-flight connections with a timeout |
| 1441, 1453 | both listeners wired via `.with_graceful_shutdown(...)` |

Nothing needs designing. The token is created *internally* and only signal
handlers can fire it, so the only thing a test lacks is a way to **own** it.

`start_server_sidecar` (`server.rs:1357`) already accepts a caller-supplied
`oneshot::Sender<SocketAddr>` for readiness. A caller-supplied
`CancellationToken` is the same shape of parameter on the same function —
additive, and the existing signal handler keeps working untouched.

> An earlier revision of this phase's plan classified this as boot-path refactor
> work requiring its own scoping decision, and held it out of the execute
> handoff. Reading the code refuted that. The classification error ran in the
> expensive direction: it made L4 look costly and pushed real work out of scope.

## Impact

- Affected specs: `spec-conformance-measurement`
- Affected code: `src/server.rs` (additive parameter),
  `tests/integration/live/harness.rs` (optional fixed DB path),
  `tests/integration/live/capability_cases.rs` (C-12, C-13 round-trips)
- Risk: medium — this is the only change in the phase touching runtime source.
  The parameter is additive and the existing signal path is unchanged, but it is
  a public-ish seam and deserves the tier-2 gate before merge.
- Depends on: `conformance-baseline-gate`
