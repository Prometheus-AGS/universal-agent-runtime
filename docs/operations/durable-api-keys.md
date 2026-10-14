# Durable API-key authority

The trusted runtime host persists API-key records with Argon2id hashes, authority
version, issuer, tenant, subject, roles, creation time, expiry, and revocation.
The raw secret is returned once on creation. Management lists contain metadata
only. Provider credentials remain a separate capability and storage namespace.

Startup reuses the configured persistence connection: local `surrealkv://` and
networked SurrealDB use `auth_api_keys`; Postgres uses the same table name in its
configured database. Startup installs the additive migration. The operator must
provide durable storage for a networked SurrealDB server; a WebSocket URL alone
does not prove durability. Use the release's pinned server and SDK versions.
Both required release profiles need independent acceptance evidence.

Explicit `persistence.provider: memory` remains local development mode and requires
the existing `in-memory-backend` build feature. Its keys disappear when the process
ends. Explicit local SurrealDB memory endpoints are also volatile. Remote security
profiles reject memory authority rather than promoting it to production storage.
No database failure switches a configured durable service to memory.

## API

All management routes require a verified caller. The exact issuer/tenant/subject
scope owns the record; an equal subject under a different tenant or issuer does
not own it. A configured exact API-key administrator may manage keys only within
the same issuer and tenant scope. Requested roles can only attenuate the caller's
delegable roles; reserved `admin` and `host-session` roles are rejected.

| Method | Route | Outcome |
| --- | --- | --- |
| POST | `/api/uar/auth/keys` | 201 with `metadata` and one-time `raw_key`; accepts `name`, optional `roles`, optional positive `expires_in_secs` |
| GET | `/api/uar/auth/keys` | 200 with owned key metadata, including revoked keys |
| DELETE | `/api/uar/auth/keys/{id}` | 200 after committed revocation; 404 for absent or foreign records |
| POST | `/api/uar/auth/exchange` | `X-API-Key` or JSON `api_key`; 200 with Bearer token and actual `expires_in` |

Exchange tokens contain signed `uar_credential_kind: api_key` and
`uar_api_key_id` provenance. Their lifetime is capped at one hour and at the key's
expiry. HTTP accepts raw keys through `X-API-Key`; gRPC accepts the exchanged
Bearer token. Every new HTTP or gRPC request re-reads authoritative key validity,
including tokens issued before another runtime revokes the key. Current owner,
roles, and expiry must agree with the signed token. There is no cache of allow
decisions. API-derived sidecar delegation grants also recheck their captured key
authority on each newly authenticated request. Host-principal grants keep their
independent launch authority. Revocation applies to new admission; this change does not cancel work
that was already admitted.

Invalid, revoked, or expired authority returns HTTP 401 / gRPC Unauthenticated.
Missing legacy key references return explicit reissue outcomes. Durable storage
failure returns HTTP 503 / gRPC Unavailable for key authentication, exchange, and
management, with no backend payload, plaintext, or hash in the response. Ordinary
issuer JWT signature verification keeps its existing behavior. Exchanging local
keys into tokens accepted by an external JWKS is unsupported and returns HTTP 501.

A severed database transport was observed to leave an authority query pending
until the ordinary HTTP timeout returned 408. Durable key reads and mutations now
have a separate bounded wait derived from the existing
`resilience.request_timeout_ms`: `min(5000ms, max(request_timeout_ms, 1000ms) / 8)`.
The default request budget gives 3750ms per authority operation. Direct store
constructors use that same default unless the host supplies its request budget.
The division leaves deliberate headroom for chained operations; raw-key
authentication and revocation together currently perform five authority reads or
mutations. It does not guarantee that every route completes before its global
timeout, since hashing and other route work also consume time. The global timeout
and its disable setting are unchanged. Authority waits retain their finite cap
even when ordinary request timeouts are disabled, and gRPC uses the same bound.
Elapsed authority waits return the existing sanitized unavailable outcome.

A timed-out mutation has no confirmed result and may already have committed.
After storage recovers, authenticate with an issuer credential and inspect owned
metadata to reconcile creation or revocation. A creation whose raw key was never
received requires authenticated reissue; its plaintext cannot be recovered from
storage. Do not automatically replay a timed-out mutation or assume that timing
out rolled it back. Retain revoked rows and never restore authority from a stale
record. Tokio's timeout bounds waiting on a yielding future; it is not proof of
database rollback. [Tokio 1.53.1 timeout documentation](https://docs.rs/tokio/1.53.1/tokio/time/fn.timeout.html).

The public `authenticate_bearer_context` helper authenticates ordinary issuer JWTs
for identity-only host adapters. It has no storage binding and returns 503 for
key-derived tokens. An adapter admitting such tokens must bind `ApiKeyService`
and call `validate_admission` after signature verification; the production HTTP
and gRPC services do this. A signature check by itself is not current API-key
authority.

## Migration and rollback

Existing volatile keys cannot be recovered after their issuing process ends. This
release selects **authenticated reissue**, with no hash import or automatic owner
inference. Authenticate using a verified issuer JWT, create a new durable key
through the management route, update the client securely, then retire the old
volatile runtime. Never save plaintext keys to migration files, logs, command-line
arguments, receipts, or database records. A missing-reference legacy exchange JWT
cannot bootstrap the migration; use the original verified issuer credential.

Revocation updates retain the record, and creation is insert-only. Neither reissue
nor repeated insertion can replace a revoked record with active authority. Keep
the table and migrations during rollback; there is no down migration that deletes
revocation history. Roll back only to a build that enforces durable revocation.
If that is unavailable, disable API-key admission until a compatible build is
restored. Restoring an older database snapshot without reconciling later revocations
can revive authority and is outside the safe rollback path.

The uncomfortable thing: the service cannot make a memory-only older binary
honor durable revocations or prove that an operator's remote server persists them.
A row existing in one process is also insufficient evidence for another host's
admission. Use the actual process gate below before accepting a release.

## Local acceptance gate

The target `tests/durable_api_keys_integration.rs` starts actual runtime binaries
with clean working directories and explicit fixture configuration. It requires
the production gRPC listener on ordinary runtime launches. A separate actual
`uar-sidecar` process uses `--config` and `--port`, receives its launch token only
over stdin, and keeps that pipe open; the supervised sidecar intentionally exposes
one guarded HTTP listener. Both launchers come from `src/main.rs` and
`src/bin/uar-sidecar.rs` through the existing Cargo binary targets. No external
model call or real provider secret is used; startup uses an unused synthetic
provider placeholder pointed at loopback.
Run it serially at task 2, after the capability slice is complete:

```bash
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test durable_api_keys_integration -- --include-ignored --test-threads=1
```

Supply these through a private environment source, never an echoed command line:

| Environment variable | Fixture requirement |
| --- | --- |
| `UAR_API_KEY_TEST_SURREAL_URL` | Dedicated durable SurrealDB `ws://` endpoint, non-TLS loopback transport |
| `UAR_API_KEY_TEST_SURREAL_USER` | Test-only root username |
| `UAR_API_KEY_TEST_SURREAL_PASS` | Test-only root password |
| `UAR_API_KEY_TEST_POSTGRES_URL` | Dedicated empty Postgres database using the existing pgvector image, non-TLS loopback transport |

The local test owns a temporary SurrealKV database. The shared test creates a
unique namespace; Postgres must be an isolated database because startup runs the
runtime's existing migrations. Tests own TCP proxies to disconnect real database
channels after successful admissions. Assertion output contains statuses and
scenario names only; retained process logs redact observed keys, tokens, and hash
lines. Do not publish the private environment, generated config files, or DB data.

Coverage includes authenticated creation, raw-key and exchange-token use after
restart, two-runtime revocation, both runtimes restarting, HTTP/gRPC expiry and
outage denial, equal-subject tenant/issuer isolation, unauthorized management,
current-role checks, actual sidecar API-derived grant revocation/outage denial
with host-grant preservation, insert-only tombstones, hashed-only storage, and explicit
volatile development mode. The Postgres scenario qualifies modified API-key paths
only; it does not certify observer/checkpoint parity. Test authorship is not a
passing runtime result; execution receipts belong to the subsequent task.
