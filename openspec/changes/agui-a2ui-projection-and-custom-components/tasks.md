# Tasks

Revision 4. Operator decisions D-1 to D-5 accepted 2026-10-27. Tasks assume
the recommended answers to new decisions D-6 (split rejections into I23-I25,
denominator 107) and D-7 (other-owner refusal by source review); task 0.1b
records them. The work-package letter after each group names the sole writer
(ownership table in the KBD child phase `plan.md`).

## 0. Workflow and decisions (lead)

- [x] 0.1 Record the operator's answers to the design's open questions in the KBD child phase `plan.md` and adjust affected tasks (D-1 to D-5 accepted as recommended, 2026-10-27; no task changes)
- [ ] 0.1b Record the operator's answers to D-6 and D-7 in `plan.md` and adjust groups 6-7 if either differs
- [ ] 0.2 Add catalog rows I21 (template component), I22 (host component and fail-closed), I23 (invalid definitions refused), I24 (disallowed components in surfaces refused) and I25 (isolation: library, presentations, test trigger) to `.prometheus/verification/bdd-validation-catalog.md`, and revise the denominator from 102 to 107
- [ ] 0.3 Before frontend code, run the repo's UI routing (Impeccable audit and critique, frontend-design, UI/UX Pro Max, Vercel React and composition) and record a one-paragraph summary in the KBD child phase
- [ ] 0.4 Reconcile KBD child state through the KBD runtime, not by hand-editing generated files: one canonical child directory holding the plan documents, implementation tasks registered so progress reflects them, and `scope.json` widened to the source paths in the ownership table

## 1. A2UI state patches and the dedicated route outbound

Tasks 1.1, 1.2 and 1.6 were withdrawn on 2026-10-27 and moved to `a2ui-surface-state-patches`.


- [ ] 1.3 (WP A) Extract the chat adapter's value builders for `uar.artifact.available`, `uar.artifact.input_required` and `STATE_DELTA` in `src/uar/api/adapters.rs` into shared functions with explicit identity and an option to omit `sourceRunId`; chat output unchanged
- [ ] 1.4 (WP A) Add `Artifact`, `ArtifactDisplay`, `ArtifactInputRequest` and `StatePatch` arms to `Projector::project` in `src/uar/api/agui.rs`, with wire identity and no native run id
- [ ] 1.5 (WP A) At the start of each dedicated-route run stream, emit a `STATE_SNAPSHOT` built by applying the replay of the thread's earlier runs (native ids from the bindings, ordinal order) to `{"run":null,"a2ui":{"surfaces":{},"actions":{}}}` — to be rebuilt on changes 1 and 2; under change 1 the replay log is not RFC 6902 state
- [ ] 1.7 (WP A) Document the dedicated route's A2UI events, the thread snapshot and that `input_required` is display-only, in `docs/protocols/ag-ui-profile.md`

## 2. Dedicated route inbound actions

- [ ] 2.1 (WP C) Split `continue_run` in `src/uar/a2ui/routes.rs` into `prepare_continuation` (request preparation, host resources, matching check) and execution via `RunManager::execute_request_with_run_id`; confirm whether `attach_host_resources` creates durable effects and, if so, release them when a prepared request is dropped
- [ ] 2.2 (WP C) Create `src/uar/a2ui/actions.rs` (registered in `src/uar/a2ui/mod.rs`) with the action-name check (action event names only) and the shared prepare/execute helpers; switch `submit_action` to the new check, keeping its source-run action patch and responses
- [ ] 2.3 (WP A) Add an action-dependencies field to `RunApiState` (`src/uar/api/routes.rs`) and update the `RunApiState` literals in `tests/bauar_full_harness_cursor.rs`, `tests/bauar_receipt_projection.rs`, `tests/context_history_integrity.rs`, `tests/truthful_tool_outcomes.rs` and `tests/stream_terminal_contract.rs`
- [ ] 2.4 (WP A) In `agui.rs`, accept `forwardedProps.uar.a2ui_action` plus host fields: walk the thread's bindings to the declaring run, prepare the continuation, then reserve with a pre-assigned native id, execute, and stream; refuse undeclared actions, other owners' threads, protected identity keys, invalid host fields and pre-restart surfaces before reserving; keep the existing unknown-outcome reservation semantics
- [ ] 2.5 (WP A) Synthesize the action `STATE_DELTA` (`add /a2ui/actions/<wireRunId>`) right after `RUN_STARTED` from the binding
- [ ] 2.6 (WP A) Make a replayed wire run with the same action attach to its reserved continuation
- [ ] 2.7 (WP A) Document action admission in `docs/protocols/ag-ui-profile.md`

## 3. Component registry: server (WP C)

- [ ] 3.1 Add template and host definition records in the design-system store with distinct `primitive_type` values and catalog-namespaced slugs; exclude them from the generic `/api/uar/a2ui/components` list, get, update and delete
- [ ] 3.2 Implement the props-schema subset checker, the matching value validator and the template checker
- [ ] 3.3 Create `src/uar/a2ui/component_registry.rs` (registered in `a2ui/mod.rs`): atomic snapshot, surface catalog index keyed by owner, session and surface, and `expand_templates` (root keeps the use id; inner `<useId>__<nodeId>`; references rewritten)
- [ ] 3.4 Extract `require_admin_key_for(required, expected, headers)` in `src/uar/api/settings.rs`; add `GET/POST /api/uar/a2ui/templates`, `GET/PUT/DELETE /api/uar/a2ui/templates/{name}` under it, and read-only `GET /api/uar/a2ui/catalogs` and `/catalogs/{catalogId}`; enforce name collisions with built-ins, host components and templates
- [ ] 3.5 Add `parse_message` modes `Baseline` and `AllowExtensions` in `src/uar/a2ui/protocol.rs`; accept `urn:uar:a2ui:catalog:1+<name>` only in `AllowExtensions`
- [ ] 3.6 Keep `src/uar/a2ui/presentations.rs` on `Baseline`
- [ ] 3.7 Make `A2uiRenderSkill` (`runtime/native_skills/a2ui_render.rs`) hold the registry and index, implement `execute_with_context` (expand, validate with owner and session; refuse extension catalogs when either is absent), and generalize its schema; update its registration in `runtime/native_skills/mod.rs`
- [ ] 3.8 Add the read-only `a2ui_catalog` tool (`runtime/native_skills/a2ui_catalog.rs`, registered in `native_skills/mod.rs`); point the render tool's description at it
- [ ] 3.9 In `runtime/a2ui_output.rs`, parse `a2ui_render` output with `AllowExtensions` and `presentation_render` output with `Baseline`; no signature change
- [ ] 3.10 Apply expansion and catalog-aware validation in `submit_messages`/`publish_messages` (run owner and session) and in saved surface templates (batch `createSurface` catalog, else baseline), in `a2ui/routes.rs`
- [ ] 3.11 Route `surface_test_trigger` through the same validation as `submit_messages`; remove its unvalidated publication
- [ ] 3.12 Update `docs/protocols/a2ui-profile.md`: templates, extension catalogs, catalog index, props subset, authority, seed-file authority, the `add` patch ops and the loopback-caller risk

## 4. Host seeding and wiring (WP L)

- [ ] 4.1 Add the new config key `a2ui.host_component_seed_dir` and the seed file format (catalog, name, version, props schema); add a seed file for the 7 `+entities` definitions with schemas derived from the client APIs
- [ ] 4.2 In `src/server.rs`: load the registry snapshot at startup; apply seed files authoritatively (upsert on version change, remove undeclared host definitions, skip and log name collisions); give `A2uiApiState` the two security values; wire `RunApiState`'s action dependencies; pass the registry and index to native skill registration

## 5. Component registry: clients (WP D)

- [ ] 5.1 Extend `frontend/src/platform/a2ui/protocol.ts` to accept extension catalogs and extension components structurally; baseline validation unchanged
- [ ] 5.2 Add `registerA2uiComponent(catalogId, name, impl)` to `@prometheus-ags/a2ui-uar` on `a2ui-core`'s `Catalog`, with a resolver from a surface's declared catalog to built-ins plus registered implementations
- [ ] 5.3 Register the 7 entity components for `urn:uar:a2ui:catalog:1+entities`; route `frontend/src/features/chat/components/a2ui-artifact-block.tsx` and `frontend/src/features/presentations/ui/presentation-preview.tsx` through the resolver
- [ ] 5.4 Report a surface that uses an unimplemented component as invalid, naming the component and catalog, with the existing bounded source disclosure
- [ ] 5.5 Make `frontend/packages/a2ui-lit` and `frontend/packages/a2ui-svelte` fail closed for extension components

## 6. Behavior scenarios (lead; authored now, run only at the verification boundary)

- [ ] 6.1 Author I19 (dedicated route): surface event with wire identity and no native id; snapshot then deltas apply, including `createSurface` then `updateComponents` and a repeated update; chat-route event shape parity; chat replay snapshot holds components and data model; reconnect with `Last-Event-ID`; first action → one continuation with the action delta right after `RUN_STARTED`; continuation snapshot holds the source surface; second action on the same surface; replayed wire run → no second continuation; non-action string refused; mismatched host fields refused and the thread still accepts a run; action after an owned restart refused
- [ ] 6.2 Author I21 (template): define `StatusBadge` over REST without an API key (auth flag off on the owned instance); `a2ui_catalog` lists it; agent uses it as `root`; published surface holds expanded built-ins with valid references; browser renders it; its button action continues the run; definition survives an owned restart; deleting it leaves published surfaces rendering
- [ ] 6.3 Author I22 (host): `+entities` lists all 7 components; agent publishes `EntityCard`; browser renders it and its action dispatches; a later update in a separate call keeps the catalog; a host fixture adds a `+bdd` seed file with a component no client implements and a newer version of one definition, restarts, checks the upgrade landed and that a surface using the `+bdd` component is reported invalid with the component named, then removes the seed file and restarts, and checks `+bdd` is gone
- [ ] 6.4 Author I23 (invalid definitions refused): templates named `Button`, `EntityCard` or an existing template; a template using a non-built-in; props schemas with `format: uri` or an open object
- [ ] 6.5 Author I24 (disallowed components in surfaces refused): unregistered component; host component in the baseline catalog; host component on an unindexed surface; `functionCall` action value; a built-in continuation update still accepted
- [ ] 6.6 Author I25 (isolation): the generic library list, get, update and delete cannot reach a definition; a presentation and a `presentation_render` output with `EntityCard` are refused; the surface test trigger refuses an unregistered component and publishes nothing
- [ ] 6.7 Record the implementation-complete inventory with exact source identity and present the gate commands to the operator

## 7. Verification boundary (operator-approved commands only)

- [ ] 7.1 Build: the owned binary profile `cargo build`; `cargo test --no-run` for the same profile (compiles test targets, runs nothing); owned restart; `pnpm build`; the Lit package `build` (`tsc`) and the Svelte package `build` (`svelte-check`); copy `static/` into the owned UI directory
- [ ] 7.2 Run `openspec validate agui-a2ui-projection-and-custom-components`
- [ ] 7.3 Run I19, I21, I22, I23, I24 and I25 once each and review raw evidence
- [ ] 7.4 Rerun C14, C15 and C53, whose rendering, patch or action path changed
- [ ] 7.5 Record receipts, catalog credits and the phase reflection

## Workflow follow-up

- Archive the OpenSpec change after the operator accepts the results.
- Close the KBD child phase with `handoff-out.md` and return the waypoint to `uar-verification-simplification`.
- Follow-ups outside this change: host-component rendering in Lit and Svelte; durable surface replay, catalog index and thread snapshots; answering `input_required` on the dedicated route; a thread-level chat snapshot; I01-I04 chat and native AG-UI modes.
