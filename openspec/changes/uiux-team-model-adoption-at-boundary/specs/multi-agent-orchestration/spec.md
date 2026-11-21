## ADDED Requirements

### Requirement: A running team adopts model changes at its next task boundary

A running agent team SHALL adopt a change to a member's model binding, or to a route the team captured, at its next task boundary. The boundary SHALL be attempt creation by admission, delegation or wait continuation, and the queued-to-running dispatch claim. At each boundary the host SHALL capture the current routes and record the binding revision it captured.

#### Scenario: A member-model rebind is adopted by the next task

- **GIVEN** a running team whose binding revision 1 maps a model alias to route A and has one task in flight
- **WHEN** the operator installs binding revision 2 that changes only that alias to route B and a later task is admitted and dispatched
- **THEN** the later task's attempt records binding revision 2 and effective model route B
- **AND** no "team deployment binding changed" refusal occurs

#### Scenario: An edit to a captured route is adopted by the next task

- **GIVEN** a team whose binding does not pin a profile or settings revision and whose earlier attempt captured route A at settings revision 1
- **WHEN** the operator edits route A's endpoint through the provider settings, which advances its settings revision, and the next task is dispatched
- **THEN** the next attempt records route A at the new settings revision
- **AND** its provenance names the settings edit as the cause

#### Scenario: A queued task starts on the models current at dispatch

- **GIVEN** an attempt admitted and queued on route A, then a model-only binding revision pointing its alias at route B before it is dispatched
- **WHEN** the dispatch claim succeeds
- **THEN** the claimed attempt records route B and the new binding revision

### Requirement: A model-only binding change re-pins the team and other changes still block

When the current binding differs from the team's pinned revision only in `modelBindings` and the per-revision fields `revision`, `contentDigest` and `effectiveBindingReceiptRef`, the host SHALL advance the pin in the same durable transaction that creates or claims the attempt. Any other difference, or a missing revision, SHALL keep blocking as today.

#### Scenario: The team is re-pinned with the attempt

- **GIVEN** a team pinned to binding revision 1 and a model-only revision 2
- **WHEN** the next task is admitted
- **THEN** the team's pinned revision is 2 after the same transaction that created the attempt
- **AND** an operation that is not a boundary does not change the pin

#### Scenario: A skill or package change still blocks

- **GIVEN** a running team and a new binding revision that changes a model route and also changes a skill binding, the package, limits, budget, grants, storage, policy revision, extensions, version or provenance
- **WHEN** the next task is admitted
- **THEN** admission is refused with the existing "team deployment binding changed" conflict
- **AND** no attempt is created and the team's pinned revision is unchanged

#### Scenario: A missing revision fails closed

- **GIVEN** a team whose pinned binding revision is absent from the binding history
- **WHEN** a boundary classifies the delta
- **THEN** the delta is treated as blocking and the existing refusal is returned

### Requirement: An explicit route pin in a binding is honoured

A binding model entry that pins a profile or a settings revision SHALL keep that meaning at a boundary. A stale pin SHALL fail the boundary with the existing revision conflict code and SHALL NOT be replaced by the route's current settings. A model entry without a pin SHALL use the route's current settings.

#### Scenario: A stale explicit pin is not silently replaced

- **GIVEN** a binding whose model entry pins a settings revision, and a route edit that advances that revision
- **WHEN** the next task reaches its boundary
- **THEN** the boundary fails with `TEAM_REVISION_CONFLICT` and no attempt runs on the newer settings

#### Scenario: A new binding revision clears a stale pin

- **GIVEN** the stale pin above and a model-only binding revision that carries the current settings revision
- **WHEN** the next task reaches its boundary
- **THEN** the attempt is created on the current settings

### Requirement: A task in flight finishes on the model it started with

Once an attempt is dispatched, the host SHALL NOT change its models, binding revision or captured endpoint profile. Revalidation, tool-claim revalidation, fencing and settlement SHALL compare against the attempt's own captures and SHALL NOT recompute model facts from the live provider registry or from a newer model-only binding revision. The existing cancellation control SHALL remain the way to stop it.

#### Scenario: A rebind and a route edit during a running task

- **GIVEN** an attempt running on route A that calls tools
- **WHEN** the operator installs a model-only binding revision pointing the alias at route B, edits route A's endpoint settings, and the attempt makes further tool and model calls
- **THEN** every request of that attempt is served through route A as captured
- **AND** the attempt settles with its own usage and no `TEAM_ROUTE_PROFILE_MISMATCH` or `TEAM_REVISION_CONFLICT`

#### Scenario: A route disabled during a running task does not stop it

- **GIVEN** an attempt running on route A
- **WHEN** route A is disabled
- **THEN** the attempt continues on its captured route until it settles or the operator cancels it

#### Scenario: A non-model change still invalidates a running attempt

- **GIVEN** an attempt running on route A
- **WHEN** a new binding revision changes a skill binding of that member
- **THEN** the attempt's next revalidation fails as it does today and the failure is visible on the attempt

### Requirement: An unavailable route at the boundary fails visibly

When the route required at a task boundary is unavailable, the host SHALL fail visibly and SHALL NOT substitute another model, the system default model, a fallback model or the previous route. Admission and delegation SHALL refuse with the existing conflict code and create no attempt. The dispatch claim SHALL settle the queued attempt as failed with state reason `model_route_unavailable_at_boundary`, a diagnostic carrying the existing code, and zero model requests sent.

#### Scenario: Admission with a disabled new route

- **GIVEN** a model-only binding revision that points the alias at route B, and route B is disabled
- **WHEN** the next task is admitted
- **THEN** admission is refused with `TEAM_ROUTE_PROFILE_MISMATCH` and no attempt is created
- **AND** no request reaches route A or route B

#### Scenario: A queued attempt whose route disappears before dispatch

- **GIVEN** an attempt admitted and queued on a route that is then disabled before the dispatch claim
- **WHEN** the dispatch claim is attempted
- **THEN** the attempt is settled failed with state reason `model_route_unavailable_at_boundary` and a diagnostic with code `TEAM_ROUTE_PROFILE_MISMATCH`
- **AND** the attempt is not left queued and no model request is sent

#### Scenario: Other unavailable cases fail the same way

- **GIVEN** a new route whose profile or reasoning setting the team profile does not support, whose catalog price identity is missing, or that fails the member's model requirements, or a role with no model binding
- **WHEN** the next task reaches its boundary
- **THEN** it fails visibly with the existing code for that case and no other model is used

#### Scenario: The same rebind with an available route runs

- **GIVEN** the model-only binding revision above with route B enabled
- **WHEN** the next task is admitted and dispatched
- **THEN** the attempt runs on route B, which shows that the refusal came from availability and not from the rebind

### Requirement: Each team attempt records and exposes its model provenance

The host SHALL record on every team attempt the boundary at which its models were fixed, the binding revision captured, the earlier attempt of the same member it was compared with, and per model role whether the route was `initial`, `unchanged` or `changed`, with previous and current route, settings revision and cause. The record SHALL be durable, additive to existing records, and returned by the existing team execution summary.

#### Scenario: Provenance names the change and survives restart

- **GIVEN** two tasks, the second adopted after a model rebind from route A to route B
- **WHEN** the team execution summary is read, and read again after the runtime restarts on the same store
- **THEN** the first attempt's provenance is `initial` for route A and the second attempt's is `changed` from A to B with cause `member-model-rebind` and its binding revision
- **AND** both reads return the same provenance

#### Scenario: An unchanged model is reported as unchanged

- **GIVEN** a second task for the same member with no binding or route change
- **WHEN** its attempt is created
- **THEN** its provenance reports `unchanged` and no change cause

#### Scenario: An attempt without a record is not given one

- **GIVEN** an attempt written before this record existed
- **WHEN** the summary is read
- **THEN** the attempt has no provenance and its `effectiveModels` still show the route that served it

### Requirement: Concurrency checks unrelated to model adoption are unchanged

Relaxing the binding-revision comparison for model-only deltas SHALL NOT change any other `TEAM_REVISION_CONFLICT` check: the expected team revision on dispatch, roster and message cursors, wait and peer command revisions, team-context evidence revisions, and the provider settings-revision edit check.

#### Scenario: A stale team revision is still refused after an adoption

- **GIVEN** a team that adopted a model rebind at a boundary
- **WHEN** a dispatch request carries an expected team revision older than the team's current revision
- **THEN** it is refused with `TEAM_REVISION_CONFLICT`

#### Scenario: A stale provider settings edit is still refused

- **GIVEN** a provider route whose settings revision has advanced
- **WHEN** a provider update carries the older settings revision
- **THEN** the update is refused with `TEAM_REVISION_CONFLICT` and the provider is unchanged
