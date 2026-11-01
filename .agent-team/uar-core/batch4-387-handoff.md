# Batch 4 handoff: issue #387 owner-scoped compiler specs and sessions

Writer: uar-runtime. Verification lock was CLOSED: nothing was compiled, tested or run. All claims below are source inspection only.

## Files changed
- `src/uar/compiler/owner.rs` (new): `CompilerOwner` (verified owner capture + legacy rule) and its unit tests.
- `src/uar/compiler/mod.rs`: `pub mod owner;`.
- `src/uar/compiler/storage.rs`: `SpecRecord.owner`, `ReportRecord.owner` (both `#[serde(default, skip_serializing_if = "Option::is_none")]`), `SpecRecord::with_owner`, `ReportRecord::with_owner_key`; `SpecStorage` trait methods now take `&CompilerOwner` (get/list/update/delete spec, get/list report); in-memory impl filters; unit tests updated/extended.
- `src/uar/compiler/session/mod.rs`: `CompilerSession.owner` (serde default) + `with_owner`.
- `src/uar/compiler/session/persistence.rs`: `SessionStorage` get/delete/list take `&CompilerOwner`; in-memory impl filters. `save_session` persists the owner carried by the session.
- `src/uar/compiler/service.rs`: every `CompilerService` spec/report/session method takes `&CompilerOwner`; `store_spec`/`create_session` stamp it; `compile_spec` copies the spec owner to the report; tests updated.
- `src/uar/compiler/storage/surreal.rs`: owner-filtered queries (`select_owned`, report list query), id-addressed get/update/delete/delete_session prove ownership first; fresh-db tests updated.
- `src/uar/compiler/storage/postgres.rs`: `owner` column in all insert/select/update/delete statements for specs, reports, sessions.
- `migrations/20261031010000_compiler_owner_scope.sql` (new): nullable `owner TEXT` on `uar_specs`, `uar_reports`, `uar_compiler_sessions` + three indexes. No existing migration edited, no data rewritten.
- `src/uar/api/compiler.rs`: all spec, report and session handlers take `Option<Extension<UserContext>>` and resolve the owner via `verified_owner`.
- `scripts/behaviors/features/compiler-owner-isolation.feature`, `scripts/behaviors/steps/compiler-owner-isolation.steps.mjs`, `scripts/behaviors/compiler-owner-isolation.cucumber.mjs` (new): BDD per the brief.

## Owner-resolution rule
`CompilerOwner::from_context(&UserContext)` in `src/uar/compiler/owner.rs` calls `CredentialPrincipal::from_verified_context` (the same capture the accounting API uses) and uses `storage_key()` as the record owner. Anonymous/synthetic contexts fail, so the handler returns 401 with `{"code":"verified_compiler_owner_required","error":"Verified compiler owner required"}`. `session_owner_key()` was deliberately not used because it admits the `anonymous` owner. The local-host owner is `principal.issuer_kind() == "local-host"` (what `local_loopback_owner` and the authenticated sidecar launch both produce). Existing remote-profile 403 boundary and both route aliases are untouched (`src/server.rs` not edited; no change needed).

## Legacy-record rule
A record whose stored owner is absent/NULL/empty is readable, updatable, compilable and deletable only by an owner with `reads_legacy_records()` (the local-host principal). It is never rewritten or claimed; cancelling a legacy session keeps it ownerless. Another owner's record is 404 everywhere (storage returns `None`/`false`; no 403).

## HTTP statuses the BDD asserts
- Create spec and session: 201 (both owners). Lists, own-record GETs: 200. Foreign GET spec/session (both aliases), POST spec compile, DELETE spec, POST session cancel, POST session compile: 404. Own spec still 200 and own session `status == "active"` after the foreign attempts.
- With `local_loopback_owner` off, loopback no-credential POST `/specs` and POST `/sessions`: 401 (strictly 401, not 401/403).

## Behavior notes and decisions
- Surreal `delete_spec` previously returned `true` unconditionally; it now returns `false` (404 at the API) when the spec is absent or foreign. This is required for the 404 assertion and matches Postgres/in-memory.
- Not owner-scoped, deliberately: stateless `POST /compile` and `POST /verify` (no storage), and `POST /compile-and-register` (writes the global agent catalog via `agent_store`, not compiler records). If the lead wants an owner required on those, it is a one-line `verified_owner` call per handler; unresolved decision.
- In-agent compiler tools (`uar.session.*`, `CompilerSessionStore` in `conversational.rs`) do not use the storage traits; they already scope by host-verified owner + conversation id, so no change was needed. Nothing else in `src/` calls the changed trait/service methods (grepped).
- Trait signatures are a breaking change for any out-of-tree `SpecStorage`/`SessionStorage` implementor; none exists in this repo.

## Cross-domain follow-ups (not mine, not edited)
- `tests/integration/live/capability_cases.rs` `l3_c09_*` call `/api/compiler/specs`, `/api/uar/compiler/specs`, `/api/compiler/sessions` with no owner. Whether they still pass depends on how `boot_test_server` authenticates; if it yields an anonymous context they will now get 401 and need the loopback owner (or equivalent) in that harness.
- `tests/bdd/steps/product-screen-validation.steps.ts` `validateCompiler` creates a session through the console; it needs a verified owner in that deployment (proxy/loopback owner) or it will get 401.
- `scripts/release/uar_1_1_1_compiler.py` expects local single-owner control to work with a JWT principal and remote to 403; consistent with this change, but it was not re-read for ownerless-record assumptions.
- No `src/server.rs` change is required.

## Unverified (everything)
Nothing was compiled, linted, formatted or run. Specifically unverified: Rust compiles warning-free (axum `Option<Extension<_>>` extraction, sqlx bool binds, Surreal `owner = NONE` / `$legacy` query parameters and `ORDER BY created_at` on `SELECT *`), the migration applies on Postgres, Surreal row deserialization of legacy rows, the BDD step expressions and 401/404 statuses against a live runtime on both backends, and that the fixture's identity proxy yields a principal distinct from `local-host`. rustfmt line wrapping was not applied.
