---
sidebar_position: 2
title: Installation
---

# Installation

UAR can be run three ways: with Docker Compose (recommended for a full stack),
from a prebuilt container image, or by building the binary from source. In every
case there is a small set of **required** configuration values without which the
server will not start or will not be able to serve requests.

## What is required at boot

Before you start, decide two things:

1. **Persistence** — `persistence.provider` and `persistence.database_url`
   have **no compiled defaults**. If they are missing, configuration loading
   fails and the process exits (exit code `1`) with a "missing field" error.
   The simplest choice is embedded SurrealDB:

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

See the **[Configuration reference](./configuration)** for the full list.

## Option 1 — Docker Compose (recommended)

The repository ships several compose files:

| File | Stack |
|---|---|
| `docker-compose.prod.yaml` | App + SurrealDB (application persistence) + Redis. |
| `docker-compose.prod.postgres.yaml` | App + PostgreSQL persistence + Surreal (optional memory). |
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
curl -sf http://localhost:3000/healthz
```

The compose stack sets persistence for you — the app talks to the SurrealDB
service over the network:

```yaml
UAR_PERSISTENCE__PROVIDER: surreal
UAR_PERSISTENCE__DATABASE_URL: http://surreal:8000
UAR_PERSISTENCE__SURREAL_USER: ${SURREAL_USER:-root}
UAR_PERSISTENCE__SURREAL_PASS: ${SURREAL_PASS:-changeme}
```

Ports (host defaults): app `3000`, gRPC `50051`, SurrealDB `8000`, Redis `6379`.
Persistent data lives in named Docker volumes (`surreal_data_prod`,
`uar_data_prod`, `uar_uploads_prod`, `redis_data_prod`).

For PostgreSQL instead of SurrealDB, use `docker-compose.prod.postgres.yaml`
(sets `UAR_PERSISTENCE__PROVIDER=postgres` with `vector_dimension: 1536`). Note
the Postgres backend requires an image built with the `postgres-backend` Cargo
feature.

## Option 2 — Prebuilt binary / container image

The published image is `tribehealth/universal-agent-runtime:latest`. To run it
standalone (embedded SurrealDB, single container):

```bash
docker run -d --name uar \
  -p 3000:3000 \
  -v uar_data:/data \
  -e UAR_PERSISTENCE__PROVIDER=surreal \
  -e UAR_PERSISTENCE__DATABASE_URL=rocksdb:///data/uar-db \
  -e UAR_LLM__MODEL=openai/gpt-4o \
  -e OPENAI_API_KEY=sk-... \
  -e UAR_SECURITY__JWT_SECRET="$(openssl rand -base64 64)" \
  -e UAR_SECURITY__JWT_REQUIRED=false \
  tribehealth/universal-agent-runtime:latest
```

Mount a volume at the datastore path (here `/data`) so the embedded database
survives container restarts.

## Option 3 — Build from source

Prerequisites: **Rust** (latest stable, edition 2024), **Bun** (frontend
assets), and — unless you use embedded SurrealDB — a **PostgreSQL** (with
pgvector) or **SurrealDB** instance.

```bash
# 1. Clone
git clone https://github.com/Prometheus-AGS/universal-agent-runtime.git
cd universal-agent-runtime

# 2. Configure
cp .env.example .env
#   Set UAR_LLM__MODEL + a provider key, and persistence provider/URL.

# 3. Build frontend assets
bun install && bun run build

# 4. Run (embedded persistence via the example config)
CONFIG_FILE=config.embedded.yaml cargo run
#   → http://localhost:1906   (config.embedded.yaml uses port 1906)

# …or a release build
cargo build --release
./target/release/universal-agent-runtime
```

`config.embedded.yaml` uses `surrealkv://./data/uar-dev-db`, binds to
`127.0.0.1:1906`, and disables JWT for local development.

## First-run checklist

1. **`.env` created** from `.env.example`.
2. **`UAR_LLM__MODEL`** set to a `provider/model` string.
3. **A provider key** present (`UAR_LLM__API_KEY` or a `*_API_KEY` shortcut), or
   a local `base_url` for Ollama/LM Studio.
4. **Persistence** chosen: `UAR_PERSISTENCE__PROVIDER` + `..._DATABASE_URL`.
   The datastore directory (embedded) or the DB service (remote) is reachable.
5. **`UAR_SECURITY__JWT_SECRET`** set to a strong random value (any deployment
   reachable off-localhost).
6. **`vector_dimension`** matches your embedding model (`1536` for OpenAI
   `text-embedding-3-small`, `384` for BGE-small).
7. **Health check passes**: `curl -sf http://<host>:<port>/healthz`.
8. **A test chat request** returns a streamed response (see
   [API reference](./api-reference)).

If any step fails, see **[Troubleshooting](./troubleshooting)**.
