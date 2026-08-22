# Local Development Tools

Tooling in `tools/` that supports local development but is not part of the
distributed server/sidecar product. Each tool is its own workspace member —
build and install it independently of the main `universal-agent-runtime`
binary.

## `uar-jwt-proxy`

**What it is** — a local reverse proxy (`tools/uar-jwt-proxy`) that mints a
fresh HS256 JWT and injects it into every request and WebSocket connection it
forwards to a running UAR instance, so a browser session (or `curl`, or any
HTTP client) never has to hold, refresh, or submit a token itself.

**When to use it** — for local development and manual testing against a UAR
instance that has `security.jwt_required: true` (the default). It removes the
need to either mint a token by hand for every request or set
`UAR_SECURITY__JWT_REQUIRED=false` (which disables auth checking entirely,
including for other clients that connect directly to UAR's real port).

**What it is not** — a production authentication gateway. It has no TLS, no
rate limiting, and mints an admin-role token for any client that can reach its
listening port. Its own `Cargo.toml` description says "Local dev reverse
proxy" for exactly this reason — never expose its port beyond `127.0.0.1`.

### Build and install

```bash
# From the repo root -- it's already a Cargo workspace member
cargo install --path tools/uar-jwt-proxy --locked
```

This installs `uar-jwt-proxy` to `~/.cargo/bin/`.

### Run it

With no arguments it auto-discovers the same `config.yaml` UAR itself would
use (`$CONFIG_FILE`, then `./config.yaml`, then `~/.uar/config.yaml`) and
reads `security.jwt_secret`, optional `security.jwt_issuer`/`jwt_audience`, and
`server.host`/`server.port` from it:

```bash
uar-jwt-proxy
# uar-jwt-proxy ready: listen=127.0.0.1:8088 upstream=http://127.0.0.1:1906 ...
```

Point your browser or client at `http://127.0.0.1:8088` instead of UAR's real
port — every request that arrives at UAR already carries a valid
`Authorization: Bearer <jwt>` header.

### Options

| Flag | Env var | Default | Description |
|---|---|---|---|
| `--listen` | `PROXY_LISTEN` | `127.0.0.1:8088` | Address the proxy itself binds to |
| `--config` | `CONFIG_FILE` | (auto-discovered) | Path to a UAR `config.yaml` |
| `--upstream` | `PROXY_UPSTREAM` | from config, else `http://127.0.0.1:1906` | UAR base URL to forward to |
| `--secret` | `UAR_SECURITY__JWT_SECRET` | from config's `security.jwt_secret` | HS256 signing secret — must match the secret the target UAR instance was started with |
| `--issuer` | `UAR_SECURITY__JWT_ISSUER` | from config's `security.jwt_issuer` | Optional `iss` claim — must match the target UAR configuration |
| `--audience` | `UAR_SECURITY__JWT_AUDIENCE` | from config's `security.jwt_audience` | Optional `aud` claim — must match the target UAR configuration |
| `--sub` | `PROXY_JWT_SUB` | `dev` | `sub` claim baked into the minted token |
| `--name` | `PROXY_JWT_NAME` | `Local Dev` | `name` claim |
| `--roles` | `PROXY_JWT_ROLES` | `admin,user` | Comma-separated `roles` claim |
| `--ttl-secs` | `PROXY_JWT_TTL_SECS` | `3600` | Token lifetime in seconds (a fresh token is minted per request, so this mainly bounds clock-skew tolerance) |

### Troubleshooting

- **`JWT secret not provided`** at startup — pass `--secret`, set
  `UAR_SECURITY__JWT_SECRET`, or ensure `security.jwt_secret` is present in the
  config file the proxy discovered (check the `config = ...` field in its
  startup log line).
- **401s still coming through the proxy** — the proxy's signing secret must be
  the *same* `security.jwt_secret` the target UAR instance is running with;
  a mismatch produces a token UAR itself will reject.
- **WebSocket/SSE streams don't connect** — confirm you're pointing the client
  at the proxy's port (`8088` by default), not UAR's real port directly; the
  proxy bridges both plain HTTP and WebSocket upgrades, but only for traffic
  that actually reaches it.

See also [Troubleshooting → HTTP 401 Unauthorized on API calls](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/website/docs/troubleshooting.md#http-401-unauthorized-on-api-calls) for the underlying JWT requirement this tool works around.
