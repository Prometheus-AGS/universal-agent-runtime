# A2UI live update

## Purpose

Give A2UI surface updates a path onto the existing `NormalizedEvent::StatePatch`/
`RunManager` broadcast pipeline, and a durable-replay abstraction for
late-joining clients — the two structural pieces of "live update" that
don't depend on an actual orchestrator call site emitting A2UI messages.

## ADDED Requirements

### Requirement: A2UI wire messages convert to StatePatch ops
`surface_message_to_state_patch` MUST convert each of the 4 A2UI wire
message kinds (`createSurface`, `updateComponents`, `updateDataModel`,
`deleteSurface`) into a `StatePatchOp` rooted at
`/a2ui/surfaces/{surface_id}`: `createSurface` MUST produce an `add` op at
that path, `updateComponents`/`updateDataModel` MUST produce `replace` ops
at `.../components` and `.../dataModel` respectively, and
`deleteSurface` MUST produce a `remove` op with no value.

#### Scenario: A createSurface message is converted
- **WHEN** `surface_message_to_state_patch` is called with
  `A2uiWireKind::CreateSurface` and a surface id
- **THEN** the returned op's `op` field is `"add"`, its `path` is
  `/a2ui/surfaces/{surface_id}`, and its `value` is `Some`

### Requirement: Durable replay for late-joining clients
An `A2uiReplayBackbone` trait MUST exist with `publish(run_id, op)` and
`replay(run_id) -> Vec<StatePatchOp>` methods. A conforming implementation
MUST return every patch published for a given `run_id`, in publish order,
and MUST NOT return patches published under a different `run_id`.

#### Scenario: A late-joining reader replays a run's full patch history
- **WHEN** two patches have already been published for `run_id = "run-a"`
- **AND** a reader calls `replay("run-a")`
- **THEN** it receives both patches, in the order they were published

#### Scenario: Replay is isolated per run
- **WHEN** patches have been published for both `"run-a"` and `"run-b"`
- **THEN** `replay("run-a")` returns only `run-a`'s patches, never `run-b`'s

### Requirement: Surface test-trigger and replay endpoints
`POST /api/uar/runs/{run_id}/a2ui/surface-test-trigger` MUST convert the
request body into a `StatePatchOp`, publish it to the run's replay
backbone, and emit it as a `NormalizedEvent::StatePatch` via the same
`RunManager::emit_to_run` path every other run event uses.
`GET /api/uar/runs/{run_id}/a2ui/surface-replay` MUST return the replay
backbone's full history for that run.

#### Scenario: A surface update is triggered and then replayed
- **WHEN** `POST .../surface-test-trigger` is called for an active run
  with a `createSurface` payload
- **THEN** the response is `200 OK`
- **AND** a subsequent `GET .../surface-replay` for the same run includes
  the resulting patch

#### Scenario: The trigger endpoint rejects an unknown message kind
- **WHEN** `POST .../surface-test-trigger` is called with a `kind` value
  outside `createSurface`/`updateComponents`/`updateDataModel`/`deleteSurface`
- **THEN** the response is `400 Bad Request`

#### Scenario: The trigger endpoint requires an active run
- **WHEN** `POST .../surface-test-trigger` is called for a `run_id` that
  does not exist or is not active
- **THEN** the response is `404 Not Found`
