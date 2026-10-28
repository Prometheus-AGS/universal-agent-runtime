# Tasks

Revision 4. Change 1 of 3 in the re-scoped AG-UI/A2UI work. These tasks
assume the recommended answers to the design's open questions Q2-Q6. Q1 is
resolved by D11.

The verification lock applies: tests may be authored, but nothing runs before
group 6. Groups 1-5 are blocked until tasks 0.1 and 0.2 are done.

## 0. Workflow (lead)

- [ ] 0.1 Record the operator's answers to Q2-Q6 in the KBD plan, and adjust groups 1-5 if any answer differs
- [ ] 0.2 Register this change in KBD through the runtime, together with reconciling the existing child directories, so that the waypoint names this change; do not hand-edit generated files
- [x] 0.3 Withdraw the overlapping patch requirement from `agui-a2ui-projection-and-custom-components`, add its superseded banner, and clean up its withdrawn tasks (design D10)
- [x] 0.4 Record the coordination with `agui-runs-stream-fidelity` in that change's proposal (design D6)

## 1. Types and reducer (`src/uar/a2ui/realtime.rs`, `src/uar/a2ui/protocol.rs`)

- [ ] 1.1 Add documented `pub` types to `realtime.rs`: `A2uiStateOperation`, `StateRefusal` (with `Display`) and `StateOutcome { patches, refusal }` (design D3). Build the operation in `parse_message` (`protocol.rs:292-327`) and carry it on `ValidatedA2uiMessage` (`protocol.rs:20-25`)
- [ ] 1.2 Port `DataModel.set` statements S1-S9 and `get` per design D2, in that order, keeping mutation made before a throw. Port JavaScript `undefined` as follows: array holes are `null`, and `"//"` targets the key `"undefined"`
- [ ] 1.3 Port `createSurface`, `deleteSurface` and the component upsert per design D2, and add the `ForbiddenId` whole-message refusal (design D3)
- [ ] 1.4 Implement value equality per design D2: structural, with numbers compared by numeric value
- [ ] 1.5 Implement patch encoding per design D4: escape every segment; per-component `add` for changed components only; change site P; object `add`/`remove`; array `replace`, `add` at the end, and a whole enclosing array when the write lands more than one index past the end
- [ ] 1.6 Add per-run reduced surfaces to `InMemoryReplayBackbone` under the `history` mutex, and clear them in `remove`
- [ ] 1.7 Add `A2uiReplayBackbone::publish_message(run_id, surface_id, kind, payload, &A2uiStateOperation) -> StateOutcome`, with `///` docs: append the unchanged log entry, then reduce. Leave `surface_message_to_state_patch`, `publish` and `replay` unchanged

## 2. Publication paths

- [ ] 2.1 In `src/uar/runtime/a2ui_output.rs:160-173`, call `publish_message`, and emit one `StatePatch` holding the outcome's patches when the list is non-empty. Log a refusal with the run id, surface id and reason, and no payload: `SurfaceExists` and `SurfaceAbsent` with `tracing::debug!`, others with `tracing::warn!`. Artifact, source lines and diagnostics stay unchanged
- [ ] 2.2 In `publish_messages` (`src/uar/a2ui/routes.rs:739-755`), do the same; HTTP responses are unchanged
- [ ] 2.3 Leave `surface_test_trigger` (`routes.rs:1325`) and `submit_action` (`routes.rs:866-890`) unchanged

## 3. Snapshot initial state and console

- [ ] 3.1 In `build_agui_replay_snapshot` (`src/uar/api/sse.rs:127-130`), start from `{"run":null,"a2ui":{"surfaces":{},"actions":{}}}`
- [ ] 3.2 In `projectA2uiReplay` (`frontend/src/features/chat/model/run-trace-projection.ts:343-357`), skip operations whose path starts with `/a2ui/actions/` before `replayEnvelope`; other behavior is unchanged (design D11). This is a pure function and not a React component or entity hook

## 4. Documentation and doc comments

- [ ] 4.1 In `docs/protocols/ag-ui-profile.md`, document:
  - the run state shape and the initial state;
  - the A2UI state patches;
  - partial mutation, refusal and no-op;
  - that only the run stream route sends `STATE_SNAPSHOT`;
  - limits L1-L4, L6 and L8.
- [ ] 4.2 In `docs/protocols/a2ui-profile.md`, add:
  - the server reduction (a port of four `web_core` state operations, per message);
  - the `ForbiddenId` departure;
  - that `surface-replay` is a message and action log;
  - limits L7 and L9.
- [ ] 4.3 In `frontend/packages/a2ui-core/UPSTREAM.md` ("How to update the pin"), add: re-port the reducer in `src/uar/a2ui/realtime.rs` against the new `MessageProcessor` state operations and `DataModel.set`/`get`, update C14's expected version, and rerun C14
- [ ] 4.4 Correct the doc comments that call the log the late-join path for live state, or that say the patch alone reconstructs the surface: `realtime.rs:20-37`, `:61-67` and `:104-111`; `routes.rs:11-12`, `:48-51`, `:782-783`, `:1286-1295` and `:1351-1357`

## 5. Behavior scenario (lead; authored now, run only at the gate)

- [ ] 5.1 In `scripts/behaviors/steps/a2ui-response.steps.mjs:155-161`, assert:
  - the create patch is `add /a2ui/surfaces/<sid>` with value `{surfaceId, catalogId, components:{}, dataModel:{}}`;
  - the component patch is `add /a2ui/surfaces/<sid>/components/root` with value equal to the published root component.
- [ ] 5.2 Add a bounded SSE reader to the step file. It parses `id:`, `event:` and `data:`. It stops at a named frame type or when a numeric SSE id reaches a target. It fails after 60 s, naming what was missing. It never uses payload `eventId` as a cursor.
- [ ] 5.3 Add a `web_core` reference loader. It:
  - uses `createRequire` from `frontend/packages/a2ui-core/package.json` to resolve `@a2ui/web_core/state`;
  - reads `<resolved>/../../package.json` and asserts name `@a2ui/web_core` and version `0.12.0`;
  - imports the module by file URL;
  - uses one `DataModel({})` per surface, applying messages in order, each in its own try/catch, and records changed, unchanged or refused from the JSON forms.
- [ ] 5.4 Add the seeded generator: mulberry32, seed from `UAR_C14_SEED` or random, recorded in the evidence. Use the segment, path-form and value distributions in design D9 step 3, and keep the 18 fixed cases at the start of G0.
- [ ] 5.5 Extend `scripts/behaviors/features/a2ui-response.feature` (C14) with design D9 steps 1-9 as separate Gherkin steps, with a 180 s timeout on the continuation step. Include:
  - all frames collected (step 6) before the comparisons (step 7);
  - per-body pairing by `artifactId`;
  - the stated counts for bodies without data messages and for the action tail;
  - the running-state comparison after each corpus body, and against S2;
  - the event-budget assertion;
  - L2 diagnosis.

  The evidence records the seed, every message, its reference outcome, the per-body counts and the first mismatch with its index and both values. No credentials are used.
- [ ] 5.6 Record the implementation-complete inventory with exact source identity, and present the gate commands to the operator

## 6. Verification boundary (operator-approved commands only)

- [ ] 6.1 Build:
  - `cargo build` for the owned binary profile;
  - `cargo test --no-run` for the same profile, which compiles existing tests (`presentations_tests.rs:318-349` and the two integration tests) and runs nothing;
  - `pnpm build` (the console change), then copy `static/` into the owned UI directory;
  - restart the owned instance.
- [ ] 6.2 Run `openspec validate a2ui-surface-state-patches`
- [ ] 6.3 Run C14 once and review the raw evidence, including the seed
- [ ] 6.4 Rerun C15, whose publication path (`a2ui_output.rs`) changed
- [ ] 6.5 Record receipts, catalog credits and the reflection, including the coverage table below

## 7. Coverage (for the reflection)

| Spec scenario | Verified by |
|---|---|
| Created surface, first component update | C14 existing steps; D9 step 1 |
| Partial component update | D9 steps 2.2 and 7.5 |
| Forbidden component id | D9 steps 2.3, 7.3 and 7.5 |
| Data-model state equals `DataModel.set` on generated input | D9 steps 3 and 7.1-7.4 (seeded corpus plus 18 fixed cases, compared with `web_core`) |
| Containers kept before a refusal | D9 fixed case 16, through the step 7 comparison |
| Update for a surface not held | D9 steps 4, 7.3, 7.5 and 8 |
| Deleted surface | D9 steps 4 and 7.5 |
| Continuation does not hold source surface | D9 step 5 |
| Trigger voids guarantee; trigger payload as sent | D9 step 9 |
| Log encoding, refused messages and actions | D9 step 8 |
| Conformance: action patch applies | D9 step 7.5 |
| Conformance: cursor resume reaches newest state | D9 steps 6 and 7.4 |
| Console skips action entries | source review only (pure function; no console scenario) |
| Component upsert compared with `web_core` `MessageProcessor` | hand assertions only |
| Refusal log levels; retention discard (`manager.rs:1610`) | source review only |
| Limits L1 and L2 | not verified; named limits (the event budget keeps R clear of L2) |

## Workflow follow-up

- Archive this change after the operator accepts the results. It can be archived before or after `agui-runs-stream-fidelity`.
- Follow-ups pending the operator's answers:
  - a cumulative byte cap (Q3);
  - aligning admission with the client validator and catalog (Q4);
  - a per-run reduce-and-emit lock (Q5).
- Next: plan change 2 (a durable surface store scoped to a thread), which resolves L8 and takes up Q5.
