## 1. Seed and migration order (build item 1)

- [ ] 1.1 Regenerate the schema seed at the latest migration version with `scripts/build-pglite-seed.mjs` (produces `frontend/src/platform/pglite/pglite-seed-v4.tar.gz`), delete `pglite-seed-v3.tar.gz`, and point the import at `frontend/src/platform/pglite/assets.ts:3` to the new file. Do not run the builder before the phase boundary; the regeneration is part of the boundary batch (task 6.1).
- [ ] 1.2 Make the migration loop (`runMigrations`, `client.ts:130-157`) run after every database creation, seeded or not, inside the worker `init` (task 2.1). Reuse the existing idempotent per-version loop; do not add a second mechanism.
- [ ] 1.3 Add a drift guard so the seed import path and the file `build-pglite-seed.mjs` names for the latest version cannot diverge (a check in the existing `pglite:seed:check` script; outside `frontend/src`, needs the lead's single-writer assignment for `scripts/build-pglite-seed.mjs`).
- [ ] 1.4 Add `pglite:seed:check` to the phase's final local gate list. It runs locally only; do not add any GitHub Actions job (repository GitHub Actions policy).

## 2. Worker-hosted, leader-elected database (build item 2)

- [ ] 2.1 Create the worker entry `frontend/src/platform/pglite/pglite.worker.ts` using `worker({ init })` from `@electric-sql/pglite/worker`. `init(options)` loads the PGlite assets with `loadPgliteFsBundle`, `loadPgliteWasmModule` and `loadPgliteSeedForFreshDatabase` from `assets.ts`, creates `PGlite` for `options.dataDir`, runs the migration loop (task 1.2) and returns the instance. Move the migration loop out of `UarDb` into a module the worker imports.
- [ ] 2.2 Rewrite `UarDb.open` (`client.ts:92-125`) to `PGliteWorker.create(new Worker(new URL("./pglite.worker.ts", import.meta.url), { type: "module" }), { id: PGLITE_ELECTION_ID, dataDir: "idb://uar-threads", extensions: { live } })` with `PGLITE_ELECTION_ID = "uar-threads"`. Keep the `UarDb` public methods and the `getDbInstance`/`setDbInstance` singleton API.
- [ ] 2.3 Retype `UarPGlite` (`client.ts:31`) and `getPersistenceClient()` (`client.ts:293`) to the `PGliteWorker` with `live`, narrowing the persistence handle to the `query`/`exec` shape (`PGlitePersistenceClient`). Confirm `RunEventRepository` still receives a client that exposes `live`.
- [ ] 2.4 Add the leader-change retry wrapper in the platform layer around `UarDb` methods, `RunEventRepository` and the persistence client: retry on `LeaderChangedError` up to `DB_WRITE_RETRY_ATTEMPTS` with `DB_WRITE_RETRY_DELAY_MS` between attempts, then reject. Plan fixes the two values.
- [ ] 2.5 Verify by reading that the worker path opens the existing `idb://uar-threads` database name unchanged (`PGLITE_INDEXED_DB_NAME = "/pglite/uar-threads"`, `assets.ts:9`) so existing profiles upgrade in place.
- [ ] 2.6 Move `migrateFromLocalStorage` (`client.ts:159-214`) to a main-thread function run after the database is ready inside `navigator.locks.request("uar-legacy-localstorage-migration", ...)`, re-reading the `uar-pglite-migrated-v1` flag inside the lock.
- [ ] 2.7 Update `frontend/src/lib/db-context.tsx` only as far as the new open sequence requires (it calls `UarDb.open` then `bootstrapDurableEntityGraph(db.getPersistenceClient())`, `db-context.tsx:192-195`).
- [ ] 2.8 Request single-writer assignment from the lead for the files outside `frontend/src`: `frontend/vite.config.ts` (`worker: { format: "es" }`), `scripts/check-frontend-budgets.mjs` and `frontend/performance-budgets.json` (worker-emitted PGlite assets, `vendor-pglite` chunk rule), `frontend/playwright.local-first.config.ts` (new). No dependency pin changes.
- [ ] 2.9 Add two compile-time test flags, inert and compiled out when unset, so negative controls can reject: `VITE_UAR_TEST_OMIT_ELECTION_ID` (omit the election `id`, the defect D2 prevents) and `VITE_UAR_TEST_DB_RETRY_ATTEMPTS` (override `DB_WRITE_RETRY_ATTEMPTS`). Read once via `import.meta.env` in the module that owns the constants. They are set only for scratch mutant builds at the boundary and never in a release build.

## 3. Single snapshot owner and replay (build item 2, multi-tab rule)

- [ ] 3.1 Add a storage-adapter wrapper (platform layer) that gates `set`, `remove` and `setWithCheckpoint` on `db.isLeader` and resolves without writing for non-owners; `get` and `getCheckpoint` pass through. Wire it in `entities/bootstrap.ts` `createStorage` (`bootstrap.ts:36`).
- [ ] 3.2 Pass `replayPendingActions` to `startLocalFirstGraph` only when the tab is leader once the database reports ready (`bootstrap.ts:37-55`).
- [ ] 3.3 On `db.onLeaderChange` to leader: read the stored snapshot, add pending-action records missing from `runtime.scope.pendingActions`, replay the added records with the configured retry policy, then `persistNow()`. Unsubscribe in `teardownEntityGraph`.
- [ ] 3.4 Document the accepted limitation (local-only state in a non-owner tab is not durable until it owns the snapshot) next to the wrapper and in the persistence documentation chosen in Plan (the current persistence documentation file is UNVERIFIED).

## 4. Bounded snapshot (build item 5)

- [ ] 4.1 In `frontend/src/entities/runtime-ingest.ts` `ingestAgUiEvent`, evict the oldest `RuntimeAgUiEvent` entities beyond `RUNTIME_AG_UI_EVENT_RETENTION` (at least 12; Plan fixes the value) through one atomic graph action exported by `@/platform/entities`, never a per-row loop. Do not touch the PGlite `run_event` table.
- [ ] 4.2 Set `persistDebounceMs` explicitly in the `startLocalFirstGraph` call (`bootstrap.ts:37-55`). Plan fixes the value.
- [ ] 4.3 Add the owner-only max-wait scheduler in the platform layer: subscribe to graph changes through the platform entity facade, note the first unpersisted change, call `runtime.persistNow()` after `GRAPH_PERSIST_MAX_WAIT_MS`, reset, single-flight against itself. Start it with the runtime and stop it in `teardownEntityGraph`.
- [ ] 4.4 Relabel the runtime console AG-UI count as retained events (`features/runtime/ui/runtime-console-page.tsx:489`) so the capped number is not read as a total.
- [ ] 4.5 Add a read-only diagnostic hook that exposes per-type entity counts, snapshot bytes, the count of `ingestAgUiEvent` calls, and a reader for the `_graph_snapshot` row digest, for SCN-PGL-007 and SCN-PGL-009 evidence, without changing graph behavior. Do not add further caps for other `Runtime*` types until SCN-PGL-009 shows growth.

- [ ] 4.6 Extend the stub model server so SCN-PGL-009 can stream: add a fixture response kind `stream_content { chunks, interval_ms, chunk_text }` to `tests/integration/live/stub_llm.rs` (streaming currently emits two chunks, lines 355-392) and to the fixture schema in `src/bin/stub-llm.rs:23-41`, then keep `fixtures/stub-llm-pgl-long-stream.json` loadable. This is a Rust change outside `frontend/src`; the lead assigns the owner and the single writer for the test-support files.

## 5. Specification maintenance

- [ ] 5.1 When this change is synced or archived, replace the `## Purpose` text "TBD" of `openspec/specs/frontend-local-first-persistence/spec.md` with a one-paragraph purpose (delta files cannot change Purpose; lead owns the main spec edit).
- [ ] 5.2 Update the Plan inputs: record OQ-1 to OQ-5 from `design.md` as decisions or deferrals.

## 6. Boundary acceptance (runs only at the phase boundary)

These tasks describe the final integration gate. They must not be executed early.

- [ ] 6.1 Run SCN-PGL-001 against stub-llm, with the baseline worktree `pgl-baseline` as its negative control. Also run the supplemental gate SCN-PGL-002: `pnpm --dir frontend pglite:seed:check` against the regenerated seed (expect exit 0 and migrations `[1,2,3,4]`), then against a scratch copy with an added migration (expect non-zero). SCN-PGL-002 is supplemental and never counts as acceptance.
- [ ] 6.2 Build with `pnpm --dir frontend build`, serve `static/` through the real UAR process, and run SCN-PGL-001, SCN-PGL-003, SCN-PGL-004 in real Chromium against `UAR_BASE_URL` with no credentials. Reconcile the bundle budget gate for worker-emitted PGlite assets.
- [ ] 6.3 Run SCN-PGL-006 (run snapshot subscription across tabs through the worker) against stub-llm, 5 samples.
- [ ] 6.4 Run SCN-PGL-007 and SCN-PGL-008 (single owner, queued action; the latter is NOT_EXERCISED without OQ-5), and SCN-PGL-012 inside the packaged desktop app when the phase gate builds The Boss.
- [ ] 6.5 Run SCN-PGL-005 (leader closes during writes): 10 samples on the real bundle and 20 samples on the `VITE_UAR_TEST_DB_RETRY_ATTEMPTS=0` mutant.
- [ ] 6.6 Run SCN-PGL-010 (profile created by the baseline build upgrades) and SCN-PGL-011 (legacy `localStorage` import).
- [ ] 6.7 Run SCN-PGL-009 (long stream) against the extended stub-llm, 10 samples, every sample must pass; record per-type entity counts, snapshot bytes and the machine. A sample whose precondition fails is BLOCKED.
- [ ] 6.8 Record, for every scenario, the exact command, exit code, source identity, bundle hash and receipt path; report BLOCKED rather than skip when a prerequisite is missing.
- [ ] 6.9 A mandatory scenario whose negative control did not run is BLOCKED, never PASS. SCN-PGL-004 uses the `VITE_UAR_TEST_OMIT_ELECTION_ID` mutant bundle as its control.
