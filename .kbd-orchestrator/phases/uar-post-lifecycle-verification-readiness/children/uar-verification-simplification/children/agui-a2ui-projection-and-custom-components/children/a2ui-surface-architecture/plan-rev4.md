# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 4)

Stage: Plan. Written 2026-10-28. Nothing here is implemented; the plan stops
before Execute.

- **Supersedes** revision 3 (`plan-rev3.md`). The review of each earlier
  revision is appended to its file.
- **Operator direction for revision 4:**
  - enforce every PEM default limit with a margin;
  - replace the integration test with one this repository can run, or state
    that the unit test is the guarantee;
  - reserve the restatement frame-index range;
  - drive the press through PEM's binder;
  - fix warnings 4–8;
  - have a fresh critic review the plan, then stop.
- **Carried over unchanged:** what revision 3's review confirmed as correct.
  Surfaces belong to the thread, each binding contributes only its own cursor
  range, the window is anchored to the binding ordinal, and the record does
  not depend on the 512-event ring.
- **Marking:** sections marked *(r4)* changed in this revision.

## 0. *(r4)* One design change, flagged for the operator

The operator's revision 3 direction kept action data on `AguiRunBinding`
(behind `skip_serializing_if`). Revision 3's review then found that this breaks
**rollback** (warning 8):

- An older binary drops the unknown field when it re-serializes a row.
- Its compare-and-swap check (Surreal `surreal.rs:12`, `:42`; Postgres
  `postgres.rs:45`) then no longer matches the stored row.
- Every later run on a thread that ever carried an action fails with
  `thread_run_conflict` or `thread_run_not_closed`.

`skip_serializing_if` cannot fix that, because the old binary has never heard
of the field.

**Revision 4 therefore stores action data in a sibling record and leaves
`AguiRunBinding` byte-for-byte unchanged (S0).**

- **Why this is safe both ways.** The old binary never reads the new record,
  and the new binary never changes the binding's JSON. Neither direction of
  deployment can break a compare-and-swap.
- **What this replaces.** The `skip_serializing_if` field and its
  pre-upgrade-row tests become unnecessary. A byte-identity guard on the
  unchanged binding remains (§5).
- **Operator approval needed.** This departs from the revision 3 direction, so
  it is listed first.

## 1. The outcome a user sees

An application on UAR's dedicated AG-UI route
(`/api/uar/ag-ui/agents/{agent_id}`), using an AG-UI A2UI client (here the PEM
4.3.0 headless runtime), can:

1. **See the agent's UI.** A surface published with `a2ui_render` renders in the
   client.
2. **Click, and the agent answers.** A real press of a rendered button sends
   that exact action as a new run on the same thread, and the agent receives
   it.
3. **Keep using the surface.** The same button works again later, including
   after a typed follow-up.
4. **Join late and still see it.** A client that attaches to the thread's
   latest run sees the thread's live surfaces without the agent running again.

None of this works today. On 2026-10-28, I19 on the Mac fixture showed the run
reporting success while the client received no surface. The cause is in
`agui.rs:651`: `_ => Vec::new()` drops every A2UI event on this route.

## 2. Operator decisions applied

| Decision | Applied |
|---|---|
| D1 (AG-UI activity carriage) | Adopted for this slice: surfaces travel as `ACTIVITY_SNAPSHOT` events with `activityType: "a2ui-surface"`. |
| D3 (incremental carriage) | Snapshots only, cumulative, with `replace: true`. |
| D2, D4 | Partly pulled in: the surface record is derived from durable run history plus live run events. No stored surface record. |
| D5–D17 | Deferred. |

## 3. Upstream contract followed

- **Activity.**
  `{type: "ACTIVITY_SNAPSHOT", messageId, activityType: "a2ui-surface", content: {a2ui_operations: [...]}, replace: true}`.
  Each activity's operations are cumulative (upstream-research.md
  §L125-L129).
- **Action.** It arrives in the next run's `forwardedProps.a2uiAction`:
  `{userAction: {name, surfaceId, sourceComponentId, timestamp, context, catalogId?, userMessage?}, actionId, heldVersion?, dataModel?}`.
  - **The core envelope** is defined by PEM 4.3.0 (`forwarded-props.ts`;
    `runtime-core.ts:736-755`).
  - **`catalogId` and `userMessage`** are optional fields that web_core's
    `dispatchAction` adds when they are present (`surface-model.js:125-132`).
  - **`actionId`** is a fresh UUIDv7 for every dispatch, reused on retry
    (`forwarded-props.ts:52`).
- **Context resolution.** A *real* press resolves bound `context` values before
  dispatch, through web_core's `GenericBinder` (`generic-binder.js:331-346`,
  `:382`). Calling `SurfaceModel.dispatchAction` directly skips that step,
  which is why S4 drives the press through the binder.
- **Activity messages in history.** Clients should strip `role: "activity"`
  messages from history (upstream-research.md:106); UAR tolerates them anyway
  (S3).

## 4. Changes

There is one OpenSpec change, `agui-a2ui-activity-slice` (§7), and one Rust
writer.

- **New module:** `src/uar/api/agui_a2ui.rs` holds the logic.
- **`agui.rs`:** gets call sites only, which keeps it under the 800-line
  ceiling.
- **Persistence:** a new sibling record (S0).
- **`manager.rs`:** one new read method (S1).

### S0. *(r4)* Action data in a sibling record; the binding stays unchanged

- **The new record.** `agui_a2ui_actions` is keyed by the binding's
  `storage_key()`:

  ```text
  { binding_key, owner, agent_id, thread_id, workspace_id,
    action_id, digest, surface_id, name, source_component_id,
    context_keys, context_digest }
  ```

  `context_digest = hex(sha256(canonical JSON of userAction.context))`, where
  the canonical JSON has keys sorted and no whitespace. The echo frame (S2)
  carries `{actionId, surfaceId, name, sourceComponentId, contextKeys, contextDigest}`
  from this record.

  `digest = hex(sha256(surfaceId \0 name \0 sourceComponentId \0 catalogId \0 workspace_id))`.
- **Written atomically with the binding.** `create_agui_run` gains an optional
  action argument. When it is present, the action row is written in the
  *same* transaction as the binding row:
  - **Surreal:** the existing `BEGIN … COMMIT` in
    `providers/agui_runs/surreal.rs:7-17`.
  - **Postgres:** the existing transaction in
    `providers/agui_runs/postgres.rs:5-26`.
- **Uniqueness.** A thread can hold an `action_id` only once:
  - **Surreal:** checked inside the transaction, with
    `THROW 'agui_action_conflict'`.
  - **Postgres:** a unique index on `(owner, agent_id, thread_id, action_id)`.
- **Trait change.** The `PersistenceLayer` methods
  `create_agui_run(row, previous)` and its `scoped.rs` wrapper become
  `create_agui_run(row, previous, action: Option<&A2uiActionRecord>)`. Every
  existing caller passes `None`.
  - **New method:** `load_agui_actions(owner, agent, thread)`.
  - **Default for stores without durable AG-UI runs:** an empty list, the same
    as their `create_agui_run` refusal.
- **Postgres migration.** A new file,
  `migrations/20261028120000_agui_a2ui_actions.sql`, creates the table and the
  unique index. It is additive and leaves `agui_runs` untouched.
- **Surreal.** The table is schemaless, matching the existing `agui_runs` and
  `agui_contexts` tables.
- **`AguiRunBinding` is not modified.** A byte-identity guard test protects it
  (§5).
- **Rollback.**
  - **An older binary** ignores both the new table and the Postgres index.
    Every row it reads or compares is unchanged, so threads stay usable.
  - **What is lost:** after a rollback, `actionId` replay and the action echo
    stop working until the new binary returns.

### S1. The thread surface record (unchanged from revision 3, except bounds)

`ThreadSurfaces(thread, anchor_ordinal)` folds the thread's bindings with
ordinal ≤ the anchor's ordinal. It takes the newest 64 of those, reads them
oldest first, and folds each binding's published A2UI messages within its
range, `start_cursor < id ≤ terminal.source_cursor`. An open anchor binding
contributes up to the tip of its run.

- **Events.**
  - **Source:** `published_a2ui_events(run_id, after, up_to)`, a new manager
    method. It merges durable run history (`manager.rs:9172-9190`) with the
    live ring and deduplicates by event id.
  - **Deduplication across bindings:** by native run id and event id.
- **Completeness.**
  - **Gap detection:** event ids are sequential (`manager.rs:126-127`), so a
    gap shows up when ids in the range are missing from both sources. Such a
    binding is marked `history_incomplete`.
  - **Read errors:** a read error marks the binding `history_unavailable`.
- **Fold rules.**
  - **Message handling:** messages are re-parsed with `parse_message`, and the
    top-level `profile` field is stripped.
  - **Component table:** maintained by component id.
  - **Reachability:** components reachable from `root` are actionable, and
    orphaned components are not.
  - **Delete and recreate:** `deleteSurface` removes a surface, and a later
    `createSurface` with the same id starts it fresh.

#### S1a. *(r4)* PEM limits enforced with a margin

Every PEM 4.3.0 default that a cumulative snapshot or a thread's restatement
can hit is enforced at **75 % of the default** (`limits.ts:42-56`). The margin
leaves room for the event envelope and for clients that lower their limits.

| PEM default | UAR bound | Where it is checked |
|---|---|---|
| `maxOperationsPerCall` 500 | 375 operations in one surface's cumulative snapshot | per surface |
| `maxCallBytes` 1 MiB | 786,432 bytes for the serialized snapshot event | per surface |
| `maxComponentsPerSurface` 2,000 | 1,500 components in the current table | per surface |
| `maxComponentDepth` 32 | 24 levels from `root` | per surface |
| `maxComponentBytes` 1 MiB | 786,432 bytes of serialized component properties | per surface |
| `maxDataModelBytes` 1 MiB | 786,432 bytes of the folded data model (each `updateDataModel` applied at its path) | per surface |
| `maxSurfaces` 64 | 48 live surfaces restated per run | per thread |
| `maxComponentsTotal` 10,000 | 7,500 components across restated surfaces | per thread |
| `maxTrackedMessages` 1,000 | at most 48 activity `messageId`s are restated per stream | per thread (structurally below) |

- **Over a per-surface bound.** The surface is marked `over_limit`: it is
  neither restated nor actionable (`422 a2ui_surface_over_limit`), and a
  publication that pushes it over the bound emits no snapshot for it.
- **Over the per-thread bounds.** The *most recently created* 48 surfaces that
  fit within 7,500 components are restated. Older ones are marked
  `over_limit` in the same way.
- **Not applicable to this slice:** `maxBufferedDeltas` and `bufferTimeoutMs`,
  because no deltas are sent.
- **Not bounded by UAR:** `maxTombstones` (1,000) counts deletes the client
  remembers, and PEM evicts tombstones itself. A thread with more than 1,000
  deletes is therefore not refused (§10).

### S2. *(r4)* Outbound: project surfaces as activities

- **`messageId`:** `"a2ui:" + hex(sha256(thread_id \0 surfaceId))[..32]`. It is
  stable per thread and surface.
- **Reserved frame-index range** (revision 3, warning 3). Frames are
  `(cursor, index)` (`agui.rs:659-661`). At the run's start cursor, today's
  code already uses:
  - `(start_cursor, 0)` for `RUN_STARTED` (`:680`);
  - `(cursor, 99)` for errors (`:682`, `:696`, `:718`);
  - `(source_cursor, 1..=3k)`, `(source_cursor, terminal_index − 10)` and
    `(source_cursor, terminal_index ≥ 100)` for caller-tool, close and
    terminal frames (`:699-714`).

  Revision 4 reserves `(start_cursor, 1..=50)` for this slice:

  | Index | Frame |
  |---|---|
  | 1–48 | one `ACTIVITY_SNAPSHOT` per restated surface, in a deterministic order |
  | 49 | `CUSTOM uar.a2ui.restatement_unavailable`, emitted only when a binding is `history_unavailable` or `history_incomplete` |
  | 50 | `CUSTOM uar.a2ui.action.accepted`, emitted only for an action run |

  - **The indices are fixed, not computed from the number of surfaces,** so the
    echo always sits at index 50 and the diagnostic at 49, whatever is
    restated. Unused indices are simply skipped.
  - **No collision with errors or terminal frames.** All reserved indices are
    below 99 and below 100.
  - **Bindings with no source events of their own**
    (`terminal.source_cursor == start_cursor`) emit *no* reserved frames. That
    covers a cold denial (`(0,0]`, `agui.rs:371-382`) and a caller-tool recovery
    sealed on an empty history (`:515`, `:548`). On those bindings,
    caller-tool frames occupy `(start_cursor, 1..=3k)`. Leaving the reserved
    range empty there is the only way to avoid a collision without
    renumbering existing frames.
  - **What a client attached to such a binding sees:** no restatement. That
    binding is never the publisher of a surface. The next normal run on the
    thread restates everything (§10).
  - **Order is monotonic.** Reserved frames are emitted after `RUN_STARTED` and
    before any source event, whose cursor is above `start_cursor`. The existing
    `Last-Event-ID` filter (`(cursor, index) > after`, `:691-692`) skips them on
    a reconnect that has already passed them.
  - **Restatement order:** surfaces are ordered by their first `createSurface`
    in the fold.
  - **`restatement_unavailable` (49):** when emitted, complete surfaces are
    still restated in 1–48.
- **Updates during the run.** Each `ArtifactDisplay` (a2ui) updates the
  projector's working record, and the projector emits one cumulative snapshot
  per surface the batch touched, as source-event frames
  `(event.id, index + 1)`. Two cases are special:
  - **`over_limit`:** a surface over its bound emits nothing and is marked
    `over_limit`.
  - **`deleteSurface`:** emits a final snapshot whose operations end with the
    delete.
- **Order relative to the tool result.** Each snapshot precedes the
  `a2ui_render` `TOOL_CALL_RESULT` (`a2ui_output.rs:89-90`;
  `manager.rs:8444-8463`).
- **What is forwarded.** Messages are forwarded exactly, minus the top-level
  `profile`. web_core's strict schemas reject that field (`protocol.rs:40-41`,
  `:57`, `:202`, `:219`).
- **The echo frame's content** comes from the sibling action record (S0), so a
  reconnect re-emits the identical frame.

### S3. *(r4)* Inbound: `forwardedProps.a2uiAction` as a same-thread run

- **Parsing.** The action is read from the top-level
  `forwardedProps.a2uiAction`, using closed structs. Unknown fields are
  rejected.
  - **`A2uiAction`:** `userAction` (required), `actionId` (optional, 1–128
    characters), `heldVersion` (optional; recorded only) and `dataModel`
    (optional object, at most 64 KiB).
  - **`UserAction`:** `name`, `surfaceId` and `sourceComponentId` (1–256
    characters each), `context` (optional object), `timestamp` (optional),
    and, new in revision 4 (warning 5), `catalogId` (optional) and
    `userMessage` (optional, at most 4 KiB).
    - When `catalogId` is present, it must equal the surface's `catalogId`.
    - The whole `userAction` serializes to at most 16 KiB.
  - **Failure:** `400 invalid_a2ui_action`.
- **Check order.**
  1. **Same `runId`:** the existing replay applies (`agui.rs:185-189`).
  2. **Retry by `actionId`:** look the `actionId` up with `load_agui_actions`.
     - If the digest and workspace match, stream the bound binding (replay).
     - Otherwise, return `409 a2ui_action_id_conflict`.
  3. **The latest-run rule** applies, as today.
  4. **The declaration check** runs against
     `ThreadSurfaces(thread, latest.ordinal)` (unchanged from revision 3). The
     surface must be live, not `over_limit`, and from complete history, and it
     must have a reachable `Button` with `id == sourceComponentId` and a
     matching `action.event.name`. The keys of `context` must be a subset of
     the declared keys. Failures return `422` with
     `a2ui_action_not_declared`, `a2ui_surface_over_limit` or
     `a2ui_surface_history_unavailable`.
  5. **Reserve.** Call `create_agui_run(row, previous, Some(action))`.
  6. **Race re-check (warning 4).**
     - **Store-level fix:** today Surreal's `create` maps only a thrown
       `agui_context_conflict` to `false`. A `TransactionConflict` comes back
       as `Err` and turns into `503` (`surreal.rs:22`; `agui.rs:350`). Revision
       4 makes `create` map `TransactionConflict` and `agui_action_conflict`
       to `false`, the way `transition` already does (`surreal.rs:44-62`). In
       Postgres, a unique-index violation on the action index maps to
       `false`.
     - **Request-level retry:** if create returns `false`, re-run step 2
       *once* against fresh `load_agui_actions`. A concurrent twin is then
       replayed. Any other conflict keeps today's `409 thread_run_conflict`.
- **Building the action run.**
  - **Identity stripping:** it uses the identity-stripping helper extracted
    from `create_request` (`agui.rs:408`).
  - **History:** all prior messages are kept. With client tools present, the
    action input is appended as a new user message. An action together with
    new user text returns `400 a2ui_action_with_text`.
  - **Input:** `CreateRunRequest.input` is a `String` (`routes.rs:90`), so the
    run input is the *serialized* JSON string of
    `{"type": "a2ui.user_action", "sourceRunId", "interaction": {surfaceId, name, sourceComponentId, timestamp, context, catalogId, userMessage, actionId, a2uiClientDataModel, clientContext}}`.
- ***(r4)* Authority, stated precisely** (warning 7).
  - **Fixed by the route:** the path and the verified binding fix the agent and
    the session. The stripped keys are `artifact`, `deployment_binding_id`,
    `input`, `agent_id`, `session_id` and `history`.
  - **Still client-supplied:** `model`, `mcp_servers`, `tool_admission`,
    `run_credentials` and `working_directory` (`routes.rs:73-100`). That is
    *exactly as on every run of this route today*. An action run takes the
    same `forwardedProps.uar` as any other run on the route, so a click adds
    no authority beyond what the same client already has when it sends a text
    run.
  - **Who controls the content:** the action values reach the model as the
    run's user turn, which the client controls, as it does any user text.
- **Activity history messages.** On every run, `role: "activity"` messages are
  dropped instead of failing with `400 unsupported_history_message`
  (`agui.rs:75-78`).
- **REST divergence, stated.** The REST action path keeps its looser
  validation and has no `actionId` (§10).

### S4. *(r4)* Acceptance: I19, pressing through the binder

- **Files:** `scripts/behaviors/features/agui-route-a2ui.feature` and
  `scripts/behaviors/steps/agui-route-a2ui.steps.mjs`.
- **Gherkin.** The same as revision 3, except that every "presses" step is now
  a binder press:

```gherkin
Scenario: An AG-UI app shows the agent's surface, keeps answering its button, and late clients see it
  Given a disposable agent that publishes an action surface on the dedicated AG-UI route
  And a PEM 4.3.0 headless AG-UI client configured with UAR's catalog ids
  When the client asks the agent for a fresh ticket surface whose Confirm button carries a context value bound to the data model
  Then the client renders the surface with the requested heading and a Confirm button
  When the user enters a note and presses the rendered Confirm button
  Then UAR admits a new run that carries exactly that click's actionId and the entered note
  And the agent's reply contains that actionId
  When the user enters a different note and presses the same Confirm button again
  Then UAR admits another run that carries the second click's different actionId and note
  And the agent's reply contains the second actionId
  When the client retries the second click with PEM's retryAction
  Then UAR replays the second run and does not admit a new one
  When a fresh client attaches to the thread's latest run without sending any message
  Then that client renders the same surface and no run is admitted
  When a forged client sends an action the surface never declared
  Then UAR refuses it and the thread's latest run is unchanged
```

- **The press runs through web_core's real press path** (revision 3,
  warning 6).
  1. **Locate the button:** the test finds the rendered Confirm `Button` in the
     PEM headless runtime's `SurfaceModel` (`componentsModel.getAll()`).
  2. **Bind it:** it builds a `ComponentContext(surface, buttonId)` and a
     `GenericBinder(context, ButtonSchema)` from `@a2ui/web_core@0.12.0`. That
     is the version PEM pins exactly, and the same binder its renderers use.
  3. **Press it:** it invokes the bound `action` closure from
     `binder.snapshot`. The binder resolves the button's `context` (for
     example `{"note": {"path": "/form/note"}}`) against the data model, which
     the test sets first through the surface's `dataModel`, as a `TextField`
     edit would. It then calls `dispatchAction`. PEM's `onActionEnvelope`
     supplies `forwardedProps` and `actionId`, which the test sends unchanged.
     The test never types an action name, a component id or a context value
     into the request.
  4. **An open check for Execute:** PEM bundles its own web_core copy
     (`dist/chunk-*.mjs`). Whether a binder from the separately installed
     `@a2ui/web_core@0.12.0` works on a `SurfaceModel` from PEM's bundle is
     unverified.
     - **If it works:** the step proceeds as described.
     - **If it doesn't:** the step stays *failed*, and a PEM issue is filed
       asking for a headless press helper (for example
       `runtime.pressComponent(surfaceId, componentId)`). The test does not
       fall back to calling `dispatchAction` directly.
- **Observables.**
  - **The click and its resolved context are carried** (model-independent).
    The new run's echo frame `(start_cursor, 50)` carries the envelope's
    `actionId`. The `a2ui.user_action` input the agent sees includes the
    resolved `context.note`, which the test checks through the agent's reply:
    the reply must contain the `actionId`. The note itself is checked in the
    echo, which revision 4 extends with `contextKeys` and a
    `contextDigest = sha256(canonical context)` the test can compute
    independently.
    - **Why it proves two clicks:** each click has a fresh `actionId` and a
      different note, so the second click can't be a replay or the model
      repeating itself.
  - **The model saw it** (model-dependent, secondary). The reply contains the
    `actionId`.
  - **Retry.** `retryAction` resends the envelope with a fresh `runId`. The
    response's `RUN_STARTED.runId` is the bound second run's wire `runId`. That
    departs from the AG-UI convention that `runId` echoes the request, and it
    is listed in §10. A later text run with `parentRunId` set to the second run
    is accepted.
  - **Late attach.** A fresh headless runtime calls `GET` on the existing
    reconnect route (`agui.rs:17-18`) for the latest run, with no
    `Last-Event-ID`. The pass condition is that the reserved restatement
    frames render the surface. "No run is admitted" is observed directly: the
    attach is a `GET`, which never reaches admission (`reconnect`,
    `agui.rs:443-449`). A later text run with `parentRunId` set to the same
    latest run is accepted, which shows the latest run is unchanged.
  - **Forged action.** The one hand-built request copies an envelope and
    renames the action to `doesNotExist`. It gets
    `422 a2ui_action_not_declared`. A following text run with `parentRunId`
    set to the latest run is accepted.
- **Bounds.** One model, `liter/MiniMax-M3`, no credentials, and **five** model
  turns: the publication, two clicks, and two follow-up text runs.

## 5. *(r4)* Tests

### 5a. Store tests this repository can run

These replace revision 3's gated integration test (critical 1). They use the
tempfile `surrealkv://` `SurrealDbProvider` that the repository's unit tests
already open (`a2a/handler.rs:319-326`; `thread/service_tests.rs:40-55`). That
provider counts as `EmbeddedFile`, which is durable, so durable AG-UI runs are
supported (`surreal.rs:934`). The tests go in
`src/uar/persistence/providers/agui_runs/surreal.rs` `#[cfg(test)] mod tests`,
which can reach `store.db` (`:7`).

| Test | Proves |
|---|---|
| `binding_written_by_today_reads_and_transitions_unchanged` | A binding inserted raw with today's exact JSON is listed, transitioned and sealed by the new code. |
| `action_record_written_atomically_with_binding` | The binding and action rows commit together or not at all. |
| `duplicate_action_id_conflicts_without_error` | A second create with the same `actionId` returns `false`, not `Err`. |
| `concurrent_creates_one_wins_other_false` | Two concurrent creates on one context give exactly one `true` and one `false`; `TransactionConflict` maps to `false`. |
| `old_binary_shape_ignores_action_table` | The binding JSON contains no action fields, so an older reader is unaffected. |

**Postgres.** The repository has no Postgres instance its tests can open
without external setup. The Postgres half of S0 (the migration, the
transaction and the unique-violation mapping) therefore has **no automated
test in this slice**. It stays unverified, and the gate report must say so. The
binding's Postgres comparison is unaffected, because the binding doesn't
change.

### 5b. Unit tests (authored with the code; run only at the gate)

| Case | Test | File |
|---|---|---|
| The binding's serialization is byte-identical to a fixture captured before the change | `agui_run_binding_serialization_unchanged` | `agui_runs.rs` |
| Durable history beyond the ring, merged with the ring | `published_a2ui_events_reads_durable_beyond_ring`, `published_a2ui_events_merges_ring_and_durable` | `manager.rs` tests |
| Per-binding ranges, the anchored window and frame ids, including the reserved range and the no-source-event case | `fold_uses_per_binding_cursor_ranges`, `restatement_window_anchored_to_binding_ordinal`, `reserved_frame_indices_fixed`, `no_reserved_frames_when_binding_has_no_source_events` | `agui_a2ui.rs` |
| Each limit in S1a | `limit_operations_per_surface`, `limit_snapshot_bytes`, `limit_components_per_surface`, `limit_component_depth`, `limit_component_bytes`, `limit_data_model_bytes`, `limit_surfaces_per_thread`, `limit_components_per_thread` | `agui_a2ui.rs` |
| Snapshot order, delete and recreate, orphaned buttons, `profile` stripping, incomplete history | as in revision 3 | `agui_a2ui.rs` |
| Action checks, including `catalogId` and `userMessage` | `action_check_*`, `action_accepts_web_core_catalog_and_user_message`, `action_catalog_mismatch_refused` | `agui_a2ui.rs` |
| `actionId` replay, digest mismatch, the action builder's identity stripping, activity messages | as in revision 3 | `agui_a2ui.rs` |

## 6. PEM 4.3.0 client contract

Unchanged from revision 3, with these additions:

- **Limits:** the defaults are at `limits.ts:42-56` (S1a).
- **Headless exports:** the headless entry exports no binder
  (`dist/headless.d.ts:3`, `:35`), so the binder comes from
  `@a2ui/web_core@0.12.0` (S4).
- **The binder path:** `GenericBinder` resolves an event action's `context` and
  `userMessage` when invoked (`generic-binder.d.ts:160-163`) and exposes
  `snapshot` and `subscribe` (`:177-180`). `ComponentContext(surface,
  componentId, …)` is defined at `component-context.d.ts:34`.

## 7. Spec delta (OpenSpec `agui-a2ui-activity-slice`)

A new capability, `ag-ui-a2ui-activities`, with five requirements:

1. **Published A2UI surfaces are thread objects on the AG-UI route.**
   - *A cumulative snapshot precedes the tool result.*
   - *The surface is restated in the reserved frames `(start_cursor, 1..=48)`
     with a stable `messageId`.*
   - *A late attach renders it without admission.*
   - *A deleted surface is not restated.*
   - *A surface survives the live ring when durable history exists.*
   - *No surface, no activity.*
   - *A binding with no source events emits no reserved frames.*
2. **Snapshots stay within PEM's default limits with a 25 % margin.** One
   scenario per bound in S1a: an over-limit surface is neither restated nor
   actionable.
3. **Declared A2UI actions are accepted as same-thread runs.**
   - *A reachable declared Button is admitted, and the echo `(start_cursor, 50)`
     carries the `actionId` and the context digest.*
   - *The surface works again after the first answer.*
   - *Undeclared, orphaned or mismatched-catalog actions are refused with 422.*
   - *A retry with the same `actionId` replays, under a race as well.*
   - *A reused `actionId` with a different action is refused with 409.*
   - *Web_core's `catalogId` and `userMessage` fields are accepted.*
4. **Forwarded messages carry no UAR-only fields.** *`profile` is stripped.*
5. **Bindings stay compatible across upgrade and rollback.**
   - *The binding JSON is unchanged.*
   - *Action data lives in a sibling record.*
   - *An older binary is unaffected.*

## 8. *(r4)* Verification (exact commands; not run; the gate is closed)

The steps run serially, after the recorded implementation-complete inventory
and the operator's gate instruction.

0. **Install and validate.**
   - Install the behaviour-test client dependencies:
     `npm install --no-save @prometheus-ags/a2ui-react@4.3.0 @a2ui/web_core@0.12.0`.
     PEM 4.3.0 was confirmed on npm on 2026-10-28.
   - Validate the OpenSpec change:
     `openspec validate agui-a2ui-activity-slice`.
1. **Unit and store tests:**
   `cargo test --locked --no-default-features --features server-full --lib -- agui_a2ui:: agui_runs:: published_a2ui_events_`.
   This includes the §5a store tests under `agui_runs::surreal::tests`.
2. **Rebuild the owned binary in place.**
   - **Open precondition:** Execute must find, in the phase records or the shell
     history, the exact command that built
     `target/uar-team-concurrency-repair/debug/universal-agent-runtime`, and
     write it here verbatim before the gate request.
   - **Candidate (not run until confirmed):**
     `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
3. **Restart the owned UAR with the new binary's digest:**
   `node --input-type=module -e "import { restartOwnedUar } from './scripts/behaviors/support/restart-owned-uar.mjs'; const r = {}; await restartOwnedUar(r, process.argv[1]); console.log(JSON.stringify(r))" "$(shasum -a 256 target/uar-team-concurrency-repair/debug/universal-agent-runtime | cut -d' ' -f1)"`.
4. **Set the environment for the behaviour scenarios:**
   `export UAR_BASE_URL="$(node -p "require('./.prometheus/verification/manual-runtime.json').base_url")" UAR_MODEL=liter/MiniMax-M3 UAR_WORKSPACE_ID=uar-user-outcomes-007-supporting-workspace`.
5. **Run I19 once:**
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/agui-route-a2ui.cucumber.mjs`.
6. **Regression, once each, in this order:**
   1. `selection-stream-replay`
   2. `live-roster-routing`
   3. `native-tool-approval`
   4. `agui`
   5. `writing`

   Run each with
   `node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/<name>.cucumber.mjs`.

C14 and C15 are not rerun.

## 9. Deferred decisions

| Later decision | How the slice stays compatible |
|---|---|
| D3 deltas | They can be added under the same `messageId`. |
| D2 stored record | It replaces the fold. Wire shape, reserved frames and checks stay the same, and it removes the 64-binding window and the per-request cost. |
| D4 versions | `heldVersion` is already recorded. |
| D7 compaction | It relieves the S1a bounds. |
| D11, D15 | The catalog-id mismatch. |
| D17 | The new capability name avoids a collision. |

## 10. The uncomfortable parts

- **An out-of-the-box PEM 4.3.0 client still renders nothing from UAR.** PEM
  defaults to the catalog id `urn:prometheus-ags:a2ui:catalog:v3`, while UAR
  publishes `urn:uar:a2ui:catalog:1` or the basic v0.9 URL. Fixing that belongs
  to D11/D15, and the change's proposal must document the client setting.
- **The binder press may not work across PEM's bundled web_core** (S4). If it
  doesn't, I19's press steps fail until PEM exports a headless press helper. The
  slice does not paper over that by calling `dispatchAction` directly.
- **The Postgres half of S0 has no automated test.** That covers its migration,
  its transaction and the mapping of a unique-index violation to `false`. It is
  reviewed, not verified.
- **Bindings with no source events show no surfaces on attach.** A client that
  attaches to a cold-denied run, or to a caller-tool run recovered on an empty
  history, sees no surfaces until the next normal run. This is the price of
  not renumbering existing frames.
- **A long-lived surface eventually goes over its limit.** Any surface that
  keeps updating passes 375 cumulative operations. From then on it is no longer
  restated or actionable, and the user sees the card stop working.
  - **What the user sees:** for a chat card, this is rare. For a San
    Saba-style dashboard, it is expected within a session.
  - **The real fix:** D3 deltas and D7 compaction. This slice makes the failure
    explicit; it doesn't solve it.
- **Tombstones aren't bounded by UAR.** A thread with more than 1,000 deletes
  depends on PEM evicting its own tombstones.
- **The reconcile blind spot is existing behaviour.** `reconcile` swallows
  history read failures with `unwrap_or_default()` (`agui.rs:514`). It can
  therefore seal a binding at `start_cursor` with an empty range, and such a
  binding shows neither surfaces nor gaps. Fixing that is outside this slice.
- **A replay reports the original `runId`.** A retried action's stream reports
  the bound run's `runId`, not the request's, which differs from AG-UI
  convention.
- **Rollback loses replay.** After a rollback, `actionId` replay and the echo
  are unavailable, though threads stay usable.
- **Two costs remain:**
  - every run and every click re-folds up to 64 bindings' durable history;
  - the REST action path stays looser.
- **The rebuild command is still an open precondition** (§8, step 2).

## 11. Tasks (for Execute; none started)

1. **OpenSpec:** create the `agui-a2ui-activity-slice` change with the §7 delta.
2. **Capture the binding fixture:** record today's exact `AguiRunBinding` JSON
   as a fixture *before* any persistence change, for the guard test in §5b.
3. **S0 storage:**
   - the sibling action record, its atomic write in both stores, the Postgres
     migration and index, and the trait change with all callers updated to
     `None`;
   - mapping `TransactionConflict`, action conflicts and unique violations to
     `false`;
   - the §5a store tests.
4. **Manager:** add `published_a2ui_events` and its tests.
5. **`agui_a2ui.rs`:** the fold, the S1a limits, the reserved-range frames, the
   action parsing and checks (including web_core's fields) and the builder,
   with the §5b tests authored but not run.
6. **`agui.rs` wiring.**
7. **I19 (S4):** the binder press, plus the cross-bundle check.
8. **Confirm the rebuild command** (§8, step 2).
9. **Record the implementation-complete inventory,** then wait for the
   operator's gate instruction.

## Revision 4 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 4 against UAR, PEM 4.3.0
and web_core 0.12.0. It ran nothing. Verdict: **defects found** — 1 critical,
9 warnings, 8 suggestions. The lead confirmed the starred items in the source
(✓).

### Critical

1. **✓ Rollback on Postgres breaks: the previous binary won't start.**
   - **How migrations run:** UAR runs `sqlx::migrate!("./migrations").run(&pool)`
     at startup (`providers/postgres.rs:70`). sqlx-core 0.8.6 returns
     `MigrateError::VersionMissing` when an applied migration is unknown to the
     binary, unless `ignore_missing` is set (`migrator.rs:32-51`; default
     `false`). UAR never sets it.
   - **Consequence:** once the new migration is applied, the older binary
     refuses to boot.
   - **Why the gate misses it:** the gate builds `server-full`, which doesn't
     include `postgres-backend` (`providers/mod.rs:3`; `Cargo.toml:259`).
   - **Scope:** this holds for *any* new Postgres migration in this repo, not
     only this one. But it falsifies revision 4's central claim.

### Warnings

2. **Mapping `TransactionConflict` to `false` in `create` loses a click.**
   - **What `transition` actually does:** it re-reads the row and returns
     `false` only if its predicate no longer matches (`surreal.rs:46-61`).
   - **The effect of a blind mapping in `create`:** a transient `503` becomes a
     final `409`, and the click is lost.
   - **Twins already serialize:** two creates for the same thread already
     contend on the `agui_contexts` slot (`surreal.rs:8-13`;
     `postgres.rs:8-23`).
3. **The exemption for bindings with no source events can't be decided when
   the reserved frames go out.**
   - **When it becomes known:** the binding is usually still open at that
     point. `reconcile` can later seal it at `start_cursor` with
     `clientToolCalls` (`agui.rs:515`, `:548`).
   - **What then happens:** the caller-tool frames `(start_cursor, 1..=3k)`
     reuse already-emitted ids, so the stream carries duplicate SSE ids.
   - **Restatement should exclude its own run:** it must fold only ordinals
     below the stream's binding. Otherwise the same id carries different
     content on each reconnect.
4. **✓ Surreal's AG-UI tables are `SCHEMAFULL`, not schemaless**
   (`migrations/surrealdb/agui_runs.surql:1`, `:7`; registered at
   `surreal_host.rs:114`). The plan adds no `.surql` file and no index, so
   lookups and the uniqueness check scan the whole table.
5. **✓ The Postgres half isn't compiled by the gate** (see the critical
   finding). "Reviewed, not verified" should read "not compiled".
6. **The binder API names are wrong.**
   - **The schema:** v0.9 and v1.0 use `ButtonApi.schema` from
     `@a2ui/web_core/v0_9/basic_catalog` or `v1_0`. `ButtonSchema` exists only
     for v0.8.
   - **`ComponentContext`:** it lives under `resolution/`.
   - **The action closure:** it doesn't return the dispatch promise, so the
     test must await `onActionEnvelope` instead.
   - **"The same binder its renderers use" is false:** PEM's renderers use its
     bundled copy.
7. **S4 omits the action policy.** PEM's default denies every action
   (`runtime-core.ts:124`, `:138-140`).
8. **The per-thread caps cover restatement only.** A single live run can still
   exceed PEM's runtime-wide `maxSurfaces` (64; `limit-checks.ts:150`).
9. **The 64 KiB `dataModel` cap is too low.** PEM sends every surface's data
   model in one object (`runtime-core.ts:637-643`, `:749`).
10. **`npm install --no-save` in a pnpm repository** risks pruning pnpm's
    links. Use pnpm, or a separate prefix.

### Suggestions

- **Depth:** PEM measures it from every unreferenced component, not only from
  `root` (`limit-checks.ts:54-96`).
- **The echo proves storage, not input:** it shows the action was stored, not
  that it reached the run's input.
- **Postgres unique violations** map to `false` only after `tx.rollback()`.
- **`load_agui_actions`** must be added to `read_only_operation`
  (`scoped.rs:95-117`).
- **The `create` default bails** rather than returning an empty list
  (`mod.rs:118`).
- **The authority list** also misses `response_format`, `service_placement`
  and `reasoning_effort`.
- **Activity-role messages** are also refused at `agui.rs:397`.
- **The concurrency test** can't reliably trigger a `TransactionConflict`.

### Confirmed sound

- the atomic write fits in both existing transactions;
- the trait has four implementors and one caller;
- the tempfile `surrealkv` store is `LocalDurable` and loads `agui_runs.surql`;
- the reserved indices clear 99, 100 and above, and `terminal_index − 10`, when
  `start_cursor` is 0;
- `reconnect` never admits a run;
- the cucumber, openspec and `restartOwnedUar` commands are real.

### Lead assessment

The sibling record fixed rollback on Surreal and created a new rollback
failure on Postgres. Any new Postgres migration makes the previous binary
refuse to start. A schema-free option exists:

- **Store the action as a run event.** The action run's first event is a
  `NormalizedEvent::Custom` (`uar.a2ui.action`). Durable run history already
  persists it, and the fold already reads that history. The echo, the
  `actionId` lookup and the context digest all come from that event.
- **What it removes:**
  - every binding change;
  - every new table and migration;
  - the race mapping;
  - most of warnings 2, 4 and 5.
- **Its cost:** a concurrent twin whose winner hasn't written its first event
  yet gets today's `409 thread_run_not_closed` instead of a replay. A later
  retry then replays normally.

That trade is the operator's to make. Warnings 3 and 6–10 are specification
fixes either way.

After four revisions, the design is stable: thread-scoped surfaces and
per-binding ranges. What remains comes from where the action data is stored
and from client-library details.
