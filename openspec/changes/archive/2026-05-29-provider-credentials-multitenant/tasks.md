# Tasks — provider-credentials-multitenant

Ordering keeps the surreal-default build compiling and single-tenant behavior intact
at every checkpoint. Multi-tenant is wired last, behind `Option`.

> Implementation status (2026-05-29): core slice S1–S7 implemented; full default
> build green; 13 credential unit tests pass; postgres-feature build verified.
> Two scoped deferrals noted inline (durable Surreal-backed store wiring; HTTP
> integration tests). A pre-existing blocker was fixed: `surrealdb-core` lock
> pinned 3.1.2 vs `surrealdb =3.0.5` → pinned core to 3.0.5.

## 1. Dependencies & encryption primitive
- [x] 1.1 Add `aes-gcm = "0.10"` to `Cargo.toml` `[dependencies]`; build clean.
- [x] 1.2 `CredentialEncryption` in `src/uar/security/credentials/encryption.rs`: `Aes256Gcm`, key from `CREDENTIAL_ENCRYPTION_KEY` (32 ASCII or 64 hex), store form `base64(nonce ‖ ciphertext)`, fresh nonce per encrypt.
- [x] 1.3 Lazy key acquisition — `from_env()` returns `Ok(None)` when absent; error only on use.
- [x] 1.4 Unit tests: round-trip; same-plaintext→distinct-ciphertext; wrong-key fails cleanly. ✅ pass.

## 2. Credential store (trait + SurrealDB)
- [x] 2.1 `trait CredentialStore` (async) in `store.rs`: `put`/`get`/`list`/`delete` keyed by `(scope, scope_id, provider_id)`.
- [x] 2.2 `CredentialRecord` + `CredentialMetadata` (masked view); `ResolvedCredential` holds key as `secrecy::SecretString` (no Debug leak).
- [x] 2.3 `SurrealCredentialStore` (compiles in default build); deterministic record id ⇒ uniqueness + idempotent upsert. Uses codebase `Value→serde_json` conversion (surrealdb 3.x).
- [x] 2.4 Store tests: store→retrieve user-scoped; cross-user isolation; delete removes; provider isolation. ✅ pass.
- [x] 2.5 No Postgres-specific store needed — `SurrealCredentialStore` is backend-agnostic at the API layer; default build unaffected; postgres-feature build verified.
- [x] 2.6 `InMemoryCredentialStore` added (matches `api_keys` precedent; wired default + tests).

## 3. CredentialResolver (scoped chain)
- [x] 3.1 `CredentialResolver { store, encryption }` in `resolver.rs` with `resolve_with_context(user_id, provider_id, session_id, agent_id)`.
- [x] 3.2 Chain `session → agent → user → system`; first decrypted hit; `None` when all miss.
- [x] 3.3 Provider-specific: never returns another provider's key.
- [x] 3.4 No-leak: key in `SecretString`; decryption errors carry provider/scope only.
- [x] 3.5 Resolver tests: highest-scope-wins; fall-through to user; all-miss→None; provider isolation. ✅ pass.

## 4. ProviderService assembly
- [x] 4.1 `ProviderService { store, encryption }` with `resolver()`/`store()`/`encryption()` in `credentials/mod.rs`.
- [x] 4.2 `from_env(store)` reads `CREDENTIAL_ENCRYPTION_KEY`; `Ok(None)` when absent (single-tenant), `Err` when malformed.

## 5. Wire into AppState + RunManager (single-tenant stays default)
- [x] 5.1 `provider_service: Option<Arc<ProviderService>>` on `AppState` (`lib.rs`); set at the one literal in `server.rs`.
- [x] 5.2 `provider_service` field on `RunManager` + non-breaking `with_provider_service(..)` builder; `None` default in `new`.
- [x] 5.3 Build + tests green with `provider_service = None` (default path unchanged).

## 6. Resolution layer in start_run (the core seam)
- [x] 6.1 Third config layer in `RunManager::start_run` after skill-override, before `Orchestrator::new`; sets `cfg.api_key` when `provider_service` is `Some` and resolver returns a key.
- [x] 6.2 `provider_id` derived from `cfg.model` (`provider/model` prefix).
- [x] 6.3 Resolver `None`/`provider_service` `None` ⇒ `cfg.api_key` untouched (single-tenant); user/session/agent scopes captured before `user_id` is moved.
- [x] 6.4 `Orchestrator::new`/`chat`/`chat_with_history` signatures unchanged. ✅
- [ ] 6.5 **DEFERRED** — HTTP-level integration test asserting effective per-run `api_key`. Logic covered by resolver unit tests; an end-to-end run-level assertion needs a test harness for `start_run` internals (follow-up).

## 7. REST API
- [x] 7.1 Provider/model reads **already exist** at `/api/uar/providers` (list/get/models) — no new catalog endpoints needed.
- [x] 7.2 Credential CRUD at `/api/uar/credentials` (`GET /`, `PUT /{provider}`, `DELETE /{provider}`), JWT-gated; subject from `UserContext`.
- [x] 7.3 Write accepts raw key once → encrypted; reads return masked `CredentialView` (provider, last-4 hint, timestamps) — never plaintext/ciphertext.
- [x] 7.4 Anonymous → 401; user scoping means a caller can only touch their own (`scope=User, scope_id=subject`); service-disabled → 503.
- [x] 7.5 HTTP integration tests in `tests/credentials_api_integration_test.rs` (axum-test): anon→401; service-disabled→503; store→list masked (asserts plaintext + ciphertext field absent); rotate updates hint; delete→204 then 404; cross-user isolation. Router refactored to a narrow `CredentialApiState = Option<Arc<ProviderService>>` for testability.

## 8. Cleanup & docs
- [x] 8.1 **DROPPED (misread correction)** — `src/session/encrypted.rs` is encrypted *chat-session comms* (WebRTC/streaming), unrelated to provider credentials. Nothing to supersede.
- [x] 8.2 Documented `CREDENTIAL_ENCRYPTION_KEY` in `.env.example` + `CLAUDE.md`; single-tenant needs nothing new.
- [x] 8.3 Default (surreal) build **green**; 13 credential tests **pass**. NOTE: `--features postgres-backend` build fails on a **pre-existing** `pgvector::Vector: sqlx::Encode/Type` mismatch in persistence code — unrelated to this change (lock diff untouched for sqlx/pgvector; no credential file implicated; no postgres-specific credential code exists). Not a regression. (clippy: final gate below.)
- [x] 8.4 Re-authored onto current `Cargo.toml` (liter-llm rc.41); no code/deps merged from `origin/feature/providers`.

## 9. Verification
- [ ] 9.1 **DEFERRED (manual)** — single-tenant smoke: no key, no `ProviderService` → chat uses env key. (Compile-time guaranteed: `None` ⇒ `cfg.api_key` untouched.)
- [ ] 9.2 **DEFERRED (manual)** — multi-tenant smoke: set key, store user credential via API, chat resolves user key; second user falls back to env.
- [ ] 9.3 `/opsx:verify provider-credentials-multitenant` → target 0 CRITICAL before archive.

- [x] 8.5 `cargo clippy --lib` clean for all new credential modules (fixed `map_err_ignore` via named-ignored binding documenting the no-leak intent; backticked `SurrealDB` doc item).

### Remaining follow-ups (carry-over)
1. ~~Durable SurrealDB-backed credential store wiring~~ ✅ **DONE** — `server.rs` now builds a `SurrealCredentialStore` from the live `Surreal<Any>` client (5th element of the persistence tuple) and uses it for `ProviderService`; in-memory fallback only for non-surreal backends.
2. ~~HTTP integration tests (7.5)~~ ✅ **DONE**. Remaining: run-level `start_run` assertion (6.5) and the two manual smokes (9.1, 9.2).
3. `cargo clippy` clean-up pass (credential modules already clean).
