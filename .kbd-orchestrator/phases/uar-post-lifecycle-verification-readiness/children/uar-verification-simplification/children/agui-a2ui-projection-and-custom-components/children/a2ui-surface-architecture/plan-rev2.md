# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 2)

- **Stage:** Plan. Written 2026-10-28. Nothing in this plan is implemented, and
  it stops before Execute.
- **Supersedes:** revision 1 (`plan-rev1.md`). Its independent review is
  appended there.
- **Operator direction for revision 2:** keep a thread-scoped record of
  published surfaces, built from durable run history plus live run events, for
  both the action check and the snapshot restatement. Fix the warnings, have a
  fresh critic review the plan, then stop.

## 1. The outcome a user sees

An application connected to UAR's dedicated AG-UI route
(`/api/uar/ag-ui/agents/{agent_id}`) with an AG-UI A2UI client, here PEM 4.3.0's
headless runtime, can do four things:

1. **See the agent's UI.** When the agent publishes a surface with `a2ui_render`,
   the client renders it.
2. **Click, and the agent answers.** Pressing a button sends the action as a new
   run on the same thread, and the agent answers that action.
3. **Keep using the surface.** The same button works again on a later click,
   including after a typed follow-up.
4. **Reconnect or join late and still see it.** A reconnecting client, or a new
   client starting a new run on the thread, sees the thread's live surfaces.

Today, none of these happen. I19 on Mac, 2026-10-28: the agent publishes, the
run reports success, and the client receives no surface. The route's projector
drops every A2UI event (`src/uar/api/agui.rs:651`, `_ => Vec::new()`).

Revision 1 met outcomes 1 and 2 only for a single click in a short run. Its
review found two problems:

- **Its surfaces belonged to one run.** The run that answered a click became the
  thread's latest run, so the next click on the same card was refused.
- **It read the live 512-event ring.** A chatty run could evict its own surface
  (`manager.rs:46`, `:9150-9190`).

Revision 2 makes the surface a thread object, which matches what a user expects.

## 2. Operator decisions applied

| Decision | Applied to this slice |
|---|---|
| D1 (combination R, AG-UI activity carriage) | Adopted for this slice. Surfaces travel as `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`. |
| D3 (incremental carriage) | Started with snapshots only. Each surface is a cumulative snapshot with `replace: true`; `ACTIVITY_DELTA` is deferred. |
| D2, D4 (durable record, reconnect) | Partly pulled in, by operator instruction. The thread surface record is *derived* from durable run history plus live events. No new store, table or mirror is added (§4 S1). Client-declared versions (D4) stay deferred. |
| D5, D6–D17 | Deferred. |

## 3. Upstream contract followed

These shapes come from `upstream-research.md` and from PEM 4.3.0 (§6):

- **Carriage.** Each surface is one `ACTIVITY_SNAPSHOT` event:
  `{type: "ACTIVITY_SNAPSHOT", messageId, activityType: "a2ui-surface", content: {a2ui_operations: [...]}, replace: true}`.
- **Cumulative content.** The operations accumulate, so a single snapshot is
  enough to render the surface (upstream-research.md §L125-L129).
- **Actions.** The client sends an action in the next run's
  `forwardedProps.a2uiAction`:
  - PEM 4.3.0 sends
    `{userAction: {name, surfaceId, sourceComponentId, timestamp, context}, actionId, heldVersion?, dataModel?}`
    (`forwarded-props.ts`; `runtime-core.ts:736-755`);
  - `actionId` is PEM's idempotency key, and it is reused on retry
    (`forwarded-props.ts:52`).
- **History.** Clients should strip `role: "activity"` messages from history
  (upstream-research.md:106). UAR tolerates them anyway (S3).

## 4. Changes

- **One OpenSpec change:** `agui-a2ui-activity-slice` (§7).
- **One Rust writer** owns both files.
- **New module.** The logic moves into a new module,
  `src/uar/api/agui_a2ui.rs`, so `agui.rs` (740 lines) stays under the
  800-line ceiling and the logic can be unit-tested without the HTTP stack.
- **`agui.rs` gets only call sites.**

### S1. The thread surface record (new: `agui_a2ui.rs`)

A pure fold, computed on demand, never stored:

```text
ThreadSurfaces = fold over the thread's runs, oldest ordinal first,
                 of each run's published A2UI messages, in event-id order
```

- **Runs.** The thread's runs come from `db.list_agui_runs(owner, agent, thread)`,
  which `run()` already loads (`agui.rs:244`). The fold reads at most
  `MAX_THREAD_RUNS_SCANNED = 64` runs: the 64 newest by ordinal, read
  oldest-first. Older runs are outside the window (§10).
- **Published messages of one run.** These are the run's
  `NormalizedEvent::ArtifactDisplay` events with `artifact_type == "a2ui"`, read
  from two sources:
  - **durable run history:** `PersistenceLayer::load_run_history_events(native_run_id)`
    (`manager.rs:9172-9190`). It is written for every emitted event by the run
    history writer (`manager.rs:253-300`) and is not bounded by the 512-event
    ring;
  - **the live ring:** for runs still in `active_runs`, merged with the durable
    rows to cover the asynchronous writer's lag;
  - **merging:** the two are combined and deduplicated by event id. A new
    manager method, `published_a2ui_events(run_id)`, returns this merge. It
    reads durable history even while the run is active, unlike
    `history_since`.
- **Why not the backbone.** The live realtime backbone (`InMemoryReplayBackbone`)
  is not used as a source. Its operations are lossy: `createSurface` keeps only
  `{surfaceId, catalogId}` (`a2ui/realtime.rs:69-97`). Only `ArtifactDisplay`
  carries the raw validated messages (`a2ui_output.rs:173-206`). This changes
  the operator's "live backbone" wording to "live run events", for that reason.
- **Fold rules.**
  - Each `content` line is re-parsed with `protocol::parse_message`. A line that
    fails to parse is skipped and counted. Lines cannot normally fail, because
    `publish_tool_output` admitted them.
  - Per surface, the record keeps the ordered operation list as sent, minus
    UAR's top-level `profile` field (S2). It also keeps the current component
    table (id → component), with the latest `updateComponents` replacing by
    component id, as web_core does (upstream-research.md:38).
  - `deleteSurface` removes the surface from the record.
  - A later `createSurface` for the same id starts a fresh operation list.
- **Without durable run history.** When `supports_durable_run_history()` is false
  (non-durable stores, a remote Surreal store that is not attested), the fold
  uses live runs only. Surfaces from runs that have left memory are absent:
  - an action on one is refused with `422 a2ui_surface_history_unavailable`;
  - restatement omits it.

  The owned Mac fixture is an embedded `surrealkv` store, which counts as
  `LocalDurable` (`persistence/providers/surreal.rs:422-433`), so durable
  history is on there.
- **Bounds.** At most 64 runs are scanned. Each surface's operation list is
  capped at `MAX_SURFACE_OPERATIONS = 256`; past the cap, that surface is marked
  `over_limit`. An over-limit surface is not restated, and actions on it are
  refused with `422 a2ui_surface_over_limit`. Compaction (D7) is what would
  lift this.

### S2. Outbound: project surfaces as activities

All changes are in the `Projector` in `agui.rs:627-652`, which delegates to
`agui_a2ui.rs`.

- **messageId.**
  - The id is stable per thread and surface:
    `"a2ui:" + hex(sha256(thread_id + "\u0000" + surfaceId))[..32]`.
  - The same surface keeps the same id across runs, reconnects and warm
    resumes. PEM 4.3.0 re-applies a redelivered `replace: true` snapshot with no
    change (`activity-ledger.ts:115-117`), so restating is idempotent.
  - The hash keeps raw surface ids, which are model-chosen strings, out of a
    protocol identifier.
- **Restatement at run start.**
  - **When:** each run's stream emits, right after `RUN_STARTED`, one
    `ACTIVITY_SNAPSHOT` per live surface in the thread record. The record covers
    every *earlier* run's events and, for a warm resume, this native run's
    events before `start_cursor`.
  - **Sent once:** the restatement is part of the deterministic replay. It is
    keyed to the source cursor of the run's first event, so a `Last-Event-ID`
    past it does not resend it.
  - **What it fixes:**
    - a new client on the thread renders existing surfaces;
    - a warm-resumed run's later updates have a base, which was the critic's
      warm-resume warning;
    - a thread that has published no surface emits nothing.
- **Updates during the run.**
  - **Trigger:** on `ArtifactDisplay` (a2ui), the projector applies the batch to
    its working copy of the record.
  - **Output:** it then emits one `ACTIVITY_SNAPSHOT` per surface the batch
    touched, carrying that surface's full cumulative operation list, including
    operations from earlier runs.
  - **Deletion:** a `deleteSurface` emits a final snapshot whose operations end
    with the `deleteSurface`, which is how PEM 4.3.0 deletes. The surface then
    leaves the record.
- **Order.** `publish_tool_output` emits `ArtifactDisplay` *before* `ToolEnd`
  (`a2ui_output.rs:89-90`; `manager.rs:8444-8463`). Each snapshot therefore
  precedes the `a2ui_render` `TOOL_CALL_RESULT`, and unit tests assert that
  order.
- **Messages are sent as A2UI defines them.**
  - Each forwarded message has UAR's top-level `profile` removed.
  - UAR accepts and records that field (`protocol.rs:40-41`, `:57`, `:202`,
    `:219`), but web_core's strict schemas reject unknown fields, and PEM then
    refuses the whole batch.
  - Nothing else is altered.
- **Reconnect.** The projector already re-projects from `row.start_cursor` and
  filters only the output by `(cursor, index)` (`agui.rs:679`, `:691-692`). A
  reconnect therefore yields the same snapshots with the same ids and frame ids.
- **Unchanged.** `StatePatch` and non-A2UI artifacts stay unprojected on this
  route. Text, tool, approval, skill, agent and terminal events keep their
  current shapes.
- **Other clients.**
  - Existing clients of this route see the new `ACTIVITY_SNAPSHOT` events
    interleaved.
  - AG-UI defines the event, so a conforming client ignores activity types it
    doesn't render.
  - Our own clients of the route get the regression reruns in §8.

### S3. Inbound: `forwardedProps.a2uiAction` as a same-thread run

All changes are in `run()` and `create_request()` in `agui.rs`, with parsing and
checks in `agui_a2ui.rs`.

- **Parse.**
  - **Where:** the top-level `forwardedProps.a2uiAction`, not
    `forwardedProps.uar`.
  - **Structs:** closed, with unknown fields rejected at both levels:
    - `A2uiAction` has `userAction` (required), `actionId` (optional, 1–128
      chars), `heldVersion` (optional `{epoch, version}`, recorded, otherwise
      ignored until D4) and `dataModel` (optional object);
    - `UserAction` has `name`, `surfaceId` and `sourceComponentId` (strings of
      1–256 chars), `context` (optional object) and `timestamp` (optional
      string).
  - **Size limits:** `userAction` serialized must be at most 16 KiB, and
    `dataModel` at most 64 KiB.
  - **Failure:** anything malformed or over size returns
    `400 invalid_a2ui_action`.
- **Order of checks.** The existing code already handles a same-`runId`
  replay first (`agui.rs:185-189`). The new checks follow:
  1. **Retry by `actionId`.** If any run on the thread is bound to the same
     `actionId` (a new optional `a2uiActionId` field on `AguiRunBinding`, with a
     serde default), stream that run (replay) and admit nothing new. This holds
     even when the `runId` is fresh, which is PEM's `retryAction` pattern.
  2. **The existing latest-run rule:** the predecessor must be closed, and
     `parentRunId` must name it. A second, *different* action while a run is
     open still gets `409`.
  3. **The declaration check** against the thread surface record (S1):
     - the surface must exist in the record's current state, after deletes;
     - the surface must not be `over_limit`;
     - the current component table must contain a `Button` with
       `id == sourceComponentId` and `action.event.name == name`;
     - the keys of `userAction.context` must be a subset of the declared
       `action.event.context` keys. Values are not compared, because A2UI
       resolves bound values on the client.

     If any check fails, the request gets `422`, with
     `a2ui_action_not_declared`, `a2ui_surface_history_unavailable` or
     `a2ui_surface_over_limit`. No run is reserved or executed.
- **Building the run.** This is a separate builder, so the user-text path is
  untouched.
  - **History:** all prior input messages, *including the last*, which is the
    assistant's reply. Nothing is split off or rewritten.
  - **Input:** the action becomes the run's input:
    `{"type": "a2ui.user_action", "sourceRunId": <native run id that published the surface>, "interaction": {surfaceId, name, sourceComponentId, timestamp, context, actionId, a2uiClientDataModel}}`.
    - `a2uiClientDataModel` carries the client's `dataModel` under the field
      name the REST path uses (`a2ui/routes.rs:865`).
    - Client `context` and `state` go in as a separate field,
      `interaction.clientContext`, never appended to the input text. The input
      stays valid JSON.
  - **Client tools:** when the request also carries client tools, the canonical
    history gets the action input as a *new* user message at the end. The
    assistant message is never overwritten (fixing the warning about
    `agui.rs:423-426`).
  - **Action plus text:** an action together with a new trailing user text
    message returns `400 a2ui_action_with_text`.
- **Activity history messages.** For every run, not only action runs,
  `role: "activity"` messages in the history are dropped before validation
  instead of failing with `400 unsupported_history_message`
  (`agui.rs:75-78`). The only behavior change is that requests rejected today
  are now accepted.
- **Identity.** Same thread, same session, admitted through
  `routes::admit_run`, with governance, approvals and the effective policy
  applied as for any run. No echo event is added; revision 1's
  `uar.a2ui.action.accepted` is dropped.
- **Stated plainly.** The REST action path (`a2ui/routes.rs:827-897`) differs:
  - it reuses the source run's effective policy and presentation negotiation
    (`manager.rs:3162-3187`);
  - its validation stays looser;
  - it has no `actionId`.

  Tightening it is out of scope (§10).

### S4. Acceptance: I19, rewritten as the user outcome

Files:

- `scripts/behaviors/features/agui-route-a2ui.feature`
- `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`

The 2026-10-28 single-step version's recorded failure stays in the session log.

```gherkin
Scenario: An AG-UI app shows the agent's surface and keeps answering its button
  Given a disposable agent that publishes an action surface on the dedicated AG-UI route
  And a PEM 4.3.0 headless AG-UI client configured with UAR's catalog ids
  When the client asks the agent for a fresh ticket surface
  Then the client renders the surface with the requested heading and a Confirm button
  When the user presses the rendered Confirm button
  Then the agent's answer acknowledges that action as the first on this thread
  When the user presses the same Confirm button again
  Then the agent's answer acknowledges it as the second on this thread
  When the client retries the second action with the same actionId
  Then UAR replays the second run and the agent does not run again
  When a fresh client joins the thread with a new run
  Then that client renders the same surface without the agent republishing it
  And an action the surface never declared is refused and runs nothing
```

**The client is the real one.** The test drives
`@prometheus-ags/a2ui-react@4.3.0` `./headless` in Node:

- **Install:** pinned exactly from npm, in the behaviour test dependencies.
  4.3.0 was published on 2026-10-28.
- **Ingest:** every stream event goes through `processAgUiEvent`.
- **Catalogs:** two headless catalogs, registered under
  `https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json` and
  `urn:uar:a2ui:catalog:1`, the two UAR admits (`protocol.rs:264`).
- **Action policy:** `executed`.

**The click is genuine.**

- **Locating the button:** the test finds the Confirm button in the rendered
  surface with `componentsModel.getAll()`, a component whose type is `Button`,
  whose child text is "Confirm", and whose action it reads from the component's
  own `properties.action.event`.
- **Pressing it:** it calls `dispatchAction` with that action and that component
  id.
- **Sending it:** PEM's `onActionEnvelope` supplies `forwardedProps`, which the
  test sends unchanged. The test never types an action name or a component id.

**Observables.**

- **Rendered heading:** a `Text` component carries the requested heading,
  which is random per scenario.
- **Acknowledgement:**
  - the agent's instructions say: "When the input is an a2ui.user_action, reply
    exactly `ACK <name> <count>`, where count is how many actions you have
    received in this conversation";
  - the test checks `ACK <the name it read from the button> 1`, then `… 2`;
  - this proves the agent saw the action, its name, and the thread's history.
- **Retry:** the retry returns the second run's frames, and the thread's run
  count is unchanged.
- **Late join:** a new client sends a user-text run on the same thread, asking
  "list my open surfaces in one word". Its first activity is the restated
  surface, with the same `messageId`, and it renders. The test asserts that no
  `a2ui_render` tool call happens in that run.
- **Refusal:** an action naming `doesNotExist` gets `422 a2ui_action_not_declared`,
  and no new run appears.

**Bounds.** One model, `liter/MiniMax-M3`. No credentials. Four model turns.

## 5. What I19 does not cover, and where it is covered

These cases are covered by Rust unit tests in
`agui_a2ui.rs` `#[cfg(test)] mod tests`, authored with S1–S3 and run only at
the gate:

| Case | Unit test |
|---|---|
| The surface outlives a 512-event ring (more than 512 events after the publication in a live run) | `thread_record_reads_durable_history_beyond_ring` |
| Durable writer lag (the surface is only in the ring) | `thread_record_merges_live_ring_and_durable` |
| Warm resume after an approval (create before, update after) | `restatement_seeds_warm_resume_from_prior_events` |
| `deleteSurface`, then recreate | `fold_deletes_then_recreates_fresh` |
| `profile` is stripped and nothing else changes | `forwarded_messages_strip_profile_only` |
| Snapshot before `TOOL_CALL_RESULT` | `snapshot_precedes_tool_result` |
| Wrong component, wrong name, extra context key, oversized, malformed, action plus text | `action_check_*` (six tests) |
| `actionId` replay with a fresh `runId` | `action_id_retry_replays_bound_run` |
| `role:"activity"` history messages dropped | `activity_history_messages_are_dropped` |
| No durable history: earlier-run surface refused | `no_durable_history_refuses_old_surface` |
| Over-limit surface | `over_limit_surface_not_restated_or_actionable` |

## 6. PEM 4.3.0 client contract

From PEM source at `b7796047`, `packages/a2ui-react`, read by a read-only
agent, with the review confirming the API names.

- **Entry.** `createPrometheusA2uiHeadlessRuntime` from
  `@prometheus-ags/a2ui-react/headless` (`package.json:27-30`;
  `src/headless.ts:50-54`). It needs no DOM; `zod` is its only runtime
  dependency.
- **Ingest.**
  - `processAgUiEvent(event)` handles `ACTIVITY_SNAPSHOT`, `ACTIVITY_DELTA` and
    `MESSAGES_SNAPSHOT` (`runtime-core.ts:261-277`).
  - `version: "v0.9"` is rewritten to `"v0.9.1"` (`:399`).
  - A redelivered `replace: true` snapshot is re-applied with no change.
- **Catalog.** The default id is `urn:prometheus-ags:a2ui:catalog:v3`
  (`catalog-core.ts:19`). An unknown id fails the batch with `Catalog not found`
  (`runtime-core.ts:425-430`, `:848-874`), so the client must register UAR's
  ids through the catalog `id` option (`catalog-core.ts:130-131`).
- **State.**
  - `getSurface(id)` returns a `SurfaceModel`, with components through
    `componentsModel.get`/`getAll` and data through `dataModel.get`.
  - `exportSurfaceMessages(id)` is also available.
- **Action.**
  - `dispatchAction({event: {name, context}}, sourceComponentId)` on the
    `SurfaceModel` triggers it, with a policy of
    `{handle: async (a) => ({status: "executed", action: a, result: null})}`.
  - `onActionEnvelope` then receives `{actionId, surfaceId, forwardedProps}`.
  - `retryAction(actionId)` resends the same envelope.

## 7. Spec delta (OpenSpec `agui-a2ui-activity-slice`)

The change adds a new capability, `ag-ui-a2ui-activities`. It is not the
superseded change's `ag-ui-agent-endpoint`, to avoid a collision while D17 is
open. It has three requirements:

1. **Published A2UI surfaces are thread objects on the AG-UI route.**
   - *A published surface reaches the client:* one cumulative
     `ACTIVITY_SNAPSHOT` per touched surface, before the tool result.
   - *A surface is restated to every later run of the thread:* the same
     `messageId`, cumulative operations.
   - *A deleted surface is not restated.*
   - *The thread record survives the live ring:* with durable run history, a
     surface published more than 512 events ago is restated and actionable.
   - *No surface, no activity.*
2. **Declared A2UI actions are accepted as same-thread runs.**
   - *A declared Button action is answered:* the run input is
     `a2ui.user_action`.
   - *The same button works again after its answer.*
   - *An undeclared component, name or context key is refused with 422 and runs
     nothing.*
   - *A retry with the same `actionId` replays, even with a new `runId`.*
3. **Forwarded A2UI messages carry no UAR-only fields.**
   - *`profile` is stripped:* the forwarded operations validate against the
     A2UI schema for their version.

## 8. Verification (proposed exact commands; not run; the gate is closed)

The gate opens only on the operator's instruction, after a recorded
implementation-complete inventory. The commands run serially:

1. Unit tests for the new module:
   `cargo test --locked --no-default-features --features server-full --lib uar::api::agui_a2ui`.
2. The existing integration golden for this route's events, kept green:
   `cargo test --locked --no-default-features --features server-full --test agui_golden`.
3. Rebuild the owned UAR binary, then restart it through the existing owned-UAR
   restart helper:
   `cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
4. I19 once:
   `UAR_BASE_URL=http://127.0.0.1:60675 UAR_MODEL=liter/MiniMax-M3 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs`.
5. Regression, once each and in this order, using the same environment
   variables. These are every behaviour scenario that drives
   `/api/uar/ag-ui/agents/`:
   - `selection-stream-replay` (E10);
   - `live-roster-routing` (C34);
   - `native-tool-approval` (C37);
   - `agui`, which runs `agui-math.feature`;
   - `writing`, which runs `writing-orchestrator.feature`.

   Each runs as
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`.
   These names were confirmed against `scripts/behaviors/*.cucumber.mjs` on
   2026-10-28: they are the only step files that call
   `/api/uar/ag-ui/agents/`, other than I19's own.

C14 and C15 are not rerun: the chat stream and the console are untouched.

## 9. What the slice leaves for the deferred decisions

| Later decision | How the slice stays compatible |
|---|---|
| D3 deltas | `ACTIVITY_DELTA` can be added per surface under the same stable `messageId`. |
| D2 (a durable surface record) | A stored record would replace S1's derived fold. The wire shape and checks stay the same. |
| D4 (declared versions) | `heldVersion` is already parsed and recorded. |
| D7 (compaction) | It replaces the 256-operation and 64-run caps. |
| D11, D15 (catalogs and the renderer) | The catalog-id mismatch (§10) is theirs to fix. |
| D17 | The new capability name avoids a collision. |

## 10. The uncomfortable parts

- **An out-of-the-box PEM 4.3.0 client still renders nothing from UAR.** PEM
  defaults to the catalog id `urn:prometheus-ags:a2ui:catalog:v3`, and UAR
  publishes `urn:uar:a2ui:catalog:1` or the basic v0.9 URL. An application with
  default PEM options sees `Catalog not found`, and I19 passes only because its
  client registers UAR's ids. Until D11/D15 agree on one id, the change's
  proposal must document the client configuration. This slice does not fix it.
- **The thread record is recomputed on every run and every action.**
  - **Cost:** one durable history read per scanned run, up to 64. A long thread
    with many runs therefore makes each new run and each click slower.
  - **The window:** surfaces published more than 64 runs back fall out of it.
    The user sees them disappear from late-joining clients, and a click on one
    is refused with `422 a2ui_action_not_declared`.
  - **Why accepted:** the operator chose a derived record over a stored one.
    D2's stored record removes both effects.
- **Surfaces depend on durable run history.** On a store without it, a surface
  works only while its run is in memory, and fails closed afterwards. That is
  safe, but users of such deployments see surfaces stop working after the
  run, or the process, ends.
- **Restatement grows every run's stream.** Each run on a thread resends every
  live surface in full. That is fine for a few chat cards; for a San Saba-style
  full application, every turn carries the whole UI until D3 deltas and D7
  compaction land.
- **Action context is checked by key, not by value.** A2UI resolves bound values
  on the client, so the server can't check them. A client can send any values
  for the declared keys. The values reach the model as untrusted data inside
  `interaction`, never as instructions, and within the size limits. Agents that
  act on those values must still treat them as user input.
- **The two action paths disagree.** The REST path stays looser and has no
  `actionId`.
- **Restatement isn't covered across a restart.** I19 doesn't restart UAR. It
  relies on the unit test `thread_record_reads_durable_history_beyond_ring` and
  on the durable history read path that E-series reconnects already use.
  Neither has been run against a restart for this exact flow.

## 11. Tasks (for Execute; none started)

1. **OpenSpec.** Create `agui-a2ui-activity-slice` with the §7 delta.
   `openspec validate` is a check, so it waits for the gate.
2. **Manager.** Add `published_a2ui_events(run_id)`: durable history merged with
   the live ring, deduplicated by event id.
3. **Fold.** Write `agui_a2ui.rs`: the thread record fold, the bounds, the
   `messageId` derivation and the `profile` stripping, with unit tests §5
   authored but not run.
4. **Projector.** Wire restatement and per-batch snapshots into `Projector` and
   `stream()` in `agui.rs`.
5. **Action path.** In `agui.rs`, add the action parse and checks, the action
   run builder, the `a2uiActionId` binding field (serde default) and the
   activity-message drop, with unit tests authored but not run.
6. **Acceptance.** Rewrite I19 (S4), and add
   `@prometheus-ags/a2ui-react@4.3.0` to the behaviour test dependencies.
7. **Gate.** Record the implementation-complete inventory, then wait for the
   operator's gate instruction (§8).

## Revision 2 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 2 against UAR and against PEM
at `ded34ef0`, whose 4.3.0 source matches `b7796047`. It ran nothing. Verdict:
**defects found**, with 2 critical defects, 7 warnings and 1 suggestion. The
lead confirmed the starred items in source (marked ✓).

### Critical

1. **✓ The new binding field breaks every existing persisted AG-UI thread.**
   - `AguiRunBinding` has no `skip_serializing_if` (`agui_runs.rs:5-21`).
   - The stores compare the stored JSON with the re-serialized binding: Surreal
     with `$old.data != $previous` (`providers/agui_runs/surreal.rs:12`) and
     `WHERE data=$before` (`:42`), Postgres with `data=$3`
     (`postgres.rs:45-46`).
   - A pre-upgrade row re-serializes with `"a2ui_action_id": null` and no longer
     matches. The next run on an existing thread gets `409 thread_run_conflict`,
     and a run that is live across the upgrade can never seal.
   - Fix: `#[serde(default, skip_serializing_if = "Option::is_none")]` on
     `a2ui_action_id`, plus a test with a pre-upgrade row.
2. **✓ The fold double-counts native runs that two bindings share.**
   - A warm resume and a cold denial reuse the predecessor's `native_run_id`
     (`agui.rs:325`), but only a warm resume sets `start_cursor` (`:326`).
   - Reading by native id returns the whole history to every binding. For a
     run still being resumed, the restatement then depends on timing, so it
     isn't deterministic; update-only batches count twice.
   - Fix: fold each binding over `(start_cursor, terminal.source_cursor]`, and
     deduplicate by native id and event id.

### Warnings

3. **The restatement's frame-id key is mis-specified.** The first event's frames
   already use `(event.id, index+1)` (`agui.rs:692`). Collision-free
   restatement frames are `(row.start_cursor, 1..N)`, after RUN_STARTED's
   `(start_cursor, 0)` (`:680`). The 64-run window must be anchored to
   `row.ordinal`, or reconnecting to an older run recomputes a different
   window.
4. **The `actionId` replay loses a race and checks no identity.**
   - Two concurrent requests with the same `actionId` and fresh `runId`s both
     pass. The context-slot transaction admits one, and the other gets `409`,
     not a replay.
   - The replay doesn't check that the bound action matches this one, or the
     workspace (cf. `agui.rs:188`, `:201`).
   - Fix: re-check by `actionId` after a create conflict, and reject a
     mismatched action.
5. **The separate action builder must keep `create_request`'s identity
   stripping.** That stripping removes `artifact`, `deployment_binding_id`,
   `input`, `agent_id`, `session_id` and `history` from `forwardedProps.uar`
   (`agui.rs:408`). The claim that the values arrive "never as instructions" is
   unsupported: the JSON input is the user turn.
6. **The bounds count operations, not bytes.**
   - PEM caps each call at 1 MiB (`limits.ts:52`; it throws at
     `runtime-core.ts:439-447`), and a surface with 256 operations can exceed
     that.
   - Each scan loads every event of up to 64 runs, twice per action (once in
     `run()` and once in `stream()`).
7. **A failed durable write silently loses a surface.** The writer logs and
   continues (`manager.rs:276-283`). The resulting refusal reads
   `a2ui_action_not_declared`, not `history_unavailable`, and fold read errors
   inside `stream()` have no defined behavior.
8. **The tests don't prove the claimed outcomes.**
   - **ACK count:** the action travels in `forwardedProps`, not as a message,
     so the count only shows the model echoing its previous "ACK 1". An exact
     output from MiniMax-M3 is also likely to be flaky.
   - **Run counts:** the route offers no way to list runs, so "run count
     unchanged" and "no new run" have no observable. RUN_STARTED's `runId`
     equal to the second run's id is one.
   - **Late join:** a new run with copied history is not a late attach. The
     "no `a2ui_render`" check depends on the model, and `presentation_render`
     also publishes.
   - **`doesNotExist`:** this refusal requires a hand-typed action.
   - **Misplaced tests:** the two merge tests belong to `manager.rs` (Task 2),
     so `--lib uar::api::agui_a2ui` wouldn't run them.
9. **§8's scope is overstated, and its commands are not exact.**
   - `agui_golden` is a static fixture checked against a schema
     (`tests/agui_golden.rs:4-17`), not this route's events.
   - The compare-and-swap code that `tests/uar_integration.rs:83-113` exercises
     is out of scope, yet the binding change touches it.
   - The restart helper isn't named.
   - Port 60675 is hardcoded, but the port changes between restarts.

### Suggestion

10. **Merge-by-id never removes components.** A Button that a later update drops
    from the tree still passes the declaration check. UAR's backbone replaces
    the whole list (`realtime.rs:82-86`).

### Lead assessment

Both critical defects are real and concrete, and each has a narrow fix:
`skip_serializing_if` plus a pre-upgrade-row test, and per-binding cursor ranges
with deduplication. Neither defect changes the design.

Warnings 3–7 are specification gaps that revision 3 can close. Warning 8 is the
most important for the operator's goal of proving user outcomes:

- **Repeated clicks:** the acceptance needs an observable that doesn't rest on
  the model echoing itself. For example, the button's declared context carries
  a value only the action input contains, and the agent must report it.
- **Late join:** it must be a real attach (a reconnect or `MESSAGES_SNAPSHOT`),
  not a new model turn.

Revision 2 correctly fixed revision 1's two critical defects: surfaces now
belong to the thread, and the record no longer depends on the 512-event ring.
