## ADDED Requirements

### Requirement: Acknowledged resource ownership
The runtime SHALL atomically admit and register work, retain completion outcomes, and close resources in dependency order. Owned stores SHALL acknowledge engine closure; borrowed stores SHALL remain host-owned.

#### Scenario: Shutdown waiter is canceled
- **WHEN** a caller waiting for shutdown is canceled
- **THEN** host-owned finalization continues and subsequent callers observe the retained result

#### Scenario: Initialization fails
- **WHEN** startup fails after resources have been acquired
- **THEN** every acquired resource is finalized through the same ownership graph before startup returns

#### Scenario: A standalone command fails or panics
- **WHEN** a command fails after configuration and telemetry initialization
- **THEN** its entrypoint awaits configuration and telemetry finalization, retains all cleanup failures, and returns an exit outcome only afterward

#### Scenario: Windows service readiness and status reporting
- **WHEN** a Windows service is initializing or its shutdown status publication fails
- **THEN** Running requires the actual server readiness acknowledgment and a reporting failure does not abandon server finalization

#### Scenario: An OS shutdown signal arrives during server initialization
- **WHEN** a shutdown signal reaches an owned subscription while database initialization is pending
- **THEN** the host records cancellation before serving, retains accepted initialization until its result, and finalizes acquired resources without publishing readiness

#### Scenario: Store ownership is borrowed
- **WHEN** UAR closes a borrowed persistence scope
- **THEN** UAR users finish while the embedding host retains usable database ownership

#### Scenario: Shutdown races a filesystem or configuration event
- **WHEN** host shutdown begins while a watcher or reload can produce work
- **THEN** external request admission closes first, configuration and file watchers stop and join before dependent teardown, and already admitted watcher work retains its required resources until completion

#### Scenario: A native watcher callback is blocked on a full queue
- **WHEN** watcher construction fails or its owned task exits during shutdown or panic
- **THEN** its event receiver closes before native watcher destruction joins the callback, and lifecycle completion accounts for construction and actual native release

#### Scenario: Startup cancellation closes intake before graph configuration
- **WHEN** cancellation closes startup admission before the acquired resource prefix has its final dependency graph
- **THEN** the supervisor attaches cleanup dependencies before finalization starts and retains every acquired resource until its dependents complete

#### Scenario: A persistence or native-tool caller disappears
- **WHEN** a caller drops its response waiter after a persistence, memory-storage or native-tool operation was admitted
- **THEN** the host retains the operation, its owned inputs and its actual outcome independently of that waiter, and dependency closure waits for the operation

#### Scenario: A blocking extraction exceeds its execution budget
- **WHEN** the configured execution budget expires while an admitted blocking extractor is executing
- **THEN** cancellation is requested, the actual worker remains accounted for until its join, and elapsed time alone does not establish termination or safe dependency release

#### Scenario: Several shutdown callers race
- **WHEN** concurrent callers request shutdown or abandon their shutdown waiters
- **THEN** finalizer registration is atomic with closing supervisor admission, one retained cleanup operation owns all resources, and later callers observe the same result

#### Scenario: An index builder leaves its lookup map before exit
- **WHEN** index removal takes a running builder out of the index lookup map
- **THEN** the engine still joins its independently registered task before physical store closure, and rollback cleanup waits for actual exit before deleting durable build state

### Requirement: Private host lifecycle control
The managed sidecar SHALL advertise `host_lifecycle_control_v1`, retain the `READY:<port>` startup line, and accept versioned lifecycle snapshot and generation-checked reconnect commands through its launch-authenticated inherited input. This authority SHALL remain independent of database availability and database-backed request authorization.

#### Scenario: Windows launcher supplies cancellable input
- **WHEN** a managed Windows sidecar launches
- **THEN** the launcher supplies an inherited overlapped named pipe and the reader awaits completion of any canceled I/O before releasing it

#### Scenario: An older binary lacks lifecycle control
- **WHEN** the managed launcher receives capabilities without `host_lifecycle_control_v1`
- **THEN** it rejects compatibility before publishing a running endpoint and awaits child closure

#### Scenario: Reconnect remains pending while the parent disconnects
- **WHEN** an accepted reconnect is pending and inherited input reaches EOF
- **THEN** the input owner requests shutdown independently of command completion and retains the admitted reconnect obligation until it actually settles

#### Scenario: Independent cleanup remains pending
- **WHEN** one resource branch has not acknowledged completion
- **THEN** independent branches may finalize while its prerequisites remain available

### Requirement: Event-capable remote agent execution
Before dispatching a remote agent operation, the trusted host SHALL establish an authenticated event-streaming capability using the peer's advertised protocol interface. It SHALL reject unsupported peers before dispatch and SHALL NOT fall back to periodic status requests.

#### Scenario: Peer supports polling only
- **WHEN** discovery does not establish event streaming for the configured peer interface
- **THEN** UAR rejects dispatch without creating remote work

#### Scenario: Remote completion stream ends without a terminal receipt
- **WHEN** a remote stream ends, loses continuity or fails before an identity-validated terminal receipt
- **THEN** UAR retains explicit unavailability or outcome uncertainty and does not infer cancellation or rollback
