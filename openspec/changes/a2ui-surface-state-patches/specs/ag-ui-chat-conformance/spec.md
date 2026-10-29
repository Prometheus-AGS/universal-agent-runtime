## ADDED Requirements

### Requirement: The UAR initial state holds A2UI surfaces and actions

The UAR initial state of a run, which "Cursor-consistent attach and replay
snapshots" builds state snapshots from, SHALL be
`{"run":null,"a2ui":{"surfaces":{},"actions":{}}}`. While the conditions of the
`a2ui-surface-state` guarantee hold for the run:

- an action patch `add /a2ui/actions/<uuid>` SHALL apply to the run's state;
- the `STATE_DELTA` patches replayed after a cursor SHALL apply in order to
  the snapshot at that cursor.

#### Scenario: An action patch applies to the initial state
- **WHEN** an action is submitted for a published surface of a run whose earlier state patches all applied
- **THEN** the run stream's state snapshot holds that action under `a2ui.actions` and claims synchronization

#### Scenario: Cursor resume reaches the newest state
- **WHEN** a client resumes the run stream with a valid last-event cursor and applies the replayed `STATE_DELTA` patches in order to its `STATE_SNAPSHOT`
- **AND** a new attachment at the newest replayed event receives a `STATE_SNAPSHOT` that claims synchronization
- **THEN** every patch applies
- **AND** the result equals that new attachment's `STATE_SNAPSHOT`
