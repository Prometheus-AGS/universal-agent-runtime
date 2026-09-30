# Parent execution amendment — C09.3 A, planned C09.4 B

Documentation finalized 2026-09-30 under the operator-authorized documentation-only `uar-team-execution-architecture` child. C09.4 is approved planning scope; product dispatch waits for explicit parent execution. All added work remains pending. Existing C09.1/.2 receipts, original tasks and failed C09.3 receipts remain intact. This document does not certify runtime behavior or introduce a competing change.

Local provenance only: [approved child handoff](/Users/gqadonis/Projects/prometheus/worktrees/agent-fabric-c06/librefang/docs/plans/agent-fabric-convergence/.kbd-orchestrator/phases/agent-fabric-convergence/children/uar-team-execution-architecture/parent-repair-handoff.md), [execution contract](/Users/gqadonis/Projects/prometheus/worktrees/agent-fabric-c06/librefang/docs/plans/agent-fabric-convergence/.kbd-orchestrator/phases/agent-fabric-convergence/children/uar-team-execution-architecture/execution-profile-contract.md), and [field migration](/Users/gqadonis/Projects/prometheus/worktrees/agent-fabric-c06/librefang/docs/plans/agent-fabric-convergence/.kbd-orchestrator/phases/agent-fabric-convergence/children/uar-team-execution-architecture/legacy-migration.md). Product publication must carry a repository-owned migration document at `docs/agents/collaboration/team-execution-profile-migration.md` before release; that path is an assigned future output, not an existing deployed receipt. Implementation references below are repository-relative.

## A — complete existing C09.3

| Work | UAR claim / handoff |
|---|---|
| A1 Contract and migration | Provider/domain writer: registry, collaboration domain/schema, bindings/runtime semantics and provider API. Freeze route, wire alias, price, profile/settings, fit disposition, safe diagnostic and catalog claim DTOs; publish field migration. |
| A2 Exact model execution | Runtime/provider writer: team resolution, manager, orchestrator, Liter driver, dialect and error normalization. Preserve/integrate existing unbuilt candidate; attach actual leaf/profile/preparation, retain alias and profile capture on retries. |
| A3 Execution owner/recovery | State/runtime writer: catalog storage/service and admission/settlement/recovery, runtime controller/epoch/execution, request revalidator, server/API. Non-expiring generation-CAS claim and current-epoch checks; privileged evidenced reclaim; preserve known output with held unknown accounting. Serialize shared files with A1/A2. |
| A4/A5 Consumer contract/UI | Boss desktop/provider and renderer/i18n owners consume fixed typed DTOs, persist settings/rebind and expose safe ownership/error/context/usage feedback in all 13 locales; UAR stays sole state/execution owner. |
| A6 Payload and operation | Single release/build writer freezes exact source and packages UAR for Boss; one local Mac ARM64 build, launch and real completed A operation. All source/UI/IPC/locales/scenario/payload complete before this gate. |

Keep [catalog CAS](../../../src/uar/compiler/collaboration/storage.rs), [existing actor execution](../../../src/uar/runtime/team_execution/execution.rs) and [effect revalidator](../../../src/uar/runtime/turn/request.rs). No new scheduler, broker, lease timer, daemon, port or dependency. The preserved candidate is unbuilt/unoperated until intentionally integrated at the parent boundary.

The exact endpoint profile separates configured route identity from served wire alias and canonical pricing. Default reasoning off emits no unsolicited family/multi-turn `thinking`; explicit unsupported or unknown reasoning refuses before dispatch. Settings-only validation is labelled no guaranteed fit; synthetic fixture counting and price catalog identity do not establish real gateway framing/capacity. Known context/output metadata carries provenance; unknown capacity keeps conservative behavior. Settings saves and effective rebinds are revisioned; old attempt receipts remain immutable. Relevant seams: [preparation](../../../src/llm/orchestrator.rs), [profile leaf](../../../src/llm/liter_driver.rs), [registry](../../../src/llm/registry.rs).

Migration: nonempty unsupported AgentDefinition `/context/artifacts/{i}`, `/context/history` (`selected`/`authorized-summary`), `/context/memoryScopes/{i}`, `/context/mode` (`authorized-fork`), and binding `/contextGrants/{i}` refuse with `TEAM_CONTEXT_REQUIRED_UNSUPPORTED` and exact document/pointer diagnostics. Empty selections remain compatible, including an empty selected mode; no broad rejection of supported `/contextStrategy`. Legacy nonempty selections are required by default; optional exclusion requires the explicit next-schema representation and receipt. Required `/ragConfiguration` is already refused with its existing `runtime.component-unavailable` diagnostic. Unmapped `/permittedChildren/{i}` and nested `/members/{i}/kind: team` execution refuse with `TEAM_CAPABILITY_UNSUPPORTED`. Exact SkillRefs/bindings stay intact; attempt `contextArtifactIds` remains the supported artifact path. No automatic rewriting or private-grant export. See [canonical context shape](../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/common.schema.json), [semantic resolver](../../../src/uar/compiler/collaboration/runtime_semantics.rs) and [selected artifacts](../../../src/uar/compiler/collaboration/team_execution/scope.rs).

Dedicated catalog initially has one executing service/incarnation/epoch, held/draining/released. Admission, dispatch, recovery, effects and mutations use that fence. Crash has no expiry takeover. Reclaim requires authenticated privileged operator authorization, expected epoch, reason and verified evidence that the old executor and children stopped or are externally fenced; model/tool credentials cannot authorize it. Transfer only queued authority with audit receipt; old running attempts remain uncertain, never replayed. Unknown usage alone must not erase known task success/output or readiness. A record cannot fence an old binary that ignores it; remote qualification requires dedicated catalog and controlled versions/credentials.

Gate A operates actual alias marker/settings readback/restart, unsupported reasoning refusal, selected artifacts/isolation/stale authority/revocation/budget/deduplication/cancel/recovery, two current competing remote executors, confirmed clean release, crash/no takeover, authorized evidenced replacement, rejected unauthenticated/unauthorized/stale or insufficient-evidence replacement, queued transfer and running uncertainty. Selected local backend also operates before advertisement; other backends remain unqualified. Gate A is a named source/payload/build/launch/operation receipt owned by lead, not build-only proof.

## B — approved planning scope, dispatch after Gate A

B1 compiler/domain plus full/mini authoring owners add versioned optional shared instructions and bounded authorized roster/context receipts, with host/security → team → member → task precedence; task/messages/artifacts remain untrusted attributed data. B2 runtime/trust-tools exposes only `team_roster`, `team_send`, `team_delegate`, `team_wait` under attempt-derived identity and current directed edge/mode plus scope authorization before visibility/send/admission/result disclosure. `team_send` queues only. Delegation commits task/envelope/attempt/reservation/receipt in one catalog CAS; same command/different payload conflicts. Reverse reply/result edges are explicit, never inferred.

B3 state/runtime adds queued-versus-active accounting, catalog-order controller drain, typed kernel yield, confirmed root/child cleanup, durable all-target waits and exactly one linked fresh continuation attempt/run/root under current policy and budget. Yield intent alone does not release capacity. Unknown effects block unsafe continuation; unknown accounting remains reserved. B4 Boss/full/mini owns visible roster/messages/waits/blocked state, all locales and exact skill payloads. B5 release owner operates the completed one-slot coordinator→worker→continuation and crash/duplicate/isolation/control matrix.

Gate B includes allowed and **forbidden/revoked same-team directed edges**: valid member identity cannot disclose inbox, send or delegate over the latter; queue-only permission cannot start work. Operate all-target completion, failed/cancelled target outcomes, cyclic wait refusal and reassignment/revocation invalidating old wait authority. Repeated notifications/restart cannot duplicate continuation or blindly replay dispatched work. Ordinary root-local child tools retain their meanings; no broad inbox/broadcast/cross-team/timer/subteam feature is implied.

### Scenario: Packaged sidecar supervised outside The Boss

The first actual Gate B operation reached packaged UAR `READY` but failed The Boss's initial `instances.test` before inference: sidecar bootstrap overwrote explicit `UAR_SERVICE_INSTANCE__OWNERSHIP=external` with `managed`. The operation host owns this isolated process's launch/shutdown, so it is external to The Boss even though it uses the packaged sidecar executable.

- **WHEN** a trusted supervisor explicitly sets `UAR_SERVICE_INSTANCE__OWNERSHIP=external` before launching the packaged sidecar
- **THEN** bootstrap preserves that ownership and discovery reports external ownership for The Boss's unchanged compatibility check.
- **WHEN** the ownership environment variable is absent
- **THEN** sidecar bootstrap defaults to managed ownership, preserving ordinary The Boss-managed launches.
- **AND** loopback binding, forced local workspace location, launch-token authentication, stdin EOF shutdown and execution qualification gates remain unchanged.

This records the observed failure and intended correction; successful rebuilt negotiation and cooperating inference still require the actual completed-boundary operation receipt.

### Scenario: Starter binds to the selected catalog storage

After external connection and gateway selection succeeded, actual Gate B starter setup called The Boss's managed `ensureReady` path instead of using the selected external runtime. The external endpoint has no managed storage profile, so its catalog backend must come from runtime discovery rather than lifecycle ownership.

- **WHEN** an authenticated host reads `/api/v1/collaboration/capabilities`
- **THEN** `catalogStorage.backend` identifies the initialized catalog as `surrealdb`, `surrealkv`, `postgresql` or `memory`, without URLs, paths or credentials.
- **AND** The Boss uses the selected catalog's backend for starter bindings rather than starting a different managed runtime or treating external ownership as remote database storage.
- **AND** a remote SurrealDB connection does not claim its server's disk durability; the backend descriptor is separate from the binding's configured durability declaration and operation qualification.

Successful rebuilt starter setup remains pending the actual completed-boundary receipt.

## Release and limits

Finish the entire relevant source/DTO/UI/13-locale/payload increment before one designated build/operation boundary; review agents remain dormant until then. Local `pnpm build:mac:arm64`, packaged launch and actual feature receipts are required per product success. Rerun only the failed relevant boundary after an observed repair. Cadence iteration 4 continues without reset; next full publication remains delivery 5, four Mac/Windows installers/assets/checksums/source/architecture/signing/metadata/site links, no Linux. Child return is no successful delivery. Windows human installed acceptance remains separate.

Deferred: arbitrary KB/history/memory/grants to UAR context/compiler/memory C03/C09; permittedChildren/subteams to C09 extension; protocols to C14; workflow/connectors and Restate/Temporal comparison to C10; broad cross-harness export to C15; multi-host/backend expansion to C18; domain use cases to C16/C17. Lead owns parent canonical registration and dispatch. Final Plan reviewer BLOCK on pre-correction edge criteria remains immutable; corrected B2/B5 criteria are explicit here, not falsely described as independently passed.
