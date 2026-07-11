---
sidebar_position: 5
title: Backup and Restore
---

# Backup and Restore

This runbook covers backing up and restoring UAR's application state. The
procedure depends on which persistence provider you run. The primary focus is
the **embedded SurrealDB (SurrealKV) datastore**, which stores everything in a
plain on-disk directory and is the default for single-machine and development
deployments.

## Where the data lives

The datastore location comes from `persistence.database_url`
(`UAR_PERSISTENCE__DATABASE_URL`):

| Provider / URL | Data location |
|---|---|
| `surreal` + `surrealkv://./data/uar-db` | The directory `./data/uar-db` (relative to the working directory). |
| `surreal` + `rocksdb://./data/uar-db` | Same as above — `rocksdb://` is normalized to `surrealkv://`; the path is the on-disk directory. |
| `surreal` + `memory` / `mem` | In-memory only — **not persisted**, nothing to back up. |
| `surreal` + `http(s)://` / `ws(s)://` | Data is owned by the remote SurrealDB server, not by UAR. |
| `postgres` + `postgres://…` | Data is owned by the PostgreSQL server. |

For the embedded engine, the entire database is that one directory. Backing up
UAR embedded state is therefore a **directory copy**. In Docker, this directory
is inside the volume mounted at the datastore path (e.g. `/data`).

:::caution The embedded engine holds an exclusive lock
SurrealKV takes an exclusive `LOCK` on its data directory while the server is
running. **Do not copy the directory while UAR is live** — a hot copy can be
inconsistent, and two processes cannot open the same embedded datastore at once.
Stop the server first (cold backup), as shown below.
:::

## Backing up the embedded datastore (cold copy)

```bash
# 1. Stop the server so the exclusive lock is released.
#    Docker:
docker compose -f docker-compose.prod.yaml stop app
#    or systemd:  sudo systemctl stop uar
#    or foreground:  Ctrl-C

# 2. Copy the data directory to a timestamped archive.
#    (Path is whatever UAR_PERSISTENCE__DATABASE_URL points at, minus the scheme.)
tar czf uar-backup-$(date +%Y%m%d-%H%M%S).tar.gz -C ./data uar-db

# 3. Restart the server.
docker compose -f docker-compose.prod.yaml start app
#    or:  sudo systemctl start uar
```

For a Docker **named volume**, back up the volume contents rather than a bind
path:

```bash
docker run --rm \
  -v uar_data_prod:/data:ro \
  -v "$PWD":/backup \
  alpine tar czf /backup/uar-backup-$(date +%Y%m%d).tar.gz -C /data .
```

Store backups off-host. Verify each archive is non-empty and lists the expected
files (`tar tzf <archive>`).

## Restoring the embedded datastore

```bash
# 1. Stop the server.
docker compose -f docker-compose.prod.yaml stop app

# 2. Move the current (possibly corrupt) directory aside — never delete first.
mv ./data/uar-db ./data/uar-db.broken-$(date +%s)

# 3. Extract the backup into place.
tar xzf uar-backup-20260711-120000.tar.gz -C ./data

# 4. Confirm the restored directory sits exactly where DATABASE_URL expects,
#    then restart.
docker compose -f docker-compose.prod.yaml start app

# 5. Health-check and spot-check data.
curl -sf http://localhost:1906/healthz
```

Restore into a datastore built by the **same or a compatible UAR version**.
Restoring an older on-disk format into a much newer build (or vice versa) is not
guaranteed to work — see the [Upgrade guide](./upgrade-guide).

## Remote SurrealDB

When `database_url` is an `http(s)://` or `ws(s)://` endpoint, the data belongs
to the SurrealDB server, and you back it up with SurrealDB's own tooling rather
than by copying a UAR directory.

```bash
# Logical export of the UAR namespace/database (default ns=uar, db=uar):
surreal export \
  --endpoint http://127.0.0.1:8000 \
  --username "$SURREAL_USER" --password "$SURREAL_PASS" \
  --namespace uar --database uar \
  uar-surreal-$(date +%Y%m%d).surql

# Restore:
surreal import \
  --endpoint http://127.0.0.1:8000 \
  --username "$SURREAL_USER" --password "$SURREAL_PASS" \
  --namespace uar --database uar \
  uar-surreal-20260711.surql
```

The namespace/database default to `uar`/`uar` but can be overridden with
`UAR_PERSISTENCE__SURREAL_NS` / `UAR_PERSISTENCE__SURREAL_DB` — export/import the
same names you configured. In the prod compose stack the SurrealDB container
persists to its own volume (`surreal_data_prod`, `rocksdb:/data/surreal.db`); a
cold volume copy of that container's data is an alternative to a logical export,
using the same stop-copy-start discipline.

## PostgreSQL

When `persistence.provider = postgres`, use standard PostgreSQL backup tooling
against the configured `database_url`:

```bash
# Backup (pg_dump can run against a live database):
pg_dump "postgres://uar:changeme@localhost:5432/uar" -Fc -f uar-$(date +%Y%m%d).dump

# Restore into a fresh database:
pg_restore -d "postgres://uar:changeme@localhost:5432/uar" --clean uar-20260711.dump
```

Because UAR uses vector columns (pgvector), ensure the `vector` extension exists
in the target database before restoring, and keep `vector_dimension` unchanged
across backup and restore.

## Optional companion state

- **Uploaded files** live under `file_processing.upload_dir`
  (`UAR_FILE_PROCESSING__UPLOAD_DIR`, `/uploads` in the prod compose stack).
  Back these up alongside the datastore if you rely on previously uploaded
  documents.
- **Redis** (`external_cache_enabled = true`) holds cache/session data only and
  is safe to lose — it rebuilds from the primary datastore.
- **Agent memory** (`UAR_MEMORY__ENABLED=true`) uses its own SurrealKV path
  (`memory.db_path`, default `./data/memory.db`) in embedded mode, or the same
  remote Surreal cluster otherwise. Back it up with the matching method above.
