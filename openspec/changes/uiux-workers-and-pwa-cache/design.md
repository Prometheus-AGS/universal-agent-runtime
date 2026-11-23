## Context

Phase `uar-uiux-architecture-2026-11`, build items 21 (catalog worker, "if Plan's budget
allows") and 22 (service-worker eviction). The assessment said stale assets are unlikely
and only the growth defect is real. Reading the worker file shows two more defects.
Nothing was built or run; runtime claims are marked UNVERIFIED.

### Verified in source

| Fact | Where |
|---|---|
| One fixed cache name `uar-v1`; precache `/`, `/index.html`, `/manifest.json`; `skipWaiting()` and `clients.claim()` | `frontend/public/sw.js:4-8,18,32` |
| `activate` deletes caches whose name is not `uar-v1`; entries inside `uar-v1` are never removed | `sw.js:22-31` |
| Non-GET, `/api/`, `/v1/`, `/metrics` requests are not intercepted | `sw.js:36-45` |
| Cache-first for any path starting `/assets/` or ending `.js`, `.css`, `.svg`, `.png`, `.woff2`; stores `response.ok` http(s) responses | `sw.js:48-68` |
| Navigations: network-first, every ok response stored under its own request key, fallback `caches.match(request)` then `/index.html` | `sw.js:70-88` |
| Registration is `navigator.serviceWorker.register('/sw.js')` on `load`, errors swallowed | `frontend/src/main.tsx:45-49` |
| `static/sw.js` is byte-identical to `frontend/public/sw.js` (`diff` clean), so a deploy does not change the worker script and does not re-run `install` or `activate` | shell check during this spec pass |
| Built HTML references the hashed entry as `<script type="module" crossorigin src="/assets/index-BAMiIrHQ.js">` plus `modulepreload` and stylesheet links | `static/index.html` |
| The server serves `static/` through `ServeDir` with `index.html` as the not-found fallback; files are read per request | `src/server.rs:2842-2845` |
| `projectModelRows` and `filterModelRows` are pure and already separate from the page; filtering is one linear pass lowercasing `key` and `name` | `frontend/src/features/models/model/model-catalog-view.ts:26-76` |
| The page memoizes projection on `rawItems` and filtering on query and filters, then renders every filtered row with no virtualizer | `features/models/ui/models-page.tsx:110,136,326` |
| The model-selector popover groups only the bounded projection of configured providers, not the full catalog | `features/models/ui/model-selector.tsx:62-79` |
| `@tanstack/react-virtual` is installed and used once | `features/chat/ui/run-trace-timeline.tsx:2,119` |
| Vite 8.3.5 `build.manifest` accepts `boolean | string`; worker `format` defaults to `iife` | `node_modules/vite/dist/node/index.d.ts:2962,3615-3619` |

### Not verified (UNVERIFIED)

- Whether Cache Storage `keys()` order is insertion order in every target (not relied on).
- Whether `ServeDir` serves `/.vite/manifest.json`; not needed by the chosen design.
- The runtime cost of filtering 8k rows. The statement in D6 is an estimate from the
  algorithm, not a measurement.
- Whether a stale tab on build B can still lazy-load a chunk the server deleted when build C
  replaced `static/` (`emptyOutDir: true`, `frontend/vite.config.ts`). The cache keeps B's
  already-cached chunks usable; chunks never cached before the deploy are unreachable with or
  without this change.

## Goals / Non-Goals

Goals: bounded service-worker storage across deploys; no frozen unhashed files; one HTML
entry; no impact on API or stream traffic; an evidence-gated decision on the catalog worker.

Non-goals: precaching the whole build for offline-first (Workbox-style); changing the
offline message queue behavior (existing `pwa-offline` scenario, untouched); caching API
data; any change to the PGlite worker (`uiux-local-first-persistence-worker`).

## Decisions

### D1 Build identity comes from the received HTML, not from the worker's bytes

Because the worker script does not change per deploy (table above), the eviction trigger
cannot be `activate`. It is the network response for a navigation: when the worker receives
an ok HTML response it reads the body, extracts the first `<script type="module" src="/assets/...">`
file name, and takes its hash segment as the build id. A different id than the one at the
head of the recorded history starts a rotation.

Rejected: stamping a build id into `sw.js` with a Vite plugin (the canonical Workbox-style
revision). It needs a build step and `frontend/vite.config.ts` changes, and would make the
worker bytes change per deploy, which is also fine, but the HTML-derived id needs no new
build tooling and works for the existing `static/` layout. Revisit if a precache manifest is
ever wanted.

Rejected: LRU or entry-count cap. It bounds storage but never says which entries are dead,
and a cap sized above one build's chunk count (hundreds of lazy chunks, per `static/assets`
after a build; count not measured here) retains several builds.

### D2 Cache names and retention

| Cache | Content | Lifetime |
|---|---|---|
| `uar-assets-<buildId>` | hashed `/assets/` files first fetched while `<buildId>` was current | kept while `<buildId>` is the current or previous build |
| `uar-shell-v2` | one entry, key `/index.html` | replaced on every ok navigation |
| `uar-static-v2` | unhashed `.svg`, `.png`, `.woff2` | refreshed by revalidation |
| `uar-meta-v2` | one JSON entry, the build history `[current, previous]` | rewritten on rotation |

The history lives in Cache Storage because the worker can be killed at any time and globals
do not survive. Rotation is serialized through a promise held in the worker global; two
tabs navigating together cannot interleave the history write. `activate` deletes every
cache not in this table (this removes `uar-v1`).

Keeping the previous build matters: a tab still running build B can request a lazy chunk
after C is observed. A chunk of B already cached is served; one never cached is gone from the
server anyway (UNVERIFIED note above). An asset fetched by a stale B tab after C is observed is
stored with C and is dropped when C rotates out; that is a known, bounded imprecision.

`skipWaiting()` and `clients.claim()` stay as they are, since changing activation semantics is
not an observed problem.

### D3 Asset policy

Cache-first applies only to `/assets/` paths whose file name carries a content hash
(pattern `-[A-Za-z0-9_-]{8,}\.[a-z0-9]+$`; Vite and Rolldown name outputs `[name]-[hash]`, as
`static/index.html` shows). When no build id is recorded the worker does not cache and does
not respond; the request goes to the network. This also removes the cache-first rule for
every `.js` and `.css` file outside `/assets/`, which in a development server would freeze
modules.

### D4 Unhashed static files

Same-origin `.svg`, `.png` and `.woff2` outside `/assets/` are served stale-while-revalidate:
return the cached copy at once and refresh it in the background with `event.waitUntil`. A
changed file is therefore served by the second load after the change. The set of such files is
the finite contents of `public/`; no entry cap is added. `/manifest.json` and any other
non-hashed file stay on the network-first path.

### D5 One shell entry

On an ok navigation response the worker stores the document under the fixed key `/index.html`
in `uar-shell-v2` and never under the request URL. Offline navigation answers with that entry.
This replaces `sw.js:73-80` and the `caches.match(event.request)` fallback; both only existed
to serve per-URL copies of what is the same shell.

### D6 Catalog worker: recommended default is to defer it

The Models page cost splits into projection (`useMemo` over `rawItems`), filtering (`useMemo`
over query and filters) and rendering every filtered row (`models-page.tsx:326`). Rough order of
magnitude, UNVERIFIED: one pass over 8k small rows with two `toLowerCase` calls is on the order of
milliseconds per keystroke; structured-cloning 8k row objects into a worker is the same order or
worse, and has to be paid again on every catalog change. Rendering up to 8k rows is the part
that costs tens to hundreds of milliseconds and a worker cannot touch it. A worker therefore
risks adding latency and code while leaving the dominant cost in place.

Decision for this change:

1. Ship the measurement (task 3.1): two User Timing measures, `uar:catalog-project` and
   `uar:catalog-filter`, around the existing pure calls, and the compute-budget requirement.
2. Keep the worker requirements in the spec as `Where the catalog worker is enabled`, so
   they bind only if Plan keeps item 21.
3. Recommended Plan disposition: defer the worker. Revisit only if SCN-WRK-009 shows the
   measures above budget on the real catalog. Row rendering is a separate defect: the page
   needs a virtualizer (`@tanstack/react-virtual`, cand-003 in the phase library candidates).
   That work is not in this change; the lead assigned it to
   `uiux-model-identity-graph-ownership` (revision round 1).

If Plan keeps item 21 anyway, the design is: a module worker `catalog.worker.ts` that imports
only `model-catalog-view.ts`; the main thread keeps `allModels = projectModelRows(rawItems)`
for rendering and posts a minimal projection (`key`, `provider_id`, `name`, `tool_call`,
`reasoning`, `vision`) once per catalog change; each keystroke posts `{ id, providerId,
capabilities, query }` and receives `{ id, keys }`; the page keeps rows in a `Map` keyed by `key`
and renders in the returned key order; results with a stale `id` or catalog version are dropped;
a failed construction or a worker error switches that page instance to the main-thread
function. The worker must use the Model key `${providerId}/${modelId}` from
`uiux-model-identity-graph-ownership`, so it follows that change.

## Risks / Trade-offs

- **HTML parsing is a coupling.** The worker depends on the entry script appearing as a
  `type="module"` script with an `/assets/` source. A future build that inlines the entry would
  leave the build id unknown; D3 then disables asset caching rather than failing. Offline
  support degrades silently in that case; the boundary scenario SCN-WRK-008 records the
  behavior.
- **A stale tab may lose lazy chunks** that were never cached before a deploy. Unchanged by this
  change (server-side), only no longer masked beyond two builds.
- **Offline shell is one HTML copy.** Deep links offline now rely on client routing; the
  existing spec already expects previously visited pages to load.
- **No build-time precache.** First offline load still needs one online visit, as today.
- **Static cache has no cap.** Bounded by the contents of `public/`; a deploy that renames
  many unhashed files would leave old names until the browser evicts them. Not observed.

## Open questions

- **OQ-1 (Plan):** keep or defer item 21 (recommended: defer; see D6).
- **OQ-2 (Plan):** value of `CATALOG_COMPUTE_BUDGET_MS`.
- **OQ-3 (lead):** single-writer assignment for `frontend/public/sw.js` (outside `frontend/src`).
- **OQ-4 (resolved by the lead, revision round 1):** virtualizing the Models page list is owned by `uiux-model-identity-graph-ownership`.

## Acceptance

Common to every table: the console is a built SPA tree served by the real UAR process from a
scratch static directory (`UAR_STATIC_DIR`, honored first by `resolve_static_dir`,
`server.rs:4415-4423`; the directory is fixed at startup but files are read per request) at
`UAR_BASE_URL`; the browser is real Chromium under Playwright with service workers allowed;
Cache Storage is read with `caches.keys()` and `cache.keys()` from the page; offline is
`context.setOffline(true)`. No scenario calls a model, so no stub or paid route is needed. No
credentials or JWTs are created, sent or required (operator rule 2026-10-26). Build trees A,
B and C and the other inputs are generated by the rules in `fixtures/wrk-oracle.json`; the
renamed entry and chunk synthesize build identity and are recorded as the substituted
boundary. Baseline: a git worktree created with `scripts/worktree-new.sh wrk-baseline --base
f2c9e36e` under `~/.claude/worktrees/` and built with the frontend build script; the lead adds
the build commands and receipts to the boundary packet. The gate runs only at the phase
boundary; the Playwright config (`frontend/playwright.workers-pwa.config.ts`) and specs are
authored then and reviewed independently against these rows.

## Frozen oracle fixtures

Written now, read by the boundary runner, never produced by the implementation (rule R1).
Values with status `proposed` change only by editing the fixture and this document together
before the contract is adopted.

| File | Content |
|---|---|
| `fixtures/wrk-oracle.json` | owned cache names and prefixes and the legacy name that must vanish; build-tree derivation rule and the sets retained and absent after C; unhashed probe paths with before and after bytes and the maximum loads until the new bytes are served; navigation URL pattern and expected HTML entry count; bypass probe URLs, mutant rule, latency tolerance and sample count; catalog minimum rows, query length, compute budget, long-task threshold and baseline-must-lack measures |

The mutant rule requires markers `// uar-sw:bypass-start` and `// uar-sw:bypass-end` around the
bypass block in `sw.js` (task 1.6), so the mutant is derived mechanically from the shipped file.

## Behavior B1: old builds' assets are evicted and the current build works offline

| Field | Declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-workers-and-pwa-cache` / tasks 1.1-1.5 |
| Source identity | HEAD at gate time; hashes of trees A, B, C and of `sw.js` recorded |
| Production entry point | Operator opens the console in Chromium three times as builds A, B, C are deployed to the same origin (static files replaced between loads), then reloads offline |
| Real collaborators | UAR server (`ServeDir`), Chromium, the registered service worker, Cache Storage |
| Boundary exercised | `/sw.js` registration; `fetch` handler; Cache Storage names `uar-assets-*`, `uar-meta-v2` |
| Observable result | After C: `uar-assets-` caches exist for C and B only (`trees.expectedRetainedAfterC`); offline reload renders; offline fetch of a B chunk succeeds, of an A-only file fails |
| Negative control | Baseline `sw.js` (worktree `wrk-baseline`, cache `uar-v1`) through the same three loads: A-only entries remain, so the oracle rejects. For the offline fetch: the baseline serves the A-only file from its cache, so the oracle rejects |
| Isolation | New browser context per run; scratch static directories under the session scratchpad; the repository `static/` is not modified |
| Prerequisites | One built tree to derive A, B, C from; baseline worktree built |
| Final local gate | Playwright run of SCN-WRK-001, SCN-WRK-002, SCN-WRK-003 with `playwright.workers-pwa.config.ts` |
| Evidence | pending: receipt path assigned by the lead's boundary packet |
| Limitations | Synthesized build identity; does not prove behavior across a real multi-asset deploy; Chromium only |

## Behavior B2: upgrade from the legacy cache

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-workers-and-pwa-cache` / task 1.3 |
| Production entry point | A profile that loaded the baseline build (legacy `uar-v1` cache) then loads the new build |
| Real collaborators | As B1 |
| Boundary exercised | Service worker `install` and `activate`; Cache Storage |
| Observable result | After activation `caches.keys()` has no name in `ownedCaches.legacyMustBeAbsent` and no name outside `ownedCaches`; online load works |
| Negative control | The baseline build alone: `uar-v1` remains, so the oracle rejects |
| Isolation | New context; baseline tree from the worktree |
| Prerequisites | Baseline build artifact |
| Final local gate | Playwright run of SCN-WRK-004 |
| Evidence | pending |
| Limitations | One browser engine |

## Behavior B3: unhashed files and navigations stay fresh and bounded

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-workers-and-pwa-cache` / tasks 1.4, 1.5 |
| Production entry point | Operator loads the console, a deploy changes the favicon and a probe script (`unhashedProbes`), operator reloads; operator visits the 20 URLs of `navigation`; then deep-links offline |
| Real collaborators | As B1 |
| Boundary exercised | `fetch` handler branches for static files and navigations |
| Observable result | The image bytes seen by the page equal `newBytes` within `maxLoadsUntilServed` loads; `/probe.js` shows `bytesAfter` on the next load; HTML entries across caches equal `expectedHtmlEntriesInCaches`; the offline deep link renders |
| Negative control | Baseline `sw.js`: old image and `/probe.js` bytes are still served after the change and the cache holds one HTML entry per visited URL, so the oracle rejects |
| Isolation | New context; scratch static directory |
| Prerequisites | As B1 |
| Final local gate | Playwright run of SCN-WRK-005 and SCN-WRK-006 |
| Evidence | pending |
| Limitations | Compares bytes through `fetch` from the page, not pixels. The background revalidation race makes SCN-WRK-005 sampled |

## Behavior B4: API and stream traffic is untouched

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-workers-and-pwa-cache` / tasks 1.5, 1.6 |
| Production entry point | Console open with its live event stream while the server emits a change; the probe URLs in `bypass.probeUrls` are fetched from the page |
| Real collaborators | UAR server, real change feed, Chromium |
| Boundary exercised | Event stream and `/api/` GETs through the service worker scope |
| Observable result | No cached URL under a bypass prefix (`expectedCachedUnderBypassPrefixes`); stream latency within `eventLatencyToleranceMs` of the same page with the worker unregistered |
| Negative control | Mutant `sw.js` (`bypass.mutantRule`) registered in the same scratch tree: the probe URLs appear in Cache Storage, so the oracle rejects. If a probe does not return 200 the control cannot reject and the scenario is BLOCKED |
| Isolation | New context; scratch configuration for the published change |
| Prerequisites | A change the server publishes on its live feed (a harmless provider display-name edit on a scratch configuration; identity comes from the proxy); markers in `sw.js` (task 1.6) |
| Final local gate | Playwright run of SCN-WRK-007, 10 samples |
| Evidence | pending |
| Limitations | The existing "non-http(s) requests are never cached" scenario needs a browser extension content script and is NOT_EXERCISED; the code path is retained |

## Behavior B5: catalog compute stays in budget, and the optional worker agrees with the main thread

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-workers-and-pwa-cache` / tasks 3.1-3.2 (worker tasks 3.3-3.6 only if kept) |
| Production entry point | Operator opens Models and types the frozen query (`catalog.queryLength` characters) over the full catalog |
| Real collaborators | UAR server catalog endpoint, Chromium, `PerformanceObserver` for `longtask`, User Timing |
| Boundary exercised | Models page projection and filter; (if enabled) module worker |
| Observable result | Every `uar:catalog-filter` and `uar:catalog-project` measure is at most `catalog.computeBudgetMs`; no long task of `longTaskThresholdMs` attributed to the steps; with the worker enabled, the displayed key list equals the main-thread list |
| Negative control | Baseline build (no measures exist): the oracle requires the measures named in `baselineMustLackMeasures`, finds none, and rejects |
| Isolation | New context |
| Prerequisites | The real catalog with at least `catalog.minimumRows` rows, else BLOCKED; baseline worktree |
| Final local gate | Playwright run of SCN-WRK-009 and, if the worker is kept, SCN-WRK-010 |
| Evidence | pending |
| Limitations | Rendering cost of the unvirtualized list is not part of this oracle (see D6, OQ-4; virtualization is owned by `uiux-model-identity-graph-ownership`). SCN-WRK-010 is NOT_EXERCISED if the worker is deferred |

## Scenario matrix rows

Scenario IDs are fixed. "Source requirement" values are `openspec:<capability>#<Requirement
name>`. Oracle is observation / expected truth source / comparator; every expected-value
source is a file in `## Frozen oracle fixtures` or a recorded pre-action observation, never
implementation output. Owner tasks refer to `tasks.md`. Capability is `pwa-offline` unless
stated.

### SCN-WRK-001 Old build assets are evicted after successive deploys

- Actor: operator of a deployed console. Useful outcome: browser storage does not grow with each deploy.
- Source requirement: `openspec:pwa-offline#Hashed build assets are retained for the current and previous build only`.
- Reason now: confirmed growth defect (`sw.js:22-31`); impact UNKNOWN. Likelihood: certain per deploy. Mandatory: true. Nondeterministic: false.
- Production entry: load A, B, C in turn, then list Cache Storage.
- Real collaborators: UAR `ServeDir`, Chromium, service worker, Cache Storage.
- Oracle: observation = URLs in `uar-assets-*` caches; truth = the file lists of the generated trees per `trees` in `wrk-oracle.json`; comparator = set equality with (C ∪ B) and empty intersection with A-only files.
- Negative control: baseline `sw.js`; expected rejection = A-only URLs still cached.
- Fixtures: scratch static trees A, B, C; new context. Evidence required: cache listing JSON per step, tree file digests.
- Substituted boundaries: build identity synthesized by renaming the entry and one chunk. Limitations: Chromium only; real deploy sizes not reproduced.
- Prerequisites: baseline worktree `wrk-baseline` built.
- Invalidation keys: `frontend/public/sw.js`, `static/index.html` entry naming, `vite.config.ts` output naming, `fixtures/wrk-oracle.json`.
- Restoration: delete scratch trees, unregister the worker, close contexts, `scripts/worktree-rm.sh wrk-baseline`. Owner tasks: 1.1, 1.2, 4.1.

### SCN-WRK-002 Current build loads offline after eviction

- Actor: operator on a flaky network. Outcome: eviction never removes what the current build needs.
- Source requirement: `openspec:pwa-offline#Hashed build assets are retained for the current and previous build only` (scenario "The current build loads offline after eviction").
- Reason now: over-eviction control for SCN-WRK-001. Likelihood: medium. Mandatory: true. Nondeterministic: false.
- Production entry: after C, `context.setOffline(true)`, reload.
- Oracle: observation = rendered root and failed requests; truth = tree C's `index.html` references; comparator = the app root has children and no request for a C asset reached the network.
- Negative control: offline reload in a fresh context that never visited online; expected rejection = browser offline error page and an empty root.
- Fixtures: as SCN-WRK-001. Evidence required: screenshot, request log. Substituted boundaries: as SCN-WRK-001.
- Limitations: only the shell render is asserted. Invalidation keys: as SCN-WRK-001. Restoration: restore network. Owner tasks: 1.2, 4.1.

### SCN-WRK-003 Previous build's chunk survives; A-only file does not

- Actor: operator with a tab left open across a deploy. Outcome: the older tab keeps working with cached chunks.
- Source requirement: `openspec:pwa-offline#Hashed build assets are retained for the current and previous build only` (scenario "The previous build's chunk is still served").
- Reason now: motivates keeping two builds. Likelihood: medium. Mandatory: true. Nondeterministic: false.
- Production entry: after C and offline, `fetch` a cached B chunk and an A-only file from the page.
- Oracle: observation = fetch status; truth = the generated trees (B chunk exists in B; A-only file exists only in A); comparator = B chunk 200, A-only network error.
- Negative control: baseline `sw.js` through the same loads; expected rejection = the A-only fetch also succeeds from the legacy cache.
- Fixtures and evidence as SCN-WRK-001. Substituted boundaries: as SCN-WRK-001.
- Limitations: the B chunk must have been cached during B's session. Invalidation keys: as SCN-WRK-001. Restoration: as SCN-WRK-001. Owner tasks: 1.2, 4.1.

### SCN-WRK-004 Legacy `uar-v1` cache is removed on upgrade

- Actor: returning operator. Outcome: the old cache is reclaimed.
- Source requirement: `openspec:pwa-offline#Superseded service worker caches are deleted on upgrade`.
- Reason now: every existing user holds `uar-v1`. Likelihood: certain. Mandatory: true. Nondeterministic: false.
- Production entry: load the baseline tree, then the new tree; read `caches.keys()` after activation.
- Oracle: observation = cache names; truth = `ownedCaches` in `wrk-oracle.json`; comparator = every name is owned and none is in `legacyMustBeAbsent`.
- Negative control: the baseline build alone; expected rejection = `uar-v1` present.
- Fixtures: baseline worktree build; new context. Evidence required: cache name lists. Substituted boundaries: none.
- Limitations: one engine. Invalidation keys: `frontend/public/sw.js`, `wrk-oracle.json`. Restoration: remove the worktree. Owner tasks: 1.3, 4.2.

### SCN-WRK-005 Unhashed static files and scripts are not frozen

- Actor: operator after a deploy that changes a logo. Outcome: the new logo appears without clearing site data.
- Source requirement: `openspec:pwa-offline#Unhashed static files are revalidated`.
- Reason now: `sw.js:50-56` serves any `.svg`, `.png`, `.js`, `.css` from cache forever. Likelihood: certain once a public file changes. Mandatory: true.
- Nondeterministic: true (background revalidation races the next load). Stochastic: samples 5; grader = SHA-256 of the fetched image equals the digest of `newBytes` within `maxLoadsUntilServed` loads and `/probe.js` equals `bytesAfter` within `maxLoadsUntilServed`; stoppingRule = fixed N=5, all must pass.
- Production entry: load; change the image and `/probe.js` in the scratch tree; reload.
- Oracle: observation = bytes returned by `fetch` of each path from the page; truth = `unhashedProbes` in `wrk-oracle.json`; comparator = digest equality.
- Negative control: baseline `sw.js`; expected rejection = old bytes still served after three reloads.
- Fixtures: scratch tree with a probe script. Evidence required: digests per load. Substituted boundaries: none.
- Limitations: Chromium only. Invalidation keys: `sw.js`, `wrk-oracle.json`. Restoration: delete scratch tree. Owner tasks: 1.4, 4.3.

### SCN-WRK-006 Navigations share one shell entry

- Actor: operator using many deep links. Outcome: the cache does not grow per URL and deep links work offline.
- Source requirement: `openspec:pwa-offline#Navigations share one cached shell`.
- Reason now: `sw.js:73-80` stores one document per request key. Likelihood: certain. Mandatory: true. Nondeterministic: false.
- Production entry: visit the `navigation` URLs, count HTML entries; go offline; reload the Models page deep link (the `/admin/models` form is assumed; the route path is UNVERIFIED and the runner reads it from the console's own navigation link).
- Oracle: observation = HTML entries across caches and the rendered route; truth = `expectedHtmlEntriesInCaches`; comparator = count equality and the Models route heading is visible.
- Negative control: baseline `sw.js`; expected rejection = about 20 HTML entries.
- Fixtures: new context. Evidence required: entry list, screenshot. Substituted boundaries: none.
- Limitations: Models data is not loaded offline. Invalidation keys: `sw.js`, router paths. Restoration: restore network. Owner tasks: 1.5, 4.3.

### SCN-WRK-007 API and stream traffic bypasses the worker

- Actor: operator watching live updates. Outcome: realtime push is not delayed or cached by the worker.
- Source requirement: `openspec:pwa-offline#API and stream requests bypass the service worker`.
- Reason now: the phase depends on live model changes (decision 3). Likelihood: high exposure. Mandatory: true.
- Nondeterministic: true (network timing). Stochastic: samples 10 (`bypass.samples`); grader per sample = no URL under a bypass prefix in any cache and median event latency within `eventLatencyToleranceMs` of the same sample with the worker unregistered; stoppingRule = fixed N=10, all must pass.
- Production entry: open the console, fetch each `bypass.probeUrls` entry, publish a harmless change on the live feed, observe delivery and caches.
- Oracle: observation = cache contents and per-event receipt time; truth = `bypass` in `wrk-oracle.json` and the server's emission timestamp; comparator = zero cached bypass URLs and latency difference at most the tolerance.
- Negative control: the mutant `sw.js` built by `bypass.mutantRule`; expected rejection = the probe URLs appear in Cache Storage. If a probe does not return 200 the scenario is BLOCKED.
- Fixtures: scratch configuration; scratch tree with the mutant. Evidence required: cache listing, latency series.
- Substituted boundaries: none. Limitations: the non-http(s) scheme scenario is NOT_EXERCISED.
- Invalidation keys: `sw.js` and its bypass markers, `frontend/src/entities/sync.ts`. Restoration: revert the change, unregister workers. Owner tasks: 1.5, 1.6, 4.4.

### SCN-WRK-008 Unidentifiable build disables asset caching

- Actor: developer or a build that inlines its entry. Outcome: the worker never freezes unknown files.
- Source requirement: `openspec:pwa-offline#Hashed build assets are retained for the current and previous build only` (scenario "The entry script cannot be identified").
- Reason now: D3 fallback. Likelihood: low. Mandatory: false. Nondeterministic: false.
- Production entry: serve an `index.html` with no hashed entry script from the scratch directory.
- Oracle: observation = `uar-assets-*` caches; truth = none expected; comparator = none exist.
- Negative control: the real tree; expected rejection = a `uar-assets-*` cache exists.
- Fixtures: scratch tree. Evidence required: cache names.
- Substituted boundaries: none. Limitations: does not reproduce a Vite dev server. Invalidation keys: `sw.js`. Restoration: delete tree. Owner tasks: 1.2, 4.2.

### SCN-WRK-009 Catalog projection and filtering stay within budget

- Actor: operator filtering about 8k models. Outcome: typing stays responsive.
- Source requirement: `openspec:frontend-web-workers#Catalog projection and filtering stay within a compute budget`.
- Reason now: decides whether item 21 is worth building (D6). Likelihood: UNKNOWN. Mandatory: true.
- Nondeterministic: true (timing, hardware). Stochastic: samples 10 (`catalog.samples`); grader per sample = every `uar:catalog-project` and `uar:catalog-filter` measure at most `computeBudgetMs`, no long task of `longTaskThresholdMs` or more attributed to the steps, and the catalog has at least `minimumRows` rows (otherwise the sample is BLOCKED); stoppingRule = fixed N=10, all must pass; the runner records the machine.
- Production entry: Models page, type the frozen query.
- Oracle: observation = User Timing measures and `longtask` entries; truth = `catalog` in `wrk-oracle.json`; comparator = thresholds as stated.
- Negative control: the baseline build, which has no such measures: the oracle requires them (`baselineMustLackMeasures` names what the baseline lacks) and rejects.
- Fixtures: new context; real catalog. Evidence required: measure list, long-task list, row count, machine description.
- Substituted boundaries: none. Limitations: excludes DOM rendering.
- Invalidation keys: `model-catalog-view.ts`, `models-page.tsx`, catalog size, `wrk-oracle.json`. Restoration: none. Owner tasks: 3.1, 3.2, 4.5.

### SCN-WRK-010 Catalog worker agrees with the main thread, is side-effect free and falls back (conditional)

- Actor: operator filtering the catalog. Outcome: the worker never changes what is shown or touches anything else.
- Source requirement: `openspec:frontend-web-workers#Catalog filtering runs in a worker when the catalog worker is enabled`, `#Catalog worker results match the main-thread functions and fall back safely` and `#Computation workers are side-effect free`.
- Reason now: binds only if Plan keeps item 21. Likelihood: UNKNOWN. Mandatory: false.
- Nondeterministic: true (typing races the worker). Stochastic: samples 10; grader = the displayed key list after fast typing equals the main-thread list for the last query, no network request originates from the worker, and no storage key changes; stoppingRule = fixed N=10, all must pass.
- Production entry: type fast; repeat with `Worker` deleted before page load.
- Oracle: observation = ordered key lists shown, request log, storage diff; truth = the main-thread function output for the same input recorded in the same page; comparator = equality and zero worker requests.
- Negative control: none while non-mandatory; when exercised, the `Worker`-deleted run must show an identical list and no error. NOT_EXERCISED when item 21 is deferred.
- Fixtures: new context; real catalog. Evidence required: key lists, request log, console log.
- Substituted boundaries: none. Limitations: conditional on OQ-1.
- Invalidation keys: the worker module, `model-catalog-view.ts`. Restoration: none. Owner tasks: 3.3-3.6, 4.5.
