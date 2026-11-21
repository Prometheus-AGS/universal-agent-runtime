## Context

Source facts, re-read at the cited lines on 2026-11-19. Nothing was built, run or tested.

**Where the team is pinned.** `TeamInstance.binding` is a `TeamBindingRef { id, revision }` set once when the team is created (`compiler/collaboration/team_planning.rs:131-134`). No code assigns it afterwards (grep of `src` for writes to `team.binding`). Each attempt stores `binding_revision` (`domain/team_execution.rs:53`). Installing a binding revision appends to `binding_history` and replaces `bindings[...]` (`compiler/collaboration/bindings.rs:132-138`; history type `domain/collaboration.rs:476`).

**Every place a binding revision is compared in the team path.** This is wider than the single gate named in Build item 15.

| # | Site | Compares | Role today |
|---|---|---|---|
| G1 | `team_execution/resolution.rs:35-48` | current stored binding revision == `attempt.binding_revision`, package == team package | start and in-flight; error "Team deployment binding changed" |
| G2 | `team_execution/mod.rs:36` (`fence`, used by `revalidate_team_attempt` `mod.rs:158-165`, `settlement.rs:20`, `dispatch.rs:34`, `context.rs:43`, `scope.rs:54`) | current binding revision == `expected.binding_revision` | every fenced operation, including settlement of a running attempt |
| G3 | `team_execution/admission.rs:117-128` | current binding revision == `selected.binding.revision` (team pin) | new attempt; "reinstall the team binding" |
| G4 | `team_execution/continuations.rs:44-52` | team pin == `prior.binding_revision` and current == `prior.binding_revision` | wait validity; otherwise `TEAM_WAIT_INVALIDATED` |
| G5 | `compiler/collaboration/team_mailbox.rs:321` | `task.assignment_authority.binding_revision` == team pin | narrowed-scope mailbox check |
| G6 | `runtime/team_execution/host.rs:137-138, 192-195, 227-229` | host-context binding revision vs team pin / attempt revision | host workspace context, code `TEAM_REVISION_CONFLICT` / `TEAM_WORKSPACE_DENIED` |
| G7 | `workflow_execution/mod.rs:59-60`, `workflow_execution/start.rs:71,85` | team pin and current vs the workflow run's binding revision | workflow runs on a team |
| G8 | `team_execution/context.rs:194` | continuation authority revision == `attempt.binding_revision` | both values are attempt-captured; not affected by a binding edit |

**Every place a route is re-resolved.**

| # | Site | What it does |
|---|---|---|
| R1 | `resolution.rs:71-117` | per role: look the captured receipt up by route, call `resolve_team_model_settings` with the captured profile and settings revision, require JSON equality with the capture, then read the price identity from the live registry |
| R2 | `resolution.rs:118-124` -> `runtime_semantics.rs:48-176` | `modelRequirements` (tools, reasoning, vision, structured output, `minContext`, preferred provider) evaluated against the live registry's enabled provider and model |
| R3 | `runtime/team_execution/epoch.rs:7-25` | `resolve_team_member_run` again and require the whole effective receipt (minus `createdAt`, `contentDigest`) to be unchanged |
| R4 | `runtime/team_execution/controller.rs:236`, `execution.rs:263` | `resolve_team_member_run` at pre-dispatch and at turn start |
| R5 | `runtime/manager.rs` ~5970-5985 and ~6850-6900 -> `llm/team_profile.rs:114-140` | `capture_team_endpoint_profile` re-resolves the captured receipt and requires equality at run start; failover is disabled for team attempts (`manager.rs:5988-5989`, `6808-6809`) |
| R6 | `team_execution/model_capture.rs:6-35` | boundary capture used at admission (`admission.rs:251`), continuation (`continuations.rs:148,175`) and delegation (`peer_commands.rs:282`); a binding entry that carries `profile` or `settingsRevision` overrides the registry's current request (`model_capture.rs:23-30`) |

`resolve_team_model_settings` (`llm/team_profile.rs:62-112`) fails with `TEAM_ROUTE_PROFILE_MISMATCH` for a missing or disabled provider or model (`:65-69`), a profile mismatch (`:72`), a missing catalog price (`:82`), and with `TEAM_REVISION_CONFLICT` for a settings-revision mismatch (`:73-74`). `prepare_provider_update` bumps the settings revision when profile, reasoning or endpoint changes (`llm/registry.rs:178-184`) and enforces the editor's own compare-and-swap with the same code at `:176-177`; that second use is unrelated to teams and stays.

**What already makes in-flight safe.** After run start the model client and endpoint profile are captured once into `RunModelBindings` (`runtime/turn/bindings.rs:283-301`); a team attempt has no fallback models. The risk to an in-flight task is therefore only the revalidation calls in G2, G1 and R1 to R3, which run from `CollaborationRunBinding::revalidate_authority` (`runtime/turn/request.rs:33-48`).

**What the console already has.** `EffectiveTeamModelReceipt` is in the attempt contract (`frontend/src/platform/entities/teams/contracts.ts:437`) and `ModelLine` shows the first model of one attempt (`frontend/src/features/teams/ui/run/attempt-parts.tsx:12-30`). The console polls the team run every 3 s (`console-agent-teams/tasks.md` item 1.5).

## Decisions

**D1. A binding delta classifier.** Add `BindingCompat { Same, ModelOnly, Incompatible }` next to `binding_key` in `compiler/collaboration/bindings.rs`. Given `(owner, workspace, binding_id, from_revision, to_revision)` it returns `Same` for equal revisions. Otherwise it loads both documents from `binding_history` (`binding_revision_key`, `bindings.rs:988`). A missing revision is `Incompatible` (fail closed). It removes `revision`, `contentDigest`, `effectiveBindingReceiptRef` and the whole `modelBindings` array from both, and returns `ModelOnly` when the remainders are equal JSON values, otherwise `Incompatible`. Skill bindings, package, limits, budget, grants, storage, policy revision, extensions, version and provenance stay compared. `credentialRef` lives inside `modelBindings`, so it is part of the model change; no consumer of `modelBindings[].credentialRef` was found in the team path (`grep` in `src`, only validation at `bindings.rs:917` and the export fixture). **UNVERIFIED**: no other consumer exists; if one does, `credentialRef` must move to the compared set.

**D2. The task boundary.** The boundary is where an attempt's models become final: attempt creation (G3 admission, delegation, continuation) and the queued-to-running dispatch claim (`settlement.rs:13-118`, called from `controller.rs:287`). Admission captures provisional models so an unavailable route is refused before a task is queued. The claim re-captures, so a task that waited in the queue starts on the models current when it starts. After the claim the attempt is in flight.

**D3. Re-pin at the boundary.** When a boundary finds `ModelOnly` between the team pin and the current binding, it sets `team.binding.revision` to the current revision in the same catalog compare-and-swap that creates or claims the attempt, and increments `team.revision` and `updated_at` as other team mutations do (`peer_commands.rs:262`). The new attempt stores the current revision as `binding_revision`. No new endpoint, and no operation outside a boundary can re-pin. `Incompatible` keeps the existing refusal text unchanged.

**D4. Two resolution modes.** `resolve_team_member_run` gains `mode: TeamRouteMode { Boundary, Pinned }`.

- `Boundary` is used by `capture_team_models` callers and by the pre-dispatch check at `controller.rs:236` while the attempt is `queued`. It reads the current binding, resolves current routes (R6 with D5), evaluates `modelRequirements` against the new routes (R2), and compares nothing against an older capture.
- `Pinned` is used for every `running` or later attempt (R3 via `revalidate_member_binding`, R4 at `execution.rs:263`, G2 fences). It loads the binding document from `binding_history[attempt.binding_revision]`, requires the current binding to be `Same` or `ModelOnly` relative to it (G1), uses `attempt.effective_models` as the model facts including the price identity, and takes the `modelRequirements` outcome from the attempt's own admitted receipt instead of recomputing it from the live registry (R1, R2). Skill resolution still runs live, so a skill change still fails the stable-receipt compare at `epoch.rs:14-24`.

`capture_team_endpoint_profile` (R5) is unchanged. It runs once at run start. A route edit between the dispatch claim and run start fails visibly with `TEAM_ROUTE_PROFILE_MISMATCH`. That window is short and the failure is a visible, retryable refusal; it is accepted, not hidden.

**D5. Explicit pins are honoured.** `model_capture.rs:23-30` lets a binding entry pin a profile and settings revision. For an entry with a pin, a stale pin fails the boundary with `TEAM_REVISION_CONFLICT` (existing behavior of `resolve_team_model_settings`, `team_profile.rs:73-74`). For an entry without a pin, the boundary uses the registry's current request, so a route edit is adopted. The console builder writes no pin (`binding-builder.ts:60-90`), so console-created teams adopt route edits. A pin is the operator's explicit statement, and the cure for a stale pin is another model-only binding revision, which this change adopts at the next boundary. **Pending operator confirmation.**

**D6. Failure surface.** No new public error code.

- Admission and delegation: refuse with the existing conflict code from `capture_team_models`; no attempt is created; the task stays `ready`. The REST caller gets the 409.
- Dispatch claim: `claim_team_dispatch` returns the typed conflict. The controller then settles the still-queued attempt through the existing `settle_team_attempt` path used for pre-dispatch denial (`controller.rs:245-256`) with state reason `model_route_unavailable_at_boundary`, a `TeamExecutionDiagnostic` whose `code` is the existing code, `retryable` true and `action` naming the route to fix, and `TeamReservation::default()` usage. Today a failed claim other than `TEAM_PENDING_LIMIT` returns the error and leaves the attempt queued (`controller.rs:287-293`); this change settles it.
- Continuation: keep the existing blocked wait with reason `TEAM_ROUTE_PROFILE_MISMATCH` (`continuations.rs:148-156`) so the wait is visible, and add the same diagnostic fields.
- No fallback: team attempts already clear failover (`manager.rs:5988-5989`, `6808-6809`). The system default model never reaches a team member (`uiux-system-model-live-default`). The boundary never reuses the previous route.

**D7. Wait continuation is a boundary.** A continuation is a fresh attempt with a new `run_id`, `root_id` and execution epoch, built from selected artifacts and wait outcomes, not from the earlier run's conversation (`continuations.rs:167-191`). It already re-captures models at `:148`. It is treated as a boundary and adopts. **Pending operator confirmation**: the alternative is to carry `prior.effective_models` forward, which makes a waiting task end on the model it began with but fail whenever the route was edited meanwhile.

**D8. Provenance on the attempt.** Add `model_provenance: Option<TeamModelProvenance>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` to `TeamExecutionAttempt` (`domain/team_execution.rs:35-78`), which has no `deny_unknown_fields`. Do not touch `EffectiveTeamModelReceipt`, which does (`:198-199`).

```text
TeamModelProvenance {
  boundary: "admission" | "delegation" | "continuation" | "dispatch",
  bindingRevision: u64,
  comparedAttemptId: Option<String>,   // latest earlier attempt of the same member, by queueSequence
  roles: [ { requestedAlias, status: "initial"|"unchanged"|"changed",
             previous: Option<{providerId, modelId, settingsRevision}>,
             current: {providerId, modelId, settingsRevision},
             cause: Option<"member-model-rebind"|"route-settings-edit"|"price-change"> } ],
  capturedAt
}
```

`cause` is derived by comparing the previous route and settings revision and price identity with the current ones. At the dispatch claim the record is replaced only when the re-captured models differ from the admission capture; otherwise the admission record stands. The existing `effectiveModels` remains the authority for which route served the attempt. An attempt written before this change has no record and is shown as having none. Nothing is inferred.

**D9. What stays untouched.** Do not edit these `TEAM_REVISION_CONFLICT` or related sites: `dispatch.rs:32,43` (expected team revision on dispatch), `waits.rs:139,180,203`, `peer_commands.rs:47,128,185,337`, `context_evidence.rs:29,63,145`, `model_capture.rs:27` (a pin without a settings revision), `llm/registry.rs:177,183,190,197` and `llm/team_profile.rs:37,74` (provider settings editor and route settings), `api/providers.rs:285` (HTTP 409 mapping), `native_skills/team_tools.rs:231,266`, `llm/team_failure.rs:33-36` and `runtime/manager.rs:7118-7119` (code allowlists). The packet named four of these; the rest were found by grep and carry the same code for unrelated reasons.

**D10. Out of scope.** Workflow runs (G7) get the same classifier only where they compare the team pin to the run's recorded revision, so a re-pin by another task cannot invalidate an active workflow run; their own task attempts follow D2 and D4. How a workflow run records its binding revision is **UNVERIFIED** and is the first thing task 2.6 reads. No change to the pricing catalog (`catalog.rs:32` is a static singleton; a price change in one process means a reassigned pricing identity, which the `price-change` cause records). No change to cancellation. A disabled or removed route does not stop a task in flight; the operator stops it with the existing attempt cancel route. **Pending operator confirmation** that this is the intent of "a task in flight finishes on the model it started with" for a route disabled as a safety measure.

## Console contract

The console needs, per task: the model that ran, whether it changed from the member's previous attempt, and why a task failed at the boundary.

- `modelProvenance` is added to the attempt contract (`contracts.ts` `TeamExecutionAttempt`, optional). `ModelLine` shows the route of the attempt, and a "changed from X" note and the binding revision when `status` is `changed`.
- A task row shows the model of its latest attempt and, when a task has several attempts, the model per attempt in order.
- A boundary failure shows `stateReason` and `diagnostic.code` already exposed on the attempt.
- Push: the console keeps polling through `watchTeamRun` until the public team and binding topic from `uiux-realtime-topics-and-replica-refresh` exists. This change does not name that topic.

The console edit is owned by `uar-console` and needs a task-specific single-writer assignment from the lead.

## Frozen oracle fixtures

Frozen on 2026-11-19, before any implementation. The boundary runner reads these and nothing else for expected values. They live in this change and no implementer edits them; a change needs a lead-approved spec revision.

| File | Content |
|---|---|
| `fixtures/team-adoption-oracle.json` | Routes A, B and ABSENT, binding revisions 1, 2, 2-invalid (`policyRevision` changed), 2-absent, the route edit, HTTP status and code shapes, and the expected result and negative control for each of SCN-TEAM-001 to 008 |
| `fixtures/stub-a.fixtures.json`, `fixtures/stub-a.hold.fixtures.json`, `fixtures/stub-b.fixtures.json` | Fixture files for two `stub-llm` instances. Each member turn is one `team_roster` tool call then a final text (`stub-a-complete` or `stub-b-complete`). The hold variant delays the tool-call response 4000 ms so edits land between model start and tool claim |

Frozen facts the files rely on, each verified at source:

- `stub-llm` request log: `GET /_stub/requests` returns every request body in order and `POST` resets it (`tests/integration/live/stub_llm.rs:174-208`). Which instance received a request is the oracle for which route served it.
- HTTP shapes: a `CollaborationError::Conflict` returns 409 with `error.code` equal to the detail when the detail is a `TEAM_` code, otherwise `collaboration_revision_conflict` (`src/uar/api/collaboration.rs:489-519`). `NotFound` returns 404 `collaboration_not_found`.
- Tool: `team_roster` takes `{"limit": 1..50}` (`runtime/native_skills/team_tools.rs:66`) and is enabled by default (`domain/execution_capabilities.rs:29-31`, qualified stage).
- `UAR_TEAM_EXECUTION_MAX_ACTIVE` caps global active attempts (default 4, `settlement.rs:63-67`); at 1 a second attempt stays queued.
- `policyRevision` is a required non-model binding field (`bindings.rs:228-268`), so changing it is a real invalid delta.

## Prerequisites (named for the lead's build commands)

1. `stub-llm` binary: `cargo build --bin stub-llm --features server` (`Cargo.toml:127-130`). Invocation: `STUB_LLM_PORT=<port> stub-llm <fixture.json>` (`src/bin/stub-llm.rs:85-108`).
2. The `stub-llm` CLI fixture schema must be extended before these fixtures load. Today it matches only exact `last_user_message` with `has_tools`/`has_tool_result` (`src/bin/stub-llm.rs:27-36`), a team turn's user message is the serialized team context and is not freezable, and the CLI omits `DelayedContent` (`src/bin/stub-llm.rs:19-22`, though the shared module has it, `tests/integration/live/stub_llm.rs:38-42`). Required additions: `model: "*"`, `last_user_message_prefix` matched together with the tool flags, and an optional `delay_ms` on `tool_call`. This is task 7.1. The files are in the shared roots `src/bin/` and `tests/integration/live/`, outside `uar-runtime` ownership, so it needs a single-writer assignment from the lead. When delay support lands in the CLI, the `#[expect(dead_code)]` at `src/bin/stub-llm.rs:19-22` becomes unfulfilled and must be removed (clean-compile rule).
3. The `uar` server binary and, for SCN-TEAM-004, the built console bundle served by it.
4. Two provider registrations (`stub-team-a`, `stub-team-b`) pointing at the two stub instances, provisioned by the runner through the existing provider API with no secret value; the runner sends no credential or JWT, the external identity proxy supplies the owner.
5. The installed team package and bindings from the fixture file. Their authoring is part of task 7.2.
6. SCN-TEAM-006 needs a stub fixture whose `team_delegate` and `team_wait` arguments carry the live `teamRevision` and member ids. A static stub cannot supply them, so a template feature in the stub is required. UNVERIFIED whether it is worth building; if it is not provided, SCN-TEAM-006 is NOT_EXERCISED, not skipped silently.
7. No baseline worktree is needed by this change. The lead's runner-integrity gate (a pre-phase build must fail every mandatory scenario) uses a worktree under `~/.claude/worktrees/` per the project rule; scenarios SCN-TEAM-001 to 005 and 008 fail on the pre-phase build by construction (revision 2 admission is refused there).

## Acceptance

Shared declarations for every table below. Real collaborators: the shipped UAR server process, its real collaboration catalog store (embedded persistence in a scratch data directory), the real provider registry with two registered routes, and two `stub-llm` processes. `stub-llm` is the project's local model server, so no paid or hosted model is called; it is the real HTTP model endpoint the provider path talks to, and its own request log is the oracle. Tests never carry, mint or send credentials or JWTs; the existing external identity proxy supplies the verified owner and workspace (operator rule 2026-10-26). Expected values come only from `fixtures/` (R1). A mandatory scenario whose negative control did not run is BLOCKED, never PASS. Final local gate: the phase's single serialized production integration batch at the completed-implementation boundary; the lead records exact argv in the phase contract. Nothing here runs earlier.

### Behavior: adopt a member-model rebind at the next task boundary

| Field | Declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-team-model-adoption-at-boundary` / tasks 1.1-1.2, 2.1-2.3, 2.5, 4.1, 8.1; scenario SCN-TEAM-001 |
| Source identity | repository tree and built `uar` binary at the phase boundary; recorded by the lead |
| Production entry point | REST: `POST /api/v1/collaboration/deployment-bindings` (revision 2), `POST .../team-instances/{id}/tasks/{taskId}/admit`, `POST .../attempts/{attemptId}/dispatch`, `GET .../team-instances/{id}/execution`; route edit `PUT /api/providers/stub-team-a` |
| Real collaborators | server, catalog store, provider registry, `stub-llm` A and B, installed team package, binding revision 1 |
| Boundary exercised | HTTP API, catalog persistence, provider HTTP calls |
| Observable result | frozen in `fixtures/team-adoption-oracle.json` `SCN-TEAM-001`: task 2 on route B at binding revision 2, pin 2, provenance `changed` A to B, stub request counts, output text; the route-edit and queued cases as listed there |
| Negative control | input `2-invalid` (policy revision changed, route also changed): 409 `collaboration_revision_conflict` with detail "team deployment binding changed", no attempt, pin 1, zero stub requests |
| Isolation | scratch data dir, ephemeral ports for server and both stubs, scratch workspace id, no live service port; cleanup by the runner |
| Prerequisites | Prerequisites 1-5 above |
| Final local gate | phase integration batch (lead records argv) |
| Evidence | pending |
| Limitations | console-shaped binding only; proves routing by stub identity, not model quality |

### Behavior: a task in flight finishes on its model

| Field | Declaration |
|---|---|
| Canonical identity | tasks 3.1-3.3, 8.2; scenario SCN-TEAM-002 |
| Source identity | as above |
| Production entry point | the REST calls above while `stub-a.hold.fixtures.json` holds the tool-call response 4000 ms: install `2`, apply `routeEdit`, then let the attempt continue |
| Real collaborators | as above |
| Boundary exercised | in-flight tool-claim revalidation, settlement fence, provider settings API |
| Observable result | frozen `SCN-TEAM-002.attempt`: succeeded on A at revision 1, 2 requests to A and 0 to B, output `stub-a-complete`, no state reason or diagnostic, and the tool message in A's second logged request carries the roster result (`authorizationRevision`, `teamRevision`), so a failed tool call cannot pass |
| Negative control | input `2-invalid` installed while the attempt is held: the attempt does not settle `succeeded`, carries a diagnostic or state reason, and no further request reaches A |
| Isolation | as above |
| Prerequisites | as above |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | timing-dependent; sampled (see row) |

### Behavior: an unavailable route fails visibly at the boundary

| Field | Declaration |
|---|---|
| Canonical identity | tasks 2.2, 2.5, 4.2, 8.3; scenario SCN-TEAM-003 |
| Source identity | as above |
| Production entry point | binding revision 2 to route B; `PUT /api/providers/stub-team-b` with `enabled=false`; admit and dispatch; for the queued case `UAR_TEAM_EXECUTION_MAX_ACTIVE=1` with a held first task |
| Real collaborators | as above |
| Boundary exercised | admission refusal, dispatch claim settlement, execution summary read |
| Observable result | frozen `SCN-TEAM-003`: admission 409 `TEAM_ROUTE_PROFILE_MISMATCH` with no attempt; queued attempt `failed` with state reason `model_route_unavailable_at_boundary` and diagnostic code `TEAM_ROUTE_PROFILE_MISMATCH`; zero requests to both stubs, with route A enabled and default as substitution bait |
| Negative control | input `2-absent` (a route that does not exist): 409 `TEAM_ROUTE_PROFILE_MISMATCH`, no attempt, zero requests; the companion run with B enabled runs on B |
| Isolation | as above |
| Prerequisites | as above |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | a route edited between the dispatch claim and run start fails at run start by D4 and is not exercised |

### Behavior: model provenance is recorded and shown

| Field | Declaration |
|---|---|
| Canonical identity | tasks 4.1, 4.3, 5.1-5.2, 8.4; scenario SCN-TEAM-004 |
| Source identity | as above plus the built console bundle |
| Production entry point | `GET .../team-instances/{id}/execution` before and after a server restart on the same store; console Teams run view in a real browser |
| Real collaborators | server, store, console served by the server, browser |
| Boundary exercised | persistence, restart, REST read, rendered DOM |
| Observable result | frozen `SCN-TEAM-004`: `initial`, `changed` A to B with cause `member-model-rebind`, and `unchanged` for a third task; identical after restart; the rendered text contains both routes and "changed from" |
| Negative control | execution read with another workspace id returns 404 and the body contains no route or `modelProvenance` |
| Isolation | as above; scratch browser profile |
| Prerequisites | as above plus console assignment from the lead |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | polling, not push; push depends on `uiux-realtime-topics-and-replica-refresh` |

### Behavior: an explicit route pin is honoured

| Field | Declaration |
|---|---|
| Canonical identity | tasks 2.1, 8.7; scenario SCN-TEAM-007 |
| Source identity | as above |
| Production entry point | install a binding whose model entry pins `profile` and `settingsRevision`, apply `routeEdit`, admit, then install a model-only revision with the current revision and admit again |
| Real collaborators | as above |
| Boundary exercised | binding install, provider settings API, admission |
| Observable result | frozen `SCN-TEAM-007`: first admission 409 `TEAM_REVISION_CONFLICT` with no attempt; after the new revision one attempt on settings revision 2 |
| Negative control | pin with profile id `uar.bogus-profile`: 409 `TEAM_ROUTE_PROFILE_MISMATCH`, no attempt |
| Isolation | as above |
| Prerequisites | as above; decision D5 confirmed |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | not mandatory until D5 is confirmed |

### Behavior: unrelated revision checks are unchanged

| Field | Declaration |
|---|---|
| Canonical identity | tasks 6.1, 8.5; scenario SCN-TEAM-005 |
| Source identity | as above |
| Production entry point | dispatch with a stale `expectedTeamRevision`; a stale roster cursor; `PUT /api/providers/stub-team-a` with a stale `settingsRevision` |
| Real collaborators | as above |
| Boundary exercised | REST and team tool boundaries |
| Observable result | frozen `SCN-TEAM-005`: each stale request returns `TEAM_REVISION_CONFLICT` (provider edit 409) and leaves state unchanged |
| Negative control | dispatch of an attempt id that belongs to no team: 404 `collaboration_not_found` |
| Isolation | as above |
| Prerequisites | as above, after an adoption has occurred |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | the wait and context-evidence variants (`waits.rs`, `context_evidence.rs`) are covered by source exclusion in task 6.1, not by a live call |

### Behavior: a route disabled during a running task does not stop it

| Field | Declaration |
|---|---|
| Canonical identity | tasks 3.1-3.3, 8.8; scenario SCN-TEAM-008 |
| Source identity | as above |
| Production entry point | `stub-a.hold.fixtures.json`; `PUT /api/providers/stub-team-a` with `enabled=false` during the hold |
| Real collaborators | as above |
| Boundary exercised | in-flight revalidation, provider settings API |
| Observable result | frozen `SCN-TEAM-008`: attempt succeeds on A with 2 requests and `stub-a-complete` |
| Negative control | `POST .../attempts/{attemptId}/cancel` during the hold: the attempt ends cancelled or cancellation_requested without `stub-a-complete`, showing cancel is the stop |
| Isolation | as above |
| Prerequisites | as above; decision D10 confirmed |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | not mandatory until D10 is confirmed |

### Behavior: a wait continuation adopts at its boundary

| Field | Declaration |
|---|---|
| Canonical identity | tasks 2.3, 3.2, 8.6; scenario SCN-TEAM-006 |
| Source identity | as above |
| Production entry point | a coordinator that delegates and yields, then a model-only revision before the wake |
| Real collaborators | as above plus the team host-workspace context where used |
| Boundary exercised | wait wake, continuation attempt creation, host-context check |
| Observable result | frozen `SCN-TEAM-006`: wait `resumed` once, continuation on B at revision 2 |
| Negative control | `2-invalid` between yield and wake: wait `invalidated` with `TEAM_WAIT_INVALIDATED`, no continuation |
| Isolation | as above |
| Prerequisites | as above plus Prerequisite 6; decision D7 confirmed |
| Final local gate | phase integration batch |
| Evidence | pending |
| Limitations | NOT_EXERCISED unless Prerequisite 6 is built |

## Scenario matrix rows

All rows: substituted boundaries none (the `stub-llm` endpoints are real HTTP model endpoints, not substituted boundaries of the system under test). Isolation: scratch data directory, ephemeral ports, scratch workspace id, no credential or JWT in any step. Restoration: stop the server and both stub instances, delete the scratch directory. Every row's expected values are in `fixtures/team-adoption-oracle.json` (R1). Every mandatory row has a negative control that rejects a real invalid input and is different from the scenario (R2). Stochastic rule for sampled rows: each sample uses a fresh scratch store and fresh stub instances; stopping rule is fixed N with all samples required to pass.

**SCN-TEAM-001, member-model rebind adopted at the next task boundary**
- Actor: operator running a team. Outcome: the next task runs on the newly bound model, the running one is not disturbed.
- Source: `openspec:multi-agent-orchestration#A running team adopts model changes at its next task boundary` and `openspec:multi-agent-orchestration#A model-only binding change re-pins the team and other changes still block`. Reason now: operator decision 2026-11-19; today a rebind kills the team. Likelihood: UNKNOWN. Mandatory: true. nondeterministic: true.
- Stochastic: samples 5; grader = every assertion in `SCN-TEAM-001` of the oracle file holds, including the route-edit and queued cases; stoppingRule = fixed N=5, all must pass.
- Entry: REST binding install, admit, dispatch, execution read, provider edit. Collaborators: server, catalog store, provider registry, `stub-llm` A and B, installed team package.
- Oracle: observation = execution summary and both stub request logs; expected truth source = `fixtures/team-adoption-oracle.json`; comparator = field-by-field equality for routes, binding revisions, pin, provenance, output text and per-stub request counts.
- Negative control: input = binding `2-invalid`; expected rejection = 409 `collaboration_revision_conflict`, detail "team deployment binding changed", no attempt, pin 1, zero stub requests.
- Fixtures: `fixtures/stub-a.hold.fixtures.json`, `fixtures/stub-b.fixtures.json`, oracle file; package and bindings per oracle `bindingRevisions`. Evidence: request and response bodies, summary JSON, both stub logs, exit codes.
- Substituted boundaries: none. Limitations: console-shaped binding only. Invalidation keys: `team_execution/admission.rs`, `model_capture.rs`, `resolution.rs`, `compiler/collaboration/bindings.rs`, `llm/team_profile.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 1.1, 1.2, 2.1, 2.2, 2.5, 4.1, 7.1, 7.2, 8.1.

**SCN-TEAM-002, a task in flight finishes on its model**
- Actor: operator. Outcome: editing a route or rebinding mid-task does not break or re-route the running task. Source: `openspec:multi-agent-orchestration#A task in flight finishes on the model it started with`. Reason now: in-flight revalidation hazard in `epoch.rs` and `request.rs`. Likelihood: UNKNOWN. Mandatory: true. nondeterministic: true (timing of edits against the hold).
- Stochastic: samples 5; grader = the `SCN-TEAM-002.attempt` assertions hold and both edits were applied while stub A's log shows the first request received and unanswered; stoppingRule = fixed N=5, all must pass.
- Entry: REST plus `PUT /api/providers/stub-team-a` during the held response. Collaborators: as 001, `stub-a.hold.fixtures.json`.
- Oracle: observation = attempt record and stub logs; expected truth source = oracle file; comparator = equality on status, route, revision, per-stub counts, output.
- Negative control: input = `2-invalid` installed while held; expected rejection = the attempt does not settle `succeeded`, carries a diagnostic or state reason, no further A request.
- Fixtures: as 001. Evidence: stub logs with arrival order showing the edits between first request and release, attempt JSON.
- Substituted boundaries: none. Limitations: disable during run is SCN-TEAM-008. Invalidation keys: `resolution.rs`, `runtime/team_execution/epoch.rs`, `runtime/turn/request.rs`, `team_execution/mod.rs`, `settlement.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 3.1, 3.2, 3.3, 7.1, 8.2.

**SCN-TEAM-003, unavailable route fails visibly with no substitution**
- Actor: operator. Outcome: a task whose new route is unavailable fails with a named reason and sends no model request. Source: `openspec:multi-agent-orchestration#An unavailable route at the boundary fails visibly`. Reason now: decision text "never silently substitute". Likelihood: UNKNOWN. Mandatory: true. nondeterministic: true (queue and claim timing).
- Stochastic: samples 5; grader = admission refusal and queued failure assertions hold and both stub logs are empty for the task; stoppingRule = fixed N=5, all must pass.
- Entry: REST admit and dispatch with B disabled and `UAR_TEAM_EXECUTION_MAX_ACTIVE=1`. Collaborators: as 001.
- Oracle: observation = HTTP result, attempt record, stub logs; expected truth source = oracle file; comparator = status, code, state reason, diagnostic code, attempt count, request counts.
- Negative control: input = `2-absent`; expected rejection = 409 `TEAM_ROUTE_PROFILE_MISMATCH`, no attempt, zero requests.
- Fixtures: as 001; route A enabled and registry default as substitution bait. Evidence: responses, attempt JSON, logs.
- Substituted boundaries: none. Limitations: run-start drift window (D4) not exercised. Invalidation keys: `model_capture.rs`, `continuations.rs`, `settlement.rs`, `runtime/team_execution/controller.rs`, `llm/team_profile.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 2.2, 2.5, 4.2, 7.1, 8.3.

**SCN-TEAM-004, provenance recorded, durable and shown**
- Actor: operator reading the console. Outcome: sees which model each task used and what changed. Source: `openspec:multi-agent-orchestration#Each team attempt records and exposes its model provenance`. Reason now: decision cost note. Likelihood: UNKNOWN. Mandatory: true. nondeterministic: true (restart and rendering).
- Stochastic: samples 3; grader = JSON assertions equal before and after restart and the rendered text contains the frozen strings; stoppingRule = fixed N=3, all must pass.
- Entry: execution read, server restart, console Teams run view. Collaborators: server, store, console, browser.
- Oracle: observation = summary JSON and rendered text; expected truth source = oracle file `SCN-TEAM-004`; comparator = equality and substring presence.
- Negative control: input = execution read with another workspace id; expected rejection = 404 with no route or provenance in the body.
- Fixtures: SCN-TEAM-001 flow data, oracle file. Evidence: JSON before and after restart, DOM text capture.
- Substituted boundaries: none. Limitations: polling, not push. Invalidation keys: `domain/team_execution.rs`, `frontend/src/platform/entities/teams/contracts.ts`, `frontend/src/features/teams/ui/run/attempt-parts.tsx`, `fixtures/*`. Restoration: as above plus close the scratch browser profile. Owner tasks: 4.1, 4.3, 5.1, 5.2, 8.4.

**SCN-TEAM-005, unrelated revision checks unchanged**
- Actor: operator and team members. Outcome: stale-state protection still works after adoption. Source: `openspec:multi-agent-orchestration#Concurrency checks unrelated to model adoption are unchanged`. Reason now: risk that relaxing the gate leaks into other checks. Likelihood: UNKNOWN. Mandatory: true. nondeterministic: false (sequential REST calls with fixed revisions).
- Entry: stale dispatch, stale roster cursor, stale provider settings edit. Collaborators: as 001.
- Oracle: observation = response codes and state; expected truth source = oracle file `SCN-TEAM-005`; comparator = equality on status and code, and state unchanged.
- Negative control: input = dispatch of an attempt id that belongs to no team; expected rejection = 404 `collaboration_not_found`.
- Fixtures: team after an adoption, oracle file. Evidence: responses.
- Substituted boundaries: none. Limitations: wait and context-evidence variants by source exclusion only. Invalidation keys: `dispatch.rs`, `peer_commands.rs`, `llm/registry.rs`, `api/providers.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 6.1, 8.5.

**SCN-TEAM-006, wait continuation adopts at its boundary**
- Actor: operator with a delegating coordinator. Outcome: a coordinator woken after a rebind continues on the new model. Source: `openspec:multi-agent-orchestration#A running team adopts model changes at its next task boundary`. Reason now: D7. Likelihood: UNKNOWN. Mandatory: false until D7 is confirmed and Prerequisite 6 exists. nondeterministic: true (wait and wake timing).
- Stochastic: samples 5; grader = `SCN-TEAM-006` assertions hold; stoppingRule = fixed N=5, all must pass.
- Entry: delegate, yield, binding install, wake. Collaborators: as 001, host context where used.
- Oracle: observation = wait state, continuation attempt, stub logs; expected truth source = oracle file; comparator = equality.
- Negative control: input = `2-invalid` between yield and wake; expected rejection = wait `invalidated`, `TEAM_WAIT_INVALIDATED`, no continuation.
- Fixtures: coordinator and worker package, templated stub fixture (Prerequisite 6). Evidence: wait and continuation JSON, logs.
- Substituted boundaries: none. Limitations: NOT_EXERCISED unless Prerequisite 6. Invalidation keys: `continuations.rs`, `runtime/team_execution/host.rs`, `team_mailbox.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 2.3, 3.2, 8.6.

**SCN-TEAM-007, an explicit route pin is honoured**
- Actor: operator with a pinned binding. Outcome: a stale pin fails visibly and a new revision clears it. Source: `openspec:multi-agent-orchestration#An explicit route pin in a binding is honoured`. Reason now: D5, `model_capture.rs:23-30` against route edits. Likelihood: UNKNOWN. Mandatory: false until D5 is confirmed. nondeterministic: false.
- Entry: binding install with pin, provider edit, admit. Collaborators: as 001.
- Oracle: observation = admission responses and attempt list; expected truth source = oracle file; comparator = status, code, attempt count, settings revision.
- Negative control: input = pin with profile id `uar.bogus-profile`; expected rejection = 409 `TEAM_ROUTE_PROFILE_MISMATCH`, no attempt.
- Fixtures: pinned binding revisions, oracle file. Evidence: responses, attempt JSON.
- Substituted boundaries: none. Limitations: depends on D5. Invalidation keys: `model_capture.rs`, `llm/team_profile.rs`, `llm/registry.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 2.1, 8.7.

**SCN-TEAM-008, a route disabled during a running task does not stop it**
- Actor: operator. Outcome: disabling a route does not change a task already running; cancel is the stop. Source: `openspec:multi-agent-orchestration#A task in flight finishes on the model it started with`. Reason now: D10. Likelihood: UNKNOWN. Mandatory: false until D10 is confirmed. nondeterministic: true (timing).
- Stochastic: samples 5; grader = `SCN-TEAM-008.attempt` assertions hold with the disable applied while A's first request is held; stoppingRule = fixed N=5, all must pass.
- Entry: `PUT /api/providers/stub-team-a` with `enabled=false` during the hold. Collaborators: as 002.
- Oracle: observation = attempt record and stub log; expected truth source = oracle file; comparator = equality.
- Negative control: input = attempt cancel during the hold; expected rejection = attempt cancelled or cancellation_requested, no `stub-a-complete`.
- Fixtures: `stub-a.hold.fixtures.json`. Evidence: attempt JSON, stub log.
- Substituted boundaries: none. Limitations: depends on D10. Invalidation keys: `resolution.rs`, `epoch.rs`, `team_profile.rs`, `fixtures/*`. Restoration: as above. Owner tasks: 3.1, 3.2, 3.3, 8.8.

## Risks and open items

- **Pending operator confirmation:** D5 (explicit pins honoured), D7 (continuation is a boundary), D10 (disable does not stop a running task), and the D1 reading that `credentialRef` is part of the model change.
- **UNVERIFIED:** consumers of `modelBindings[].credentialRef` outside the team path; how a workflow run records its binding revision (G7); whether `prepare_representation_tool` revalidates through `revalidate_authority`; the exact name and payload of the public team topic; the TEAM-01 fixture script path (the scenario is recorded under `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/.../agent-team-concurrency-repair/`).
- **The uncomfortable part.** Re-pinning is the one place this change lets an operator edit alter a durable team record without a new team. A wrong classifier would let a skill or authority change through under the model-change label. The classifier therefore fails closed, compares everything except the model array and three per-revision fields, and has its own negative-control scenario. A second discomfort: a disabled route no longer stops a running task by itself, which is what the decision says and which an operator reaching for a kill switch may not expect.
- The `settle` path for a failed claim runs on an attempt the executor owns. Settlement requires the execution fence (`settlement.rs:135-136`); if the fence is lost the attempt stays queued, as today.
