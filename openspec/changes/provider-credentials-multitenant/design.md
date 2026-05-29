# Design — provider-credentials-multitenant

## Context

UAR's LLM key is process-global. The driver is built in `RunManager::start_run`
(`src/uar/runtime/manager.rs:698`, `Orchestrator::new(run_llm_config, …)`), where
`run_llm_config` is assembled by layering:

1. **Registry/policy** (`manager.rs:654`) — `provider_registry.resolve_llm_config_from_policy(...)` or `self.llm_config`.
2. **Skill overrides** (`manager.rs:677`) — first matched skill's `preferred_model` mutates `cfg.model`.

`start_run` already receives `user_id: Option<String>` and `session_id: Option<String>`.
This change adds a **third config layer** — credential resolution — that sets
`cfg.api_key` from a per-user encrypted store, falling back to the existing key when
absent. No orchestrator/driver signatures change.

## Goals / Non-Goals

**Goals**
- Per-user, encrypted-at-rest provider credentials with scoped resolution.
- Single-tenant operation preserved with zero configuration (no behavior change).
- Minimal blast radius: one new config layer + a store + an API; signatures stable.

**Non-Goals**
- Runtime models.dev re-sync (build-time catalog stays authoritative).
- WISC `scout` / `decide` / `prime` / `handoff` salvage (separate change).
- Postgres credential store in the default build (feature-gated only).

## Key Decisions

### Decision 1: Resolve as a third config layer in `start_run` (not a route or orchestrator change)
The route (`src/uar/api/openai/routes.rs`) does **not** build the driver — it calls
`run_manager.start_run(agent, …, Some(user_context.user_id), …)`. The RunManager owns
construction. Therefore credential resolution is performed **inside `start_run`**, in
the `run_llm_config` assembly block, immediately after the skill-override layer
(`manager.rs:~692`), before `Orchestrator::new`:

```rust
// 3. Credential resolution layer (NEW)
let run_llm_config = {
    let mut cfg = run_llm_config;
    if let Some(ref provider_service) = self.provider_service {
        let provider_id = provider_of(&cfg.model); // "anthropic/claude…" -> "anthropic"
        if let Some(resolved) = provider_service
            .resolver()
            .resolve_with_context(
                user_id.as_deref().unwrap_or("anonymous"),
                &provider_id,
                session_id.as_deref(),
                /* agent_id */ Some(&artifact.id),
            )
            .await?
        {
            cfg.api_key = Some(resolved.api_key); // overrides env/registry key for this run
        }
        // None => leave cfg.api_key as-is (env/config/registry fallback = single-tenant)
    }
    cfg
};
```

- **Why here:** it's the one place that already has `user_id`, `session_id`, the
  resolved model (hence provider), and produces the exact `cfg` consumed by
  `Orchestrator::new`. `Orchestrator::chat` / `chat_with_history` stay unchanged
  (satisfies `credential-resolution` spec: "Orchestrator signature unchanged").
- **Single-tenant equivalence:** when `self.provider_service` is `None` **or** the
  resolver returns `None`, `cfg.api_key` is untouched — identical to today.

### Decision 2: `ProviderService` threaded via `RunManager`, sourced from `AppState`
- Add `provider_service: Option<Arc<ProviderService>>` to `RunManager`
  (`manager.rs:73`), constructor-injected (mirror the existing `persistence`/
  `provider_registry` optional-field pattern).
- Add `provider_service: Option<Arc<ProviderService>>` to `AppState` (`src/lib.rs:67`);
  `None` ⇒ pure single-tenant.
- `ProviderService { store: Arc<dyn CredentialStore>, encryption: Arc<CredentialEncryption> }`
  (catalog reads come from the existing `ModelCatalog`, not a new store).

### Decision 3: Encryption — port the branch design verbatim onto current deps
- `CredentialEncryption` wraps `Aes256Gcm`; 32-byte key from `CREDENTIAL_ENCRYPTION_KEY`.
- Stored form: `base64(12-byte-nonce ‖ ciphertext)`; fresh random nonce per encrypt.
- Add `aes-gcm = "0.10"` to `Cargo.toml`.
- Key is read lazily: required only when the store encrypts/decrypts — never at boot,
  so single-tenant deployments need not set it.

### Decision 4: Store — SurrealDB default, Postgres feature-gated, trait-abstracted
- `trait CredentialStore` (async) with `get_credential_row(scope, id, provider)`,
  `put`, `delete`, plus scope-specific helpers (`get_session_credential_row`, etc.) as
  needed by the resolver's chain.
- `SurrealCredentialStore` in the default build. Optional `PostgresCredentialStore`
  behind `#[cfg(feature = "postgres-backend")]`, matching UAR's established pattern.
- Schema (Surreal): record per `(scope, scope_id, provider_id)` →
  `{ api_key_encrypted: String, created_at, updated_at }`. Unique on the triple.

### Decision 5: Resolution chain order and fallthrough
`resolve_with_context(user_id, provider_id, session_id, agent_id)` tries in order:
`session → agent → user → system → env`. The **`env` step delegates to the existing
key** already present on `cfg`/registry (i.e. resolution returning `None` *is* the env
step — we simply don't override `cfg.api_key`). First hit wins; provider-specific.

### Decision 6: REST API surface
- `GET /api/providers`, `GET /api/models` — read from existing catalog.
- `GET/POST/PUT/DELETE /api/credentials` (or `/api/providers/:id/credential`) — per-user
  CRUD; JWT-protected via existing `src/uar/security/middleware.rs`; subject from
  `UserContext`. Plaintext accepted on write only; reads return masked metadata
  (provider, last-4, timestamps) — never plaintext or ciphertext.

## Data / Control Flow

```
chat_completions route (has UserContext)
  └─ run_manager.start_run(agent, …, user_id, session_id, …)
       ├─ layer 1: registry/policy        → run_llm_config
       ├─ layer 2: skill override          → cfg.model
       ├─ layer 3: credential resolution    → cfg.api_key   ← NEW (this change)
       │     provider_service.resolver().resolve_with_context(user, provider, session, agent)
       │       session → agent → user → system → (None ⇒ keep env/registry key)
       └─ Orchestrator::new(cfg, …)         → unchanged
```

## Risks / Mitigations

| # | Risk | Mitigation |
|---|------|-----------|
| 1 | **(was highest — now resolved)** UserContext reaching the driver | Confirmed: `user_id`/`session_id` already flow into `start_run`; resolution layered there. No deep threading. |
| 2 | Resolver duplicating provider config | Resolver only supplies `api_key`; model/provider routing stays with registry. Single source of truth. |
| 3 | `CREDENTIAL_ENCRYPTION_KEY` ops burden | Lazy read; hard error only on per-user key access; never required for single-tenant. |
| 4 | Surreal-default build regressing | Postgres store strictly behind `postgres-backend`; CI default build must compile + pass without it. |
| 5 | Dep drift from the stale branch | Re-author files onto current `Cargo.toml` (liter-llm rc.41, axum-test 19.x); cherry-pick code, never merge the branch. |
| 6 | `provider_of(model)` mis-parse | Derive provider from the `provider/model` prefix already used by the catalog/registry; reuse existing parsing, don't invent. |
| 7 | Plaintext leakage via logs | `ResolvedCredential` is non-`Debug`-printing for the key (or `secrecy::SecretString`); resolver never logs the value; errors carry provider/scope only. |

## Migration / Compatibility
- No migration for single-tenant: `provider_service = None` ⇒ unchanged behavior.
- Multi-tenant rollout: set `CREDENTIAL_ENCRYPTION_KEY`, construct `ProviderService`,
  inject into `AppState`/`RunManager`. Existing env/config key becomes the house-account
  fallback automatically.
- Supersedes the `src/session/encrypted.rs` stub (remove or redirect once the store lands).

## Open Questions (resolve during apply)
- Use `secrecy::SecretString` (already a UAR dep) for in-memory resolved keys? (Leaning yes — Risk 7.)
- Credential endpoint shape: flat `/api/credentials` vs nested `/api/providers/:id/credential`? (Cosmetic; pick during apply.)
