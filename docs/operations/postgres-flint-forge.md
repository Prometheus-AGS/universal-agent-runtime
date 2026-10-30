# UAR on the flint-forge PostgreSQL 18 image

UAR supports the flint-forge PostgreSQL 18 data-plane image as its PostgreSQL
store. This page says how to run it and why UAR always gets its own database.

## Image

`tribehealth/flint-forge-pg:18@sha256:55c12cbf613346315a48a5c465e804c1ca61ccd1be283db4683ca7c690a9069c`

Built from `images/postgres18/Dockerfile` in
[Know-Me-Tools/flint-forge](https://github.com/Know-Me-Tools/flint-forge). It is
`postgres:18` plus pgvector, pgcrypto, pg_net, pg_cron, pg_graphql and the flint
pgrx extensions (`flint_llm`, `flint_vault`, `ext-flint-meta`, `ext-flint-auth`,
`ext-flint-hooks`). UAR needs only pgvector.

## Why UAR has its own database

On first boot the image creates every flint extension in `POSTGRES_DB`, then
`99-assert.sql` refuses to finish if any is missing. It also starts the server
with `cron.database_name=flint`, so pg_cron only works in a database named
`flint`, and `ext-flint-meta` reflects every table in that database into
GraphQL. So:

- The server keeps `POSTGRES_DB=flint`. Setting it to `uar` breaks first boot.
- UAR never uses `flint`. Its tables would otherwise be reflected into Forge's
  GraphQL surface, and UAR's tenant isolation of its own stores is UAR's job
  (see `docs/SPECIFICATION.md`, partner boundaries).
- UAR gets a role and a database of its own, both named `uar` by default, with
  pgvector created by a superuser. UAR's migrations run as the `uar` role at
  startup.

The init script `scripts/acceptance/fixtures/flint-forge-uar-db.sh` does this.
Mount it as `/docker-entrypoint-initdb.d/50-uar-database.sh` and set
`UAR_DB_PASSWORD` (and optionally `UAR_DB_USER`, `UAR_DB_NAME`). It runs once,
on an empty data directory.

## Layouts

**Dedicated instance.** `docker-compose.prod.postgres.yaml` runs the image for
UAR alone. `.env` keeps `POSTGRES_USER`, `POSTGRES_PASSWORD` and `POSTGRES_DB`
as UAR's role, password and database, so `UAR_PERSISTENCE__DATABASE_URL` is
unchanged. `POSTGRES_SUPERUSER_PASSWORD` is the image's `postgres` superuser.

**Shared flint-forge server.** On a server Forge already runs, a superuser
creates the role and database once (the same statements as the init script),
then UAR connects with
`postgres://uar:<password>@<host>:5432/uar`. Forge's `flint` database is not
touched.

## Upgrading from the PostgreSQL 17 image

PostgreSQL 18 cannot open a PostgreSQL 17 data directory. Dump the old `uar`
database with `pg_dump`, start the new image on an empty volume, and restore
into the `uar` database. The PostgreSQL 18 image also moved `PGDATA` to
`/var/lib/postgresql/18/docker`, so volumes mount at `/var/lib/postgresql`, not
`/var/lib/postgresql/data`.

The PostgreSQL 17 image built from `k8s/postgres/Dockerfile` (pgvector plus
pgmq) is superseded. UAR does not use pgmq.

## Kubernetes

`k8s/base/postgres-statefulset.yaml` and the Helm chart still deploy
`pgvector/pgvector:pg17`. Moving them needs the init script as a ConfigMap, a
superuser password secret key, and the new volume path. That change is not
verified on a cluster yet and is tracked for the 1.4.0-final checklist.

## Verification

`scripts/behaviors/features/postgres-flint-forge.feature` starts this image with
`scripts/acceptance/fixtures/dedicated-runtime.mjs` (backend `postgres`) and
checks startup, restarts, settings provenance, session expiry, memory, skill
embedding dimensions and the capability matrix through UAR's public API.
