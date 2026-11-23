## 1. Service worker cache policy (build item 22)

- [ ] 1.1 In `frontend/public/sw.js`, replace the single `CACHE_NAME = 'uar-v1'` with the owned cache set from `design.md` D2 (`uar-assets-<buildId>`, `uar-shell-v2`, `uar-static-v2`, `uar-meta-v2`). Derive the build id from the first `<script type="module" src="/assets/...">` in each ok navigation response and keep the history `[current, previous]` in `uar-meta-v2`. Serialize rotations through one promise held in the worker global; delete `uar-assets-*` caches not in the history. Run the same code path from `install` for the `/index.html` precache.
- [ ] 1.2 Cache-first only for `/assets/` files whose name carries a content hash, written to `uar-assets-<current>`. If no build id is recorded, do not intercept or cache (design D3).
- [ ] 1.3 In `activate`, delete every cache not in the owned set, including `uar-v1`. Keep `skipWaiting()` and `clients.claim()` unchanged.
- [ ] 1.4 Serve unhashed `.svg`, `.png` and `.woff2` stale-while-revalidate from `uar-static-v2` (refresh with `event.waitUntil`). Remove the cache-first rule for `.js` and `.css` outside `/assets/`.
- [ ] 1.5 Store the navigation response only under the fixed key `/index.html` in `uar-shell-v2`; answer offline navigations with it. Keep the early return for non-GET and `/api/`, `/v1/`, `/metrics`, and keep the `http:`/`https:` check before every `cache.put` (existing `pwa-offline` requirement).

- [ ] 1.6 Wrap the bypass block (non-GET and `/api/`, `/v1/`, `/metrics` early return) in the marker comments `// uar-sw:bypass-start` and `// uar-sw:bypass-end` so the boundary runner can derive the mutant worker used as the negative control for SCN-WRK-007 by deleting the lines between them. The markers are comments only; no behavior changes.

## 2. Ownership and documentation

- [ ] 2.1 Do not change `frontend/src/main.tsx:45-49`. Request single-writer assignment from the lead for `frontend/public/sw.js` and a new `frontend/playwright.workers-pwa.config.ts` (both outside `frontend/src`).
- [ ] 2.2 Document the cache policy (names, two-build retention, shell entry, bypass paths, how the build id is derived and what happens when it cannot be) in the frontend documentation file chosen in Plan. The current service-worker documentation location is UNVERIFIED.

## 3. Catalog compute measurement and the optional worker (build item 21)

- [ ] 3.1 Add User Timing measures `uar:catalog-project` and `uar:catalog-filter` around the existing `projectModelRows` and `filterModelRows` calls (`frontend/src/features/models/ui/models-page.tsx:110,136`). Coordinate as one writer for `models-page.tsx` with the changes that redesign the Models page; this task adds two lines only.
- [ ] 3.2 Add the named constant `CATALOG_COMPUTE_BUDGET_MS` beside `model-catalog-view.ts`; Plan fixes its value.
- [ ] 3.3 [Plan-gated; recommended default: deferred, see design D6] Create `catalog.worker.ts` beside `features/models/model/model-catalog-view.ts`, importing only that pure module, answering `{ id, providerId, capabilities, query }` with `{ id, keys }` after one catalog message per change.
- [ ] 3.4 [Plan-gated] Add a feature-local hook that owns the worker, posts the catalog once per change, drops stale results by request id and catalog version, and returns rows in key order from the main-thread `Map`. The hook receives rows from the existing domain hook; it does not import the graph store, services or transports.
- [ ] 3.5 [Plan-gated] On construction failure or worker error, switch that page instance to the main-thread `filterModelRows` with identical output and no user-visible error.
- [ ] 3.6 [Plan-gated] Use the Model key `${providerId}/${modelId}` from `uiux-model-identity-graph-ownership`; sequence after that change.

## 4. Boundary acceptance (runs only at the phase boundary)

These tasks describe the final integration gate. They must not be executed early.

- [ ] 4.1 Build the SPA once, derive scratch trees A, B and C by the rule in `fixtures/wrk-oracle.json`, serve each with the real UAR process using `UAR_STATIC_DIR`, and run SCN-WRK-001, SCN-WRK-002 and SCN-WRK-003 in real Chromium with the baseline `sw.js` as the negative control.
- [ ] 4.2 Run SCN-WRK-004 (legacy `uar-v1` upgrade, baseline build from a worktree under `~/.claude/worktrees/`) and SCN-WRK-008 (no hashed entry script).
- [ ] 4.3 Run SCN-WRK-005 (changed favicon and probe script) and SCN-WRK-006 (shell entry count and offline deep link).
- [ ] 4.4 Run SCN-WRK-007, 10 samples, with the mutant `sw.js` (derived from the markers) as its negative control; BLOCKED if a probe URL does not return 200. Record the non-http(s) scheme scenario as NOT_EXERCISED.
- [ ] 4.5 Run SCN-WRK-009 on the real catalog, 10 samples, with the baseline build as its negative control, and record the measures with the machine description; run SCN-WRK-010 only if tasks 3.3-3.6 were kept, otherwise record NOT_EXERCISED. Record BLOCKED, not skipped, when the catalog or a prerequisite is missing.
- [ ] 4.6 Record for every scenario the exact command, exit code, source identity (HEAD, tree digests, `sw.js` digest) and receipt path. No credentials, tokens or JWTs are created or sent.
- [ ] 4.7 A mandatory scenario whose negative control did not run is BLOCKED, never PASS. Baseline worktrees are created with `scripts/worktree-new.sh wrk-baseline --base f2c9e36e` under `~/.claude/worktrees/`.
