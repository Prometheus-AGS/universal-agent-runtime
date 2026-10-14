# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-02-principal-credentials, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: none. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Implement isolate credentials by complete verified principal. Introduce a host-derived versioned issuer/tenant/subject key in a namespace disjoint from legacy subjects, including explicit local issuer namespace. Carry it through create/list/read/delete/resolve/use and caches on SurrealDB and Postgres. Quarantine ambiguous legacy rows; provide auditable authenticated mapping with conflict detection and resumable migration, preserving ciphertext and scope precedence. Author integration fixtures and operator/API documentation. Use source inspection only; run acceptance after every production item in the expanded Dot phase is implemented. Historical earlier acceptance remains tied to its original revision.

## 2. Production-boundary acceptance

- [x] 2.1 Verify isolate credentials by complete verified principal through production boundaries. Exercise the real credential API and provider resolution with equal subjects across tenants/issuers, raw subjects matching encoded keys, direct IDs, rotation/deletion and two processes. Prove mapped migration is idempotent and unmapped rows cannot be claimed.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.



## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
