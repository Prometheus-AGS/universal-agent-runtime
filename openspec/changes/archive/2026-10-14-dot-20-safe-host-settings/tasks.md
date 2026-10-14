# Tasks

Assignments: parent Dot plan, `dot-20-safe-host-settings`, canonical backend task IDs 1–3, registered on 2026-10-12. Dotted checklist labels are display labels. Dot-03 production auth contract is prerequisite. Task 1 may proceed under exclusive server/config ownership; task 2 follows dot-16 production capability inventory and Kubo settings contract. The lead serializes `src/server.rs` with dot-18/19 and all other writers.

## 1. Host default and CORS implementation

- [x] 1.1 Resolve actual configuration keys, installable profile overrides and no-implicit-model behavior against current source. Implement explicit origin allowlist with empty default, local/off memory embedding, loopback native bind and JWT-enabled shipped configurations while preserving saved operator settings. Document shared-host overrides. Author fixtures without executing them and hand off changed files/unverified behavior. **Type: implementation.**

## 2. Effective settings implementation

- [x] 2.1 Pin the path, Kubo acceptance schema/translation, aggregate revision/hash policy and existing admin JWT claim shape. Implement one secret-free read-only effective view from live registry, persisted settings synchronization and governance/capability authority; require Bearer admin alone with 401/403 semantics. Preserve existing mutation ownership. Author docs and fixtures without running them. **Type: implementation.**

## 3. Deferred production verification

- [ ] 3.1 After the lead records the full Dot implementation-complete inventory, exercise default/override CORS, installed sockets/egress/auth/model/embedder behavior and effective Bearer-admin schema/revision/secrecy through actual local and shared profiles. Dot-17 checks the published installed artifact. Record exact commands, output and source/artifact identities. **Type: verification; blocked until full-phase boundary.**

## Workflow follow-up

Use KBD typed task lifecycle for this registered change. No OpenSpec validation, product test, build, lint, format, typecheck, QA detector or real-path probe before the complete combined Dot implementation boundary. Tool workflow integration is unchanged; no Codex/Claude Code/Cursor/OpenCode refresh is requested. For the required native installer model-selection fixes, the latest direct full-phase no-testing instruction defers the mandatory workflow-policy validator to the final boundary. The validator and deployment-only Actions policy remain required.

Sequencing clarification (2026-10-12): task2 composes the existing live provider/configuration/governance authorities after task1; it does not consume new assistant capability descriptors. Dot16 inventories this endpoint once implemented. Removing that false source prerequisite allows parallel work without reducing the response contract or full-phase acceptance. Pinned Kubo schema: `d5dea2115ad59ed7b12df561e5b085b562df8674`, mcp/contract/agentic/uar-effective-settings.schema.json.


## Operator closure — 2026-10-14

Production implementation is committed on main. Remaining unchecked verification tasks were cancelled by explicit operator instruction and remain unchecked; no new passing result is asserted. This change is archived and does not block consumers.
