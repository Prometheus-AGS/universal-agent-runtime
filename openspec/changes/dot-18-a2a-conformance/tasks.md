# Tasks

Assignments: parent Dot plan, `dot-18-a2a-conformance`, canonical backend task IDs 1–2, registered on 2026-10-12. Dotted checklist labels are display labels. Prerequisite production contracts: dot-03 durable API keys and dot-06 fresh-process resume; dot-19 depends on the selected A2A contract. The lead assigns one A2A writer and serializes the `src/server.rs` handoff.

## 1. Contract and production implementation

- [x] 1.1 Pin the authoritative A2A 1.0 card/task/method/part/role/state/auth-error schemas and legacy compatibility disposition. Implement the anonymous public card, authenticated 1.0 task surface, durable owner-bound task/thread correlation with defined atomic submission/restart transitions, and card version from actual package/release provenance. Reuse `A2AThreadService`, actor execution and host admission. Author documentation and conformance fixtures without running them. Record files changed, contracts selected and unverified behavior in the phase handoff. **Type: implementation.**

## 2. Deferred production verification

- [ ] 2.1 After the lead records the full Dot implementation-complete inventory, exercise the pinned 1.0 card/tasks, auth/errors, restart and competing-runtime correlation through shipped paths; dot-17 separately proves the version against the actual installed published artifact in both profiles. Preserve exact commands, output and source/artifact identities. **Type: verification; blocked until full-phase boundary.**

## Workflow follow-up

Use KBD typed task lifecycle for this registered change. Do not run OpenSpec validation, product tests, lint, format, typecheck, builds, QA detectors or real-path probes before the full combined Dot implementation boundary. Tool workflow integration is unchanged; no Codex/Claude Code/Cursor/OpenCode refresh is requested.
