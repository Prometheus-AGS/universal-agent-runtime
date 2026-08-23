# Deep Health Probes Specification

## Purpose

Define lightweight process liveness and dependency-aware readiness behavior for deployed UAR services.

## Requirements

### Requirement: Liveness probe is lightweight
The `/healthz` endpoint SHALL return HTTP 200 if the process is alive and can serve HTTP, without checking external dependencies.

#### Scenario: Process alive
- **WHEN** a GET request is made to `/healthz`
- **THEN** the server returns HTTP 200 with body `{"status": "ok"}`

### Requirement: Readiness probe checks dependencies
The `/readyz` endpoint SHALL verify connectivity to PostgreSQL, Redis, and SurrealDB, returning HTTP 200 only if all dependencies are reachable.

#### Scenario: All dependencies healthy
- **WHEN** a GET request is made to `/readyz` and PostgreSQL, Redis, and SurrealDB are all reachable
- **THEN** the server returns HTTP 200 with body `{"status": "ready", "checks": {"postgres": "ok", "redis": "ok", "surrealdb": "ok"}}`

#### Scenario: PostgreSQL unreachable
- **WHEN** a GET request is made to `/readyz` and PostgreSQL is unreachable
- **THEN** the server returns HTTP 503 with body `{"status": "not_ready", "checks": {"postgres": "failed", "redis": "ok", "surrealdb": "ok"}}`

#### Scenario: Multiple dependencies down
- **WHEN** a GET request is made to `/readyz` and both Redis and SurrealDB are unreachable
- **THEN** the server returns HTTP 503 listing all failed dependencies

### Requirement: Health probes are unauthenticated
The `/healthz` and `/readyz` endpoints SHALL bypass JWT and API key authentication middleware.

#### Scenario: No auth header required
- **WHEN** a GET request is made to `/healthz` without any Authorization header
- **THEN** the server returns HTTP 200 (not 401)
