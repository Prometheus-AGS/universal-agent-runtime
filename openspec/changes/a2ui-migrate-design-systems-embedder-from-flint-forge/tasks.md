## 1. Audit flint-forge sources
- [x] 1.1 Confirm `flint-forge/crates/fdb-app/src/a2ui/` exists: `mod.rs`, `types.rs`, `design_md_parser/{mod,components,sections,text_extract,w3c,tests}.rs`. Confirmed real, pure-Rust (no DB dependency).
- [x] 1.2 Confirm `flint-forge/crates/fdb-gateway/src/a2ui_embedder.rs` exists (319 lines). Confirmed real, Postgres/`sqlx`/`pgvector`-specific.
- [x] 1.3 Confirm `flint-forge/migrations/0009_flint_a2ui_design_systems.sql` exists (82 lines). Confirmed real: adds `source_format`/`source_content`/`imported_at` to `flint_a2ui.design_systems`, creates `flint_a2ui.component_overrides` with RLS policies.
- [x] 1.4 Check `flint-forge/crates/fdb-reflection/src/compilers/a2ui.rs` (plan's claimed path). **Confirmed absent as a single file.** Found instead: `flint-forge/crates/fdb-reflection/src/compilers/a2ui/` — a real module (`mod.rs`, `types.rs`, `assembler.rs`, `error.rs`, `helpers.rs`, `rows.rs`, `tests.rs`) implementing an **event→A2UI-surface assembler** (`A2uiAssembler`), unrelated to the design-system/token-override bridge. Documented the correction and scope decision in `proposal.md`.
- [x] 1.5 Check `flint-forge/crates/fdb-gateway/tests/a2ui_*_test.rs`. Confirmed 5 files: `a2ui_application_model_test.rs`, `a2ui_embedder_test.rs`, `a2ui_schema_test.rs`, `a2ui_seed_test.rs`, `a2ui_trigger_test.rs`. Only `a2ui_embedder_test.rs` covers code in this change's scope; it requires live Postgres + `ext-flint-llm`, so its intent is reproduced as adapted tests rather than copied verbatim.
- [x] 1.6 Read UAR's existing backend-per-feature convention (`src/uar/security/credentials/store.rs`'s `CredentialStore`/`InMemoryCredentialStore`/`SurrealCredentialStore`/`PostgresCredentialStore`) to model the new store on an established UAR pattern rather than flint-forge's Postgres-only shape.
- [x] 1.7 Confirm UAR's persistence/feature reality: `minimal = ["surreal-backend"]`, `server-full = ["minimal", ...]` (no `postgres-backend`) — the certified build profile is SurrealDB, not Postgres. This drove the store-design decision in `proposal.md`.
- [x] 1.8 Confirm no A2UI component catalog exists anywhere in UAR yet (`grep -rl "primitive_type|ResolvedComponent|component_overrides"` — no hits before this change). Documented as an "out of scope" scope note (Change 18 owns the real catalog).
- [x] 1.9 Confirm `flint-forge` has no publishable/git-installable crate setup usable as a Cargo git dependency; documented the source-port decision in `proposal.md`.

## 2. `src/uar/a2ui/design_systems/types.rs`
- [x] 2.1 Port `Renderers`, `DesignToken`, `DesignTokenMap` from `fdb-app/src/a2ui/types.rs`.
- [x] 2.2 Add `SourceFormat`, `DesignSystem`, `Component`, `ComponentOverrideRecord`, `ResolvedComponent` (new — UAR-side persistence records corresponding to the migration-0009 schema plus a minimal base-component catalog row).
- [x] 2.3 Add `resolve_component()` replicating `flint_a2ui.resolve_components_with_overrides()`'s merge semantics in Rust.
- [x] 2.4 Unit tests: resolve-without-override, resolve-with-override, `SourceFormat` round-trip.

## 3. `src/uar/a2ui/design_systems/design_md_parser/`
- [x] 3.1 Port `mod.rs`, `sections.rs`, `components.rs`, `text_extract.rs`, `w3c.rs` near-verbatim (same module split, same function names).
- [x] 3.2 Port `tests.rs` verbatim (all 9 original test cases pass unmodified against the ported code).

## 4. `src/uar/a2ui/design_systems/store.rs`
- [x] 4.1 Define `DesignSystemStore` trait (put/get/list for design systems, components, overrides; missing-embedding listing; embedding write).
- [x] 4.2 `InMemoryDesignSystemStore` — always-available implementation, with unit tests (round-trip, sort order, slug lookup, missing-embedding tracking, override scoping).
- [x] 4.3 `SurrealDesignSystemStore` (feature `surreal-backend`) — matches `SurrealCredentialStore`'s id-stripping-on-write / `RecordId`-unwrapping-on-read pattern. Integration tests against an embedded SurrealKV instance (no external server required).
- [x] 4.4 `PostgresDesignSystemStore` (feature `sqlx`) — matches `PostgresCredentialStore`'s row-mapping pattern.

## 5. `src/uar/a2ui/design_systems/embedder.rs`
- [x] 5.1 Port `build_embedding_text` (slug/type/category/description/usage-examples/prop-names concatenation) — same shape as flint-forge's version, unit test ported from `a2ui_embedder.rs`'s `build_embedding_text_includes_props`.
- [x] 5.2 `embed_component` — embeds one component via UAR's `EmbeddingBackend` trait (`src/uar/rag/embeddings/mod.rs`) instead of an in-database `llm.embed()` call; primary/fallback-model behavior preserved.
- [x] 5.3 `backfill_missing` — sweeps `list_components_missing_embedding()`, best-effort (logs and continues past per-component failures), matching flint-forge's backfill semantics.
- [x] 5.4 Unit tests with a stub `EmbeddingBackend`: successful embed, fallback-on-primary-failure, missing-component error, backfill embeds all + is best-effort.
- [x] 5.5 **Deferred**: live `a2ui_embed`-channel-equivalent notification. Documented in `proposal.md`'s "Out of scope" as Change 20's responsibility (the `flint-realtime-fabric` SSE/fan-out backbone).

## 6. `src/uar/a2ui/design_systems/import.rs` (new UAR-side glue)
- [x] 6.1 `import_design_md` — parse + persist a `DesignSystem`, apply §5 overrides against slug-matched base components, report skipped slugs.
- [x] 6.2 `import_w3c_tokens` — parse + persist a `DesignSystem` from raw W3C token JSON.
- [x] 6.3 Unit tests: full import with one matched + one unmatched override slug; W3C token import.

## 7. Module wiring
- [x] 7.1 `src/uar/a2ui/design_systems/mod.rs` — re-exports, module docs cross-referencing the flint-forge sources and this change's proposal.
- [x] 7.2 Add `pub mod design_systems;` to `src/uar/a2ui/mod.rs`.

## 8. Migrations
- [x] 8.1 `migrations/20260714120000_a2ui_design_systems.sql` — Postgres migration (feature `postgres-backend`/`sqlx`): `a2ui_design_systems`, `a2ui_components`, `a2ui_component_overrides`, `a2ui_component_embeddings` (`vector` column, `pgvector` extension already enabled by `20251225000000_init_uar.sql`).
- [x] 8.2 `migrations/surrealdb/schema.surql` — add `design_systems`, `components`, `component_overrides`, `component_embeddings` `SCHEMAFULL` tables matching the Rust structs' fields.

## 9. Verification
- [ ] 9.1 `cargo check --no-default-features --features server-full` — pass (exercises the default `surreal-backend` code paths; Postgres-gated code is excluded from this build, matching every other `sqlx`-gated module in this repo).
- [ ] 9.2 `cargo check --no-default-features --features server-full,postgres-backend` — pass (exercises the `PostgresDesignSystemStore` code path).
- [ ] 9.3 `cargo test --no-default-features --features server-full uar::a2ui::design_systems` — pass (parser, types, in-memory store, embedded-SurrealKV store, embedder unit tests).
- [ ] 9.4 `cargo fmt --all -- --check` — pass.
- [ ] 9.5 `openspec validate a2ui-migrate-design-systems-embedder-from-flint-forge --strict` — pass.

## 10. Operator follow-up
- [ ] 10.1 Operator/coordinating session reviews the source-port-vs-Cargo-git-dependency decision and the reflection-compiler scope correction in `proposal.md` before merge. **Not auto-approved.**
- [ ] 10.2 Coordinating session decides on merge (per task instructions, this session does not merge its own branch).
- [ ] 10.3 Change 18 (entity component migration) should populate `a2ui_components`/`components` with a real catalog once it lands — this change's catalog is intentionally minimal/empty by default.
- [ ] 10.4 Change 20 (`a2ui-realtime-backbone-from-flint-realtime-fabric`) should evaluate migrating `fdb-reflection/src/compilers/a2ui/`'s `A2uiAssembler` as part of its live-update backbone scope, per the correction in `proposal.md`.
