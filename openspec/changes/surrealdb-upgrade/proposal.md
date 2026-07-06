# surrealdb-upgrade

## Why

`surrealdb` was pinned `=3.0.5` — 2 minor versions behind crates.io's
current `3.2.0`. `gh api repos/.../dependabot/alerts` showed multiple
**high**-severity open alerts (an HTTP RPC session-UUID leak enabling
anonymous session hijack, and a privilege-escalation race condition via
HTTP RPC) plus 20+ medium-severity entries. `surreal-backend` is this
project's `default` Cargo feature — the embedded SurrealKV backend is
what nearly every deployment actually runs, and the Helm chart's
networked-server path (`ws://surrealdb:8000`) makes the HTTP-RPC
surface plausibly reachable within the cluster network even with the
existing `NetworkPolicy`. The highest alert requirement was `<3.1.5`;
the user asked for the latest version specifically rather than the more
conservative dedicated-patch release, so this bumped to `3.2.0` (actual
crates.io latest), which resolves every open alert.

This is the highest-blast-radius change in the
`uar-security-deps-and-hygiene` phase — `surrealdb` is the default
persistence backend — so it was sequenced last (Round 4, its own
checkpoint) and required checking existing schema/query surface for
breaking changes before landing.

## What changed

Bumped `surrealdb = { version = "=3.2.0", ... }` in `Cargo.toml` (was
`=3.0.5`). `surreal-memory`'s own pin (`surrealdb = "3.0.5"`, a caret
range) is satisfied by `3.2.0` without needing a change in that repo.

**Compatibility review before bumping** (per `docs/DEPENDENCY_MANAGEMENT.md`'s
upgrade SOP and `plan.md`'s explicit instruction to check for breaking
schema/query changes):

- `migrations/surrealdb/schema.surql` — the only SurrealDB schema file
  in this repo (85 `DEFINE` statements: `SCHEMAFULL` tables, `FIELD`,
  `INDEX`, one `RELATION` edge table). All standard, no exotic syntax
  (no `PERMISSIONS`, no full-text `ANALYZER`, no `SPLIT`). Reviewed in
  full — nothing here is affected by 3.0.5→3.2.0.
  - **Correction to this phase's assessment/plan**: both referred to
    "12 SurrealDB migrations." That count is real but describes
    `migrations/*.sql` — which are Postgres/sqlx migrations
    (`sqlx::migrate!("./migrations")` in
    `src/uar/security/credentials/store.rs`), not SurrealDB. The actual
    SurrealDB schema is the single `schema.surql` file above. Noted
    here since it changes where the real risk surface is, not because
    it changes this change's outcome.
- SurrealQL call sites in `src/uar/persistence/providers/surreal.rs`
  and `src/uar/compiler/storage/surreal.rs` — mostly `type::record()`,
  which is correct/current syntax in 3.2.0. Found one **pre-existing**
  inconsistency: `type::thing()` (not `type::record()`) at
  `surreal.rs:524` and `compiler/storage/surreal.rs:71,109`. This
  matches a bug already discovered and separately tracked
  (`task_7c2fd7ee` in `project.json`'s `discoveredBugs`, "rejected by
  pinned SurrealDB 3.0.5, wants type::record") — i.e. already broken at
  `3.0.5`, not a regression introduced by this bump. Left unfixed here,
  deliberately: it's already assigned to a separate in-flight task and
  fixing it wasn't in this change's scope.

No Rust-API breaking changes surfaced (unlike `rmcp-pin-bump` and
`wasmtime-disposition` earlier in this phase, both of which needed
source fixes) — `cargo check` was clean immediately after the bump.

## Verification

- `cargo check --lib`: clean (only 2 pre-existing unrelated `tool_router`
  dead_code warnings).
- `cargo test --lib`: 363/363 green — identical to the pre-upgrade
  baseline.
- `cargo test --test integration`: 56/56 green, 2 pre-existing
  `#[ignore]`d. First run showed 55 passed/1 failed
  (`live::baseline_cases::credential_chain_put_then_list`, "server did
  not become healthy within 10s") — diagnosed as a resource-contention
  flake (58 tests, several spinning up their own embedded server
  instances concurrently under load), not a regression: the same test
  run in isolation passed in 3.82s, and a full rerun of the suite
  passed clean at 56/56.
- `cargo clippy --lib --tests`: zero new warnings; zero warnings at any
  surreal-touched line. Only pre-existing, unrelated `tests/bdd.rs`
  cucumber step-signature warnings (already-carried debt).
- **Live-server smoke check** (this change's dedicated checkpoint,
  given it's the default persistence backend): booted
  `target/debug/universal-agent-runtime` against a scratch
  `surrealkv://` path (`UAR_PERSISTENCE__PROVIDER=surreal`,
  `UAR_PERSISTENCE__DATABASE_URL=surrealkv://<scratch>/data/uar.db`).
  Boot log shows `Settings bootstrapped from config into DB
  seeded=162 updated=0 drift=0 types=27` with zero errors — 162 real
  `UPSERT type::record(...) CONTENT` writes against the embedded
  SurrealKV backend on `3.2.0`. `GET /health` → `200 {"status":"ok"}`.
  `GET /readyz` → `200 {"status":"ready","checks":{"postgres":"ok",
  "surrealdb":"not_configured","mcp":{"status":"ok","tools":6}}}` — the
  `"postgres"` check key is legacy naming; under the default
  `surreal-backend` feature, `state.persistence` is the
  `SurrealProvider`, so this is a genuine `list_skills()` SurrealQL
  query round-trip against the same embedded 3.2.0 database, confirmed
  `"ok"`. Together: a real write path (settings bootstrap) and a real
  read path (readiness check) both round-tripped successfully.

## Note on scope

The disclosed risk in `plan.md` — "if 3.1/3.2 introduce a breaking
change this repo's migrations can't absorb cleanly, stop and re-carry
as debt" — did not materialize. No schema or query breakage found; the
only issue surfaced was the pre-existing, separately-tracked
`type::thing()` bug, which is orthogonal to this version bump.
