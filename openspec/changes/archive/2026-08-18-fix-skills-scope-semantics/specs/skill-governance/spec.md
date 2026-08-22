## MODIFIED Requirements

### Requirement: Configuration changes take effect without a restart
A scoped configuration change SHALL affect subsequent skill matching without
requiring a restart. Runs already in flight SHALL retain the binding established
at run start. A persisted conversation policy SHALL constrain the skills eligible
for the next run in that conversation.

#### Scenario: Disable takes effect on the next request
- **WHEN** a skill is disabled and a new request is made that would otherwise match it
- **THEN** the skill is not activated, with no restart in between

#### Scenario: In-flight run is unaffected
- **WHEN** a skill is disabled while a run using it is in progress
- **THEN** that run continues with the binding it started with

#### Scenario: Session skill selection gates activation
- **WHEN** a persisted session agent-config excludes a skill for one conversation
- **THEN** the next run in that conversation emits no activation event and injects no prompt overlay for that skill

### Requirement: Deletability is determined by origin, and origin is visible to clients
Built-in skills, including skills loaded from the installed pack, SHALL NOT be
editable or deletable. User-created skills SHALL remain editable and deletable.
The skills API SHALL expose each skill's origin.

#### Scenario: Built-in delete is refused
- **WHEN** a delete is requested for a skill whose origin is built-in
- **THEN** the request is refused and the skill remains present and configurable

#### Scenario: User skill delete succeeds
- **WHEN** a delete is requested for a user-created skill
- **THEN** the skill is removed

#### Scenario: Origin is exposed
- **WHEN** a client lists skills
- **THEN** each entry carries its origin, so a client can offer disable rather than delete for built-ins

#### Scenario: Built-in edit is refused
- **WHEN** a client submits an edit for a built-in skill
- **THEN** the API refuses the edit and leaves the skill unchanged

#### Scenario: Built-in delete control offers disable instead
- **WHEN** a client renders a built-in or installed-pack skill
- **THEN** Edit and Delete are unavailable while the global enable/disable control remains available
