# Assessment — agui-a2ui-projection-and-custom-components

Read-only source assessment, 2026-10-27. No code, builds or tests were run.

## Gap 0 — A2UI state patches cannot apply (pre-existing, found in review)

`createSurface` → `add /a2ui/surfaces/<id>` with no `components` key
(`src/uar/a2ui/realtime.rs:76-80`); `updateComponents`/`updateDataModel` →
`replace …/components`/`…/dataModel` (`:81-90`). The server applier refuses
`replace` on a missing key (`src/uar/api/sse.rs:79`), so the chat replay
snapshot stops synchronizing at the first surface (`:142-145`); the browser
applier does the same (`frontend/src/platform/agui/agui-adapter.ts:72`).
`run-trace-projection.ts:325,331` keys on `replace`. Source-verified, not yet
observed at runtime.

## Gap 1 — dedicated AG-UI route drops A2UI events (catalog I19, outbound)

`src/uar/api/agui.rs:635-652` `Projector::project` maps `ChatDelta`, `ToolStart`,
`ToolCallApprovalRequired`, `ClientToolsRequested`, `ToolEnd`, `SkillActivated`
and `AgentSelected`; all other events return `Vec::new()`. The chat route's
official adapter projects `Artifact`/`ArtifactDisplay` (`CUSTOM
uar.artifact.available`), `ArtifactInputRequest` (`CUSTOM
uar.artifact.input_required`) at `src/uar/api/adapters.rs:205-229`, and
`StatePatch` (`STATE_DELTA`) at `:332-337`. `docs/protocols/ag-ui-profile.md`
makes `CUSTOM` the extension transport and excludes draft activity events.

## Gap 2 — no A2UI action path on the dedicated route (catalog I19, inbound)

Dedicated-route resume accepts approval and MCP elicitation answers only
(`agui.rs:295-322,347-367`). Admission validates resources (`:338-349`), reserves
the binding (`:350`) and admits with a pre-assigned native id (`:352`). Actions enter only through `POST
/api/uar/runs/{run_id}/a2ui/actions` (`src/uar/a2ui/routes.rs:827-896`, state
`A2uiApiState` at `:45-58`, continuation via `continue_run` at `:212`; run ids minted in
`runtime/manager.rs:3298-3302`, with `execute_request_with_run_id` at `:3307`). The
dedicated stream never exposes the native run id.

## Gap 3 — custom components do not exist

- Server: closed 9-variant `Component` enum (`src/uar/a2ui/protocol.rs:70-80`;
  the id match over it is at `:267-279`); `src/uar/a2ui/registry.rs` already
  holds the artifact-schema `A2uiRegistry`;
  only `urn:uar:a2ui:catalog:1` and the A2UI basic catalog accepted (`:264`);
  A2UI admission paths: `a2ui_render` tool (`native_skills/a2ui_render.rs:169-170`;
  schema pins `catalogId` at `:41` and `component` to the 9 built-ins near
  `:72-97`, compiled once at registration, `native_skill.rs:439-442,460-464`),
  host renderer output (`runtime/a2ui_output.rs:137`), `submit_messages`/
  `publish_messages` (`a2ui/routes.rs:733-736`), saved surface templates
  (`routes.rs:989,1035,1175`), presentations (`presentations.rs:59,147`,
  baseline-only at `:52`), and `surface_test_trigger` (`routes.rs:1296-1349`),
  which publishes without validation; `a2ui_output.rs:101,137` serves both
  `a2ui_render` and `presentation_render`.
- Existing library: `/api/uar/a2ui/components` and `/custom-schemas` store
  `design_systems::types::Component` records, today saved surface templates
  (`primitive_type: "A2uiSurface"`) and artifact schemas, not new component types.
- Client: chat renders through `UarSurface` + `uarBasicCatalog`
  (`@prometheus-ags/a2ui-uar`, on `a2ui-core`'s `Catalog` registry). A
  client-only `EntityCard` lives in `urn:uar:a2ui:catalog:1+entities`, which
  the server rejects. Lit and Svelte renderers exist.
- Spec `a2ui-uar-renderer` requires the catalog to implement exactly 9
  components; `docs/protocols/a2ui-profile.md` forbids agent-supplied code and
  requires fail-closed diagnostics.

## Capability ownership

No spec owns the dedicated AG-UI route; this change adds `ag-ui-agent-endpoint`.
New `a2ui-component-registry`; modified `a2ui-uar-renderer`.
