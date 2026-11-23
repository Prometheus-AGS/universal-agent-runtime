## Context

Phase `uar-uiux-architecture-2026-11`, build items 1, 2 and 5 (analysis.md "Build
required"). Source facts below were read at the cited lines in this checkout.
Nothing was built or run; every runtime claim is marked UNVERIFIED.

### Verified in source

| Fact | Where |
|---|---|
| PGlite opens on the main thread: `PGlite.create("idb://uar-threads", { fsBundle, loadDataDir, pgliteWasmModule, extensions: { live } })` | `frontend/src/platform/pglite/client.ts:106-111` |
| A seed load skips `runMigrations()`; otherwise migrations run | `client.ts:113-117` |
| The seed is fetched only if `indexedDB.databases()` lacks `/pglite/uar-threads`; if `databases` is unavailable it reports "exists" and loads no seed | `assets.ts:35-52` |
| The only seed imported is `pglite-seed-v3.tar.gz`; `MIGRATIONS` ends at version 4 | `assets.ts:3`, `migrations.ts:75-78` |
| Seed builder writes and checks `pglite-seed-v${latestVersion}.tar.gz` (here `-v4`, absent) and the seed carries `schema_migrations` rows plus a migration digest | `scripts/build-pglite-seed.mjs:122-129,25-49` |
| `pglite:seed:check` is referenced only from `budget:proofs` and the `pglite:seed:check` script; no workflow references it | `frontend/package.json:12,15`; grep of `.github/workflows` found none |
| Legacy `localStorage` migration lives in `UarDb` and uses `localStorage` | `client.ts:159-214` |
| `getPersistenceClient()` returns the `PGlite` handle that PEM's adapter uses; PEM only needs `query` and `exec` | `client.ts:293`; `entity-graph-core/dist/index.d.ts:2184-2188` |
| Run snapshots use `db.live.query` | `run-event-repository.ts:276,287` |
| Keyed, idempotent writes: `insertMessage` and `upsertThread` use `ON CONFLICT (id) DO UPDATE`; `startRun` `ON CONFLICT (id) DO UPDATE`; `appendEvent` `ON CONFLICT (run_id,event_id) DO NOTHING`; `finishRun` is a guarded `UPDATE` | `client.ts:226-290`; `run-event-repository.ts:151-228` |
| Callers discard write failures: `void getDbInstance().upsertThread(...)`, `.insertMessage(...).catch(() => {})` | `stores/thread-registry-store.ts:60-98`; `stores/chat-message-store.ts:1171` |
| `PGliteWorker.create(worker, options)` posts `options` minus `extensions` to the worker; extensions such as `live` run client-side; election lock name is `pglite-election-lock:${options.id ?? `${import.meta.url}:${dataDir}`}`; only the lock holder calls `init`; a pending RPC at handover rejects with `LeaderChangedError` | installed `dist/worker/index.js` (minified), `dist/worker/index.d.ts` |
| `PGliteWorker` exposes `isLeader`, `onLeaderChange`, `waitReady`, `close` | `dist/worker/index.d.ts` |
| PEM `persistGraphToStorage` clones and `JSON.stringify`s the whole snapshot before `storage.set`; `hydrateGraphFromStorage` calls `store.setState(parsed.snapshot)` (replaces, does not merge) | `entity-graph-core/dist/index.mjs:1267-1289,1290-1316` |
| PEM persists on a trailing `persistDebounceMs` (default 50) timer reset by every store change; `runPersist` swallows errors; `persistNow()` exists on the runtime; replay of pending actions runs inside `ready` when `replayPendingActions` is true | `index.mjs:1343,1370-1450` (`persistNow` at 1444, replay at 1406); `index.d.ts:946,966` |
| `ingestAgUiEvent` upserts one `RuntimeAgUiEvent` per row; the only reader renders a count | `entities/runtime-ingest.ts:76-87`; `features/runtime/ui/runtime-console-page.tsx:482,489,520` |
| The budget gate requires exactly one named `vendor-pglite` chunk in the entry's static closure and PGlite `.wasm`/`.data`/seed assets | `scripts/check-frontend-budgets.mjs:11-18,150-160` |
| Vite 8.3.5 worker bundle format defaults to `iife` | `frontend/node_modules/vite/dist/node/index.d.ts:3615-3619` |

### Not verified (UNVERIFIED)

- That `live.query` and `listen` behave correctly through `PGliteWorker` in 0.5.12.
  Upstream documents the `extensions: { live }` pattern (PGlite multi-tab worker docs,
  retrieved through Context7), but the repository has never run it.
- That `WebAssembly.Module`, `Blob` seed and `fsBundle` can be posted to the worker, which
  is why asset loading moves into the worker `init` instead.
- That `indexedDB.databases()` is available in every worker target.
- That Vite 8 emits worker-imported `?url` assets into the manifest the budget gate reads.
- That the desktop shell webview (The Boss) exposes dedicated module workers, Web Locks and
  `BroadcastChannel`.
- Whether the stale seed reproduces in a fresh browser (assessment: "Runtime confirmation
  is required").

## Goals / Non-Goals

Goals: a fresh install persists its first session; two or more tabs share one database
with one leader; the graph snapshot cannot be overwritten by a stale tab; streaming keeps
persisting and the snapshot stops growing with events.

Non-goals: PEM package changes (phase exclusion), the pglite-oxide desktop data layer,
bumping `@electric-sql/pglite` from 0.5.12 (the worker API ships in 0.5.12; a bump is a
Plan decision and a pin change this change does not make), moving snapshot serialization
off the main thread (the PEM graph store is a main-thread singleton, assessment G8),
surfacing write failures that callers currently discard.

## Decisions

### D1 Seed: regenerate and always migrate

Do both. Regenerate the seed so it is a startup optimization that matches `MIGRATIONS`,
and run the idempotent migration loop after every database creation so a stale seed is
never a correctness failure again. `runMigrations` already selects per version
(`client.ts:130-157`), so rerunning it on a seeded database costs one query per migration.
Rejected alternative: regenerate only. The defect class is "seed lags migrations"; a
regeneration fixes today's instance and the unenforced guard will fail again at migration 5.

The seed import in `assets.ts:3` becomes the file the builder names for the latest version.
A guard compares the two so renaming one without the other fails the check.

### D2 Worker topology

`UarDb.open` constructs `PGliteWorker.create(new Worker(new URL("./pglite.worker.ts",
import.meta.url), { type: "module" }), { id: PGLITE_ELECTION_ID, dataDir:
"idb://uar-threads", extensions: { live } })`. `PGLITE_ELECTION_ID` is the constant
`"uar-threads"`, so the lock name is `pglite-election-lock:uar-threads` in every build.
Without `id`, the lock name embeds the worker script URL, which is hashed per build
(minified source: `n.id ?? \`${import.meta.url}:${n.dataDir ?? ""}\``), so a tab from the
previous deploy and a tab from the new deploy would each elect a leader and both open the
same IndexedDB database. That is the failure this decision prevents.

The worker module calls `worker({ init })`. `init(options)` does, in order:
`loadPgliteFsBundle()`, `loadPgliteWasmModule()`, `loadPgliteSeedForFreshDatabase()`
(existence is evaluated at election time, so a follower that later becomes leader never
loads a seed over an existing database), `PGlite.create(options.dataDir, ...)`, then the
migration loop. `assets.ts` is reused unchanged apart from the seed import.

`UarDb` keeps its public methods; its field becomes the `PGliteWorker` with the `live`
extension. `getPersistenceClient()` widens to the `query`/`exec` shape PEM needs.

### D3 Leader change: retry keyed writes

Every current repository write is keyed and idempotent (table above), so a write that
fails with `LeaderChangedError` ("pending operation in indeterminate state") may have
applied; a bounded retry reapplies it safely. The wrapper lives in the platform layer
around `UarDb` methods, `RunEventRepository` and the persistence client. Bounds are named
constants (`DB_WRITE_RETRY_ATTEMPTS`, `DB_WRITE_RETRY_DELAY_MS`), values fixed in Plan.
`appendEvent` returns `null` on a replayed identity; its only caller ignores the value
(`run-event-persistence.ts:280`).

Reads are retried the same way. Transactions are not wrapped: PGlite rolls back a tab's
open transaction when the tab's lock is released.

### D4 Snapshot owner = worker leader

Decision: the `PGliteWorker` leader (`db.isLeader`, `db.onLeaderChange`) is the only tab
that writes the graph snapshot and replays pending actions.

- A wrapper around the storage adapter that `createPGlitePersistenceAdapter` returns gates
  `set`, `remove` and `setWithCheckpoint` on ownership; non-owners resolve without writing
  so PEM's status does not record an error. `get` and `getCheckpoint` are never gated.
- `startLocalFirst` passes `replayPendingActions` only when the tab is the leader once the
  database reports ready (`isLeader` is true by then for the first tab: the worker posts
  `leader-now` before the `connected` message `waitReady` waits for; UNVERIFIED at runtime).
- On `onLeaderChange` to leader: read the stored snapshot, add pending-action records missing
  from `runtime.scope.pendingActions`, replay the added records with the configured retry
  policy, then call `persistNow()`. Adoption is needed because `hydrateGraphFromStorage`
  replaces state, so a new owner's graph does not contain the previous owner's queued
  actions and its first persist would delete them.

Rejected alternatives:

1. Per-tab snapshot keys. Each tab keeps a private row; storage grows with tab count, stale
   rows are never collected, and there is still no merge. Rejected.
2. Foreground tab owns the snapshot (Web Lock with `steal`). It protects the operator's
   active tab but still overwrites, because hydration replaces state; local-only state in the
   previous owner is lost at every switch. It also adds a second election beside PGlite's.
   Rejected for this phase; recorded as OQ-1.
3. Merge on write. PEM has no merge API and PEM changes are excluded.

Accepted cost, stated plainly: local-only graph state created in a non-owner tab is not
durable until that tab becomes owner. Server-confirmed entities are not affected, because
the sync transport started after hydration re-delivers them (`entities/bootstrap.ts:98`, `initSyncTransport`; refetch behavior per the analysis, not re-read here). Follower tabs
still pay the snapshot serialization on each PEM debounce even though the write is dropped,
since `persistGraphToStorage` clones and stringifies before `storage.set`
(`index.mjs:1267-1275`). Plan may give followers a longer `persistDebounceMs`; not decided.

### D5 Bounded snapshot

- `RuntimeAgUiEvent` retention: `ingestAgUiEvent` evicts the oldest entities beyond
  `RUNTIME_AG_UI_EVENT_RETENTION` after each upsert. The durable copy is the PGlite
  `run_event` table, which eviction never touches. The only graph reader shows a count and lists `agUi.slice(0, 12)`
  (`runtime-console-page.tsx:489,520`), so retention must be at least 12; the count label
  becomes "retained events" so the number is not misread. Whether `useEntities` returns
  insertion order is UNVERIFIED, so the displayed 12 may be the oldest retained.
- `persistDebounceMs` is set explicitly in `bootstrap.ts` instead of inheriting 50.
- Max wait: a small platform-layer scheduler, active only in the owner tab, subscribes to
  graph changes through the platform entity facade (never the package directly), records
  the time of the first unpersisted change, and calls `runtime.persistNow()` when
  `GRAPH_PERSIST_MAX_WAIT_MS` elapses, then resets. It is single-flight against its own
  call; PEM's timer may also persist, and two valid snapshots racing is harmless because the
  last `UPSERT` wins.

Other `Runtime*` entity types written by `ingestRuntimeEvent` (`RuntimeRun`,
`RuntimeToolCall`, ...) also never leave the graph (`runtime-ingest.ts:55-74`). Baseline
growth is UNKNOWN. SCN-PGL-009 records per-type entity counts and snapshot bytes so Plan can
decide whether a further cap is needed; this change does not add speculative bounds.

### D6 Main-thread legacy migration

`migrateFromLocalStorage` moves out of `UarDb.open` into a main-thread function that runs
after the database is ready and inside `navigator.locks.request("uar-legacy-localstorage
-migration")`; the flag `uar-pglite-migrated-v1` is read again inside the lock. It keeps
using `upsertThread` and `insertMessage`, both idempotent. Pre-existing risk, not changed
here: its `catch` blocks swallow failures and it removes the legacy keys regardless.

## Risks / Trade-offs

- **Budget gate breaks.** Moving PGlite into a worker changes which chunk holds it;
  `check-frontend-budgets.mjs:159` demands exactly one `vendor-pglite` chunk in the entry's
  closure and the `.wasm`/`.data`/seed assets in the manifest. The main thread still imports
  the `PGliteWorker` client and `live` from `node_modules/@electric-sql/pglite/`, which the
  chunk rule at `vite.config.ts:30-31` maps to `vendor-pglite`, so the chunk probably remains;
  the assets are the unknown. Task 6.2 reconciles the gate; files are outside `frontend/src`.
- **Worker output format.** `worker.format` defaults to `iife`; a module worker that imports
  PGlite needs `es`. One line in `vite.config.ts`, outside `frontend/src`.
- **Leader handover is observable.** Writes queue during election; the retry bound defines
  how long a user waits before a rejected write that callers then discard. Surfacing it is
  out of scope.
- **First paint.** The first tab now waits for worker start, asset fetch and election before
  hydration; today it waits for the same assets on the main thread. Net effect UNKNOWN; the
  boundary run records the time to "Database ready" for both builds.
- **Safari and WebKit.** `navigator.locks`, `BroadcastChannel` and module workers need
  Safari 15.4 or newer (training-era knowledge, not verified here); the desktop shell is the
  supported WebKit target (SCN-PGL-012).
- **Status lines.** `onStatus` messages from migrations (`client.ts:130-157`) are produced
  inside the worker after this change; the loading screen may show fewer lines. Not a
  requirement; Plan chooses a `BroadcastChannel` relay or drops them.

## Open questions

- **OQ-1 (Plan, operator):** accept "leader owns the snapshot" or switch to
  "foreground tab owns it" (rejected alternative 2). Default: leader.
- **OQ-2 (Plan):** retention and max-wait values; baseline UNKNOWN until SCN-PGL-009 runs.
- **OQ-3 (Plan):** bump `@electric-sql/pglite` 0.5.12 to 0.5.17 (registry latest per
  `library-candidates.json` cand-001). Not needed for the worker API; changelog unread.
- **OQ-4 (Plan):** whether follower tabs should lengthen `persistDebounceMs` to cut wasted
  serialization.
- **OQ-5 (Plan):** an operation that enqueues a keyed graph action while the server is
  unreachable, needed as the fixture for SCN-PGL-008 (candidate:
  `bindA2uiActionPersistence`, `entities/bootstrap.ts:99-101`; not verified as a user-facing
  path).

## Acceptance

Common to every table: the console is the built SPA served by the real UAR process from
`static/` at `UAR_BASE_URL`; the browser is real Chromium driven by Playwright; the database
is the real IndexedDB-backed PGlite in the worker. No credentials, tokens or JWTs are
created, sent or required by any scenario (operator rule 2026-10-26; the external proxy
supplies identity). The final local gate runs only at the phase boundary; the Playwright
config and specs are authored at that boundary and reviewed independently against these rows.

Model replies come from the local stub model server, never a paid route. Invocation, read
from source: the `stub-llm` binary is run with a fixture file as its first argument and the
environment variable `STUB_LLM_PORT` (`src/bin/stub-llm.rs:84-106`; the same shape is used at
`tests/bdd/playwright.config.ts:70-72`). The binary is built by the Rust step of the
`test:bdd:prepare` recipe (`package.json:14`: binaries `stub-llm` and `universal-agent-runtime`,
`--features server-full`). UAR is pointed at it with `UAR_LLM__BASE_URL=http://127.0.0.1:<stub
port>/v1` and `UAR_LLM__MODEL=openai/gpt-5.4-mini`; the key `test-stub-key` is a stub
placeholder, not a credential. The stub matches requests by exact fingerprint (model, last user
message, tools flags), which is why the fixtures list both tool flags.

Baselines: git worktrees created with `scripts/worktree-new.sh <name> --base f2c9e36e` under
`~/.claude/worktrees/` (project rule) and built with the frontend build script. The lead adds
the build commands, the clean-build rule and receipts to the boundary packet.

Mutant builds: two compile-time test flags (task 2.9) produce deliberately defective bundles in
a scratch output directory so negative controls can reject: `VITE_UAR_TEST_OMIT_ELECTION_ID`
and `VITE_UAR_TEST_DB_RETRY_ATTEMPTS`. They are unset in every release build.

## Frozen oracle fixtures

Written now, read by the boundary runner, never produced by the implementation (rule R1).
Values with status `proposed` change only by editing the fixture and this document together
before the contract is adopted.

| File | Content |
|---|---|
| `fixtures/pgl-oracle.json` | election lock name `pglite-election-lock:uar-threads`; IndexedDB name; migration versions `[1,2,3,4]`; required `messages` columns; stub model; first-session prompt and expected occurrences; handover counts (20 created, leader closed after 5, expected delta 20); other-origin control; proposed bounds (retention 500, max wait 2000 ms, tolerance 1000 ms, retry attempts 5, delay 200 ms); long-stream parameters and precondition; baseline ref and worktree name |
| `fixtures/stub-llm-pgl.json` | stub fixtures for the first-session prompt (both tool flags); loads under the current stub |
| `fixtures/stub-llm-pgl-long-stream.json` | stub fixture for the long stream using the new kind `stream_content` (2000 chunks, 20 ms apart); loads only after task 4.6 extends the stub; the current stub would fail to parse the kind |

## Behavior B1: first session on a fresh profile persists

| Field | Declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-local-first-persistence-worker` / tasks 1.1-1.4 |
| Source identity | repository HEAD at gate time; built `static/` bundle hash recorded in the receipt |
| Production entry point | Operator opens the console on a new browser profile and sends the prompt `uar-pgl-first-session-001` in a new conversation (Chat page) |
| Real collaborators | UAR server serving `static/`, stub-llm, Chromium, real IndexedDB, PGlite WASM in the worker |
| Boundary exercised | Browser IndexedDB `/pglite/uar-threads`; `messages` columns `run_id`, `client_turn_id`, `history_position` |
| Observable result | After a reload the conversation history shows the prompt exactly once (`firstSession`, `fixtures/pgl-oracle.json`) |
| Negative control | Baseline build (worktree `pgl-baseline`, main-thread seed v3) under the same steps: the message is absent after reload, so the oracle rejects |
| Isolation | New browser context; baseline in `~/.claude/worktrees/pgl-baseline`; stub-llm on its own port |
| Prerequisites | stub-llm built and running with `fixtures/stub-llm-pgl.json`; baseline worktree built |
| Final local gate | Playwright run of SCN-PGL-001 with `playwright.local-first.config.ts` (config authored at the boundary) |
| Evidence | pending: receipt path assigned by the lead's boundary packet |
| Limitations | Chromium only. Firefox without `indexedDB.databases()` never loads a seed (`assets.ts:35-38`), which is the migrate-from-empty path |

### Supplemental gate S1 (static; not a behavior scenario)

`pnpm --dir frontend pglite:seed:check` plus a scratch copy with an added migration. It runs real
PGlite in Node but does not exercise the browser, so by the packet rule it cannot establish
acceptance; it is recorded as SCN-PGL-002 with classification `supplemental` and `mandatory: false`.
Behavior acceptance for the stale seed is SCN-PGL-001.

## Behavior B2: tabs share one worker-hosted database

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / tasks 2.1-2.6 |
| Source identity | as B1; two built bundles for the different-build case |
| Production entry point | Operator opens the console in tab 1, then tab 2, same browser context; creates a conversation in each; opens a run trace in one while the other sends the prompt `uar-pgl-first-session-001` |
| Real collaborators | UAR server, stub-llm, Chromium, IndexedDB, Web Locks, `BroadcastChannel`, PGlite worker, `live` extension |
| Boundary exercised | `navigator.locks` election lock; worker-to-tab RPC; `subscribeRunSnapshot` live query |
| Observable result | `page.workers()` shows one dedicated worker per tab; `navigator.locks.query()` shows exactly 1 held election lock named `electionLockName` and (tabs-1) pending requests; a conversation created in either tab appears in the other; the run subscriber receives the appended events |
| Negative control | (a) Baseline build, same steps: no `pglite-election-lock:*` held, oracle rejects. (b) Different-build case with a mutant bundle built with `VITE_UAR_TEST_OMIT_ELECTION_ID` (task 2.9): two distinct held election locks, oracle rejects. (c) Run subscriber closed before events: no update is received |
| Isolation | One browser context, tabs from `context.newPage()`; the second build is served by replacing files in the scratch static directory (`UAR_STATIC_DIR`) between tab opens |
| Prerequisites | stub-llm with `fixtures/stub-llm-pgl.json`; two distinct built bundles; the mutant bundle; baseline worktree |
| Final local gate | Playwright run of SCN-PGL-003, SCN-PGL-004, SCN-PGL-006 |
| Evidence | pending |
| Limitations | Chromium only at the gate; the desktop shell is B2-desktop |

## Behavior B2-desktop: the worker build opens in the desktop shell

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / task 6.4 |
| Production entry point | The packaged desktop app from the phase's `pnpm build:mac:arm64` gate, launched normally; operator creates a conversation, quits, relaunches |
| Real collaborators | The packaged app, its webview, its bundled frontend, real local IndexedDB |
| Boundary exercised | Module worker creation, Web Locks and `BroadcastChannel` inside the webview |
| Observable result | The conversation created before quitting is present after relaunch |
| Negative control | Relaunch with the app data directory emptied: the conversation is absent and the app still starts |
| Isolation | Scratch application data directory per run |
| Prerequisites | The phase gate builds The Boss (project rule); how a session drives the packaged app is UNVERIFIED |
| Final local gate | Part of the phase's The Boss gate; command and artifact recorded by the lead |
| Evidence | pending |
| Limitations | A webview missing any of the three APIs fails this scenario; no fallback is specified |

## Behavior B3: leader handover does not lose writes

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / tasks 2.4, 2.9, 6.5 |
| Production entry point | Tab 1 (leader, opened first) and tab 2 (follower); tab 2 creates 20 conversations with the New conversation control; tab 1 is closed after the fifth (`handover`, `fixtures/pgl-oracle.json`) |
| Real collaborators | As B2 |
| Boundary exercised | Election handover; `LeaderChangedError` retry in the database wrapper |
| Observable result | After a reload of tab 2 the conversation count equals the starting count plus 20 and no conversation is duplicated, in every one of 10 samples |
| Negative control | Mutant bundle built with `VITE_UAR_TEST_DB_RETRY_ATTEMPTS=0` run for 20 samples: at least one sample ends below start+20, so the oracle can reject |
| Isolation | New browser context per sample |
| Prerequisites | The mutant bundle (task 2.9) |
| Final local gate | Playwright run of SCN-PGL-005 |
| Evidence | pending |
| Limitations | Conversations are the write workload because it needs no model reply. Thread and message writers discard rejections (`thread-registry-store.ts:60-98`), so the oracle reads database content after reload. The kill moment relative to an in-flight RPC is not controlled; that is why the scenario is sampled |

## Behavior B4: one snapshot owner, one replay

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / tasks 3.1-3.4 |
| Production entry point | Two tabs; a graph-changing operator action in the non-owner tab; the owner tab closed; for SCN-PGL-008 an operator action that enqueues a keyed graph action while the server is unreachable |
| Real collaborators | As B2 plus the PEM local-first runtime and the `_graph_snapshot` table |
| Boundary exercised | `_graph_snapshot` row for key `uar:entity-graph`; `graphSyncStatusStore.lastPersistedAt` |
| Observable result | The row's digest and `updated_at` do not change when the non-owner edits; after the owner closes the row equals the survivor's graph; a queued action is replayed once |
| Negative control | Baseline build, same steps: the non-owner's edit does change the row (every tab writes), so the oracle rejects. SCN-PGL-008: the same sequence with the action already settled before reload must show no replay |
| Isolation | New browser context; offline through Playwright context offline mode, restored after |
| Prerequisites | A read path to the row (a diagnostic reader added by task 4.5, or a third tab opening a read-only client, UNVERIFIED). SCN-PGL-008 needs the fixture from OQ-5 |
| Final local gate | Playwright run of SCN-PGL-007 and SCN-PGL-008 |
| Evidence | pending |
| Limitations | SCN-PGL-008 is NOT_EXERCISED until OQ-5 names a real user path and is non-mandatory; it is never replaced by a mock |

## Behavior B5: streaming keeps persisting and the snapshot stops growing

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / tasks 4.1-4.6 |
| Production entry point | Operator sends the prompt `uar-pgl-long-stream-001`; stub-llm streams 2000 chunks 20 ms apart; in a second run the tab is killed mid-stream |
| Real collaborators | UAR server with stub-llm (extended, task 4.6), Chromium, PGlite worker |
| Boundary exercised | `/api` run stream into `ingestAgUiEvent`; PEM debounce and `persistNow()`; `run_event` table |
| Observable result | `lastPersistedAt` advances at least once per `graphPersistMaxWaitMs` during the stream; the `RuntimeAgUiEvent` count never exceeds `runtimeAgUiEventRetention`; after a kill and reload the hydrated graph is no older than `graphPersistMaxWaitMs + persistDurationToleranceMs`; per-type entity counts and snapshot bytes are recorded |
| Negative control | Baseline build: `lastPersistedAt` does not advance during the stream and the count grows with events, so the oracle rejects |
| Isolation | New browser context per sample |
| Prerequisites | stub-llm extension (task 4.6); `fixtures/stub-llm-pgl-long-stream.json`; precondition that ingested events exceed retention, else the sample is BLOCKED |
| Final local gate | Playwright run of SCN-PGL-009, 10 samples |
| Evidence | pending |
| Limitations | Baseline likelihood UNKNOWN (assessment VS-10); tolerance is hardware-dependent and the runner records the machine; the stub's chunking may not map one chunk to one AG-UI event, which the precondition checks |

## Behavior B6: existing data and legacy data survive

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-local-first-persistence-worker` / tasks 2.5, 2.6, 6.6 |
| Production entry point | A profile populated by the baseline main-thread build (threads, messages, a run, a graph snapshot) is opened by the new build; for legacy data, `localStorage` keys `uar-thread-registry` and `uar-chat-messages` seeded and two tabs opened together |
| Real collaborators | Chromium persistent profile directory, real IndexedDB and `localStorage` |
| Boundary exercised | Upgrade of an existing IndexedDB database; `navigator.locks` guard around the legacy migration |
| Observable result | Every earlier thread and message is visible with counts equal to those recorded before the upgrade; each legacy thread and message appears once; flag set and legacy keys removed |
| Negative control | The same server opened at its other origin (`localhost` instead of `127.0.0.1`, a different IndexedDB origin) shows an empty database, so the oracle rejects an absent profile. Legacy: flag already set, no import occurs |
| Isolation | Playwright persistent context in a scratch user-data directory copied per run |
| Prerequisites | Baseline build artifact |
| Final local gate | Playwright run of SCN-PGL-010 and SCN-PGL-011 |
| Evidence | pending |
| Limitations | SCN-PGL-011 covers legacy data few users hold and is non-mandatory (the lead should keep "one-time legacy import" out of the mandatory functional scope) |

## Scenario matrix rows

Scenario IDs are fixed. "Source requirement" values are `openspec:<capability>#<Requirement
name>`, capability `frontend-local-first-persistence`. Oracle is observation / expected truth
source / comparator; every expected-value source is a file in `## Frozen oracle fixtures` or a
recorded pre-action observation, never implementation output. Owner tasks refer to `tasks.md`.

### SCN-PGL-001 Fresh profile first session persists chat messages

- Actor: operator on a new install. Useful outcome: the first conversation survives a reload.
- Source requirement: `openspec:frontend-local-first-persistence#Fresh-install schema is current before first use`.
- Reason now: confirmed in source, certain on every new install (assessment VS-7). Likelihood: certain by source reading; runtime UNKNOWN. Mandatory: true. Nondeterministic: false.
- Production entry: new Chromium context, `UAR_BASE_URL/`, send `uar-pgl-first-session-001` to the stub-backed model, reload.
- Real collaborators: UAR static bundle, stub-llm, Chromium, IndexedDB, PGlite worker.
- Oracle: observation = conversation history after reload; truth = `firstSession.prompt` in `pgl-oracle.json`; comparator = exact text present `expectedHistoryOccurrences` times.
- Negative control: baseline build `pgl-baseline`, same steps; expected rejection = message absent after reload.
- Fixtures: empty context; `stub-llm-pgl.json`. Evidence required: Playwright trace, reloaded-history screenshot, both bundle hashes.
- Substituted boundaries: model replies come from stub-llm (reply content not asserted). Limitations: Chromium only.
- Prerequisites: stub-llm and baseline worktree built (lead adds commands).
- Invalidation keys: `frontend/src/platform/pglite/{client.ts,assets.ts,migrations.ts,pglite.worker.ts,pglite-seed-v*.tar.gz}`, `scripts/build-pglite-seed.mjs`, `@electric-sql/pglite` version, `fixtures/pgl-oracle.json`.
- Restoration: close contexts; remove the baseline worktree with `scripts/worktree-rm.sh pgl-baseline`. Owner tasks: 1.1, 1.2, 2.1, 2.2, 6.1.

### SCN-PGL-002 Seed drift guard fails closed (supplemental, static)

- Actor: release engineer. Outcome: a stale seed cannot ship unnoticed.
- Source requirement: `#Fresh-install schema is current before first use` (scenario "Seed drift is detected").
- Classification: supplemental. Mandatory: false. It is a structural script check and does not satisfy acceptance by itself; SCN-PGL-001 does.
- Production entry: `pnpm --dir frontend pglite:seed:check` from the repository root; and the same script in a scratch copy.
- Real collaborators: Node, `@electric-sql/pglite`, esbuild, the committed seed and `migrations.ts`.
- Oracle: observation = exit code and last JSON line; truth = `migrationVersions` in `pgl-oracle.json`; comparator = exit 0 with `"status":"valid"` and versions equal for the repository, non-zero for the scratch copy.
- Negative control: scratch copy with migration 5 added and the old seed; expected rejection = non-zero exit naming a migration or digest mismatch.
- Fixtures: scratch root in the session scratchpad, `node_modules` symlinked. Evidence required: argv, exit codes, stdout.
- Substituted boundaries: none. Limitations: does not exercise the browser. Nondeterministic: false.
- Invalidation keys: `scripts/build-pglite-seed.mjs`, `migrations.ts`, the seed file. Restoration: delete scratch root. Owner tasks: 1.1, 1.3, 6.1.

### SCN-PGL-003 Two tabs, one leader, no IndexedDB contention

- Actor: operator with two console tabs. Outcome: both tabs show the same data and nothing corrupts.
- Source requirement: `#Local database runs in a leader-elected worker`. Reason now: unspecified hazard today (assessment VS-8). Likelihood: medium. Mandatory: true.
- Nondeterministic: true (election and tab timing). Stochastic: samples 5; grader = exactly one held lock named `electionLockName`, pending count equal to tabs-1, and equal conversation lists in both tabs; stoppingRule = fixed N=5, all must pass.
- Production entry: open tab 1, create a conversation, open tab 2, create another.
- Real collaborators: as B2.
- Oracle: observation = `navigator.locks.query()` and each tab's conversation list; truth = `electionLockName` from `pgl-oracle.json` and the conversations created; comparator = equality.
- Negative control: baseline build; expected rejection = no election lock held.
- Fixtures: one context, two pages. Evidence required: lock query JSON per sample, screenshots.
- Substituted boundaries: none. Limitations: Chromium only.
- Invalidation keys: `pglite.worker.ts`, `client.ts`, `vite.config.ts` worker settings. Restoration: close pages. Owner tasks: 2.1, 2.2, 2.8, 6.2.

### SCN-PGL-004 Tabs from different builds elect against one lock

- Actor: operator during a deploy. Outcome: no two writers on one database.
- Source requirement: `#Worker election identity does not depend on the build`. Reason now: worker script URL is hashed per build. Likelihood: medium. Mandatory: true. Nondeterministic: false.
- Production entry: load build A in tab 1, replace the files in the `UAR_STATIC_DIR` directory with build B, load tab 2.
- Real collaborators: UAR static serving, two bundles, Chromium.
- Oracle: observation = held election lock names; truth = `electionLockName`; comparator = exactly one held lock with that name.
- Negative control: mutant bundle built with `VITE_UAR_TEST_OMIT_ELECTION_ID`; expected rejection = two distinct held election locks.
- Fixtures: scratch static directory holding bundle copies. Evidence required: lock query JSON, bundle hashes.
- Substituted boundaries: none. Limitations: relies on the server reading static files per request (`ServeDir`, `server.rs:2842-2845`).
- Invalidation keys: election id constant, worker entry path, test flag. Restoration: restore the original static directory. Owner tasks: 2.2, 2.9, 6.2.

### SCN-PGL-005 Leader closes during writes; survivor takes over

- Actor: operator closing the tab that opened first. Outcome: nothing the other tab did is lost or doubled.
- Source requirement: `#Leader change does not lose or duplicate client writes` and `#Database initialization runs in the worker at election time`. Reason now: handover is new with the worker. Likelihood: medium. Mandatory: true.
- Nondeterministic: true (kill timing relative to in-flight RPC). Stochastic: samples 10 (`handover.samples`); grader = after reload the conversation count equals start + `expectedCountDelta` and ids are unique; stoppingRule = fixed N=10, all must pass.
- Production entry: B3 steps. Real collaborators: as B2.
- Oracle: observation = conversation count after reload; truth = start count recorded before the run plus `expectedCountDelta`; comparator = exact equality.
- Negative control: mutant bundle with `VITE_UAR_TEST_DB_RETRY_ATTEMPTS=0`, 20 samples; expected rejection = at least one sample with a count below start + 20.
- Fixtures: fresh context per sample. Evidence required: counts before and after per sample, console log of retries.
- Substituted boundaries: none. Limitations: callers discard rejections; the "no leader within the bound" path has no UI route and is covered only by the mutant's observed loss.
- Invalidation keys: the retry wrapper module, `client.ts`, test flag. Restoration: none beyond context close. Owner tasks: 2.4, 2.9, 6.5.

### SCN-PGL-006 Run snapshot subscription across tabs through the worker

- Actor: operator watching a run trace. Outcome: live trace works with the worker.
- Source requirement: `#Local database runs in a leader-elected worker` (scenario "Run snapshot subscription works through the worker"). Reason now: `live` through `PGliteWorker` is UNVERIFIED here. Likelihood: high usage, UNKNOWN failure rate. Mandatory: true.
- Nondeterministic: true (cross-tab event delivery). Stochastic: samples 5; grader = trace rows in tab 1 satisfy `runTrace` within 5 s of the stub reply finishing; stoppingRule = fixed N=5, all must pass.
- Production entry: open a run trace in tab 1 while tab 2 sends `uar-pgl-first-session-001`.
- Oracle: observation = trace timeline rows in tab 1; truth = `runTrace` in `pgl-oracle.json` (first type, last type, required reply text); comparator = first row type equals `firstType`, last equals `lastType`, a row contains `mustContainText`, row order equals `seq` order.
- Negative control: trace panel closed before the run: no new rows appear in tab 1.
- Fixtures: fresh context, `stub-llm-pgl.json`. Evidence required: trace screenshots, event id lists.
- Substituted boundaries: stub-llm for the model. Limitations: none beyond Chromium.
- Invalidation keys: `run-event-repository.ts`, `pglite.worker.ts`, PGlite version. Restoration: none. Owner tasks: 2.3, 6.3.

### SCN-PGL-007 One snapshot owner; stale tab never overwrites

- Actor: operator with several tabs. Outcome: the stored graph never regresses.
- Source requirement: `#One tab owns the graph snapshot row and pending-action replay`. Reason now: last writer wins today. Likelihood: medium. Mandatory: true.
- Nondeterministic: true (ownership transfer timing). Stochastic: samples 5; grader = row digest unchanged after a non-owner edit, and equal to the survivor's graph within 5 s of the owner closing; stoppingRule = fixed N=5, all must pass.
- Production entry: B4 steps.
- Oracle: observation = `_graph_snapshot` row digest and `lastPersistedAt`; truth = the digest recorded just before the non-owner edit, then the survivor's graph digest; comparator = equality.
- Negative control: baseline build; expected rejection = the digest changes after the non-owner edit.
- Fixtures: fresh context. Evidence required: digests, status dumps. Substituted boundaries: none.
- Limitations: needs the row read path (B4 prerequisites). Invalidation keys: storage wrapper module, `bootstrap.ts`. Restoration: none. Owner tasks: 3.1-3.3, 4.5, 6.4.

### SCN-PGL-008 Queued action survives owner reload

- Actor: operator offline with two tabs. Outcome: no lost and no double-applied queued action.
- Source requirement: `#One tab owns the graph snapshot row and pending-action replay` (scenario "Owner reloads while another tab stays open") and `#PEM local-first graph lifecycle`. Reason now: hydration replaces state, so a new owner would drop queued actions. Likelihood: low. Mandatory: false.
- Nondeterministic: true. Stochastic: samples 5; grader = exactly one server-side effect and the stored pending list empty afterwards; stoppingRule = fixed N=5, all must pass.
- Production entry: enqueue an action offline in the owner tab, reload it, restore the network.
- Oracle: observation = server-side effect count and stored `pendingActions`; truth = one effect; comparator = equality.
- Negative control: the same sequence with the action settled before reload; expected = no replay.
- Fixtures: offline mode on the context. Evidence required: server record count, snapshot pending list.
- Substituted boundaries: none. Limitations: NOT_EXERCISED until OQ-5 names a real action path.
- Invalidation keys: as SCN-PGL-007. Restoration: restore network, remove the server record. Owner tasks: 3.3, 3.4.

### SCN-PGL-009 Long stream persists within the bound and the snapshot stays bounded

- Actor: operator reading a long reply. Outcome: input stays responsive and a crash loses at most the bound.
- Source requirement: `#Graph persistence has a maximum wait` and `#Retained AG-UI event entities are bounded`. Reason now: starvation and unbounded events confirmed in source, impact UNKNOWN (VS-10). Likelihood: UNKNOWN. Mandatory: true.
- Nondeterministic: true (timing, rendering, hardware). Stochastic: samples 10 (`longStream.samples`); grader per sample = precondition met (ingested events above retention) and `lastPersistedAt` advances at least once per `graphPersistMaxWaitMs` throughout the stream and entity count never exceeds `runtimeAgUiEventRetention` and, after the kill and reload, hydrated age is at most `graphPersistMaxWaitMs + persistDurationToleranceMs`; stoppingRule = fixed N=10, all must pass; a sample failing the precondition is BLOCKED and the scenario cannot PASS.
- Production entry: send `uar-pgl-long-stream-001`; sample `lastPersistedAt`; kill the tab mid-stream; reload.
- Real collaborators: UAR server, stub-llm extended with `stream_content`, Chromium, PGlite worker.
- Oracle: observation = `lastPersistedAt` samples, `RuntimeAgUiEvent` count, snapshot bytes, hydrated graph age; truth = `bounds` in `pgl-oracle.json`; comparator = thresholds as stated.
- Negative control: baseline build; expected rejection = `lastPersistedAt` does not advance during the stream and the count grows with events.
- Fixtures: fresh context, `stub-llm-pgl-long-stream.json`. Evidence required: sample series, per-type counts, bytes, machine description.
- Substituted boundaries: stub-llm for the model. Limitations: tolerances are hardware-dependent.
- Invalidation keys: `runtime-ingest.ts`, `bootstrap.ts`, scheduler module, the bounds constants, `src/bin/stub-llm.rs`. Restoration: none. Owner tasks: 4.1-4.6, 6.7.

### SCN-PGL-010 Existing profile upgrades without data loss

- Actor: existing operator. Outcome: all previous conversations and the graph snapshot remain.
- Source requirement: `#Existing local data survives the move to the worker`. Reason now: the worker takes over the existing database. Likelihood: certain for every existing user. Mandatory: true. Nondeterministic: false.
- Production entry: open a profile populated by the baseline build with the new build.
- Oracle: observation = thread and message counts and hydrated graph entity counts; truth = counts recorded from the baseline build before the upgrade; comparator = equality.
- Negative control: the same server opened at its other origin (`threadOriginControl`); expected rejection = empty database.
- Fixtures: persistent context directory copied per run. Evidence required: before and after counts.
- Substituted boundaries: none. Limitations: Chromium profile only. Invalidation keys: `client.ts`, `pglite.worker.ts`, IndexedDB name. Restoration: delete the copy. Owner tasks: 2.5, 6.6.

### SCN-PGL-011 Legacy localStorage data is imported once

- Actor: operator with pre-PGlite data. Outcome: legacy conversations appear once.
- Source requirement: `#Legacy localStorage migration runs once on the main thread`. Reason now: the migration moves out of `UarDb.open`. Likelihood: low. Mandatory: false.
- Nondeterministic: true (two tabs racing). Stochastic: samples 5; grader = each seeded record exactly once, flag set, keys removed; stoppingRule = fixed N=5, all must pass.
- Production entry: seed the two legacy keys, open two tabs concurrently.
- Oracle: observation = conversation list and messages; truth = the seeded records (hand-built from `client.ts:165-198` formats); comparator = each exactly once.
- Negative control: flag already set; expected = no import. Fixtures: scratch context. Evidence required: lists, `localStorage` dump.
- Substituted boundaries: none. Limitations: legacy format fixture hand-built.
- Invalidation keys: the migration function. Restoration: none. Owner tasks: 2.6, 6.6.

### SCN-PGL-012 Worker build runs in the desktop shell

- Actor: operator of the desktop app. Outcome: local database works in the webview.
- Source requirement: `#Local database runs in a leader-elected worker`. Reason now: the desktop shell is a supported environment. Likelihood: UNKNOWN. Mandatory: true when the phase gate builds The Boss. Nondeterministic: false.
- Production entry: the packaged app from `pnpm build:mac:arm64`, launched; create a conversation; quit; relaunch.
- Oracle: observation = conversation after relaunch; truth = the created title; comparator = present once.
- Negative control: emptied data directory; expected = absent, app starts.
- Fixtures: scratch app data directory. Evidence required: artifact path, screenshots.
- Substituted boundaries: none. Limitations: how the packaged app is driven is UNVERIFIED.
- Invalidation keys: worker entry, the desktop shell frontend bundle. Restoration: remove the scratch data directory. Owner tasks: 6.4.
