## Why

The console's local database has four defects that the phase assessment
(`uar-uiux-architecture-2026-11`, goal G1/G8) confirmed in source. None was run.

1. **A fresh install cannot persist messages in its first session.** `UarDb.open`
   loads a packaged schema seed only when the IndexedDB database does not exist, and
   then skips `runMigrations()` (`frontend/src/platform/pglite/client.ts:101-117`).
   The only seed is `pglite-seed-v3.tar.gz` (`assets.ts:3`) but `MIGRATIONS` ends at
   version 4 (`migrations.ts:78`), which adds `run_id`, `client_turn_id` and
   `history_position` that `insertMessage` writes (`client.ts:262-290`). The seed
   guard `pglite:seed:check` resolves `pglite-seed-v${latestVersion}.tar.gz`
   (`scripts/build-pglite-seed.mjs:122-129`), i.e. `-v4`, which does not exist, so the
   guard would fail today and is only reachable through `budget:proofs`
   (`frontend/package.json:12`).
2. **Every tab opens the IndexedDB database itself on the main thread.** No
   `PGliteWorker`, `navigator.locks` or `BroadcastChannel` exists in `frontend/src`
   (grep, 0 hits). WASM Postgres also runs on the thread that renders and streams.
3. **Every tab writes the whole entity graph to one row.** PEM persists the full
   graph as one JSON text row under key `uar:entity-graph` through
   `createPGlitePersistenceAdapter` (`entity-graph-core/dist/index.mjs:1267-1289`,
   `:2994-3030`; key at `entities/bootstrap.ts:39`). With several tabs, the last
   writer wins and an older tab can overwrite a newer snapshot.
4. **Persistence starves under streaming and the snapshot grows without bound.**
   PEM schedules a trailing 50 ms debounce with no maximum wait
   (`index.mjs:1343,1370-1376`; `persistDebounceMs` is not set in `bootstrap.ts`), so a
   stream that updates the graph faster than 50 ms never persists until it pauses.
   `ingestAgUiEvent` adds one `RuntimeAgUiEvent` entity per event and nothing removes
   them (`entities/runtime-ingest.ts:76-87`); the graph also holds about 8k `Model`
   entities (assessment G1).

## What Changes

- Fix the seed at its source and make migration order independent of how the database
  was created: regenerate the seed at the latest migration, load the seed file the
  builder names, and always run migrations after a seed load.
- Move PGlite into a `PGliteWorker` (`@electric-sql/pglite/worker`, present in the
  installed 0.5.12) with a fixed election id, seed selection and migrations inside the
  worker `init`, and the `live` extension kept on the main thread.
- Handle leader change: retry keyed writes that fail with `LeaderChangedError`.
- Give the graph snapshot a single owner (the worker leader tab) and make pending-action
  replay single-flight across tabs.
- Bound the snapshot: cap retained `RuntimeAgUiEvent` entities, set `persistDebounceMs`
  explicitly, and add an app-side maximum wait that calls `persistNow()`.
- Keep the one-time `localStorage` migration on the main thread, serialized across tabs.

No PEM package change. No change to dependency pins. The pglite-oxide desktop data
layer stays excluded (the Intel-mac spike failed, assessment G1).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-local-first-persistence`: adds requirements for current-schema first use,
  worker-hosted leader-elected PGlite, leader-change safety, single snapshot owner,
  upgrade continuity, one-time legacy migration, and a bounded snapshot; modifies
  `PEM local-first graph lifecycle` so persistence and replay are owned by one tab.

## Impact

- `frontend/src/platform/pglite/**` (client, assets, new worker entry and helpers),
  `frontend/src/entities/bootstrap.ts`, `frontend/src/entities/runtime-ingest.ts`,
  `frontend/src/lib/db-context.tsx`, call sites of `getDbInstance()` keep their API.
- Outside `frontend/src/**`, needing a single-writer assignment from the lead:
  `frontend/vite.config.ts` (worker output format; the default `worker.format` is
  `iife`, `node_modules/vite/dist/node/index.d.ts:3615-3619`),
  `scripts/check-frontend-budgets.mjs` and `frontend/performance-budgets.json`
  (the budget gate requires exactly one `vendor-pglite` chunk in the entry's static
  closure, `check-frontend-budgets.mjs:159`), `scripts/build-pglite-seed.mjs` outputs
  and a new `frontend/playwright.local-first.config.ts`.
- Cross-change: the Model identity and atomic ingestion work in
  `uiux-model-identity-graph-ownership` shrinks the graph this change bounds; no
  ordering dependency, but the boundary measurement should run after both.

## Related open changes

Existing open changes under `openspec/changes/` that overlap. Dispositions are
suggestions; the operator decides in Plan (decision 7). Nothing here was edited.

| Change | Suggested disposition | Reason |
|---|---|---|
| `desktop-data-layer-pglite-oxide` | archive | Empty directory; the pglite-oxide path is excluded by the phase (spike failed on x86_64-apple-darwin). |
| `migrate-settings-mutations-and-retire-stores` | continue (fold the store-retirement items into `uiux-model-identity-graph-ownership`, not here) | 0/15 tasks; concerns Zustand store retirement, not persistence topology. |
| `bootstrap-entity-engine-and-realtime` (8/9 tasks), `configure-entity-engine-and-realtime-bridge` (6/11) | continue, reconcile ledgers in Plan | Engine and realtime bootstrap wiring (`main.tsx`, `lib/entity-engine.ts`). Overlap with this change is limited to graph bootstrap order; the realtime work belongs to `uiux-realtime-topics-and-replica-refresh`. |
| `implement-opencode-suggestions` | archive after verifying | 25/25 tasks checked; its "PGlite-backed offline reading" and storage-health items are not changed here. |
| `uiux-remediation-wave-1` | archive | Empty directory (assessment 0.5). |
