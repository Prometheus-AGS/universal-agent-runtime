# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-15-proactive-followups, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-13-memory-controls, dot-14-durable-schedules. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement deliver governed follow-ups from durable events. Persist opt-in rules bound to owner/source/grant and authorized conversation. Consume durable occurrences and commit admission/suppression reasons plus delivery outbox. First-release destination is a durable owner conversation inbox; existing external connectors are allowed only when individually declared supported with idempotency/reconciliation and approval policy. No new email/push connectors. Recheck quiet hours, consent, cancellation and current destination authority before dispatch. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Run the next task's real-path acceptance only after the full expanded Dot production implementation inventory is complete.

## 2. Production-boundary acceptance

- [ ] 2.1 Verify deliver governed follow-ups from durable events through production boundaries. Crash at rule evaluation, decision commit and delivery acknowledgement; race scanners and revoke destination access. Prove one logical inbox item, visible suppression, quiet-hour deferral and unknown external outcome held for reconciliation. No universal exactly-once human-receipt claim.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.

