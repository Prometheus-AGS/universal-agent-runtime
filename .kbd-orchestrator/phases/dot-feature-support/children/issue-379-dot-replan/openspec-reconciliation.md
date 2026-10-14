# OpenSpec reconciliation handoff — issue #379 in current Dot release

2026-10-12. Child Plan handoff for the lead. The three proposed OpenSpec changes are authored as planning artifacts. This file does not mark KBD tasks complete, validate OpenSpec, implement production code or certify a release. Existing 01–17 changes and canonical projections were not edited here.

## Files authored

- `openspec/changes/dot-18-a2a-conformance/`: `.openspec.yaml`, `proposal.md`, `design.md`, `tasks.md`, `specs/a2a-grpc/spec.md`. Covers issue #379 items 1, 2 and 9 through the existing A2A capability, with the old 0.3 gRPC binding explicitly unadvertised as 1.0 until migrated.
- `openspec/changes/dot-19-interactive-protocols/`: `.openspec.yaml`, `proposal.md`, `design.md`, `tasks.md`, and deltas for `tool-approval-workflow`, `runtime-console-governance-certification`, `ag-ui-chat-conformance`, and `mcp-runtime-projection`. Covers items 3, 4 and 6. The governance-Off requirement is MODIFIED in full to retain risk-policy bypass while preserving explicit descriptor Required approval; MCP elicitation is a distinct interaction response, never tool authorization.
- `openspec/changes/dot-20-safe-host-settings/`: `.openspec.yaml`, `proposal.md`, `design.md`, `tasks.md`, and deltas for `native-service-deployment`, `provider-model-settings-certification`, and `jwt-hardening`. Covers items 5, 7 and 8 using existing capabilities.

Each change has at least one spec delta. The final dot-17 acceptance in the parent plan remains responsible for all nine against the actual installed published artifact in both local and shared profiles. Items 1, 8 and 9 are Kubo's reader minimum, not the Topology B release gate.

## Exact proposed task import

| Change | Backend ordinal / display | Title for canonical import | Type |
|---|---|---|---|
| dot-18-a2a-conformance | 1 / 1.1 | Pin A2A contract and implement card, task and version production behavior | Implementation |
| dot-18-a2a-conformance | 2 / 2.1 | Verify A2A 1.0 and published-version behavior after the full-phase boundary | Verification, deferred |
| dot-19-interactive-protocols | 1 / 1.1 | Pin interaction contracts and implement approval/AG-UI production behavior | Implementation |
| dot-19-interactive-protocols | 2 / 2.1 | Implement MCP elicitation against the same host-owned interaction state | Implementation |
| dot-19-interactive-protocols | 3 / 3.1 | Verify interactive protocols after the full-phase boundary | Verification, deferred |
| dot-20-safe-host-settings | 1 / 1.1 | Resolve host defaults policy and implement CORS/shipped profiles | Implementation |
| dot-20-safe-host-settings | 2 / 2.1 | Pin settings contract and implement the live effective aggregate plus Bearer admin route | Implementation |
| dot-20-safe-host-settings | 3 / 3.1 | Verify host settings after the full-phase boundary | Verification, deferred |

The parent plan's assignment table carries these ordinals. Formal task IDs are assigned only through the typed KBD import. Production successors depend on implemented prerequisite contracts; their predecessors' verification tasks remain pending until the full combined Dot implementation inventory is recorded. If KBD requires predecessor change DONE, the lead must revise canonical task dependencies without using early verification to unlock work. Historical 01–06 receipts remain historical; 07 verification stays pending.

## Contracts to settle in production tasks

1. `dot-18` task 1 pins authoritative A2A 1.0 card, methods, roles, parts, states and auth-error schemas; records legacy HTTP/gRPC compatibility and task/thread atomic binding design. Card version comes from actual release provenance; no tag is selected here.
2. `dot-19` task 1 pins AG-UI `RunAgentInput`, SSE interruption, `RUN_FINISHED` outcome and `resume[]` mapping; task 2 pins MCP elicitation capability/request/response. Both reuse the existing host approval/continuation authority. The implementation must audit current dirty source for a duplicate authority before adding one; the child architecture review did not complete that whole-tree classification.
3. `dot-20` task 1 confirms actual configuration keys and installable default/override policy. Task 2 pins one effective-settings path and Kubo schema/translation, aggregate revision/hash policy, admin JWT claim shape and local/off embedder mapping. Recommended path is `GET /api/uar/settings/effective`; it is not asserted as an existing route.

## Ownership and verification boundary

Only one writer at a time owns `src/server.rs`, `src/uar/runtime/manager.rs`, `src/mcp/registry.rs`, shared persistence/migrations or a build target. Writers share the checkout and preserve others' edits. The lead assigns file ownership and receives a handoff listing files changed, selected contracts, unresolved decisions and unverified behavior. No tests, builds, lint, format, typecheck, compiler probes, validators, QA detectors or real-path probes run until every combined Dot production behavior, migration, operational setting, required document and dot-17 runner is complete and inventoried. The historical 01–06 checks do not certify new source. If a future workflow or invoked build/package/deploy script edit is needed, the mandatory pre-edit GitHub Actions policy validator conflicts with this boundary; the lead must avoid the edit or request the operator's exact-command early-verification exception before editing.

## Current state

Source architecture analysis and its M3/K3 review are recorded in the child `analysis.md` and `review/architecture/review-log.md`. K3 passed the corrected architecture artifact, with the review log preserving M3's original findings and the stated limits. No product checks were run for this reconciliation. OpenSpec validation and canonical import remain pending; these documents are ready for the lead's contract/task reconciliation, not evidence of implementation or runtime conformance.

## Lead registration receipt

Canonical registration completed: changes18–20, eight associated tasks, and Dot17 runner task3. Existing verification remains unpassed; Dot07 task2 is blocked by the full-phase boundary. The immutable sequencing decision references `../../implementation-sequencing.json`; exact commands and observed exit codes are in `canonical-reconciliation-receipt.json`. OpenSpec semantic validation remains deferred; registration is not product verification.
