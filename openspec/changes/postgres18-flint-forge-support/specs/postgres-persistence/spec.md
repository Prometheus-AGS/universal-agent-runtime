## ADDED Requirements

### Requirement: UAR runs in its own database on the flint-forge PostgreSQL 18 image
UAR SHALL run against the flint-forge PostgreSQL 18 image using a dedicated role and database (default `uar`) that is not the image's `flint` database, and SHALL apply its migrations there at startup.

#### Scenario: Startup and restart on PostgreSQL 18
- **WHEN** UAR starts against a flint-forge PostgreSQL 18 server whose `uar` database has pgvector
- **THEN** `/readyz` reports `checks.persistence` as `{"backend": "postgres", "status": "ok"}`
- **AND** skills and agents saved before a UAR restart are returned after it.

### Requirement: Settings keep their source across restarts
The PostgreSQL store SHALL record whether a setting value came from configuration or the API, so that a configuration-driven value follows the configuration file across restarts and an API override survives restarts as drift until it is reset.

#### Scenario: Config changes across three boots
- **WHEN** a configuration-driven setting changes on each of three restarts
- **THEN** the effective value after the third restart is the third configuration value and is not reported as drift.

#### Scenario: API override and reset
- **WHEN** the setting is overridden through the API and UAR restarts with a new configuration value
- **THEN** the API value stays effective and is listed by `/api/uar/settings/drift`
- **AND** after a reset the configuration value is effective.

### Requirement: Persisted sessions expire on PostgreSQL
When `sessions.persisted_ttl_secs` is set, the PostgreSQL store SHALL return every expired session, including legacy-keyed rows, selected in SQL, so the sweep deletes it.

#### Scenario: A stale session is purged
- **WHEN** a session's last activity is older than the TTL
- **THEN** it is no longer listed for its owner and stays deleted after a restart.

### Requirement: Memory operations are never silently discarded
The PostgreSQL store SHALL refuse memory writes and searches with `memory_service_required` instead of acknowledging them; memory SHALL be served by the memory service.

#### Scenario: Memory survives a restart on a PostgreSQL deployment
- **WHEN** a memory record is saved through `/api/memory` with PostgreSQL as the store
- **THEN** it is readable after a UAR restart
- **AND** `/api/uar/persistence/capabilities` names the memory store.

### Requirement: Skill embeddings accept the configured dimension
The PostgreSQL store SHALL store skill embeddings of any dimension with their embedding fingerprint, re-embed skills when the active fingerprint changes, and match only skills embedded with the active fingerprint.

#### Scenario: A 1024-dimension embedder
- **WHEN** skills are saved with a 1024-dimension embedder
- **THEN** skill matching returns them before and after a restart, and no skill save fails.

### Requirement: Durable capabilities are reported and unsupported ones refused
`GET /api/uar/persistence/capabilities` SHALL report each durable persistence capability of the active store and the memory store, and an operation needing an unsupported capability SHALL fail with `capability_unsupported` naming it.

#### Scenario: Team durability on PostgreSQL
- **WHEN** the store is PostgreSQL
- **THEN** the matrix reports channel observers, durable observers, durable agent instances and checkpoint resume as unsupported
- **AND** a follow-up request is refused with `capability_unsupported`.
