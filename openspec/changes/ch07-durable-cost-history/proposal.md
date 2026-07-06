# ch07-durable-cost-history

## Why

`CostBudgetTracker`'s own module doc comment (`src/uar/runtime/cost_budget.rs:10-11`)
explicitly anticipated this: "Durable roll-ups (SurrealDB/Postgres) can
layer on top by subscribing to the emitted events; that persistence is
intentionally out of scope here." Every deployment restart wiped
accumulated spend, making the CH-07 cost dashboard a
since-last-restart snapshot rather than real history.

## What changed

- **`PersistenceLayer` trait**: added `record_cost_entry(scope,
  scope_id, cost_usd)` and `list_cost_history(scope, scope_id) ->
  Vec<CostEntry>`, both with default no-op implementations (additive,
  non-breaking for any other implementer) alongside a new `CostEntry`
  struct (`scope`, `scope_id`, `cost_usd`, `recorded_at`).
- **SurrealDB**: new `cost_ledger` table
  (`migrations/surrealdb/schema.surql`) — `SCHEMAFULL`, indexed on
  `(scope, scope_id)` and `recorded_at`, matching every other table's
  established convention in that file. Real implementation in
  `surreal.rs` using the SDK-native `.create("cost_ledger").content(...)`
  (auto-generated record id — this is an append-only ledger, not a
  keyed-record store) and a filtered, ordered `SELECT` for history.
- **Postgres**: new migration
  (`migrations/20260706000000_cost_ledger.sql`, applied automatically
  by the existing `sqlx::migrate!("./migrations")` call in
  `PostgresProvider::new`) and a matching real implementation, for
  feature parity per this project's established dual-backend pattern.
- **Wired from `manager.rs`**'s existing cost-recording block (the
  same block `ch06-wire-agent-cost-budget` reads `cost_scope_agent_id`
  from) — one `persist` call per `(scope, scope_id)` already being
  recorded in-memory (`Run`, `Session`, `Agent`, `Global`), via
  `tokio::spawn` **fire-and-forget**, matching the existing per-tool-
  call checkpoint-persistence pattern already in the same function
  (`persistence_for_run.clone()` + `tokio::spawn`). A persistence
  failure logs a warning and does not affect the run — durability here
  is a nice-to-have roll-up, not a hot-path dependency.

## Verification

- **Real round-trip test** against a live embedded SurrealKV instance
  (`cost_ledger_round_trips_against_a_real_embedded_db`,
  `surreal.rs`'s test module): writes 3 entries across 2 scope_ids,
  confirms `list_cost_history` returns only the matching scope_id's 2
  entries, correctly ordered by `recorded_at`, and confirms an unknown
  scope_id returns an empty list — not a mocked or in-memory
  substitute, an actual `SurrealDbProvider::new()` against a scratch
  `surrealkv://` path, per this change's persistence-layer risk
  profile (matching `surrealdb-upgrade`'s established pattern from
  `uar-security-deps-and-hygiene` of a dedicated checkpoint for
  persistence-touching changes).
- `cargo test --lib`: 372/372 green (371 baseline from Round 2 + 1 new
  round-trip test).
- `cargo check --lib` (default + `--features postgres-backend`): both
  clean, same 2/4 pre-existing unrelated warnings respectively.
- `cargo clippy --lib` (default + `--features postgres-backend`): zero
  new warnings, confirmed via `git stash` A/B comparison (508 warnings
  with and without this change's diff under `postgres-backend`, exact
  match).
