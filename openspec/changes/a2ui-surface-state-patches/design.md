# Design: A2UI surface state patches

Revision 4, 2026-10-27. Change 1 of 3 in the re-scoped AG-UI/A2UI work.
Revision 4 addresses the revision 3 review: C1, W1-W6, and the suggestions.
All three earlier reviews are kept at the end of this file.

Revision 4 changes how D2 is written. It is no longer a list of rules derived
from the reference. It is a port of `DataModel.set` and `get`, statement by
statement, including mutation before a throw. A seeded, generated-input
comparison with `web_core` replaces the hand-picked check.

## Context

Three A2UI publication paths call `surface_message_to_state_patch` and use the
result twice:

| Path | Source | Log | Event |
|---|---|---|---|
| `a2ui_render` / `presentation_render` tool output | `src/uar/runtime/a2ui_output.rs:165` | `backbone.publish` | `StatePatch` |
| `POST /api/uar/runs/{id}/a2ui/messages` (202, returns `artifactId`) | `src/uar/a2ui/routes.rs:743` | `realtime_backbone.publish` | `StatePatch` |
| `POST .../a2ui/surface-test-trigger` (200) | `src/uar/a2ui/routes.rs:1325` | same | same |

Two more facts about the publication paths:

- **Actions.** `submit_action` (202) writes `add /a2ui/actions/<uuid>` to both
  the log and the event stream. Its continuation is a new run
  (`routes.rs:866-890`).
- **Event order.** Each `publish_messages` call emits zero or one
  `StatePatch` per message, then exactly one `ArtifactDisplay`
  (`routes.rs:739-776`). On the run stream, that artifact is the frame
  `CUSTOM uar.artifact.available`, whose `value.artifactId` equals the 202
  response's `artifactId` (`src/uar/api/adapters.rs:208-218`).

Log readers (`GET .../a2ui/surface-replay`):

- **Runtime console.** It re-fetches the log only
  (`frontend/src/features/chat/model/run-trace-projection.ts:312-357`,
  `run-trace-store.ts:264-283`). It throws on any entry outside
  `/a2ui/surfaces/` (regex at `:39`, throw at `:314`). D11 fixes this for
  action entries.
- **The action check** (`routes.rs:837-848`).
- **Two integration tests** (`tests/run_retention/runs.rs`,
  `tests/sidecar_session_principal/isolation.rs`).

Event readers apply `StatePatch` as RFC 6902:

- **The run stream snapshot builder** (`src/uar/api/sse.rs:122-167`). Only
  `stream_run` uses it (`src/uar/api/routes.rs:895-916`).
- **The client adapter** (`frontend/src/platform/agui/agui-adapter.ts:41-80`).
- **The presentation observer** (`src/uar/runtime/presentations.rs:74-91`).

The chat renderer uses its own path:

- It renders each artifact with a fresh `@a2ui/web_core` `MessageProcessor`
  that registers only `urn:uar:a2ui:catalog:1`
  (`frontend/src/features/chat/components/a2ui-artifact-block.tsx:271-285`;
  `frontend/packages/a2ui-uar/src/catalog/uar-basic-catalog.ts:32,54-58`).
- `web_core` is pinned to 0.12.0.

Coordinating changes:

- `agui-runs-stream-fidelity` (unstarted). It owns "Cursor-consistent attach
  and replay snapshots" and its eviction base (D6).
- `agui-a2ui-projection-and-custom-components` (superseded). Its overlap is
  withdrawn, and it now carries a superseded banner (D10).

## Delta from revision 3

| Finding | Revision 4 |
|---|---|
| C1(a) absent test | The no-op test is a port of `get(path) === undefined`, including null intermediates, non-numeric segments on arrays, and holes (D2). |
| C1(b) vivification limit | A numeric next segment above 10000 throws during vivification (D2, statement S7). |
| C1(c) partial mutation | Containers created before a throw stay. Patches are emitted for them, and the refusal is logged (D2, D4). The result type is now an outcome, not a `Result` (D3). |
| C1(d) departures | The only departure is `ForbiddenId` (D3). |
| C1 coverage gap | A seeded generator produces data-model messages. Fixed edge cases are added in front, including every C1 case. Node replays everything through `web_core`'s `DataModel` and compares the results per body and at the end (D9). |
| W1 step order | All stream frames are collected before any comparison. Bodies are paired with frames by `artifactId` (D9). |
| W2 version check | The version is read from the `package.json` two directories above the resolved `src/state/index.js` (D9). |
| W3 gap size | A write more than one index past an array's end replaces the enclosing array once. There is no per-index fill (D4). The size bound is stated exactly. |
| W4 continuation | After the continuation finishes, the step posts an update for R's surface to the continuation run. It then checks that the continuation's snapshot lacks that surface. `RUN_ERROR` fails the step (D9). |
| W5 id hardening | A message with a `__proto__` component id is part of D9. The trigger's unhardened ids are named in L3. |
| W6 console | The console skips `/a2ui/actions/` entries (D11). The MODIFIED requirement gains a scenario for this. |
| Suggestions | L1 now states the consequence, Q5 is reopened, numeric equality is defined, there is one `DataModel` per surface, every create states its `catalogId`/`version`, line references are corrected, `SurfaceExists` logs at debug, the public-API break is named, and the superseded banner is added. |

## Decisions

### D1. The log and the state are separate outputs of one publication

Each admitted message produces two outputs:

- **The log entry.** It is unchanged: `surface_message_to_state_patch`,
  stored by `publish`.
- **New state patches.** These come from reducing the run's surface state.
  They are emitted as one `StatePatch` event, and only when non-empty.

### D2. The reduction is a port of web_core 0.12.0's four state operations

The reduction mirrors the state effect of four operations in
`processing/message-processor.js`: `processCreateSurfaceOp`,
`processDeleteSurfaceOp`, `applyComponentUpdate` and `processUpdateDataModelOp`.
It does **not** mirror catalog lookup, version compatibility,
`PayloadValidator`, composition and topology checks, batch abort, or
rendering. Each message is reduced on its own.

- **`createSurface`.** A surface id that already exists is refused
  (`SurfaceExists`) and nothing changes. Otherwise the surface becomes
  `{surfaceId, catalogId, components:{}, dataModel:{}}` (`protocol.rs:45-50`).
- **`deleteSurface`.** The surface is removed. An absent surface is a no-op.
- **`updateComponents`.** An absent surface is refused. Otherwise each
  component is upserted by `id`, as sent (`protocol.rs:68-80`).
- **`updateDataModel`.** An absent surface is refused. Otherwise
  `processUpdateDataModelOp` calls `set(path || "/", value)`. The surface's
  `dataModel` is the model.

**Port of `DataModel.set(path, value)` (`state/data-model.js:84-196`).**
Statements run in this order. A throw keeps every mutation made before it.

- **S1.** If `path` is `"/"` or `""`, the model becomes `value`, or `{}` when
  `value` is `null`. Stop.
- **S2.** `segments = parsePath(path)`:
  - split on `/`, then drop empty strings;
  - unescape each segment in a single left-to-right pass of `~1` to `/` and
    `~0` to `~`;
  - throw `ForbiddenSegment` on `__proto__`, `constructor` or `prototype`.
  - Then `last = segments.pop()`. When `segments` was empty (for example
    `"//"`), `last` is the string `"undefined"`. This mirrors JavaScript's
    property key for `undefined`.
- **S3.** If `value` is `null` and `get(path)` is absent, stop. `get` is
  ported below. It uses the original path, so `"//"` resolves to the model
  root, which is present.
- **S4.** If the model root is neither an object nor an array, throw
  `PrimitiveRoot`. The model is never `null` at this point.
- **S5.** Set `current` to the model. For each non-last segment `seg` at
  position i, let `next` be `segments[i+1]`, or `last` at the final position.
  - **S6, when `current` is an array:**
    - `seg` must match `^(0|[1-9]\d*)$`, or throw `NonNumericArraySegment`;
    - its index must be at most 10000, or throw `ArrayIndexLimit`;
    - if the element is present and neither an object nor an array, throw
      `PrimitiveIntermediate`;
    - if the element is absent or `null`, it is vivified by S7.
  - **S6', when `current` is an object:**
    - if `seg` is an own key whose value is present and neither an object nor
      an array, throw `PrimitiveIntermediate`;
    - if `seg` is missing or its value is `null`, it is vivified by S7.
  - **S7, vivification:**
    - if `next` is numeric and above 10000, throw `ArrayIndexLimit`;
    - otherwise create `[]` if `next` is numeric, and `{}` if not;
    - an array index past the end fills the gap with `null`.
  - Then `current` becomes `current[seg]`.
- **S8.** If `current` is an array, `last` must be numeric
  (`NonNumericArraySegment`) and at most 10000 (`ArrayIndexLimit`).
- **S9.** If `value` is `null`:
  - on an array, set the element to `null` when the index is in range;
  - on an object, delete the key.

  Otherwise set `current[last] = value`. Past an array's end, the gap fills
  with `null`.

**Port of `get(path)` (`data-model.js:234-265`), used only by S3.**

- `"/"` or `""` returns the model.
- Otherwise, walk the `parsePath` segments. The result is **absent** when, at
  any step:
  - the current value is `null` or neither an object nor an array;
  - on an array, the segment is non-numeric, or the index is out of range, or
    the element is `null` (a hole, see below);
  - on an object, the segment is not an own key.
- An object property whose value is `null` is **present**.

**Holes.** Rust stores JavaScript holes and `undefined` elements as `null`, so
a `null` array element counts as absent in `get` and is vivified in S6.
Explicit JSON `null` array elements sent in a value behave differently in
JavaScript (`get` returns `null`). The only consequence is in S3: JavaScript
then sets the element to `undefined`, while Rust stops. Both leave the JSON
state `null`, so the JSON results are equal, and both emit no patch, because
the JSON state did not change.

**Equality.** Two JSON values are equal when they are structurally equal and
numbers compare by numeric value, as in JavaScript `===` (so `1` equals
`1.0`). Object key order is ignored.

### D3. Outcome type, refusals and id hardening

`pub enum A2uiStateOperation` lives in `realtime.rs`, with the variants
`Create { catalog_id }`, `Components(Vec<(String, Value)>)`,
`Data { path, value }` and `Delete`. `parse_message` builds it, and
`ValidatedA2uiMessage` carries it.

The reducer returns a `pub struct StateOutcome` with these fields:

- `patches: Vec<StatePatchOp>`;
- `refusal: Option<StateRefusal>`.

**Refusal reasons.** `pub enum StateRefusal` has the variants
`SurfaceExists`, `SurfaceAbsent`, `ForbiddenSegment`, `ForbiddenId`,
`PrimitiveRoot`, `PrimitiveIntermediate`, `NonNumericArraySegment` and
`ArrayIndexLimit`.

**Partial mutation.** A data-model refusal can still carry patches, for
example `/p/q/20000=1` yields `{p:{}}`.

**Hardening.** One departure from the reference is deliberate. Model-controlled
surface and component ids become object keys in the client's JSON applier
(`agui-adapter.ts:71-73`). An id equal to `__proto__`, `constructor` or
`prototype` therefore refuses the whole message with `ForbiddenId`, with no
state change and no patches.

**Logging.** A refusal is logged with the run id, surface id and reason, and
no payload:

- `SurfaceExists` and `SurfaceAbsent` at `debug`, because they are normal in
  re-renders and in L8;
- all other reasons at `warn`.

Publication itself is unchanged.

### D4. Patch encoding

Every pointer segment is escaped with `~0`/`~1`.

| Message | Patches |
|---|---|
| `createSurface` | `add /a2ui/surfaces/<sid>` with the new surface |
| `updateComponents` | one `add /a2ui/surfaces/<sid>/components/<cid>` per component whose value changed |
| `updateDataModel` | the patch for change site P, below |
| `deleteSurface` | `remove /a2ui/surfaces/<sid>` |

**Change site P.** Let T be the touched key path: `segments` followed by
`last`, or `[]` for S1. P is chosen as follows:

- for S1, P is `dataModel`;
- otherwise, P is the shortest prefix of T whose node was absent or `null`
  before the message, or the whole of T if every node existed.

Every node above P existed before and stays the same container. That holds
because S6 and S7 only create nodes at absent or `null` positions, and S9
only writes at T. A throw in S6-S8 can occur only at or below P, so any
containers it leaves behind sit under P.

Compare `old` and `new` at P with D2 equality. When they are equal, or both
absent, emit nothing. Otherwise the patch depends on P's parent:

- **The parent is an object, or P is `dataModel`:** emit `add P new`, or
  `remove P` when `new` is absent.
- **The parent is an array, with index k and old length L:**
  - **k < L:** `replace P new`.
  - **k = L:** `add P new`.
  - **k > L:** one patch for the enclosing array A at its own pointer. Use
    `add A newA` when A's parent is an object or the data model, and
    `replace A newA` when A's parent is an array. Both are valid, because A
    existed before.

**Size.** For k ≤ L and for objects, a patch is at most the size of the
message's value plus the containers that S7 created. A write more than one
index past an array's end sends the enclosing array once, with at most 10001
elements of existing content. That is the exact bound.

Both appliers implement `add`, `remove` and `replace` this way
(`sse.rs:44-118`; `agui-adapter.ts:41-80`).

### D5. Reduced state in the backbone; public trait method

`InMemoryReplayBackbone` keeps one reduced surface map per run, held under
the same mutex as `history`. `remove` drops it (`manager.rs:1610`).

The `pub` trait (exported via `lib.rs:44`) gains this method:

```
fn publish_message(&self, run_id: &str, surface_id: &str, kind: A2uiWireKind,
                   payload: &Value, operation: &A2uiStateOperation) -> StateOutcome
```

It appends the unchanged log entry and then reduces. Every type it uses is
`pub` (`StatePatchOp` at `src/uar/domain/events.rs:466`), so it raises no
`private_interfaces` warning.

Adding a required method breaks external implementers of the trait. The only
in-tree implementer is `realtime.rs:144`. The proposal names this as a public
API change for embedders.

### D6. Initial state: a separate ADDED conformance requirement

`build_agui_replay_snapshot` starts from
`{"run":null,"a2ui":{"surfaces":{},"actions":{}}}` (`sse.rs:127-130`).

The delta ADDs "The UAR initial state holds A2UI surfaces and actions" to
`ag-ui-chat-conformance` and states its conditions. It does not touch
"Cursor-consistent attach and replay snapshots", which only
`agui-runs-stream-fidelity` modifies. That change's proposal records the
coordination.

### D7. The test trigger is unchanged and outside the guarantee

The test trigger:

- accepts unvalidated payloads, including unhardened ids;
- emits its log entry as a `StatePatch`;
- does not reduce.

Once it is called, the guarantee no longer holds for that run (L3).

### D8. The log is a message and action log

The log holds one entry per published message (including refused and
unchanged ones), plus action entries and trigger entries. It is not the live
state stream.

The task 4.4 doc comments are corrected, and the MODIFIED `runtime-console`
requirement says newer changes come from re-reading the log.

### D9. Verification: C14 extended with a generated comparison against web_core

All steps run on C14's run R, after its existing assertions. They are
separate Gherkin steps, each under the 125 s default; the continuation step
declares 180 s.

Ground rules for every step:

- Requests use the existing helper. No credentials are sent.
- A new SSE reader stops at a named frame type, or at a numeric SSE `id:`
  target. After 60 s it fails and names what was missing.
- Cursors are numeric SSE ids only.
- All A2UI messages carry `version` `v0.9.1` and profile `uar.a2ui/1`.
- Every `createSurface` uses `catalogId` `urn:uar:a2ui:catalog:1`.

The steps:

1. **Baseline.** Read `GET /api/uar/runs/R/stream?stream_mode=agui_spec` with
   no cursor until `STATE_SNAPSHOT`.
   - Record its id C0 and snapshot S0.
   - Assert that R's surface holds `components` `{root}` and `dataModel` `{}`.
2. **Main-surface messages.** Send one POST per message, each expecting 202.
   Record every `artifactId`.
   1. `updateComponents` with `root` (a `Column` with children
      `greeting_copy` and `confirm`), `greeting_copy` (`Text`), `confirm` (a
      `Button` with child `confirm_label` and action event `confirm`), and
      `confirm_label` (`Text`).
   2. `updateComponents` with a new `root` (`Column`: `greeting_copy`,
      `confirm`, `note`) and `note` (`Text`).
   3. `updateComponents` with `root` exactly as in 2.2, plus a `Text` with id
      `__proto__`. Expect `ForbiddenId`: no patch, components unchanged.
   4. `updateDataModel` messages: `/a`=`"x"`, then `/b/c`=`1`, then
      `/items/0`=`"first"`, then `/b/c`=`null`.
3. **Generated data-model corpus.**
   - **Seed.** Take it from `UAR_C14_SEED`, or draw one at random, and record
     it in the evidence. Use mulberry32.
   - **Surfaces.** Create G0, G1 and G2 in one body.
   - **Corpus size.** Send 180 `updateDataModel` messages, 60 per surface, in
     9 bodies of 20 messages.
   - **Fixed edge cases.** G0's first 18 messages are fixed. Each `null`
     below is a JSON `null` value:
     - 1: `/` = `{seed:1}`
     - 2: `/list/2` = `"c"`
     - 3: `/list/0` = `"a"`
     - 4: `/list/3` = `"d"`
     - 5: `/list/6` = `"g"`
     - 6: `/list/1` = `null`
     - 7: `/list/0` = `null`
     - 8: `/obj/k` = `1`
     - 9: `/obj/k/z` = `2`
     - 10: `/__proto__/x` = `1`
     - 11: `/n/0/m` = `true`
     - 12: `/list/x` = `null`
     - 13: `/list/x` = `1`
     - 14: `/rows/3/x` = `1`
     - 15: `/rows/1/x` = `null`
     - 16: `/p/q/20000` = `1`
     - 17: `/p2/20000` = `1`
     - 18: `//` = `"u"`
   - **Generated paths.**
     - Depth 0 to 4.
     - Segments drawn from `a`, `b`, `c`, `undefined`, `0`, `1`, `2`, `3`,
       `007`, `10000`, `10001`, `20000`, `__proto__`, `constructor`, `~0x`
       and `~1y`.
     - Forms: a leading `/`, no leading `/` (10 %), a trailing `/` (10 %),
       and the special paths `""`, `/` and `//`.
   - **Generated values.** `null` (25 %), integers, `1.5`, short strings
     without `<`, `>` or `:`, booleans, `{}`, `[]`, `{k:1}` and `[1,null,{z:2}]`.
   - **Event budget.** R's events stay under the 512-event ring: about 60
     existing, about 20 from step 2, at most 193 from the corpus (3 creates and 1 artifact for G0-G2; 180 patches and 9 artifacts for the bodies), about 15
     afterwards. The step asserts that C2 minus R's first retained id is
     below 512.
4. **Second surface, deletion and an absent surface.**
   - One body with `createSurface` S2 and `updateComponents` S2 (`root`
     `Text`).
   - Then `deleteSurface` S2.
   - Then `updateComponents` for `absent-<uuid>`. Each expects 202.
5. **Action and continuation.**
   - `POST /api/uar/runs/R/a2ui/actions` with
     `{surfaceId, name:"confirm", sourceComponentId:"confirm", context:{}}`,
     expecting 202 and a continuation id K.
   - Read K's stream with `last_event_id=0` until `RUN_FINISHED`. `RUN_ERROR`
     fails the step.
   - Then `POST /api/uar/runs/K/a2ui/messages` with `updateComponents` for R's
     surface, expecting 202.
   - Read K's stream with no cursor until `STATE_SNAPSHOT`. Assert that R's
     surface is absent from K's snapshot (L8). Assert that K's
     `surface-replay` holds the update.
6. **Frames.**
   - Read R's stream with no cursor until `STATE_SNAPSHOT`, and record C2 and
     S2.
   - Read R's stream with header `Last-Event-ID: C0`:
     - assert the first frame is `STATE_SNAPSHOT` with id C0, equal to S0;
     - collect every frame until an SSE id of at least C2.
7. **Comparisons.** These run after all frames are collected.
   1. **Reference.** Resolve `@a2ui/web_core/state` with `createRequire` from
      `frontend/packages/a2ui-core/package.json`. Read
      `<resolved>/../../package.json`, and assert its name is `@a2ui/web_core`
      and its version is `0.12.0`. Import the module by file URL.
   2. **Per surface.** Use one `DataModel({})` for each of R's main surface,
      G0, G1 and G2. Apply that surface's data messages in send order, each
      inside its own try/catch. Before and after each message, record the
      JSON form (`JSON.parse(JSON.stringify(data))`). Classify each message:
      - **changed** when the before and after JSON forms differ under D2
        equality;
      - **refused** when `set` threw;
      - **unchanged** otherwise.
   3. **Per body.** Pair each POST's `artifactId` with its
      `uar.artifact.available` frame. The `STATE_DELTA` frames between the
      previous paired artifact frame and this one belong to this body. Assert
      that their count equals the number of changed messages in the body.
      For bodies without data messages, the expected counts are: 2.1 → 1,
      2.2 → 1, 2.3 → 0, the G0-G2 create body → 3, the S2 body → 2, the
      delete → 1, and the absent surface → 0. After the last paired artifact
      frame there is exactly one `STATE_DELTA`, the action.
   4. **Running state.** Start from S0 and apply every collected
      `STATE_DELTA` in order, with RFC 6902 rules. Refuse `replace` or
      `remove` of a missing member.
      - After each corpus body, assert that the surface's `dataModel` equals
        the reference.
      - At the end, assert that the result equals S2.
   5. **Final state.** Assert in S2:
      - five main-surface components and no `__proto__` component;
      - the main `dataModel` equals the reference (`{a:"x", b:{}, items:["first"]}`);
      - G0, G1 and G2 equal their references;
      - there is no S2 surface and no `absent-*` surface;
      - exactly one action, naming `confirm`.
   6. **Ring diagnosis.** If the cursor read did not begin with
      `STATE_SNAPSHOT` at C0, record the first frame's type and fail as L2.
8. **Log.** Read R's `surface-replay`. Assert:
   - one entry per sent message, in order, in the existing encoding;
   - entries are present for refused, unchanged, absent and forbidden-id
     messages;
   - the action entry is present.
9. **Trigger, last.**
   - Send `POST .../a2ui/surface-test-trigger` with
     `{surface_id:"trigger-<uuid>", kind:"createSurface", payload:{catalogId, fact:"t"}}`,
     expecting 200.
   - Assert that the newest log entry and the delta after C2 carry the
     payload unchanged.

The evidence records:

- the seed;
- every message and its reference outcome;
- per-body counts;
- the first mismatch, if any, with its message index and both values.

### D10. The superseded change no longer states patch rules

In `agui-a2ui-projection-and-custom-components`:

- the overlapping requirement is removed;
- its proposal bullet is marked withdrawn;
- the withdrawn tasks are removed, with a note;
- task 1.5 (a snapshot built "by applying the replay") is marked as to be
  rebuilt on changes 1 and 2;
- its proposal and design open with a superseded banner.

### D11. The console skips action entries in the log

`projectA2uiReplay` (`run-trace-projection.ts:343-357`) skips entries whose
path starts with `/a2ui/actions/`, before `replayEnvelope`. They are counted
separately and do not reach the reducer. This makes "A2UI replay succeeds"
reachable for runs with actions.

Other invalid entries still fail the whole projection with their position.
That pre-existing behavior is unchanged (L9).

## Limits (normative in the spec)

- **L1. Concurrent publishers.** The reduction is serialized under the
  backbone lock, but the emit happens after the lock is released. A different
  recording order makes a later patch fail to apply. The run's snapshot then
  stops claiming synchronization for the rest of the run, not just for one
  value. This has not been observed. Q5 asks whether to hold a per-run lock
  across reduce and emit.
- **L2. History ring.** The snapshot replays only retained events (512,
  `manager.rs:46,136-138`). This has not been observed: C14 runs have 35-62
  events, and the corpus keeps R under 512. `agui-runs-stream-fidelity`'s
  eviction base retires it.
- **L3. Test trigger.** A trigger call voids the guarantee for that run. The
  trigger also delivers unhardened ids to the client applier.
- **L4. Size.** Patch size follows D4's exact bound. Per-run reduced state,
  the ring and durable history grow linearly with the bytes published. There
  is no cumulative cap, as is already true of the log.
- **L5. Not visible to chat users.** The live chat stream sends no snapshot.
- **L6. Renderer scope.** The chat renderer reduces each artifact on its own.
- **L7. Admission and catalog mismatch.** Rust admits messages that the UAR
  client validator and the renderer's single-catalog processor refuse. This
  is pre-existing. See Q4.
- **L8. Continuations.** A continuation run starts with no surfaces. Its
  updates to its source run's surfaces are refused at debug level. Change 2
  resolves this.
- **L9. Console replay.** Apart from the action entries D11 skips, the
  console fails a whole replay at its first invalid entry. That includes
  messages the client validator rejects (L7), so C14's run R stays
  unreplayable in the console. This is pre-existing and unchanged.

## Open questions for the operator

- **Q2.** Extend C14, or add a new catalog row? Recommended: extend C14.
- **Q3.** Add a cumulative per-run A2UI byte cap? Recommended: not in this
  change.
- **Q4.** Align Rust admission with the client validator and catalog?
  Recommended: as a follow-up.
- **Q5.** Hold a per-run lock across reduce and emit, which removes L1?
  Recommended: accept L1 for now, because it has not been observed. Revisit
  in change 2, which owns ordering across runs.
- **Q6.** Accept L2 until `agui-runs-stream-fidelity` lands? Recommended: yes.

Q1 is resolved: the console fix is included (D11).

## Revision 1 independent review — 2026-10-27

A fresh-context critic reviewed the four artifacts from source only, with no
build or test commands. Verdict: not ready. The lead confirmed C1, W2 and the
cursor part of W4 against source:

- `protocol.rs:269-290` requires `root`.
- The renderer uses `MessageProcessor` from `@prometheus-ags/a2ui-core/v0_9`
  (`a2ui-artifact-block.tsx:3,272`). Its `DataModel.set` replaces the whole
  model for `"/"` or `""`.
- `routes.rs:887-892` parses `Last-Event-ID` as a bare `u64`.

- **C1.** `updateComponents` must contain `root` (`protocol.rs:269-290`). The
  "only a third component" step and the "sends only `b`" scenario would be
  refused with 400. A partial update must be written as `root` plus a new
  component. D8 and task 5.2 also disagree on the component count.
- **W2.** The reference reducer is wrong. `reduceA2uiMessage` serves only the
  run-trace console. The chat renderer uses upstream `@a2ui/web_core@0.12.0`
  `MessageProcessor`, which behaves differently:
  - `"/"` and `""` replace the whole data model;
  - `null` deletes;
  - arrays are indexed;
  - primitive intermediates throw.

  Q1 rests on a false premise.
- **W3.** The client validator rejects messages that Rust accepts (`v0.9`,
  the basic catalog, paths without `/`), including C14's own messages. "The
  same rules" is undefined for real traffic.
- **W4.** The planned C14 stream reads cannot work as written:
  - they never end on a completed run (`routes.rs:966-985`, 15 s keep-alive);
  - a chat SSE id such as `35:1:3` passed as `Last-Event-ID` silently means
    "no cursor", so the snapshot-plus-delta check passes vacuously;
  - waiting for the continuation needs `last_event_id=0`.
- **W5.** The spec's apply-in-order guarantees omit the Q3 ordering and Q4
  history-ring qualifications. They also omit the test trigger's raw patches
  diverging from the reduced state.
- **W6.** Six of eleven scenarios have no verification step: absent surface,
  delete, run-trace log, trigger unchanged, retention discard, and the reducer
  itself.
- **W7.** Re-sending the whole subtree grows quadratically with the number of
  submissions. It is retained in the 512-event ring, in up to 1,000 retained
  runs, and in durable history, with no cumulative cap. This is an unbounded
  growth path at an owner-scoped REST boundary.
- **W8.** The unarchived backbone change is satisfied in letter only. Its
  documented protocol is "replay, then switch to live"
  (`realtime.rs:109-110`, `routes.rs:1354-1357`). Replay and live would now
  differ in encoding, and the `runtime-console` "Replay and live events
  overlap" requirement assumes they can be merged.
- **W9.** The run-stream snapshot is already governed by
  `ag-ui-chat-conformance` ("Cursor-consistent attach and replay snapshots").
  The baseline change should be a MODIFIED delta there, not a second
  definition.
- **W10.** The log also holds `add /a2ui/actions/<uuid>` entries, and
  run-trace throws on them (`run-trace-projection.ts:39,312-314`). This defect
  already exists, and C14's action step would reproduce it.
- **Suggestions:**
  - give the reducer typed inputs or an error type, rather than letting one
    `None` mean both "absent surface" and "bad shape";
  - state that patch paths use the escaped surface token;
  - block groups 1-5 on tasks 0.1 and 0.2.

Status: stopped before implementation at the operator's instruction.

## Revision 2 independent review — 2026-10-27

A fresh-context critic reviewed the revision 2 artifacts using source only.
Verdict: not ready. The lead confirmed C1, W1's catalog claim and W2 against
the source:

- `SurfaceTestTriggerPayload` has the field `surface_id` and no `rename_all`
  (`routes.rs:190-196`).
- `uarBasicCatalog` is `urn:uar:a2ui:catalog:1`
  (`packages/a2ui-uar/src/catalog/uar-basic-catalog.ts:32,54-58`).
- `openspec/changes/agui-runs-stream-fidelity` also MODIFIES "Cursor-consistent
  attach and replay snapshots". That change has 0 of its 20 tasks done.

Findings:

- **C1. The D9 step-7 trigger body would be refused.** It uses `surfaceId`,
  but the route requires `surface_id`. axum would refuse it with 422, so the
  extended C14 fails at the gate.
- **W1. The spec's "equals one MessageProcessor" requirement is false as UAR
  configures the reference.**
  - The renderer's processor registers only `urn:uar:a2ui:catalog:1`, so it
    refuses C14's basic-catalog `createSurface`. It also runs component and
    topology validators.
  - `processMessages` aborts on the first refusal, so "one processor" is
    undefined.
  - D2 mirrors only the state operations, and the spec must say exactly that.
- **W2. Two changes modify the same requirement.** `agui-runs-stream-fidelity`
  MODIFIES the same requirement with a different body (exact snapshots after
  eviction) and two other scenarios. Whichever change is archived second
  erases the other's text. Its eviction base also contradicts L2 and Q6.
- **W3. The superseded change is still live.**
  `agui-a2ui-projection-and-custom-components` still ADDs an overlapping
  patch requirement, and no task withdraws it.
- **W4. Arrays reintroduce quadratic growth.** Each update resends the whole
  array. Per-index `add`/`replace` would apply correctly in both appliers,
  and the delta table still claims linear growth.
- **W5. The no-op rule contradicts D4.** "Changes nothing, no patch" conflicts
  with D4's unconditional `add P`. Array holes (Rust `null` versus JS
  `undefined`) make no-op detection differ.
- **W6. Component ids are not escaped or guarded.** A component id
  `__proto__` causes the client applier to set the prototype. This already
  exists for surface ids, and the change extends it to component ids.
- **W7. The public trait method would take a crate-private type.** That
  triggers `private_interfaces`, which breaks the zero-warnings rule.
  Visibility is unspecified.
- **W8. The reducer is never executed against web_core.** C14 compares the
  server with itself. Root path and forbidden path could be covered cheaply,
  and a Node step could compare against `web_core` `DataModel` directly.
- **W9. The runtime-console requirement goes stale.** "SHALL reconstruct
  late-join state / merges newer persisted state events" is contradicted by
  the new live encoding, and no delta updates it.
- **W10. Step 2.1 says "five" components but lists four.**
- **W11. The MODIFIED text is unconditional.** "Snapshot SHALL be initial
  state + patches" and "action patch applies" are stated without the
  conditions the design names.
- **W12. The normal action loop refuses its own updates.** Continuations run
  in a new run with empty reduced state, so the usual action loop (C15's
  pattern) updates the source surface, is refused and logs a warning. No limit
  names this.
- **Suggestions:**
  - The ring-eviction diagnosis is wrong: it shows as an unsynchronized
    snapshot plus `uar.presentation.snapshot`, not a gap event.
  - Paths made only of slashes (`"//"`) set the key `"undefined"` in the
    reference.
  - Rule ordering: `null` with a primitive root is a no-op, and a delete of an
    absent surface is not a refusal.
  - The doc-comment targets are incomplete: `realtime.rs:20-37,61-67`,
    `routes.rs:48-51,780-781`.
  - The request helper cannot read SSE. Split the steps under the 125 s step
    timeout.
  - The action returns 202 and the trigger returns 200.

The critic confirmed these as correct:

- every D2 rule against `web_core` 0.12.0;
- the D4 traces through both appliers;
- D9 steps 1-6 against the real routes (numeric cursors, snapshot id equal to
  the cursor, bounded reads needed, `last_event_id=0`, 202s, about 10 deltas);
- no missed reader of `StatePatch` events.

Status: stopped before implementation at the operator's instruction.

## Revision 3 independent review — 2026-10-27

A fresh-context critic reviewed revision 3 and the edits to the two other
changes, using source only. Verdict: not ready.

The lead confirmed C1 against `web_core` `data-model.js`:

- line 95: the no-op test is `get(path) === undefined`;
- lines 129 and 157: a numeric next segment above 10000 throws during
  vivification;
- lines 160 and 163: containers above the throw point have already been
  created.

Findings:

- **C1. D2 differs from `DataModel.set` in four input classes, and d1-d13
  covers none of them:**
  - **(a) Rule 3's absent test is incomplete.** `get` also returns
    `undefined` through a `null` intermediate and through a non-numeric
    segment on an array. So `/list/x=null` is a no-op in the reference, but
    D2 refuses it. A `null` set through an array hole, which UAR stores as
    `null`, is a no-op in the reference, but D2 vivifies `{}` and emits
    `replace`.
  - **(b) Vivification past 10000.** It must throw, but D2 makes `{}`.
  - **(c) Partial mutation.** The reference mutates before it throws: for
    `/p/q/20000=1` it leaves `{p:{}}`. D2 says a refusal leaves the state
    unchanged.
  - **(d) The "one deliberate departure" claim is therefore false.**
- **W1.** Step 7's per-message outcome check needs deltas that only step 8
  collects, and no frame-to-message mapping is defined. One fix: pair each
  POST's `artifactId` with its `uar.artifact.available` frame.
- **W2.** `web_core` does not export `./package.json`, so the version check
  must read the file at the resolved path. Resolving `@a2ui/web_core/state`
  and running `new DataModel({})` standalone are fine.
- **W3.** Gap fill emits one `add` per index. `/list/10000` on a short list
  is about 10k ops (about 1 MB) from a message of about 70 bytes, so "linear"
  is false.
- **W4.** The continuation scenario is never observed. C14's continuation
  likely re-creates rather than updates, and step 5 accepts `RUN_ERROR`.
- **W5.** The id hardening is never executed, although one `__proto__`
  component id would test it. The trigger still sends unhardened ids.
- **W6.** The MODIFIED `runtime-console` requirement is false for any run with
  an action, because the console aborts on action log entries. The Q1 limit
  must be stated, or the console fixed.
- **Suggestions:**
  - L1 is worse with per-index patches: a lost order breaks state for the
    rest of the run. A per-run lock across reduce and emit would fix it.
  - Define number equality (`1` against `1.0`).
  - Step 7 should use one `DataModel` per surface, not one per message.
  - Give `catalogId` and `version` for the DM and S2 creates.
  - Stale lines: `routes.rs:782-783`, the console throw at `:314`, and the
    backbone change has three requirements, not two.
  - Log `SurfaceExists` below warn.
  - Adding a required trait method breaks external implementers
    (`lib.rs:44`).
  - Give the superseded change a superseded banner, and close out its task
    1.5 and its withdrawn checkboxes.

The critic confirmed these as correct:

- the D4 traces and the count of 19 deltas;
- Rust admission of `null` values and the `__proto__` path;
- the trigger and action bodies;
- visibility;
- that no other unarchived change collides with the deltas;
- that existing C14 runs have 35-62 events, so L2 is not a risk at the gate.

**The uncomfortable point.** The only `web_core` comparison runs 13 inputs
chosen from the author's own rules. It can confirm D2 as written, but it
cannot find where D2 differs from `web_core`.

Status: stopped before implementation at the operator's instruction.

## Revision 4 independent review — 2026-10-27

A fresh-context critic reviewed revision 4 from source only. Verdict: not
ready.

The critic confirmed the D2 port against `data-model.js:84-196,234-265`
statement by statement, for the depths the corpus generates. It also
confirmed:

- all 18 fixed cases through both appliers (11 change state, 7 do not, and
  case 16 both changes state and is refused);
- the four operations;
- the event mapping;
- that no `/presentation` projection appears after R's terminal event;
- the shared backbone.

The lead confirmed the following:

- `protocol.rs` sets no limit on path length or depth;
- the body limit is 10 MB (`server.rs:2898`);
- one `Mutex<HashMap>` covers every run (`realtime.rs:121-124`).

Findings:

- **C1. A long path can crash the process.** A path such as `/a/a/a…` builds
  about 50k levels. `Value` clone, drop and serialization recurse, so the
  process aborts with a stack overflow, under a mutex every run shares. Both
  the REST endpoint and model output can reach this. The 0-4 depth corpus
  cannot detect it. A depth cap is a second required departure from the
  reference.
- **C2. The size bounds are false.** The amplification comes from two
  sources:
  - Gap fill materializes `null`s. `/x/10000/10000/…` turns about 600 bytes
    into roughly 1M nulls.
  - A write more than one index past an array's end resends the whole
    enclosing array, so repeated `/A/<L+2>` writes grow quadratically.

  Ring, durable history, every subscriber and the client's double clone all
  amplify this, with no cap.
- **W1. The continuation check depends on the model.** If K re-publishes
  `createSurface` with the same id, the step-5 assertion and the spec scenario
  are false.
- **W2. The 60 s reader cap kills the 180 s continuation step.**
- **W3. The event-budget assertion can never fail.** The ring is contiguous.
  The correct test is "first retained id = 1" (a `last_event_id=0` read with
  no gap).
- **W4. A server-side patch failure surfaces as a missing `STATE_SNAPSHOT`
  timeout in step 6.** The comparisons, and the first-mismatch diagnosis,
  never run.
- **W5. Case 16 both changes state and refuses.** "Changed" and "refused" must
  be independent flags in the harness.
- **W6. The harness value pool aliases objects.** `set` stores values by
  reference, so each sent value must be cloned.
- **W7. The `<resolved>/../../package.json` path is wrong under path-join
  semantics.** Use `dirname(resolved)/../../package.json`.
- **Suggestions:**
  - Holes above index 10000 classify differently, although the state is equal.
  - Use non-overflowing digit comparison.
  - D4's "throws only at or below P" claim is false for `PrimitiveIntermediate`
    (harmless).
  - The partial-mutation scenario needs the precondition that `p` was absent.
  - "Counted separately" in D11 needs a type change and a UI change.
  - The console re-reads only on refresh.
  - The SSE reader must cancel the response body.
  - The name S2 is ambiguous.
  - The superseded spec and the `a2ui-live-update` purpose still frame the
    log as late-join state.
  - No Rust reducer unit tests exist, so every reducer bug costs a full C14
    run to find.

**The uncomfortable point.** Fidelity to `web_core` is itself the defect.
Browser-tab semantics (free holes, deep objects, one viewer) become denial of
service when ported into a shared server mutex, and the D9 harness cannot see
it.

Status: stopped before implementation at the operator's instruction.
