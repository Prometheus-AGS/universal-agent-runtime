# EXECUTION: fix-mcp-reconnect-shared-service-state

Project: universal-agent-runtime
Date: 2026-08-21
Selected backend: openspec
Dispatched to: Codex
Backend rationale: The observed installed-runtime defect crosses private MCP ownership, request-policy projections, streamed evidence, and release certification, so its behavior and proof must remain spec-backed and independently reviewable.
Backend entrypoint: `/opsx:apply fix-mcp-reconnect-shared-service-state`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-mcp-reconnect-shared-service-state/plan.md`

## EXECUTION SCOPE

- `fix-mcp-reconnect-shared-service-state`: share replaceable per-server MCP service identity across already-authorized registry views, retain separate policy maps, never replay a failed operation, and prove crash/timeout recovery at the installed subprocess boundary.

## DISPATCH CONTRACTS

- OpenSpec `tasks.md` is the implementation ledger; canonical change and stage transitions go through `prometheus kbd`.
- Only paths in child `scope.json` may be written. No dependency, public API, protocol, UI, submodule, or GitHub Actions change is permitted.
- The source/tooling commit precedes candidate construction. Evidence, reflection, and review land in a second commit so the tested source SHA is immutable and reproducible.

## APPROVAL GATES

- The operator directed autonomous execution of the planned phases.
- This child authorizes local source commits and pushes on the existing branch; it does not authorize a release tag, GA publication, deployment, or new PR.

## FALLBACK CONDITIONS

- Stop if service replacement requires sharing unfiltered registry maps or holding a synchronous lock across `.await`.
- Stop if the failed call is replayed, a later call uses the stale process, or a denied server/tool becomes visible.
- Stop if a new dependency, out-of-scope runtime file, GitHub Actions test, or parent release claim is required.
- Stop if the installed crash/timeout trace is not exactly `echo, crash, echo, timeout, echo` with replacement process identifiers after each destructive call.

## VERIFICATION REQUIREMENTS

- Run package-scoped Tier 0 check and Clippy locally after Rust edits; identify new-file warnings separately from pre-existing repository warning debt.
- Run only the exact new library tests at Tier 1 while the unit is active.
- Run local release-contract and GitHub Actions deployment-only policy validators.
- Commit source/tooling, construct a fresh immutable candidate from that commit, then run the short local installed-artifact preflight including the real 30-second timeout.
- Run strict child and parent OpenSpec validation, artifact-refiner, and history-free critic/judge gates before reflection and archive.

## PROGRESS LEDGER

- [IN_PROGRESS] `fix-mcp-reconnect-shared-service-state` — Codex

## OUTPUTS

- Private MCP registry repair and focused tests.
- Stream/process-boundary certification validator and negative controls.
- Exact local verification evidence, artifact-refiner state, child reflection/handoff, canonical spec sync, and scoped source/evidence commits.

## BLOCKERS

- None. The prior immutable candidate is invalidated by this required source repair.

## REFLECTION HANDOFF

- Record the delta between event-level failure observation and cross-request service ownership, the temporary child-expanded waypoint denominator, all pre-existing warning debt excluded from scope, and the new immutable candidate SHA used by the resumed parent certification.

## EXECUTION READY
