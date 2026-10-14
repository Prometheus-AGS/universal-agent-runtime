# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-13-memory-controls, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-12-memory-isolation, dot-08-durable-ingestion. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement persist capture consent retention and deletion. Add owner-visible durable capture policy, provenance and retention controls to MemoryService APIs; new assistant automatic capture defaults off until opt-in. Require explicit retention choice at enablement. Use tombstones to immediately suppress deleted/expired content across records/index/history/queued capture; idempotent purge completes without retaining content in audit logs. Preserve existing explicit user memory behavior. Author integration tests and operator/API documentation in the same production slice. Inspect source wiring and explicit state/error paths without executing checks. Run the next task's real-path acceptance only after the full expanded Dot production implementation inventory is complete.

## 2. Production-boundary acceptance

- [ ] 2.1 Verify persist capture consent retention and deletion through production boundaries. Opt-out/revoke during queued capture, expire during retrieval and crash during deletion/index cleanup; no deleted content is returned and purge resumes. Verify both processes honor policy revision, export/history redact removed content and legacy records are not silently assigned new consent.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
