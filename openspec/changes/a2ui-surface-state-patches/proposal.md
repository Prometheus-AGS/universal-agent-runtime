## Why

UAR publishes every A2UI message twice: once to the per-run surface replay
log and once as a `STATE_DELTA` state patch. Both use the same patch, built by
`surface_message_to_state_patch` (`src/uar/a2ui/realtime.rs:69-97`). That
patch encodes the message, not the state the message produces:

- **Updates fail to apply.** `updateComponents` and `updateDataModel` become
  `replace` at `/a2ui/surfaces/<id>/components` and `/dataModel`.
  `createSurface` never creates those keys, so the first update fails to
  apply under RFC 6902. The server snapshot builder then stops claiming state
  (`src/uar/api/sse.rs:79`, `:143-146`), and the client adapter does the same
  (`frontend/src/platform/agui/agui-adapter.ts:72`).
- **Each patch carries one message's payload.** A second `updateDataModel` or
  a partial `updateComponents` therefore overwrites earlier state. The
  `@a2ui/web_core` 0.12.0 state operations behave differently: they upsert
  components by id and set data by path.
- **Actions fail to apply.** An action adds `/a2ui/actions/<uuid>`
  (`src/uar/a2ui/routes.rs:866-870`), but the initial state has no `actions`
  object (`sse.rs:127-130`).

As a result, the run stream's `STATE_SNAPSHOT` (`GET
/api/uar/runs/{id}/stream?stream_mode=agui_spec`) drops all state once a run
updates a surface or receives an action.

This is the first of three changes that replace the single AG-UI/A2UI plan.
The second adds a durable surface store scoped to a thread. The third rebuilds
`agui-a2ui-projection-and-custom-components` on the first two.

## What Changes

- **Server-side reduction.** Reduce each run's surfaces in the A2UI replay
  backbone, under the same lock as the log. Each message is reduced on its
  own by a port of `web_core` 0.12.0's four state operations: `createSurface`,
  `deleteSurface`, component upsert, and `DataModel.set`/`get`, statement by
  statement, including the mutation it makes before a throw. Nothing else
  from `web_core` is mirrored.
- **RFC 6902 patches for exactly the state change.**
  - One patch per changed component.
  - One patch at the data-model change site, using `replace` inside an array,
    `add` at its end, and the whole enclosing array for a write more than one
    index past the end.
  - Values are compared numerically. An unchanged message emits no patch.
  - A refused message is still logged and published, and the refusal is
    logged too.
- **Hardening.** Refuse surface and component ids of `__proto__`,
  `constructor` or `prototype` for state. This is a trust-boundary measure and
  the one deliberate departure from the reference.
- **Initial state.** ADD to `ag-ui-chat-conformance` a requirement that
  defines the UAR initial state with `actions: {}`. The shared requirement
  "Cursor-consistent attach and replay snapshots" stays with
  `agui-runs-stream-fidelity`, which records the coordination.
- **Runtime console.** MODIFY `runtime-console` "Validated A2UI Surface
  Replay": newer changes come from re-reading the log, and action entries are
  skipped. The console fix is a one-function change in
  `run-trace-projection.ts`.
- **Log and trigger unchanged.** Keep the replay log and the surface test
  trigger as they are. Correct the doc comments that describe the log as live
  state.
- **The guarantee's conditions in the spec.** The guarantee holds only while
  all of these hold:
  - the history ring still holds the run's surface creations;
  - the trigger has not been called for the run;
  - no publications to the run were concurrent.

  It is also scoped to a single run: continuations start with no surfaces.
- **Superseded change.** Withdraw the overlapping requirement and tasks from
  the superseded change, and give it a superseded banner.
- **C14.** Update C14's two state assertions, and extend it with:
  - bounded reads on numeric cursors;
  - component and forbidden-id updates;
  - a seeded generated data-model corpus with 18 fixed edge cases, compared
    per body and at the end with `web_core`'s `DataModel`;
  - a deletion and an absent surface;
  - an action with an explicit continuation check;
  - the log and the trigger.

## Capabilities

### New Capabilities

- `a2ui-surface-state`: per-message reduction with `web_core`'s state
  operations, the guarantee's conditions, and the replay log as a message and
  action log.

### Modified Capabilities

- `ag-ui-chat-conformance`: ADDED requirement "The UAR initial state holds
  A2UI surfaces and actions". No existing requirement is modified.
- `runtime-console`: MODIFIED "Validated A2UI Surface Replay". The console
  re-reads the log, does not merge live state patches, and skips action
  entries.

The unarchived change `a2ui-realtime-backbone-from-flint-realtime-fabric` keeps
all three of its requirements: `surface_message_to_state_patch`, the trigger, and the
replay trait's `publish` and `replay` are
unchanged.

## Impact

- **Rust:** `src/uar/a2ui/protocol.rs`, `src/uar/a2ui/realtime.rs`,
  `src/uar/runtime/a2ui_output.rs`, `src/uar/a2ui/routes.rs` (publication and
  doc comments) and `src/uar/api/sse.rs`.
- **Docs:** `docs/protocols/ag-ui-profile.md`, `docs/protocols/a2ui-profile.md`
  and `frontend/packages/a2ui-core/UPSTREAM.md`.
- **Other changes:** `agui-a2ui-projection-and-custom-components` (withdrawal)
  and `agui-runs-stream-fidelity` (coordination note). Both are already
  edited.
- **Behavior scenario:** `scripts/behaviors/features/a2ui-response.feature`
  and `scripts/behaviors/steps/a2ui-response.steps.mjs` (C14).
- **Frontend:** `frontend/src/features/chat/model/run-trace-projection.ts`
  (skip action entries); `pnpm build` at the gate.
- **Public API:** `A2uiReplayBackbone` (exported via `lib.rs:44`) gains a
  required method, which breaks external implementers. There is one in-tree
  implementer.
- **Reruns at the gate:** C14, and C15, whose publication path changed.
- **Not visible to users:** the live chat stream sends no snapshot, and the
  renderer renders each artifact on its own.
