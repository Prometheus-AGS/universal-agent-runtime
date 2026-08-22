# PLAN: fix-graceful-shutdown-deadline-semantics

Project: universal-agent-runtime
Date: 2026-08-22
OpenSpec available: YES
Changes to implement: 1

## CHANGE LIST (ordered)

1. `fix-graceful-shutdown-deadline-semantics`: Begin Axum drain immediately and enforce the configured shutdown timeout as a maximum process deadline.
   - Scope: server shutdown orchestration | process-boundary tests | release-candidate container certification | OpenSpec and KBD evidence
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Details: Keep the existing signal, runtime-run cancellation, ingestion-pool cleanup, A2A shutdown, and dual-listener behavior. Make Axum's shutdown signal resolve as soon as HTTP shutdown is requested and supervise the complete shutdown sequence against one deadline that begins at shutdown initiation. Explicitly cancel configured MCP services and wait for their transports to close on the normal branch. If work remains at expiry, an OS watchdog enforces a non-graceful process exit with code 0; it never reports that branch as graceful or cleanup-complete. Add focused deadline-semantic tests and a real child-process SIGTERM test for the no-active-connection one-second requirement. Give Docker's stop deadline explicit margin beyond UAR's internal default and retain exit-code-zero as the external assertion.

## EXECUTION ROUND ORDER

Round 1 (serial): `fix-graceful-shutdown-deadline-semantics`

## IMPLEMENTATION ORDER

1. Create the OpenSpec proposal, graceful-shutdown delta, design, and tasks; validate strictly before Execute.
2. Add focused tests that distinguish immediate signal-to-drain from the current mandatory-sleep behavior. Preserve the observed candidate exit-137 evidence as the negative control.
3. Change `src/server.rs`, `src/mcp/registry.rs`, and `src/uar/realtime/surreal_bus.rs` so the Axum signal future resolves immediately, configured MCP transports close explicitly, UAR-owned embedded-database background tasks stop and join, and an OS-thread watchdog enforces one graceful-drain deadline across listeners, cleanup, MCP, embedded persistence, and A2A. The deadline starts at signal observation, including ingestion-pool work.
4. Update the non-root container certification to leave bounded orchestration margin outside the runtime's internal deadline; do not weaken the exit-code-zero assertion.
5. Run Tier 0 after each source edit, then focused Tier 1 tests when the unit is complete. Run `openspec validate fix-graceful-shutdown-deadline-semantics --strict` and artifact-refiner validation.
6. Commit and reflect the child, resume the parent at `certify-operational-resilience`, freeze the corrected SHA, and restart the complete 10,800-second local certification from zero.

## PERMITTED WRITE SURFACE

- `src/server.rs`
- `src/mcp/registry.rs`
- `src/uar/realtime/surreal_bus.rs`
- `tests/integration/live/harness.rs`
- `tests/integration/live/capability_cases.rs` only if the existing process harness cannot host the focused assertion without it
- `scripts/certify-release-candidate.sh`
- `openspec/changes/fix-graceful-shutdown-deadline-semantics/**`
- `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-graceful-shutdown-deadline-semantics/**`
- `.kbd-orchestrator` canonical projections produced by `prometheus kbd`
- `.prometheus` append-only history for the observed defect and resolution

## EXPLICIT CUTS AND TRADE-OFFS

- Do not redesign runtime lifecycle ownership, replace Axum, or alter public APIs.
- Under `server-full`, SQLx is excluded because `postgres-backend` is not active, and Redis is excluded because UAR owns no Redis client. These are profile facts to record, not cleanup successes to claim.
- Do not count the synthetic `operational_resilience` lifecycle test as process-boundary proof.
- Do not merely increase Docker's timeout; that would hide the product defect.
- Axum has no public force-close handle in the documented `serve` API. When UAR's deadline expires, a dedicated OS watchdog makes one bounded non-blocking attempt to record `deadline_enforced` and terminates the process; the marker is required when that write is accepted, while exit timing and status remain authoritative if the sink rejects it. The forced branch is not reported as graceful or cleanup-complete and must pass held-SSE and held-cleanup process tests.
- Any source or certification-script edit invalidates the previous candidate. The full three-hour certification must restart on the new committed SHA.

## COMMANDS TO RUN

`/opsx:new fix-graceful-shutdown-deadline-semantics`

## PLAN COMPLETE
