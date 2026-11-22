# Round 3 handoff: uar-state, item 2a (R3-2)

Phase: uar-uiux-architecture-2026-11. Classification: implementation (source edit) plus source inspection. No test, build, typecheck, lint, script, browser or install was run.

## Files changed

- `frontend/src/platform/pglite/client.ts` (only product file). `UarDb.open()` now always calls `instance.runMigrations(onStatus)` after `PGlite.create`. The "Versioned schema seed loaded" status line is kept for seeded opens. Diff: +4/-2, lines 113-119.
- `.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/handoffs/round3-state.md` (this file).

Interface effects: none. `UarDb.open` signature, `_instance`/`_openPromise` handling, `runMigrations` body, `MIGRATIONS`, the seed asset and `assets.ts` are unchanged. A seeded open now emits extra status lines ("Bootstrapping schema migrations table...", "Migration 1..3 ... already applied", "Applying migration 4: ...", "Migration 4 applied") through the existing `onStatus`.

## Checks by reading

1. Seed tables vs `runMigrations`: compatible.
   - Seed builder creates `schema_migrations(version INTEGER PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW())`, identical to the `CREATE TABLE IF NOT EXISTS` in `client.ts:133-136` (no-op on a seeded DB). `scripts/build-pglite-seed.mjs:26-31`.
   - Seed rows are inserted per migration with `applied_at = 2000-01-01`. `scripts/build-pglite-seed.mjs:43-48`. The seed is built from the migrations present at build time and v3 holds versions 1-3. `migrations.ts:75-77` define 1-3; migration 4 is at `migrations.ts:78-87`. `runMigrations` selects by `version = $1` (`client.ts:140-143`), so 1-3 are skipped and only 4 is applied.
   - `_uar_seed_metadata(key, value)` with `migration_sha256` is created at `scripts/build-pglite-seed.mjs:33-41`. Nothing in `client.ts` reads or writes it; only the `--check` mode reads it (`:144-151`). It stays stale (digest of migrations 1-3), which is harmless at runtime.
   - Not confirmed from the tarball itself: the binary page contents were not opened (loading it in PGlite is forbidden). The version list 1-3 is inferred from the builder at the commit that produced the seed plus `.prometheus/gotchas.md:2345-2348` ("The seed stops at migration 3").
2. Migration 4 on an empty seeded DB and through `db.exec`: expected to work.
   - `ADD COLUMN IF NOT EXISTS` x3 (`migrations.ts:79-81`) on the empty `messages` table. The `WITH ordered ... UPDATE` (`:82-83`) matches zero rows. `DROP CONSTRAINT IF EXISTS run_status_check` then `ADD CONSTRAINT` (`:84-85`) on the empty `run` table; the inline column CHECK of `RUN_TABLE_SQL` (`:30`) gets the default name `run_status_check`.
   - `db.exec(m.up)` with multiple statements is the same call every migration used for the unseeded path (`client.ts:146`). This is also the exact path a reload takes today on a seeded profile: `hasExistingDatabase()` is then true, so no seed loads and `runMigrations` runs (`assets.ts:34-54`, `client.ts` previous `else` branch). The gotcha entry (`.prometheus/gotchas.md:2345-2349`) records that the reload repairs the profile. Runtime evidence of this stays unverified by me.
   - Crash between `exec` and the `INSERT INTO schema_migrations` (`client.ts:147-150`): migration 4 is fully idempotent (IF NOT EXISTS, `history_position IS NULL` guard, drop-if-exists), so re-running is safe. Pre-existing property, not new.
3. Two tabs / concurrent open on a fresh profile: no new race class found.
   - In-process: `_openPromise` (`client.ts:94-97`, `:100`) makes a second `open()` in one page wait on the first (also the React StrictMode double effect). `_instance` is set only after migrations and `migrateFromLocalStorage` (`:119-122`), so nothing observes a half-migrated instance.
   - Cross-tab: no lock exists. `PGlite.create("idb://uar-threads")` is the only PGlite construction in `frontend/src` (grep: `client.ts:106`). The bundled PGlite 0.5.12 IDB filesystem is Emscripten IDBFS (`DB_VERSION`, `FILE_DATA` store; no `navigator.locks` or `BroadcastChannel` in `node_modules/@electric-sql/pglite/dist`). Each tab holds its own in-memory copy of the data dir and syncs files to IndexedDB, last write wins. This is a pre-existing limit of multi-tab PGlite and affects any write (messages, runs) in two tabs, whether or not migrations run.
   - What changes with this edit: a seeded first open now performs a migration write at open time, not at the first message. Concrete sequence that behaves differently: two pages open on a brand-new profile within the same instant. Both see no IndexedDB database, both load the seed and both apply migration 4 to their own copy; whichever syncs last wins on the shared files. Before the edit both pages would leave the seed schema untouched at open (and fail on first send). I did not find a double-apply that errors: `INSERT INTO schema_migrations` has no `ON CONFLICT`, but a PK clash can only occur inside one data dir, and each tab has its own copy. A tab that opens after the first tab has created the IDB database takes the unseeded path and sees either 1-4 applied (skip) or whatever the first tab had flushed (applies the rest to its own copy), the same as today's reload path.
   - BDD harness: `tests/bdd/support/s1.ts:141-144` creates the operator page lazily, after the chat page exists, in the same browser context (same origin, same IndexedDB). By then the chat page has finished `open()` (it renders only when ready), so the operator page takes the unseeded path and finds version 4 applied. I did not read the operator page's own DB use; if it writes messages concurrently with the chat tab, that is the pre-existing last-write-wins limit above.
   - No speculative guard added.
4. Exposure only after migrations: confirmed.
   - `DbProvider` renders the loading screen until `UarDb.open()` resolves, then hydrates the entity graph from `db.getPersistenceClient()` and only then calls `setDbInstance` and renders children (`frontend/src/lib/db-context.tsx:192-205, 212-214`). `open()` resolves only after `runMigrations` and `migrateFromLocalStorage` (`client.ts:116-123`). `main.tsx:38-42` wraps the app in `DbProvider`.
   - `getDbInstance()` consumers (stores, trace APIs) call it from user actions inside the provider tree; `thread-registry-store.ts:33-36` `hydrateFromDb` is the earliest and runs after the provider is ready. I did not trace each caller for module-load-time use; the grep showed none at import time.
5. Existing tests/scripts that this change contradicts: none found.
   - No test or script asserts the status messages (grep of `frontend/src`, `tests`, `scripts` for "Versioned schema seed", "already applied", "Bootstrapping schema", `runMigrations`, `UarDb.open`).
   - `frontend/src/platform/pglite/assets.test.ts` tests seed selection only, unaffected.
   - Tests that call `getDbInstance`/mock the DB do not call `UarDb.open`.

## Risks and notes

- First open of a new profile is slower by the migration 4 execution; the seed's size and load path are unchanged.
- Pre-existing, not fixed: `pnpm pglite:seed:check` (`scripts/build-pglite-seed.mjs:122-127`, `:129`) derives the file name from the latest migration (now `pglite-seed-v4.tar.gz`), which does not exist; `frontend/package.json:12` `budget:proofs` runs it. `assets.ts:3` and `scripts/test-frontend-budget-gate.mjs` still name v3. Regenerating the seed was out of scope.
- Pre-existing, not fixed: no cross-tab lock for the IDB-backed PGlite.
- A seed regenerated at v4 later makes the migration loop a pure no-op; this edit stays correct for that.

## Unverified

Everything above is source reading. Not run: typecheck, lint, tests, build, any browser. Not confirmed: the actual rows inside `pglite-seed-v3.tar.gz`, migration 4 executing on that data directory in a browser, and the SYS-001 first-message behavior. These belong to the full-phase integration gate.
