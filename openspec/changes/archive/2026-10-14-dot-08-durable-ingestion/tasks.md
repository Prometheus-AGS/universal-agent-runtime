# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-08-durable-ingestion, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-02-principal-credentials. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement commit upload content and ingestion work together. Persist immutable source bytes and owner-scoped job metadata in one supported-backend transaction before 202. Workers discover committed jobs independently of the in-memory queue, claim idempotent document/chunk work and expose durable failed/retryable states. Use the database initially; do not add an object store. Preserve upload limits and tenant authorization. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Run the next task's real-path acceptance only after the full expanded Dot production implementation inventory is complete.

## 2. Production-boundary acceptance

- [ ] 2.1 Verify commit upload content and ingestion work together through production boundaries. Crash after acceptance, before dispatch and during indexing; restart workers and prove recoverable bytes plus one logical job/chunk set. Failed commit returns no acceptance; no-worker acceptance is permitted only with a durable discoverable job. Verify delete/cancel prevents later ingestion.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
