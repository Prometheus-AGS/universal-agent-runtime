# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 6)

- **Stage:** Plan. Written 2026-10-28. Nothing here is implemented, and the
  plan stops before Execute.
- **Supersedes:** revision 5 (`plan-rev5.md`). The independent review of each
  earlier revision is appended to its own file.
- **Operator direction for revision 6:**
  - store the action as an `ArtifactDisplay` with
    `artifact_type: "a2ui_action"`;
  - fix revision 5's warnings 3–8 and its suggestions;
  - list every load-bearing source fact, with a line the lead re-read
    personally;
  - have a fresh critic review the plan, then stop.

## 0. Facts ledger (re-read by the lead on 2026-10-28)

Every claim this plan's safety rests on is listed here, with the line the lead
re-read in this session. A plan statement that relies on a fact not in this
table is a defect.

| # | Fact | Where re-read |
|---|---|---|
| F1 | The domain event enum used by run history, the emitter and `agui.rs` has an `ArtifactDisplay { run_id, artifact: ArtifactPayload }` variant. | `src/uar/domain/events.rs:232-235` |
| F2 | `ArtifactPayload` is `{artifact_id, artifact_type, title, content: String, language: Option<String>, metadata: Value}`. | `events.rs:456-463` |
| F3 | The domain enum has **no** `Custom` variant. Revision 5's premise was false. | `grep -c Custom src/uar/domain/events.rs` → `0` |
| F4 | The durable read drops any row that fails to deserialize into the domain enum. | `src/uar/runtime/manager.rs:9180-9187` (`filter_map … from_value::<NormalizedEvent>(row.event).ok()`) |
| F5 | Every emitted event passes through `project_or_stage`. Admission-time events are *staged* and published only after admission completes. | `manager.rs:524-526`; staged list completed at `:6285-6289`; published at `:6334` (`for event in staged_events { emitter.emit(event).await; }`) |
| F6 | An admission `Error` clears every staged event. | `src/uar/runtime/turn/host/event_projection.rs:64-69` |
| F7 | `ArtifactDisplay` content and metadata are scrubbed. | `event_projection.rs:238-240` (`Artifact \| ArtifactDisplay \| ArtifactInputRequest => self.project_artifact(artifact)`) |
| F8 | The output ceiling in text-only runs treats these artifacts as surfaces: `artifact_type` `"a2ui"` or `"display"`, a `"a2ui/"` prefix, a `language` of `a2ui`, `application/a2ui+json` or `application/vnd.uar.a2ui+json`, or a metadata `profile` starting `uar.a2ui/`. | `src/uar/runtime/a2ui_output.rs:43-58` |
| F9 | The dedicated route's projector drops every event it doesn't match. | `src/uar/api/agui.rs:651` (`_ => Vec::new()`) |
| F10 | The native legacy stream projects any `ArtifactDisplay` as `agui.artifact`, with `artifact_type` and `content`. | `src/uar/api/sse.rs:623-636` |
| F11 | The native `agui_spec` stream projects any `ArtifactDisplay` as `CUSTOM uar.artifact.available`. | `src/uar/api/adapters.rs:208-218` |
| F12 | When `create_agui_run` returns `false`, today's code responds `409 thread_run_conflict`; when it returns `Err`, `503`. | `agui.rs:350` |
| F13 | `history_since` reads only the in-memory ring while a run is active. | `manager.rs:9151-9153` (`if let Some(state) = runs.get(run_id) { … history.lock() …`) |
| F14 | The ring holds 512 events. | `manager.rs:46` (`EVENT_HISTORY_LIMIT: usize = 512`) |
| F15 | A gap at the start of a replay ends the stream with `RUN_ERROR stream_history_unavailable`. | `agui.rs:685-686` |
| F16 | Terminal-block frames: caller-tool frames `(cursor, index+1)`, close at `terminal_index - 10`, terminal at `terminal_index`, with `terminal_index = 100 + 3k`. | `agui.rs:700-714` |
| F17 | The pnpm workspace packages are `.`, `frontend`, `frontend/packages/*` and `frontend/packages/prometheus-entity-management/packages/*`. A nested `scripts/behaviors/clients/*` directory is not a workspace package. | `pnpm-workspace.yaml:1-5` |
| F18 | PEM 4.3.0's headless build imports only `zod`, `zod/v3` and its own chunks. | `prometheus-entity-management/packages/a2ui-react/dist/headless.mjs` and `dist/chunk-PZW47VJJ.mjs` import lists |
| F19 | `@prometheus-ags/a2ui-react@4.3.0` peers are react 19, react-dom 19, `@ag-ui/core` (`>=0.0.59 <0.1.0 \|\| ^1.0.0`) and `@prometheus-ags/entity-graph-core@4.3.0`. All three dependencies, plus `@a2ui/web_core@0.12.0`, are published. | `npm view` on 2026-10-28: `entity-graph-core` 4.3.0, `@ag-ui/core` 1.0.3, `@a2ui/web_core` 0.12.0 |
| F20 | `routes.rs`, `agui_runs.rs`, `providers/agui_runs/surreal.rs`, `a2ui_output.rs` and `providers/run_history/surreal.rs` contain no `#[cfg(test)]` module. `manager.rs` contains nine. | `grep -c '#\[cfg(test)\]'` → 0, 0, 0, 0, 0, 9 |
| F21 | Run-history storage lives in `src/uar/persistence/providers/run_history/{postgres,surreal}.rs`. | directory listing |
| F22 | `CreateRunRequest` is deserialized from the client's `forwardedProps.uar`, and `client_tools` already uses `#[serde(skip)]` there. | `routes.rs:72-78` (cited by the revision 5 critic); `agui.rs:412` (`serde_json::from_value(resources)`) |
| F23 | `parse_message` and `CreateSurface` admit only `surfaceId` and `catalogId`, so UAR surfaces never set `sendDataModel`. | `src/uar/a2ui/protocol.rs:46-50` |
| F24 | web_core 0.12.0 `v0_9` re-exports `../resolution/index.js`. `ButtonApi` lives in `v0_9/basic_catalog`. | `@a2ui/web_core/src/v0_9/index.d.ts:7-20`; `src/v0_9/basic_catalog/components/basic_components.d.ts` |

**Taken from the critics, not re-read by the lead.** The plan cites these but
does not rest its safety on them:

- the `GenericBinder` closure shape (`generic-binder.js:362-376`);
- PEM's limit checks (`limit-checks.ts:54-96`, `:131`, `:150`);
- PEM's deny-all default policy (`runtime-core.ts:124`, `:138-140`).

Execute re-reads them before relying on them.

## 1. The outcome a user sees

The outcome is unchanged from revision 4 §1. An app on UAR's dedicated AG-UI
route, using PEM 4.3.0's headless runtime, can:

1. **See** the agent's surface.
2. **Click** a rendered button and have the agent receive that exact action.
3. **Keep using** the surface after the first answer and after typed
   follow-ups.
4. **Join late** by attaching to the thread's latest run, and see the
   thread's surfaces without the agent running again.

None of this works today. On 2026-10-28, I19 measured a run that reported
success while the client received no surface (F9).

## 2. Operator decisions applied

These are unchanged from revision 4 §2:

- **D1 (AG-UI activities):** adopted.
- **D3 (cumulative snapshots):** cumulative snapshots only.
- **D2 and D4 (derived record):** partly pulled in. The record is derived
  from run history, and nothing new is stored.
- **D5–D17:** deferred.

## 3. *(r6)* Storage: none added — the action is an artifact event of its own run

- **The event.** The action run records its action as
  `NormalizedEvent::ArtifactDisplay { run_id, artifact }` (F1). The `artifact`
  fields are:
  - `artifact_id`: `"a2ui-action:" + actionId`;
  - `artifact_type`: `"a2ui_action"`;
  - `title`: `"A2UI action"`;
  - `content`: **the run's input string, byte for byte**. This is the same
    `String` that becomes `CreateRunRequest.input`; see S0;
  - `language`: `Some("application/json")`;
  - `metadata`: `{"profile": "uar.a2ui-action/1", "actionId", "surfaceId", "name", "sourceComponentId", "contextKeys", "contextDigest", "threadId"}`.
- **It escapes the output ceiling.** None of those values match the
  surface-artifact patterns (F8): `"a2ui_action"` is neither `"a2ui"` nor
  `"a2ui/…"`, `application/json` is not an A2UI language, and
  `uar.a2ui-action/1` does not start with `uar.a2ui/`. So a text-only run
  never turns it into a diagnostic.
- **It is scrubbed and persisted like every artifact.** Scrubbing is F7.
  Persistence follows the normal path: staged, then published after admission
  (F5), then written by the durable history writer.
- **No persisted shape is added.** `AguiRunBinding`, the tables, the `.surql`
  files, the migrations and the persistence trait are all unchanged.
- **Rollback.**
  - **Older binaries keep working.** An older binary deserializes this event
    into its existing `ArtifactDisplay` variant, so the durable read keeps the
    row (F4). Its dedicated-route projector drops it (F9). Run histories stay
    contiguous, so there is no `stream_history_unavailable` gap (F15). That was
    revision 5's critical 2.
  - **What rollback loses:** `actionId` replay and the echo.
- **Admission failure** (revision 5 warning 3).
  - **What happens:** the action event is staged. If admission fails, the
    staged events, this one included, are cleared (F6). The run then ends with
    `ADMISSION_FAILED` and no action record.
  - **Consequence:** a retry of that action finds no record and is admitted
    again.
  - **Why that is correct:** the failed attempt executed nothing. Admission
    failed before the model ran.
- **Event id.** Because of staging (F5), the action event's id is *not*
  guaranteed to be 1. Nothing depends on its position. The projector, the
  lookup and the tests identify it by `artifact_type`.
- **Visibility to other clients** *(stated, not hidden)*.
  - **Native-route observers see it.** Anyone watching the action run on the
    native stream, such as the console, sees this artifact (F10, F11):
    - as `agui.artifact` in legacy mode;
    - as `CUSTOM uar.artifact.available` in `agui_spec` mode.
  - **What that exposes:** its content is the run's own input, which those
    owner-scoped observers already see as the user turn.
  - **Console rendering:** how the console displays `a2ui_action` is
    unverified. Execute checks it, and if needed adds the type to the
    console's list of artifacts it doesn't render. That is a UI change and
    goes through the repository's UI routing rules.

## 4. Changes

There is one OpenSpec change (§7) and one Rust writer.

| File | Change |
|---|---|
| `src/uar/api/agui_a2ui.rs` (new) | the fold, the limits, the action logic |
| `src/uar/api/agui.rs` | call sites |
| `src/uar/runtime/manager.rs` | `published_a2ui_events`, plus emitting the action artifact |
| `src/uar/api/routes.rs`, `src/uar/runtime/turn/request.rs` | one server-set field |

### S0. *(r6)* The validated action travels with the run's input

- **The field.** `a2ui_action: Option<A2uiActionRecord>` is added in two places:
  - `RunExecutionRequest` (`turn/request.rs`);
  - `CreateRunRequest`, with `#[serde(skip)]` (F22). This follows the
    `client_tools` precedent.
- **Who sets it.** Only `agui.rs`, after the S3 checks. `admit_run` copies it
  across.
- **What it holds.** `A2uiActionRecord` holds the already-serialized input
  `String` and the metadata fields.
- **Where it is emitted.** The manager emits the artifact (§3) right after it
  constructs `RunEventEmitter` (`manager.rs:3975-3986`).
- **One string, two uses.** `agui.rs` serializes the input once. That same
  `String` becomes both `CreateRunRequest.input` and the artifact's `content`.
  A unit test, `action_artifact_content_equals_run_input`, asserts the two are
  byte-identical (revision 5 suggestion 11).
- **Why this proves the input.** The echo is projected from that content, so
  it shows what the model actually received.

### S1. *(r6)* The thread surface record

The record is unchanged from revision 5 S1:

- **Per-binding cursor ranges.**
- **Restatement** from closed bindings with an ordinal below the stream's own
  binding.
- **Durable plus ring merge** in `published_a2ui_events`.
- **Completeness tracking.**
- **Reachability and PEM-style depth.**
- **Stripping** of the `profile` field.

Revision 6 adds:

- **The fold reads surfaces only from `artifact_type == "a2ui"`.** An
  `"a2ui_action"` artifact is never a surface.
- ***(r6)* The `actionId` lookup has its own scope** (revision 5 warning 5).
  - **What it scans:** the action artifacts of every binding with ordinal ≤ the
    thread's *latest* ordinal, inside the same 64-binding window. That
    includes the latest binding even while it is open.
  - **What it reads:** each binding's merged events, the same read as
    `published_a2ui_events`, filtered to `artifact_type == "a2ui_action"`.
  - **Order:** the lookup runs *before* the latest-run rule. A retry while the
    winning run is still open therefore replays it, once the winner's action
    artifact has been published, which happens after admission completes (F5).
  - **What it can miss** (§10):
    - an action whose binding has fallen out of the 64-binding window;
    - an action in a `history_incomplete` binding.

    In both cases the declaration check still applies: a surface outside the
    window, or one from incomplete history, is refused with `422`, so neither
    miss leads to a duplicate admission.

#### S1a. *(r6)* PEM limits, as one consistent rule

The bounds are revision 4's S1a table plus revision 5's live caps. Revision 5
warning 6 found two competing rules; they are replaced by one:

- **The rule.** Over a bound, a surface is marked `over_limit` *at the moment
  a publication would put it, or the thread, over that bound*. The surface
  stays `over_limit` until a `deleteSurface` removes it.
- **First come, first served.** Restatement emits every live surface that is
  not `over_limit`. Those can number at most 48, because the 49th was already
  refused at publication.
- **No "newest 48" selection.** That revision 4 S1a sentence is withdrawn, so
  a surface refused live is never restated later, and a surface the client
  holds is never dropped by restatement.
- **Data model.** The incoming `dataModel` cap is 1 MiB. PEM sends the data
  model of every `sendDataModel` surface. UAR surfaces never set
  `sendDataModel` (F23), so this field carries other producers' data only.

### S2. *(r6)* Outbound

- **`messageId`:** unchanged.
- **Reserved range `(start_cursor, 1..=49)`:** unchanged.
  - Indices 1–48 carry restatement snapshots.
  - Index 49 carries `restatement_unavailable`.
  - The range is emitted for every binding.
- **The echo.** The projector maps the action artifact, wherever it lands, to
  a source-event frame `(event.id, 1)`:
  `CUSTOM uar.a2ui.action.accepted` with `{actionId, surfaceId, name, sourceComponentId, contextKeys, contextDigest}`.
  The values are parsed from the artifact's metadata.
  - **No reserved index.** Its id follows the event (F5).
- ***(r6)* The edge-binding shift, narrowed** (revision 5 suggestion 9).
  - **Which bindings:** those that seal with
    `terminal.source_cursor == row.start_cursor`.
  - **What moves:** only their *caller-tool* frames, from `(cursor, 1..=3k)`
    to `(cursor, 51..=50+3k)`.
  - **What stays:** the close frame `90+3k` and the terminal frame `100+3k`
    keep today's indices (F16). Since `50+3k < 90+3k` for every `k`, the
    caller-tool frames still sit below the close frame.
  - **Who is affected:** caller-tool frames for cold denials, interrupt seals
    and empty-history recoveries at `start_cursor`. The terminal and close
    frames of those bindings don't change.
  - **Ordering:** the existing error frame `(cursor, 99)` in that block is
    emitted on a branch that `break`s before any caller-tool frames
    (`agui.rs:704`), so the two never share a stream.
- **Updates during the run, order relative to the tool result, and `profile`
  stripping:** unchanged.

### S3. *(r6)* Inbound

- **Parse, bounds and `resume`:** unchanged from revision 5 S3. The `resume`
  refusal stays.
- **Check order.**
  1. **Same `runId`:** replay.
  2. **Look up `actionId` (S1).** If an action artifact with the same
     `actionId` exists, and its digest and workspace match, stream its binding
     (replay). If it exists but doesn't match, refuse with
     `409 a2ui_action_id_conflict`.
  3. **The latest-run rule.**
  4. **The declaration check.**
  5. **Reserve:** `create_agui_run(row, previous)`, unchanged.
  6. ***(r6)* A twin that loses the race** (revision 5 warning 4).
     - **The usual case:** when `create` returns `false`, step 2 runs once
       more. The winner's action artifact is published only after admission
       completes (F5), so the re-check *usually* finds nothing. The loser then
       gets today's `409 thread_run_conflict` (F12), and a later retry replays.
     - **A store conflict:** a `TransactionConflict` still returns `503` (F12),
       which a client may retry.
     - **Nothing is remapped**, and §10 states this.
- **Building the run:** unchanged.
  - **Identity stripping** uses the shared helper.
  - **History** is kept in full. With client tools present, the action is
    appended as a new user message.
  - **An action plus new text** is refused with `400`.
  - **The input** is serialized once (S0).
- **Authority:** unchanged from revision 5. These fields are client-supplied
  exactly as on every run of this route: `model`, `response_format`,
  `service_placement`, `mcp_servers`, `tool_admission`, `run_credentials`,
  `working_directory` and `reasoning_effort`.
- **Activity messages:** dropped before both history validations
  (`agui.rs:75-78`, `:397`).

### S4. *(r6)* Acceptance: I19

The Gherkin is unchanged from revision 4 S4.

- ***(r6)* An isolated client install** (revision 5 warning 8).
  - **Location:** the test client's dependencies live in
    `scripts/behaviors/clients/a2ui-pem/`. That directory has its own
    `package.json`, which pins `@prometheus-ags/a2ui-react` `4.3.0` and
    `@a2ui/web_core` `0.12.0` exactly, and a committed `package-lock.json`.
  - **Why it's isolated:** the directory is not a pnpm workspace package
    (F17). So pnpm's workspace linking, overrides and
    `minimumReleaseAgeExclude` don't apply to it. The two same-name workspace
    copies of `@prometheus-ags/a2ui-react` can't shadow the published one, and
    the repository's root `node_modules` is untouched.
  - **Install:** Execute task 6 runs
    `npm ci --prefix scripts/behaviors/clients/a2ui-pem` once the lockfile has
    been generated.
  - **Peer dependencies:** npm installs them: react and react-dom 19,
    `@ag-ui/core`, and `@prometheus-ags/entity-graph-core@4.3.0`. All are
    published (F19). The headless build imports only `zod` at runtime (F18),
    so the extra packages are inert.
  - **Imports:** the steps import the client by path, through
    `createRequire(resolve('scripts/behaviors/clients/a2ui-pem/package.json'))`.
- **Runtime setup:** unchanged from revision 5.
  - **Catalogs:** two headless catalogs, under UAR's two catalog ids.
  - **Action policy:** `actionPolicy` with `status: 'executed'`.
  - **Envelope capture:** a promise on `onActionEnvelope`.
- **The press:** unchanged from revision 5.
  - **How:** `GenericBinder(new ComponentContext(surface, buttonId), ButtonApi.schema)`
    from `@a2ui/web_core/v0_9` and `/v0_9/basic_catalog` (F24). The test sets
    the data model, invokes `binder.snapshot.action()`, and awaits the
    envelope.
  - **The cross-bundle check:** it stays an Execute item. If it fails, the
    step fails and a PEM issue is filed. There is no fallback to
    `dispatchAction`.
- **Observables:**
  - **What the echo carries:** the envelope's `actionId`, plus a
    `contextDigest` the test recomputes from the context the binder resolved.
  - **What that proves:** because the echo is projected from the artifact
    whose content *is* the run input (S0), it shows what the run received.
  - **Retry, late attach and the forged action:** unchanged.
- **Late attach: the scope, stated** (revision 5 warning 7).
  - **The rule:** a late attach replays the latest binding's own events
    through `history_since`. For a still-active run, that reads only the
    512-event ring (F13, F14).
  - **When the ring has already dropped the start:** the stream emits the
    reserved restatement of *earlier* bindings' surfaces first. Then it hits
    the existing gap check and ends with `stream_history_unavailable` (F15).
    The latest run's own surfaces are then not re-delivered.
  - **Why it isn't fixed here:** this is the route's existing behaviour for any
    long active run. I19's runs are short. Fixing it means the replay reading
    merged durable history, and that is outside this slice (§10).
- **Bounds:** five model turns, one model, no credentials.

## 5. *(r6)* Tests (authored with the code; run only at the gate)

New `#[cfg(test)]` modules are created where none exist (F20).

| Case | Test | File |
|---|---|---|
| The binding JSON is byte-identical to a fixture captured before the change | `agui_run_binding_serialization_unchanged` | `src/uar/persistence/agui_runs.rs` (new tests module) |
| An action artifact round-trips through durable run history unchanged | `a2ui_action_artifact_round_trips` | `src/uar/persistence/providers/run_history/surreal.rs` (new tests module; tempfile `surrealkv://` provider, as in `a2a/handler.rs:319-326`) (F21) |
| An action artifact deserializes as a plain `ArtifactDisplay` and is skipped by the surface fold | `a2ui_action_is_artifact_display_not_surface` | `agui_a2ui.rs` |
| The action artifact's content equals the run input byte for byte | `action_artifact_content_equals_run_input` | `manager.rs` tests (the emitter and staging live there) |
| An admission failure clears the action artifact, and a retry is admitted afresh | `admission_failure_clears_action_artifact` | `manager.rs` tests |
| `a2ui_action` cannot be set from `forwardedProps.uar` | `forwarded_uar_cannot_set_a2ui_action` | `src/uar/api/routes.rs` (new tests module) |
| The output ceiling leaves `a2ui_action` untouched in a text-only run | `output_ceiling_ignores_a2ui_action` | `src/uar/runtime/a2ui_output.rs` (new tests module) |
| The lookup scans open latest bindings and runs before the latest-run rule | `action_lookup_includes_open_latest_binding` | `agui_a2ui.rs` |
| Only caller-tool frames shift for edge bindings | `edge_binding_shifts_only_caller_frames` | `agui_a2ui.rs` |
| One rule for the surface cap | `surface_cap_first_come_first_served` | `agui_a2ui.rs` |

Revision 5 §5b's remaining unit tests are kept:

- restatement covers earlier closed bindings only;
- live thread caps;
- PEM depth;
- an action together with `resume` is refused;
- activity messages are dropped in both validators;
- ranges, window, limits, action checks and `profile` stripping.

Postgres: no Postgres code changes, so there is nothing for the gate to
compile there.

## 6. Client contract

This is unchanged from revision 5 §6, plus F18, F19 and F24.

## 7. *(r6)* Spec delta (OpenSpec `agui-a2ui-activity-slice`)

These are revision 5's five requirements. Requirements 3 and 5 are restated:

- **3. Declared actions become same-thread runs, recorded as an
  `a2ui_action` artifact of that run.**
  - The artifact's content equals the run input.
  - The echo is projected from that artifact.
  - On admission failure, no artifact is recorded and a retry is admitted.
  - A retry with the same `actionId` replays. That includes an open latest
    run whose artifact has been published.
  - A losing twin receives `409 thread_run_conflict`.
- **5. No persisted shape changes.**
  - The action is an existing `ArtifactDisplay` event.
  - An older binary reads it as an ordinary artifact.
  - Run histories stay contiguous across a rollback.

## 8. *(r6)* Verification (exact commands; not run; the gate is closed)

The steps run serially, after the recorded implementation-complete inventory
and the operator's gate instruction:

0. Validate the OpenSpec change: `openspec validate agui-a2ui-activity-slice`.
1. Run the unit tests:
   `cargo test --locked --no-default-features --features server-full --lib -- agui_a2ui:: agui_runs:: run_history:: published_a2ui_events_ action_artifact_ admission_failure_clears_action_artifact forwarded_uar_cannot_set_a2ui_action output_ceiling_ignores_a2ui_action`.
2. Rebuild the owned binary in place. **This step is still an open
   precondition:** the exact command that built it must be confirmed verbatim
   first. The candidate is
   `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
3. Restart with `restartOwnedUar` and the new digest. The command is unchanged
   from revision 5 §8.
4. Set the environment from `manual-runtime.json`, unchanged from revision 5 §8.
5. Run I19 once.
6. Run the five regressions once each: `selection-stream-replay`,
   `live-roster-routing`, `native-tool-approval`, `agui` and `writing`.

C14 and C15 are not rerun.

## 9. Deferred decisions

Unchanged from revision 4 §9.

## 10. The uncomfortable parts

- **An out-of-the-box PEM client renders nothing from UAR.**
  - **The mismatch:** PEM's default catalog id is
    `urn:prometheus-ags:a2ui:catalog:v3`, while UAR publishes
    `urn:uar:a2ui:catalog:1` or the basic v0.9 URL.
  - **Where it gets fixed:** D11/D15. Until then, the proposal must document
    the client setting.
- **The press may fail across PEM's bundled web_core.** If the separately
  installed binder doesn't work on PEM's surfaces, the press steps fail until
  PEM exports a headless press helper.
- **A losing twin usually gets `409 thread_run_conflict`.** The winner's action
  artifact is published only after admission, so the twin's re-check usually
  finds nothing. A later retry replays. A store conflict still returns `503`.
- **A late attach to a long, still-active run can fail.** If the ring has
  already dropped that run's start, the client gets earlier surfaces and then
  `stream_history_unavailable`, not that run's own surfaces. That is the
  route's existing behaviour for long active runs.
- **Native-route observers see the action artifact**, and the console's
  rendering of it is unverified.
- **Caller-tool frames are renumbered for edge bindings.** For cold denials,
  interrupt seals and empty-history recoveries, a client holding a pre-upgrade
  `Last-Event-ID` inside the old range may see those frames once more.
- **A long-lived surface eventually goes over limit.** It passes the
  375-operation cumulative bound and stops being restated or actionable. Deltas
  (D3) and compaction (D7) are the real fix.
- **Some actions can still be missed.** The lookup can miss actions outside
  the 64-binding window or in incomplete history. Their surfaces are then
  refused too, so this never leads to a duplicate admission. The user sees the
  click refused instead.
- **Each run and each click re-folds up to 64 bindings' durable history.**
- **Existing behaviours this slice doesn't change:**
  - the `reconcile` read failure handled with `unwrap_or_default()`
    (`agui.rs:514`);
  - tombstones are unbounded;
  - a replay reports the original `runId`;
  - the REST action path's looser checks.
- **The rebuild command is still unknown.**

## 11. Tasks (for Execute; none started)

1. Create the OpenSpec change (§7).
2. Capture the binding fixture.
3. Add the server-set `a2ui_action` field, the artifact emission and
   `published_a2ui_events`, each with its tests.
4. Write `agui_a2ui.rs`:
   - the fold, which reads only `"a2ui"` artifacts as surfaces;
   - the lookup with its own scope;
   - the single cap rule;
   - the reserved range;
   - the narrowed edge shift;
   - the action handling;
   - the tests, authored but not run.
5. Wire it into `agui.rs`.
6. Create `scripts/behaviors/clients/a2ui-pem/` (`package.json`, generated
   `package-lock.json`, `npm ci`) and write I19.
7. Check the console's rendering of `a2ui_action` artifacts. Any UI change goes
   through the repository's UI routing.
8. Confirm the rebuild command.
9. Record the inventory, then wait for the gate instruction.

## Revision 6 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 6. Its first task was to
check the facts ledger line by line. It ran only `npm view`.

**Verdict: defects found.** The review has 4 ledger errors, 1 critical defect,
4 warnings and 2 suggestions. The lead confirmed the starred items in source
(✓).

### Ledger check

20 of the 24 entries match. Four do not:

- **✓ F7 is wrong.** `project_artifact` scrubs only `title`, `content` and
  `metadata.warnings`. The rest of the metadata stays exact by design
  (`event_projection.rs:261-268`). The lead had read only the match arm at
  `:238-240`, not the function body.
- **✓ F19 is wrong.** `@ag-ui/core` is an optional peer
  (`peerDependenciesMeta: {"@ag-ui/core": {"optional": true}}`), so npm does
  not install it.
- **F18 is unproven.** It was read from the local PEM `dist/`, but the client
  installs the registry tarball (`dist.fileCount` 18). Nothing shows that the
  two are identical.
- **✓ F22 is mis-cited.** `from_value(resources)` is at `agui.rs:417`, not
  `:412`.

### Critical

1. **The claim that "neither miss leads to a duplicate admission" is false.**
   The lookup misses on the *action's* binding, but the declaration check reads
   the *surface's* history, and those can be different bindings:
   - **Incomplete history:** the action artifact sits in a `history_incomplete`
     binding while its surface was published in a complete one.
   - **The 64-binding window:** the action's binding has left the window, but a
     later run updated the same `surfaceId`, so the surface is still inside it.

   In both cases the same `actionId` is admitted twice. Nothing ties the two
   scopes together.

### Warnings

2. **Client-supplied metadata is published unscrubbed.** `name`,
   `sourceComponentId`, `surfaceId`, `contextKeys` and `threadId` reach
   persistence and native observers without scrubbing (this follows from F7).
3. **✓ One emit point doesn't cover every path.**
   - **The orchestrator agent takes another path.** `orchestrator-agent` takes
     the agent-graph path (`manager.rs:2372-2376`, `:3316-3330`), which skips
     the emitter at `:3975`, so it produces no record, no echo and no replay.
   - **Other entry points.** Hosted root turns (`:3632`) and captured threads
     (`:3793`) also enter `execute_request_inner`. If a cloned request carries
     the action into them, it may be emitted once per turn.
4. **The console's rendering of `a2ui_action` is decided by source, not
   unverified.**
   - **How it renders today:** the legacy `agui.artifact` becomes a
     `__a2ui_display__` tool call (`chat-stream-store.ts:1139-1153`).
     `a2ui_action` with `application/json` is then classified as a non-A2UI
     JSON artifact (`chat-message-store.ts:367-382`).
   - **What the user sees:** every action run shows an "A2UI action" JSON
     block containing the full run input.
   - **The planned remedy doesn't exist:** there is no skip list in
     `frontend/src` to add the type to.
5. **Admission failure is described imprecisely.**
   - **The error code:** the client never sees `ADMISSION_FAILED`. The `Error`
     keeps its original code, and only its message is replaced.
   - **What has already happened:** before admission, the run record,
     `session_current_run`, MCP admission and RAG work have all run, though no
     model or tool has.
   - **How the staged events are cleared:** the model-binding path clears them
     through `begin_reduced_failure` (`:6226`).

### Suggestions

- **What the echo proves with caller-tool continuations.** With a caller-tool
  continuation, `prompt` is empty and the turn goes into the continuation
  history (`agui.rs:393`, `:418-421`), so "the echo shows what the model
  received" doesn't hold. Client context is also appended to `prompt`
  (`:405-408`).
- **Two tests are in the wrong place.** `admission_failure_clears_action_artifact`
  ("a retry is admitted afresh") is route behaviour and can't be tested from
  `manager.rs`. `action_artifact_content_equals_run_input` can't prove that
  `agui.rs` used one string. The simpler design is to emit `content` from the
  request's own `input` inside the manager, keeping no second copy.

### Confirmed sound

- **Older binaries and contiguity:** `ArtifactPayload` has no
  `deny_unknown_fields`.
- **The output ceiling:** every branch lets `a2ui_action` through.
- **Staging at `:3975-3986`:** the event is staged until `:6288` and published
  at `:6334`.
- **`presentations.rs`:** it ignores the new type (`:82-90` requires `"a2ui"`).
- **Other paths:** A2A, ACP and chat completions have no `ArtifactDisplay`
  consumer.
- **The losing twin:** the `agui_contexts` compare-and-swap serializes it.
- **The edge shift:** no collision, and the `99` branch does break first.
- **The npm prefix:**
  - `.` matches only the root package, and `.gitignore` covers nested
    `node_modules`.
  - Node 24.18 supports `require(esm)`.
  - There is precedent in `website/` and `sdks/typescript/`.
- **The cargo filters:** they match, including through the `#[path]` module
  mapping (`providers/surreal.rs:10-11`).

### Lead assessment

The ledger did its job: it exposed four bad facts directly, including two from
the lead's own reading. The duplicate-admission hole is a real design defect,
and it has an obvious fix: tie the two scopes together.

- **The rule:** refuse any action whose `actionId` lookup is uncertain. That
  means any scanned binding is `history_incomplete`, or the thread has more
  bindings than the window. In those cases the request is refused with
  `422 a2ui_action_history_uncertain` instead of being admitted. A
  replay-or-refuse rule can never duplicate.
- **The other findings:** the scrubbing, orchestrator-path, console and
  admission findings are each bounded, but each needs either a decision or
  code:
  - scrub the action metadata, or keep it out of metadata;
  - decide whether the orchestrator path emits too, or is refused for actions;
  - change the console so it hides `a2ui_action` artifacts, a UI change that
    goes through UI routing;
  - restate admission failure precisely.

This is the sixth revision. Since revision 2, the design has been stable:
thread-scoped surfaces derived from run history, with the action recorded in
its own run. Each revision since has surfaced narrower defects, mostly in the
facts the plan relies on. The operator may reasonably decide whether a seventh
revision, or Execute with these findings carried as tasks, is the better next
step.
