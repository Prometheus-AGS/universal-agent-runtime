## ADDED Requirements

### Requirement: Durable team planning is distinct from execution
UAR SHALL persist a team planning instance only for an authenticated owner and workspace, with an exact immutable TeamDefinition package entrypoint and current private DeploymentBinding revision. It SHALL materialize bounded member slots and persist an inactive lifecycle without scheduling any team turn. Capability discovery SHALL distinguish planning support from team execution support.

A planning-only DeploymentBinding MAY have an empty model binding list. Ordinary AgentDefinition activation SHALL still resolve its authored model requirements; this relaxation does not grant executable team or agent capability.

#### Scenario: Bind a reusable team without claiming execution
- **WHEN** an owner installs a package with one TeamDefinition entrypoint, installs a private binding to that package, and creates a scoped team instance
- **THEN** UAR records the exact definition, package, binding revision, input, inactive member slots and revision in durable storage while reporting team execution unavailable

#### Scenario: Refuse wrong immutable or private scope
- **WHEN** the requested TeamDefinition is not an exact entrypoint in the binding package or a different owner or workspace reads the team
- **THEN** the mutation is rejected or the resource is absent to that caller

### Requirement: A team has a bounded dependency task board
UAR SHALL persist task input, output contract, role, dependency IDs, queued state and revision with an atomic expected-team-revision check. Dependencies SHALL reference distinct existing tasks on the same board, so the graph remains acyclic. Replaying an identical command SHALL not create duplicate tasks; reusing its command ID with different content SHALL fail.

#### Scenario: Create a peer or map-reduce board
- **WHEN** an owner adds tasks for materialized roles and references existing same-team dependencies
- **THEN** UAR returns the revised board with the full dependency graph and durable queued tasks

#### Scenario: Concurrent stale editor
- **WHEN** two writers submit task additions against the same expected team revision
- **THEN** at most one addition commits and the other receives a revision conflict

### Requirement: Team administration is discoverable
UAR SHALL expose owner-scoped definition, team and task list/read/create REST methods through its versioned administration descriptor, while preserving the existing ordinary-agent binding contract.

#### Scenario: A host renders usable team roles
- **WHEN** a host lists TeamDefinitions
- **THEN** each summary includes exact identity, package, title, purpose and bounded member role definitions sufficient to select a task role without typing an opaque ID
