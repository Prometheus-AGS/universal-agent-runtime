---
sidebar_position: 2
title: Installation
---

# Installation

UAR can be run three ways: with Docker Compose (recommended for a full stack),
from a prebuilt container image, or by building the binary from source. In every
case there is a small set of configuration values needed before the runtime can
serve model requests.

## What is required at boot

Before you start, decide two things:

1. **Persistence** — packaged binaries default to embedded SurrealDB at
   `surrealkv://./data/uar.db`, so a clean installation starts without an
   external database. Override both settings when you need a different path or
   a remote backend:

   ```bash
   UAR_PERSISTENCE__PROVIDER=surreal
   UAR_PERSISTENCE__DATABASE_URL=rocksdb://./data/uar-db
   ```

2. **A provider API key** — the LLM layer needs a key for the provider named in
   `UAR_LLM__MODEL`. Supply it via `UAR_LLM__API_KEY` or the matching provider
   shortcut (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GROQ_API_KEY`, …). Local
   providers (`ollama/*`, `lmstudio/*`) need no key. **Without a resolvable key
   for the configured default model the server exits at startup (exit code 1)
   with `authentication failed: no API key provided …`** — the key is required
   to boot, not just to serve requests.

See the **[Configuration reference](./configuration/intro.md)** for the full list.

## Option 1 — Docker Compose (recommended)

The repository ships several compose files:

| File | Stack |
|---|---|
| `docker-compose.prod.yaml` | App + SurrealDB (application persistence) + Redis. |
| `docker-compose.prod.postgres.yaml` | Preview source-build stack: app + PostgreSQL persistence + Surreal (optional memory). |
| `docker-compose.dev.yaml` | Local development stack. |

Steps:

```bash
# 1. Configure
cp .env.example .env
# Edit .env — at minimum set:
#   UAR_LLM__MODEL, UAR_LLM__API_KEY (or a provider shortcut key)
#   UAR_SECURITY__JWT_SECRET (openssl rand -base64 64)
#   SURREAL_USER / SURREAL_PASS  (for the surreal service)

# 2. Bring up the stack (SurrealDB persistence)
docker compose -f docker-compose.prod.yaml --env-file .env up -d

# 3. Check health
curl -sf http://localhost:1906/healthz
```

The compose stack sets persistence for you — the app talks to the SurrealDB
service over the network:

```yaml
UAR_PERSISTENCE__PROVIDER: surreal
UAR_PERSISTENCE__DATABASE_URL: http://surreal:8000
UAR_PERSISTENCE__SURREAL_USER: ${SURREAL_USER:-root}
UAR_PERSISTENCE__SURREAL_PASS: ${SURREAL_PASS:-changeme}
```

Ports (host defaults): app `1906`, gRPC `50051`, SurrealDB `8000`, Redis `6379`.
Persistent data lives in named Docker volumes (`surreal_data_prod`,
`uar_data_prod`, `uar_uploads_prod`, `redis_data_prod`).

For PostgreSQL instead of SurrealDB, the Preview
`docker-compose.prod.postgres.yaml` is a source-build reference. PostgreSQL
requires a binary built with `postgres-backend`; it is not included in the
Stable prebuilt image.

## Option 2 — Prebuilt binary / container image

The published image is
`ghcr.io/prometheus-ags/universal-agent-runtime:<version>`. Pin a released tag
or the signed digest from `release-manifest.json`; do not deploy `latest` in
production. To run it standalone with embedded SurrealDB:

```bash
docker run -d --name uar \
  -p 1906:1906 \
  -v uar_data:/var/lib/uar \
  -e UAR_PERSISTENCE__DATABASE_URL=surrealkv:///var/lib/uar/data/uar-db \
  -e UAR_LLM__MODEL=openai/gpt-4o \
  -e OPENAI_API_KEY=sk-... \
  -e UAR_SECURITY__JWT_SECRET="$(openssl rand -base64 64)" \
  -e UAR_SECURITY__JWT_REQUIRED=false \
  ghcr.io/prometheus-ags/universal-agent-runtime:v1.0.0
```

Mount a volume at the datastore path (here `/var/lib/uar`) so the embedded database
survives container restarts.

## Option 3 — Build from source

Prerequisites: **Rust** (latest stable, edition 2024), **Node.js 22**, and
**pnpm 10.33.0**. PostgreSQL or a remote SurrealDB instance is optional.

```bash
# 1. Clone
git clone https://github.com/Prometheus-AGS/universal-agent-runtime.git
cd universal-agent-runtime

# 2. Configure
cp .env.example .env
#   Set UAR_LLM__MODEL + a provider key, and persistence provider/URL.

# 3. Install the locked frontend dependencies and build assets
pnpm install --frozen-lockfile
pnpm -C frontend install --frozen-lockfile
pnpm -C frontend --filter @prometheus-ags/prometheus-entity-management build
pnpm build

# 4. Run (embedded persistence via the example config)
CONFIG_FILE=config.embedded.yaml cargo run --features server-full
#   → http://localhost:1906   (config.embedded.yaml uses port 1906)

# …or a release build
cargo build --release --features server-full
./target/release/universal-agent-runtime
```

`config.embedded.yaml` uses `surrealkv://./data/uar-dev-db`, binds to
`127.0.0.1:1906`, and disables JWT for local development.

If you're running against a config with `jwt_required: true` instead, see
[Dev Tools → uar-jwt-proxy](./dev-tools/intro.md) — it mints and injects a
valid JWT automatically rather than requiring you to disable auth.

## First-run checklist

1. **`.env` created** from `.env.example`.
2. **`UAR_LLM__MODEL`** set to a `provider/model` string.
3. **A provider key** present (`UAR_LLM__API_KEY` or a `*_API_KEY` shortcut), or
   a local `base_url` for Ollama/LM Studio.
4. **Persistence** confirmed: accept the packaged embedded default or set
   `UAR_PERSISTENCE__PROVIDER` + `..._DATABASE_URL`. The selected datastore is
   writable and reachable.
5. **`UAR_SECURITY__JWT_SECRET`** set to a strong random value (any deployment
   reachable off-localhost).
6. **`vector_dimension`** matches your embedding model (`1536` for OpenAI
   `text-embedding-3-small`, `384` for BGE-small).
7. **Health check passes**: `curl -sf http://<host>:<port>/healthz`.
8. **A test chat request** returns a streamed response (see
   [API reference](./api-reference)).

If any step fails, see **[Troubleshooting](./troubleshooting)**.
