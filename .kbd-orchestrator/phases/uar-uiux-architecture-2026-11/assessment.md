ASSESSMENT: uar-uiux-architecture-2026-11
Project: universal-agent-runtime (UAR)
Date: 2026-11-19
Codebase baseline: `main` at 46dd7148 (PEM 5.0.1 bump merged in #529); React 19 operator console in `frontend/`, Rust runtime in `src/`.
Cross-tool progress: none — the phase was created today, `progress.json` has no changes.
Method: source inspection only. Nothing was built, run, type-checked or tested (full-phase verification boundary; VLBV). Paths in `frontend/` are relative to `frontend/src` unless marked.

## 0. Preconditions and process state

- **KBD runtime does not know this phase.** `prometheus kbd status --json` reports `activePhase: null` at revision 3436 (run `fix-runtime-settings-namespace-routes-20260825T091750Z`). `kbd-new-phase` took its local path. Consequences:
  - `node shared/method-context.mjs --phase uar-uiux-architecture-2026-11` fails with "method context needs a resolvable canonical phase". The selected method is therefore recorded from the project default (VLBV), not from a resolved method context.
  - `prometheus kbd stage enter|transition` cannot record the assess stage until the phase is created and activated canonically (`prometheus kbd phase create` + `phase activate`). That is an append-only canonical write; it needs operator approval.
  - `.kbd-orchestrator/position-reminder.txt` is stale (revision 700, a phase from October).
- **Recalled lessons that apply** (from `prior-context.md`):
  - "`flush()` now applies all coalesced changes … in one `store.setState` recipe … for the models snapshot, one write instead of 16,360." — PEM 5.0.1 fixed the freeze from per-row realtime writes. It shapes this assessment: the graph is now usable for the full model catalog, but this phase must not reintroduce per-row ingestion (37 grandfathered per-row write sites remain, §1).
  - "Once 5.0.1 is published … confirm in the browser that the page-load freeze is gone without the swapped-in build." — that browser confirmation was never recorded. It stays UNVERIFIED and should be one boundary scenario.
- **Knowledge gaps:** none recorded in `prior-context.md`.
- **Prior analysis to reconcile against:** `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/receipts/config-lifecycle-map-2026-10-19.md` (409 lines; bootstrap order, the "ADR-010" DB-authority rationale, file reload as a documented gap). `openspec/specs/skill-config-reconciliation` ("reconciliation fails safe when the configuration source looks broken") is the existing YAML→DB precedent pattern for G3.

## 0.5 In-flight OpenSpec changes (added after adversarial review, round 1)

There are 259 unarchived changes under `openspec/changes/`. Many overlap this phase, and several have task ledgers that contradict the code. Analyze/Plan must give each one an explicit disposition (archive, supersede or continue) before new changes are proposed. Confirmed examples:

- **`make-config-authoritative-on-boot`** (2026-06-02; 15/18 tasks checked):
  - Its proposal records "Product decision (R3, confirmed): env/YAML is **authoritative on every boot**".
  - It marks "[x] Changed `seed_providers_from_registry` from insert-if-absent to upsert-on-every-boot".
  - That ledger was true when written: R3 was implemented in `c41da57e` (2026-06-02, in `main`).
  - It was then **reverted inside an unrelated feature commit**, `7a6aa305` (2026-07-20, "feat: add scoped chat control plane and A2UI loop"). That commit removed the "Config-authoritative: always upsert" code and restored insert-if-absent (`src/uar/settings/manager.rs:976-978`).
  - The revert came 9 days after the archived DB-wins change `2026-07-11-certify-provider-model-settings-flow`. The current `provider-model-settings-certification` spec (last edited 2026-11-15) says DB wins for operator-edited rows.
  - **So R3 looks superseded, but by a silent revert. The open change and its ledger now describe code that no longer exists.** Whether the revert was deliberate needs operator confirmation.
- **`config-hot-reload-vault`** (2026-07-14; 28/30 checked): claims handlers access the live config. The code snapshots it once at boot (`server.rs:490`, confirmed).
- **`migrate-settings-mutations-and-retire-stores`** (0/15): mandates a "terminal aesthetic" and decides that `settings-types-meta-store` "REMAINS — schemas are not entities". It conflicts with G9 and informs G1.
- **`impeccable-uiux-audit`** and **`admin-agent-provider-first-model-picker`**: contain only `.openspec.yaml`; they are empty shells.
- **Other open changes in scope by name (unreviewed):**
  - Settings: `add-settings-fetcher-and-hook`, `add-settings-form-cache`, `migrate-settings-reads-transitional`, `rewrite-use-settings-hook`, `retire-settings-store-and-bus-source`, `audit-doc-flip-settings-direct`.
  - Providers, models and agents: `migrate-providers-models-agents-to-entity-mgmt`, `retire-providers-admin-store`, `providers-page-direct-{reads,mutations}`, `migrate-providers-page-to-helpers`, `provider-meta-singleton`, `agent-selector-and-store-retire`, `agent-entity-realign`, `add-models-fetcher-and-hook`, `effective-policy-model-backfill`, `harness-model-profiles-routing`, `per-model-context-strategy`.
  - Page redesigns: `migrate-{models,knowledge,memory,skills,compiler}-page-direct-and-redesign`, `migrate-admin-knowledge-to-entity-mgmt`, `uiux-remediation-wave-1`, `setup-design-skills-and-playwright`.
  - Other: `config-rs-schemars-migration`, `desktop-data-layer-pglite-oxide` (empty).
  - Their ledgers have not been reconciled against code in this assessment; treat their checked tasks as unverified.

## IMPLEMENTATION STATUS

### G1. PGlite architecture and client-side state with PEM — PARTIAL

- **PGlite: PARTIAL.** `@electric-sql/pglite ^0.5.12`, opened on the main thread at `platform/pglite/client.ts:106` (`idb://uar-threads`, `live` extension). No `PGliteWorker`, no multi-tab leader election (no `navigator.locks` / `BroadcastChannel` in `src`); every tab opens the IndexedDB database itself.
  - Stores `threads`, `messages`, `run`, `run_event`; migrations v1–v4 (`platform/pglite/migrations.ts:74-87`).
- **Defect, confirmed in source, not at runtime:** the schema seed is stale.
  - Only `platform/pglite/pglite-seed-v3.tar.gz` exists (last committed 2026-08-08; migration v4 landed 2026-11-06), and `assets.ts:3` imports it.
  - `client.ts:113-117` skips `runMigrations()` when a seed was loaded. The seed is loaded only when the IndexedDB database does not exist yet.
  - Impact: **in the first session after a fresh install**, the schema lacks migration 4 (`run_id`, `client_turn_id`, `history_position`), which `insertMessage` (`client.ts:262`) writes, so message writes fail until a reload. On the next open, `runMigrations` applies v4.
  - The existing guard `pglite:seed:check` (`frontend/package.json:15`, digest comparison) would catch this but was not enforced.
  - Runtime confirmation is required.
- **Graph restore at startup: DONE.** `lib/db-context.tsx:192-196` → `entities/bootstrap.ts:35-107` (`createPGlitePersistenceAdapter`, `startLocalFirstGraph`, `await ready`, then `initSyncTransport`).
- **PEM facade boundary: PARTIAL.** Only `platform/entities/**` imports the PEM package (enforced by `scripts/check-frontend-boundaries.mjs:451`, repo root). The facade still leaks the raw store:
  - `platform/entities/runtime.ts:18` re-exports `useGraphStore`, and `index.ts:7` re-exports `runtime`.
  - A component subscribes to raw graph state (`features/cost/ui/cost-dashboard-page.tsx:3,28`).
  - 27 feature `.ts` files call `useGraphStore` and issue raw upserts (for example `features/models/model/models-store.ts:27-49`).
  - Provider and Model have no typed domain hooks in `platform/entities`.
- **Business state outside the graph: PARTIAL.** 31 Zustand stores (12 in `stores/`, 19 in `features/*/model`). Worst:
  - `stores/chat-message-store.ts` (1206 lines) and `stores/chat-stream-store.ts` (1508 lines): all messages, no graph references.
  - Threads exist three times: PGlite, Zustand (`thread-registry-store.ts`) and graph `Thread`, reconciled by `use-thread-graph-sync.ts`.
  - `features/models/model/models-store.ts:10` and `features/providers/model/provider-models-store.ts:12` duplicate graph-owned Provider/Model records. The boundary gate misses them because it matches type names.
  - `features/settings/model/settings-types-meta-store.ts` holds settings schema metadata outside the graph. `SettingsType` is registered (`entities/schemas.ts:86`) but never written, so this is not a duplicate. It follows the prior decision "schemas are not entities" (`migrate-settings-mutations-and-retire-stores`). Changing that is an operator decision.
  - Server lists with no graph type at all: credentials, auth keys, A2UI schemas, user JWT settings, health.
  - 34 component files read data through `@/features/*/model` Zustand hooks, not typed `@/platform/entities` hooks (36 files do use the platform hooks).
- **Per-row graph ingestion:** 37 grandfathered per-row write sites (`check-frontend-boundaries.mjs:34-73`).
  - The model catalog is written to the graph row by row in three places (`models-graph.ts:45`, `models-store.ts:49`, `providers-graph.ts:86`). All three are grandfathered.
  - Separately, it is duplicated into Zustand (`provider-models-store.ts:26-35`).
  - **The writers disagree on Model identity.** `models-graph.ts:22,45` and `models-store.ts:28,49` key Models as `${providerId}/${modelId}`. `providers-graph.ts:86` keys them by bare `row.id` (the catalog model id, `features/models/api/models-api.ts:30-31`).
    - The same model therefore exists twice in the graph.
    - Bare ids collide across providers that share a model name; the last writer wins and `provider_id` flips.
    - This plausibly explains the 16,360 writes for about 8k models in the recalled lesson.
    - Any atomic-ingestion plan must first settle one provider-qualified Model identity.
  - `loadModelsForProvider` refetches the full catalog for each provider. The batch `upsertEntities` is used in only two domains (A2UI actions, presentations). This violates the project rule "never publish a fetched entity list by looping over rows".
- **Snapshot cost:** PEM persists the entire graph as one JSON text row (`structuredClone` + `JSON.stringify` + UPSERT).
  - Persistence uses a trailing 50 ms debounce with no max-wait (`entity-graph-core dist/index.mjs:1343-1376`). Under steady sub-50 ms updates during streaming it is deferred until a quiet gap.
  - The risks are therefore persistence starvation (loss on crash) plus a large serialization burst on the main thread, which also runs WASM Postgres.
  - The graph holds about 8k `Model` entities plus an unbounded, never-pruned set of `RuntimeAgUiEvent` rows (`entities/runtime-ingest.ts:76`). Those rows are also written to the PGlite `run_event` table.
  - Levers: `persistDebounceMs` (a `startLocalFirstGraph` option, not set in `bootstrap.ts:37`), and excluding or bounding `RuntimeAgUiEvent` in the snapshot.
- **Specs:** `frontend-local-first-persistence` and `pglite-oxide-portability` both leave `## Purpose` as "TBD". Neither says anything about workers or multiple tabs. The pglite-oxide desktop path is MISSING by design (the spike failed on x86_64-apple-darwin); `openspec/changes/desktop-data-layer-pglite-oxide/` is an empty directory.

### G2. Impeccable CLI update/initialize, then evaluate and improve — NOT MET

- **Tool versions disagree:** the `impeccable` CLI on PATH is 4.9.2; the skill's `scripts/VERSION` is 0.1.19; `IMPECCABLE_BIN` points at engine 0.1.17. `impeccable check` / `update` were not run (that is execution work).
- **Context:**
  - `frontend/PRODUCT.md` (schema 1) is narrow: it covers the Presentation feature only, and states "This work extends the existing admin console. It does not redesign the application". That directly contradicts this phase's goals G5 and G9, so it must be rewritten before any Impeccable command is trusted.
  - **No `DESIGN.md` exists anywhere.** `impeccable context --target src/App.tsx` resolves `designPath: null`.
  - The workspace packages (`packages/a2ui-*`, `entity-graph-react`) appear as separate Impeccable targets.
- **Existing critiques:** `.impeccable/critique/` holds 6 critiques from July to September (scores 22–29/40). The ai-settings layout fix and the run-inspector fix landed. The Skills P1/P2 items (dirty-draft loss, `h-7` row actions) were deliberately deferred. The A2UI P1s (empty/error lifecycle, field-error association) are unverified.
- **Design authorities drift:**
  - `docs/UI_DESIGN.md` (January; Material 3, purple) is stale, and the August assessment already said to retire it.
  - `docs/ui-design-authority.md` (August) keeps Flat 2.0 binding, with Base UI instead of shadcn.
  - `docs/uar-uiux-assessment-and-implementation-plan-2026-08-01.md` cites shell files that no longer exist.
  - `docs/design/` (October prototypes, `react-implementation-plan.md`, `brand-spec.md`) is the newest reference: a single collapsible sidebar with a tabbed inspector, and Settings grouped into about 7 areas. The code has drifted far from it (30 settings sections, §G5).
  - These must be reconciled into one `DESIGN.md`.

### G3. Settings persisted in the DB and kept in sync with YAML without bricking the system — PARTIAL

- **YAML loading and env layering: DONE** (`src/config.rs:1741-2039`).
- **Hot reload: STUB.** `src/config_manager.rs:152-200` watches the file (`notify` + `ArcSwap`) and keeps the last good config on a failed reload.
  - But `config_manager.current()` is read once, at `src/server.rs:490` (confirmed); `AppState.config` is that boot snapshot.
  - A YAML change after boot changes nothing at runtime, and the DB is not re-reconciled.
  - `POST /.well-known/uar-config/reload` (`server.rs:2426-2429` → `:3742`) returns 200 although handlers keep reading the snapshot: a success response with no runtime effect.
  - The open change `config-hot-reload-vault` claims otherwise (§0.5).
- **DB persistence and API: DONE.**
  - `settings_types` + `settings` tables, JSON-Schema validation on write, optimistic revision tokens.
  - Routes in `src/uar/api/settings.rs` (GET/PUT/DELETE per key, namespace bulk PUT, `/drift`, `/types`), plus `/api/uar/settings/effective`.
- **DB values used at runtime: PARTIAL.**
  - Read live: resilience, `harness.*`, `prompt_caching.enabled`, intent classifier, skills matching, `run_policy.global`, governance, providers and the default provider.
  - Read only at boot (restart needed): `llm.embedding.*` outside the hot-swap path, `rag.*`, `memory.*`, `native_tools.*`, provider hydration.
  - **Persisted but with no reader found:** `server.*`, `security.*`, `persistence.*`, `llm.model`, skill_evolution, llm_failover, context_management, file_processing, sycophancy, sandbox, vision, acp, kreuzberg, unstructured, mistral_ocr. The UI lets operators edit these, and the edits have no effect.
  - **Apply-mode metadata is false for these keys.** `manager.rs:73-80` (`setting_apply_mode`) labels every namespace except governance, server, security and persistence as `NextTurn`, including inert ones. `api/settings.rs:557-600` returns `apply` / `application_status` (`effective|pending|restart_required`) / `saved` / `effective`. The frontend `SettingWithMeta` type (`frontend/src/types/index.ts:320-341`) declares none of these fields; it models only a nested `meta`. No frontend code reads `application_status`. This is part of the contract mismatch below.
  - YAML-only (no DB namespace): skill_source_roots, service_instance, supervisor_recovery, runs, sessions, project_instructions, world_state, guardrails.
- **Precedence and drift: PARTIAL, boot-only** (`src/uar/settings/manager.rs:191-343`).
  - Missing row → seeded from config. API-owned row that differs → DB wins, `is_drift` held in memory. Otherwise → overwritten from YAML.
  - API ownership is inferred from `updated_at` on Surreal; Postgres has a durable `source` column.
  - Provenance is config-vs-api only; env, CLI and default are not distinguished.
  - **The frontend never shows drift or source.** It expects `meta.{source,is_drift}` (`frontend/src/types/index.ts:320-345`); the backend sends flattened fields (`settings.rs:557-620`). This is a contract mismatch.
- **YAML→DB for existing providers: MISSING.** YAML edits to an existing provider are silently ignored with no drift flag (`manager.rs:976-998`).
  - **This is stricter than the current spec.** The spec protects only API/UI-created rows (`provider-model-settings-certification/spec.md:42`), and requires the seed to "merge missing YAML entries" (:56).
  - The code skips *every* existing `provider.*` row, including config-seeded rows no operator touched and models newly added in YAML for an existing provider. It may violate the merge clause.
  - General settings already follow the narrower rule (`manager.rs:276-305`: API-owned rows win and show drift; config-owned rows are overwritten from YAML).
  - The earlier R3 decision (YAML wins on every boot) was implemented and then silently reverted (§0.5).
  - It also conflicts with the goal's "stay in proper sync".
- **Versioning, rollback, last-known-good for DB values: MISSING.**
  - Reset (DELETE) evicts the row, which is not re-seeded until restart.
  - Preserved API values are not re-validated when a settings-type schema changes.
- **Paths that can stop startup** ("render the system unusable"):
  1. A wrong-typed or out-of-range persisted `llm.embedding.vector_dimension` / `send_dimensions` hits `?` / `ensure!` and aborts boot (`server.rs:1251-1263`).
  2. With `LOCAL_ONLY`, a non-loopback enabled provider in the DB bails boot (`server.rs:1189-1220`). This is intentional, but recovery requires editing the DB.
  3. **Invalid YAML or a failing security validation aborts every boot** (`src/main.rs:94-99`; `config_manager.rs:42-48`). There is no fallback to the DB or to a last-good file.
     - Because a failed hot reload keeps the last good config silently, an operator who saves broken YAML while the server runs sees nothing until the next restart, and then boot aborts.
     - This **delayed brick** is the most likely "renders the system unusable" path for G3.
- **Paths where boot succeeds but the system is degraded or unusable:**
  4. One malformed provider row fails the whole DB provider load (`manager.rs` `load_provider_configs_from_db` ~1168-1184, `serde_json::from_value(...)?` inside the loop).
     - The caller only logs (`server.rs:1173-1179`).
     - Boot continues with **zero** persisted providers and no default provider, silently.
  5. An edit that touches a route a running team captured blocks that team's tasks on resume (`src/llm/team_profile.rs:62-110`; `team_execution/resolution.rs:76-101` checks only the team's own `effective_models`). Triggers:
     - the captured model or provider is disabled or removed;
     - that route's execution profile, reasoning, endpoint or protocol changes (`settings_revision` / `TEAM_REVISION_CONFLICT`; `src/llm/registry.rs:165-192`);
     - that model's catalog price changes.
     - Edits to other providers or models do not block (per the round-2 reviewer; not independently re-verified).
- **Dangling reference:** code cites "ADR-010" as the settings-authority decision (`server.rs:1236`, `src/llm/local_only.rs`), but `docs/adr/0010` is about the A2UI renderer.
- **Spec coverage: PARTIAL.** `skill-config-reconciliation` covers skills only (fail-safe reconciliation). No spec covers general YAML↔DB precedence, drift resolution, hot reload, or startup protection from bad persisted values. Two contradictory authority decisions exist in change records (§0.5).

### G4. Realtime provider/model changes at system, agent, team and conversation levels — PARTIAL

- **Per-turn resolution (backend): DONE.** `src/uar/domain/policy.rs:510-566` resolves Global (`run_policy.global`) → Agent → Conversation → Turn, records provenance per field, and is recomputed every turn.
  - Agents and conversation policy are re-read from the store each turn (`server.rs:7609-7737`, `api/discovery.rs:274-283,699-735`).
  - No agent, compiled-agent or LLM-client cache was found.
  - Agent and conversation changes therefore take effect on the next turn.
- **System level: PARTIAL/MISSING (corrected after review).** Three competing "system default" mechanisms exist, and none is a reachable, effective system picker:
  1. `run_policy.global.model` sits in the resolution chain, but **no UI writes it**. The frontend has no `run_policy` section; its only reference is `platform/entities/presentation-assignments/api.ts`, and the backend presentation-policy route (`api/settings.rs:840-868`) edits only presentations.
  2. **The registry default provider plus that provider's `default_model`.** It is set from the Providers page, or from the Models page "set default", which is unreachable (§G5). It is consulted only when the agent has no `provider.default` and `global.model` is unset (`server.rs` `resolve_requested_model`; `policy.rs:683-689`; `defaults.rs:26-36`).
  3. **`llm.model`.** Editable in Settings, but it has no runtime reader (§G3).
  - Which of these is "the system level" is an operator decision.
- **Gaps:**
  - **Teams are pinned at admission.** Member models come from binding `modelBindings` (`compiler/collaboration/bindings.rs:744-801`) and are captured as `effective_models`.
    - Any drift on resume raises `TEAM_ROUTE_PROFILE_MISMATCH` and blocks the task (`team_execution/resolution.rs:71-100`, `continuations.rs:150-175`).
    - Changing a member's model requires a new binding revision; a running team never sees it.
    - Beyond member changes, **edits to a captured route also block that team** (disable, profile/endpoint change, price change; see §G3 path 5).
    - This is a deliberate safety property that conflicts with the goal, so the operator must decide.
  - **Bound-agent targets are pinned per turn.** For `uar:agent:` targets, `admit_bound` sets `turn_policy.model` at Turn scope, which outranks Conversation (`server.rs:7552-7577`), and policy agent changes are rejected (`:7695`). This affects API clients only; the frontend does not use bound targets.
  - **Conversation overrides do not reach sub-agents** that carry their own model (`runtime/thread/policy_intersection.rs:582-590`).
  - **The system level is not a force.** `run_policy.global.model` applies before the agent scope, so any agent with an explicit model ignores it.
  - **No realtime push for conversation policy.** `POST /api/uar/sessions/{id}/agent-config` and `PUT /api/uar/conversations/{id}/policy` publish no topic. SSE sync (`/api/uar/sync/stream`) exists only with local Surreal and returns 404 with a remote database (`server.rs:4823`).
  - **Embedding model:** `llm.embedding.*` hot-swaps after pre-validation (`api/settings.rs:1148-1240`), except when a failover pair is configured (`:1198`). `memory.*` needs a restart.
- **Spec:** `multi-agent-orchestration` specifies nothing about child or team-member model precedence.

### G5. Remove UI/UX that does not make sense — NOT MET (inventory below)

- **Unreachable pages:** 8 of 19 admin sections have no nav entry, no command-palette entry (`pages/admin-page.tsx:32-52`, `app/shell/command-palette.tsx:40`) and no in-app link: protocols, auth, cost, memory, compiler, credentials, models, mcp-health.
  - Two more lack nav entries but are linked: the runtime cockpit (`/admin` default; `pages/chat-page.tsx:81`) and approvals (`teams/ui/run/member-lane.tsx:164`).
- **Duplicate editing surfaces for the same records:**
  - Settings "Provider Overrides", "Model Files", llm and llm_failover vs the Providers, Models and Credentials pages.
  - Settings "Agents" vs the Agents page.
  - Settings "Skills" and "Skill Evolution" vs Skills and Compiler.
  - Settings "Memory" and "Knowledge Bases" vs Knowledge and `/admin/memory`.
  - Settings "Presentation Policy" vs Presentations.
  - Embedding provider/model/chunking edited in 3 places.
- **Controls that write nothing useful:**
  - The RAG embedding provider/model fields (`ai-settings-panels.tsx:504-518`) write `rag.embedding_*`, which the backend rejects on every write path (`api/settings.rs:916`, called at :1141, :1194, :1371). Confirmed in source.
    - Worse, the namespace save submits every dirty key, and the bulk path rejects the whole request before anything persists (`api/settings.rs:1353-1361`).
    - So a valid co-edited RAG change is lost too: `chunk_size`, `chunking_strategy` and `semantic_threshold`, all of which have live readers.
  - The reload endpoint that reports success with no effect (§G3).
  - Every namespace in G3's "no reader" list.
- **Misleading display:** the Providers page "default" banner shows `models[0]`, not `default_model` (`providers/ui/providers-page.tsx:368-371`).
- **Navigation oddities:**
  - `AdminWelcome` is stacked on the Providers page only.
  - Settings has 30 sections in 5 categories (`settings-page.tsx:29-35`, `settings-navigation.tsx`). Section state lives in `useState`, so sections cannot be linked.
  - The cockpit highlights "Runtime settings" though it is a dashboard.
- **Correct as is:** the A2UI tester stays dev-only (`nav-destinations.ts:153-173`, `pages/admin-page.tsx:26-30,59`).

### G6. A picker everywhere an LLM or embedding provider/model is chosen — PARTIAL

- **Two pickers exist:** `features/models/ui/model-selector.tsx:51` (Popover + Command, fed by graph `ConfiguredModel`) and a second `SettingModelPicker` (`settings/ui/settings-primitives.tsx:244`). They are not unified.
- **Pickers already used:** session model override, agent default/fallback, team binding model, skill model override, provider default model, memory embedding provider, `llm.embedding.backend`. The Models page "set default" is a picker too, but it is unreachable (§G5).
- **Free text (violates the goal):**
  - In `ai-settings-panels.tsx`: vision model override (:268/271), summarization model (:402/406), RAG embedding provider/model (:504/513, broken, see G5), KB default embedding provider/model (:548/558).
  - Memory embedding model (`memory-settings-panel.tsx:142`).
  - OCR model (`file-processing-settings-panels.tsx:139`; the backend schema says `select`).
  - Schema-driven `llm.default_provider` and `llm.embedding.model` (`x-control: text` at `src/uar/settings/manager.rs:2851,2864`). Both are operative, so the live embedding model is chosen by free text.
  - `llm.model` (`:2751`) is also free text, but it has no runtime reader. It is a G5 wire-or-remove item, not a picker gap.
  - `skill_evolution.trigger_model` (`manager.rs:3391`).
  - `resilience.fallback_models` (JSON, `manager.rs:3011`, not shown by the resilience panel).
  - Credentials provider ID is free text with a datalist (`credentials/ui/credentials-page.tsx:226`).

### G7. ALL settings are pickers/toggles; text only for prompts and content — NOT MET

- **Control counts by feature** (text inputs / textareas vs selects): settings 56/2 vs 19; teams 6/3 vs 3; skills 5/1 vs 0; knowledge 3/1 vs 0; memory 3/0 vs 0; providers 3/0 vs 0. Agents is the model: tabs and pickers, with text only for name, description and prompt.
- **Systemic cause:** `settings/ui/generic-schema-panel.tsx` renders every array as comma-separated text (:253-266), every object as a raw JSON textarea (:165-206, 268-270), plain strings as monospace text (:287-293), and numbers as bare number fields (:240). This one component drives 13 panels. The backend `x-control` hints exist but default to `text`.
- **Other worst cases:**
  - Retryable HTTP status list (`resilience-settings-panel.tsx:430`) plus 9 bare millisecond/count fields.
  - Agent allowed tools as comma-separated text (`governance-settings-panels.tsx:796`), a native checkbox (:583), and 4 bare budget fields.
  - Skill keywords and preferred tools as CSV (`skills-page.tsx:370,384`).
  - Skill import by typed absolute path (`skill-import-dialog.tsx:119`).
  - Memory page user/agent ID filters (`memory/ui/memory-page.tsx:198,204`).
  - Team launch raw JSON (`teams/ui/launch-sheet.tsx:155`).
  - Team binding typed references with a "use example value" button (`binding-reference-fields.tsx:26`).
  - Team workspace typed path (`workspace-control.tsx:26`).
  - Presentation template 18-row JSON textarea (`presentations/ui/presentation-editor.tsx:66`).
  - Allowed MIME types as CSV and OCR language as free text (`file-processing-settings-panels.tsx:78,215`).
  - Provider base URL (`providers-page.tsx:279`).
- **Primitives already exist and are unused in settings:** 55 files in `components/ui`, including select, combobox, slider, switch, toggle-group, radio-group and field.

### G8. Web workers where they help without side effects — NOT MET (none exist)

- No `Worker`, `SharedWorker`, comlink or `?worker` import exists.
- The service worker (`public/sw.js`, registered at `main.tsx:46`) uses a fixed cache name `uar-v1`. HTML is network-first and hashed `/assets/` are cache-first, so stale assets are unlikely. The real defect: the activate cleanup never deletes old hashed assets, so the cache grows without bound across deploys.
- **Candidates, ordered by benefit:**
  1. `PGliteWorker` with leader election. Moves WASM Postgres off the main thread and removes the multi-tab `idb://` hazard. The `live` extension and run-snapshot subscriptions must work through it.
  2. Model catalog normalize/filter (`models-page.tsx:115,461`, `model-selector.tsx:77`).
  3. Shiki and Mermaid parsing/highlighting (already lazy; DOM-bound parts stay on the main thread).
  4. `JSON.parse(JSON.stringify)` deep copies in streaming stores.
- **Constraint:** the PEM `graphStore` is a main-thread Zustand singleton, so snapshot serialization cannot move to a worker without a PEM change. Shrinking the snapshot (exclude `RuntimeAgUiEvent`, bound retention) is the safer lever.

### G9. S-tier, minimalist UX with selective discovery — NOT MET

- 16 components exceed 400 lines. The largest: `governance-settings-panels.tsx` 1023, `models-page.tsx` 893, `knowledge-page.tsx` 793, `enhanced-thread.tsx` 778, `agent-editor.tsx` 678, `ai-settings-panels.tsx` 598.
- **Progressive disclosure is nearly absent:** `AdvancedSection` is used twice, and every schema-driven panel shows every field at once.
- **Raw JSON shown to operators:** effective policy preview, tool schema/result, `json-schema-form`, team launch, presentation template.
- **Tokens vs usage:** `--border` is defined in `index.css:46,119,209` (transparent in the default themes), and `tokens.css:87` only maps `--color-border` to it. 327 `border` and 123 `border-border` classes remain.
  - These are **load-bearing for high contrast**: `index.css:166` sets a visible `--border` ("HC exception: visible lines are the accessibility feature"). Any cleanup must preserve HC borders.
  - Some pages use raw `var(--color-ember|fg-sub)`.
- The terminal-style label `font-mono text-xs uppercase tracking-widest` appears in 13 feature files.

## CROSS-TOOL PROGRESS
- NONE — no cross-tool activity recorded for this phase.

## SPEC GAP SUMMARY
- **No general spec for YAML↔DB settings precedence**, drift surfacing, hot-reload application, last-known-good, or startup protection (G3). Only the skills-scoped `skill-config-reconciliation` precedent exists, and the change records hold contradictory authority decisions (§0.5).
- **No spec for team-member and sub-agent model precedence** or for live team model change (G4; `multi-agent-orchestration` is silent).
- **`frontend-configuration-surfaces` lacks a "no free-text for enumerable values" requirement**, and drift/provenance display (G6/G7).
- **`frontend-local-first-persistence` has Purpose "TBD"**, and no worker or multi-tab requirement (G1/G8).
- **`provider-model-settings-certification` specifies DB-wins with no YAML propagation** for existing providers. That conflicts with G3's "stay in proper sync" and needs an operator decision.
- **Frontend/backend settings contract mismatch:** nested `meta.{source,is_drift}` vs flattened fields.
- **Implemented but unspecified:** sub-agent model inheritance; team `TEAM_ROUTE_PROFILE_MISMATCH` blocking.

## BUILD HEALTH
- **Build check: UNKNOWN.** Not run, by rule: verification is closed until the full phase implementation boundary (CLAUDE.md "Full-phase verification boundary"; VLBV B4). Last observed in this session (prior work, 2026-11-18): frontend typecheck and lint passed on PEM 5.0.1. Tests, build and app were not run then either.
- **Known violations:** 37 grandfathered per-row graph writes; 2 gate exception lists (`check-frontend-boundaries.mjs:28-73`); per-type store duplication the gate misses (§G1).
- **Test coverage: UNKNOWN.** Not measured. Playwright configs exist for settings routes and session config (`frontend/playwright.installed-*.config.ts`).

## CONSTRAINT CHECK
- **AGENTS.md / CLAUDE.md violations:**
  - Business records duplicated in Zustand (Provider, Model, threads, messages) violate "must not duplicate Provider, Model, AgentSession … or another graph-owned entity".
  - Per-row list publication violates "never publish a fetched entity list by looping over rows".
  - Both are pre-existing.
- **constraints.md:** N/A (absent).
- **PRODUCT.md:** its "does not redesign the application" statement conflicts with this phase's goals and must be superseded by operator decision.

## GOAL PROGRESS
1. PGlite + PEM client state — **PARTIAL**: restore done; facade leaks the raw store; main-thread PGlite, first-session stale-seed defect, duplicated stores, per-row ingestion, unbounded event snapshot with persistence starvation under streaming.
2. Impeccable CLI update/init + evaluate — **NOT MET**: versions disagree, PRODUCT.md contradicts the phase, no DESIGN.md.
3. DB-persisted settings in sync with YAML, never bricking — **PARTIAL**: persistence real; hot reload inert; many namespaces have no reader; drift invisible; no rollback; 3 boot-abort paths (including a delayed brick from YAML saved while running) plus 2 silent-degradation paths; the R3 YAML-authority decision was silently reverted (§0.5); apply-status metadata false and unused.
4. Realtime model change at all four levels — **PARTIAL**: agent and conversation apply next turn; the system level has 3 competing defaults and no reachable effective picker; teams pinned, and edits to a team's captured routes block it; sub-agents ignore conversation overrides; no conversation push; push needs local Surreal.
5. Remove nonsensical UI — **NOT MET**: 8 unreachable pages, 6 duplicate surfaces, a broken RAG embedding control that also discards valid co-edits, a reload endpoint with no effect, a misleading default banner.
6. Pickers for every model/provider choice — **PARTIAL**: 7 reachable picker sites; about 11 free-text sites, including the operative `llm.embedding.model` and `llm.default_provider`.
7. All settings non-typed — **NOT MET**: generic schema panel renders text/CSV/JSON for 13 panels.
8. Web workers where useful — **NOT MET**: none exist; PGliteWorker is the strongest candidate.
9. S-tier minimalist UX with disclosure — **NOT MET**: 30 settings sections with no deep links, 16 oversized components, near-zero progressive disclosure.

## ADOPTED-OUTCOME CONTRACT INPUTS (elicited; adoption source = operator goals in `goals.md`, 2026-11-19)

- **The one useful shipped user operation (proposed; operator to confirm):** an operator opens the console, changes the model for the system, an agent or a live conversation using pickers only, and the next reply in an open conversation uses the new model. All settings involved persist in the DB, survive a restart, and do not stop startup when YAML changes.
- **Observed baseline:** as in G1–G9. **UNKNOWN:**
  - whether the stale-seed defect reproduces in a fresh browser;
  - whether the page-load freeze is gone on 5.0.1;
  - actual main-thread cost of the 50 ms snapshot during streaming;
  - which "no reader" namespaces operators actually rely on.
- **Desired observable result:**
  - zero free-text inputs for enumerable values;
  - no unreachable or duplicate settings surfaces;
  - model changes visible in the next turn and pushed to other open clients;
  - YAML changes either applied or shown as drift with a one-click resolution;
  - a bad persisted value never prevents startup (fallback to last-known-good, with a visible warning).
- **Supported environments (proposed):** web console against embedded/local Surreal; the Tauri desktop shell (The Boss) on macOS arm64 per project rules. Remote Surreal and Postgres: **in scope, operator decision 2026-11-19**, including realtime push (absent there today).
- **Required safety checks (non-waivable):**
  - Capability inversion: settings mutations stay in the trusted host.
  - Governance-Off proof rules (`decisions.md:889`).
  - Persist-before-publish for defaults (`decisions.md:461-476`).
  - Masked-secret round-trip.
  - No credential handling in test scripts.
  - Migrations add defaults without overwriting operator values (`gotchas.md:778-786`).
- **Exclusions (proposed):** pglite-oxide desktop data layer (spike failed); A2UI/AG-UI protocol changes; new commercial claims; PEM package changes beyond what G1/G8 strictly require (operator to confirm).
- **Risk acceptance:** **UNKNOWN**, operator.
- **Decision owner:** the operator (Travis James).

## VALUE SCENARIO CANDIDATES (proposals only; not adopted)

| ID | Actor → outcome | Source | Why now | Likelihood |
|---|---|---|---|---|
| VS-1 | Operator switches a conversation's model by picker; the next turn uses it; a second tab updates | G4/G6 | Core goal | high |
| VS-2 | Operator changes an agent default; open conversations without an override switch next turn | G4 | Core goal | high |
| VS-3 | Operator changes the system default (which mechanism is an operator decision; see §G4) from a reachable picker; agents without explicit models follow | G4 | Core goal; no reachable effective control today | high |
| VS-4 | Operator changes a team member model, or edits a provider/model route a running team uses; the team either adopts it at the next task boundary or keeps running with an explicit "applies to new runs", and is never blocked (operator decision) | G3/G4 | Today edits to a team's captured routes block it | medium |
| VS-5 | YAML edited while running, valid or invalid → a valid change is applied or shown as drift; an invalid one is reported immediately and never bricks the next boot (last-known-good persisted) | G3 | Core goal; delayed-brick path confirmed | high |
| VS-6 | An invalid persisted embedding dimension, or one malformed provider row → server starts with the valid providers, flags the bad value, falls back | G3 | Confirmed boot-abort and silent-drop paths | medium (rare-critical) |
| VS-7 | Fresh browser profile, first session → chat persists messages (stale-seed defect) | G1 | Confirmed in source | certain on every new install |
| VS-8 | Two tabs open → no IndexedDB corruption; one PGlite leader | G1/G8 | Unspecified hazard | medium |
| VS-9 | Every settings section: no free-text field for an enumerable value (mechanical audit) | G6/G7 | Core goal | high |
| VS-10 | Streaming a long reply keeps input latency under budget, and graph state is persisted within a bounded delay (snapshot cost and starvation) | G1/G8 | Unknown baseline | UNKNOWN |

## OPEN QUESTIONS FOR ANALYZE/PLAN (operator decisions)

1. **RESOLVED 2026-11-19 (operator): running teams adopt model changes at the next task boundary**, covering both member changes and edits to captured routes. In-flight tasks finish on their starting model. Recorded in `.prometheus/decisions.md`. Planning must define behavior when the new route is unavailable. Original question: **Team model pinning:** should a running team adopt member model changes, or keep the current pinning (safety) and make "applies to the next run" explicit in the UI? Also: should edits to a route a running team uses (profile, endpoint, price) keep blocking it, or apply at the next task boundary?
2. **RESOLVED 2026-11-19 (operator): DB wins; the R3 revert was deliberate.** Recorded in `.prometheus/decisions.md`. Remaining planning work: disposition `make-config-authoritative-on-boot` as superseded, and bring the provider seed into line with the spec. Original question: **YAML vs DB authority:** R3 (YAML authoritative on every boot) was implemented in `c41da57e` and silently reverted in `7a6aa305` nine days after the DB-wins certification was archived.
   - Was that revert deliberate? If so, archive `make-config-authoritative-on-boot` as superseded.
   - Note that the current spec's rule (operator-edited rows win, YAML flows into config-owned rows, drift is visible) already meets the goal's wording. The provider seed is stricter than that rule today, so the planning default would be to bring providers into line with the spec, not to make a new authority decision.
3. **RESOLVED 2026-11-19 (operator): yes, realtime push must work with remote Surreal and Postgres in this phase.** Recorded in `.prometheus/decisions.md`. First Analyze task: establish whether the per-topic SSE path (`entities/sync.ts:32-52`) is served for remote stores. Original question: **Remote persistence:** must realtime push work with remote Surreal/Postgres in this phase?
4. **RESOLVED 2026-11-19 (operator): the registry default provider plus its `default_model` is the system level.** Recorded in `.prometheus/decisions.md`. `llm.model` leaves the UI; `run_policy.global.model` is retired from model selection or kept internal. **Sub-question RESOLVED 2026-11-19 (operator): no, the agent's explicit model wins.** The system default fills only unset levels; conversation and turn still override the agent; no force-to-system option this phase. Original question: **What is the system level?** Choose one of `run_policy.global.model`, the registry default provider/model, or `llm.model` (the others are wired to it or removed). Should it ever override an agent's explicit model?
5. **RESOLVED 2026-11-19 (operator): redesign authorized; PRODUCT.md may be rewritten; `docs/design/` (October) is the visual authority for the new DESIGN.md.** `docs/UI_DESIGN.md` is retired. Recorded in `.prometheus/decisions.md`. Original question: **Redesign scope:** may PRODUCT.md be rewritten to authorize a redesign, and is `docs/design/` (October) the visual authority for the new `DESIGN.md`?
6. **RESOLVED 2026-11-19 (operator): remove them from the UI.** YAML keys and boot-time YAML readers stay. Analyze confirms per namespace that no runtime reader exists before removing its control; a key found to have a reader moves to the wiring/picker work. Recorded in `.prometheus/decisions.md`.
7. **RESOLVED 2026-11-19 (operator): decide per change during planning.** Nothing is archived or edited in Assess. The plan records continue, supersede (named successor; old change marked, not deleted) or archive (with reason) for every overlapping change in §0.5, including R3, consistent with DB-wins. Recorded in `.prometheus/decisions.md`.
8. **RESOLVED 2026-11-19 (operator): created and activated.** Phase created (revision 3462) and activated (3463); assess stage entered and completed (3465). The rc5 producer path is paused, not cancelled. method-context now resolves the phase but is BLOCKED on "adopted contract unavailable" until Plan binds the contract. Recorded in `.prometheus/decisions.md`.

## ADVERSARIAL REVIEW RECORD

- **Dispatch:** the external REST judge was blocked by the session's auto-mode permission classifier (sending the artifact to an outside model gateway). Fallback used: a harness-native fresh-context `artifact-critic` subagent, which received only the artifact paths and the mandate. `isolation_mode: harness-native`, same model family: a weaker guarantee than cross-model, stated rather than hidden.
- **Round 1:** verdict BLOCK, 2 CRITICAL / 13 WARNING / 4 SUGGESTION. Both CRITICAL findings were confirmed in source by the producer and corrected:
  1. In-flight OpenSpec changes, including the contradictory R3 authority decision (now §0.5 and Open Question 2).
  2. The system-level model has no reachable effective control (now §G4).
- All 13 WARNINGs and all 4 SUGGESTIONs were applied: seed scope, debounce mechanics, unreachable count, provider-load silent drop, apply-mode metadata, team blocking on system edits, RAG co-edit loss, facade leak, the `llm.model` contradiction, HC borders, `SettingsType`, per-row site corrections, the prior config-lifecycle receipt, the reload endpoint, bound-agent pinning, status vocabulary, and service-worker cache growth.
- **Correction to the round-1 record:** the round-1 WARNING "providers-graph site not grandfathered" was applied without source verification and was false (`check-frontend-boundaries.mjs:55` lists it). The other round-1 WARNINGs were taken from the reviewer's cited evidence. The producer verified only the two CRITICAL findings and this item.
- **Round 2:** verdict BLOCK, 3 CRITICAL / 4 WARNING / 1 SUGGESTION. All were applied:
  - R3 was implemented in `c41da57e` and reverted in `7a6aa305` (verified with git).
  - Team blocking is scoped to captured routes (taken from the reviewer, not re-verified).
  - Invalid YAML aborts every boot, a delayed brick (verified in `main.rs:94-99`).
  - Provider seed is stricter than the spec; Model identity diverges across writers; the grandfather list correction (verified); 30 sections (verified); contract-type wording.
- Findings JSON: `review/assess/findings-round1.json`, `review/assess/findings-round2.json`.

## UNRESOLVED REVIEW FINDINGS

The two-round review cap is reached. The round-2 corrections above have **not** been re-vetted by a third reviewer. Analyze/Plan should treat these claims as reviewer-sourced and not independently verified:
- team blocking is limited to captured routes (`registry.rs:165-192`);
- the Model identity divergence (`models-api.ts:30-31`);
- the spec-vs-seed strictness reading.

ASSESSMENT COMPLETE
