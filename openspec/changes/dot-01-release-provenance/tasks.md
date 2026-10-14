# Tasks

Assignments: see the Task model assignments table in ../../../../.kbd-orchestrator/phases/dot-feature-support/plan.md, scoped by phase dot-feature-support, change dot-01-release-provenance, and backend ordinals 1–2. Dotted labels below are display labels; kbd-apply list ordinals are canonical task IDs.

Prerequisites: none. No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

Workflow/bootstrap: use KBD begin/end task ownership, read applicable AGENTS.md and skills, verify current source and route, and preserve unrelated work before editing. No workflow tool integration changes are requested here; if scope changes, validate Codex, Claude Code, Cursor and OpenCode integrations explicitly.

Current Dot scheduling (2026-10-12): These task labels remain unchanged, but future executable verification is deferred until the lead records an explicit inventory that **every production implementation item in the expanded Dot phase, including changes 18–20 and the Dot 17 runner, is complete**. Before that boundary, use source/document inspection and author tests or runners without executing them. Do not run tests, lint, format, typecheck, compiler probes, builds, policy validators, QA detectors, real-path probes, or product review gates at a per-edit, per-task, or per-change boundary. After the inventory, run the planned production integration and customer-build gates serially. Checked 01–06 receipts are historical evidence for their earlier source, not acceptance of the final combined revision. This scheduling note supersedes earlier Tier 0 and completed-change timing language below; it changes no completion checkbox.

## 1. Complete capability slice

- [x] 1.1 Reconcile intended release inputs and implement approved locked packaging. Prepare an exact intended-versus-effective pin reconciliation for the operator; do not edit versions.toml. Once policy resolution is recorded, align source/lock/build inputs, retain frozen pnpm installs and enforce locked Cargo resolution without redoing landed fixes. Preserve embedded schema inclusion and deployment-only workflow policy. Author integration fixtures and operator/API documentation. Use source inspection only; run acceptance after every production item in the expanded Dot phase is implemented. Historical earlier acceptance remains tied to its original revision.

## 2. Production-boundary acceptance

- [x] 2.1 Verify clean source closure and packaging provenance locally. From a clean recorded checkout, demonstrate frozen dependency closure and locked production packaging. Capture source/gitlinks and effective skill provenance. Publication/health of the final phase revision belongs to change 17; the existing digests prove only the analyzed revision.

## Workflow follow-up

- Reflect and archive only after required product evidence and independent review pass; do not treat file existence as completed implementation.

