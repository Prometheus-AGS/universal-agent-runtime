# Plan — minimal A2UI slice on the dedicated AG-UI route (revision 7)

- **Stage:** Plan, written 2026-10-28. Nothing here is implemented, and the
  plan stops before Execute.
- **Supersedes:** revision 6 (`plan-rev6.md`). The independent review of each
  earlier revision is appended to its own file.
- **Operator direction for revision 7:**
  - replay-or-refuse for uncertain `actionId` lookups;
  - keep action fields out of unscrubbed metadata;
  - decide the orchestrator path;
  - plan the console hide through UI routing;
  - fix the ledger entries and the admission wording.

  Then a fresh critic reviews it, and work stops.
- **How this revision is written:** it changes revision 6 in the places named
  below. Every section not mentioned stands as written in `plan-rev6.md`,
  including §1 Outcome, §2 Decisions, the S1 fold, S1a limits, S2 reserved
  range and edge shift, the S4 Gherkin and press, and §9.

## 0. Facts ledger (re-read by the lead on 2026-10-28)

The entries carry over from revision 6 §0, with these corrections and
additions. Anything the plan's safety rests on must appear here.

**Corrected entries:**

| # | Fact | Where re-read |
|---|---|---|
| F7 *(corrected)* | `project_artifact` scrubs **only** `title`, `content` and `metadata.warnings`. Every other metadata field, and `artifact_id` and `artifact_type`, stay exact by design. | `src/uar/runtime/turn/host/event_projection.rs:261-268` (the function body this time, not only the match arm at `:238-240`) |
| F18 *(scoped)* | The **local** PEM build's headless entry imports only `zod`, `zod/v3` and its own chunks. Nothing shows the registry tarball is identical. Execute re-checks the *installed* `node_modules/@prometheus-ags/a2ui-react/dist/headless.mjs` (task 6). | local `dist/headless.mjs` and `dist/chunk-PZW47VJJ.mjs` |
| F19 *(corrected)* | 4.3.0's peers are react 19, react-dom 19, `@ag-ui/core` (**optional**: `peerDependenciesMeta: {"@ag-ui/core": {"optional": true}}`) and `@prometheus-ags/entity-graph-core@4.3.0`. npm installs the required peers, but not the optional `@ag-ui/core`. The steps never import `@ag-ui/core`. | `npm view @prometheus-ags/a2ui-react@4.3.0 peerDependencies peerDependenciesMeta` |
| F22 *(corrected)* | `CreateRunRequest` is deserialized from the client's `forwardedProps.uar`: `serde_json::from_value(resources)`. | `src/uar/api/agui.rs:417` |

**New entries:**

| # | Fact | Where re-read |
|---|---|---|
| F25 | `orchestrator-agent` with a configured agent graph takes the graph path. That path does not enter `execute_request_inner` directly. | `manager.rs:2372-2376` (`uses_agent_graph`); `:3314-3332` (`graph_roots.start(…)`) |
| F26 | A graph root's turns *do* enter `execute_request_inner`, through `start_hosted_root_turn`. | `manager.rs:3619-3636` |
| F27 | Child threads enter `execute_request_inner` through `execute_captured_thread`. | `manager.rs:3784-3796` |
| F28 | A model-binding failure clears staged events through `begin_reduced_failure`, which empties `pending`. | `manager.rs:6222-6229`; `event_projection.rs:34-40` |
| F29 | Before admission completes, the run record is written to durable history and `session_current_run` is set. | `manager.rs:4759-4761`; `:4800-4804` |
| F30 | An admission `Error` keeps its original `code`. Only its `message` is replaced, and `ADMISSION_FAILED` is an internal refusal marker. | `event_projection.rs:64-69` (`self.refused.get_or_insert("ADMISSION_FAILED"); *message = "Run admission failed before protected content publication"`) |
| F31 | The console's legacy path maps every `agui.artifact` to a `__a2ui_display__` tool call. | `frontend/src/stores/chat-stream-store.ts:1139-1155` |
| F32 | The console's message store renders a non-A2UI `__a2ui_display__` as a visible artifact block (`application/json` or `text/plain`), using the artifact's `content`. | `frontend/src/stores/chat-message-store.ts:367-382` |
| F33 | The console's `agui_spec` path treats `uar.artifact.available` as `a2ui-display` only if the top-level `profile` starts with `a2ui`. Otherwise it becomes a generic visible `artifact` chunk. The adapter also maps it to legacy `agui.artifact`. | `frontend/src/platform/agui/runtime-chunk-projection.ts:168-174`; `frontend/src/platform/agui/agui-adapter.ts:98-99` |

**Taken from critics and not re-read by the lead:** the `GenericBinder` closure
shape, PEM's limit-check internals, and PEM's deny-all default. Execute
re-reads them before relying on them.

## 3. *(r7)* The action artifact: data in scrubbed `content`, nothing client-supplied in metadata

The revision 6 §3 metadata carried client-supplied fields (`name`,
`sourceComponentId`, `surfaceId`, `contextKeys` and `threadId`). Those fields
are not scrubbed (F7). Revision 7 moves them.

- **`content`** is the run's input string (§4 S0). It is serialized once from
  `{"type": "a2ui.user_action", "sourceRunId", "interaction": {surfaceId, name, sourceComponentId, timestamp, context, catalogId, userMessage, actionId, a2uiClientDataModel, clientContext}}`.
  - `content` *is* scrubbed (F7). Every client value, including the
    `actionId`, lives only here.
- **`metadata`** is exactly `{"profile": "uar.a2ui-action/1"}`. That is a
  host-chosen constant: no client value and nothing per action.
- **Other fields:**
  - `artifact_id`: `"a2ui-action:" + hex(sha256(actionId))[..32]`. It is
    host-derived, so a raw client string never appears in an unscrubbed field.
  - `artifact_type`: `"a2ui_action"`.
  - `title`: `"A2UI action"`.
  - `language`: `Some("application/json")`.
- **Reading it back.** The echo, the `actionId` lookup and the digest
  comparison all parse the persisted, *scrubbed* `content`.
  - **When scrubbing changes nothing:** they see the original values.
  - **When scrubbing redacts something:** if a client field happens to contain
    a value the scrubber redacts, the parsed field differs from the request.
    Two consequences follow:
    - a retry of that action gets `409 a2ui_action_id_conflict` instead of a
      replay, which fails closed;
    - the echo shows the redacted value.

    This is stated in §10. It never admits twice.
- **It escapes the output ceiling.** The ceiling only fires on recognised
  artifact types, languages or profiles; this artifact's type, language and
  profile are none of those (F8). The same holds in revision 6.
- **Rollback compatibility is unchanged from revision 6.** `ArtifactDisplay`
  is an existing variant (F1). The durable read keeps the row (F4), and old
  projectors drop it (F9).
- ***(r7)* Admission failure, worded precisely** (revision 6 warning 5).
  - **What clears the artifact:** the action artifact is staged (F5). If
    admission fails, it is cleared along with every staged event. There are
    two ways this happens:
    - an admission `Error` clears staged events (F6);
    - a model-binding failure clears them through `begin_reduced_failure`
      (F28).
  - **What the client sees:** the run's `Error` with its **original** code and
    the replaced message "Run admission failed before protected content
    publication" (F30). The client never sees `ADMISSION_FAILED`, which is an
    internal marker.
  - **What already happened before the failure:** the run record was written
    to durable history, `session_current_run` was set (F29), and MCP admission
    and RAG preparation may have run. **No model call and no tool call has
    run.**
  - **Why a retry is then admitted afresh:** a retry finds no action artifact
    in that binding. Within S3's certainty rule, that binding is complete
    (sealed with the error terminal) and has no action. So the retry is
    admitted afresh, which is correct because the earlier attempt reached
    neither the model nor any tool.
- **Visibility.**
  - **Native-route observers** see the artifact (F10, F11). Its content is the
    run's own input, already visible to them as the user turn.
  - **The console** renders it as a visible JSON block today (F31–F33). §4 S5
    plans the hide.

## 4. Changes

### S0. *(r7)* The action travels with the input and is emitted at most once

- **Field:** unchanged from revision 6. `a2ui_action: Option<A2uiActionRecord>`
  is added to `RunExecutionRequest`, and to `CreateRunRequest` with
  `#[serde(skip)]` (F22). Only `agui.rs` sets it, after S3.
- **No second copy** (revision 6 suggestion).
  - `A2uiActionRecord` holds **no copy** of the input.
  - The manager builds the artifact `content` from the request's own `input`
    string, so the two can't diverge.
  - It builds `metadata` from the fixed profile.
  - A unit test, `action_artifact_content_is_request_input`, lives in the
    `manager.rs` tests, where the emitter is. It checks that the content and
    the input are the same string.
- ***(r7)* Emitted at most once** (revision 6 warning 3).
  - **Where:** `execute_request_inner` emits the artifact right after it
    constructs `RunEventEmitter` (`manager.rs:3975-3986`). It does so only
    when `request.a2ui_action.take()` returns `Some`.
  - **Why `take()`:** a request cloned or forwarded after this point no longer
    carries the action. That covers a graph root's hosted turns (F26) and child
    threads (F27), so neither can emit it again.
  - **The graph path never receives an action:** S3 refuses actions for graph
    agents. That makes the graph path's not entering `execute_request_inner`
    directly (F25) irrelevant.

### S1. *(r7)* Thread surface record and lookup: unchanged, except the lookup rule

The fold, the merge, completeness tracking and S1a are unchanged. The
`actionId` lookup is now *replay or refuse* (revision 6 critical).

The revision 6 lookup missed actions in two cases:

- the action's binding was `history_incomplete`;
- the action's binding had left the 64-binding window while its surface was
  still inside it.

Either miss let the declaration check admit the same `actionId` twice.
Revision 7 ties the lookup's scope to the surface the action targets and
refuses whenever the scope can't be complete.

- **Scope.** For an action targeting surface *S*, the lookup scans every
  binding from the earliest binding in the window that contains a
  `createSurface` for *S* up to the latest binding, including the latest even
  while it is open. It reads each binding's action artifacts with the same
  merged read as the fold.
- **Why that scope is complete when certain.** An action on *S* can only be
  dispatched from a rendered surface, which needs *S*'s `createSurface` to have
  come first. Every earlier action on *S*, from any incarnation that began
  inside the window, therefore sits in a binding within that scope.
- **Certain, and the lookup can be trusted, when both hold:**
  1. no binding in the scope is `history_incomplete` or `history_unavailable`;
  2. *S*'s earliest `createSurface` found in the window is **not** in the
     window's oldest binding, or the window's oldest binding is the thread's
     first binding. The thread has 64 or fewer bindings, or *S* was first
     created after the window begins.
- **Uncertain otherwise.** An earlier incarnation of *S*, and its actions,
  could then lie before the window or in a binding whose history is missing.
- **The rule:**
  - **Found, digest and workspace match:** replay that binding.
  - **Found, but they don't match:** `409 a2ui_action_id_conflict`.
  - **Not found, certain:** continue to the latest-run rule and the
    declaration check.
  - **Not found, uncertain:** refuse with
    `422 a2ui_action_history_uncertain`. Never admit.
- **The guarantee:** an `actionId` that already ran is never admitted again,
  as long as its run's action artifact was published, which happens once
  admission completed (F5).
  - **The only exception is the losing twin.** Its rival's artifact isn't
    published yet, so the twin is refused with `409 thread_run_conflict` by the
    latest-run rule (F12), because the winning run is still open. It is still
    not admitted twice.
  - **Unpublished artifacts never ran.** A run whose admission failed has no
    artifact, and it reached neither the model nor any tool.
- **Cost:** the lookup reuses the fold's reads, so it adds no I/O.

### S3. *(r7)* Inbound: the graph-agent decision

These replace the revision 6 S3 items; everything else in S3 stays.

- ***(r7)* Graph agents are refused for actions** (revision 6 warning 3).
  - **The rule:** if the path agent uses the agent graph, `run()` refuses an
    `a2uiAction` with `422 a2ui_action_unsupported_for_graph_agent`. The
    agent-graph check is `manager.uses_agent_graph(artifact)`, exposed to the
    route as `uses_agent_graph_for(agent_id)` (F25).
  - **What this means today:** that is `orchestrator-agent` whenever an agent
    graph is configured.
  - **Why refuse rather than support:**
    - surfaces published by an orchestrator's specialists live in *child*
      native runs, not in the thread's bindings, so the thread record can't
      see them;
    - the graph path doesn't emit through `execute_request_inner` (F25).

    Supporting it needs the thread record to follow child runs. That is out of
    this slice (§10).
- **Unchanged otherwise:**
  - the check order: same `runId`, then the S1 lookup, then the latest-run
    rule, then the declaration check, then reserve;
  - the losing-twin behaviour: `409 thread_run_conflict`, or `503` on a store
    conflict;
  - the bounds and the `resume` refusal;
  - the authority list;
  - dropping activity messages.
- **New refusal codes:**
  - `422 a2ui_action_history_uncertain` (S1);
  - `422 a2ui_action_unsupported_for_graph_agent`.
- **Caller-tool continuations** (revision 6 suggestion).
  - **What happens today:** with a frontend continuation, `create_request`
    builds the turn from `continuation.history` and leaves `prompt` empty
    (`agui.rs:393`, `:418-421`).
  - **The rule:** an `a2uiAction` on a request that is a caller-tool
    continuation is refused with `400 a2ui_action_with_continuation`. Actions
    are always ordinary turns, so "the echo shows the run input" holds.
  - **Client `context` and `state` text:** it goes into
    `interaction.clientContext` (revision 5 S3), never appended to `prompt`
    (`:405-408` is bypassed by the action builder).

### S5. *(r7, new)* Console: hide the raw action artifact (UI change, through UI routing)

Today the console shows every action run's `a2ui_action` artifact as a JSON
block containing the full run input (F31–F33). Revision 7 plans to change
that. The plan names the change; the design is decided in Execute.

- **Where.** Two classification points:
  - the `__a2ui_display__` branch in `chat-message-store.ts:367-382`;
  - the `uar.artifact.available` case in `runtime-chunk-projection.ts:168-174`.
- **What.** An artifact with `artifactType` / `artifact_type` equal to
  `"a2ui_action"` is not rendered as a raw JSON block.
  - **Proposed default** (subject to UI routing): one compact, non-interactive
    line naming the action, for example "Action: Confirm". The name is parsed
    from the scrubbed content. Showing nothing is the fallback.
- **How it gets decided.** Execute follows the repository's mandatory UI
  routing *before* any frontend edit:
  - memory recall;
  - UI/UX Pro Max analysis;
  - Impeccable `audit` and `critique`, run by two isolated subagents, followed
    by a fresh adversarial review of the resulting design;
  - Anthropic `frontend-design` and `ux-designer`;
  - Vercel React Best Practices and Composition Patterns;
  - a one-paragraph summary of the applied rules.

  This follows CLAUDE.md "UI/UX work routing", with Impeccable first per
  "UI design skill precedence".
- **State path.** The change is a classification inside the existing message
  projection. It adds no new state and no new store. Execute confirms that it
  doesn't duplicate any graph-owned entity, per "React and entity-state
  architecture".
- **Tests.** Two frontend unit tests are authored and run only at the gate:
  - `a2ui_action artifacts are not rendered as JSON blocks`, in the tests for
    `chat-message-store` and `runtime-chunk-projection`;
  - the existing artifact tests stay unchanged.

## 5. *(r7)* Tests

This is revision 6 §5 with these changes:

- **Replaced:** `action_artifact_content_equals_run_input` becomes
  `action_artifact_content_is_request_input` (in `manager.rs` tests).
- **Moved:** `admission_failure_clears_action_artifact` stays in `manager.rs`
  tests, but now only asserts that the artifact is cleared and that the
  original error code is kept. "A retry is admitted afresh" moves to
  `agui_a2ui.rs` as `failed_admission_binding_is_certain_and_actionless`.
- **Added to `agui_a2ui.rs`:**

  | Case | Test |
  |---|---|
  | The lookup scope starts at *S*'s earliest in-window `createSurface` | `lookup_scope_from_surface_create` |
  | An incomplete binding in scope → `422 a2ui_action_history_uncertain` | `incomplete_scope_refuses_uncertain` |
  | *S* may predate the window → refuse as uncertain | `surface_predating_window_refuses_uncertain` |
  | A thread of 64 or fewer bindings with *S* in the oldest binding is still certain | `short_thread_oldest_binding_is_certain` |
  | Graph agents refuse actions | `graph_agent_refuses_action` |
  | An action on a caller-tool continuation is refused | `action_with_continuation_refused` |
  | Metadata is exactly the fixed profile, and every client value is in content | `action_artifact_metadata_is_constant` |
  | A scrubbed field makes a retry conflict, never re-admit | `scrubbed_field_retry_conflicts` |
- **Added to `manager.rs` tests:** `action_emitted_once_after_take` checks
  that a cloned request emits nothing.
- **Added to the frontend tests:** S5.

## 7. *(r7)* Spec delta changes

These change revision 6's requirements 3 and 5:

- **Requirement 3** adds four scenarios:
  - *A retry is replayed when its action artifact is found, and refused when
    the lookup is uncertain; never admitted twice.*
  - *A graph agent refuses actions.*
  - *An action on a caller-tool continuation is refused.*
  - *Client values are carried only in the scrubbed artifact content.*
- **Requirement 5** is unchanged.
- **Requirement 6 is new: the console does not render raw action artifacts.**

## 8. *(r7)* Verification (exact commands; not run; the gate is closed)

Revision 6 §8 still applies, with these changes:

- **Step 1** cargo filters also run
  `action_artifact_ lookup_scope_ incomplete_scope_ surface_predating_window_ short_thread_ graph_agent_refuses_action action_with_continuation_refused scrubbed_field_retry_conflicts action_emitted_once_after_take failed_admission_binding_`.
- **New step 1a:** `pnpm -C frontend test chat-message-store runtime-chunk-projection`,
  the S5 frontend tests. The repository's frontend tier is listed in
  `.claude/rules/typescript.md`.
- **New step 1b:** `pnpm typecheck`, the repository's type gate for the S5
  edit, run once here.
- **Steps 2–6** are unchanged. Step 2's rebuild command is still an open
  precondition.

## 10. The uncomfortable parts

Revision 6's list stands, with these changes.

**Removed** (now fixed):

- the duplicate-admission claim;
- "console rendering unverified";
- the imprecise admission wording.

**Added:**

- **Clicks are refused on very long threads.** On a thread with more than 64
  bindings, an action on a surface whose earliest create sits in the window's
  oldest binding is refused as uncertain, even though nothing went wrong. The
  user sees the click refused. Surfaces created within the last 63 runs are
  unaffected. D2's stored record would remove this.
- **Incomplete history refuses clicks.** Any `history_incomplete` binding
  between a surface's creation and now makes every click on that surface
  refused as uncertain. This is fail-safe, but it is visible.
- **Scrubbing can change parsed values.** If the scrubber redacts part of a
  client field, retries of that action conflict with `409`, and the echo shows
  the redacted value.
- **Orchestrator threads get no actions.** On `orchestrator-agent` threads,
  actions are refused. The surfaces that specialists publish in child runs are
  not on the thread record at all, so those threads get no A2UI activities from
  specialists. Supporting graph agents is a follow-up.
- **The losing twin still gets a `409`.** It is refused with
  `thread_run_conflict`, not a replay, and a later retry replays. It is never
  admitted twice.
- **The console hide's exact design is undecided.** It waits for UI routing in
  Execute, so its final form is not specified here.

## 11. Tasks (for Execute; none started)

Revision 6 §11 still applies, with these changes:

- **Task 3** also covers:
  - `take()`-based single emission;
  - constant metadata;
  - content built from `input`.
- **Task 4** also covers:
  - the surface-scoped lookup with replay-or-refuse;
  - the graph-agent and continuation refusals.
- **Task 6** also covers:
  - re-reading the installed tarball's `headless.mjs` imports (F18);
  - not depending on `@ag-ui/core` (F19).
- **New task 7a:** S5. Run the UI routing first, then make the classification
  edit and author its tests.
- **Task 7** ("check the console") is superseded by 7a.

## Revision 7 independent review — 2026-10-28

A fresh-context artifact critic reviewed revision 7 together with revision 6,
which revision 7 references. It ran only `npm view`. Verdict: **defects found**:
1 critical, 4 warnings, 3 suggestions. **The ledger is correct:** every
corrected and new entry (F7, F18, F19, F22, F25–F33) matches its cited line.
One refinement to F33: the server nests `profile` inside `metadata` with
camelCase keys (`adapters.rs:213-216`). The lead confirmed the starred items in
source (✓).

### Critical

1. **The proof that a "certain" lookup is complete fails when a surface is
   recreated.**
   - **Counterexample:**
     - the window is B10..B73;
     - S is created in B5, outside the window;
     - an action on S runs in B12;
     - S is deleted in B13 and recreated with the same id in B40.
   - **Why it passes:** the earliest `createSurface` inside the window is in
     B40, which isn't the window's oldest binding, so condition 2 rates the
     lookup certain. The scope B40..B73 excludes B12, so a retry of B12's
     `actionId` is admitted again.
   - **Root cause:** the A2UI envelope carries no incarnation.
   - **A second gap:** an earlier in-window `createSurface` inside a
     `history_incomplete` binding before the scope is invisible to the check.
   - **What would make it sound:** without a persisted record, the only sound
     certainty is that the window starts at the thread's first binding and no
     binding in it is incomplete.

### Warnings

2. **✓ A scrubbed `actionId` re-admits instead of returning `409`.**
   - **The mechanism:** `content` is scrubbed as plain text
     (`event_projection.rs:263`), not with the JSON-preserving
     `project_json_text` (`:252-259`).
   - **The trigger:** the scrubber includes client-supplied `run_credentials`
     values (`credentials.rs:206-212`). A captured value inside `content` can
     alter the `actionId` or break the JSON.
   - **The fix:** key the lookup on the host-derived `artifact_id`, which is
     never scrubbed (`:266-267`), and refuse when `content` doesn't parse.
3. **`uses_agent_graph_for(agent_id)` on the path string can miss a deployment
   binding that resolves to `orchestrator-agent`.**
   - **The mechanism:** `agent:<binding>` resolves through
     `deployment_binding_id` to `bound.artifact` (`agui.rs:412-414`;
     `routes.rs:551-584`).
   - **The precedent:** the check belongs on the resolved `request.artifact`,
     as `structured_graph_unsupported` does (`manager.rs:3854-3858`).
   - **What is otherwise right:** the refusal itself is complete. Non-graph
     agents spawn no children (`manager.rs:6381-6387`; `kernel.rs:593`).
4. **The `take()` rationale is wrong.**
   - **Why `take()` doesn't apply:** `request` is passed by value and
     destructured exhaustively (`manager.rs:3887-3921`), so no `request`
     exists at `:3975`.
   - **What actually ensures single emission:** fresh child construction plus
     the graph refusal.
   - **The test contradicts the design:** clones made earlier still carry the
     field (`:3313`), so the test "a cloned request emits nothing" can't pass.
5. **The console hide misses two paths.**
   - **The `agui_spec` path:** the adapter spreads camelCase `artifactType`
     (`agui-adapter.ts:98-99`), but `chat-stream-store.ts:1145-1147` reads
     `e.artifact_type`. The type then arrives as `""`, so the hide never
     triggers.
   - **Reload:** content blocks are re-chunked through `to-chunks.ts:97-105`
     (via `use-chat-runtime.ts:34` and `pglite/legacy-chat-content.ts:113`),
     so the block reappears after a reload.

### Suggestions

- **Admission wording:** staged events are cleared on more paths than the two
  listed:
  - `ADMISSION_ENDED` (`event_projection.rs:71-77`);
  - the content limit (`:84-95`);
  - executable-artifact errors (`manager.rs:6280-6284`);
  - `durable_client_tools_unavailable` (`:3989`).

  The conclusion holds, but "no model call" isn't verified against RAG
  embedding work.
- **Replay target:** a warm-resumed action run shares its native run with its
  predecessor (`agui.rs:323-326`). Replaying "that binding" streams the
  interrupt terminal, not the outcome.
- **Frontend gaps:**
  - no `runtime-chunk-projection` test file exists;
  - "two tests" names only one;
  - the T0 tier also lists `pnpm lint`;
  - the Prometheus Entity Management skill read required by CLAUDE.md is not
    listed.

### The framing gap the critic raised

A client can always mint a fresh `actionId` that passes the declaration check.
So `actionId` deduplication protects **honest retries**; it is not a security
boundary. Revision 7's fail-closed wording implied the stronger guarantee.

### Lead assessment

The critical defect is sound. A deleted and recreated surface id has a history
that no condition computed inside the window can certify, because the A2UI
envelope carries no incarnation. Revision 7 tried to prove completeness from
the surface's history. The proof can't work without a persisted record (D2).

**Decision for the operator: what should `actionId` deduplication guarantee?**

- **A: honest-retry idempotency, bounded in time.** This is what PEM's
  `retryAction` needs. Honest retries happen within seconds to minutes of the
  original click.
  - **The certainty rule:** the action's UUIDv7 timestamp, which is part of the
    `actionId` (PEM generates UUIDv7; `forwarded-props.ts:52`), must be no
    older than the window's oldest binding. The whole window is scanned, keyed
    on the host-derived `artifact_id`. Any incomplete binding in the window
    makes the lookup uncertain, and an uncertain lookup refuses.
  - **Why forging the timestamp doesn't help:** making it older only gets the
    action refused, and a retry reuses the same `actionId`, so its time is
    fixed.
  - **What has to be checked first:** that each binding's start time is
    available from its native run record.
- **B: exact-once for the whole thread.** The lookup scans *all* the thread's
  bindings for action artifacts, with no window. It is sound, but its cost grows
  with the thread's length, and it is still bounded by incomplete history.
- **C: defer to D2.** A stored record keyed by `actionId`. This is the correct
  long-term answer, but it needs a schema, which revision 4 showed blocks
  rollback on Postgres.

The warnings are bounded fixes:

- key the lookup on `artifact_id`, and refuse unparseable content;
- check for graph agents on the resolved artifact;
- drop the `take()` claim, and state why emission happens only once;
- hide the action artifact on both console paths and in the reload path.

This is the seventh revision. The surface design has been stable since revision
2. What has kept failing, in the last three revisions, is action
deduplication. Choosing A, B or C settles it.
