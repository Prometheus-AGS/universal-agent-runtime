# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 3)

- **Stage:** Plan. Written 2026-10-28. Nothing here is implemented; it stops
  before Execute.
- **Supersedes:** revision 2 (`plan-rev2.md`, whose independent review is
  appended there). Revision 1 is in `plan-rev1.md`.
- **Operator direction for revision 3:**
  - add `skip_serializing_if` plus a test against a row written before the
    upgrade;
  - fold each binding over its own cursor range;
  - fix warnings 3–9;
  - add a click observable that doesn't depend on the model, and a real late
    attach;
  - have a fresh critic review the plan, then stop.

Unchanged from revision 2, which its review confirmed: surfaces belong to the
thread, and the surface record does not depend on the 512-event ring. Sections
marked *(r3)* changed in this revision.

## 1. The outcome a user sees

An application on UAR's dedicated AG-UI route
(`/api/uar/ag-ui/agents/{agent_id}`), using an AG-UI A2UI client (here PEM
4.3.0's headless runtime), can:

1. **See the agent's UI.** A surface the agent publishes with `a2ui_render` is
   rendered by the client.
2. **Click and be answered.** Pressing a button sends that exact action as a new
   run on the same thread, and the agent receives it.
3. **Keep using the surface.** The same button works again later, including
   after a typed follow-up.
4. **Reconnect or join late and still see it.** A client that attaches to the
   thread's latest run sees the thread's live surfaces without the agent running
   again.

Today none of these work. On 2026-10-28, I19 on the Mac fixture showed the run
reporting success while the client received no surface. The route's projector
drops every A2UI event (`src/uar/api/agui.rs:651`, `_ => Vec::new()`).

## 2. Operator decisions applied

| Decision | Applied to this slice |
|---|---|
| D1 (combination R, AG-UI activity carriage) | Adopted for this slice. Surfaces travel as `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`. |
| D3 (incremental carriage) | Snapshots only: each surface is sent as a cumulative snapshot with `replace: true`. `ACTIVITY_DELTA` is deferred. |
| D2, D4 (durable record, reconnect) | Partly pulled in, by operator instruction. The thread's surface record is *derived* from durable run history plus live run events. No new store is added. Client-declared versions stay deferred. |
| D5–D17 | Deferred. |

## 3. Upstream contract followed

The source is `upstream-research.md`, plus PEM 4.3.0 (§6).

- **Surface events.** Each surface is one event:
  `{type: "ACTIVITY_SNAPSHOT", messageId, activityType: "a2ui-surface", content: {a2ui_operations: [...]}, replace: true}`.
  The operations are cumulative (upstream-research.md §L125-L129).
- **Actions.** An action arrives in the next run's
  `forwardedProps.a2uiAction`, as
  `{userAction: {name, surfaceId, sourceComponentId, timestamp, context}, actionId, heldVersion?, dataModel?}`
  (PEM `forwarded-props.ts`; `runtime-core.ts:736-755`).
  - **`actionId`.** It is a UUIDv7 that PEM generates for each dispatched
    action. PEM uses it as the idempotency key and reuses it on retry
    (`forwarded-props.ts:52`).
  - **Context values.** web_core's `dispatchAction` passes the action's
    `context` through without resolving any bound value (`@a2ui/web_core`
    0.12.0 `src/state/surface-model.js:103-130`). This is why the click
    observable (§4 S4) does not rely on context.
- **Activity messages in history.** Clients should strip `role:"activity"`
  messages from history (upstream-research.md:106). UAR also tolerates them
  (S3).

## 4. Changes

There is one OpenSpec change, `agui-a2ui-activity-slice` (§7), and one Rust
writer for every file below.

- **New module.** The logic goes in `src/uar/api/agui_a2ui.rs`, so `agui.rs`
  (740 lines) stays under the 800-line ceiling.
- **`agui.rs`.** It holds only the call sites.
- **Storage changes:**
  - `src/uar/persistence/agui_runs.rs` gains one field;
  - `src/uar/runtime/manager.rs` gains one read method.

### S0. *(r3)* The binding field, and why existing rows survive it

- **The field.** `AguiRunBinding` gains:

  ```rust
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub a2ui_action: Option<A2uiActionBinding>,
  ```

  It sits in `src/uar/persistence/agui_runs.rs:5-21`.
  `A2uiActionBinding` is `{action_id, digest, surface_id, name, source_component_id}`,
  with `digest = hex(sha256(surfaceId \0 name \0 sourceComponentId \0 workspace_id))`.
  Rust field names stay snake case, because the struct has no `rename_all`.
- **Why it's safe.** Both stores detect concurrent changes by comparing the
  stored JSON with a fresh serialization of the in-memory row:
  - Surreal: `$old.data != $previous`, `providers/agui_runs/surreal.rs:12`, and
    `WHERE data=$before`, `:42`;
  - Postgres: `data=$3`, `providers/agui_runs/postgres.rs:45-46`.

  With `skip_serializing_if`, a row without an action serializes exactly as it
  did before the upgrade, so both comparisons still match. Revision 2 left this
  out; it would have broken every existing thread.
- **Tests:**
  - **Unit:** `agui_run_binding_without_action_serializes_byte_identically`, in
    `agui_runs.rs` `#[cfg(test)]`. It takes a JSON string captured from today's
    serializer, deserializes it with the new type, and re-serializes it. The
    output must be byte-identical. Execute captures the fixture string from the
    *current* code before changing the struct.
  - **Integration:** `agui_run_transition_accepts_pre_upgrade_row`, in
    `tests/post_lifecycle_main.rs`. That is the target whose support file,
    `tests/support/post_lifecycle_store.rs:10`, `:83-91`, already drives
    `create_agui_run` against `SurrealDbProvider`. The test inserts a row
    serialized without the field, then transitions and seals it with the new
    code. Both steps must succeed. (`tests/uar_integration.rs` has no AG-UI run
    tests; its lines 83–113 are environment setup.)

### S1. *(r3)* The thread surface record

The record is a pure fold, computed on demand and never stored:

```text
ThreadSurfaces(thread, anchor_ordinal) =
  fold over the thread's bindings with ordinal ≤ anchor_ordinal,
  newest 64 of those (MAX_THREAD_BINDINGS_SCANNED = 64), read oldest-first,
  of each binding's published A2UI messages within that binding's cursor range
```

- **Anchor.** The window is anchored to a binding's ordinal, never to "now". A
  stream for binding *B* always folds the bindings up to and including *B*. A
  reconnect to an older run therefore recomputes the same window (revision 2,
  warning 3).
- **Per-binding cursor range.** A warm resume and a cold denial reuse the
  predecessor's `native_run_id` (`agui.rs:325`), so reading by native id
  double-counts (revision 2, critical 2). Instead:
  - **Range:** each binding contributes only events with
    `start_cursor < id ≤ terminal.source_cursor` from its `native_run_id`.
    A cold denial has `start_cursor = 0` and no terminal events of its own: it
    is sealed with a synthetic `RUN_ERROR` at cursor 0 (`agui.rs:371-374`), so
    it contributes nothing.
  - **Open binding:** a binding with no terminal yet contributes up to the
    current tip of its native run. Only the anchor binding can be open, because
    the latest-run rule closes all earlier ones.
  - **Deduplication:** messages are deduplicated by native run id and event id
    across bindings, so a shared range can never fold twice.
- **Event source.** A new manager method, `published_a2ui_events(run_id, after,
  up_to)`, returns `NormalizedEvent::ArtifactDisplay` (a2ui) events in the
  range.
  - **Sources:** it reads durable run history
    (`PersistenceLayer::load_run_history_events`, `manager.rs:9172-9190`) and,
    for a run still in `active_runs`, its live ring. The two are merged and
    deduplicated by event id. The ring covers the asynchronous history writer's
    lag (`manager.rs:253-300`).
  - **Why not the realtime backbone.** It is lossy: `createSurface` keeps only
    `{surfaceId, catalogId}` (`a2ui/realtime.rs:69-97`). Only `ArtifactDisplay`
    carries the raw validated messages (`a2ui_output.rs:173-206`).
- **Completeness tracking** *(r3, revision 2 warning 7)*.
  - **Incomplete range:** if a binding's range is closed and the merged events
    reach `terminal.source_cursor` with gaps, the binding is marked
    `history_incomplete`. Gaps are judged against the event ids the binding's
    own stream projected; the check is "missing ids in `(start_cursor,
    source_cursor]` that the ring no longer holds and durable history lacks".
    This happens when a durable write failed: the writer only logs a failed
    write (`manager.rs:276-283`).
  - **Read errors:** a read error for a binding marks it `history_unavailable`.
- **Fold rules.**
  - **Parsing.** Each content line is re-parsed with `protocol::parse_message`.
  - **Per-surface state.** The record keeps:
    - the ordered operation list, minus UAR's top-level `profile` field (S2);
    - the current component table, where a later `updateComponents` replaces
      components by id, as web_core does;
    - the set of components *reachable from `root`* in that table. *(r3,
      revision 2 suggestion 10)* An orphaned Button is not reachable, so it is
      not actionable, which matches what a renderer shows.
  - **Deletion.** `deleteSurface` removes the surface; a later `createSurface`
    for the same id starts fresh.
- **Bounds** *(r3, revision 2 warning 6)*.
  - **Byte limit.** The serialized `a2ui_operations` of a surface may be at
    most `MAX_SURFACE_SNAPSHOT_BYTES = 786_432` (768 KiB). That is under PEM's
    1 MiB limit per call (`limits.ts:52`; it throws at
    `runtime-core.ts:439-447`), which leaves room for the envelope.
  - **Over the limit.** Such a surface is marked `over_limit`: it is not
    restated, and actions on it are refused.
  - **No separate operation limit.** Bytes are what fail at the client.
- **Without durable run history.** When `supports_durable_run_history()` is
  false, every binding whose native run has left memory is
  `history_unavailable`. The owned Mac fixture uses embedded `surrealkv`, which
  counts as `LocalDurable` (`persistence/providers/surreal.rs:422-433`).
- **Cost** *(stated, not hidden)*.
  - **Reads per fold:** up to 64 durable history reads. Each loads every event
    of that run (`run_history/postgres.rs:35-38`; the Surreal equivalent).
  - **Folds per action:** two, one in `run()` for the check and one in
    `stream()` for the restatement.
  - **Mitigation:** `run()` passes its fold to the first `stream()` call of the
    same request. A reconnect folds once.
  - **No cache across requests.** Any cache would need invalidation; that is
    deferred to D2.

### S2. *(r3)* Outbound: project surfaces as activities

All changes go in `Projector` and `stream()` in `agui.rs:627-700`, which
delegate to `agui_a2ui.rs`.

- **`messageId`.** Each surface's id is stable across runs, reconnects and
  resumes:
  - **Derivation:** `"a2ui:" + hex(sha256(thread_id \0 surfaceId))[..32]`.
  - **Why it's safe to resend:** PEM 4.3.0 re-applies a redelivered
    `replace: true` snapshot without changing anything
    (`activity-ledger.ts:115-117`).
- **Restatement frames** *(r3, revision 2 warning 3)*.
  - **Position:** right after `RUN_STARTED`, which is frame
    `(row.start_cursor, 0)` (`agui.rs:680`), the stream emits one
    `ACTIVITY_SNAPSHOT` per live surface of
    `ThreadSurfaces(thread, row.ordinal − 1)` as frames
    `(row.start_cursor, 1)` through `(row.start_cursor, N)`.
  - **Warm resume:** the record also includes the anchor binding's predecessor
    range.
  - **Why the ids can't collide:** the first source event's frames are
    `(event.id, index+1)` with `event.id > start_cursor` (`agui.rs:692`).
  - **Reconnect:** a `Last-Event-ID` past `(start_cursor, N)` skips them through
    the existing `(cursor, index)` filter (`:691-692`).
  - **Order:** surfaces are ordered by their first `createSurface` in the fold,
    which is deterministic.
- **The action echo frame** *(r3)*. If the binding carries an `a2ui_action`, it
  is followed by frame `(row.start_cursor, N+1)`:
  `{type: "CUSTOM", name: "uar.a2ui.action.accepted", value: {actionId, surfaceId, name, sourceComponentId}}`.
  - **Source:** it comes from the stored binding, so a reconnect re-emits the
    identical frame.
  - **Meaning:** this is the server's receipt that the run was admitted for
    that exact action (S4 uses it).
- **Restatement failure.** If the fold returns `history_unavailable` for any
  binding, the restatement is replaced by one frame `(row.start_cursor, 1)`:
  `{type: "CUSTOM", name: "uar.a2ui.restatement_unavailable"}`. Surfaces from
  complete bindings are still restated after it, and the stream continues.
  Behaviour is therefore defined, and it is deterministic as long as the
  storage state doesn't change.
- **Updates during the run.**
  - **On publication:** when an `ArtifactDisplay` (a2ui) arrives, the projector
    applies the batch to its working copy of the record. For each touched
    surface, it emits one snapshot carrying that surface's full cumulative
    operations.
  - **Over the limit:** a surface that now exceeds the byte limit emits nothing
    and is marked `over_limit`.
  - **Deletion:** `deleteSurface` emits a final snapshot whose operations end
    with the delete.
- **Order.** `ArtifactDisplay` precedes `ToolEnd` (`a2ui_output.rs:89-90`;
  `manager.rs:8444-8463`). A surface snapshot therefore precedes the
  `a2ui_render` `TOOL_CALL_RESULT`.
- **What is forwarded.** Each message is forwarded exactly, except that UAR's
  top-level `profile` is removed. UAR's types accept it (`protocol.rs:40-41`,
  `:57`, `:202`, `:219`), but web_core's strict schemas reject it.
- **Reconnect.** The stream re-projects from `row.start_cursor`
  (`agui.rs:679`). With S1's per-binding ranges and anchored window, a
  reconnect reproduces the same restatement, echo, snapshots and frame ids.
- **Unchanged.** State patches and non-A2UI artifacts remain unprojected on
  this route.
- **Other clients.**
  - **Conforming clients:** they ignore activity types they don't render.
  - **Our clients of this route:** the regressions in §8 cover them.

### S3. *(r3)* Inbound: `forwardedProps.a2uiAction` as a same-thread run

These changes are in `run()` and `create_request()` in `agui.rs`, with
`agui_a2ui.rs` holding the logic.

- **Parse.** The input is the top-level `forwardedProps.a2uiAction`, parsed
  with closed structs that reject unknown fields at both levels.
  - **`A2uiAction`:** `userAction` is required. `actionId` is optional, 1 to
    128 characters. `heldVersion` is optional and recorded only. `dataModel` is
    an optional object.
  - **`UserAction`:** `name`, `surfaceId` and `sourceComponentId` are strings of
    1 to 256 characters. `context` is an optional object and `timestamp` an
    optional string.
  - **Size limits:** `userAction` is capped at 16 KiB serialized and
    `dataModel` at 64 KiB.
  - **Failure:** anything else returns `400 invalid_a2ui_action`.
- **Order of checks.**
  1. **Same `runId`:** the existing replay applies (`agui.rs:185-189`),
     unchanged.
  2. **Retry by `actionId`:** if a binding on the thread carries the same
     `actionId`:
     - if its `digest` and workspace match this request, the existing run's
       stream is replayed and nothing new is admitted, even when the `runId` is
       fresh;
     - if not, the request gets `409 a2ui_action_id_conflict` (revision 2,
       warning 4).
  3. **Latest-run rule:** the existing rule applies.
  4. **Declaration check:** this request is checked against
     `ThreadSurfaces(thread, latest.ordinal)`. It is accepted only if all of
     these hold:
     - the surface is live;
     - the surface is not `over_limit`;
     - the binding that published it is not `history_unavailable` or
       `history_incomplete`;
     - a reachable `Button` exists with `id == sourceComponentId` and
       `action.event.name == name`;
     - the keys of `context` are a subset of the declared `action.event.context`
       keys.
  5. **Refusal codes:** a failed check returns
     `422 a2ui_action_not_declared`, `422 a2ui_surface_over_limit`, or
     `422 a2ui_surface_history_unavailable`. The last one is returned whenever
     the surface id is absent from the record and any scanned binding is
     incomplete or unavailable. This keeps an operator from confusing a lost
     surface with a forged action (revision 2, warning 7).
  6. **Reserve:** the run is reserved through `create_agui_run`, as today.
  7. **Losing a reservation race** *(r3, revision 2 warning 4)*: if
     `create_agui_run` returns `false`, the thread is re-listed once and step 2
     is applied again. A concurrent twin of the same action is then replayed;
     any other conflict keeps today's `409 thread_run_conflict`.
- **Building the action run.**
  - **Shared identity stripping** *(r3, revision 2 warning 5)*. The builder
    calls the same helper `create_request` uses after this refactor. That
    helper removes `artifact`, `deployment_binding_id`, `input`, `agent_id`,
    `session_id` and `history` from `forwardedProps.uar` (`agui.rs:408`), and
    inserts the path agent and the verified session. A client therefore can't
    choose the agent or the session through an action.
  - **History:** every prior input message, including the last assistant
    message, is kept. Nothing is split off or overwritten.
  - **Client tools:** if present, the action input is appended as a new user
    message.
  - **Action plus new user text:** the request gets
    `400 a2ui_action_with_text`.
  - **Input:**
    `{"type": "a2ui.user_action", "sourceRunId": <native run id of the publishing binding>, "interaction": {surfaceId, name, sourceComponentId, timestamp, context, actionId, a2uiClientDataModel, clientContext}}`.
  - **Stated honestly** *(r3)*: this JSON *is* the run's user turn. Client
    values (`context`, `a2uiClientDataModel`, `clientContext`) reach the model
    as user-supplied content, the same trust level as anything a user types.
    They are size-bounded and do not change the run's authority: the agent, the
    policy, the tools and the session all come from the path and the binding.
- **Activity history messages.** On every run, `role: "activity"` messages are
  dropped from history before validation. Today they fail with
  `400 unsupported_history_message` (`agui.rs:75-78`).
- **Divergence from REST, stated.** The REST action path
  (`a2ui/routes.rs:827-897`) reuses the source run's policy and presentation
  negotiation (`manager.rs:3162-3187`), validates more loosely, and has no
  `actionId`. It is unchanged here (§10).

### S4. *(r3)* Acceptance: I19, with observables that don't depend on the model

**Files:**
- `scripts/behaviors/features/agui-route-a2ui.feature`
- `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`

```gherkin
Scenario: An AG-UI app shows the agent's surface, keeps answering its button, and late clients see it
  Given a disposable agent that publishes an action surface on the dedicated AG-UI route
  And a PEM 4.3.0 headless AG-UI client configured with UAR's catalog ids
  When the client asks the agent for a fresh ticket surface
  Then the client renders the surface with the requested heading and a Confirm button
  When the user presses the rendered Confirm button
  Then UAR admits a new run that carries exactly that click's actionId
  And the agent's reply contains that actionId
  When the user presses the same Confirm button again
  Then UAR admits another run that carries the second click's different actionId
  And the agent's reply contains the second actionId
  When the client retries the second click with PEM's retryAction
  Then UAR replays the second run and does not admit a new one
  When a fresh client attaches to the thread's latest run without sending any message
  Then that client renders the same surface and no model call happens
  When a forged client sends an action the surface never declared
  Then UAR refuses it and the thread's latest run is unchanged
```

**The client.** It is `@prometheus-ags/a2ui-react@4.3.0` `./headless` in Node,
pinned from npm in the behaviour-test dependencies.
- **Ingest:** every stream event is fed through `processAgUiEvent`.
- **Catalogs:** two headless catalogs are registered, under
  `https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json` and
  `urn:uar:a2ui:catalog:1` (`protocol.rs:264`).
- **Action policy:** `executed`.

**The click.**
- **Finding the button:** the test locates the Confirm button through
  `componentsModel.getAll()`. It is the `Button` whose child `Text` reads
  "Confirm". The test reads its action from
  `properties.action.event`.
- **Dispatching:** the test calls `dispatchAction` with that action and that
  component id. No action name or component id is typed.
- **Sending:** PEM's `onActionEnvelope` supplies `forwardedProps` and
  `actionId`, and the test sends `forwardedProps` unchanged.

**Observables.** Each is model-independent except where noted.
- **Click received.** The new run's stream contains the
  `uar.a2ui.action.accepted` echo frame (S2). Its `actionId` equals the
  envelope's. PEM generates a fresh UUIDv7 per click, and neither the test nor
  the model can know it in advance. The echo is UAR's receipt that the exact
  click became this run.
  - The second click carries a different `actionId`, which proves the second
    click, not a replay or an echo.
- **The model saw it.** The reply contains the `actionId` (*model-dependent*:
  copying a UUID from the input). This is a secondary check. The agent's
  instructions require it. If the reply omits the id, the scenario fails, with
  the echo recorded so the cause is visible.
- **Retry.**
  - **Request:** `retryAction(actionId)` resends the same envelope, and the
    test posts it with a *fresh* `runId`.
  - **Pass:** the response's `RUN_STARTED.runId` equals the second run's wire
    `runId`, and its echo carries the same `actionId`.
  - **Follow-up:** a following text run with `parentRunId` set to the second run
    is accepted. That proves no new run became the latest.
- **Late attach.**
  - **Client:** a fresh PEM headless runtime, with no prior state.
  - **Request:** a `GET` to
    `/api/uar/ag-ui/agents/{agent}/threads/{thread}/runs/{latest}/stream` with no
    `Last-Event-ID`, the existing reconnect route at `agui.rs:17-18`.
  - **Pass:** the restatement frames render the surface, with the same
    `messageId`, the heading and the Confirm button. This is an attach, not a
    model turn: no run is admitted, and the latest run is unchanged.
- **Forged action.** This is the one hand-built request, explicitly a hostile
  client: it copies the envelope and changes `userAction.name` to
  `doesNotExist`.
  - **Pass:** it gets `422 a2ui_action_not_declared`.
  - **Follow-up:** a following text run with `parentRunId` set to the previous
    latest run is accepted, so nothing was admitted.

**Bounds.** The model is `liter/MiniMax-M3`. No credentials are used. There are
four model turns: the publication, two clicks, and the follow-up text run.

## 5. *(r3)* Unit tests (authored with the code, run only at the gate)

| Case | Test | File |
|---|---|---|
| A pre-upgrade row is byte-identical after re-serialization | `agui_run_binding_without_action_serializes_byte_identically` | `src/uar/persistence/agui_runs.rs` |
| A pre-upgrade row still transitions and seals | `agui_run_transition_accepts_pre_upgrade_row` | `tests/post_lifecycle_main.rs` |
| Durable history beyond the ring, with ring and durable merged | `published_a2ui_events_reads_durable_beyond_ring`, `published_a2ui_events_merges_ring_and_durable` | `src/uar/runtime/manager.rs` tests |
| Per-binding ranges: a warm resume and a cold denial share a native run without double counting | `fold_uses_per_binding_cursor_ranges` | `agui_a2ui.rs` |
| An anchored window gives a stable restatement for an older run | `restatement_window_anchored_to_binding_ordinal` | `agui_a2ui.rs` |
| Restatement frame ids `(start_cursor, 1..N)` and the echo `(start_cursor, N+1)` | `restatement_and_echo_frame_ids` | `agui_a2ui.rs` |
| Snapshot before `TOOL_CALL_RESULT` | `snapshot_precedes_tool_result` | `agui_a2ui.rs` |
| Delete then recreate, and an unreachable Button | `fold_deletes_then_recreates_fresh`, `orphan_button_not_actionable` | `agui_a2ui.rs` |
| Byte bound | `over_limit_surface_not_restated_or_actionable` | `agui_a2ui.rs` |
| `profile` stripped and nothing else changed | `forwarded_messages_strip_profile_only` | `agui_a2ui.rs` |
| Incomplete or unavailable history gives distinct refusal codes | `incomplete_history_refuses_with_history_unavailable` | `agui_a2ui.rs` |
| Action checks: wrong component, wrong name, extra key, oversize, malformed, action plus text | `action_check_*` (6 tests) | `agui_a2ui.rs` |
| `actionId` replay, a digest mismatch, and a lost race | `action_id_retry_replays_bound_run`, `action_id_digest_mismatch_conflicts`, `action_id_race_rechecks_after_create_conflict` | `agui_a2ui.rs` |
| The action builder keeps the identity stripping | `action_run_strips_forwarded_identity` | `agui_a2ui.rs` |
| `role: "activity"` messages are dropped | `activity_history_messages_are_dropped` | `agui_a2ui.rs` |

## 6. PEM 4.3.0 client contract

This comes from PEM source at `b7796047` (matching `ded34ef0`) in
`packages/a2ui-react`, with the API names confirmed by both reviews.

- **Entry.** `createPrometheusA2uiHeadlessRuntime` from
  `@prometheus-ags/a2ui-react/headless` (`src/headless.ts:50-54`).
- **Ingest.** `processAgUiEvent` handles `ACTIVITY_SNAPSHOT`,
  `ACTIVITY_DELTA` and `MESSAGES_SNAPSHOT` (`runtime-core.ts:261-277`).
  - It rewrites `version: "v0.9"` to `v0.9.1` (`:399`).
  - A redelivered `replace: true` snapshot is a no-op.
  - Each call is limited to 1 MiB (`limits.ts:52`).
- **Catalogs.** The default id is `urn:prometheus-ags:a2ui:catalog:v3`
  (`catalog-core.ts:19`). An unknown id fails the batch (`runtime-core.ts:425-430`,
  `:848-874`). The id can be overridden (`catalog-core.ts:130-131`).
- **State.** `getSurface(id)` returns a `SurfaceModel`; components are read
  through `componentsModel.get` and `componentsModel.getAll`.
- **Actions.**
  - `dispatchAction(action, sourceComponentId)` runs under an `executed` policy,
    then calls `onActionEnvelope({actionId, surfaceId, forwardedProps})`.
  - `retryAction(actionId)` resends the same envelope.

## 7. Spec delta (OpenSpec `agui-a2ui-activity-slice`)

This adds a new capability, `ag-ui-a2ui-activities`, with four requirements.

1. **Published A2UI surfaces are thread objects on the AG-UI route.**
   - *The client receives one cumulative snapshot per touched surface, before
     the tool result.*
   - *Every later run of the thread restates the surface, with the same
     `messageId`, in frames `(start_cursor, 1..N)`.*
   - *A late client that attaches to the latest run renders it without a model
     call.*
   - *A deleted surface is not restated.*
   - *With durable run history, a surface published more than 512 events
     earlier is restated and actionable.*
   - *A surface over the byte limit is neither restated nor actionable.*
   - *A thread with no surface produces no activity.*
2. **Declared A2UI actions are accepted as same-thread runs.**
   - *A reachable, declared Button action is admitted, its echo carries the
     `actionId`, and the run input is `a2ui.user_action`.*
   - *The same button works again after its answer.*
   - *An undeclared component, name or context key, or an orphaned Button, is
     refused with 422, and nothing is admitted.*
   - *A retry with the same `actionId` replays, even with a new `runId` and
     under a race. A different action with a reused `actionId` is refused with
     409.*
   - *A lost or unavailable surface is refused with
     `a2ui_surface_history_unavailable`, not `a2ui_action_not_declared`.*
3. **Forwarded A2UI messages carry no UAR-only fields.**
   - *`profile` is stripped.*
4. **Existing AG-UI run bindings remain valid after the upgrade.**
   - *A binding stored before this change still serializes byte-identically,
     transitions and seals.*

## 8. *(r3)* Verification (exact commands; not run; the gate is closed)

The gate opens only on the operator's instruction, after a recorded
implementation-complete inventory. The steps run serially.

1. **Unit tests, by name:**
   `cargo test --locked --no-default-features --features server-full --lib -- agui_a2ui:: agui_runs::tests:: published_a2ui_events_`
   (libtest runs every test that matches any of these filters).
2. **The AG-UI run store integration**, which exercises the compare-and-swap
   path this change touches:
   `cargo test --locked --no-default-features --features server-full --test post_lifecycle_main -- agui_run_transition_accepts_pre_upgrade_row`.
   This runs only the new test. The rest of `post_lifecycle_main` is the
   phase's main-boundary harness, outside this slice.
   `agui_golden` is not listed: it checks a static fixture against a schema
   (`tests/agui_golden.rs:4-17`), not this route.
3. **Rebuild the owned binary in place.** **Open precondition:** the records do
   not name the command that built the owned binary at
   `target/uar-team-concurrency-repair/debug/universal-agent-runtime`. Before
   the gate request, Execute must find that command, including its feature set
   and target directory, in the phase records or shell history, and put it
   here verbatim. The candidate is
   `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
   It must not be run until it is confirmed.
4. **Restart the owned UAR** with the restart helper, passing the new binary's
   digest:
   `node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r))" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"`.
5. **Set the endpoint and other variables.** The base URL is read from the
   runtime record each time, not hardcoded:
   `export UAR_BASE_URL="$(node -p "require('./.prometheus/verification/manual-runtime.json').base_url")" UAR_MODEL=liter/MiniMax-M3 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace`.
6. **I19, once:**
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs`.
7. **Regression, once each, in this order:** `selection-stream-replay` (E10),
   `live-roster-routing` (C34), `native-tool-approval` (C37), `agui` (it runs
   `agui-math.feature`) and `writing` (it runs `writing-orchestrator.feature`).
   These are the only step files other than I19's that call
   `/api/uar/ag-ui/agents/`; this was confirmed on 2026-10-28. Each runs as
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`.

C14 and C15 are not rerun: they cover the chat stream and the console, which are
untouched.

## 9. What the slice leaves for the deferred decisions

| Later decision | How the slice stays compatible |
|---|---|
| D3 (deltas) | `ACTIVITY_DELTA` can be added per surface under the same `messageId`. |
| D2 (stored record) | A stored record would replace S1's derived fold. The wire shape, frame ids and checks stay the same. It would also remove the 64-binding window and the per-request fold cost. |
| D4 (versions) | `heldVersion` is already parsed and recorded. |
| D7 (compaction) | It would relieve the byte limit. |
| D11, D15 (catalogs) | The catalog-id mismatch (§10) is theirs to resolve. |
| D17 | The new capability name avoids a collision. |

## 10. The uncomfortable parts

- **An out-of-the-box PEM 4.3.0 client still renders nothing from UAR.**
  - **The mismatch:** PEM defaults to `urn:prometheus-ags:a2ui:catalog:v3`,
    while UAR publishes `urn:uar:a2ui:catalog:1` or the basic v0.9 URL.
  - **The consequence:** an app on default PEM settings gets "Catalog not
    found". I19 passes only because its client registers UAR's ids.
  - **The fix:** that belongs to D11/D15. The change's proposal must document
    the client setting.
- **Every run and every click re-folds up to 64 bindings' durable history.**
  - **The cost:** long threads make each run and each click slower.
  - **The window:** surfaces more than 64 bindings back drop out. Clicks on
    them are refused as `a2ui_action_not_declared`, unless an incomplete
    binding makes the code `history_unavailable`.
  - **The fix:** D2's stored record.
- **A failed durable write is detected only as a gap.**
  - **Coverage:** `history_incomplete` catches gaps inside the ranges the record
    knows about.
  - **The blind spot:** if the failed write was the only event in a binding,
    and its ring has gone, the gap can't be seen, so that surface's loss reads
    as "not declared".
  - **Mitigation:** the history writer already records its failures. Surfacing
    them is the remaining mitigation, and it isn't part of this slice.
- **Restating every live surface on every run grows the streams.** That's fine
  for chat cards and costly for a full San Saba-style application until D3 and
  D7 land.
- **Context is checked by key, not value.** A2UI resolves bound values on the
  client; web_core doesn't resolve them at dispatch. The values are therefore
  user-supplied content in the run's user turn: bounded in size, with no effect
  on the run's authority.
- **The REST action path stays looser,** and it has no `actionId`.
- **A restart isn't exercised by I19.** The derived record across a restart is
  covered only by `published_a2ui_events_reads_durable_beyond_ring` and the
  durable read path. No end-to-end restart run is part of the gate.
- **The rebuild command is still unknown** (§8 step 3). That is an open
  precondition, not a settled step.

## 11. Tasks (for Execute; none started)

1. **OpenSpec.** Create the `agui-a2ui-activity-slice` change with the §7
   delta. `openspec validate` waits for the gate.
2. **The binding field.** Before changing the struct, capture the current
   serializer's output for a fixture binding. Then add `a2ui_action` with
   `skip_serializing_if`, and author both pre-upgrade tests (S0).
3. **The manager read.** Add
   `published_a2ui_events(run_id, after, up_to)`, which merges durable and ring
   events and deduplicates by event id, plus its tests.
4. **`agui_a2ui.rs`.** Write:
   - the fold, with per-binding ranges, the anchored window, completeness
     tracking, reachability and the byte limit;
   - the `messageId` derivation and `profile` stripping;
   - the action parse, checks and builder, using the shared identity-stripping
     helper extracted from `create_request`;
   - unit tests from §5, authored but not run.
5. **`agui.rs` wiring.** Wire in restatement, the echo and per-batch snapshots;
   the action check order, including the race re-check; and the dropping of
   activity history messages.
6. **I19.** Rewrite I19 (S4), and add `@prometheus-ags/a2ui-react@4.3.0` to the
   behaviour-test dependencies.
7. **Confirm the build command** for §8 step 3.
8. **Stop for the gate.** Record the implementation-complete inventory, then
   wait for the operator's gate instruction.

## Revision 3 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 3. It ran no builds or tests;
its only command read two JSON fixture fields. Verdict: **defects found**, with
2 critical defects, 6 warnings and 3 suggestions. The lead confirmed the starred
items in source (marked ✓).

### Confirmed correct

These revision 3 claims held up against the source:

- **S0 serialization survives in every comparison site:**
  - Surreal: create (`surreal.rs:12`, `:20`) and transition (`:41-42`, `:56`);
  - Postgres: jsonb compare (`postgres.rs:45-46`) and struct equality on create
    (`:17-18`);
  - `validate_transition` (`agui_runs.rs:61-64`).
- **Only the anchor binding can be open** (`agui_runs.rs:51`; `agui.rs:250`).
- **Per-binding ranges are right for all four binding kinds:** fresh, warm
  resume, cold denial `(0,0]`, and client-tool continuation.
- **§8 is accurate:** the regression list, the restart helper's argument order,
  the `base_url` key, the binary path, and the reconnect route shape.

### Critical

1. **✓ The pre-upgrade integration test can't run as written, or proves
   nothing.**
   - **Can't open a store:** `post_lifecycle_main`'s store opener requires
     `storeFixtureApprovalRef`, a dedicated `uar_post_lifecycle_*` loopback
     store manifest and Root credentials (`post_lifecycle_main.rs:249-273`;
     `post_lifecycle_store.rs:52-57`). The §8.2 command supplies none of them.
   - **Proves nothing:** written through `create_agui_run` instead, the new code
     with `None` writes the old bytes, so the test only repeats the unit test.
   - **Coverage gap:** Postgres is never exercised.
2. **✓ The operation cap PEM hits first is not the byte cap.** PEM's defaults
   include `maxOperationsPerCall: 500` (`limits.ts:51`, enforced at
   `limit-checks.ts:131`), alongside `maxSurfaces: 64` and
   `maxComponentsPerSurface: 2_000`.
   - Cumulative snapshots grow with every update, so a long-lived surface passes
     500 operations.
   - PEM then refuses every restatement, while UAR keeps accepting clicks on a
     surface nobody can see. That breaks outcomes 3 and 4, and I19, with one
     small surface, never notices.

### Warnings

3. **Frame ids collide on paths I19 doesn't run.**
   - **Frames fixed to `terminal.source_cursor`:** terminal `(…,100)`, close
     `(…,90)` and caller-tool frames `(…,1..3k)` (`agui.rs:699-714`), plus
     error frames `(cursor,99)` while `cursor == start_cursor` (`:682`, `:696`,
     `:718`).
   - **Where they meet restatement frames:** a cold denial has
     `source_cursor == start_cursor == 0`, and caller-tool recovery seals at
     `start_cursor` when history is empty (`:515`, `:548`). Restatement frames
     `(start,1..N)` therefore collide there, and with 90/99/100 once N ≥ 89.
   - **Unbounded and undefined:** N is unbounded, and the echo index is
     undefined when `restatement_unavailable` takes frame 1.
4. **The race re-check is unreachable on Surreal.** `create` maps only a thrown
   `agui_context_conflict` to `false`; a `TransactionConflict` returns `Err`
   (`surreal.rs:22`), which becomes `503` (`agui.rs:350`). A unit test can't
   show store behaviour.
5. **The closed `UserAction` struct rejects valid web_core actions.**
   `dispatchAction` adds `catalogId` and `userMessage` when present
   (`surface-model.js:125-132`).
6. **"web_core doesn't resolve context at dispatch" is wrong for a real
   press.**
   - **The real path:** a press goes through `GenericBinder`, which resolves
     `event.context` before dispatch (`generic-binder.js:331-346`, `:382`).
   - **The test's path:** I19 calls `SurfaceModel.dispatchAction` directly, so
     it never runs the real press path.
7. **The claim "authority comes only from the path and the binding" is
   overstated.**
   - **What's stripped:** the shared helper strips six keys (`agui.rs:408`).
   - **What stays client-supplied:** `model`, `mcp_servers`, `tool_admission`,
     `run_credentials` and `working_directory` (`routes.rs:73-100`).
8. **Rolling back the binary isn't covered.** An older binary drops
   `a2ui_action` when it re-serializes, so its compare-and-swap check misses.
   `seal` then returns the unsealed row (`agui.rs:476-477`), and the thread
   stays open, refusing later runs with `thread_run_not_closed`.

### Suggestions

- **Gap detection:** gaps can only be detected by contiguity, because event
  ids are sequential (`manager.rs:126-127`). The real blind spot is `reconcile`
  swallowing read failures with `unwrap_or_default()` (`agui.rs:514`), which
  can seal a binding at `start_cursor` with an empty range.
- **S4 details:**
  - it has five model turns, not four;
  - "no model call" is inferred, never observed;
  - a replay whose `RUN_STARTED.runId` differs from the request's departs from
    AG-UI convention.
- **§8 gaps:** it is missing the `@prometheus-ags/a2ui-react@4.3.0` install
  (published on npm on 2026-10-28) and `openspec validate`.
  `CreateRunRequest.input` is a `String`.

### Lead assessment

Revision 3 resolved revision 2's two critical defects:

- **Existing rows survive:** S0 keeps pre-upgrade rows valid in every
  comparison site.
- **Ranges are right:** per-binding ranges are correct for every binding kind.

The two new critical findings are of a different kind: a missing limit, and a
test that can't run. Neither is a design flaw.

- **Limits:** the record must enforce every PEM default that a cumulative
  snapshot can hit, not just bytes: operations per call (500), components per
  surface (2,000) and surfaces (64). Better still, it should apply a margin
  below them.
- **The pre-upgrade test:** it needs a store that this repository can open
  without the main-boundary manifest, against both Surreal and Postgres, or an
  honest statement that the unit-level byte-identity test is the guarantee.

Warnings 3 and 6 show the same pattern: I19's path is the easy one. The plan
should either cap and partition the frame-index space (a reserved restatement
range), or move restatement onto its own cursor. It should also drive a press
through PEM's binder rather than calling `dispatchAction` directly.

The pattern across three reviews has been several critical findings per
revision, each narrower than the last. The design has stayed stable since
revision 2; what keeps surfacing is edge cases.
