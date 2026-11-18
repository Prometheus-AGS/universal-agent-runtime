## ADDED Requirements

### Requirement: Teams destination and readiness

The console SHALL provide a Teams destination that lists installed team definitions and the team instances of the selected workspace.

#### Scenario: Selection survives a reload

- **WHEN** the operator reloads or shares the page address with a team selected
- **THEN** the same workspace and team are shown

### Requirement: Readiness facts

The console SHALL show four readiness facts: verified owner, selected workspace, configured runtime instance id, and team execution ownership. Each SHALL be shown as satisfied, blocked, unknown or being checked, with the actions it blocks. The console SHALL NOT offer to reclaim execution ownership.

#### Scenario: Runtime has no instance id

- **WHEN** the capabilities report an empty runtime instance id
- **THEN** binding creation is unavailable with that reason

#### Scenario: No teams installed

- **WHEN** the catalog has no team definitions and the workspace has no team instances
- **THEN** the page states that no team package is installed and offers the install action

#### Scenario: Unverified caller

- **WHEN** the team API answers 401 for the caller
- **THEN** the page states that a verified owner is required and that the console must be opened through the identity host
- **AND** shows workspace and execution ownership as unknown
- **AND** makes no further team requests until reloaded

#### Scenario: Execution owned by another incarnation

- **WHEN** the execution-owner view reports `ownsExecution: false`
- **THEN** the page shows the holding claim's epoch and acquisition time and states that the host operator must recover ownership
- **AND** binding creation and the routing test are unavailable with that reason

### Requirement: Package install with preflight

The console SHALL accept a team package as a manifest and its files, report missing files, unlisted files and files whose byte digest differs, then run package preflight and show its diagnostics before any install. It SHALL install only after the operator confirms a preflight the runtime answered without an error, and the confirmation SHALL state that an installed package cannot be removed. A preflight reporting `activationSupported: false` SHALL NOT prevent the install.

#### Scenario: Files do not match the manifest

- **WHEN** a listed file is missing or its digest differs from the manifest
- **THEN** the mismatch is named and preflight is not requested

#### Scenario: Preflight rejects the package

- **WHEN** preflight returns an error
- **THEN** the runtime's reason is shown and the install action stays disabled

#### Scenario: Package preflight reports activation unsupported

- **WHEN** preflight answers with diagnostics and `activationSupported: false`
- **THEN** the diagnostics are shown, the page states that the binding step decides whether a team can run, and the install action is available

#### Scenario: Install succeeds

- **WHEN** the operator installs a package that passed preflight
- **THEN** its team definitions appear in the definitions list without a page reload

### Requirement: Deployment binding creation

The console SHALL create a deployment binding for an installed package from a provider and model for every model requirement of the package's agents, effective limits and an effective budget. It SHALL run binding preflight and install the binding only when activation is supported. It SHALL NOT fill a credential, connection or policy reference without an explicit operator action.

#### Scenario: Model requirements come from the package

- **WHEN** the operator opens binding creation for a package
- **THEN** one model choice is asked per agent model entry in the package export, bound to one of that entry's preferred aliases

#### Scenario: Digest is canonical

- **WHEN** the console submits a binding
- **THEN** its `contentDigest` is the SHA-256 of its canonical JSON with the digest field absent

#### Scenario: A model alias is unbound

- **WHEN** a requested model alias has no provider and model chosen
- **THEN** the binding cannot be submitted and the missing alias is named

#### Scenario: Binding installs

- **WHEN** binding preflight reports activation supported and the operator confirms
- **THEN** the binding is installed and becomes selectable when launching a team

### Requirement: Team launch

The console SHALL create a team instance from a team definition and a deployment binding of the selected workspace, with a member count per role inside that role's declared minimum and maximum, and a JSON input.

#### Scenario: Launch succeeds

- **WHEN** the operator launches a team with valid slots and input
- **THEN** the new team opens in the detail view with its members listed by role

#### Scenario: Launch is refused

- **WHEN** the runtime refuses the launch
- **THEN** the runtime's error code and message are shown and the form keeps the operator's entries

### Requirement: Routing test through the coordinator

The console SHALL let the operator send a test request to a team member role, defaulting to the coordinator role, with an explicit token, cost and elapsed-time reservation, shown beside the team's remaining budget and remaining task capacity. It SHALL create the task, claim it for a member holding that role, and admit it, each with the team's current revisions.

#### Scenario: Test is admitted

- **WHEN** the operator submits a test request while the runtime owns team execution
- **THEN** a task is created for the chosen role, claimed for a member holding that role, and admitted
- **AND** the run view begins refreshing

#### Scenario: A later step fails

- **WHEN** the task is created but the claim or the admission is refused
- **THEN** the task remains visible with the step that is still owed as its action
- **AND** retrying performs only that step with the same command id and creates no second task

#### Scenario: Runtime reports a conflict

- **WHEN** the runtime answers a command with a conflict
- **THEN** the console shows the runtime's own detail, reloads the team, and does not resubmit

### Requirement: Per-member run feedback

The console SHALL show, for every member of the selected team, its role, status, current task, and each execution attempt with its status, effective model, usage, output, state reason and diagnostic, and SHALL link each attempt to its run trace.

#### Scenario: Refreshing stops when work is finished

- **WHEN** no attempt is `queued`, `running` or `cancellation_requested` and no wait is `yield_requested`, `waiting` or `blocked`
- **THEN** the view stops refreshing

#### Scenario: Refreshing fails

- **WHEN** a refresh request fails or is rate limited
- **THEN** the view shows when it was last updated and that refreshing is failing, and offers a manual refresh

#### Scenario: Member is working

- **WHEN** a member has a non-terminal attempt
- **THEN** that member's lane shows the attempt as in progress and the view keeps refreshing

#### Scenario: Member fails

- **WHEN** an attempt ends with an error outcome
- **THEN** that member's lane shows the state reason and diagnostic, and other members' lanes are unaffected

#### Scenario: Member is waiting on tasks

- **WHEN** a member's attempt is `yielded` and its wait is `waiting` or `blocked`
- **THEN** its lane shows the tasks it waits for

#### Scenario: Member handed off

- **WHEN** a member's attempt is `yielded` and its wait has finished
- **THEN** its lane shows the attempt as handed off, not as waiting

#### Scenario: Approvals pointer

- **WHEN** a member has a live attempt
- **THEN** its lane links to the approvals page

#### Scenario: Delegated work

- **WHEN** the coordinator delegates a task to another member
- **THEN** the new task and its attempt appear in the receiving member's lane with the tasks it depends on

#### Scenario: Dependent of a failed task

- **WHEN** a task depends on a task whose attempt failed
- **THEN** the dependent task is shown as blocked by that task
