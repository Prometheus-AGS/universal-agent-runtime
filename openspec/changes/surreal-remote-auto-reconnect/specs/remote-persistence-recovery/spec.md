## ADDED Requirements

### Requirement: Standalone server recovers a lost remote SurrealDB transport
When no host lifecycle control is attached, the runtime SHALL detect the loss of a remote SurrealDB transport for persistence or memory and SHALL re-establish it through the registered, generation-checked reconnect handler, including sign-in, namespace and database selection, and subscription re-registration.

#### Scenario: SurrealDB server restarts under a running UAR
- **WHEN** a standalone UAR is ready against a remote SurrealDB server and that server process restarts with its data intact
- **THEN** `/readyz` returns 200 again without restarting UAR
- **AND** records written before the restart are readable and a new record can be saved.

### Requirement: Reconnect retries back off and are logged
The runtime SHALL retry a failed reconnect after a delay that starts at 250 ms, doubles after each failure, and does not exceed 30 s, until the handler succeeds or shutdown begins. It SHALL log the loss, each failed attempt with its cause and next delay, and the recovery.

#### Scenario: Server is still down at the first attempt
- **WHEN** the first reconnect attempt fails because the server is not yet listening
- **THEN** the runtime logs the failure and retries with a growing delay no longer than 30 s until the server accepts the connection.

#### Scenario: Shutdown during recovery
- **WHEN** host shutdown begins while the runtime is retrying
- **THEN** the retry loop stops and does not delay the persistence finalizer.

### Requirement: A sidecar host keeps reconnect authority
When a host lifecycle control is attached, the runtime SHALL NOT reconnect a remote transport on its own; reconnects SHALL occur only through the host's explicit `Reconnect` command.

#### Scenario: Sidecar transport loss
- **WHEN** a sidecar-launched UAR loses its remote SurrealDB transport
- **THEN** the runtime reports the transport as unavailable and waits for the host's `Reconnect` command.
