## Context

See `proposal.md` for why. Facts that shape the design:

- `SurrealDbProvider::new(url, user, pass, ns, db)` (`src/uar/persistence/providers/surreal.rs:35-92`): `any::connect(&endpoint)` (`:45`); for `ws/wss/http/https` endpoints (`is_server_endpoint`, `:160-166`) it signs in with `Root`, defaulting to `root`/`root` (`:47-56`); then `use_ns(ns or "uar").use_db(db or "uar")` (`:59-61`); then four schema scripts (`:64-86`). It logs the endpoint in full (`:43`) and the username (`:55`).
- The server builds the provider and turns any error into a fatal startup error (`src/server.rs:606-626`); the same connection feeds the design-system store, compiler storage, agent registry, credential store and live-query bus (`:628-665`). A fallback therefore has to happen at this one construction point; everything downstream just receives a different `Surreal<Any>`.
- `persistence_info_handler` derives `mode` from the configured URL prefix and returns `database_url` verbatim (`server.rs:3309-3325`); `sync_stream_handler` uses the same prefix test (`:3334-3349`).
- UAR's schema work needs only `DEFINE TABLE`, `DEFINE FIELD`, `DEFINE INDEX` and `.live()` selects (counted across `src/uar/persistence`, `src/uar/realtime`, `src/uar/security/credentials`, `src/uar/a2ui`, `src/uar/compiler` and `migrations/surrealdb`; no `DEFINE USER/ACCESS/NAMESPACE/DATABASE/ANALYZER/FUNCTION`, one `use_ns` call). That is within what an `EDITOR` can do; it has not been run against a live 3.2.4 server this session.
- The SDK has `Namespace { namespace, username, password }` and `Database { namespace, database, username, password }` credentials (`surrealdb-3.2.4/src/opt/auth.rs:37-69`).
- The memory subsystem opens its own embedded SurrealKV at `memory.db_path` (`src/uar/memory/service.rs:115-119`); it is not affected by this change (and `sidecar-launch-security` turns memory off in sidecar mode).
- Sidecar mode is known to the server only after `sidecar-launch-security` (it adds the guard; today `UAR_SIDECAR` is set but never read). The "never root in sidecar mode" rule uses that signal, so this change depends on that one for that rule only.

## Goals / Non-Goals

**Goals:**
- Least-privilege remote sign-in usable on SurrealDB 3.2.4.
- The desktop runtime starts with Docker stopped, in under 10 s, on macOS, Windows and Linux (review round 3 warning 6 adds Linux).
- Fallback data is visibly local-only and never silently merged.

**Non-Goals:**
- The explicit "move local-only data to the remote" action. This change only makes the data findable and marked.
- Switching stores while running, or reconnect-and-sync.
- Provisioning the shared SurrealDB (it belongs to the Docker services in prometheus-skills-mini). The statements are given below.
- Record-level users, `DEFINE ACCESS`, SCRAM, access `CONTEXT`, or anything else that needs SurrealDB ≥ 3.3.0.
- Passing the database password through stdin. For the-boss it arrives through configuration or environment (Risk 3).

## Decisions

### 1. Auth level as explicit configuration, root stays the default
`persistence.surreal_auth_level: root | namespace | database`. Keeping `root` as the default preserves every existing deployment (A-7); the-boss's sidecar configuration sets `namespace`. At `namespace`/`database`: require `surreal_user`, `surreal_pass`, `surreal_ns` (and `surreal_db` for `database`); sign in with the SDK's `Namespace`/`Database` credentials; never apply the `root`/`root` default. The `root`/`root` default itself is left alone for `root` level — removing it would break the documented local development setup, and it is not in the sidecar's path.

Namespace level, not database level, is what D4 names. It lets UAR create database `uar` inside namespace `uar` if it does not exist. Database level is supported for operators who pre-create the database and want tighter scope.

Provisioning, run once as root on the shared 3.2.4 server (syntax to be confirmed by task 2.2 against the real server):
```sql
DEFINE NAMESPACE IF NOT EXISTS uar;
USE NS uar;
DEFINE DATABASE IF NOT EXISTS uar;
DEFINE USER IF NOT EXISTS uar_app ON NAMESPACE PASSWORD $generated ROLES EDITOR;
```

### 2. Sidecar refuses root for a remote
If the sidecar guard is present (from `sidecar-launch-security`) and the URL is remote, auth level `root` or absent is a startup error. This turns D4's "never root" into a machine check rather than a convention.

### 3. Remote schemes
`ws://`/`wss://` are the documented schemes. `http://`/`https://` stay passed through to the SDK unchanged (a build that does enable `protocol-http` keeps working); when the connect error says the protocol is unsupported, the startup error adds "use `ws://host:port`". `config.remote.surreal.yaml` switches to `ws://127.0.0.1:8000` and namespace-level credentials in comments. Rejected: silently rewriting `http` to `ws` (surprising; hides config drift). Rejected: enabling `protocol-http` (adds an HTTP client path UAR does not need; D4 asked for ws).

### 4. Fallback decision at the single construction point
`remote_fallback: none | embedded` (default `none`), `remote_connect_timeout_ms` (default 3000), `fallback_database_url` (default `surrealkv://./data/uar-local-only.db`; the-boss sets an absolute path under its userData).

At `server.rs:612`: wrap `any::connect` in `tokio::time::timeout(remote_connect_timeout_ms)`. Classify the result:
- connected → sign in, use_ns/db, schema → any error here is fatal (Decision 4 rationale: a reachable server that refuses us is misconfigured; falling back would split data silently).
- timeout, connection refused, name resolution failure → if `embedded`: open `SurrealDbProvider::new(fallback_database_url, None, None, ns, db)` and continue; else fatal as today.

Classification uses the SDK error kind where it exposes one and falls back to matching the underlying I/O error kind; which variants 3.2.4 exposes for a refused WebSocket was not checked this session, so task 2.6 pins the behavior with a real refused port and a blackholed address.

The rest of startup is unchanged because it receives a `Surreal<Any>` either way. The fallback provider records `FallbackState { active: true, reason, since, configured_url_redacted }` in `AppState` (explicit state, not a global), read by the status handlers.

Time budget: a stopped Docker container gives connection refused immediately; the 3 s timeout bounds the blackhole case; embedded open plus schema scripts must fit in the remainder of 10 s. The embedding backend is already lazy (`server.rs:562-564`). Measured, not assumed: tasks 2.7–2.9.

### 5. Local-only marking is per store, not per record
The whole fallback store is local-only. On first open UAR writes one record `uar_store_meta:store` with `{ local_only: true, created_at, origin: <redacted configured remote host:port/ns/db> }` into the fallback store. Rejected: a `local_only` field on every record — dozens of tables, schema churn, and every record in that store would carry the same value.

What a host sees:
- `GET /api/config/persistence` → `{ provider, mode: "embedded"|"remote" (effective), configured_mode, database_url (userinfo removed), fallback: { active, reason: "remote_unreachable"|"remote_timeout"|null, since } }`.
- `GET /api/uar/persistence/local-only` → `{ present, active, sessions: [{ id, title, created_at, updated_at }] }`. When the fallback store is not active, the handler opens it read-only-in-practice (queries only), lists sessions, and closes it. The "session" read is the same record type the-boss's D1 driver creates through UAR (UAR `session_id`); the exact table was not pinned this session and is fixed by task 2.10.
- On a start that connects to the remote while a fallback store with sessions exists, UAR logs `persistence.local_only_store_present sessions=N` once.

Nothing reads the fallback store into the remote or vice versa. The fallback store is never deleted by UAR.

### 6. Redaction
`database_url` userinfo is removed in the status response and in the connect log line (`surreal.rs:43`). The username log (`:55`) stays; usernames are not secrets.

### 7. SurrealKV on Windows
The fallback runs SurrealKV on Windows. The D4 research recorded that SurrealKV documents limited, not thread-safe file operations on Windows; that documentation was not re-read this session. UAR already uses embedded SurrealKV by default on every platform (`src/config.rs:1288`), so the risk is not new, but fallback makes Windows desktops depend on it. Task 2.12 is a soak test on Windows: concurrent sessions writing through the sidecar, then an unclean kill and restart, then an integrity check. If it fails, the fallback on Windows reopens D4's storage decision (for example, one embedded store per process with serialized writes, or a different engine); it does not get silently accepted.

## Risks / Trade-offs

- [EDITOR cannot do something UAR's schema needs on 3.2.4] → task 2.2 runs UAR's real schema scripts and a live query as the scoped user. If it fails, the missing permission is named and D4's role choice is reopened; UAR does not quietly use root.
- [Namespace isolation assumption false (D4 assumption)] → task 2.3 is the falsifier; a failure reopens D4's storage decision.
- [Database password handed to the sidecar via environment is visible to same-user process inspection] → the credential is scoped to namespace `uar` only, so exposure is bounded to UAR's own data. Moving it to stdin like the launch token is a follow-up if the-boss wants it; not in this change.
- [Split data between remote and fallback confuses users] → the status and local-only endpoints make it visible; the explicit move action is a follow-up and must exist before the-boss ships fallback to users (D4's "explicit user action moves it later").
- [Settings, providers and global MCP servers live in the remote and are missing during fallback] → accepted. For the-boss, D1 passes credentials and tools per run, so runs still work.
- [Timeout misclassification: a slow but healthy remote is treated as down] → 3 s default; configurable; the fallback reason `remote_timeout` is visible.
- [SurrealKV lock held by a crashed process on the fallback path] → existing lock handling (`server.rs:394-441`) applies; a held lock is a startup error, not a second fallback.

## Migration Plan

1. Ship with defaults `surreal_auth_level: root`, `remote_fallback: none` — no behavior change for existing deployments.
2. the-boss's sidecar configuration sets `namespace`, `ws://`, `remote_fallback: embedded`, and an absolute `fallback_database_url`.
3. Provision the `uar` namespace user on the shared Docker SurrealDB (prometheus-skills-mini docker services).
4. Rollback: revert; the fallback store remains on disk untouched; nothing was merged.
