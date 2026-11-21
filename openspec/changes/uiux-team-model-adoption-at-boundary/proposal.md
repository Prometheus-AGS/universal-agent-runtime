## Why

Operator decision 2026-11-19 (`.prometheus/decisions.md`, "Running teams adopt model changes at the next task boundary"): a running agent team picks up a member-model change, or an edit to a route the team captured (profile, endpoint, price, disable), at its next task boundary. A task already in flight finishes on the model it started with. Per-task model provenance must be recorded and shown. When the new route is unavailable at the boundary the task fails visibly and never silently substitutes another model.

Today the opposite holds, and a rebind kills the team. Verified at source:

- A team pins its deployment binding revision at creation (`compiler/collaboration/team_planning.rs:131-134`) and nothing ever re-pins it. Installing a new binding revision therefore makes every later admission fail with "team deployment binding changed; reinstall the team binding" (`team_execution/admission.rs:124-128`).
- Every attempt is also fenced against the current binding revision, so an attempt that is already running fails the same way at its next revalidation (`team_execution/mod.rs:36`, `resolution.rs:42-48`) and at settlement (`settlement.rs:20`).
- Routes are re-resolved against the live provider registry and compared byte for byte with the capture (`resolution.rs:88-101`). Profile, reasoning and endpoint edits bump the route's settings revision (`llm/registry.rs:178-184`), so the compare fails with `TEAM_REVISION_CONFLICT`. Removal, disable, profile mismatch, a missing price identity or a changed receipt fail with `TEAM_ROUTE_PROFILE_MISMATCH` (`llm/team_profile.rs:62-112`).
- That re-resolution is not only at the start of a task. `CollaborationRunBinding::revalidate_authority` runs it through `revalidate_member_binding` (`runtime/turn/request.rs:33-48`, `runtime/team_execution/epoch.rs:7-25`), and the claim-revalidation trait routes tool preparation and invocation through `revalidate` by default (`runtime/tool_admission/mod.rs:667-679`; the `CollaborationRunBinding` override `prepare_representation_tool` is UNVERIFIED). An operator who edits a route while a task is calling tools can therefore break that task mid-run.

Analyze item 6 (`analysis.md`, "Delta from the assessment") and Build item 15 add that the binding-revision gate must be relaxed too, and the round-2 review (warning) requires that it be relaxed only for model-binding changes. Skill or package changes must still block.

## What Changes

- Add a binding delta classifier over `binding_history`: `Same`, `ModelOnly` (only model-route fields of `modelBindings` differ) or `Incompatible`. It fails closed when a revision is missing.
- Define the task boundary as the instant a task attempt's models are fixed: attempt creation (admission, delegation, wait continuation) and the queued-to-running dispatch claim. Re-capture current routes at each, re-pin the team to the current binding revision when the delta is `ModelOnly`, and record what changed.
- Make an attempt that is already running immune to later route and model-binding edits: its revalidation compares against the attempt's own captured models and its own binding revision, never the live registry.
- Fail visibly when the route is unavailable at the boundary: refuse the admission, or settle the queued attempt as failed with a named reason and a diagnostic, with no fallback model.
- Record per-attempt model provenance on `TeamExecutionAttempt` (additive, serde-default) and expose it through the existing execution summary so the console can show the model per task and what changed.
- Leave every unrelated `TEAM_REVISION_CONFLICT` check as it is.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `multi-agent-orchestration`: adds seven requirements for team model adoption at the task boundary. The pending team execution requirements live only in the open change `afc-c09-team-admission-runtime` (`specs/durable-team-execution/spec.md`, not yet in `openspec/specs/`), so this change adds to the existing main spec rather than modifying an unsynced one.

## Impact

Owner: `uar-runtime` (`src/uar/compiler/collaboration/**`, `src/uar/runtime/team_execution/**`, `src/uar/domain/team_execution.rs`). The console contract task is handed to `uar-console` and needs a single-writer assignment from the lead. No new endpoints, no dependency pins, no new public error codes. Persisted state gains one optional attempt field, so old catalog records stay readable.

Interface effects:

- `TeamExecutionAttempt` gains optional `modelProvenance`. `EffectiveTeamModelReceipt` is unchanged because it carries `deny_unknown_fields` (`domain/team_execution.rs:198-213`).
- `TeamInstance.binding.revision` can advance without a new team being created, only through the boundary paths in `design.md`.
- A queued attempt's `bindingRevision` and `effectiveModels` can change once, at the dispatch claim.

## Dependencies

- `uiux-realtime-topics-and-replica-refresh` (writer state-D, build item 14) owns the public team and binding topic the console needs for push. This change does not define, publish or name a topic. Until that topic exists the console keeps its existing 3 s `watchTeamRun` polling (`openspec/changes/console-agent-teams/tasks.md` item 1.5, marked done). Topic name and payload: UNVERIFIED here.
- `uiux-system-model-live-default` (writer providers-E, build items 12, 13, 16). A team member's model comes from its binding, an explicit choice, so the system default never reaches a team member. That is consistent with "an explicit model wins over the system default". This change does not touch the default-model path.

## Related open changes

| Open change | Overlap | Suggested disposition |
|---|---|---|
| `afc-c09-team-admission-runtime` | Owns the pending `durable-team-execution` spec: admission, "Effective authority is current at effects", "Exact endpoint settings remain distinct from route and price". Its sentence "Historical attempts SHALL retain captured settings across explicit revisioned rebind" is the closest existing text to this decision. | Continue. Its requirements are still unsynced; this change adds its own requirement names and does not contradict them. |
| `afc-c09-durable-team-planning` | Introduced the pinned `TeamBindingRef` this change re-pins. | Continue. |
| `afc-c09-task-ownership-mailbox` | `assignment_authority.binding_revision` compared at `team_mailbox.rs:321`. | Continue. |
| `repair-agent-team-concurrent-operation` | Same executor, settlement and TEAM-01 fixture. | Continue, or archive once accepted; this change reuses its fixture. |
| `team-workspace-host-context`, `team-host-extension-envelope`, `team-attempt-mcp-lifetime` | Host context compares binding revision (`runtime/team_execution/host.rs:137,192,227`). | Continue. This change updates the same comparison to the classifier. |
| `team-tool-admission-ownership` | Claim revalidators on tool admission. | Continue. |
| `console-agent-teams` | Console team surface and `ModelLine` that shows an attempt's model. | Continue. This change adds a contract field and one row of UI. |
| `harness-model-profiles-routing` | Provider/endpoint templates; different layer. | Continue, unrelated. |
