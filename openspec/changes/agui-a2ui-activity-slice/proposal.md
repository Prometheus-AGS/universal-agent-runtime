## Why

Apps that connect to UAR's dedicated AG-UI route (`/api/uar/ag-ui/agents/{agent_id}`) with an AG-UI A2UI client, such as PEM 4.3.0, can't use A2UI today. The agent publishes a surface with `a2ui_render` and the run reports success, but the client receives nothing to render. The route's projector drops every A2UI event (`src/uar/api/agui.rs`, `_ => Vec::new()`). This was measured on Mac on 2026-10-28 (catalog I19).

A user's button click can't come back either. A real AG-UI client sends the action in `forwardedProps.a2uiAction` and resends the conversation without a new user message. The route rejects that request, because it requires every run to end with user text.

## What Changes

- **Surfaces reach the client.** Each published A2UI surface is sent as an AG-UI `ACTIVITY_SNAPSHOT` with `activityType: "a2ui-surface"`:
  - `content.a2ui_operations` holds the surface's operations so far;
  - `replace: true` is set;
  - the `messageId` is stable per conversation and surface.

  UAR's top-level `profile` field is stripped from forwarded messages.
- **Surfaces belong to the conversation.** Each run's stream restates the live surfaces published by the conversation's earlier runs, read from durable run history. Buttons keep working across turns, and a reconnecting client sees the surfaces again.
- **Clicks become runs.** A request carrying `forwardedProps.a2uiAction` becomes a new run on the same conversation, whose input is `{"type":"a2ui.user_action",…}`. It is accepted only for a Button the conversation's live surfaces actually declare. Such a request needs no new user message, and `role: "activity"` history messages are dropped instead of rejected.
- **Orchestrator conversations refuse actions** (`422 a2ui_action_unsupported_for_graph_agent`). Their specialists publish surfaces in child runs, which this change doesn't follow.
- **The operations console stops showing the action's raw JSON.** It shows a click-started user turn as a compact action line instead.

**Known limits,** deliberately out of scope:
- a resent click with a new run id acts twice;
- PEM's default catalog id doesn't match UAR's, so the app must register UAR's catalog ids;
- very long runs and very long conversations;
- surfaces past PEM's size limits;
- A2UI from orchestrator specialists.

## Capabilities

### New Capabilities
- `ag-ui-a2ui-activities`: A2UI surfaces as AG-UI activities on the dedicated AG-UI route, and A2UI actions as same-conversation runs.

### Modified Capabilities
(none)

## Impact

- **Code:**
  - `src/uar/api/agui.rs` (projection, restatement, action admission);
  - new `src/uar/api/agui_a2ui.rs`;
  - possibly a small helper in `src/uar/runtime/manager.rs`;
  - the console's message classification under `frontend/src/`.
- **Clients:** existing clients of the route see new `ACTIVITY_SNAPSHOT` events. Conforming AG-UI clients ignore activity types they don't render.
- **Storage:** no change to stored shapes, tables or migrations.
- **Plan:** `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agui-a2ui-projection-and-custom-components/children/a2ui-surface-architecture/plan.md` (revision 8).
