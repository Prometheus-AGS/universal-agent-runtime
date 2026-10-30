## MODIFIED Requirements

### Requirement: Readiness probe checks dependencies
The `/readyz` endpoint SHALL verify the configured persistence backend, returning HTTP 200 only if every configured dependency check passes. The response SHALL report `checks.persistence` as an object naming the configured `backend` and its `status` (`ok`, `failed` or `not_configured`). The `checks.postgres` key SHALL be present only when the backend is PostgreSQL. When a check fails, the runtime SHALL log the underlying error and SHALL NOT include it in the response.

#### Scenario: All dependencies healthy
- **WHEN** a GET request is made to `/readyz` and the configured persistence backend is reachable
- **THEN** the server returns HTTP 200 with `status` `ready` and `checks.persistence.status` `ok`

#### Scenario: PostgreSQL unreachable
- **WHEN** a GET request is made to `/readyz`, the backend is PostgreSQL, and it is unreachable
- **THEN** the server returns HTTP 503 with `status` `not_ready`, `checks.persistence` `{"backend": "postgres", "status": "failed"}` and `checks.postgres` `failed`

#### Scenario: Remote SurrealDB unreachable
- **WHEN** a GET request is made to `/readyz`, the backend is SurrealDB, and the persistence read fails
- **THEN** the server returns HTTP 503 with `checks.persistence` `{"backend": "surreal", "status": "failed"}` and no `postgres` key
- **AND** the runtime logs `readiness.persistence_failed` with the backend and the underlying error, which the response does not contain.

#### Scenario: Multiple dependencies down
- **WHEN** a GET request is made to `/readyz` and more than one configured dependency is unreachable
- **THEN** the server returns HTTP 503 listing all failed dependencies
