# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 1)

Stage: Plan. Written 2026-10-28. Nothing in this plan is implemented. It stops
before Execute for the operator.

## 1. The outcome a user sees

An application connected to UAR's dedicated AG-UI route
(`/api/uar/ag-ui/agents/{agent_id}`) with an AG-UI A2UI client, here PEM 4.3.0's
headless runtime, does two things it cannot do today:

1. **It shows the agent's UI.** When the agent publishes a surface with
   `a2ui_render`, the client receives it and renders it.
2. **The user's click reaches the agent.** When the user presses a button on that
   surface, the client sends the action as a new run on the same thread, and the
   agent answers it.

Today the agent publishes, the run reports success, and the client receives
no surface at all. This was measured on Mac on 2026-10-28 (I19,
`.prometheus/verification/agui-route-a2ui/`). The cause is in
`src/uar/api/agui.rs:651`: the route's projector ends in `_ => Vec::new()`,
which drops every A2UI event.

## 2. Operator decisions applied

| Decision | Applied to this slice |
|---|---|
| D1 (combination R, AG-UI activity carriage) | Adopted for this slice only: surfaces travel as `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`. |
| D3 (incremental carriage) | Started with snapshots only: each surface goes out as a cumulative snapshot with `replace: true`. `ACTIVITY_DELTA` is deferred. |
| D2, D4–D17 | Deferred. Nothing here builds the durable surface record, the server mirror, client-declared versions, push outside a run, compaction, scopes, morphing or the renderer replacement. |

The slice is shaped so that each deferred decision can be added later without
undoing it (§8).

## 3. Upstream contract followed

These shapes come from `upstream-research.md` and from PEM 4.3.0 (§5):

- **Carriage.** Each surface is sent as one `ACTIVITY_SNAPSHOT` event:
  `{type: "ACTIVITY_SNAPSHOT", messageId, activityType: "a2ui-surface", content: {a2ui_operations: [...]}, replace: true}`.
- **Cumulative content.** `content.a2ui_operations` holds every operation
  published for that surface, so any single snapshot is enough to render it.
  This follows `@ag-ui/a2ui-middleware` (upstream-research.md §L125-L129).
- **Actions.** The client sends an action in the next run's input as
  `forwardedProps.a2uiAction.userAction`, with the fields
  `{name, surfaceId, sourceComponentId, context, timestamp}`
  (upstream-research.md §L133-L135).

## 4. Changes

There is one OpenSpec change, `agui-a2ui-activity-slice`, and one Rust writer
owns `src/uar/api/agui.rs`.

### S1. Outbound: project published surfaces as activities

- **Where.** `src/uar/api/agui.rs`, in `Projector` (struct at line 627) and
  `Projector::project` (line 635).
- **Source event.** `NormalizedEvent::ArtifactDisplay` with
  `artifact_type == "a2ui"`. UAR emits it only after the whole batch has passed
  `parse_message` (`src/uar/runtime/a2ui_output.rs:107-206`). Its `content`
  holds the raw messages, newline-joined. Nothing new is admitted, so
  `protocol.rs` remains the one validation gate.
- **Mapping.**
  - Split `content` on newlines and parse each line as JSON.
  - Group the messages by `surfaceId`.
  - Append each group to the projector's per-surface operation list (a new
    `surfaces: BTreeMap<String, Vec<Value>>` field).
  - For every surface the batch touched, emit one snapshot:
    `{"type": "ACTIVITY_SNAPSHOT", "messageId": "<wire_run_id>:a2ui:<surfaceId>", "activityType": "a2ui-surface", "content": {"a2ui_operations": [<cumulative ops>]}, "replace": true}`.
- **Identity.** `messageId` is derived from the wire run and the surface, so a
  redelivered or replayed snapshot carries the same id. PEM 4.3.0 makes that
  idempotent (changelog: "redelivery is idempotent and `replace:true` no longer
  deletes surfaces").
- **Replay and reconnect come for free.** `stream()` always re-projects from
  `row.start_cursor` and filters only the output by `Last-Event-ID`
  (`agui.rs:662-690`). A reconnect therefore rebuilds the same cumulative
  snapshots and does not double them. No storage is added.
- **Unchanged.** `StatePatch` (state) and non-A2UI artifacts stay unprojected on
  this route. The text, tool-call and terminal events keep their current shape
  and order. A snapshot is emitted where its `ArtifactDisplay` occurs, which is
  after the `a2ui_render` tool result and before the assistant's closing text.

### S2. Inbound: accept `forwardedProps.a2uiAction` as a new run on the same thread

- **Where.** `src/uar/api/agui.rs`, in `run()` and `create_request()`.
- **Parse.** Read the top-level `forwardedProps.a2uiAction` (not
  `forwardedProps.uar`) with closed structs. Unknown fields are rejected at
  both levels, and a malformed action returns `400 invalid_a2ui_action`.
  - **`a2uiAction`** has these fields, matching PEM 4.3.0's envelope (§5):
    - `userAction`: required;
    - `actionId`: an optional string;
    - `heldVersion`: an optional `{epoch, version}`;
    - `dataModel`: an optional object.
  - **`userAction`** has these fields:
    - `name`, `surfaceId` and `sourceComponentId`: strings;
    - `context`: an optional object;
    - `timestamp`: an optional string.
- **Validate against what the thread actually published.**
  - **The source run.** The action must refer to a surface published by the
    thread's latest run. That run is `previous`, which the existing latest-run
    rule (`agui.rs:244-252`) already requires to be closed.
  - **Reading the surface.** Read that run's `ArtifactDisplay` (a2ui) events
    from `history_since(previous.native_run_id, None)` and re-parse them with
    `protocol::parse_message`.
  - **Acceptance rule.** Accept only if a `Button` with
    `id == sourceComponentId` and `action.event.name == name` exists on that
    surface. Otherwise return `422 a2ui_action_not_declared`, and execute no
    run.
  - **Why stricter.** The existing REST path (`a2ui/routes.rs:838-857`) accepts
    the name appearing anywhere in the surface. This rule is deliberately
    stricter.
- **Input.** Give the run the same input UAR's existing A2UI path already uses
  (`manager.rs:3156-3161`):
  `{"type": "a2ui.user_action", "sourceRunId": <previous native run id>, "interaction": {surfaceId, name, sourceComponentId, timestamp, context, actionId, a2uiClientDataModel}}`.
  The model sees one input shape for actions from either path.
  - `a2uiClientDataModel` takes the client's `dataModel` under the name the
    REST path already uses (`a2ui/routes.rs:865`). It is untrusted data.
  - `heldVersion` is ignored until D3 or D4 adds versions.
- **Last message.** When an action is present, `create_request` no longer
  requires the last message to be user text. Upstream clients send an action
  with the thread's existing messages, whose last message is the assistant's
  (`agui.rs:395`). The prior messages still become history exactly as today.
  An action and a new user text in the same run is `400 a2ui_action_with_text`:
  one input per run.
- **Identity, replay and admission are unchanged.**
  - Same thread and session.
  - `parentRunId` must name the latest run, as today.
  - The input digest covers the action, so an identical retry replays the same
    run instead of dispatching again (E10's guarantee).
  - Admission goes through `routes::admit_run`, so governance, approvals and the
    effective policy apply as for any run.
- **Echo.** The new run's stream starts with `RUN_STARTED`, followed by one
  `CUSTOM` event `{name: "uar.a2ui.action.accepted", value: {surfaceId, name, sourceComponentId}}`.
  The client can then confirm which action this run answers.

### S3. Tool guidance

- **The problem.** `a2ui_render`'s example and prompt guidance teach v0.9 with
  the basic catalog, and use no `Button`. An agent asked for an interactive
  surface needs one example action.
- **The fix.** Add a `Button` with `action.event.name` to the tool's description
  example (`src/uar/runtime/native_skills/a2ui_render.rs:127-136`). Make no
  schema or version change; D11 is deferred.

### S4. Acceptance: I19, rewritten as the user outcome

Files:

- `scripts/behaviors/features/agui-route-a2ui.feature`
- `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`

Both replace the 2026-10-28 single-step version, whose recorded FAIL stays in
the session log.

```gherkin
Scenario: An AG-UI app shows the agent's surface and the user's click reaches the agent
  Given a disposable agent that publishes a greeting surface with a Confirm button on the dedicated AG-UI route
  When a PEM 4.3.0 headless AG-UI client asks the agent for a fresh greeting surface
  Then the client renders the surface with the requested greeting and a Confirm button
  When the user presses Confirm in the client
  Then the agent answers that exact action in a new run on the same thread
  And reconnecting to the first run re-delivers the same surface without duplicating it
  And an action the surface never declared is refused and runs nothing
```

- **The client is the real one.** The test drives
  `@prometheus-ags/a2ui-react@4.3.0` `./headless` (§5) in Node. Its catalog is
  registered under the basic v0.9 catalog URL the agent publishes with (§5,
  §9), and its action policy is `executed`. It feeds every
  stream event to the runtime, reads the rendered surface from the runtime, and
  triggers the button through the runtime. It builds the next run's
  `forwardedProps` with PEM's own helper.
- **No UAR-side shortcut.** The test never constructs `ACTIVITY_SNAPSHOT` or
  `a2uiAction` JSON by hand, so it proves that the two systems interoperate.
- **Install.** The package is installed in the behaviour test's own dependency
  set, pinned to exactly 4.3.0 from npm. It is never linked to the local PEM
  checkout.
- **Model.** `liter/MiniMax-M3`.
- **Credentials.** None.

## 5. PEM 4.3.0 client contract

Read from PEM source at `b7796047`, `packages/a2ui-react`, by a read-only agent.
Nothing was built or run.

- **Entry.** Import `createPrometheusA2uiHeadlessRuntime` from
  `@prometheus-ags/a2ui-react/headless` (`package.json:27-30`;
  `src/headless.ts:50-54`). It is ESM, needs no DOM, and its only runtime
  dependency is `zod`.
- **Ingest.** `runtime.processAgUiEvent(event)` handles `ACTIVITY_SNAPSHOT`,
  `ACTIVITY_DELTA` and `MESSAGES_SNAPSHOT`, and returns `false` for anything
  else (`src/official/runtime-core.ts:261-277`).
  - The snapshot needs `messageId`, `activityType: "a2ui-surface"` and a
    `content.a2ui_operations` array; `surfaceVersions` is optional.
  - `replace !== false` re-applies a redelivered snapshot, which changes
    nothing.
  - `version: "v0.9"` is rewritten to `"v0.9.1"` (`:399`), so UAR's v0.9
    surfaces are accepted.
- **Catalog id: this mismatch blocks rendering.**
  - **PEM's default:** the headless catalog id is
    `urn:prometheus-ags:a2ui:catalog:v3` (`src/official/catalog-core.ts:19`).
    For an unknown catalog, web_core throws `Catalog not found`, PEM reports
    `batch-refused`/`processing-error`, and no surface changes
    (`runtime-core.ts:425-430`, `:848-874`).
  - **UAR's ids:** UAR publishes `urn:uar:a2ui:catalog:1` or
    `https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json`
    (`src/uar/a2ui/protocol.rs:264`).
  - **The client fix:** the client must register its headless catalog under the
    id UAR publishes, via the catalog's `id` option (`catalog-core.ts:130-131`).
  - **I19 does exactly that:** it registers the basic v0.9 catalog URL, which
    the I19 agent is told to use. This is the client configuration a real
    application would also need. The longer-term agreement on one catalog id is
    a D11/D15 question (§9).
- **Read state.** `runtime.getSurface(id)` returns a web_core `SurfaceModel`:
  - components via `componentsModel.get(componentId)` (`.type`, `.properties`);
  - data via `dataModel.get(path)` (`runtime-core.ts:594`);
  - a lossless export via `exportSurfaceMessages(id)` (`:607-623`).
- **Action.**
  - **Trigger:** `await runtime.getSurface(sid).dispatchAction({event: {name, context}}, sourceComponentId)`.
  - **Policy:** the default action policy denies everything, so the client
    passes `actionPolicy: {handle: async (a) => ({status: "executed", action: a, result: null})}`
    (`src/policy/action-policy.ts:42-58`).
  - **Envelope:** `onActionEnvelope` receives `{actionId, surfaceId, forwardedProps}`.
  - **forwardedProps:** `{a2uiAction: {userAction: {name, surfaceId, sourceComponentId, timestamp, context}, actionId, heldVersion?, dataModel?}}`
    (`runtime-core.ts:736-755`). The client sends it unchanged as the next
    run's `forwardedProps`.
- **No checked-in headless AG-UI action test exists in PEM.** That flow is
  covered only in PEM's browser specs, so I19 is the first headless end-to-end
  use of it.

## 6. Spec delta (OpenSpec `agui-a2ui-activity-slice`)

The change adds a new capability, `ag-ui-a2ui-activities`. It is deliberately
not the `ag-ui-agent-endpoint` capability of the superseded change
`agui-a2ui-projection-and-custom-components`, so that the two don't collide
while D17 is open. It has two requirements:

1. **The route carries published A2UI surfaces as AG-UI activities.**
   - **Scenario: a published surface reaches the client.** When an agent
     publishes one surface, the stream carries exactly one
     `ACTIVITY_SNAPSHOT` per surface touched, with cumulative operations.
   - **Scenario: replay is idempotent.** A reconnect re-delivers the same
     `messageId` and content.
   - **Scenario: no surface, no activity.** A run that publishes no surface
     emits no activity.
2. **The route accepts declared A2UI actions as same-thread runs.**
   - **Scenario: a declared action is answered.** A declared Button action
     starts a run whose input is `a2ui.user_action`.
   - **Scenario: an undeclared action is refused.** An action naming an
     undeclared component or name is refused with 422, and no run executes.
   - **Scenario: an identical retry replays.** Re-sending the identical action
     request replays the admitted run.

## 7. Verification (proposed commands, not run; the phase gate is closed)

The gate opens only on the operator's instruction, after the
implementation-complete inventory:

1. The Rust unit tests authored with S1 and S2 (each test is named in the
   tasks):
   - Projector mapping: one surface, two surfaces, replay;
   - action validation: declared, undeclared name, wrong component, malformed,
     action with text.
2. The single integration target that exercises the real route:
   `cargo test --locked --no-default-features --features server-full --test <agui route test>`.
3. A rebuild of the owned UAR binary and a restart.
4. I19 once, using the same command as every other scenario:
   `UAR_BASE_URL=http://127.0.0.1:60675 UAR_MODEL=liter/MiniMax-M3 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs`.

C14 and C15 are not rerun: their paths (chat stream and console) are not
touched. E10 is rerun once, because S2 touches the admission path that E10's
replay guarantee depends on.

## 8. What this slice leaves for the deferred decisions

| Later decision | How the slice stays compatible |
|---|---|
| D3 deltas | Cumulative snapshots stay valid. `ACTIVITY_DELTA` can be added per surface later, keyed by the same `messageId`. |
| D2, D4, D5 (durable record, reconnect) | The snapshot is rebuilt from process-local run history. A durable surface record replaces only its source; the wire shape doesn't change. |
| D11 (versions) | The route forwards exactly what `parse_message` already admits (v0.9 and v0.9.1). |
| D15 (UAR renderer) | The console path is untouched. |
| D17 (supersede the old change) | The new capability name avoids a collision. Superseding stays the operator's decision. |

## 9. The uncomfortable parts

- **An out-of-the-box PEM 4.3.0 client renders nothing from UAR.** PEM's
  default catalog id is `urn:prometheus-ags:a2ui:catalog:v3`, and UAR publishes
  `urn:uar:a2ui:catalog:1` or the basic v0.9 URL. An application that installs
  PEM with default options sees a `Catalog not found` refusal, not a surface,
  even after this slice ships. I19 passes only because its client is configured
  with UAR's catalog id.
  - **The real fix:** UAR and PEM agree on, or advertise, one catalog id. That
    belongs to D11/D15 and is not part of this slice.
  - **Until then:** document the client configuration in the change's
    proposal.

- **Surfaces don't survive a restart or long histories.** Both the snapshot and
  the action check read process-local, bounded run history. After a UAR
  restart, or once history retention drops the publication, an older surface is
  neither re-delivered nor actionable. The user sees the surface vanish on
  reconnect, and a click is refused with `422`. That is the durable-record
  problem D2, D4 and D5 exist to solve. This slice does not solve it; it fails
  closed instead.
- **Surfaces don't carry across runs.** Cumulative content covers one run
  only. If a later run updates a surface created in an earlier run, that run's
  snapshot holds only its own operations. Rendering it correctly depends on PEM
  4.3.0 keeping the existing surface, which its changelog says it now does.
  I19 does not cover cross-run updates, so this remains unverified until a
  scenario does.
- **Every surface is resent in full.** Without deltas, a frequently updated
  surface is retransmitted completely each time. Fine for chat-sized surfaces;
  not fine for the full-application case (San Saba), which needs D3 deltas.
- **The test depends on PEM's npm release.** I19 needs 4.3.0 on npm, which was
  published on 2026-10-28 (npm `@prometheus-ags/a2ui-react` reports 4.3.0).
- **Two action paths now disagree.** The stricter Button check is new code next
  to the existing, looser REST check (`a2ui/routes.rs:838-857`). Until that path
  is tightened, it accepts actions that this route refuses.

## 10. Tasks (for Execute; none started)

1. Create the OpenSpec change `agui-a2ui-activity-slice` with the §6 delta,
   then validate it with `openspec validate`, which is a check and so waits for
   the gate.
2. S1: the projector mapping in `agui.rs`, with its unit tests authored but not
   run.
3. S2: action parsing, validation, input and echo in `agui.rs`, with its unit
   tests authored but not run.
4. S3: the `a2ui_render` example gains a Button.
5. S4: rewrite I19, and add `@prometheus-ags/a2ui-react@4.3.0` to the behaviour
   test dependencies once npm lists it.
6. Record the implementation-complete inventory, then wait for the operator's
   gate instruction (§7).

## Revision 1 independent review — 2026-10-28

A fresh-context artifact critic reviewed this plan against UAR and PEM source.
It ran nothing. Verdict: **defects found**, with 2 critical defects, 10 warnings
and 5 suggestions. The lead confirmed the starred items in source (marked ✓).

### Critical

1. **✓ Every surface goes dead after one click.**
   - S2 accepts actions only for surfaces published by the thread's *latest*
     run, which is the highest ordinal (`agui.rs:244-245`).
   - The run that answers a click becomes the latest run, and S1 does not
     restate older surfaces in it.
   - Result: a second click, or any click after a typed follow-up, returns
     `422`. I19 clicks once, so it cannot see this.
2. **✓ A 512-event ring governs both the action check and reconnect while the
   run is live, and §9's explanation was wrong.**
   - `EVENT_HISTORY_LIMIT = 512` (`manager.rs:46`). `history_since` reads only
     that ring while the run is active, and falls back to durable run history
     only after it leaves memory (`manager.rs:9150-9190`).
   - A chatty run can evict the `ArtifactDisplay`. A valid click then gets
     `422`, and reconnect gets `RUN_ERROR stream_history_unavailable`
     (`agui.rs:685-686`).
   - §9 blamed restarts. The durable fallback means a restart is the case that
     may actually keep the surface.
   - The REST path validates against the realtime backbone instead, which this
     ring does not bound (`a2ui/routes.rs:838`).

### Warnings

- **✓ Event order is wrong in S1.** The snapshot precedes `TOOL_CALL_RESULT`:
  `ToolEnd` is published only after the publication returns
  (`a2ui_output.rs:89-90`).
- **S2's input handling corrupts the action run.**
  - `split_last` drops the assistant's closing reply from history (`agui.rs:390`).
  - With client tools present, the action JSON is written into that assistant
    message (`agui.rs:423-426`).
  - Client context and state text is appended to the input, so it is no longer
    valid JSON (`agui.rs:401-404`).
- **✓ `protocol.rs` is not the gate at the client.** UAR accepts a top-level
  `profile` field (`protocol.rs:40-41`) and forwards raw messages. web_core's
  strict schemas then make PEM reject the whole batch.
- **Idempotency doesn't match PEM.**
  - PEM's `actionId` is the retry key (`forwarded-props.ts:52`), and S2
    ignores it.
  - A retry with a fresh `runId` gets `409 thread_run_not_closed`, not a replay.
- **The trust boundary checks only the action name.** `userAction.context` is
  not checked against the declared `action.event.context`, and has no size
  limit.
- **I19 doesn't prove a real click.**
  - The test supplies `name` and `sourceComponentId` itself, and
    `dispatchAction` doesn't check them against the component.
  - "The agent answers that exact action" has no defined observable.
  - Both UAR catalog ids should be registered on the client.
- **✓ S3 is already done.** `a2ui_render.rs:133` already has a `Button` with
  `continueDemo`.
- **§7 can't be approved, and its regression scope is too narrow.**
  - `--test <agui route test>` is a placeholder.
  - S1 and S2 change every stream and the resume path. The other users of this
    route also need reruns: native-tool-approval, agui-math,
    live-roster-routing, writing-orchestrator and selection-stream-replay.
- **"Any single snapshot is enough" fails across a warm resume.** A resumed run
  starts at the terminal cursor (`agui.rs:326`, `:679`), so its snapshot can
  lack `createSurface`.

### Suggestions

- **Activity echo-back:** clients that send `role:"activity"` messages back hit
  `400 unsupported_history_message` (`agui.rs:75-78`).
- **The action echo:** its frame id and replay behavior are unspecified.
- **"Exists on that surface":** define it given `deleteSurface` and later
  updates.
- **"One input shape":** the claim is inexact, because REST continuations reuse
  the source run's policy and negotiation (`manager.rs:3162-3187`).
- **Citation:** `agui.rs:395` is cited for something it doesn't show.

### Lead assessment

Both critical defects are real and come from one design gap. The slice treats a
surface as belonging to the run that published it, but a user experiences a
surface as belonging to the thread, across clicks and follow-ups. Revision 2
needs a thread-scoped source of published surfaces:

- the action check and the snapshot restatement both read it;
- it is not bounded by the live 512-event ring;
- the restatement covers later runs and warm resumes.

The likely minimal fix is to scan the thread's earlier runs through durable run
history, plus the realtime backbone for the live run. That pulls part of D2/D4
(the durable surface record) into this slice. The operator should decide that
scope before revision 2. Correct in revision 1, per the critic: the
re-projection from `start_cursor`, and every PEM API name and behavior cited.
