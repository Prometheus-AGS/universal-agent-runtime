# EXECUTION: fix-broken-session-configuration-ui

Project: Universal Agent Runtime with one controlled Prometheus Entity Management upstream change
Date: 2026-08-23
Selected backend: hybrid
Dispatched to: Codex primary executor
Backend rationale: UAR changes use KBD-owned OpenSpec task execution; the upstream change is implemented in its separate repository/worktree while this UAR phase remains the canonical progress authority.
Backend entrypoint: `/kbd-apply <change>` semantics, one task at a time; no bare `/opsx:apply`
OpenSpec available: YES in both repositories
Source plan: `.kbd-orchestrator/phases/fix-broken-session-configuration-ui/plan.md`

## Execution scope

- `adopt-entity-management-3-0-2`: exact registry dependency and lockfile baseline
- `repair-session-configuration-entity-flow`: entity-backed session editor and effective inference route
- `prevent-session-configuration-regressions`: durable rules, scoped static negative controls, and one post-code-completion functional proof
- `fix-atomic-fetched-list-ingestion`: upstream atomic graph list ingestion, patch version, and PR

## Dispatch contracts

`model_policy` is absent from `.kbd-orchestrator/project.json`; the model-routing
contract therefore defaults every dispatched change to the current frontier
Codex executor. The plan's class remains recorded for later policy activation.

- `adopt-entity-management-3-0-2` → Codex / KBD-owned OpenSpec
  - Entry: task-by-task execution of `openspec/changes/adopt-entity-management-3-0-2/tasks.md`
  - Model class: medium
  - Concrete model: current Codex frontier model (required fallback because project model policy is absent)
  - Model rationale: five bounded tasks cross two lockfile authorities but introduce no product abstraction
  - Worktree: `~/.claude/worktrees/uar-adopt-entity-management-3-0-2`
  - Progress: this phase's canonical KBD task/change transitions and `progress.json`

- `repair-session-configuration-entity-flow` → Codex / KBD-owned OpenSpec
  - Entry: task-by-task execution of `openspec/changes/repair-session-configuration-entity-flow/tasks.md`
  - Model class: frontier
  - Concrete model: current Codex frontier model
  - Model rationale: crosses entity platform, React domain/UI, typed API, persistence policy, and Rust provider routing
  - Worktree: new UAR worktree based on the accepted dependency commit
  - Progress: this phase's canonical KBD task/change transitions and `progress.json`

- `prevent-session-configuration-regressions` → Codex / KBD-owned OpenSpec
  - Entry: task-by-task execution of `openspec/changes/prevent-session-configuration-regressions/tasks.md`
  - Model class: medium
  - Concrete model: current Codex frontier model (required fallback because project model policy is absent)
  - Model rationale: bounded instruction/static-gate changes plus one existing Playwright workflow
  - Worktree: new UAR worktree based on the accepted entity-flow commit
  - Progress: this phase's canonical KBD task/change transitions and `progress.json`

- `fix-atomic-fetched-list-ingestion` → Codex / upstream OpenSpec with UAR KBD transitions
  - Entry: task-by-task execution of `/Users/gqadonis/.claude/worktrees/entity-management-fix-atomic-fetched-list-ingestion/openspec/changes/fix-atomic-fetched-list-ingestion/tasks.md`
  - Model class: frontier
  - Concrete model: current Codex frontier model
  - Model rationale: new graph-store action plus engine, bindings, adapters, compatibility, and fixed-group release behavior
  - Worktree: `/Users/gqadonis/.claude/worktrees/entity-management-fix-atomic-fetched-list-ingestion`
  - Progress: UAR phase canonical KBD transitions; upstream OpenSpec checkboxes are the task artifact
  - Handoff: commit, push, and PR the isolated upstream branch after verification; never modify the dirty primary checkout

## Task-driving rule

For UAR changes, use the KBD apply driver for begin/end task boundaries and
backend verification. For the external upstream repository, run the equivalent
single-task loop explicitly: transition the UAR canonical task to in-progress,
perform only that upstream OpenSpec task, mark its checkbox through a surgical
edit, transition the canonical task complete, and preserve both ledgers. Never
invoke a backend do-everything command.

## Approval gates

- No approval is required for scoped implementation, local checks, UAR commits, or the user-authorized upstream commit/push/PR.
- New UAR push/PR authority is not inferred.
- Any `spec-index.md` stop condition halts execution and is reported before further mutation.

## Fallback conditions

- If the UAR KBD apply driver cannot preserve task/waypoint state, stop and repair the KBD seam; do not use bare `/opsx:apply`.
- If the upstream worktree or fixed-group release contract prevents a safe code fix, create the fully evidenced upstream issue instead of patching UAR around it.
- If a required file is outside the permitted surface, revise the reviewed plan before touching it.

## Verification requirements

- During implementation: only required Tier 0 typecheck/lint/cargo-check feedback.
- After all UAR code is complete: the one short functional HTTP/browser sequence at `http://localhost:1906` defined in `plan.md`; no broad suite or soak.
- After upstream code is complete: the public 1/12/7,248-row publication integration fixture, 3.0.2 negative control, and only affected package/consumer checks.
- Every change: strict OpenSpec validation, row-form verification, artifact-refiner gate, fresh-context diff review, then archive when its phase ordering permits.
- All product verification is local. GitHub Actions remain deployment-only.

## Progress ledger

- [PENDING] `adopt-entity-management-3-0-2` — Codex
- [PENDING] `repair-session-configuration-entity-flow` — Codex
- [PENDING] `prevent-session-configuration-regressions` — Codex
- [PENDING] `fix-atomic-fetched-list-ingestion` — Codex

## Outputs

- Three independently committed UAR changes on a forward-only branch chain
- One upstream Entity Management patch branch/PR or exact evidenced issue
- Per-change OpenSpec verification and independent QA/review receipts
- Browser/network/server evidence and durable `.prometheus` history

## Blockers

- NONE at dispatch time. Registry 3.0.2 and current upstream source were directly verified during Analyze.

## Reflection handoff

Reflection consumes the four change verification files, QA/diff review receipts,
the 3.0.2 negative-control output, upstream PR/issue link, installed-service
browser/network/server evidence, plan-versus-delivery delta, and any stop
condition encountered. It must report UAR and upstream results separately.

EXECUTION READY
