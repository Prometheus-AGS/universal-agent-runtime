# Implementation roadmap — approved design, runtime work pending

This is a future implementation backlog, not authorization to modify production code in the specification child. Each phase has one implementation owner and one integration gate after ALL planned production behavior in that phase is complete. No unit/per-edit/mock-only/partial verification loops. Fix gate failures and rerun only the failed gate. Runtime and UX production roles may work in parallel only with explicit file ownership; verifier/reviewer roles remain dormant until the complete phase boundary.

## Readiness checkpoint

Before runtime execution, consume accepted D-UAR-P1 with exact UAR/Boss/mini/payload SHAs and installed Windows x64 + Mac ARM64 receipts. Refresh retained worktrees to the recorded merge checkpoint through a separately owned integration step; do not start on stale convergence branches. Preserve dirty state. Specify exact paths and acceptance owner in each repository-scoped KBD/OpenSpec child. No dependency on accepting the entire 18-change parent roadmap.

## I1 — Definitions and catalog

Owner: UAR compiler/catalog implementation role. Adapter owners: full and mini skill-pack roles. libraries cand-001/cand-005. Covers C03-local/C15-local, G04–G06/G11/G12.

Implement immutable normalized descriptors and resolved locks; TeamDefinition and bounded subteam DAG; skill ID/version/required/config and all five v2 fields preserved or explicitly rejected at activation; field-level legacy migration/export diagnostics. Reuse existing catalog CAS and descriptor persistence. Schema/version adoption is additive. Persist original descriptor; do not discard extension fields or silently claim unsupported semantics.

One completed-phase gate: actual compile → catalog registration → retrieval → installed binding → ordinary single-agent use, plus team validation/preflight. Demonstrate required unsupported semantics refused; legacy agents remain usable; full/mini round-trip retains supported meaning. Team execution is not claimed before I2. Migration receipt records original/resolved hashes and downgrade support.

## I2 — Durable local teams

Owner: UAR runtime/data role, governance co-owner for real effect boundary. libraries cand-001/cand-002/cand-003. Depends I1 and D-UAR-P1. Covers C06-local/C09-local/G01–G03/G07.

Implement team/agent instances, transactional claims and epochs, tasks/attempts/inbox/outbox, serialized member turns, bounded nested delegation, context selections, direct messages and authorized broadcast, budget reservation/settlement and fair queue admission. Existing admitted kernel runs own execution. Add administrative API and event projection. Persist pending approval intent; fresh authority on restart. Unknown effects enter reconciliation. No new daemon, workflow engine or replacement model loop.

Storage decision before implementation: identify selected existing backend's actual transactions/CAS/uniqueness behavior. Record commit/dispatch recovery boundaries, migration and backup policy. Durable profile must refuse incompatible backend capabilities. No backend upgrade in this docs phase.

One completed-phase gate: real provider/tool work with two isolated teams/workspaces, bounded subteams, task dependencies, concurrent messages, stale claim rejection, disconnect/restart and pending approval recovery, cancellation while tool runs, duplicate delivery, budget exhaustion and uncertain external effect. Measure proposed capacity/latency targets from runtime.md on the documented host. Provider stalls may not occupy the control-plane queue. Evidence includes task/attempt/effect IDs and no private content leakage.

## I3 — Team protocols and Boss experience

Owner: UAR API/protocol role; Boss desktop/UX roles own their adapters only. library cand-004. Depends I2. Covers C14-local/G08–G10.

Implement AG-UI run segments/team events, negotiated attribution and replay/snapshots; A2UI surface ownership/action routing; A2A released-version team facade over the SAME task service. Preserve legacy endpoints. Boss Teams admin and conversation targeting expose state, assignments, model choices, approved capabilities, effective limits, progress, approvals, cancellation and recovery. Update typed IPC, settings sources/generated schemas and every locale. Persist domain data in UAR; preferences hold UI selection and connection settings only.

One completed-phase gate: ordinary and enhanced AG-UI/A2A clients interact with same tasks; two teams emit simultaneous text/artifacts/surfaces; authorized actions resolve to correct team/task/member; stale or cross-workspace actions fail visibly; reconnect and retention-gap recovery; accurate terminal run behavior. Real Boss integration exercises model selection, translation coverage and persisted settings. No second task board or approval authority.

## I4 — Complete workflows and customer release

Owners: full/mini template roles, Boss product/release role, UAR workflow support role. Depends I1–I3 plus accepted payload checkpoints. Covers C10-local/C15-local and C16-local templates.

Development story: parallel workers implement scoped production tasks; bounded design subteam contributes artifacts; integration role waits until the complete implementation set is done. Feedback story: intake/product/critic produce an issue proposal; operator approves the precise issue creation via supported connector; ambiguous provider result is reconciled before any retry. First connector: GitHub issue creation, subject to implementation inventory confirming the existing approved route; if absent, implementing that narrow adapter is INCLUDED in I4, not deferred silently. No requirement for new Slack/JIRA/Notion adapters.

Freeze canonical schemas/catalog/skills/runtime dependency closure and versions. Build and publish Windows x64 and Mac ARM64 as equal customer priorities directly to GitHub Releases; update each available platform's manifest/site immediately. Verify produced/downloaded bytes, source pins, signing status. No IPFS.

One complete release acceptance boundary, with separate installed evidence on both platforms: both workflows, ordinary agents, UAR lifecycle/port settings, existing service/MCP integrations, two workspaces, restart, approvals and recovery. Artifact build success alone is insufficient. Follow existing release permissions and user-approved outbound connector actions; sample feedback does not grant arbitrary publication.

## I5 — Later federation and business integration

Owner mapping retained in dependency-map.json. UAR owns remote execution/fencing; librefang owns delegated BossFang workflow integration; Fabric owns transport; Gate owns resource-side identity/effect policy; Forge owns business state; memory owns scoped retrieval; KnowMe owns its clients/embedding; full/mini own external harness adapters.

Keep full C05/C07/C08/C10/C11/C12/C13/C14/C15/C16/C17/C18 acceptance open. Runtime implementations for marketing/design/executive representation, remote members, third-party harness execution and mobile sync are later. Each gets separate requirements/grants and its own complete integration gate. No distributed ownership or exactly-once side-effect claim follows from local-team success.

## Cross-phase scenario register

| ID | Scenario | First complete gate | Owner |
|---|---|---|---|
| A01 | Required definition fields survive catalog/binding; legacy migration reports loss | I1 | UAR + packs |
| A02 | Parallel workers, bounded subteams, integration waits for all production dependencies | I2 runtime; I4 full story | UAR/Boss |
| A03 | Two teams/workspaces, direct/broadcast context isolation | I2 | UAR governance |
| A04 | Restart, mailbox replay, pending approval, stale ownership and uncertain effects | I2 | UAR data/runtime |
| A05 | Fair admission, aggregate budgets, cancellation, duplicate commands | I2 | UAR runtime |
| A06 | Concurrent output, surface action correlation, snapshots/reconnect | I3 | UAR/Boss |
| A07 | Ordinary protocol clients and single-agent compatibility | I3 | UAR protocols |
| A08 | Feedback→product/critic→explicitly approved issue, no duplicate effect | I4 | Boss/UAR/packs |
| A09 | Installed Windows x64 and Mac ARM64, exact payload/site artifacts | I4 | Release + operator |

Validation may overlap NEXT phase's unrelated production work only after its dependency contract is available and write/build resources are isolated. It never retroactively marks the earlier phase accepted.
