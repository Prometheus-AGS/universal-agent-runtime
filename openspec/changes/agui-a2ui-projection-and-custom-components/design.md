> **Superseded 2026-10-27.** This change is being rebuilt as change 3 of the
> re-scoped AG-UI/A2UI work, on top of `a2ui-surface-state-patches` (change 1)
> and a durable thread-scoped surface store (change 2). Its A2UI state-patch
> fix has moved to change 1. Do not implement it from this text.

## Context

Revision 4, after three independent artifact reviews (2026-10-27). Operator decisions D-1 to D-5 are accepted; revision 4 raises two new decisions (D-6, D-7), recorded in the KBD child phase `plan.md`. Source facts (read 2026-10-27):

- **A2UI state patches (pre-existing defect).** `createSurface` becomes `add /a2ui/surfaces/<id>` with `{surfaceId, catalogId}` and no `components` key (`src/uar/a2ui/realtime.rs:76-80`); `updateComponents` and `updateDataModel` become `replace …/components` and `replace …/dataModel` (`:81-90`). The server replay applier refuses `replace` on a missing key (`src/uar/api/sse.rs:79`), so the chat replay snapshot stops synchronizing at the first surface (`:142-145`); the browser's applier does the same (`frontend/src/platform/agui/agui-adapter.ts:72`). Browser rendering does not depend on this snapshot. `frontend/src/features/chat/model/run-trace-projection.ts:325,331` keys on `replace` for components and dataModel. Rust consumers accept `add` or `replace` (`runtime/a2ui_output.rs:16`, `runtime/presentations.rs:77`). Unit tests in `realtime.rs:176-199` assert the current ops. The replay backbone is per run and in memory (`realtime.rs:106,111,145,155`).
- **Dedicated AG-UI route** (`src/uar/api/agui.rs`): projector `:635-652` drops all but text, tools, approvals, client tools, skills and agent selection. A new run requires the previous run terminal (`thread_run_not_closed`, `:250-252`); resources validate at `:338-349`; the binding is reserved by compare-and-swap (`create_agui_run(row, previous)`, `:350`) and admitted with a pre-assigned id (`:352`); an unknown admission outcome keeps the reservation held so a retry reads the mapping and never repeats execution (`:369-373`). Resume handles approvals and elicitations (`:295-322,347-367`). Events past the terminal cursor are dropped (`:689`).
- **Chat adapter** (`src/uar/api/adapters.rs`): `uar.artifact.available`/`input_required` at `:205-229`, with native `sourceRunId` at `:217,228`; `STATE_DELTA` at `:332-337`. Chat replay baseline `{"run":null,"a2ui":{"surfaces":{}}}` (`sse.rs:127-130`).
- **A2UI actions** (`src/uar/a2ui/routes.rs`): `submit_action` (`:827-896`) checks the run's replay with `json_contains_string` (`:812-823`), publishes `add /a2ui/actions/<uuid>` to the **source** run (`:867-882`), then `continue_run` (`:212-250`). `continue_run` prepares the request (`prepare_interaction_request`, `attach_host_resources`, `require_matching_host_resources`) and then calls `execute_request` (`:250`), which mints a run id (`runtime/manager.rs:3298-3302`); `execute_request_with_run_id` (`:3307-3308`) takes a reserved id and returns `String` (cannot fail).
- **A2UI admission paths**: `parse_message` (`src/uar/a2ui/protocol.rs:229`) over the closed `Component` enum (`:70-80`); `createSurface` alone names a catalog (`:47-50,247-252`); baseline and A2UI basic catalogs only (`:264`); event-only actions (`:101-110`). Paths: `a2ui_render` tool (`runtime/native_skills/a2ui_render.rs`, reached through `execute_with_context`, `runtime/native_skill.rs:150,320`); renderer output for both `a2ui_render` and `presentation_render` (`runtime/a2ui_output.rs:101,137`); `submit_messages`/`publish_messages` (`a2ui/routes.rs:733-736`); saved surface templates (`:989,1035,1175`; handlers `:1026-1109,1171-1241` have no `UserContext`); presentations (`a2ui/presentations.rs:59,147`, baseline at `:52`); and `surface_test_trigger` (`:1296-1349`, mounted via `server.rs:2435`), which publishes unvalidated payloads and has no caller in `frontend/` or `scripts/`.
- **Native tools**: schemas compile once (`native_skill.rs:439-442`); a changed descriptor collides (`:460-464`). `NativeExecutionContext` carries optional `verified_owner` and `session_id` (`:246-262`). `A2uiRenderSkill` is a unit struct registered at `native_skills/mod.rs:52`. `policies/tool-approval.cedar:32-37` already permits every tool.
- **Library and authority**: generic `/api/uar/a2ui/components` CRUD has no admin check and store-wide slugs (`routes.rs:1026-1169,1064,1187`). `require_admin_key(&SettingsApiState, &HeaderMap)` (`src/uar/api/settings.rs:397-407`) is skipped when `settings_mutation_auth_required=false` (owned Mac instance).
- **Clients**: chat A2UI block `frontend/src/features/chat/components/a2ui-artifact-block.tsx` and presentation preview `frontend/src/features/presentations/ui/presentation-preview.tsx` render via `UarSurface` + `uarBasicCatalog`. `+entities` has 7 client components (`frontend/packages/a2ui-uar/src/catalog/uar-entity-catalog.ts:23-32`); their schemas fit the props subset below (`entities/entity-card-api.ts:32-76`, `entities/entity-extensions.tsx:11-18`).
- **KBD state**: the plan documents live in `children/agui-a2ui-projection-and-custom-components/`, while runtime-generated `progress.json` (do not hand-edit) lives in `children/uar-verification-simplification/children/agui-a2ui-projection-and-custom-components/` and reports no implementation tasks. `scope.json` allows writes only to the plan directory.

## Goals / Non-Goals

**Goals:**
- A2UI state patches apply cleanly on the server and in the browser, and every route's snapshot reflects the surfaces its deltas build on.
- AG-UI clients on the dedicated route render UAR A2UI surfaces, act on them exactly once, and receive the continuation on the same thread.
- Operator-defined template components that every renderer and third-party client draws with no new code.
- Host components in declared extension catalogs, validated by the server, rendered by clients that implement them and refused elsewhere.
- No regression for built-in surfaces.

**Non-Goals:**
- Agent-defined components or runtime-supplied executable UI.
- Answering `uar.artifact.input_required` on the dedicated route (display only).
- Custom components in presentations (baseline-only).
- Host-component rendering in Lit and Svelte (fail closed only).
- Durable surface replay, catalog index or thread snapshot across restart.
- A thread-level snapshot for the chat route (its per-run snapshot is fixed by D1 only).

## Decisions

### D1. Fix A2UI patch generation
`realtime.rs` emits `add` (not `replace`) for `…/components` and `…/dataModel`. RFC 6902 `add` on an existing member replaces it, so repeated updates still work, and the first update now has a parent. `run-trace-projection.ts` accepts `add` or `replace` for those suffixes. The `realtime.rs` unit-test assertions are updated (authored, not run before the gate). Rust consumers already accept both ops.

### D2. Dedicated route outbound projection and thread snapshot
Add `Artifact`, `ArtifactDisplay`, `ArtifactInputRequest` and `StatePatch` arms to `Projector::project` from value functions extracted from `adapters.rs`, with wire identity and no `sourceRunId`. At the start of each run's stream, the route emits a `STATE_SNAPSHOT` built by applying, in ordinal order, the replay of every earlier run in the thread (by native id from the bindings) to `{"run":null,"a2ui":{"surfaces":{},"actions":{}}}`. A continuation therefore starts from the source run's surfaces. After a restart the replay is empty, so the snapshot holds only `{}` containers (consistent with D4's refusal). The chat baseline also gains `actions:{}`.

### D3. Inbound action: prepare, reserve, execute
A client sends a new `RunAgentInput` on the same closed thread with `forwardedProps.uar.a2ui_action` = `{surfaceId, name, sourceComponentId, timestamp?, context, a2uiClientDataModel?}` and optional host fields matching `ContinuationHostInput` (`routes.rs:78-88`). `continue_run` is split into `prepare_continuation` (the existing request preparation, host resources and matching check, returning the request or an error response) and execution through `execute_request_with_run_id`. The route:
1. walks the thread's bindings back through `previous_wire_run_id` to the run whose replay holds the surface and declares the action (D4); refuses if none;
2. runs `prepare_continuation`; refuses on error. Nothing is reserved yet;
3. reserves the binding with `create_agui_run(row, previous)` and a pre-assigned native id; if the compare-and-swap loses, the prepared request is dropped and the route attaches to the winner;
4. executes with `execute_request_with_run_id`, which cannot fail; an unknown outcome (crash) keeps the reservation held, exactly as today's admission path (`agui.rs:369-373`), and a retry reads the mapping;
5. streams the continuation. The route synthesizes one `STATE_DELTA` `add /a2ui/actions/<wireRunId>` immediately after `RUN_STARTED` from the binding's action, so the echo cannot race the run's terminal.
`submit_action` (REST) keeps publishing its action patch to the source run before continuing; its only change is the action-name check (D4). Task 2.x confirms during implementation whether `attach_host_resources` has durable side effects that a lost compare-and-swap would leave behind; if it does, those are released when the prepared request is dropped.

### D4. A declared action is an action event name
`apply_action` checks that the action name is the `event.name` of an action-valued prop of a component on the surface, replacing `json_contains_string` for this check on both routes.

### D5. Template components expand on the server
Unchanged from revision 3: the template root keeps the use id; inner ids are `<useId>__<nodeId>`; references inside the template are rewritten; published surfaces contain only built-ins. Template uses remain visible in the agent's own tool-call arguments.

### D6. Host components and the catalog index
Host definitions `{catalogId: "urn:uar:a2ui:catalog:1+<name>", name, version, propsSchema}` come from host seed files in a configured directory. **Seed files are authoritative for host catalogs:** at startup UAR upserts definitions whose version changed and removes host definitions that no seed file declares. This makes test fixtures removable and host upgrades land. A seed entry whose name collides with a built-in or an existing template is skipped with an error log.
A surface catalog index keyed by `(owner, session_id, surface_id)` records validated `createSurface` messages. A surface found in the index uses its catalog; a surface not found is validated as baseline (built-ins only). When the context has no owner or no session, extension catalogs are refused and validation is baseline-only.

### D7. Validation mode per admission path
`parse_message` gains modes `Baseline` and `AllowExtensions` (flat `Extension` component).
- `a2ui_render` (`execute_with_context`): expand templates, then validate with the registry and catalog index using the context's owner and session.
- `a2ui_output.rs`: `AllowExtensions` for `a2ui_render` output, `Baseline` for `presentation_render` output.
- `submit_messages`/`publish_messages`: expand and validate with the run's owner and session.
- Saved surface templates (no `UserContext`): expand and validate using only the batch's own `createSurface` catalog; a batch without one is baseline.
- `surface_test_trigger`: validate the payload through the same path as `submit_messages`; unvalidated publication is removed.
- Presentations: `Baseline`.

### D8. Agent discovery
The render tool's schema is generalized once (`catalogId` pattern; component items `oneOf` the strict built-in item or a generic extension item). The read-only `a2ui_catalog` tool returns templates and host components with props at call time. No per-tool governance permit is needed (`tool-approval.cedar:32-37`).

### D9. Props schema subset
Unchanged from revision 3: strings, numbers, booleans, string enums, arrays, data-model bindings, event-only action props, closed objects to depth 3; refusing `format: uri`, open objects, deeper nesting, `on*` props and `functionCall` actions. All 7 entity schemas fit.

### D10. Storage, names, authority
Distinct `primitive_type` records with catalog-namespaced slugs, excluded from generic library CRUD. Names: PascalCase; no collision with built-ins, any host component, or another template. The admin-key check is extracted to `require_admin_key_for(required: bool, expected: &Secret, headers)` and `A2uiApiState` receives the two security values at wiring. Template endpoints: `GET/POST /api/uar/a2ui/templates`, `GET/PUT/DELETE /api/uar/a2ui/templates/{name}`; read-only `GET /api/uar/a2ui/catalogs`, `/catalogs/{catalogId}`. With the flag off, any loopback caller (including an agent with `terminal_exec`) can write templates; templates only produce built-ins.

## Risks / Trade-offs

- **The uncomfortable one:** host components still need code in every client; third-party AG-UI clients and Lit/Svelte refuse surfaces with host components they don't implement.
- D1 changes the wire op for components and dataModel from `replace` to `add`. External clients that key on `replace` (as `run-trace-projection.ts` did) will miss updates until they accept both.
- Seed-file authority removes host definitions no seed declares, even if a saved surface template uses them; those surfaces then fail closed.
- With the flag off, loopback callers can write templates.
- The render tool's compiled schema is more permissive; strict checks live in `execute_with_context`.
- In-memory replay and index: after restart, old surfaces refuse actions, extension surfaces cannot be updated, and dedicated snapshots start empty.
- The stricter action check can refuse actions that today's string match accepts by accident; C14, C15 and C53 reruns cover shipped flows.

## Migration Plan

No data migration. Existing library entries keep their `primitive_type`. Wire-op change: `add` replaces `replace` for components/dataModel; consumers accepting both are unaffected. Rollback: revert D1 together with the client change; remove endpoints, seed loading, expansion and the `a2ui_catalog` tool; stored definitions are inert.

## Open Questions

None for D-1 to D-5 (accepted). Two new decisions are raised in `plan.md`: D-6 (splitting the rejection scenarios into I23-I25, denominator 107) and D-7 (other-owner refusal verified by source review).
