# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-16-assistant-capability-surface, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract, dot-06-fresh-process-resume, dot-07-approval-recovery, dot-08-durable-ingestion, dot-10-durable-budget-admission, dot-11-supervisor-fencing, dot-13-memory-controls, dot-14-durable-schedules, dot-15-proactive-followups. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement expose precise supported assistant operations. Expose the six contracts through authenticated server/sidecar HTTP management and supported MCP/native assistant tools using the same host services. Publish per-operation capability descriptors for memory controls, schedule CRUD/pause/cancel, follow-up policy/inbox, tool outcomes, approval reads/decisions and checkpoint recovery. Postgres/in-memory/unattested remote and direct embedded Runtime must explicitly identify unsupported operations; no silent fallback. Provide API examples for consumers, not a new assistant UI. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Run the next task's real-path acceptance only after the full expanded Dot production implementation inventory is complete.

## 2. Production-boundary acceptance

- [ ] 2.1 Verify expose precise supported assistant operations through production boundaries. Generate an operation-by-profile inventory from actual registrations and exercise every advertised operation through real entry points. Existing HTTP Rust client can wrap APIs; do not certify absent direct embedded APIs. Assert unsupported requests fail explicitly and shared/local owners receive identical contract semantics.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
