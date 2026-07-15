## Why

The `uar-grade-a-upgrade-2026-07` phase plan (operator decision Q2)
commits UAR to migrating its A2UI component library from three
sources: `prometheus-entity-management` (entity components, Change
18), `flint-realtime-fabric` (the live-update backbone, Change 20),
and `flint-forge` (the design-system import + override bridge and its
embedder, this change — Change 19). Change 19's done condition asks
for the design-systems layer, the embedder, and "the application
model + reflection compiler" at
`flint-forge/crates/fdb-reflection/src/compilers/a2ui.rs` to be
migrated into a new `src/uar/a2ui/design_systems/` module with a new
SQL migration.

## Plan corrections (audited against the real flint-forge tree)

1. **`flint-forge/crates/fdb-reflection/src/compilers/a2ui.rs` does not
   exist as a single file** (the plan's premise here is wrong), but a
   directory module at
   `flint-forge/crates/fdb-reflection/src/compilers/a2ui/` **does**
   exist and is real, non-trivial code: `mod.rs`, `types.rs`,
   `assembler.rs` (226 lines), `error.rs`, `helpers.rs`, `rows.rs`,
   `tests.rs`. It implements `A2uiAssembler` — a rules-based compiler
   that turns an *event* (`AssemblyContext`: event type, JSON payload,
   application id, JWT claims) into an assembled `A2uiSurface` (a
   sequence of A2UI protocol messages), by resolving assembly rules
   from a `flint_a2ui.assembly_rules`-shaped table and falling back to
   a default table→component binding. **This is a materially different
   thing from a "design-system reflection compiler"**: it does not
   touch `design_systems`, `component_overrides`, tokens, or
   `DESIGN.md` at all — it is an *event-to-surface* compiler that
   happens to live under `fdb-reflection`'s `compilers/` tree
   alongside GraphQL/OpenAPI/MCP/REST compilers for other purposes.
   Nothing in `flint-forge/crates/fdb-app/src/a2ui/` or
   `flint-forge/migrations/0009_flint_a2ui_design_systems.sql`
   references it, and it references neither of those.
2. Given (1), the plan's phrase "the application model + reflection
   compiler" conflated two things that are not actually connected in
   flint-forge: the *design-system application model* (real, in
   `fdb-app/src/a2ui/types.rs` — `ResolvedComponent`, `Renderers`,
   `DesignToken`, `DesignTokenMap`) **is** migrated by this change (see
   `types.rs`); the *event-assembly reflection compiler*
   (`fdb-reflection/src/compilers/a2ui/`) is a separate, larger
   subsystem (state machine over `assembly_rules`, JWT-claim-filtered
   resolution, an `A2uiPublisher` trait for FRF topic fan-out) that
   belongs with Change 20 (`a2ui-realtime-backbone-from-flint-realtime-fabric`)
   or a dedicated follow-up change, not this one — see "Out of scope"
   below.
3. `flint-forge/crates/fdb-gateway/tests/a2ui_*_test.rs` names in the
   plan were confirmed accurate:
   `a2ui_application_model_test.rs`, `a2ui_embedder_test.rs`,
   `a2ui_schema_test.rs`, `a2ui_seed_test.rs`, `a2ui_trigger_test.rs`.
   Only `a2ui_embedder_test.rs` (117 lines) exercises code this change
   actually migrates; the other four exercise the base component
   catalog, GraphQL/hybrid-search schema, DB seed data, and Postgres
   triggers from earlier flint-forge migrations (0002–0008) that are
   out of this change's scope (see below). `a2ui_embedder_test.rs`
   itself requires a live Postgres with `pgvector` and an
   `ext-flint-llm` extension providing `llm.embed()` — neither exists
   in UAR, so its scenarios are reproduced as adapted unit/integration
   tests against the ported logic rather than copied verbatim (see
   `tasks.md` §5).

## What Changes

- **New UAR module** `src/uar/a2ui/design_systems/`:
  - `types.rs` — ported domain types (`Renderers`, `DesignToken`,
    `DesignTokenMap`, plus new `DesignSystem` / `Component` /
    `ComponentOverrideRecord` / `ResolvedComponent` records and a
    `resolve_component()` function that replicates flint-forge's
    `flint_a2ui.resolve_components_with_overrides()` SQL function's
    merge semantics in Rust).
  - `design_md_parser/` — the 9-section `DESIGN.md` parser, ported
    near-verbatim from `fdb-app/src/a2ui/design_md_parser/` (pure
    Rust, zero database dependency, same module split and same test
    suite).
  - `store.rs` — a new `DesignSystemStore` trait (backend-agnostic
    CRUD for design systems / components / overrides / embeddings)
    with three implementations, mirroring the existing
    `CredentialStore` pattern in
    `src/uar/security/credentials/store.rs`:
    `InMemoryDesignSystemStore` (always available),
    `SurrealDesignSystemStore` (feature `surreal-backend` — UAR's
    default and `server-full` backend), `PostgresDesignSystemStore`
    (feature `sqlx`/`postgres-backend`).
  - `embedder.rs` — the component embedder, re-architected around
    UAR's existing `EmbeddingBackend` trait
    (`src/uar/rag/embeddings/mod.rs`) instead of flint-forge's
    Postgres-only `LISTEN`/`NOTIFY` + in-database `llm.embed()`
    design. Preserves the primary/fallback-model behavior and the
    startup-backfill behavior as `embed_component` /
    `backfill_missing`.
  - `import.rs` — new glue (`import_design_md`, `import_w3c_tokens`)
    connecting the parser to the store; flint-forge's own
    "apply a parsed DESIGN.md" use case lives in its interface layer,
    which is out of scope here (see below), so this makes the ported
    parser and types usable end-to-end inside UAR.
- **New Postgres migration**
  `migrations/20260714120000_a2ui_design_systems.sql` — creates
  `a2ui_design_systems`, `a2ui_components`, `a2ui_component_overrides`,
  `a2ui_component_embeddings` (Postgres/`pgvector`, feature-gated
  behind `postgres-backend`/`sqlx`).
- **SurrealDB schema addition** in `migrations/surrealdb/schema.surql`
  — `design_systems`, `components`, `component_overrides`,
  `component_embeddings` `SCHEMAFULL` tables, matching the fields the
  Rust structs (de)serialize.
- Unit + integration tests across all four new source files (parser,
  types, store — both in-memory and embedded-SurrealKV — and
  embedder), adapting the intent of flint-forge's
  `a2ui_embedder_test.rs` to UAR's store/backend abstractions.

## Migration approach: source port, not a Cargo git dependency

The plan's done condition says "UAR consumes via Cargo git dep until
the integration stabilizes, then promotes to path dep." This change
implements a **direct source port** instead, for reasons made
explicit per this phase's audit instructions:

- `flint-forge` is a private, unpublished workspace of Cargo crates
  (`fdb-app`, `fdb-gateway`, `fdb-reflection`, ...). It has no crate
  published to crates.io, no `git` tag/release discipline for
  individual crate versions, and (checked from this worktree) no
  externally-fetchable remote configured for depending on it as a
  `{ git = "..." }` Cargo dependency in a way this session could
  verify resolves. There is no forcing function to stand up
  publishing infrastructure just to satisfy this change.
- The ported code is not a drop-in reuse of flint-forge's crates
  regardless: flint-forge's embedder is `sqlx::PgPool`-shaped and
  Postgres/`pgvector`-specific; UAR's default and `server-full`
  release profile runs on SurrealDB (`Cargo.toml`:
  `minimal = ["surreal-backend"]`, `server-full = ["minimal", ...]`
  — `postgres-backend` is opt-in and absent from `server-full`). A
  git dependency on `fdb-gateway`/`fdb-app` would still require a
  from-scratch SurrealDB-backed reimplementation of the storage layer
  to run under UAR's certified build profile, at which point the
  git-dependency wrapper adds indirection without adding reuse.
  Depending on `fdb-app`'s parser alone (its one dependency-free
  piece) for one file's worth of logic is not proportionate either.
- This matches the done condition's own next line — "`flint-forge`
  retains the original" — which already anticipates a fork/port
  relationship rather than a live shared dependency. If flint-forge
  is later extracted into a standalone, versioned, publishable crate
  (e.g. as part of a dedicated SDK-extraction effort), promoting this
  module to depend on it becomes a mechanical follow-up; that
  extraction work is not started by this change.

## Capabilities

### New Capabilities

- `a2ui-design-system-bridge`: the ported `DESIGN.md`/W3C-token import
  pipeline, per-design-system component override storage, and
  component-catalog embedding pipeline.

## Impact

- **New Rust module**: `src/uar/a2ui/design_systems/` (7 files, see
  `tasks.md`), wired into `src/uar/a2ui/mod.rs`.
- **New migrations**:
  `migrations/20260714120000_a2ui_design_systems.sql` (Postgres,
  gated behind the `postgres-backend`/`sqlx` feature — not part of
  the certified `server-full` profile, matching every other
  Postgres-specific table in this repo);
  `migrations/surrealdb/schema.surql` (extended, SurrealDB — the
  profile `server-full` actually ships).
- **Dependencies**: none added. Reuses `async-trait`, `chrono`,
  `serde`/`serde_json`, `thiserror`, `uuid` (all already workspace
  dependencies), `surrealdb` (feature `surreal-backend`, already a
  dependency), `sqlx`/`pgvector` (feature `sqlx`/`postgres-backend`,
  already dependencies), and UAR's existing `EmbeddingBackend` trait
  (`src/uar/rag/embeddings/mod.rs`).
- **No API routes added.** This change lands the storage/import/embed
  layer only. HTTP endpoints for design-system CRUD (analogous to
  `src/uar/a2ui/routes.rs` for artifact schemas) are not part of this
  change's done condition and are left for whichever later change
  wires design systems into an operator-facing API.

## Out of scope

- **`fdb-reflection/src/compilers/a2ui/`'s event→surface assembler**
  (`A2uiAssembler`, assembly-rule resolution, `A2uiPublisher`). Per
  the audit above, this is a distinct subsystem from the design-system
  bridge; migrating it belongs with Change 20
  (`a2ui-realtime-backbone-from-flint-realtime-fabric`), which already
  owns "A2UI surface updates become AG-UI `StatePatch` events" and an
  FRF-based fan-out backbone — the natural home for an event-driven
  surface assembler and its `A2uiPublisher` trait. Attempting it here
  would also require porting `flint_a2ui.assembly_rules` and the
  table→component default-binding schema from flint-forge migrations
  0002–0008, which this change does not migrate (next bullet).
- **The full base component catalog** (`flint_a2ui.components` seeded
  from flint-forge migrations 0002/0004/0006/0008, and the curated
  component library the grade-A plan's Change 18 vendors from
  `prometheus-entity-management` /`flint-realtime-fabric`). This
  change creates a minimal, self-contained `components` table/struct
  with just enough fields to host per-design-system overrides and
  embeddings, but does not seed or vendor a real component catalog —
  UAR has no A2UI component catalog of any kind yet (confirmed: no
  `primitive_type`/`ResolvedComponent`/`component_overrides` hits
  anywhere in `src/` or `openspec/` before this change). Change 18 is
  the natural owner of populating this table for real.
- **Live change notification** (flint-forge's `a2ui_embed` Postgres
  channel). `embedder.rs` exposes `embed_component` (on-demand,
  single component) and `backfill_missing` (startup sweep) as
  library functions; wiring either into a live event stream is
  Change 20's scope (the `flint-realtime-fabric` SSE/fan-out
  backbone), not duplicated here.
- **HTTP routes** for design-system CRUD — see "Impact" above.
- **Cargo git dependency setup** — superseded by the source-port
  decision above; not attempted.
