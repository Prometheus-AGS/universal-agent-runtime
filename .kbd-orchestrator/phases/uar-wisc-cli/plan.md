# Plan — `uar-wisc-cli` phase: port multi-tenant encrypted credentials (both modes)

- Generated: 2026-05-29
- Author: claude-code (`/kbd-plan`)
- Decision input: **BOTH** single-tenant and multi-tenant required (user, 2026-05-29)
- Primary salvage: Finding 2 (G6+G7+G8) from `assessment.md`
- Secondary (deferred): Finding 1 (`scout`, composite recipes) — separate change

## Architectural thesis

> Layer request-scoped, per-user credential resolution **in front of** the existing
> process-global router. The router (`src/llm/registry.rs`) stays exactly as-is and
> becomes the **single-tenant fallback** (the `system → env var` tail of the chain).
> Multi-tenant keys overlay on top via a new resolution step that consumes
> `UserContext` — which the HTTP layer already has but the router currently ignores.

Today's gap (confirmed in code): `src/llm/registry.rs` stores one `api_key` per
provider and has **no `UserContext`**. Single design satisfies both modes because
the resolution chain terminates in the env-var step = current behaviour.

```
request (UserContext) ─► CredentialResolver.resolve_with_context(user, provider, session, agent)
                            │  session → agent → user → system   (NEW: encrypted store)
                            │                          ↓ None
                            └────────────────────────► env var / config   (EXISTING: registry)
```

## Scope

### In scope (this change)
1. **Encryption** — port `providers/encryption.rs` (AES-256-GCM, `CREDENTIAL_ENCRYPTION_KEY`,
   `base64(nonce ‖ ciphertext)`) onto current deps. Add `aes-gcm = "0.10"` to `Cargo.toml`.
2. **Catalog/credential store (SurrealDB only)** — port `catalog_store.rs` +
   `catalog_store_surreal.rs`. Gate a Postgres impl behind the existing
   `postgres-backend` feature (mirror UAR's established cfg pattern); do **not**
   port Postgres in the default build.
3. **CredentialResolver** — port `credential_resolver.rs` with the 5-level chain;
   the final step delegates to the **existing registry/config key** so single-tenant
   is the zero-config default.
4. **Request-path integration** — thread `UserContext` (already in `claims.rs`) into
   the LLM call path so the resolver runs per request. Wire `ProviderService` into
   `AppState` (`src/lib.rs:67`) as `Option<Arc<ProviderService>>` (None ⇒ pure single-tenant).
5. **REST API (G8)** — `/api/providers`, `/api/models`, and credential CRUD
   (store/rotate/delete per-user keys; raw key accepted once, encrypted at rest,
   never returned). JWT-protected via existing `middleware.rs`.
6. **Replace the `session/encrypted.rs` stub** or clearly supersede it.

### Out of scope (deferred to follow-up changes)
- Runtime models.dev re-sync (`sync_service.rs`, `toml_parser.rs`) — `main`'s
  build-time `build.rs` catalog is sufficient for now; revisit only if runtime
  provider addition is needed. **Do not** reintroduce a second catalog that
  conflicts with `build.rs`.
- Finding 1 salvage (`scout` + `decide`/`prime`/`handoff` recipes) — own change
  (`uar-wisc-scout-mcp` or similar) if non-Claude-Code agents come into scope.
- Branch's `model_classification.rs`, `seed.rs` — evaluate during execution, likely skip.

## OpenSpec change shape

Recommend splitting into **one capability** with clear specs:

| Capability | Spec focus |
|---|---|
| `provider-credentials` | encrypted-at-rest BYO keys; CRUD API; never-return-plaintext invariant |
| `credential-resolution` | 5-level scoped chain; single-tenant env fallback equivalence; per-request `UserContext` threading |

(Decide proposal vs. capability granularity at `/opsx:new` time.)

## Risks / decisions to surface during planning

1. **Key threading depth — RESOLVED (traced 2026-05-29).**
   - `UserContext` reaches the chat route: `src/uar/api/openai/routes.rs:50`
     (`Extension(user_context)`); `user_context.user_id` already used at `routes.rs:94`.
   - BUT `Orchestrator::chat_with_history(messages)` (`src/llm/orchestrator.rs:285`)
     takes **only messages** — no user/credential context. This is the single-tenant
     seam: the orchestrator is credential-agnostic by construction.
   - **Chosen integration = Option A (resolve-then-construct), NOT deep threading:**
     the **route** runs `CredentialResolver` (it already holds `UserContext` +
     target provider), then builds/configures the per-request orchestrator/driver
     with the resolved key. If resolver returns `None`, the orchestrator uses its
     existing registry/env key = single-tenant default. **Orchestrator signature
     stays unchanged.** This keeps the blast radius at the route layer.
   - Rejected Option B (thread `UserContext` into the orchestrator + resolve
     internally): more invasive, couples orchestrator to credential storage.
2. **Registry vs resolver ownership** — resolver must call the registry for the
   fallback, not duplicate provider config. Single source of truth = registry/catalog.
3. **`CREDENTIAL_ENCRYPTION_KEY` ops** — required only in multi-tenant mode; absence
   must be a hard error *only* when a user key write/read is attempted, never for
   single-tenant env-only operation. (Keeps self-hosted zero-config.)
4. **Backend parity** — SurrealDB is default; Postgres behind feature flag. Do not
   regress the surreal-only default build.
5. **Do not regress deps** — port code onto *current* `Cargo.toml` (liter-llm rc.41,
   axum-test 19.x, uar-jwt-proxy workspace member). Cherry-pick files, never merge the branch.

## Suggested next steps
1. Verify risk #1 (UserContext reaches the LLM call path) — a 15-min code trace.
2. `/opsx:new provider-credentials-multitenant` → author proposal + specs.
3. `/opsx:continue` through design + tasks.
4. Preserve `origin/feature/providers` as the reference until execution lands.
