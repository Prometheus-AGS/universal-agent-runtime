## ADDED Requirements

### Requirement: A2UI state patches apply web_core's state operations one message at a time

For every A2UI message admitted through tool output (`a2ui_render`,
`presentation_render`) or through `POST /api/uar/runs/{id}/a2ui/messages`,
UAR SHALL reduce the run's surface state by applying that message, on its
own, with the state effect of one of four `@a2ui/web_core` state operations.
The `web_core` version is the one pinned by `frontend/packages/a2ui-core`.
The four operations are:

- `createSurface`;
- `deleteSurface`;
- the component upsert of `updateComponents`;
- `DataModel.set` for `updateDataModel`. Mutations that `DataModel.set` makes
  before it throws SHALL be kept.

The reduction SHALL NOT include any of these:

- catalog lookup;
- version compatibility;
- component property validation;
- composition or topology checks;
- batch processing;
- rendering.

The one exception is hardening: the whole message SHALL be refused, with no
state change, when a surface or component id is `__proto__`, `constructor`
or `prototype`.

UAR SHALL emit exactly the state change the reduction made, as RFC 6902
patches in one `StatePatch` event. Every pointer segment SHALL be escaped
with `~0`/`~1`. The patches are:

- `createSurface`: `add /a2ui/surfaces/<id>` with
  `{surfaceId, catalogId, components:{}, dataModel:{}}`.
- `updateComponents`: one `add /a2ui/surfaces/<id>/components/<componentId>`
  for each component whose value changed.
- `updateDataModel`: one patch at the first node on the path that was absent
  or `null` before the message, or at the target node otherwise. The patch is
  chosen by where that node sits:
  - under an object: `add` or `remove`;
  - inside an array's length: `replace`;
  - at the array's end: `add`;
  - more than one index past the end: one patch of the whole enclosing array.

Values SHALL be compared by structure, with numbers compared by numeric
value. A message whose reduction changes nothing SHALL emit no patch.

A refused message SHALL be logged with its reason and without its payload. A
refused or unchanged message SHALL still be published to the replay log and
as an artifact.

Surface state SHALL be discarded when the run leaves retention.

#### Scenario: A created surface receives its first component update
- **WHEN** a run publishes `createSurface` and then `updateComponents` for the same surface
- **THEN** the state patches apply in order to the run state
- **AND** the surface holds the components keyed by id

#### Scenario: A partial component update keeps the components it omits
- **WHEN** a surface holds `root`, `greeting_copy`, `confirm` and `confirm_label`, and an `updateComponents` message sends only a new `root` and `note`
- **THEN** the surface holds those five components, with `root` replaced by the new one

#### Scenario: A forbidden component id leaves the surface unchanged
- **WHEN** an `updateComponents` message contains a component with id `__proto__`
- **THEN** no state patch is emitted and the surface's components are unchanged
- **AND** the message is still published to the replay log and as an artifact

#### Scenario: Data-model state equals web_core DataModel.set on generated input
- **WHEN** a surface receives a seeded, generated sequence of `updateDataModel` messages, together with fixed cases covering the root path, slash-only paths, array growth with gaps, `null` through holes and non-numeric array segments, vivification past index 10000, primitive intermediates, forbidden segments and unchanged values
- **THEN** after every published batch, the surface's state built from the emitted patches equals the result of applying the same messages, in order and each on its own, to one `web_core` `DataModel`
- **AND** each batch emitted one state patch event for each message whose JSON state changed

#### Scenario: Containers created before a refusal are kept
- **WHEN** a surface receives `updateDataModel` at `/p/q/20000` with a value
- **THEN** the refusal is logged, the data model holds `p` as an empty object, and one `add` patch for `p` is emitted

#### Scenario: An update for a surface the run does not hold
- **WHEN** a run publishes `updateComponents` for a surface it never created
- **THEN** no state patch is emitted
- **AND** the message is published to the replay log and as an artifact
- **AND** later state patches of the run still apply

#### Scenario: A deleted surface leaves the state
- **WHEN** a run publishes `deleteSurface` for a surface it holds
- **THEN** a `remove` state patch applies, and the surface is no longer in the run state

#### Scenario: A continuation does not hold its source run's surface
- **WHEN** an update for a surface created by an action's source run is published to the action's continuation run
- **THEN** the continuation's state snapshot does not contain that surface
- **AND** its replay log holds the update

### Requirement: The state guarantee states its conditions

The state patches of a run SHALL apply in order to the UAR initial state, and
the run stream's state snapshot SHALL equal the reduced state, only while all
of these hold:

- The run's retained history still contains the events that created its
  surfaces. The run history keeps its newest 512 events.
- The surface test trigger has not been called for the run.
- No two publications to the run were concurrent. UAR does not guarantee
  that concurrent publications are recorded in the order they were reduced.
  A different order can leave the snapshot unsynchronized for the rest of the
  run.

When a condition fails, the snapshot SHALL follow the existing rule: it does
not claim synchronization for state it cannot reconstruct.

Surface state is scoped to one run, so a continuation run starts with no
surfaces.

#### Scenario: A test trigger call voids the guarantee for that run
- **WHEN** the surface test trigger is called for a run
- **THEN** its log-encoded entry is emitted unchanged as a state patch
- **AND** later state snapshots of that run are not required to equal the reduced state

### Requirement: The surface replay log is a message and action log

`GET /api/uar/runs/{id}/a2ui/surface-replay` SHALL keep returning one entry
per published message, in publish order and in the existing encoding. This
includes refused and unchanged messages:

- `createSurface`: `add` at the surface path, carrying the message payload.
- `updateComponents`: `replace` at the `components` subpath, carrying the
  message payload.
- `updateDataModel`: `replace` at the `dataModel` subpath, carrying the
  message payload.
- `deleteSurface`: `remove`.

The log SHALL also return each submitted action as `add /a2ui/actions/<uuid>`,
and each surface test trigger entry as sent.

The log is not RFC 6902 run state and is not the live state stream. Clients
that need run state SHALL use the run stream snapshot.

#### Scenario: The log keeps its message encoding
- **WHEN** a run publishes `createSurface` and two `updateComponents` messages, and the replay log is read
- **THEN** it holds three entries whose values are the message payloads, in publish order, with `replace` for the component updates

#### Scenario: The log holds refused messages and actions
- **WHEN** a run publishes an update for a surface it never created and then receives an action
- **THEN** the log holds an entry for the update and an `add /a2ui/actions/<uuid>` entry for the action

#### Scenario: The surface test trigger keeps its payload as sent
- **WHEN** the surface test trigger is called with a `createSurface` payload carrying an extra field
- **THEN** the replay log entry and the emitted state patch carry that payload as sent
