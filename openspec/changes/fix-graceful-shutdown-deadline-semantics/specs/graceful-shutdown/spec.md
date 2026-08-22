## MODIFIED Requirements

### Requirement: Server handles SIGTERM gracefully
Under the `server-full` profile, the server SHALL intercept SIGTERM and SIGINT, stop accepting new HTTP connections as soon as the shutdown signal is observed, and begin draining in-flight HTTP requests and SSE streams without a pre-drain delay. The configured shutdown timeout SHALL be the maximum graceful-drain window measured from signal observation. If shutdown remains incomplete at expiry, UAR SHALL begin forced process termination, attempt a synchronous non-blocking `deadline_enforced` marker before exit, and be observed exited with code 0 within 1 second after the internal deadline. A blocked or backpressured stderr sink MUST NOT extend that exit bound.

#### Scenario: Rolling update with active SSE stream completes in time
- **WHEN** Kubernetes sends SIGTERM to a `server-full` UAR process while an SSE stream is active and that stream completes before the configured deadline
- **THEN** both HTTP listeners refuse new connections after signal observation, the existing stream is allowed to complete, and the process exits with code 0 through the normal completion path without a `deadline_enforced` marker

#### Scenario: No active connections during shutdown
- **WHEN** SIGTERM or SIGINT is received with no active connections
- **THEN** the server begins draining immediately and exits through the normal completion path with code 0 within 1 second

#### Scenario: Shutdown timeout exceeded
- **WHEN** cleanup or an in-flight request or SSE stream remains incomplete at the configured shutdown deadline
- **THEN** a watchdog independent of the async executor makes one bounded non-blocking attempt to emit `deadline_enforced`, forcefully terminates the process without an external kill signal, and the parent observes exit code 0 within 1 second after that deadline; the marker is required when the emergency write is accepted, while parent-observed timing and exit status are authoritative when it is rejected

#### Scenario: Shutdown evidence sink is unavailable
- **WHEN** the ordinary stderr path is locked or backpressured when the configured shutdown deadline expires
- **THEN** the watchdog uses a bounded non-blocking emergency write, does not wait for stderr availability, and the parent still observes exit code 0 within 1 second after the deadline

#### Scenario: Container orchestrator grants termination margin
- **WHEN** a non-root UAR container has a 30-second internal shutdown deadline and receives SIGTERM from an orchestrator whose kill deadline is at least 35 seconds
- **THEN** UAR exits with code 0 before the orchestrator sends SIGKILL and the evidence records the internal limit, external limit, elapsed time, outcome, and absence of external escalation

#### Scenario: Caller-owned HTTP cancellation is not process shutdown
- **WHEN** an embedded sidecar caller cancels only its caller-owned HTTP shutdown token
- **THEN** UAR drains its HTTP listeners without arming the OS-signal process deadline, without emitting `deadline_enforced`, and without terminating the host process; a later SIGTERM or SIGINT still initiates process-scoped shutdown

### Requirement: Resource cleanup on shutdown
Under the `server-full` profile, the server SHALL signal run cancellation, HTTP drain, ingestion cleanup, configured MCP transport closure, and A2A shutdown as soon as OS-signal shutdown begins. On the normal completion path, all registered cleanup work SHALL complete before process exit, including SurrealDB connection release and MCP child shutdown. This profile does not activate the optional SQLx/Postgres backend and UAR owns no Redis client, so neither is part of this profile's cleanup claim. If the graceful-drain deadline expires first, UAR MAY terminate cleanup still in progress but SHALL report `deadline_enforced` when the non-blocking evidence sink accepts the marker and MUST NOT report the shutdown as graceful or cleanup-complete.

#### Scenario: Embedded persistence released on normal completion
- **WHEN** OS-signal shutdown completes before the configured deadline
- **THEN** the active SurrealDB connection is released before process exit and a second UAR process can become ready on the same embedded database path while the original helper process remains alive at a pre-exit barrier

#### Scenario: MCP servers notified on normal completion
- **WHEN** OS-signal shutdown completes before the configured deadline
- **THEN** all configured MCP services are explicitly cancelled, their transports close, and stdio MCP server child processes observe stdin closure before UAR reports normal process completion

#### Scenario: MCP shutdown starts while another cleanup is blocked
- **WHEN** OS-signal shutdown begins while registered ingestion cleanup remains blocked
- **THEN** configured MCP cancellation starts concurrently, its stdio child observes stdin closure without waiting for ingestion cleanup, and the shared process deadline still bounds the blocked branch

#### Scenario: Cleanup exceeds the graceful-drain deadline
- **WHEN** any registered cleanup remains incomplete at the configured deadline
- **THEN** UAR makes one bounded non-blocking attempt to emit `deadline_enforced`, terminates the process within the bounded observation tolerance, and does not emit graceful or cleanup-complete evidence; the marker is required when the emergency write is accepted, while parent-observed timing and exit status are authoritative when it is rejected
