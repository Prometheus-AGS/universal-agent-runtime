# UAR A2UI surface-state inventory

Read-only source inventory, 2026-10-28, by a fresh-context read-only agent.
Nothing was run. Paths are relative to the repository root. Line numbers were
reported by the agent; the lead spot-checked the starting points against
earlier reads in this session but did not re-verify every line.

## Four representations of a surface

| Code | Representation | Where it travels |
|---|---|---|
| M | A2UI message stream (`createSurface`, `updateComponents`, `updateDataModel`, `deleteSurface`) | Artifact `content`, newline-delimited, `application/a2ui+json` |
| L | Replay log: `StatePatchOp` list per `run_id` in `InMemoryReplayBackbone` | `GET /runs/{id}/a2ui/surface-replay` |
| S | Shared-state JSON: `NormalizedEvent::StatePatch` | `STATE_DELTA` / `agui.state.patch`; folded into `STATE_SNAPSHOT` from baseline `{"run":null,"a2ui":{"surfaces":{}}}` |
| A | Artifact payload: `ArtifactDisplay`, `artifact_type:"a2ui"` | `CUSTOM uar.artifact.available` or legacy `agui.artifact` |

## 1. Server producers

| Location | Behavior | Repr | Scope |
|---|---|---|---|
| `src/uar/a2ui/realtime.rs:69-97` `surface_message_to_state_patch` | create: `add /a2ui/surfaces/<id>` `{surfaceId,catalogId}`; components: `replace …/components` with the whole message; data: `replace …/dataModel` with the whole message; delete: `remove`. Does not reduce. | L, S | — |
| `src/uar/runtime/a2ui_output.rs:91-208` `publish_tool_output` | `a2ui_render`/`presentation_render`: receipt, ceiling, `parse_message` per batch (134-139), per message `backbone.publish` + StatePatch (165-172), then one `ArtifactDisplay` (188-206) | L, S, A | L per run in memory; S/A ring + durable history |
| `src/uar/runtime/native_skills/a2ui_render.rs:20-178` | Tool schema + pre-check. Allows v0.9/v0.9.1 (35), both catalogs (41), 9 components (72). Example uses v0.9 + basic catalog (127-136). No `deleteSurface`. | M | per call |
| `src/uar/runtime/presentations.rs:357-383`, template `src/uar/a2ui/presentations.rs:130-174` | Presentation surfaces `presentation-<uuid>`; one `updateDataModel` per top-level key (163-172), so in S each replaces the last | M | per call |
| `src/uar/a2ui/routes.rs:724-810` `publish_messages` | Same flow via `emit_to_run`. No in-repo caller. | L, S, A | per run |
| `src/uar/a2ui/routes.rs:827-897` `submit_action` | `add /a2ui/actions/<uuid>` into L and S of the source run (867-882), then `continue_run` (883) | L, S | source run |
| `src/uar/a2ui/routes.rs:1296-1349` `surface_test_trigger` | Arbitrary kind + payload into L and S; no validation | L, S | per run |
| `src/uar/a2ui/policy_surface.rs:50-120`, emitted `manager.rs:4763-4797` | Effective-policy surface as a plain `Artifact`; bypasses `parse_message`, L and S | A | ring + durable |
| `src/uar/a2ui/routes.rs:978-1240` component library | `{profile, messages}` stored in the design-system store (`migrations/20260714120000_a2ui_design_systems.sql`) | M (persisted) | durable, not a run surface |

## 2. Server consumers and gates

| Location | Reads | Assumes |
|---|---|---|
| `src/uar/a2ui/protocol.rs:229-328` `parse_message` | M | v0.9.1 **or v0.9** (260); `urn:uar:a2ui:catalog:1` **or** basic URL (264); unique ids; `root` required (267-292); executable filter (230-247); closed 9-variant serde enum |
| `routes.rs:837-848` action check | L | Surface "exists" if any non-remove op path starts with the raw id (prefix match, unescaped); action "declared" if its name appears anywhere in an op value |
| `routes.rs:1358-1373` surface-replay | L | Active run plus presentation snapshot required |
| `a2ui_output.rs:6-87` output ceiling | S, A | Converts `/a2ui` output to diagnostics when surfaces are not allowed |
| `src/uar/runtime/presentations.rs:67-127` observer | S, A | Sets `surface_published`; projects `add /presentation` |
| `manager.rs:3096-3189` `continue_with_interaction` | source run record | New run, same `conversation_id`; does **not** copy L or S |

## 3. Transport and projection

| Location | In → out | Note |
|---|---|---|
| `src/uar/api/sse.rs:44-119` `apply_state_patch` | S | Strict RFC 6902; `replace` needs an existing key (79) |
| `sse.rs:122-168` `build_agui_replay_snapshot` | ring → `STATE_SNAPSHOT` | Baseline lacks `actions`; stops claiming state after first failure |
| `src/uar/api/adapters.rs:208-219, 332-338` | A → `CUSTOM uar.artifact.available`; S → `STATE_DELTA` | — |
| `src/uar/api/routes.rs:873-989` `stream_run` | ring/durable → snapshot + deltas | Active run only; bare u64 cursor; ends at terminal frame |
| `src/server.rs:6939-6960, 7104, 7260` `/v1/chat/completions` stream | S, A | **No `STATE_SNAPSHOT`**; 410 when the ring prefix is evicted. Used by the chat UI |
| `src/uar/api/run_events.rs:52-90` `GET /runs/{id}/events` | ring/durable | Works after eviction through durable fallback |
| `src/uar/api/agui.rs:627-657` dedicated route `/api/uar/ag-ui/agents/{id}` | — | **Drops** StatePatch, Artifact and ArtifactDisplay; inbound `state` stringified into the prompt; no action input (I19) |
| `src/uar/runtime/turn/host/event_projection.rs:47-61` | S/A scrub | Backbone written before scrubbing; `emit_to_run` skips scrubbing |

## 4. Frontend consumers

| Location | Repr | Note |
|---|---|---|
| `frontend/src/stores/chat-stream-store.ts:1139-1155` | A | Chat surfaces come **only** from artifacts; no `STATE_*` case |
| `frontend/src/features/chat/components/a2ui-artifact-block.tsx:212-345` | M | Exact v0.9.1, zod validator, `uarBasicCatalog` only, fresh `MessageProcessor` per artifact; action posts the full data model (318-319) |
| `frontend/src/platform/entities/a2ui-actions/*` | action receipt entity | id `[runId, surfaceId]`, single-shot, source must be `done` |
| `frontend/src/platform/agui/agui-adapter.ts:41-78` | S | Strict `replace` (72) |
| `frontend/src/features/chat/model/run-trace-projection.ts:39, 312-357` | L | Throws on `/a2ui/actions/`; rebuilds as v0.9.1/catalog:1; reduces with the hand-written `reduceA2uiMessage`, not `web_core` |

## 5. Persistence

| Store | Contents | Lifetime |
|---|---|---|
| `InMemoryReplayBackbone` (`realtime.rs:121-163`) | L | Process; removed on retention (`manager.rs:1608-1611`) |
| Event ring (`manager.rs:46, 107-141`) | S, A | 512 events per run |
| Durable run history (`src/uar/persistence/run_history.rs:13-31`; Postgres and Surreal migrations) | every event | Restart-durable, but `stream_run`, actions and surface-replay need the active run and return 404 after it leaves (asserted `tests/run_retention/runs.rs:81-92`) |
| Client PGlite | chat messages (A), run events (S), action receipts | client-local |

## 6. Catalogs and validation

- **No server catalog object.** The definition is spread across `protocol.rs:12-17, 68-80`, the `a2ui_render.rs` schema, a hardcoded builtins list (`routes.rs:919-976`) and the presentation validator (`src/uar/a2ui/presentations.rs:49-60`).
- **Client catalogs:** `frontend/packages/a2ui-uar/src/catalog/uar-basic-catalog.ts:32` and `uar-entity-catalog.ts:21` (`…:1+entities`, rejected by the server).
- **No shared definition.** Server accepts and client refuses: v0.9, the basic-catalog URL, paths without `/`. Client accepts and server refuses: `updateComponents` without `root`. Executable filters differ.
- **Two client reducers:** `web_core` in chat, `reduceA2uiMessage` in the console.

## 7. Continuations

A continuation is a new run in the same conversation with an empty log and ring. The action is recorded on the source run after its terminal event. Client: one action per (run, surface), source must be done; server: repeats allowed, no status check. Embedded hosts skip the declared-action check.

## 8. Tests, specs, docs (pointers)

- Rust unit: `realtime.rs:165-260`, `protocol.rs:330-389`, `sse.rs:1232-1453`, `presentation_history_tests.rs:163`, `embedded.rs:1282-1365`.
- Integration: `tests/sidecar_session_principal/isolation.rs`, `tests/run_retention/runs.rs`, `tests/integration/live/*a2ui*`; draft `tests/bdd/features/drafts/a2ui-live-update.feature`.
- Behaviors: C14 (`a2ui-response`, asserts delta shapes, not that they apply), C15 (`a2ui-browser`). I19 is only a catalog row.
- Specs: `ag-ui-chat-conformance:55-70`, `runtime-console:201-215`, `a2ui-testing-ui:44-58`, `runtime-event-replay-entity-sync`, `a2ui-uar-renderer`.

## Agent-reported surprises

1. `realtime.rs:1-37` module doc says there is no emission call site; false.
2. Nothing in the chat UI reads surface state; state consumers are the adapter, the trace and PGlite only.
3. C14 uses the tool's own example (v0.9, basic catalog), which the client renderer and console reject; C14 passes because it is API-only.
4. The backbone and the stream can diverge (backbone written before scrubbing and before admission suppression).
5. The action gate is weak: prefix match on unescaped ids, deleted surfaces pass, name matched anywhere, test trigger writes into the gate's log.
6. Durable history holds the events, but every A2UI route 404s once a run leaves the active map.
