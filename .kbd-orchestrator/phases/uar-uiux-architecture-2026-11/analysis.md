# Analysis — uar-uiux-architecture-2026-11

Generated 2026-11-19T19:06:35Z; revised 2026-11-19 (round 2, after adversarial review round 1: 5 critical, 9 warning, 3 suggestion; round-1 critical findings verified at source; round 2 found C1 and C3 only partly fixed, see Unresolved review findings). Stage: Analyze (after Assess, before Spec).
Mode: **stack specified** — the stack is fixed by `versions.toml`, `frontend/package.json`
and the operator decisions of 2026-11-19. Candidates were ranked inside that stack; no
stack discovery was run.

Method: source inspection, local registry reads and documentation lookups only. Nothing
was built, tested or run (full-phase verification boundary; VLBV). Two read-only
subagents inspected source and I re-checked their load-bearing claims at the cited
lines (listed under "Verified by the lead").

## Delta from the assessment (read first)

Analyze changed seven assessment conclusions. Spec and Plan must use these, not the
assessment text.

1. **Realtime push already works with remote Surreal and Postgres for most topics.**
   The assessment said push is absent for remote databases because
   `/api/uar/sync/stream` returns 404 there (`server.rs:4823-4825`). That route is only
   the embedded path. For Postgres, or Surreal in remote mode, the frontend uses a
   different adapter set: one SSE connection to `/api/live` plus
   `/api/live/{topic}/snapshot` (`frontend/src/entities/sync.ts:342-344`,
   `registerUarSseAdapters`). The backend serves `/api/live` on all three backends
   (`server.rs:2437-2438`), sourced from Surreal LIVE queries
   (`realtime/surreal_bus.rs:169-172`, started for embedded and remote alike,
   `server.rs:887-892`) or Postgres `LISTEN uar_entity_change` fed by `pg_notify`
   triggers (`realtime/postgres_bus.rs:110-117`,
   `migrations/20260220000002_notify_entity_change.sql:95-136`).
   `/api/live` sends **invalidations only**: every frame is a `reset` event with
   `requires_snapshot: true`, and the client refetches `/api/live/{topic}/snapshot`
   (`api/live.rs:98-110`). So a topic fix is a backend change: the right topic must be
   invalidated and its snapshot route must return the changed rows.
   **The real gaps are per topic**, not per backend:
   - **Conversation policy publishes nothing on any backend.** The
     `conversation_policies` / `principal_conversation_policies` tables have no topic,
     the handlers make no bus call (`api/discovery.rs:871-958, 1107-1160`), and there is
     no Postgres trigger.
   - **Providers arrive on the wrong topic on Surreal.** Providers are stored as
     `provider.{id}` rows in the `settings` table (`settings/manager.rs:975,1050`), so
     Surreal emits them on `settings`. Postgres routes `provider.%` keys to `providers`
     (`notify_entity_change.sql:66-72`). The frontend subscribes to `providers` and
     expects `Provider` entities (`frontend/src/lib/realtime/topics.ts:15`). So on Surreal a
     provider change invalidates `settings`, not `providers`, and the providers snapshot
     is not refetched. On Postgres it invalidates `providers`. Runtime behavior is
     unverified.
   - **No writer to the `models` topic was found** on either backend; the frontend
     subscribes to it (`topics.ts:16`). Treat as "never fires" until shown otherwise.
     Its snapshot is nonetheless a fourth Model writer: on a reset it replaces the graph's
     Model set and removes every Model the snapshot lacks
     (`server/lifecycle_snapshot.rs:165-193`, `frontend/src/lib/realtime/uar-sse-adapter.ts:258-283`;
     critic-sourced, not re-read by the lead).
   - **The default-provider change has no dedicated topic.** It is a `llm.default_provider`
     settings row, so it invalidates `settings` on both backends.
   - **Team and binding topics are internal** (`is_public() == false`,
     `realtime/mod.rs:193-198`), consumed only by host workers (`server.rs:2286-2291`).
   - **Postgres is an optional feature**, `postgres-backend`, not in default features
     (`Cargo.toml:318-321, 399`). Supporting it in this phase means building and
     verifying with that feature.
   - **There is no local-write event path.** Apart from `governance.enabled`
     (`settings/manager.rs:683-700`), every change event comes back through the DB
     change feed, including the process's own writes.
2. **None of the 15 "no reader" namespaces is read from the settings DB, ever.** They
   split into two classes (table below). Eleven are honored from YAML at boot; four are
   not read at all, even from YAML. The operator decision (remove from UI, keep YAML)
   fits all fifteen: a DB edit to any of them changes nothing, before or after restart.
   - **Additional defect:** for keys outside `server`/`security`/`persistence`,
     `set_value` writes the in-memory "effective" map (`settings/manager.rs:666-671`) and
     the API reports the edit as `"effective"` (`api/settings.rs:575-582`) although no
     code reads it. The status the console shows is false for these keys.
3. **Model identity divergence is confirmed** (reviewer-sourced claim, now verified).
   `models-graph.ts:22,45` and `models-store.ts:30,49` key `Model` as
   `${providerId}/${modelId}`; `providers-graph.ts:78,86` uses the bare catalog id
   (`models-api.ts:30-31`). `providers-graph.ts:84` also hard-codes `vision: false`.
4. **Team route blocking is narrower and more varied than stated.** Only the routes the
   team resolves now are compared with the captured attempt (`team_execution/resolution.rs:71-101`).
   Profile, reasoning and endpoint edits bump `settings_revision`
   (`llm/registry.rs:165-192`) and surface as `TEAM_REVISION_CONFLICT`
   (`team_profile.rs:73`). Removal, disable, profile mismatch, missing pricing identity or
   a changed rebuilt receipt surface as `TEAM_ROUTE_PROFILE_MISMATCH`. A catalog price
   change needs a new binary (`catalog.rs:32` static singleton); in one process it means
   reassigning the pricing identity. `TEAM_REVISION_CONFLICT` is also used by unrelated
   team-state concurrency checks (`dispatch.rs:31,43`, `waits.rs:139`,
   `peer_commands.rs:47`), so the next-task-boundary change must not suppress those.

5. **`llm.default_provider` only takes effect at boot through the settings API.** At
   startup `provider_sync.rs:21-23` reads it and calls `registry.set_default`. After that,
   editing the key through the settings API changes nothing until restart, while the API
   reports the edit as effective (the effective-map write below). Only
   `POST /api/providers/{id}/default` swaps the live default: it persists through
   `set_default_provider_id` and calls `state.registry.set_default` (`api/providers.rs:367-399`),
   and it publishes nothing beyond the `settings` row change. Decision 2 made this the
   system level, so the system-default picker must use the live path and must not be a
   plain settings field.
6. **A running team is also blocked by any binding revision change.** Before route
   checks, `team_execution/resolution.rs:39-48` rejects the task with "Team deployment
   binding changed" unless `binding.revision == attempt.binding_revision`. A member model
   change is a binding revision. So decision 1 of the team model (adopt at the next task
   boundary) needs this check changed too, not only route re-resolution.
7. **Hot reload already validates; the brick is at boot.** `ConfigManager::reload`
   (`config_manager.rs:103-130`) loads, validates and rejects a bad file, keeping the
   current config. The delayed brick is the next start: `main.rs:94-99` fails startup
   when the YAML does not load, and there is no last-known-good on disk. Two further boot
   paths come from bad DB values: the persisted embedding settings read at
   `server.rs:1251-1263` and a malformed stored provider row (assessment G3). Each needs
   a fallback with a visible warning.

### Settings namespace classification (G3, operator decision 6)

| Namespace | Class | Where the value is honored |
|---|---|---|
| `server.*` | YAML at boot | listener bind `server.rs:505-510` |
| `security.*` | YAML at boot | `server.rs:491,507,2237,2602,3733`; `security/middleware.rs:108` |
| `persistence.*` | YAML at boot | `server.rs:765,798-806` |
| `llm.model` | YAML at boot | `server.rs:493` (`config.llm.clone()`) |
| `llm_failover` | YAML at boot | `server.rs:1948` |
| `file_processing` | YAML at boot | `rag/ingestion_worker.rs:203`, `api/upload.rs:88` |
| `sycophancy` | YAML at boot | `server.rs:8717` |
| `acp` | YAML at boot | `server.rs:2878-2886` |
| `kreuzberg` | YAML at boot | `ingestion_worker.rs:224-225,239` |
| `unstructured` | YAML at boot | `ingestion_worker.rs:238` |
| `mistral_ocr` | YAML at boot | `ingestion_worker.rs:239` |
| `skill_evolution` | Not read at all | run manager uses `SkillEvolutionConfig::default()` (`runtime/manager.rs:1436`); `with_skill_evolution_config` has no caller (`:2243`) |
| `context_management` | Not read at all | no `AppConfig` field; schema seeds literals (`settings/manager.rs:2296+`) |
| `sandbox` | Not read at all | runner chosen by env `UAR_SANDBOX_RUNNER` / `UAR_SANDBOX_REMOTE_URL` (`sandbox/platform.rs:21,26`) |
| `vision` | Not read at all | `VisionConfig` (`config.rs:277`) has no reader |

The four "not read at all" keys are also dead in YAML. Removing their UI controls is
safe. Whether to delete their YAML fields is outside decision 6 and stays out of scope.

### Verified by the lead (re-read at the cited lines)

- `server.rs:4823-4825` returns 404 for remote URLs on `/api/uar/sync/stream`.
- `frontend/src/entities/sync.ts:342-344` selects `registerUarSseAdapters` for Postgres
  and remote Surreal.
- `settings/manager.rs:666-671` writes the effective map for non-Restart keys.
- `api/discovery.rs:871-958` (conversation policy handler) contains no bus or publish call.
- `runtime/manager.rs:1436` uses `SkillEvolutionConfig::default()`.
- `frontend/src/lib/realtime/topics.ts:15-18` subscribes `providers`, `models`, `settings`.
- Round 2: `api/live.rs:98-110` emits `reset` with `requires_snapshot: true`;
  `settings/provider_sync.rs:21-23` applies the default provider at boot;
  `api/providers.rs:367-399` swaps the live default; `team_execution/resolution.rs:39-48`
  checks binding revision; `config_manager.rs:103-130` validates on reload;
  `main.rs:94-99` fails startup on a bad config; `server.rs:1251-1263` reads persisted
  embedding values at boot; `entity-graph-core` `index.d.ts:966` exposes `persistNow()`.

Everything else in this section is subagent-sourced from source reading, with citations,
and not re-read by the lead.

## Recalled lessons that bear on this analysis

From `prior-context.md`:

- "PEM 5.0.1 … contains the flush fix, commit `84d010b6`" (project lesson, 2026-11-18).
  Bears on G1: the snapshot-flush behavior assumed below is the 5.0.1 behavior, and the
  page-load freeze confirmation on 5.0.1 is still unrecorded (assessment §0).
- The assess summary lesson ("8k models … 16,360 writes") bears on cand-012 and the
  atomic ingestion build item: the identity divergence above is the likely cause.
- The A2UI lessons (activity snapshots, `state.values`) do not bear on library choice
  here; A2UI/AG-UI protocol changes are a proposed exclusion.

Knowledge gaps listed in `prior-context.md`: none recorded.

## Candidate evaluation

Full machine record: `library-candidates.json`. Summary:

| id | Candidate | Gap | Verdict | Why |
|---|---|---|---|---|
| cand-001 | `PGliteWorker` (`@electric-sql/pglite/worker`, ships in installed 0.5.12) | G1, G8 | adapt | Built-in leader election; `live` extension exposed on the main thread through the `extensions` option. Seed load and migrations move into the worker `init`; the localStorage migration stays on the main thread (no `localStorage` in workers); tabs sharing one database must not overwrite each other's single graph-snapshot row. |
| cand-002 | Base UI `Combobox` 1.9.0 (installed; `components/ui/combobox.tsx`) | G6, G7 | adopt | Grouped items, `createItems`; already the house primitive. |
| cand-003 | `@tanstack/react-virtual` 3.14.23 (installed) | G6 | adopt | Base UI docs: memoize up to about 1k items, virtualize beyond. The model catalog is about 8k. |
| cand-004 | `@rjsf/core` 6.11.1 | G7 | reject | Brings its own form state and theme layer; the panel already renders from JSON Schema plus `x-control`. |
| cand-005 | `@jsonforms/react` 3.8.0 | G7 | reference | Its tester/renderer registry is the right shape for dispatching `x-control` kinds to Base UI controls. |
| cand-006 | `comlink` 4.4.2 | G8 | reject | Last publish 2024-11; one catalog worker does not justify the dependency. |
| cand-007 | `arc-swap` 1.9 via `ConfigManager` (incumbent) | G3 | adopt | Live config holder exists; consumers read the boot snapshot (`server.rs:490`). |
| cand-008 | `jsonschema` 0.49.4 + `schemars` 1 (incumbent) | G3 | adopt | Reload already validates (`config_manager.rs:103-130`). Reuse the same validation at boot to choose between the YAML and a last-known-good copy saved to disk. |
| cand-009 | `notify` 8 (incumbent watcher) | G3 | adopt | Watcher exists and fires. |
| cand-010 | `RealtimeBus`: Surreal LIVE + Postgres LISTEN/NOTIFY (incumbent) | G4 | adopt | Backend-neutral invalidation bus exists for all three backends. Extend topics; do not replace. |
| cand-011 | Impeccable (CLI 4.9.2 installed, registry 4.10.0; engine pinned by `IMPECCABLE_BIN` to 0.1.17 in the user's harness settings) | G2 | adopt | `impeccable update` refreshes skills but does not change the pinned engine; moving the engine is an operator change to their own settings. `init` writes PRODUCT.md. `document` derives DESIGN.md from shipped code, which conflicts with `docs/design/` as the visual authority, so DESIGN.md must be written from `docs/design/` and `document` used only afterwards to check drift. |
| cand-012 | PEM 5.0.1 `startLocalFirstGraph` (incumbent) | G1, G8 | adopt as-is | `persistDebounceMs` plus `persistNow()` (`index.d.ts:966`) give an app-side max-wait: call `persistNow()` when a streaming burst has deferred persistence past a bound. No type filter; bound the snapshot app-side. No PEM change needed. |

## Build required (no adoptable candidate)

Grouped by goal. Every item traces to an assessment gap or an operator decision.

**G1 — PGlite and PEM**
1. Fix the stale seed: regenerate the seed at the latest migration, or run migrations
   after a seed load, and enforce the existing `pglite:seed:check` guard.
2. PGliteWorker adoption per cand-001, including the main-thread localStorage migration
   and a multi-tab rule for the single snapshot row (leader writes, or per-tab keys).
3. One provider-qualified Model identity (`${providerId}/${modelId}`) across all four
   writers, including the realtime models snapshot, and one atomic catalog ingestion action.
4. Retire the Zustand stores that duplicate graph-owned records (Provider, Model,
   threads) and close the facade leak (`useGraphStore` re-export; raw upserts in feature
   files), per the project's entity-state rule. Chat message/stream stores are scoped by
   Plan; the operator's PEM-change exclusion stays.
5. Snapshot bounding: keep `RuntimeAgUiEvent` out of the graph or prune it, tune
   `persistDebounceMs`, and use `persistNow()` as the max-wait.

**G2 — Impeccable**
6. `impeccable update`; `init` to rewrite PRODUCT.md (decision 5); DESIGN.md written
   from `docs/design/`; audit and critique with the dual-critic gate (project UI rule).

**G3 — settings, YAML, never bricking**
7. Boot safety: last-known-good config saved to disk after each successful start;
   on a YAML load failure at boot, start from it with a visible warning. Same fallback
   for the persisted embedding values (`server.rs:1251-1263`) and a malformed provider row.
8. Live config consumers that re-run DB-wins reconciliation on reload, so YAML never
   overrides an operator-edited DB value (decision 1); visible drift with one-click resolution.
9. Provider seed aligned with the DB-wins spec: today `manager.rs:975-978` skips every
   existing provider row, so YAML additions to an existing provider never land (decision 1).
10. Truthful apply status end to end: stop marking unread or boot-only keys
    `"effective"` (`manager.rs:666-671`), report restart-required honestly, and make the
    console display that status (the frontend reads none of these fields today).
11. Remove the 15 namespaces from the UI (decision 6). For their stored DB rows, Plan
    chooses between leaving them inert and deleting them in a migration that never
    touches other keys (`gotchas.md:778-786`).

**G4 — realtime model changes at every level**
12. System level on the live path: the system-default picker calls the live default
    route (`POST /api/providers/{id}/default`) plus the provider's `default_model`, never
    the boot-only `llm.default_provider` field; retire `run_policy.global.model` from
    model selection (decision 2); make the picker reachable from primary navigation.
13. Agent wins over the system default (decision 9): keep precedence; show which agents
    keep an explicit model when the default changes.
14. Topic fixes: a conversation-policy topic; provider and default-provider changes
    invalidating `providers` on both backends; a real `models` writer or removal of the
    subscription; a public team/binding topic for the console (required by decision 3).
15. Team adoption at the next task boundary (decision 4): relax the binding-revision
    gate (`resolution.rs:39-48`) and route re-resolution at task admission; record
    per-task model provenance; fail visibly when the new route is unavailable; leave the
    unrelated `TEAM_REVISION_CONFLICT` concurrency checks alone.
16. Sub-agent inheritance: decide and implement whether conversation overrides reach
    sub-agents with their own model (`policy_intersection.rs:582-590`), consistent with
    agent-wins. Plan states the rule; the spec `multi-agent-orchestration` is silent today.
17. Postgres support means building and verifying the `postgres-backend` feature at the
    phase boundary.

**G5 — remove what does not make sense**
18. Remove or relink the unreachable admin pages, merge duplicate settings surfaces, and
    fix the RAG embedding control that discards co-edits (assessment G5 inventory).

**G6/G7 — pickers and no typing**
19. Schema-to-control dispatch: `x-control` picker kinds (provider, chat model,
    embedding model, enum source) mapped to cand-002/003.
20. Replace the free-text model and provider fields outside the settings schema panel
    (assessment G6 list of about 11 sites).

**G8 — workers**
21. A catalog normalize/filter worker (native module worker) if Plan's budget allows;
    PGliteWorker (item 2) is the primary worker.
22. Service-worker eviction of old hashed assets (`public/sw.js`).

**G4 (added after operator decision on question 5) — replicas**
24. Cross-replica refresh: each replica subscribes server-side to the existing change
    feed (cand-010) and refreshes its provider registry, settings cache and turn-time
    default model on `providers`/`settings` invalidations, so a change made through any
    replica takes effect on all of them for the next turn and the next snapshot. The
    boundary gate runs at least two replicas against one remote store.

**G9 — S-tier minimal UX**
23. Regroup Settings into about seven areas per `docs/design/`, with deep links and
    progressive disclosure (decision 5).

## Readiness mapping

| Decision | Contract section it satisfies | READY without new runtime code? |
|---|---|---|
| cand-001 PGliteWorker | Desired result: no stale-seed failure; WASM Postgres off the main thread | No: items 1-2 |
| cand-002/003 pickers | Desired result: zero free text for enumerable values | No: items 19-20 |
| cand-007/008/009 config | Desired result: YAML applied or shown as drift; never bricks | No: items 7-10 |
| cand-010 bus | Desired result: model changes pushed to open clients, remote included | No: items 12-17 |
| cand-011 Impeccable | G2 and decision 5 | No runtime code; PRODUCT.md, DESIGN.md |
| cand-012 PEM | Required safety: persist-before-publish unaffected | No: item 5 |

No decision reaches READY without new runtime code. That is expected for this phase.

## Research budget

Caps 8 queries per tier and 20 minutes. Used: tier 1 = 0, tier 2 = 5 (2 library-id
lookups, 3 doc queries), tier 3 = 6 (`npm view` for pglite, base-ui, comlink, rjsf,
jsonforms, impeccable), tier 4 = 0. Tier 1 was skipped because every gap maps to an
incumbent library or project glue. Source inspection by subagents (about 6, 7 and 15
minutes, parallel or sequential) exceeded the minutes cap; it is source reading, not
external research.

## Open questions

1. **PGlite bump 0.5.12 → 0.5.17.** Optional; the worker ships in 0.5.12. Plan decides
   after reading the changelog.
2. **Stored rows of removed namespaces:** leave inert or delete (item 11). Plan proposes;
   operator confirms if deletion is chosen.
3. **Impeccable engine pin.** `IMPECCABLE_BIN` points at engine 0.1.17 in the user's own
   harness settings. Whether to move it is the operator's call; this phase does not edit
   user settings.
4. **Sub-agent inheritance rule** (item 16): the operator decision covers agent vs
   system default, not conversation override vs sub-agent model.

5. **RESOLVED 2026-11-19 (operator): yes, replicas are in scope** (build item 24).
   Original question — **Multi-replica scope for decision 3.** Do realtime model changes have to be correct
   when several UAR replicas share one remote database? Today each replica loads the
   provider registry, settings cache and default model once at boot and subscribes to
   none of the change topics (critic-sourced; replica counts lead-verified).

(Round 1's question "how does the frontend handle provider Setting rows" is answered:
`/api/live` sends invalidations only, so the provider-topic fix is backend-side.)

## Recommendation

**READY to spec** (updated 2026-11-19 after operator decision on question 5: replicas
are in scope). The change inventory is the 24 build items above plus the unresolved
review findings below, which Spec must turn into changes rather than drop.
The stack is not contested and every gap has an incumbent or a build item. Plan still
needs three inputs that are not analysis outputs: the adopted-outcome contract bound to
the phase (method-context reports "adopted contract unavailable"), operator risk
acceptance, and the per-change disposition of overlapping open OpenSpec changes
(decision 7). Open questions 2 and 4 must be answered before Plan completes.

## The uncomfortable part

Round 1 of this analysis repeated the assessment's error in the opposite direction. It
narrowed decision 3 to "topic fixes", called the phase ready, and missed that the most
visible operator action, changing the system default model, went through a settings key
that only takes effect at restart while the API calls it effective. A plan built from
round 1 would have shipped a polished picker that does nothing until restart and passed
every scenario on embedded Surreal. Round 2 fixes the inventory, but its corrections
have not been re-reviewed with a fresh critic yet, and the phase is now large: 24 build
items across backend, frontend, two persistence backends and a redesign. Plan should
expect to split it into child phases.

## Adversarial review record

- Round 1: harness-native `artifact-critic` (same model family as the producer;
  `cross_model_check: same-model-collision`). The external REST judge was not retried
  because the permission classifier blocked it during Assess. Verdict BLOCK: 5 critical,
  9 warning, 3 suggestion. Findings: `review/analyze/findings-round1.json`. Round-1
  artifacts preserved as `review/analyze/analysis-round1.md` and
  `library-candidates-round1.json`.
- All five critical findings were re-read at source by the lead and addressed: readiness
  coverage (items 1-4, 16, 18, 20, 23; G5/G9 added), boot brick (delta 7, item 7),
  default provider boot-only (delta 5, item 12), binding revision gate (delta 6, item 15),
  decision-assigned work (items 9, 11, 12).
- Warnings addressed: `persistNow` (cand-012), fourth Model writer (delta 1, item 3),
  invalidation-only `/api/live` (delta 1), PGliteWorker risks (cand-001), Impeccable pin
  and `document` conflict (cand-011, OQ3), team topic required (item 14), DB-wins on
  live reload (item 8), frontend apply status (item 10). One warning stays open: the
  Postgres "push already works" claim is subagent-sourced and not lead-verified beyond
  the cited trigger and listener lines.

## Unresolved review findings

The two-round review cap is reached. Round 2 (`review/analyze/findings-round2.json`,
harness-native, same model family) returned BLOCK: 4 critical, 5 warning, 4 suggestion.
These are carried to Spec and Plan as open. They are not fixed in this artifact and have
not been re-reviewed.

Critical:
1. **Multi-replica remote push.** (Scope settled by the operator: in scope; build item
   24 now covers it. Design not yet reviewed.) Each replica holds the provider registry
   (`settings/provider_sync.rs:9-12`, called once from `server.rs:1173`), the settings
   cache (`manager.rs:861-888`) and the turn-time default model
   (`runtime/manager.rs:5946,6664,11317`) in memory from boot. No process subscribes to
   providers, settings or models topics, so topic fixes alone would make clients on
   another replica refetch stale data. Lead-verified: replica counts. Operator question 5.
2. **G7 still lacks build items** for the generic schema panel's CSV/raw-JSON/bare-number
   rendering of arrays, objects and numbers across 13 panels (`assessment.md:211`) and for
   the hand-written free-text controls listed at `assessment.md:213-223` (team launch
   JSON, binding references, workspace and skill-import paths, CSV lists, retry and
   status fields). Item 19 covers picker kinds only.
3. **A second system-model control remains.** `llm.default_provider` is still a schema
   field with `x-control: text` (`manager.rs:2851-2856`, lead-verified). Decision 2 makes
   the registry default the only system control, so this field must leave the settings
   UI (or become a read-only view of the live default), not become a picker.
4. **Unsafe last-known-good fallback.** Item 7 would fall back when boot validation
   fails, but that validation includes deliberate security refusals (`config.rs:903-909`,
   lead-verified: remote shared-secret verification refuses the fallback JWT secret). A
   tightened security config that fails would boot on an older, weaker one. The fallback
   must not cover security refusals, and what is saved to disk must exclude env- and
   vault-merged secrets (`AppConfig` derives only Deserialize).

Warnings:
- cand-008 should not be "adopt": `jsonschema` is not in the config validation path, and
  boot already runs `AppConfig::validate`.
- Item 15 must relax the binding-revision gate only for model-binding changes; relaxing it
  wholesale would admit skill and package changes mid-run, which decision 4 does not cover.
- Item 3 fixes the Model id but not the divergent record shape (`providers-graph.ts:84`
  hard-codes `vision: false`).
- Item 18 misses the Providers banner that shows `models[0]` and the reload endpoint that
  reports success with no effect.
- The round-2 header originally claimed every round-1 critical was addressed; corrected
  above.

Suggestions: evaluate a path picker (no Tauri dialog plugin installed); cand-012's risk
text predates `persistNow()`; license fields are not all SPDX; decision log count fixed.
