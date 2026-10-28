## ADDED Requirements

### Requirement: The dedicated AG-UI route projects A2UI surface, input and state events

The dedicated AG-UI agent route `/api/uar/ag-ui/agents/{id}` SHALL project
A2UI artifacts, A2UI input requests and state patches with the same event
names and value field shape as the chat route's official AG-UI adapter, carrying
this stream's wire `threadId` and `runId` and omitting the native
`sourceRunId`: `Artifact` and `ArtifactDisplay` as `CUSTOM`
`uar.artifact.available`, `ArtifactInputRequest` as `CUSTOM`
`uar.artifact.input_required` (display only on this route), and `StatePatch`
as `STATE_DELTA`. At the start of each run's stream it SHALL emit a
`STATE_SNAPSHOT` built from the replayed A2UI state of the thread's earlier
runs over the baseline `{"run":null,"a2ui":{"surfaces":{},"actions":{}}}`.

#### Scenario: An agent publishes an A2UI surface on the dedicated route
- **WHEN** a client starts a run on `/api/uar/ag-ui/agents/{id}` and the agent publishes an A2UI surface
- **THEN** the stream contains a `CUSTOM` event named `uar.artifact.available` whose value carries the surface's A2UI messages, followed by the run's terminal event
- **AND** its `threadId` and `runId` equal the client's wire thread and run, and the value contains no native run identifier

#### Scenario: The chat and dedicated routes use the same event shape
- **WHEN** an agent publishes a surface through the chat route with `stream_mode=agui_spec` and through the dedicated route
- **THEN** both `uar.artifact.available` values have the same field names and value types, apart from the omitted `sourceRunId`

#### Scenario: A continuation starts from the source run's surfaces
- **WHEN** a continuation run on the same thread updates a surface its source run created
- **THEN** the continuation's `STATE_SNAPSHOT` contains that surface, and its `STATE_DELTA` events apply to it

#### Scenario: Reconnecting replays A2UI events once
- **WHEN** a client reconnects to the dedicated route with `Last-Event-ID` after an A2UI surface was sent
- **THEN** events after that cursor are delivered and the surface event before it is not delivered again

### Requirement: The dedicated AG-UI route accepts an A2UI action as a new run on the same thread

The dedicated route SHALL accept an A2UI action carried in
`forwardedProps.uar.a2ui_action` on a new run for an existing, closed thread
owned by the caller. Before reserving anything it SHALL find a run in the
thread whose surface declares the named action as an action event name and
SHALL prepare the continuation, including host fields. It SHALL then reserve
the new wire-to-native binding, execute exactly one continuation with the
reserved run id, and stream it, with the action as a `STATE_DELTA`
immediately after `RUN_STARTED`. An unknown execution outcome SHALL keep the
reservation so a retry attaches instead of executing again. It SHALL NOT
expose native run identifiers.

#### Scenario: A user presses a button on a surface rendered from the dedicated route
- **WHEN** a client sends a new run on the same thread with `a2ui_action` naming a surface and an action event declared by a run in that thread
- **THEN** the response streams one continuation run under the new wire run id whose answer reflects the action
- **AND** a `STATE_DELTA` adding the action under `/a2ui/actions` follows `RUN_STARTED`

#### Scenario: A second action on the same surface is accepted
- **WHEN** after the first continuation finishes, the client sends another new run with an action declared by the same surface
- **THEN** the route finds the surface's run through the thread and streams one more continuation

#### Scenario: A replayed action does not run twice
- **WHEN** the same wire run with the same `a2ui_action` is submitted again, including after an interrupted response
- **THEN** the route attaches to the reserved continuation, and no second continuation or model call is made

#### Scenario: A string that is not an action event is not an action
- **WHEN** `a2ui_action` names a string that appears on the surface as text but is not any component's action event name
- **THEN** the route refuses it and nothing is reserved or executed

#### Scenario: Invalid host fields are refused before reservation
- **WHEN** `a2ui_action` is valid but its host fields do not match the source run's host resources
- **THEN** the route refuses it, nothing is reserved, and a later run on the thread is accepted

#### Scenario: An action for another owner's thread is refused
- **WHEN** a caller sends `a2ui_action` for a thread owned by a different owner
- **THEN** the route refuses it and nothing is reserved or executed

#### Scenario: An action on a surface from before a restart is refused clearly
- **WHEN** UAR has restarted since the surface was published and a client sends an action for it
- **THEN** the route refuses it with an error stating the surface is no longer active, and nothing is reserved or executed
