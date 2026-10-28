# Plan — A2UI on the dedicated AG-UI route (revision 8, final, scoped to the likely path)

Stage: Plan. Written 2026-10-28. Nothing here is implemented; the plan stops
before Execute.

## Operator direction

The operator's direction, recorded 2026-10-28:

- Prioritize what users will actually do.
- Rare failure modes are known limits, not design work.
- Duplicate-click protection is out of this slice.

Revisions 1–7 (`plan-rev1.md` … `plan-rev7.md`) and their reviews are kept as
history. This revision supersedes them. It reuses only the parts those reviews
confirmed sound.

## 1. What users get

An app connected to UAR's dedicated AG-UI route
(`/api/uar/ag-ui/agents/{agent_id}`) with an AG-UI A2UI client, such as PEM
4.3.0, can do the following:

1. **See the agent's UI.** When the agent publishes a surface with
   `a2ui_render`, the app renders it.
2. **Click, and the agent responds.** Pressing a button starts a new run on the
   same conversation, and the agent receives the action, including values the
   user entered.
3. **Click again.** The same button still works after the agent answers, and
   after the user types a follow-up.
4. **Reload or reconnect and still see it.** A client attaching to the
   conversation's latest run sees the surfaces again, without the agent running
   again.

Today none of this works. I19 showed this on Mac on 2026-10-28: the route drops
every A2UI event (`agui.rs:651`, `_ => Vec::new()`).

## 2. How it works

### Outbound: surfaces become AG-UI activities

These changes are in `src/uar/api/agui.rs`, with the logic in a new
`src/uar/api/agui_a2ui.rs`.

- **Activity per surface.**
  - **Trigger:** each published A2UI batch (`ArtifactDisplay` with
    `artifact_type == "a2ui"`, `a2ui_output.rs:186-206`).
  - **Event:** one `ACTIVITY_SNAPSHOT` per surface the batch touched, with
    `activityType: "a2ui-surface"` and `content.a2ui_operations` holding that
    surface's operations so far. The snapshot carries `replace: true` and a
    stable `messageId` derived from the thread id and the surface id.
  - **Why the stable id is safe:** this is the shape upstream AG-UI and PEM
    4.3.0 consume. PEM treats a resent snapshot as a no-op.
- **Surfaces belong to the conversation, not one run.**
  - **What gets restated:** at the start of each run's stream, right after
    `RUN_STARTED`, the route restates the live surfaces published by the
    conversation's *earlier* runs.
  - **Where those come from:** they are read from each earlier run's durable
    history, merged with the live ring so a just-finished run counts.
  - **Effects:** button clicks keep working after the first answer, and a
    reconnecting client gets the surfaces back.
  - **Frames:** these restated frames use fixed indices
    `(start_cursor, 1..=48)`.
- **`profile` is stripped.** UAR's top-level `profile` field is removed from
  forwarded messages, because PEM's strict schemas reject it.

### Inbound: a click becomes a new run

- **Where the action comes from.** The client sends the action in the next
  run's `forwardedProps.a2uiAction.userAction`, which PEM's envelope produces.
- **Validation.** UAR checks that the action names a `Button` actually present
  on a live surface of this conversation, with that component id and that
  action name. If it isn't, the request gets `422 a2ui_action_not_declared`
  and nothing runs.
- **The click request** is what a real AG-UI client sends. PEM supplies only
  `forwardedProps`, and the client resends the conversation's messages with
  nothing new from the user. Today `create_request` rejects that request in
  two places, so this slice changes it, for requests that carry an
  `a2uiAction` only:
  - **No new user message is required.** Today a run must end with a user
    text message (`agui.rs:390-393`). For an action request that check is
    lifted, and every message the client sent, including the last assistant
    reply, becomes history.
  - **Activity messages are dropped.** Today they fail as
    `unsupported_history_message` (`agui.rs:398-399`). PEM's message lists
    hold `role: "activity"` entries (`runtime-core.ts:293-300`), so they are
    dropped instead.
  - **New user text is still refused.** A request that has both an action
    and new user text gets `400 a2ui_action_with_text`.
- **The run's input.** The run's input is
  `{"type": "a2ui.user_action", "sourceRunId", "interaction": {…}}`. This is
  the shape of UAR's existing action path (`manager.rs:3156-3160`), carrying
  the click's name, component, context values and timestamp.
- **Client identity is stripped.** The same rule `create_request` already uses
  removes the identity fields a client may not set (`agui.rs:408`).
- **A new click is a new run.** Resending the same request with the same
  `runId` already replays (`agui.rs:185-189`). Otherwise each click request is
  a new run, the same as UAR's REST action path today.
- **Orchestrator conversations refuse actions** with
  `422 a2ui_action_unsupported_for_graph_agent`. Their specialists publish
  surfaces in child runs, which this slice doesn't follow.
  - **Where the check runs:** in `run()`, *before* the run is reserved
    (`create_agui_run`, `agui.rs:363`). It resolves the path agent, through a
    deployment binding when there is one, to its artifact and asks the
    manager's `uses_agent_graph`.
  - **Why not inside admission:** a check there runs after the reservation
    and can't return HTTP `422` (`manager.rs:3854-3858`).

### Console: no raw JSON

The operations console does not render A2UI action data as a raw JSON block.
This mostly matters for runs started by a click.

- **Where:** the change lands at the console's classification points:
  - `chat-message-store.ts:367-382`;
  - `runtime-chunk-projection.ts:168-174`;
  - the camelCase `agui_spec` mapping in `agui-adapter.ts:98-99` and
    `chat-stream-store.ts:1145-1147`;
  - the reload re-chunking in `to-chunks.ts:97-105`.
- **How it's designed:** Execute runs the repository's UI routing first, per
  CLAUDE.md: Impeccable critique with two isolated subagents, an adversarial
  review, `frontend-design`, UI/UX Pro Max, the Vercel skills and the
  Prometheus Entity Management skill. The final design comes out of that.

## 3. Acceptance test: I19 (one scenario, the likely path)

```gherkin
Scenario: An AG-UI app shows the agent's surface, the user clicks it twice, and a reconnect still shows it
  Given a disposable agent on the dedicated AG-UI route that publishes a ticket surface with a note field and a Confirm button
  And a PEM 4.3.0 headless client configured with UAR's catalog ids
  When the client asks for a fresh ticket surface
  Then the client renders the surface with the requested heading and a Confirm button
  When the user enters a note and presses Confirm
  Then the agent answers in a new run on the same conversation and its answer contains that note
  When the user enters a different note and presses Confirm again
  Then the agent answers again and its answer contains the second note
  When a fresh client attaches to the conversation's latest run
  Then that client renders the same surface without the agent running again
  And an action naming a button the surface never declared is refused
```

- **The client.** The test uses `@prometheus-ags/a2ui-react@4.3.0` and
  `@a2ui/web_core@0.12.0`, installed in an isolated folder,
  `scripts/behaviors/clients/a2ui-pem/`, which has its own `package.json` and
  lockfile. That folder is outside the pnpm workspace.
- **The press.** The note value travels through the real press path, using
  web_core's `GenericBinder`. The test doesn't hand-build the action.
- **The click request** has the shape a real client sends:
  - the conversation's prior messages, ending with the agent's reply;
  - the activity messages PEM holds;
  - PEM's `forwardedProps`;
  - no added user message.

  A dummy user message would make the test pass without proving outcome 2.
- **The proof the click arrived** is that the agent's answer contains each
  note. The note values are random and exist only in the client.
- **Bounds.** One model (`liter/MiniMax-M3`), no credentials, three model
  turns.

### Unit tests

Unit tests are authored with the code and run only at the gate:

- projection of a published batch, and restatement of earlier runs' surfaces;
- the `profile` strip;
- the action check: declared, wrong button, wrong name;
- the run input shape;
- the identity stripping;
- the orchestrator refusal;
- the console classification.

## 4. Gate (proposed; not run; the gate is closed until the operator says go)

Run these in order:

1. **Validate the OpenSpec change.** Run `openspec validate agui-a2ui-activity-slice`.
   The change adds a new capability, `ag-ui-a2ui-activities`, covering the four
   outcomes above.
2. ~~Run the unit tests and the typecheck.~~ Removed 2026-10-28 by the operator:
   verification is BDD only. The authored unit tests are not part of the gate.
3. **Rebuild the owned UAR and restart it** with the restart helper.
   The build command is confirmed (Execute, 2026-10-28):
   `CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`.
   It is recorded as actually run in `.prometheus/verification/c40-skill-reload-macos-pass.json:12`,
   `.prometheus/verification/team-01-completion.md:25` and
   `agent-team-concurrency-repair/evidence/build-result.json:2`.
4. **Run I19 once,** then the five scenarios that use this route, once each:
   - `selection-stream-replay`;
   - `live-roster-routing`;
   - `native-tool-approval`;
   - `agui`;
   - `writing`.

5. **Review each scenario's raw evidence** (frames, run records, answers) before
   recording a pass, per `.prometheus/gotchas.md` (2026-10-27, real-path evidence rule).

## 5. Known limits (rare; noted, not solved)

- **A retried click can act twice.** If the client resends a click with a new
  run id, the agent acts on it twice, the same as UAR's action path today. A
  stored action record would fix this later.
- **Default PEM settings render nothing.**
  - **Why:** an app using PEM's default catalog id doesn't match UAR's catalog
    ids. The app must register UAR's catalog ids, as the test does.
  - **Where to fix it:** aligning the ids is a follow-up between PEM and UAR.
- **A long run can drop its own surfaces on reconnect.** For a still-running
  run of more than about 500 events, a reconnect can lose that run's own
  surfaces. This is the route's existing behaviour for long runs.
- **Surfaces past PEM's size limits are refused.** Very large or very
  frequently updated surfaces can exceed those limits, about 500 operations or
  1 MiB, and PEM then refuses them. Deltas are the later fix.
- **Old surfaces drop out of reconnects on very long conversations.** Surfaces
  more than 64 runs back are not restated.
- **Rare runs may get colliding frame ids.** A cold-denied approval or an
  empty-history recovery can share frame ids with the restatement.
- **Orchestrator conversations don't carry A2UI from specialists yet.**

## 6. Tasks (Execute; none started)

1. Create the OpenSpec change.
2. Write `agui_a2ui.rs`: the projection, the restatement from durable plus live
   history, and the action check and builder, with their unit tests.
3. Wire it into `agui.rs`:
   - action requests skip the last-user-message check;
   - activity messages are dropped;
   - the orchestrator refusal runs before the run is reserved.
4. Make the console change, after UI routing.
5. Rewrite I19 and create the isolated client folder.
6. Confirm the rebuild command.
7. Record the implementation-complete inventory, then wait for the operator's
   gate instruction.

## Likely-path review — 2026-10-28

A fresh critic reviewed revision 8 for the likely path only. It found 1 warning
and 2 factual notes, and both have been fixed above.

- **Warning: a real client's click would be rejected.** `create_request`
  requires a final user text message and rejects activity-role history
  (`agui.rs:390-399`). A real client resends the conversation with nothing new
  from the user, so every click would get a `400`. The rule is now specified:
  action requests skip the check, activity messages are dropped, and I19 must
  send the real request shape with no dummy user message.
- **Note: the orchestrator refusal was in the wrong place.** In admission it
  runs after the run is reserved and can't return a `422`. It now runs in
  `run()`, before the reservation.
- **Note: the input shape.** It also carries `sourceRunId`.

The critic also confirmed the rest of the likely path against the code:

- **Projection:** `_ => Vec::new()` drops `ArtifactDisplay`.
- **Publication:** the raw messages keep `profile`.
- **Durable history:** the AG-UI route requires durable AG-UI runs, and both
  providers that support those also provide durable run history.
- **Frame ids:** restated frames at `(0, 1..=48)` don't collide with
  projected frames.
- **Typed input:** PEM preserves the user's typed values across a resent
  `replace: true` snapshot (`activity-plan.ts:150-160`, `:224-226`).
- **A second click works:** the stream seals the run before its terminal frame
  (`agui.rs:700-716`).

The plan is ready for Execute on the operator's go.
