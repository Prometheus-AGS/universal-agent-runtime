# Analysis — uar-uiux-architecture-2026-11

Generated 2026-11-19T19:06:35Z. Stage: Analyze (after Assess, before Spec).
Mode: **stack specified** — the stack is fixed by `versions.toml`, `frontend/package.json`
and the operator decisions of 2026-11-19. Candidates were ranked inside that stack; no
stack discovery was run.

Method: source inspection, local registry reads and documentation lookups only. Nothing
was built, tested or run (full-phase verification boundary; VLBV). Two read-only
subagents inspected source and I re-checked their load-bearing claims at the cited
lines (listed under "Verified by the lead").

## Delta from the assessment (read first)

Analyze changed four assessment conclusions. Spec and Plan must use these, not the
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
   **The real gaps are per topic**, not per backend:
   - **Conversation policy publishes nothing on any backend.** The
     `conversation_policies` / `principal_conversation_policies` tables have no topic,
     the handlers make no bus call (`api/discovery.rs:871-958, 1107-1160`), and there is
     no Postgres trigger.
   - **Providers arrive on the wrong topic on Surreal.** Providers are stored as
     `provider.{id}` rows in the `settings` table (`settings/manager.rs:975,1050`), so
     Surreal emits them on `settings`. Postgres routes `provider.%` keys to `providers`
     (`notify_entity_change.sql:66-72`). The frontend subscribes to `providers` and
     expects `Provider` entities (`frontend/src/lib/realtime/topics.ts:15`). So provider
     changes reach the console as `Setting` rows on Surreal and as `providers` events on
     Postgres. Runtime behavior is unverified.
   - **No writer to the `models` topic was found** on either backend; the frontend
     subscribes to it (`topics.ts:16`). Treat as "never fires" until shown otherwise.
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
| cand-001 | `PGliteWorker` (`@electric-sql/pglite/worker`, ships in installed 0.5.12) | G1, G8 | adapt | Built-in leader election; `live` extension exposed on the main thread through the `extensions` option. Must move seed load and migrations into the worker `init`. |
| cand-002 | Base UI `Combobox` 1.9.0 (installed; `components/ui/combobox.tsx`) | G6, G7 | adopt | Grouped items, `createItems`; already the house primitive. |
| cand-003 | `@tanstack/react-virtual` 3.14.23 (installed) | G6 | adopt | Base UI docs: memoize up to about 1k items, virtualize beyond. The model catalog is about 8k. |
| cand-004 | `@rjsf/core` 6.11.1 | G7 | reject | Brings its own form state and theme layer; the panel already renders from JSON Schema plus `x-control`. |
| cand-005 | `@jsonforms/react` 3.8.0 | G7 | reference | Its tester/renderer registry is the right shape for dispatching `x-control` kinds to Base UI controls. |
| cand-006 | `comlink` 4.4.2 | G8 | reject | Last publish 2024-11; one catalog worker does not justify the dependency. Native module worker plus `postMessage` is enough. |
| cand-007 | `arc-swap` 1.9 via `ConfigManager` (incumbent, `src/config_manager.rs`) | G3 | adopt | Live config holder already exists; handlers snapshot it once at `server.rs:490`. The work is reading it live, not adding a library. |
| cand-008 | `jsonschema` 0.49.4 + `schemars` 1 (incumbent) | G3 | adopt | Validate YAML against the derived schema before swapping it in; keep last-known-good on failure. |
| cand-009 | `notify` 8 (incumbent watcher) | G3 | adopt | Watcher exists and fires; the reload has no effect today. |
| cand-010 | `RealtimeBus`: Surreal LIVE + Postgres LISTEN/NOTIFY (incumbent) | G4 | adopt | Backend-neutral bus exists for all three backends. Extend topics; do not replace. |
| cand-011 | Impeccable CLI (installed 4.9.2; registry 4.10.0) | G2 | adopt | `impeccable update`, then the `init` and `document` skill steps produce PRODUCT.md and DESIGN.md. |
| cand-012 | PEM 5.0.1 `startLocalFirstGraph` (incumbent) | G1, G8 | adopt as-is | Only `persistDebounceMs` is configurable; no max-wait, no type filter; the adapter receives already-serialized JSON. Snapshot bounding must be app-side. |

### Build required (no adoptable candidate)

1. **Schema-to-control dispatch with picker kinds.** Extend `x-control` with
   `provider-picker`, `model-picker` (chat and embedding), `enum-source` and similar, and
   map each to cand-002/003 controls. No library maps a settings schema onto this
   project's Base UI kit and entity hooks.
2. **One provider-qualified Model identity and atomic catalog ingestion.** Settle
   `${providerId}/${modelId}` everywhere, then publish the catalog with one graph
   ingestion action instead of three per-row writers.
3. **Topic fixes:** a conversation-policy topic (Surreal table plus Postgres trigger, or
   a host publish on write), provider events on the `providers` topic for Surreal, a
   real writer for `models` or removal of the subscription, and whatever public topic the
   team console needs.
4. **Live config read path and YAML safety.** Handlers read `ConfigManager::current()`
   per request; reload validates first; invalid YAML keeps last-known-good and shows a
   visible warning; drift view with one-click resolution.
5. **Truthful apply status.** Stop marking unread keys `"effective"`
   (`settings/manager.rs:666-671`); the 15 namespaces leave the UI.
6. **Team adoption at the next task boundary.** Re-resolve routes at task admission,
   record per-task model provenance, fail visibly when the new route is unavailable, and
   leave the unrelated `TEAM_REVISION_CONFLICT` concurrency checks untouched.
7. **Snapshot bounding, app-side.** Keep `RuntimeAgUiEvent` out of the graph or prune it
   with a graph action, and tune `persistDebounceMs`. A max-wait would need a PEM change.
8. **Service-worker cache cleanup.** `public/sw.js` keeps one fixed cache name
   (`uar-v1`, line 4) and never evicts old hashed assets.
9. **Pinned-agent visibility.** When the system default changes, show which agents keep
   an explicit model (operator decision: agent wins).

## Readiness mapping

| Decision | Contract section it satisfies | READY without new runtime code? |
|---|---|---|
| cand-001 PGliteWorker | Desired result: no stale-seed failure; main thread free of WASM Postgres | No: worker module, seed/migration move |
| cand-002/003 pickers | Desired result: zero free text for enumerable values | No: dispatch build item 1 |
| cand-007/008/009 config | Desired result: YAML applied or shown as drift; never bricks | No: build item 4 |
| cand-010 bus | Desired result: model changes pushed to open clients, remote included | No: build item 3 |
| cand-011 Impeccable | G2 and the design-authority decision | No code; it writes PRODUCT.md/DESIGN.md |
| cand-012 PEM | Required safety: persist-before-publish unaffected | No: build item 7 |

No decision reaches READY without new runtime code. That is expected for this phase.

## Research budget

Caps 8 queries per tier and 20 minutes. Used: tier 1 = 0, tier 2 = 5 (2 library-id
lookups, 3 doc queries), tier 3 = 6 (`npm view` for pglite, base-ui, comlink, rjsf,
jsonforms, impeccable), tier 4 = 0. Tier 1 (GitHub search) was skipped because every gap maps to an incumbent
library or to project-specific glue; tier 4 was not needed. The minutes cap was
exceeded by the two source-inspection subagents (about 6 and 7 minutes each, in
parallel), which are source reading rather than external research.

## Open questions

1. **PEM change allowed?** A persistence max-wait or a type filter needs a PEM release.
   The assessment proposed excluding PEM changes "beyond what G1/G8 strictly require".
   Analyze's view: app-side pruning is enough for this phase, so no PEM change.
   Operator confirmation is needed only if Plan finds pruning insufficient.
2. **PGlite bump 0.5.12 → 0.5.17.** Optional; the worker ships in 0.5.12. Plan decides
   after reading the changelog.
3. **Spec-stage source check:** how the frontend handles `provider.*` `Setting` rows on
   the `settings` topic today (ignored, or mapped to `Provider`). This decides whether
   the Surreal provider-topic fix is a backend or a frontend change.

## Recommendation

**READY to spec.** Every gap has either an adopted incumbent or a named build item, and
the stack is not contested. Before Plan can finish, it needs three items that are not
analysis outputs: the adopted-outcome contract bound to the phase (method-context
currently reports "adopted contract unavailable"), operator risk acceptance, and the
per-change disposition of the overlapping open OpenSpec changes (operator decision 7).

## The uncomfortable part

The assessment overstated the remote-realtime gap, and decision 3 was framed on that
overstatement. The operator's requirement still stands and is cheaper than recorded:
topic fixes, not a new change source per backend. But Postgres is a non-default build
feature, so "supported in this phase" means building and verifying a second feature set
at the boundary, which roughly doubles that part of the integration gate.
