# Spec Delta

## ADDED Requirements

### Requirement: A2A 1.0 public card identifies a usable conformant interface

UAR SHALL serve an A2A 1.0 agent card at `/.well-known/agent-card.json` without credentials. The card SHALL match the selected pinned 1.0 schema, identify a reachable conformant interface, and advertise only capabilities that the installed service actually implements. The anonymous exemption SHALL apply only to public card reads, including any retained alias.

#### Scenario: Anonymous reader obtains the card

- **WHEN** an unauthenticated client requests `/.well-known/agent-card.json`
- **THEN** UAR returns 200 JSON conforming to the pinned A2A 1.0 card schema with a usable interface URL
- **AND** authenticated task methods do not become anonymously accessible

#### Scenario: Legacy card alias

- **WHEN** compatibility retains `/agent.json`
- **THEN** it is documented as an alias and its anonymous exemption does not extend to task methods

### Requirement: Advertised A2A tasks use the selected 1.0 wire contract

Every interface advertised as A2A 1.0 SHALL use the selected pinned 1.0 methods, message parts, roles, task states and error mapping. A legacy binding MAY remain only with an explicit version/compatibility disposition and SHALL NOT be advertised as 1.0 before it conforms.

#### Scenario: Task exchange and interaction state

- **WHEN** an authorized client sends a message and later reads or cancels its task
- **THEN** the public methods, parts, roles and state values conform to the selected A2A 1.0 schema
- **AND** pending human input is represented by the pinned input-required state rather than a working state

#### Scenario: Insufficient authority

- **WHEN** an authenticated client lacks the required scope for a task operation
- **THEN** the response uses the pinned 1.0 auth-required state or specified protocol error without disclosing another owner's task

### Requirement: A2A task correlation survives process loss without a second executor

UAR SHALL bind each public task to the current verified owner and existing durable thread/actor execution, with an atomic admission and lookup contract that remains valid across restart and competing runtime processes. A process-local mapping SHALL NOT be the sole source of task identity or execution authority.

#### Scenario: Fresh process reads an admitted task

- **WHEN** the original runtime exits after admitting a task and an authorized second runtime reads that task
- **THEN** the second runtime resolves its current durable state under the same owner without creating another logical execution

#### Scenario: Competing submissions

- **WHEN** two authorized requests race to update the same task binding
- **THEN** the durable transition admits at most the intended logical turn and exposes a consistent task identity or explicit conflict

### Requirement: Agent card version equals the published release

The card's top-level version SHALL be plain `x.y.z` and SHALL equal the actual tagged version of the installed published artifact. A hardcoded version or untagged branch build SHALL NOT count as release conformance.

#### Scenario: Installed published artifact

- **WHEN** Kubo reads the card from an installed artifact identified by its release tag and checksum or OCI digest
- **THEN** the top-level version is plain `x.y.z` with no release-candidate suffix and equals that tag's version
