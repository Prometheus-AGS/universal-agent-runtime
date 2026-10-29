# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 5)

Stage: Plan. Written 2026-10-28. Nothing here is implemented, and the plan
stops before Execute.

- **Supersedes:** revision 4 (`plan-rev4.md`). Each earlier revision's
  independent review is appended to its own file.
- **Operator direction for revision 5:** store the action as a
  `uar.a2ui.action` run event, with no change to the binding, no new table and
  no migration. Fix revision 4's warnings 3 and 6–10 and its suggestions. Then
  a fresh critic reviews the plan, and work stops.
- **What revision 5 keeps from revision 4:** the outcome (§1), the decisions
  (§2), the per-binding cursor ranges, and the PEM limit table. Those parts are
  referenced, not repeated, and are summarised where they matter. The limit
  table is S1a in `plan-rev4.md`; three of its rows change in §4 S1a here.
- **Marker:** sections marked *(r5)* are new or changed.

## 1. The outcome a user sees

The outcome is the same as in revision 4 §1. An application on UAR's dedicated
AG-UI route, using PEM 4.3.0's headless runtime, can:

1. **see** the agent's surface;
2. **click** a rendered button, and have the agent receive that exact action;
3. **keep using** the surface later, including after typed follow-ups;
4. **join late** by attaching to the thread's latest run, and still see the
   thread's live surfaces without the agent running again.

None of this works today: on 2026-10-28, I19 measured a run that reported
success while the client received no surface (`agui.rs:651`).

## 2. Operator decisions applied

These are unchanged from revision 4 §2:

- **D1, adopted:** `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`.
- **D3:** cumulative snapshots only, with `replace: true`.
- **D2 and D4, partly pulled in:** the surface record is *derived* from durable
  run history plus live events. Nothing new is stored.
- **D5–D17:** deferred.

## 3. *(r5)* Storage: none added

**The action is a run event.** Revision 5 adds no persisted shape at all:

| Item | Status |
|---|---|
| `AguiRunBinding` | unchanged; guarded by a byte-identity test (§5) |
| New table, record or `.surql` file | none |
| Postgres migration | none |
| Persistence trait | unchanged |

**How it works:**

- **The record.** The action run's first event is
  `NormalizedEvent::Custom { source: "uar.a2ui", event_name: "action", payload }`.
  The `Custom` variant already exists; `budget_receipt` uses it
  (`liter_driver.rs:753`).
- **Persistence.** The event is persisted the way every emitted event is.
  `RunEventEmitter::emit` records it in the run's history and passes it to the
  durable run history writer (`manager.rs:540-565`; writer at `:322-336`).
- **Where the fold finds it.** The fold already reads that history (§4 S1).
  The echo, the `actionId` lookup and the context digest all come from this
  one event.
- **Rollback safety.**
  - An older binary deserializes the event into its existing `Custom` variant.
    Its projector then drops it (`agui.rs:651`, `_ => Vec::new()`).
  - No row an older binary reads or compares changes, and no migration exists
    that it would fail to recognize.

  This removes revision 4's critical defect. On Postgres, any new migration
  makes the previous binary refuse to start (`postgres.rs:70`; sqlx
  `VersionMissing`).
- **What rollback loses.** On the old binary, `actionId` replay and the echo
  disappear. Threads keep working.

## 4. Changes

There is one OpenSpec change, `agui-a2ui-activity-slice` (§7), and one Rust
writer.

| File | What changes |
|---|---|
| `src/uar/api/agui_a2ui.rs` (new) | The fold, the limits, the action checks and the builder |
| `src/uar/api/agui.rs` | Call sites only |
| `src/uar/runtime/manager.rs` | One read method, plus emitting the action event |
| `src/uar/api/routes.rs` and `src/uar/runtime/turn/request.rs` | One server-set field each |

### S0. *(r5)* Carrying the validated action into the run

- **On `RunExecutionRequest`** (`turn/request.rs`, next to `skill_attachments`
  at `:153`): add `a2ui_action: Option<serde_json::Value>`.
- **On `CreateRunRequest`** (`routes.rs:72-104`): add the same field with
  `#[serde(skip)]`.
  - **Why it must be skipped:** `CreateRunRequest` is deserialized from the
    client's `forwardedProps.uar` (`agui.rs:412`). A deserializable field would
    let a client inject an unvalidated action.
  - **Who sets it:** only `agui.rs`, after the S3 checks, and `admit_run`
    copies it into `RunExecutionRequest`.
- **Emit point.** Right after `RunEventEmitter` is constructed
  (`manager.rs:3975-3986`), and before any other event, the manager emits the
  `uar.a2ui` / `action` event when `a2ui_action` is present.
  - **Event id:** an action run is always a fresh native run (S3 refuses an
    action with `resume`), so this event is event 1 of its run.
- **Payload:**
  `{actionId, surfaceId, name, sourceComponentId, catalogId?, userMessage?, contextKeys, contextDigest, sourceRunId, threadId}`.
  - **`contextDigest`:** `hex(sha256(canonical JSON of the resolved context))`,
    with keys sorted and no whitespace.
  - **Values stay out:** the context *values* are not in the event, only in
    the run input (S3).

### S1. The thread surface record

Unchanged from revision 4 S1, except for the following:

- ***(r5)* Restatement excludes the stream's own run** (warning 3).
  - **The rule:** the stream for binding *B* restates
    `ThreadSurfaces(thread, B.ordinal − 1)`, built only from *closed* earlier
    bindings. Its output is deterministic: those bindings' ranges cannot grow.
  - **B's own surfaces:** they arrive as B's own source-event snapshots, as
    before.
  - **Warm resume:** a warm-resumed B's predecessor range belongs to the
    predecessor binding. Since that binding is closed, it is included.
- ***(r5)* Action events in the same read.** `published_a2ui_events(run_id,
  after, up_to)` also returns `uar.a2ui` / `action` events. The `actionId`
  lookup (S3) folds the same bindings' action events, so there is no second
  read.
- ***(r5)* Depth** (revision 4 suggestion). Depth is measured the way PEM
  measures it: from *every* unreferenced component, with the root counted as
  level 1 (`limit-checks.ts:54-96`, `:69`). It is not measured only from
  `root`.

#### S1a. *(r5)* PEM limits with a 25 % margin

These are revision 4's S1a rows, plus the following changes:

- **The surface cap counts live runs too** (warning 8). A *thread-wide* count
  of live surfaces ≤ 48 is enforced for both restatement and live
  publication.
  - **What happens at the cap:** a new surface that would be the 49th live
    surface emits no snapshot and is marked `over_limit`. Its later actions are
    refused with `422 a2ui_surface_over_limit`.
  - **Why:** PEM's `maxSurfaces` (64) is runtime-wide (`limit-checks.ts:150`).
- **The same applies to the total component cap:** at most 7,500 components
  across live surfaces, counted for both restatement and live publication.
- **The `dataModel` cap on incoming actions** (warning 9) rises from 64 KiB to
  1 MiB, PEM's own `maxDataModelBytes`. PEM sends the data model of every
  `sendDataModel` surface in one object (`runtime-core.ts:637-643`, `:749`).
  - **UAR-published surfaces never set `sendDataModel`.** UAR's
    `CreateSurface` admits only `surfaceId` and `catalogId`
    (`protocol.rs:46-50`), so this field arrives only for surfaces from other
    producers.
  - **The value reaches the model unchanged.** It is passed as
    `a2uiClientDataModel`.

### S2. *(r5)* Outbound: project surfaces as activities

- **messageId.** Unchanged:
  `"a2ui:" + hex(sha256(thread_id \0 surfaceId))[..32]`.
- **The reserved range, simplified.** The range is `(start_cursor, 1..=49)`:

  | Index | Frame |
  |---|---|
  | 1–48 | `ACTIVITY_SNAPSHOT`, one per restated surface (S1: earlier closed bindings only) |
  | 49 | `CUSTOM uar.a2ui.restatement_unavailable` (only when a binding is `history_unavailable` or `history_incomplete`) |

  **The echo is no longer reserved.** The action event is event 1 of the
  action run (S0), so the projector maps it like any source event, to frame
  `(1, 1)`:
  `{type: "CUSTOM", name: "uar.a2ui.action.accepted", value: {actionId, surfaceId, name, sourceComponentId, contextKeys, contextDigest}}`.
  It is a normal frame, ordered after the reserved range.
- **Collisions** (warning 3). The fix removes revision 4's exemption: reserved
  frames are *always* emitted, and the colliding frames move instead.
  - **When it applies:** when a binding seals with
    `terminal.source_cursor == row.start_cursor`, its terminal block's
    caller-tool frames move. These are the cold denial and the empty-history
    caller-tool recovery cases (`agui.rs:515`, `:548`).
  - **Today's frames there:** caller-tool `(cursor, 1..=3k)`, close
    `(cursor, terminal_index − 10)`, and terminal `(cursor, terminal_index)`
    with `terminal_index = 100 + 3k` (`agui.rs:700-714`).
  - **The new frames:** caller-tool frames go to `(cursor, 51..=50 + 3k)`, and
    `terminal_index` becomes `150 + 3k`.
  - **Decidable when needed:** this branch is evaluated when the terminal is
    emitted, and the terminal is known by then.
  - **Bindings with source events keep today's ids exactly.**
- **Other fixed indices.**
  - **Errors** use `(cursor, 99)`, while `cursor == start_cursor` before any
    source event (`:682`, `:696`, `:718`). That is outside 1–49.
  - **`RUN_STARTED`** stays `(start_cursor, 0)`.
- **Upgrade effect on those edge bindings.** A client holding a pre-upgrade
  `Last-Event-ID` inside the old caller-tool range of such a binding may have
  those frames resent after the upgrade. AG-UI clients deduplicate by tool-call
  id. This is stated in §10.
- **Monotonic.** Reserved frames are emitted after `RUN_STARTED` and before
  any source event (cursor > `start_cursor`), and before any terminal block at
  `start_cursor` (indices ≥ 51). The existing `Last-Event-ID` filter
  (`:691-692`) therefore stays correct.
- **Updates during a run, order relative to the tool result, and `profile`
  stripping:** unchanged from revision 4 S2.

### S3. *(r5)* Inbound: `forwardedProps.a2uiAction`

- **Parsing.**
  - **`A2uiAction`** (closed struct): `userAction`, plus optional `actionId`,
    `heldVersion` and `dataModel`.
  - **`UserAction`** (closed struct): `name`, `surfaceId`,
    `sourceComponentId`, `context`, `timestamp`, `catalogId` and
    `userMessage`. The bounds are the same as in revision 4, except that
    `dataModel` may be up to 1 MiB (S1a).
  - **`resume`:** an action may not be combined with `resume` entries
    (`400 a2ui_action_with_resume`).
- **Check order.**
  1. **Same `runId`:** replay, as today.
  2. **Retry by `actionId`.** Fold the thread's action events (S1).
     - If one has the same `actionId`, and its digest over
       `(surfaceId, name, sourceComponentId, catalogId)` plus the workspace
       match, stream that binding (replay).
     - Otherwise, return `409 a2ui_action_id_conflict`.
  3. **The latest-run rule.**
  4. **The declaration check.** This is unchanged from revision 4, with one
     addition: `catalogId`, when present, must equal the surface's catalog.
  5. **Reserve** the run with `create_agui_run(row, previous)`. It is
     unchanged, and the persistence trait is not touched.
  6. ***(r5)* No race remapping** (warning 2 of revision 4).
     - **Surreal's create is left as it is today:** a thrown
       `agui_context_conflict` returns `false`, and a `TransactionConflict`
       returns `Err` and then `503`. Two concurrent requests on one thread
       already serialize on the `agui_contexts` slot (`surreal.rs:8-13`), so no
       remapping is needed.
     - **Losing twin:** when `create` returns `false`, step 2 runs *once* more.
       If the winner's action event is already written, the loser replays.
       Otherwise it gets today's `409 thread_run_not_closed`, and a later retry
       replays.
- **Building the run.**
  - **Identity stripping:** the action run uses the shared helper extracted
    from `create_request` (`agui.rs:408`).
  - **History:** the full history is kept. With client tools present, the
    action is appended as a new user message. An action together with new user
    text returns `400 a2ui_action_with_text`.
  - **Input:** the serialized JSON *string* (`CreateRunRequest.input` is a
    `String`, `routes.rs:90`):
    `{"type": "a2ui.user_action", "sourceRunId", "interaction": {surfaceId, name, sourceComponentId, timestamp, context, catalogId, userMessage, actionId, a2uiClientDataModel, clientContext}}`.
  - **The action event:** `a2ui_action` is set on the request (S0), so the run
    emits it as event 1.
- ***(r5)* Authority, complete** (revision 4 suggestion).
  - **Fixed by the path and the binding:** `artifact`, `deployment_binding_id`,
    `input`, `agent_id`, `session_id` and `history` are stripped.
  - **Still client-supplied, exactly as on every run of this route today:**
    `model`, `response_format`, `service_placement`, `mcp_servers`,
    `tool_admission`, `run_credentials`, `working_directory` and
    `reasoning_effort` (`routes.rs:73-101`).
  - **No new authority:** a click grants nothing that the same client's text
    run doesn't already have. The action values reach the model as the user
    turn, which the client controls.
- ***(r5)* Activity messages.** `role: "activity"` messages are dropped before
  *both* history validations, in `caller_history` (`agui.rs:75-78`) and in
  `create_request` (`:397`), on every run.

### S4. *(r5)* Acceptance: I19

The Gherkin is unchanged from revision 4 S4. The implementation details are
corrected (revision 4 warnings 6, 7 and 10):

- **Client dependencies** (warning 10). Installed with pnpm, because the
  repository is a pnpm workspace (`package.json:4`; `pnpm-workspace.yaml`).
  - **When:** during Execute, task 6, not at the gate:
    `pnpm add -D --workspace-root @prometheus-ags/a2ui-react@4.3.0 @a2ui/web_core@0.12.0`.
  - **Effect:** `package.json` and `pnpm-lock.yaml` change, and those changes
    are reviewed with the change.
  - **Availability:** PEM 4.3.0 was confirmed on npm on 2026-10-28.
- **The runtime.** `createPrometheusA2uiHeadlessRuntime` gets the following
  options:
  - **Catalogs:** two headless catalogs, registered under UAR's two catalog ids
    (`protocol.rs:264`).
  - **Action policy** (warning 7): it must be `executed`, because PEM's default
    denies every action (`runtime-core.ts:124`, `:138-140`):

    ```js
    actionPolicy: { handle: async (action) => ({ status: 'executed', action, result: null }) }
    ```
  - **Envelope capture:** `onActionEnvelope` resolves a promise that the test
    awaits.
- **The press, through web_core's binder** (warning 6).
  - **Imports:** `ComponentContext` and `GenericBinder` from
    `@a2ui/web_core/v0_9`, which re-exports `../resolution/index.js`.
    `ButtonApi` comes from `@a2ui/web_core/v0_9/basic_catalog`
    (`basic_components.d.ts:901`). UAR publishes v0.9 surfaces, and PEM
    rewrites them to v0.9.1.
  - **Bind:** `new GenericBinder(new ComponentContext(surface, buttonId), ButtonApi.schema)`.
  - **Press:** set the data model value (`surface.dataModel.set('/form/note', note)`),
    then invoke `binder.snapshot.action()`. The closure resolves the context and
    dispatches. It does not return the dispatch promise
    (`generic-binder.js:371-382`), so the test awaits the `onActionEnvelope`
    promise instead.
  - **Wording corrected:** "the same binder its renderers use" was wrong. PEM's
    renderers use PEM's *bundled* web_core copy (`dist/chunk-PZW47VJJ.mjs`).
    The test uses the separately installed `@a2ui/web_core@0.12.0`, the version
    PEM pins.
  - **The cross-bundle check stays an Execute item.** If the separate binder
    can't operate on PEM's `SurfaceModel`, the press step stays *failed*, and a
    PEM issue is filed for a headless press helper. It never falls back to a
    direct `dispatchAction`.
- **Observables.**
  - **The click reached the run's input (model-independent).**
    - **What the test checks:** the echo frame `(1, 1)` carries the
      envelope's `actionId`, and a `contextDigest` that the test recomputes
      from the context the binder resolved.
    - **Why this proves the input** (revision 4 suggestion): the echo is
      emitted *from the run's own first event*, which the manager builds from
      `RunExecutionRequest.a2ui_action`, the same request that carries the
      input. It is no longer a separately stored record. A unit test
      (`action_event_matches_run_input`) checks that the event and the input
      come from one value.
  - **The model saw it** (secondary, model-dependent): the reply contains the
    `actionId`.
  - **Retry, late attach and the forged action:** unchanged from revision 4.
    The late attach is a `GET` on the reconnect route, which never admits a run
    (`agui.rs:443-449`).
- **Bounds.** Five model turns, one model (`liter/MiniMax-M3`), no
  credentials.

## 5. *(r5)* Tests

### 5a. Tests that cover storage

Nothing in storage changes, so revision 4's store tests (the sibling-record
writes, the conflict mapping, and the concurrency test that couldn't reliably
force a conflict) are dropped. Two remain:

| Test | File | Proves |
|---|---|---|
| `agui_run_binding_serialization_unchanged` | `src/uar/persistence/agui_runs.rs` | The binding JSON is byte-identical to a fixture captured before any change. |
| `action_event_round_trips_durable_history` | `src/uar/persistence/providers/agui_runs/surreal.rs` `#[cfg(test)]`, using the tempfile `surrealkv://` provider (`a2a/handler.rs:319-326`; `thread/service_tests.rs:40-55`) | An action event appended through the history writer comes back from `load_run_history_events` unchanged, and a binding stored with today's exact JSON still lists, transitions and seals. |

**Postgres.** No Postgres code changes in this slice, so the fact that the
gate doesn't compile it (it builds `server-full` without `postgres-backend`,
`providers/mod.rs:3`; `Cargo.toml:259`) costs nothing here.

### 5b. Unit tests (authored with the code; run only at the gate)

Revision 4's §5b is kept, minus its sibling-record tests, plus these:

| Case | Test | File |
|---|---|---|
| The action event is event 1 and equals the request's action | `action_event_is_first_and_matches_request` | `manager.rs` tests |
| The action event and the input come from one value | `action_event_matches_run_input` | `agui_a2ui.rs` |
| `CreateRunRequest.a2ui_action` can't be set from `forwardedProps.uar` | `forwarded_uar_cannot_set_a2ui_action` | `routes.rs` tests |
| Restatement excludes the stream's own run | `restatement_folds_only_earlier_closed_bindings` | `agui_a2ui.rs` |
| An edge binding's caller-tool frames move to 51 and above | `edge_binding_caller_frames_shift_above_reserved_range` | `agui_a2ui.rs` |
| Live publication enforces the thread surface and component caps | `live_publication_enforces_thread_surface_cap` | `agui_a2ui.rs` |
| Depth is measured from every unreferenced component | `depth_measured_from_unreferenced_roots` | `agui_a2ui.rs` |
| An action with `resume` is refused | `action_with_resume_refused` | `agui_a2ui.rs` |
| Activity messages are dropped in both validators | `activity_messages_dropped_in_both_validators` | `agui_a2ui.rs` |

## 6. PEM 4.3.0 and web_core 0.12.0 client contract

This is revision 4 §6, corrected:

- **Binder imports:** `@a2ui/web_core/v0_9` exports `ComponentContext` and
  `GenericBinder`. `ButtonApi.schema` comes from
  `@a2ui/web_core/v0_9/basic_catalog`.
- **Action policy:** the default is deny-all.
- **Bundling:** PEM's renderers use its bundled web_core.

## 7. *(r5)* Spec delta (OpenSpec `agui-a2ui-activity-slice`)

The change adds a new capability, `ag-ui-a2ui-activities`, with five
requirements:

1. **Published surfaces are thread objects on the AG-UI route.**
   - Restatement covers earlier closed bindings, in the reserved frames
     `(start_cursor, 1..=48)`.
   - A late attach renders the surface without admitting a run.
   - A deleted surface is not restated.
   - A surface survives the live ring when durable history exists.
   - A thread with no surface produces no activity.
2. **Snapshots and live publication stay within PEM's default limits, with a
   25 % margin.** This includes the thread-wide surface and component caps for
   live runs.
3. **Declared actions become same-thread runs, recorded as a run event.**
   - The first event of an action run is the `uar.a2ui` action event, and the
     echo is projected from it.
   - A surface still works after its first answer.
   - An undeclared action, an orphaned component or a mismatched catalog is
     refused with `422`.
   - A retry with the same `actionId` replays.
   - A reused `actionId` with a different action is refused with `409`.
   - Web_core's `catalogId` and `userMessage` fields are accepted.
   - An action combined with `resume` is refused.
4. **Forwarded messages carry no UAR-only fields.** UAR's `profile` field is
   stripped.
5. **No persisted shape changes.**
   - The binding JSON is unchanged.
   - No table or migration is added.
   - An older binary reads the new event as an ordinary `Custom` event and
     ignores it.

## 8. *(r5)* Verification (exact commands; not run; the gate is closed)

The commands run serially, after the recorded implementation-complete
inventory and the operator's gate instruction.

0. Validate the OpenSpec change: `openspec validate agui-a2ui-activity-slice`.
   The dependencies are installed in Execute (S4), not here.
1. Run the unit and store tests:
   `cargo test --locked --no-default-features --features server-full --lib -- agui_a2ui:: agui_runs:: published_a2ui_events_ action_event_ forwarded_uar_cannot_set_a2ui_action`.
2. Rebuild the owned binary. **This is still an open precondition:** the build
   command must be confirmed verbatim before the gate request. The candidate
   is `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
3. Restart the owned UAR with the rebuilt binary's digest:

   ```bash
   node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r))" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"
   ```
4. Set the environment:

   ```bash
   export UAR_BASE_URL="$(node -p "require('./.prometheus/verification/manual-runtime.json').base_url")" UAR_MODEL=liter/MiniMax-M3 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace
   ```
5. Run I19 once:
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs`.
6. Regression, once each:
   - `selection-stream-replay`;
   - `live-roster-routing`;
   - `native-tool-approval`;
   - `agui`;
   - `writing`.

   Each runs as
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`.

C14 and C15 are not rerun.

## 9. Deferred decisions

These are unchanged from revision 4 §9.

| Later decision | Status |
|---|---|
| D3 deltas | Deferred |
| D2 stored record | Deferred; it would remove the per-request fold and its 64-binding window |
| D4 versions | Deferred |
| D7 compaction | Deferred |
| D11 and D15 (catalog ids) | Deferred |
| D17 | Deferred |

## 10. The uncomfortable parts

- **A PEM 4.3.0 client with default settings still renders nothing from UAR.**
  PEM's default catalog id is `urn:prometheus-ags:a2ui:catalog:v3`, and UAR
  publishes `urn:uar:a2ui:catalog:1` or the basic v0.9 URL. Fixing that belongs
  to D11/D15. Until then, the proposal must document the client setting.
- **The binder press may not work across PEM's bundled web_core.** If it
  doesn't, I19's press steps fail until PEM exports a headless press helper.
- **A losing twin sometimes gets `409`.** When two identical clicks race and
  the winner's action event isn't written yet, the loser gets `409
  thread_run_not_closed` instead of a replay. A later retry replays. This is
  the price of storing nothing new.
- **Edge-binding caller-tool frames are renumbered.** For cold denials and
  empty-history recoveries only, they move to 51 and above. A client holding a
  pre-upgrade `Last-Event-ID` inside the old range may see those frames again
  once.
- **A long-lived, frequently updated surface eventually goes over its limit.**
  It passes 375 cumulative operations, and from then on it is neither restated
  nor actionable. Deltas (D3) and compaction (D7) are the real fix.
- **The thread caps are first come, first served.** The 49th live surface on a
  thread is refused even if older surfaces are no longer visible to the user.
  Only `deleteSurface` frees a slot.
- **Every run and every click re-folds up to 64 bindings' durable history.**
  A stored record (D2) would remove that cost.
- **Some existing behaviours are not fixed here:**
  - `reconcile` swallows history read failures with `unwrap_or_default()`
    (`agui.rs:514`);
  - UAR doesn't bound tombstones;
  - a replay reports the original `runId`;
  - the REST action path is looser and has no `actionId`.
- **The rebuild command is still unknown.** It is an open precondition in §8,
  step 2.

## 11. Tasks (for Execute; none started)

1. Create the OpenSpec change `agui-a2ui-activity-slice` (§7).
2. Capture today's `AguiRunBinding` JSON fixture, for the guard test.
3. Add `a2ui_action` to `RunExecutionRequest`, add it with `#[serde(skip)]` to
   `CreateRunRequest`, emit the action event in the manager, and add
   `published_a2ui_events` with action events, together with their tests.
4. In `agui_a2ui.rs`, build:
   - the fold (earlier closed bindings for restatement);
   - the limits (including live thread caps and PEM depth);
   - the reserved range and the edge-binding shift;
   - the action parsing, checks and builder;

   and author the §5 tests, without running them.
5. Wire it into `agui.rs`, including the activity-message drop in both
   validators.
6. Write I19 (S4), and install the client dependencies with pnpm.
7. Confirm the rebuild command for §8, step 2.
8. Record the implementation-complete inventory, then wait for the operator's
   gate instruction.

## Revision 5 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 5 against UAR, PEM 4.3.0
and web_core 0.12.0. It ran nothing.

**Verdict: defects found.** The review raised 2 critical findings, 6 warnings
and 3 suggestions. The lead confirmed the starred items in the source (✓).

### Critical

1. **✓ The domain event enum has no `Custom` variant, so the plan's premise is
   false.**
   - **The wrong citation:** revision 5 cited `budget_receipt`
     (`liter_driver.rs:753`). That file imports the *provider* enum
     (`liter_driver.rs:18`, `use crate::normalized::NormalizedEvent`).
   - **The enum that matters:** `RunEventEmitter`, run history and `agui.rs`
     use `crate::uar::domain::events::NormalizedEvent`. Its file contains no
     `Custom` variant (`grep -c Custom src/uar/domain/events.rs` → 0).
   - **How provider events actually arrive:** the manager translates them into
     `Artifact` events with `artifact_type: "provider_event"`
     (`manager.rs:8302-8345`).
   - **The lead's error:** the lead asserted that the variant existed without
     checking which enum the citation used.
   - **Consequence:** adding a variant changes the persisted event shape.
     That contradicts §3 and spec requirement 5, and exposes the variant on the
     native SSE and adapter paths.
2. **✓ After a rollback, every action run would show a history gap.**
   - **The mechanism:** the durable read drops events it can't deserialize
     (`manager.rs:9182-9186`, `serde_json::from_value::<NormalizedEvent>(…).ok()`).
     An older binary would therefore read each action run's history starting
     at event 2.
   - **The effect:** `stream()` treats that as a gap and emits
     `RUN_ERROR stream_history_unavailable` (`agui.rs:685-686`). Every action
     run's stream, reconnect and late attach then fails.
   - **The fix the critic proposed:** reuse the existing `ArtifactDisplay`
     variant. An older binary deserializes it, and the projector's `_` arm
     drops it.

### Warnings

3. **"Event 1" holds only when admission succeeds.**
   - `emit` stages events (`manager.rs:525-526`). An admission error clears the
     staged events (`event_projection.rs:62-68`), and they are published only
     at `manager.rs:6334`.
   - So a failed admission never persists the action event.
4. **The losing twin almost always gets a 409, and the code differs.**
   - The winner's event becomes visible only at `:6334`.
   - Today's code for `create` returning `false` is `thread_run_conflict`
     (`agui.rs:350`), and the `TransactionConflict` path still returns 503.
5. **The `actionId` lookup's set is unspecified.** If it uses restatement's
   earlier closed bindings, capped at 64, three things go wrong:
   - a retry while the winning run is still open never replays;
   - retries past 64 bindings, or into incomplete bindings, miss;
   - those misses are admitted again as duplicates.
6. **The surface-cap rules contradict each other.** Revision 4 S1a (kept)
   restates "the most recent 48", while revision 5 §10 says "first come, first
   served".
7. **A late attach to a long latest run reads only the 512-event ring.**
   `stream()`'s own replay uses `history_since`, which reads only the ring for
   an active run (`manager.rs:9151-9169`). It does not use
   `published_a2ui_events`, so a long run can return
   `stream_history_unavailable`.
8. **The pnpm add is risky.**
   - The workspace already contains two `@prometheus-ags/a2ui-react` packages
     (`pnpm-workspace.yaml:2-5`).
   - A `minimumReleaseAgeExclude` list exists (`:16`), and 4.3.0 was published
     the same day.
   - PEM declares peer dependencies on React 19 and `entity-graph-core`.
   - The web_core paths themselves are correct.

### Suggestions

9. **The shift is larger than needed.** Moving the terminal frames
   (`150 + 3k`) is unnecessary, and it renumbers terminal frames for cold
   denials and interrupt seals. No tests assert frame ids.
10. **The planned test homes don't exist as described.**
    - `routes.rs`, `agui_runs.rs` and `providers/agui_runs/surreal.rs` have no
      `#[cfg(test)]` module.
    - `RunHistoryWriter` is private to `manager.rs`.
    - Run-history storage lives in `providers/run_history/surreal.rs`.
11. **The input proof is still indirect, and one field goes unscrubbed.**
    - Deriving the event from the serialized `input` would prove what the model
      received.
    - Provider `Custom` events are not scrubbed (`event_projection.rs:248`), so
      `userMessage` would be persisted unscrubbed.

### Confirmed sound

- `#[serde(skip)]` blocks injection (the `client_tools` precedent,
  `routes.rs:77-78`).
- `admit_run` destructures the request exhaustively (`:503-521`), so an
  unmapped field fails to compile.
- The §8 configs and the restart helper exist.
- The reserved range is collision-free, and the echo lands at `(1,1)` when
  admission succeeds.

### Lead assessment

Revision 5's design idea holds, but it was built on an unverified fact.

- **The idea:** record the action inside the action run's own durable history,
  so no new storage is needed.
- **The unverified fact:** that the domain enum had a `Custom` variant.
- **The fix the critic proposed:** reuse the existing `ArtifactDisplay` variant
  with a new `artifact_type`, `"a2ui_action"`. That variant already exists and
  is already persisted. An older binary deserializes it, its projector drops
  it, and its scrubbing path applies.
- **What the A2UI fold must also do:** filter strictly on
  `artifact_type == "a2ui"`, so it never mistakes an action for a surface.

Warnings 3–7 are specification gaps that revision 6 would close:

- admission-failure semantics;
- the twin's real code;
- the lookup's own anchor, including open bindings;
- one surface-cap rule;
- a late attach that reads the merged history.

**Process note.** Each of the last two revisions rested its safety argument on
a fact about the source that the lead had not verified (sqlx migrations in
revision 4, the enum variant in revision 5). Revision 6 should list every
load-bearing source fact with a line the lead has re-read before the critic
sees it.
