---
sidebar_position: 7
title: Troubleshooting
---

# Troubleshooting

Common boot and runtime problems, with the exact fix for each. Each section
names the symptom you will see in the logs or in an HTTP response.

## Embedded datastore cannot be opened

**Symptom** — the process exits while opening `surrealkv://./data/uar.db`, or
reports that the datastore path is not writable.

**Cause** — packaged binaries use that embedded path by default. The service
account may not own the working directory, or the deployment intended to use a
different persistent volume.

**Fix** — make the default data directory writable, or explicitly choose a
writable embedded location:

```bash
UAR_PERSISTENCE__PROVIDER=surreal
UAR_PERSISTENCE__DATABASE_URL=surrealkv:///var/lib/uar/data/uar-db
```

Or point at the bundled example file:

```bash
CONFIG_FILE=config.embedded.yaml cargo run
```

## Server exits at boot: "no API key provided"

**Symptom** — the server fails to start (exit code 1) and logs
`Server error: authentication failed: no API key provided and environment
variable OPENAI_API_KEY is not set` (or the equivalent for your provider).

**Cause** — no key was resolved for the provider named in `UAR_LLM__MODEL`. UAR
resolves the key in this order: explicit `llm.api_key`
(`UAR_LLM__API_KEY` / `LLM_API_KEY`) → `api_key_env` indirection → legacy
`LLM_API_KEY` → provider shortcut key. If none is present the key is empty.

**Fix** — provide a key for the active provider:

```bash
UAR_LLM__MODEL=openai/gpt-4o
OPENAI_API_KEY=sk-...            # or UAR_LLM__API_KEY=sk-...
```

For local providers, no key is needed — set the base URL instead:

```bash
UAR_LLM__MODEL=ollama/llama3.2
UAR_LLM__BASE_URL=http://localhost:11434
```

## Boot fails: RocksDB `LOCK` "already locked" / "IO error: lock"

**Symptom** — startup fails opening the embedded datastore with a lock error
mentioning `LOCK` on the data directory ("already held by process", "Resource
temporarily unavailable", or similar).

**Cause** — the embedded SurrealKV/RocksDB engine takes an **exclusive** lock on
its data directory. Another UAR process is already using that directory, or a
previous process did not release the lock cleanly.

**Fix**

1. Ensure only one UAR process points at that `database_url` path:

   ```bash
   ps aux | grep universal-agent-runtime
   # stop any stray instance
   ```

2. In Docker, make sure two containers are not mounting the same datastore
   volume at the same path.
3. If no process is running but the lock persists after an unclean shutdown,
   stop UAR fully and start a single instance. Never copy or open the datastore
   directory while the server is live — see
   [Backup and Restore](./backup-and-restore).

## Boot fails: `postgres` provider requires the `postgres-backend` feature

**Symptom** — with `UAR_PERSISTENCE__PROVIDER=postgres`, boot fails with a
message that `persistence.provider = 'postgres'` requires the `postgres-backend`
Cargo feature, and that this binary was built with embedded SurrealDB only.

**Cause** — the default binary/image is built without PostgreSQL support.

**Fix** — either rebuild/redeploy with the `postgres-backend` feature enabled
(use the Postgres compose file / image variant), or switch to SurrealDB:

```bash
UAR_PERSISTENCE__PROVIDER=surreal
UAR_PERSISTENCE__DATABASE_URL=rocksdb://./data/uar-db
```

## HTTP 401 Unauthorized on API calls

**Symptom** — requests return `401` even though the server is up.

**Cause** — `security.jwt_required` is `true` (the default) and the request has
no valid JWT.

**Fix** — three options, in order of preference for local development:

1. Run [`uar-jwt-proxy`](./dev-tools/intro.md) in front of UAR — it mints and
   injects a valid JWT into every request automatically, so you never have to
   handle a token or disable auth checking.
2. Send a valid `Authorization: Bearer <jwt>` token signed with
   `UAR_SECURITY__JWT_SECRET` yourself.
3. For trusted local development only, disable the requirement entirely
   (this also removes auth for any other client that connects directly):

```bash
UAR_SECURITY__JWT_REQUIRED=false   # equivalently: JWT_REQUIRED=false or --jwt-required=false
```

A related case: mutating `/api/uar/settings`
(`PUT`/`POST`/`DELETE`) returns an auth error unless the `X-UAR-Admin-Key`
header is present. Send the header, or set
`UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED=false` for local dev.

## Port already in use / server unreachable

**Symptom** — boot fails with an "address already in use" error, or nothing is
listening on the port you expect.

**Cause** — another process holds the port, or UAR is bound to a different port
than you are connecting to. Defaults differ by config: the compiled default is
`1906`; configuration files and deployment manifests should use the same default.

**Fix**

1. Confirm the effective port. It is set (highest priority first) by `--port` /
   `PORT`, then `UAR_SERVER__PORT`, then `server.port` in your config file.
2. Find and stop whatever holds it, or choose a free port:

   ```bash
   lsof -i :1906            # who has the port?
   UAR_SERVER__PORT=3010 cargo run
   ```

3. In Docker, check the host-side port mapping (`-p 1906:1906`) and that the
   container's `UAR_SERVER__PORT` matches the container-side port.

## Configuration override "not taking effect"

**Symptom** — a value you set is being ignored.

**Cause** — a higher-priority source is shadowing it. Precedence is: CLI args >
`UAR_*__*` env > legacy `LLM_*` env > provider shortcut keys > `config.yaml` >
defaults.

**Fix** — check for the same key at a higher tier. For example, a `--llm-model`
CLI arg or a `LLM_MODEL` env var overrides `llm.model` from `config.yaml`. UAR
logs the fully-resolved configuration at startup (secrets redacted) — read that
line to see the effective values.

## Vector dimension mismatch

**Symptom** — errors writing or querying embeddings, or nonsensical similarity
results after changing the embedding model.

**Cause** — `persistence.vector_dimension` no longer matches the model that
produced the stored vectors (e.g. `1536` for OpenAI `text-embedding-3-small`
vs `384` for BGE-small).

**Fix** — set `UAR_PERSISTENCE__VECTOR_DIMENSION` to your model's dimension. If
you change embedding models, re-embed existing content rather than mixing
dimensions in one datastore.

## Getting more detail

Raise log verbosity to see the full boot sequence and per-request diagnostics:

```bash
RUST_LOG=debug ./target/release/universal-agent-runtime
# or, in the compose file: RUST_LOG=debug
```

Secrets (API keys, JWT secret, Surreal password) are redacted in logs by design.
