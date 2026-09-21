# Execution: runtime-harness-gap-closure

Project: universal-agent-runtime. Date: 2026-09-16. Selected backend: `openspec`. Dispatched to: Codex self-execution through the six existing OpenSpec changes. Backend entrypoint: `/kbd-execute runtime-harness-gap-closure`, then the semantic KBD task IDs in execution-map.json. OpenSpec is available. Source plan: `.kbd-orchestrator/phases/runtime-harness-gap-closure/plan.md`.

The OpenSpec backend is selected because the phase already has 11 reviewed capability deltas, 44 registered tasks and 85 scenario mappings. Canonical execution position and task status remain in the typed KBD ledger and projected progress.json. OpenSpec tasks.md is the implementation checklist. Do not create replacement changes or use OpenSpec ordinal IDs for KBD transitions.

## Execution scope and order

1. `harness-protected-context` — canonical receipts, lossless typed history, pure protected-data budget and common final-request enforcement.
2. `harness-model-profiles-routing` — exact destination templates/settings, production task classification and constrained routing.
3. `harness-task-graph-lifecycle` — durable protocol identity/recovery, causal graph/child events, approvals, shared limits and cleanup.
4. `harness-skill-activation-quality` — scoped explicit activation, immutable skill bodies and outcome-aware evaluation.
5. `harness-knowledge-evidence` — authorization at use, claim/source verdicts and existing knowledge journey preservation.
6. `harness-governed-evaluation` — production-host evaluation, local baseline gates, historical failure/coverage debt and F1–F8 evidence.

Each change uses the same handoff: read the current waypoint, its OpenSpec proposal/design/specs/tasks and execution-map.json; transition the exact semantic KBD task ID to in-progress; make the minimum mapped edit; run the mapped tier check; retain command/output evidence; check the matching task in tasks.md; transition it complete. On failure, retain the output and leave the task in-progress or record a typed blocker. Never hand-edit generated progress or waypoint projections.

## Gates

- No product behavior edit before change 1 task 1.1 verifies the final Plan artifacts and task 1.2 captures the frozen source baseline in an external worktree with a separate target directory.
- No actual destination is enabled for guaranteed-fit behavior without exact effective configuration, alias/revision, roles/settings, context/input/output/reasoning semantics, and a defensible whole-request count bound.
- Protected tool calls, arguments/results, structured/code/log/file/media data, evidence, active skill bodies, required instructions/current input, durable decisions and pending work are never summary candidates. Authorization and retention still take precedence.
- The final outbound request is checked after provider transforms; the pinned Liter Anthropic max-token adjustment has a specific fixture.
- All non-deployment verification is local. GitHub Actions remain deployment-only.
- Tier 3 belongs only to the registered child `runtime-harness-profile-certification`, whose passing canonical state and hashed receipt are required before parent completion.
- The final Plan corrections received Tier 0 checks after the two-round review cap. Task 1.1 rechecks their hashes and predicates; this limitation remains visible in evidence.

## Verification

Run the exact unit commands from execution-map.json after each completed unit. Rust edits receive the repository Tier 0 check `cargo check --locked --no-default-features --features server-full`; use one build profile and one writer per target directory. Tier 2 and Tier 3 commands remain deferred to their defined boundaries. A successful compile cannot replace behavior fixtures or final-request/receipt evidence.

Per completed change with three or more modified files, run the configured artifact-refiner quality gate from `.kbd-orchestrator/constraints.md` when present, then OpenSpec verification. Do not archive an individual change while later registered changes still depend on its active delta; retain verified evidence and archive only when dependency-safe.

## Fallback and stop conditions

OpenSpec is already the fallback surface. Stop the affected task and record the evidence when a requirement is ambiguous enough to change design, an edit needs a path outside execution-map.json, an existing behavior would be broken, a required provider/count contract is unavailable, a baseline cannot be attributed to the frozen revision, or a destructive/hard-to-reverse operation would be required. Scope amendments require a reviewed Plan update before editing new product paths.

## Progress ledger

- `harness-protected-context` — in progress at prerequisite gate 1.1.
- `harness-model-profiles-routing` — pending predecessor completion.
- `harness-task-graph-lifecycle` — pending predecessor completion.
- `harness-skill-activation-quality` — pending predecessor completion.
- `harness-knowledge-evidence` — pending predecessor completion.
- `harness-governed-evaluation` — pending predecessor completion.

## Reflection handoff

Reflect consumes the task ledger, OpenSpec checkboxes, exact command receipts, baseline/candidate metrics, provider/profile support matrix, review/refiner receipts, F1–F8 claim/evidence matrix, unresolved blockers and the child Tier 3 receipt. It leads with plan-versus-delivery deltas. No unsupported, blocked, skipped or stale evidence counts as success.

Execution is ready. Implementation and runtime verification remain pending.
