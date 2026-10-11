# Design: exact approval and session diagnostic

## Context

Child project UUID: 06d6112e-346a-4417-8296-3b0ba3dba220. Product root: /Users/gqadonis/.claude/worktrees/bauar-uar. Accepted product base: a7cb972992d4f83db6585449ea81af0fe4a1c990. Parent phase: bossfang-uar-authorization-and-execution. Only this nested workstream owns the child KBD state. Original repository KBD state and C05 delivery gates remain untouched.

Exact source inventory, dependency closure, contracts and diagnostic protocol: [UAR prerequisites](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/.kbd-orchestrator/phases/bossfang-uar-authorization-and-execution/evidence/execute/uar-prerequisites.md).

The reviewed [parent design](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/openspec/changes/bauar-02-execution-authorization/design.md) is incorporated with one resolved decision: the user chose strict coordinated owned-client cutover; no compatibility exception is authorized. Parent requirements/scenarios are copied without omission.

## Goals / Non-Goals

Require exact pending invocation identity and establish whether HTTP session ownership is enforced. Preserve claim-before-effect and full-harness expected task revision. No speculative rmcp upgrade, SDK fork, old-binary evidence or JWT-disabled fixture.

## Decisions

D0 executes first on unchanged source using new tests/bauar_session_owner.rs and the existing production start_server_with_listeners_and_shutdowns entry. The prerequisite receipt supplies exact protocol, synthetic issuer, A/B positive controls, private state and POST/GET/replay/DELETE trace requirements. Ordinary JWT validity alone cannot close F6; record actual enforcing branch or reproduced crossover. Current LocalSessionManager wiring has no UAR-level owner comparison visible in source; this is not runtime proof.

After identity production and the separately owned client migration contract, require exact approval ID in both src/uar/api/routes.rs and src/server.rs api_run_approval, actor messages, manager and thread broker. Reject missing/stale/foreign/replayed/cancelled decisions without consuming legitimate pending state. Preserve expected revision in full-harness/handlers.rs, which remains read-only.

Exact existing production claims: src/server.rs (api_run_approval slice after identity handoff), src/uar/api/routes.rs, src/uar/runtime/actor/messages.rs, src/uar/runtime/manager.rs, src/uar/runtime/thread/approvals.rs, src/uar/runtime/tool_admission/mod.rs, src/uar/runtime/tool_admission/lifecycle.rs. Existing helper claims: tests/remote_mcp_run_grants.rs, tests/support/principal_host.rs, tests/support/sidecar_process.rs, tests/integration/live/agent_instance_cases.rs, tests/sidecar_session_principal/isolation.rs, tests/run_retention/runs.rs, tests/tool_admission_integration.rs. New diagnostic: tests/bauar_session_owner.rs. Conditional production claim after reproduced D0 only: src/uar/mcp_server.rs; proposed cohesive new src/uar/mcp_server/session_owner.rs if required. New docs: docs/operations/execution-authorization.md. Identity owns tests/bauar_authorization.rs; extend scenarios only after explicit handoff.

## Risks / Trade-offs

Uncomfortable constraint: valid-looking protocol errors can hide missing session authorization. Record successful A/B controls and real replay cursor; 400/406/500 are not acceptance denials. Cross-component Boss/client delivery and dependency closure remain driver-owned gates. Cancellation cannot undo an external effect; unknown remains unknown.

## Migration Plan

Driver registers tasks then begins D0 before fixture writes. No other product mutation before D0. If crossover reproduces, retain receipt then implement smallest owner binding across every method. Strict client/server cutover proceeds with no run-only fallback. Run V1 once after all selected production; V2 local checks/builds once. Do not complete a coordinating checkbox solely because code was authored.
