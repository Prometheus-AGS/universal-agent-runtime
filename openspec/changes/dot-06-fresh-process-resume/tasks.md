# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-06-fresh-process-resume, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: dot-02-principal-credentials, dot-03-durable-api-keys, dot-04-truthful-tool-outcomes, dot-05-stream-terminal-contract. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement restore authorized runs from durable checkpoints. Persist a complete owner-bound resume envelope atomically with the checkpoint and load it before live-manager lookup. Reconstruct through host admission using protected history/current bindings and explicit host secret/resource reattachment. Replace persistence no-op defaults with unsupported errors; never deserialize runnable authority from caller input. Before reconstruction can execute, acquire a durable CAS claim bound to source run, checkpoint revision and executor incarnation; duplicate resume returns the same continuation identity or an explicit conflict, never new execution. Reuse the existing epoch/ownership vocabulary. A predecessor that may still execute or has uncertain effects remains blocked until trusted exclusion/reconciliation; change 11 supplies abrupt-owner exclusion, so 06 alone does not certify abrupt takeover. Author integration fixtures and operator/API documentation. Use source inspection only; run acceptance after every production item in the expanded Dot phase is implemented. Historical earlier acceptance remains tied to its original revision.

## 2. Production-boundary acceptance

- [x] 2.1 Verify restore authorized runs from durable checkpoints through production boundaries. After durable proof of orderly predecessor shutdown and joined descendants, launch a new manager against the same durable backend, list/resume the checkpoint and verify authorized continuation. Abrupt-loss cases must deny takeover at this change boundary; successful killed-owner takeover is verified in 11. Deny cross-owner, altered artifact/bindings and missing host resources; explicit unsupported on Postgres until implemented, with zero execution. Race authorized resume requests from two separate runtimes and prove one continuation/effect. Kill the claimant before and after effect dispatch; a second claimant cannot replay uncertain work.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.

