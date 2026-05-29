## Why

UAR routes all LLM access through liter-llm (provider routing, 142+ providers) and
hydrates a model catalog at build time via `build.rs`. But the API key backing each
request is **process-global**: `ProviderConfig.api_key` in `src/llm/registry.rs` is a
single key per provider, resolved once from CLI/env/config precedence
(`src/config.rs`), and the LLM router has **no `UserContext`** at request time. The
`src/session/encrypted.rs` module that would hold per-user secrets is an explicit
stub ("full implementation pending").

This makes UAR single-tenant only. There is no way for end users to bring their own
provider API keys, store them encrypted at rest, and have requests billed to their
own accounts. That capability is required for multi-tenant SaaS deployment — while
self-hosted/single-tenant operation (one operator key via env) must continue to work
unchanged and with zero configuration.

These two modes are **not a fork**. A scoped credential resolution chain that
terminates in an environment-variable step satisfies both: when no per-user key is
stored, every request falls through to the env step — which is exactly today's
behavior. Per-user keys, when present, simply resolve first.

This change ports the credential subsystem designed on the `origin/feature/providers`
branch (`encryption.rs`, `credential_resolver.rs`, catalog/credential store) onto the
current codebase (liter-llm rc.41, SurrealDB-default), re-authored against current
APIs rather than merged (the branch predates liter-llm and carries regressing deps).

## What Changes

### Encrypted Credential Storage
- Add `CredentialEncryption` (AES-256-GCM, key derived from `CREDENTIAL_ENCRYPTION_KEY`,
  ciphertext stored as `base64(12-byte-nonce ‖ ciphertext)` so identical plaintexts
  produce distinct ciphertexts).
- Add a SurrealDB-backed credential store for per-(scope, provider) encrypted keys.
  Raw keys accepted once on write, encrypted immediately, **never returned** on read.
- Add `aes-gcm = "0.10"` to `Cargo.toml`.

### Scoped Credential Resolution
- Add `CredentialResolver` with a 5-level chain: `session → agent → user → system → env var → None`.
- The terminal `env var` / `system` step delegates to the **existing** registry/config
  key, so single-tenant operation is the zero-config default and current behavior is
  preserved bit-for-bit when no per-user credentials exist.
- `CREDENTIAL_ENCRYPTION_KEY` is required **only** when a per-user key is written or
  read — never for env-only single-tenant operation.

### Request-Path Integration (resolve-then-construct)
- Thread the already-available `UserContext` (`src/uar/api/openai/routes.rs:50`,
  `user_id` used at `routes.rs:94`) into a per-request resolution step at the **route
  layer**.
- The route resolves the credential, then builds/configures the per-request driver
  with the resolved key. On `None`, the orchestrator uses its existing registry/env
  key. **The `Orchestrator::chat_with_history` signature is unchanged** — credential
  concerns stay out of the orchestrator. Blast radius is confined to the route layer.
- Wire `ProviderService` into `AppState` (`src/lib.rs:67`) as
  `Option<Arc<ProviderService>>` — `None` ⇒ pure single-tenant, no behavior change.

### Provider/Credential REST API
- Add `/api/providers` and `/api/models` read endpoints over the existing catalog.
- Add per-user credential CRUD (store / rotate / delete), JWT-protected via the
  existing `src/uar/security/middleware.rs`. Plaintext keys are write-only.

## Capabilities

### New Capabilities
- `provider-credentials`: AES-256-GCM encryption of provider API keys at rest, a
  SurrealDB-backed per-scope credential store, and a JWT-protected CRUD API where raw
  keys are accepted once and never returned.
- `credential-resolution`: A 5-level scoped resolution chain
  (`session → agent → user → system → env`) whose terminal step delegates to the
  existing process-global registry key, making single-tenant the zero-config default
  and multi-tenant an overlay; plus the request-path integration that resolves
  per-request from `UserContext` and constructs the driver with the resolved key.

## Impact

- **New dependency:** `aes-gcm = "0.10"`.
- **Touched code:** `src/llm/` (registry/driver construction at the route seam),
  `src/uar/security/` (credential store + encryption, supersedes the
  `session/encrypted.rs` stub), `src/uar/api/` (provider/credential REST + route-layer
  resolution), `src/lib.rs` (`AppState` gains `Option<Arc<ProviderService>>`).
- **Backends:** SurrealDB store in the default build; a Postgres store, if added, is
  gated behind the existing `postgres-backend` feature. The surreal-only default build
  must not regress.
- **Backward compatibility:** single-tenant/self-hosted operation is preserved with
  zero configuration — when no per-user credentials are stored and `ProviderService`
  is `None` (or every lookup misses), resolution falls through to the env-var step =
  current `src/config.rs` behavior.
- **Out of scope (deferred):** runtime models.dev re-sync (build-time catalog remains
  authoritative; do not reintroduce a conflicting second catalog); WISC `scout` +
  `decide`/`prime`/`handoff` composite recipes (separate change).
- **Ops:** multi-tenant deployments must set `CREDENTIAL_ENCRYPTION_KEY`; its absence
  is a hard error only on per-user key access, never for single-tenant env operation.
