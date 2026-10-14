# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-14-durable-schedules, and backend ordinals 1–3. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-10-durable-budget-admission, dot-11-supervisor-fencing. The operator resolved busy-time behavior: latest-pending coalescing under a durable owner/schedule guard spanning revisions. All three tasks use design.md's Busy-time coalescing contract and running/approval/unknown/restart/revision scenarios. Task 1.1's requested policy resolution is recorded; timing fit and runtime evidence remain outstanding.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Timing qualification

- [x] 1.1 Inspect the exact-version timing adapter and record adoption or return-to-analysis decision. Deliver a source-and-current-documentation fit report covering the documented recurrence forms, DST, cancellation, restart and lack of duplicate authoritative storage; reject adoption and return to Analyze if source/docs cannot support the design. The busy-time overlap contract is already recorded above. Defer executable timing fit to task 3.1 after the full expanded Dot production implementation inventory.

## 2. Complete capability slice

- [x] 2.1 Implement durable schedule management and governed occurrence admission using the task 1.1 source/docs timing decision, without making the adapter's storage authoritative. Implement owner-scoped one-shot UTC and recurring calendar schedules, IANA timezone, revision/cancel/pause and atomic cursor-plus-occurrence admission. Default no overlap; coalesce backlog to the latest due occurrence with recorded skipped range, skip nonexistent DST local times and select the earlier repeated instant. Current authority, budget and approval apply on every admission. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Defer real-path acceptance to task 3.1 after the full expanded Dot production implementation inventory.

## 3. Production-boundary acceptance

- [ ] 3.1 After the full expanded Dot production implementation inventory, verify schedule semantics through real restart and competing-runtime paths. First prove timing fit (DST, cancel, downtime, no duplicate authoritative state); failure returns timing choice to Analyze and requires a plan revision, not an early probe. Then race two scanners, crash at cursor/admission, restart, pause/cancel and change schedule revision. Verify one logical occurrence and receipt, truthful skipped/coalesced state and no replay of uncertain work.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
