## Why

The operator made the flint-forge PostgreSQL 18 image (`tribehealth/flint-forge-pg:18`) UAR's supported PostgreSQL server. UAR's Postgres store also carried five open defects:

- **#389:** settings provenance is lost across restarts.
- **#390:** the session TTL scan skips legacy keys and scans the whole table.
- **#393:** memory writes report success and are discarded.
- **#394:** Postgres doesn't report which durable features it lacks.
- **#395:** skill embeddings are fixed at 384 dimensions.

## What Changes

- **Server layout:** UAR runs in its own `uar` role and database on a flint-forge PostgreSQL 18 server, which keeps `POSTGRES_DB=flint`. The init script `scripts/acceptance/fixtures/flint-forge-uar-db.sh` creates the role and database, and `docker-compose.prod.postgres.yaml` uses the pinned image.
- **Settings provenance:** settings keep an explicit `source` (`config` or `api`), so a config-driven value follows the config file across restarts and an API override survives as drift.
- **Session expiry:** the TTL sweep decodes legacy session keys and filters expired rows in SQL.
- **Memory:** Postgres refuses memory writes and searches with `memory_service_required`, as SurrealDB does. Memory lives in the independent memory service.
- **Skill embeddings:** `skills.embedding` accepts any dimension and carries an embedding fingerprint. Skills re-embed when the fingerprint changes.
- **Capability reporting:** `GET /api/uar/persistence/capabilities` reports the active store's durable capabilities and the memory store. Unsupported durable features answer `capability_unsupported`.

## Capabilities

### New Capabilities
- `postgres-persistence`: what UAR guarantees when PostgreSQL 18 on the flint-forge image is its store.

## Impact

- **Code:** `src/uar/persistence/**`, `src/uar/settings/manager.rs`, follow-up and schedule controllers, `src/server.rs` (capabilities route), two new migrations.
- **Deployment:** `docker-compose.prod.postgres.yaml`, `.env.example` (`POSTGRES_SUPERUSER_PASSWORD`), `config.remote.postgres.yaml`.
- **Docs:** `docs/operations/postgres-flint-forge.md`, `docs/operations/persistence-capabilities.md`.
- **Not changed:**
  - Kubernetes and Helm manifests still use PostgreSQL 17; they're tracked for 1.4.0-final.
  - Team execution stays SurrealDB-only.
