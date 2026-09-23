## Purpose

Defines how UAR connects to a remote SurrealDB with least privilege, which remote URLs it supports, and how it keeps running on a separate, local-only embedded store when the remote is unreachable at startup.

## ADDED Requirements

### Requirement: Remote sign-in can be scoped to a namespace or database
The runtime SHALL support signing in to a remote SurrealDB as a system user defined on a namespace or on a database, selected by configuration, in addition to root. At namespace or database level it SHALL use the configured user, password, namespace and (for database level) database, SHALL NOT fall back to default root credentials, and SHALL fail startup with a configuration error naming the missing field when any is absent. Root sign-in with today's defaults SHALL remain the behavior when no level is configured.

#### Scenario: Namespace EDITOR user starts UAR
- **WHEN** UAR starts against SurrealDB 3.2.4 with a remote `ws://` URL, auth level `namespace`, namespace `uar`, and the credentials of a user defined on namespace `uar` with role `EDITOR`
- **THEN** startup completes, UAR's schema definitions and live queries succeed, and data is written in namespace `uar`

#### Scenario: Scoped user cannot reach other namespaces
- **WHEN** the same user attempts `SELECT` on a table in namespace `memory` or `compass`, `INFO FOR NS` on those namespaces, `INFO FOR ROOT`, or defining a user
- **THEN** every statement fails with a permission error and returns no data from another namespace

#### Scenario: Missing scoped credentials
- **WHEN** auth level is `namespace` and no password is configured
- **THEN** startup fails with an error naming the missing field and UAR does not attempt a root sign-in

#### Scenario: Default remains root
- **WHEN** no auth level is configured and a remote URL is used
- **THEN** UAR signs in as root exactly as before this change

### Requirement: The sidecar never signs in as root
In sidecar mode the runtime SHALL refuse to start against a remote SurrealDB URL when the configured auth level is root or absent.

#### Scenario: Sidecar configured with root
- **WHEN** the sidecar starts with a remote URL and no auth level or auth level `root`
- **THEN** startup fails before any sign-in attempt with an error stating that sidecar mode requires namespace or database sign-in

### Requirement: WebSocket remote URLs are supported
The runtime SHALL connect to a remote SurrealDB through `ws://` and `wss://` URLs. When the database client rejects an `http://` or `https://` URL as unsupported, the startup error SHALL name the equivalent `ws://` or `wss://` URL. Credentials embedded in a URL SHALL NOT appear in any API response or log line.

#### Scenario: ws URL
- **WHEN** `persistence.database_url` is `ws://127.0.0.1:<port>` for a running SurrealDB 3.2.4
- **THEN** UAR connects and signs in

#### Scenario: Unsupported http URL
- **WHEN** `persistence.database_url` is `http://127.0.0.1:<port>` and the client does not support HTTP
- **THEN** startup fails with an error that contains `ws://127.0.0.1:<port>`

### Requirement: Unreachable remote at startup falls back to a local-only embedded store when enabled
When `persistence.remote_fallback` is `embedded` and the remote database cannot be connected at startup — connection refused, name resolution failure, or no connection within the configured connect timeout — the runtime SHALL start on the embedded store at `persistence.fallback_database_url` instead of failing. It SHALL NOT fall back on authentication, permission, schema or other errors from a reachable server. When `remote_fallback` is `none` (the default), an unreachable remote SHALL fail startup as it does today. The runtime SHALL NOT switch between the remote and the fallback store while running.

#### Scenario: Docker stopped
- **WHEN** the sidecar starts with fallback `embedded` while the SurrealDB container is stopped
- **THEN** it prints `READY` within 10 seconds of process start on macOS, Windows and Linux, and serves requests from the embedded store

#### Scenario: Remote reachable but credentials wrong
- **WHEN** the remote is reachable and sign-in fails
- **THEN** startup fails with the sign-in error and no fallback store is opened

#### Scenario: Fallback disabled
- **WHEN** `remote_fallback` is `none` and the remote is unreachable
- **THEN** startup fails as before

#### Scenario: Remote returns while running on the fallback
- **WHEN** the SurrealDB container starts while UAR runs on the fallback store
- **THEN** UAR keeps using the fallback store until it restarts

### Requirement: Fallback data is reported as local-only and never merged automatically
The runtime SHALL treat everything in the fallback store as local-only. `GET /api/config/persistence` SHALL report the effective mode, the configured mode, whether fallback is active, the reason and since when, and a database URL with credentials removed. `GET /api/uar/persistence/local-only` SHALL report whether a local-only store exists and list the sessions it holds (identifier, title, creation and last-update time), whether or not that store is the active one. The runtime SHALL NOT copy, merge or delete local-only data when it later connects to the remote; only an explicit future action may move it.

#### Scenario: Session created during fallback
- **WHEN** a session is created while fallback is active
- **THEN** `GET /api/config/persistence` reports fallback active, and the session appears in `GET /api/uar/persistence/local-only`

#### Scenario: Restart while the remote is still down
- **WHEN** UAR restarts and the remote is still unreachable
- **THEN** the same fallback store is reopened and the session is still listed as local-only

#### Scenario: Restart after the remote returns
- **WHEN** UAR restarts and the remote is reachable
- **THEN** UAR uses the remote, the session is absent from the remote, the fallback store still holds it, and `GET /api/uar/persistence/local-only` still lists it

#### Scenario: Status never exposes credentials
- **WHEN** the configured database URL contains a user and password
- **THEN** neither appears in `GET /api/config/persistence` or in any log line
