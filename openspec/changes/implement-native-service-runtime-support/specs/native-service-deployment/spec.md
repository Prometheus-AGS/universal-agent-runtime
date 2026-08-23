## ADDED Requirements

### Requirement: Explicit service environment files fail closed
UAR SHALL accept `--env-file <path>` and the equivalent `UAR_ENV_FILE`. When either explicitly selects a file, UAR SHALL load it before telemetry and configuration; unreadable or invalid content SHALL terminate startup with a contextual error.

#### Scenario: Selected environment file is invalid
- **WHEN** a service starts with an explicitly selected unreadable or malformed environment file
- **THEN** UAR exits nonzero before opening network listeners

### Requirement: Service logging supports an explicit file
When `UAR_LOG_FILE` is set, UAR SHALL open that path and direct operational tracing to it. Failure to open the selected path SHALL terminate startup rather than silently discarding service output.

#### Scenario: Service has no usable standard streams
- **WHEN** a supervisor starts UAR with a valid `UAR_LOG_FILE`
- **THEN** startup and request tracing is appended to the selected file

### Requirement: Windows service controls use graceful cancellation
The Windows-only `service` command SHALL register with SCM, report Start Pending, Running, Stop Pending, and Stopped states, and convert SCM Stop and Shutdown controls into the existing graceful server cancellation path.

#### Scenario: SCM requests stop
- **WHEN** Windows SCM sends Stop or Shutdown to a running UAR service
- **THEN** UAR reports Stop Pending, cancels through its graceful runtime path, and reports Stopped after the server completes
