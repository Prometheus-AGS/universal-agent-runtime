# EXECUTION: fix-graceful-shutdown-deadline-semantics

Project: universal-agent-runtime
Date: 2026-08-22
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The observed release blocker crosses process-signal coordination, HTTP/SSE draining, cleanup ordering, and the installed non-root container boundary, so behavior and evidence remain spec-backed and independently reviewable.
Backend entrypoint: `/opsx:apply fix-graceful-shutdown-deadline-semantics`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-graceful-shutdown-deadline-semantics/plan.md`

## EXECUTION SCOPE

- `fix-graceful-shutdown-deadline-semantics`: begin HTTP drain at signal observation, enforce one process deadline across HTTP/SSE, registered cleanup, and A2A shutdown, and preserve an externally observable exit-code-zero boundary.

## DISPATCH CONTRACTS

- OpenSpec `tasks.md` is the implementation ledger; canonical change and stage transitions go through `prometheus kbd`.
- Only paths in child `scope.json` may be written. The approved surface includes crate-private MCP registry shutdown in `src/mcp/registry.rs` and crate-private live-query task shutdown in `src/uar/realtime/surreal_bus.rs`; no dependency, public API, protocol, provider, UI, submodule, or GitHub Actions change is permitted.
- Test fixtures precede product behavior. Tier 0 runs after each Rust edit; only focused Tier 1 runs while this unit is active.

## APPROVAL GATES

- The operator directed autonomous application of this child change through completion.
- No release tag, GA publication, deployment, GitHub Actions test execution, push, or new PR is authorized by this child.

## FALLBACK CONDITIONS

- Stop if immediate listener drain or the absolute deadline requires replacing Axum, a custom Hyper accept loop, a new dependency, or a public API change.
- Stop if caller-owned HTTP cancellation would arm the process watchdog or terminate the embedding process.
- Stop if a forced branch is labeled graceful or cleanup-complete, or if the exit bound depends on Tokio or a writable stderr sink.
- Stop if the active `server-full` SurrealDB or MCP normal cleanup guarantees must be weakened. SQLx is excluded because `postgres-backend` is not active in this profile; Redis is excluded because UAR owns no Redis client.

## VERIFICATION REQUIREMENTS

- Preserve the immutable exit-137 candidate evidence and observe the new process tests fail against baseline shutdown behavior before changing it.
- Prove idle SIGTERM/SIGINT, real SSE completion, both-listener refusal, held SSE, held registered cleanup, caller-owned cancellation isolation, and locked/backpressured stderr behavior in real child processes.
- Run package-scoped Tier 0 check and Clippy locally, exact focused Tier 1 tests, strict OpenSpec, artifact-refiner, scoped diff checks, and the held-work non-root container journey.
- Commit the child source/evidence, create a new immutable candidate SHA, and restart the complete 10,800-second local certification from zero.

## PROGRESS LEDGER

- [IN_PROGRESS] `fix-graceful-shutdown-deadline-semantics` — Codex

## OUTPUTS

- Private shutdown coordinator, process-boundary tests, local container certification evidence, row-form OpenSpec verification, child reflection/handoff, and one scoped commit.

## BLOCKERS

- None. The earlier candidate remains invalid because the non-root container exited 137.

## REFLECTION HANDOFF

- Record the difference between a mandatory pre-drain delay and an absolute graceful deadline, the forced-versus-graceful outcome boundary, all negative controls, and the new immutable SHA used by restarted parent certification.

## EXECUTION READY
