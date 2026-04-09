## ADDED Requirements

### Requirement: Session-to-Sandbox Mapping

The crate SHALL provide a `SessionSandboxManager` that maintains a mapping from `session_id: String` to `SandboxHandle`. The manager SHALL be `Send + Sync` and safe for concurrent access from multiple async tasks.

#### Scenario: First tool call in a session creates a sandbox
- **WHEN** `get_or_create(session_id, config)` is called and no sandbox exists for that session
- **THEN** the manager SHALL create a new sandbox using the provided config and return its handle.

#### Scenario: Subsequent calls reuse the existing sandbox
- **WHEN** `get_or_create(session_id, config)` is called and a sandbox already exists for that session
- **THEN** the manager SHALL return the existing `SandboxHandle` without creating a new sandbox.

### Requirement: TTL-Based Expiry

Each session-sandbox mapping SHALL have a time-to-live (TTL). The default TTL SHALL be 30 minutes. The TTL SHALL be extended (reset) on every access to the sandbox via `get_or_create`.

#### Scenario: Sandbox expires after inactivity
- **WHEN** a sandbox has not been accessed for longer than the TTL duration
- **THEN** the manager SHALL automatically destroy the sandbox and remove the mapping.

#### Scenario: Access extends the TTL
- **WHEN** `get_or_create` is called for an existing session with 5 minutes remaining on its TTL
- **THEN** the TTL SHALL be reset to the full 30-minute default (or configured value).

### Requirement: Concurrent Access Safety

The `SessionSandboxManager` SHALL use a concurrent map (such as `DashMap`) for the session-to-handle mapping. Multiple async tasks SHALL be able to look up and create sandboxes concurrently without data races or deadlocks.

#### Scenario: Concurrent lookups for different sessions
- **WHEN** two tasks simultaneously call `get_or_create` for different session IDs
- **THEN** both SHALL complete successfully without blocking each other.

#### Scenario: Concurrent creation for the same session
- **WHEN** two tasks simultaneously call `get_or_create` for the same session ID and no sandbox exists yet
- **THEN** exactly one sandbox SHALL be created, and both tasks SHALL receive the same `SandboxHandle`.

### Requirement: Maximum Concurrent Sandboxes Limit

The `SessionSandboxManager` SHALL enforce a configurable maximum number of concurrent active sandboxes. The default limit SHALL be defined in the sandbox configuration.

#### Scenario: Creation within limit succeeds
- **WHEN** the number of active sandboxes is below the maximum and a new session requests a sandbox
- **THEN** the sandbox SHALL be created successfully.

#### Scenario: Creation at limit returns an error
- **WHEN** the number of active sandboxes equals the configured maximum and a new session requests a sandbox
- **THEN** the manager SHALL return `Err(SandboxError::ResourceExhausted)` with a message indicating the concurrent sandbox limit has been reached.
