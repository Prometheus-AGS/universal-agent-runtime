---
sidebar_position: 6
title: Upgrade Guide
---

# Upgrade Guide

This guide covers upgrading a self-hosted UAR deployment: pinning versions,
performing the upgrade, checking configuration compatibility, and rolling back
if something goes wrong.

## Supported versions

Security and bug fixes target the latest **1.x** release. The current
security-support matrix (from `SECURITY.md`):

| Version | Supported |
|---|---|
| 1.0.x | ✅ security fixes |
| < 1.0 (unreleased development history) | ❌ upgrade to 1.0 |

Always upgrade to a supported release rather than tracking an arbitrary commit.

## Pin the version you run

Do not deploy from a floating tag in production.

- **Container images**: pin an immutable tag or digest rather than `:latest`.
  For example, deploy `ghcr.io/prometheus-ags/universal-agent-runtime@sha256:<digest>` (or a
  specific released version tag) so a redeploy cannot silently change the binary.
- **From source**: check out a released tag, not `main`:

  ```bash
  git fetch --tags
  git checkout v1.0.0
  ```

- **Git dependencies**: UAR pins several crates to specific commit SHAs or
  release tags in `Cargo.toml` (e.g. `rmcp`, `surreal-memory`, `kreuzberg` at a
  release tag such as `v4.9.8`, `prometheus_parking_lot`). Building a given UAR
  commit reproduces the same dependency set. See
  `docs/DEPENDENCY_MANAGEMENT.md` for the pinning policy and the standard
  operating procedure for bumping a pinned dependency.

## Before you upgrade

1. **Read the release notes / changelog** for the target version, paying
   attention to any called-out breaking changes.
2. **Back up your data.** Follow the
   [Backup and Restore](./backup-and-restore) runbook and confirm the archive is
   valid before touching the running deployment. This backup is your rollback
   path.
3. **Record the current version** (image tag/digest or git SHA) so you can roll
   back to a known-good build.
4. **Diff your configuration** against the new version's `.env.example` and the
   example `config.*.yaml` files to spot any newly required or renamed keys.

## Upgrading a Docker Compose deployment

```bash
# 1. Back up (see the Backup runbook) and note the current image.

# 2. Pull the new pinned image / update the tag in your compose file or .env.
docker compose -f docker-compose.prod.yaml pull app

# 3. Recreate only the app service; the database and Redis keep their volumes.
docker compose -f docker-compose.prod.yaml up -d app

# 4. Verify.
curl -sf http://localhost:1906/healthz
docker compose -f docker-compose.prod.yaml logs -f app   # watch for boot errors
```

The persistence volumes (`surreal_data_prod` / your Postgres volume) are not
recreated, so application data carries across the upgrade.

## Upgrading a from-source / binary deployment

```bash
# 1. Back up.
# 2. Fetch and check out the new tag.
git fetch --tags && git checkout v1.0.1

# 3. Rebuild frontend + backend from locked dependencies.
pnpm install --frozen-lockfile
pnpm -C frontend install --frozen-lockfile
pnpm -C frontend --filter @prometheus-ags/prometheus-entity-management build
pnpm build
cargo build --release --features server-full

# 4. Restart the service against the same persistence configuration.
sudo systemctl restart uar   # or your process manager
curl -sf http://localhost:<port>/healthz
```

## Configuration compatibility

- **New settings** generally arrive with compiled defaults, so existing configs
  keep working. Packaged binaries default to embedded SurrealDB at
  `surrealkv://./data/uar.db`; production deployments should keep persistence
  explicit so upgrades cannot change the intended data path. If a future
  required field is introduced, the server exits on boot with a configuration
  error naming it (see [Troubleshooting](./troubleshooting)).
- **Precedence is stable**: CLI args > `UAR_*__*` env > legacy `LLM_*` env >
  provider shortcut keys > `config.yaml` > defaults. Re-check that an
  environment override you rely on is not being shadowed by a higher-priority
  source after the upgrade.
- **`vector_dimension`** must stay consistent with the embedding model your
  stored vectors were written with. Changing it mid-life invalidates existing
  vector data — plan a re-embed if you change embedding models.
- **Persistence on-disk format**: upgrade the datastore in place with the new
  binary, but keep the provider and `database_url` unchanged. Do not point a new
  major version at a datastore written by an incompatible engine version without
  a tested migration path.

## Rolling back

If the new version fails health checks or misbehaves:

```bash
# Docker: redeploy the previous pinned image/digest.
#   set the old tag in .env / compose, then:
docker compose -f docker-compose.prod.yaml up -d app

# From source: check out the previous tag and rebuild.
git checkout v1.0.0
pnpm build && cargo build --release --features server-full
sudo systemctl restart uar
```

If the upgrade also changed on-disk data in an incompatible way, restore the
pre-upgrade backup (see [Backup and Restore](./backup-and-restore)) **before**
starting the older binary against the datastore. Restore into the same
`database_url` the backup came from.

## Getting help

- Community: GitHub Issues and Discussions (best-effort triage).
- Commercial licensees receive contractual upgrade assistance — see
  `SUPPORT.md` and `LICENSE-COMMERCIAL.md`.
