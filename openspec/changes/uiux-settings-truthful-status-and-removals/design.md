## Context

Phase `uar-uiux-architecture-2026-11`, build items 10, 11 and 18; unresolved critical finding 3;
two review warnings (Providers banner, no-op reload endpoint). Binding decisions of 2026-11-19:
settings with no reader leave the UI and YAML keys stay (decision 6); the registry default provider
plus its `default_model` is the system level (decision 2); DB wins over YAML for operator-edited
values (decision 1); an agent's explicit model beats the system default (decision 9).

Method: source reading only. Nothing was built or run. "Read" below means this writer read the cited
lines. "Analysis-sourced" means the claim comes from `analysis.md` / `assessment.md` and was not
re-read here; it is marked UNVERIFIED where the design depends on it.

### Verified here

| Claim | Evidence (read) |
|---|---|
| `setting_apply_mode` returns `Live` for `governance`, `Restart` for `server`/`security`/`persistence`, `NextTurn` for everything else | `src/uar/settings/manager.rs:73-80` |
| `set_value` writes the effective map for every non-`Restart` key, before any reader has applied it | `manager.rs:666-671` |
| The effective map is seeded from the cache at boot, so after a restart saved == effective for every key, including keys whose reader is YAML | `manager.rs:327-332` |
| `application_status` is `effective` when masked saved == masked effective, else `restart_required` for `Restart` keys, else `pending` | `src/uar/api/settings.rs:575-582` |
| Response carries `saved`, `effective`, `revision`, `apply`, `application_status`, flattened `source` and `is_drift` | `settings.rs:586-620` |
| Mutation responses add an `activation` field only for embedding hot-swap and for `memory.*` and `rag.chunking_strategy/chunk_size/chunk_tokens/semantic_threshold` | `settings.rs:932-940, 1165, 1225, 1439` |
| `rag.embedding_provider` / `rag.embedding_model` writes are rejected, and one rejected key rejects the whole bulk request | `settings.rs:916-927, 1353-1361` |
| RAG chunking keys are read once at boot ("applies after restart") | `src/server.rs:1353-1368` |
| `memory.*` is read once at boot | `server.rs:1475-1508` |
| `prompt_caching.enabled` is read at request time | `server.rs:5799, 7796`; `uar/domain/policy.rs:710`; `api/discovery.rs:1008` |
| Resilience is resolved per turn from persisted rows | `src/uar/settings/persisted_resilience.rs:1-40`; `server.rs:3673`; `runtime/manager.rs:4653` |
| `harness.*` is read in `resolved_harness_config` | `runtime/manager.rs:1988-1999` (call frequency UNVERIFIED) |
| `llm.default_provider` is applied at boot only through the settings key | `settings/provider_sync.rs:15-27` |
| The live default swap persists then swaps the registry; it does not record any effective value | `src/uar/api/providers.rs:367-399`; `manager.rs:1248-1250` (`set_default_provider_id` calls `set_value`) |
| `llm.default_provider` is `x-control: text` and `llm.model` is `x-control: text` | `manager.rs:2851-2856, 2742-2750` |
| Frontend `SettingWithMeta` has only nested `meta`; the string `application_status` appears nowhere in `frontend/src` | `frontend/src/types/index.ts:320-341`; grep of `frontend/src` |
| 30 `NAV_ITEMS`; selection is `useState`; availability falls back to "all enabled" when type metadata is empty | `settings-navigation.tsx`; `settings-page.tsx:12-33` |
| Reload handler returns 200 and the config JSON schema after `config_manager.reload()` | `server.rs:3726-3743`; no frontend caller (grep of `frontend/src`) |
| `rejected_unsupported` guards the single PUT, the bulk PUT and the DELETE reset; `ApiError::Validation` answers 422 `{"error":"unsupported_setting","key","message"}` | `src/uar/api/settings.rs:404-440, 916-927, 1190-1195, 1353-1361` |
| The default route returns 404 for an unregistered provider before persisting | `src/uar/api/providers.rs:366-373` |
| A mask placeholder submitted for a missing row returns 404 | `settings.rs:1523-1526` |
| The drift list masks through `masked_response` | `settings.rs:1239-1250` |
| `stub-llm` is `[[bin]] required-features = ["server"]`, takes the fixture file as argv[1] or `STUB_LLM_FIXTURES` and the port from `STUB_LLM_PORT` (default 4600), serves `http://127.0.0.1:{port}/v1`, and handles no Authorization header | `Cargo.toml:128-130`; `src/bin/stub-llm.rs:1-130`; `tests/integration/live/stub_llm.rs` (grep: no `authorization`) |
| Provider credentials live in a separate credential store; the test route accepts a loopback provider with no key, so a 400 from it does not show key retention | `src/uar/api/providers.rs:446-488` (`CredentialScope::System`, `permits_anonymous_loopback`) |
| A sidecar write lock names `skill_evolution.enabled`; that suggests a possible reader that task 1.1 must rule out | `manager.rs:129, 612` |
| `SkillEvolutionConfig::default()`, env-chosen sandbox runner, boot-snapshot `config.acp`/`config.sycophancy` | `runtime/manager.rs:1436`; `sandbox/platform.rs:21,61`; `server.rs:2878,8717` |

## Goals / Non-Goals

**Goals**

- A setting the console shows is a setting something reads. A status the console shows is a status
  something confirmed.
- One place classifies how every setting is applied; the API, the mutation `activation` field, the
  schema markers and the console all derive from it.
- One system model control (the registry default), reachable and live.

**Non-Goals**

- Deleting YAML fields, boot readers or stored rows (decision 6 keeps them).
- Wiring a reader for any removed namespace.
- The routes behind boot status, drift resolution and reload (owned by
  `uiux-config-boot-safety-and-live-reload`). This change owns the console screens that consume them (D14).
- The system-default picker itself (owned by `uiux-system-model-live-default`).
- Schema-to-control dispatch and pickers (owned by `uiux-schema-controls-and-pickers`).
- Regrouping the surviving sections (owned by `uiux-design-context-and-settings-regroup`).

## Decisions

### D1. One classifier, four classes

Replace `setting_apply_mode`'s three-way namespace match with an explicit table. Each entry has a
key pattern (exact key or prefix), a class, and the reader that justifies it.

| Class (`apply`) | Meaning | `application_status` after save |
|---|---|---|
| `live` | A reader applies the value immediately | `effective` once the applier confirms |
| `next_turn` | A reader consults the DB per turn or per request | `effective` once saved (the reader re-reads the DB) |
| `restart` | Read from the DB once at boot | `restart_required` until a restart; `effective` afterward |
| `inert` | No DB reader. Boot reads YAML, or nothing reads it | `not_applied`, always, including after restart |

`inert` is new and additive: the `apply` enum gains a value and `application_status` gains
`not_applied`. The frontend reads neither field today, so no consumer breaks. External API
consumers are UNKNOWN.

Unclassified keys default to `restart`. Over-warning ("restart may be needed") costs an operator a
restart; a false `effective` costs them a silent no-op. A boundary scenario enumerates every seeded
key and requires an explicit table entry, so the default never carries a real key.

### D2. Only the applier records "effective"

`set_value` stops writing the effective map. The class decides:

- `next_turn` and `live` keys: `set_value` records effective after the write commits, as today,
  because the reader re-reads the DB.
- `restart` and `inert` keys: nothing records effective. After a restart the boot seeding sets
  effective from the DB (`manager.rs:327-332`), which is correct for `restart` keys and is
  overridden to `not_applied` by class for `inert` keys.
- The live default route (`providers.rs:367-399`) calls a new `record_effective(key, value)` after
  `registry.set_default` succeeds, so `llm.default_provider` shows `effective` after the live swap.
  Without this, D1 would mark the live swap as pending. The settings API no longer writes this key
  at all (D12), so there is no "bare settings write" status for it.
- Embedding hot-swap: `activate_embedding_if_touched` (`settings.rs:1148-1240`) records effective
  for the four `llm.embedding.*` keys only when the swap succeeds. When it is skipped (failover pair,
  `settings.rs:1198`, analysis-sourced) the status is `restart_required`.

### D3. Classifier table (proposed; execute task 1.1 closes the UNVERIFIED rows)

| Keys | Class | Reader | State |
|---|---|---|---|
| `governance.*` | `live` | `manager.rs:683-700` publication | read |
| `prompt_caching.enabled` | `next_turn` | `server.rs:5799,7796`; `policy.rs:710` | read |
| `resilience.*` | `next_turn` | `persisted_resilience.rs:18` | read |
| `harness.*` | `next_turn` | `runtime/manager.rs:1992-1998` | read (frequency UNVERIFIED) |
| `run_policy.global` | `next_turn` | `domain/policy.rs:693` | read |
| `llm.embedding.*` | `live` when hot-swap succeeds, else `restart` | `settings.rs:1148-1240`; boot read `server.rs:1241-1253` | read |
| `llm.default_provider` | `live`; written only by the registry route, rejected on the settings API (D12) | `provider_sync.rs:21` (boot read); route `providers.rs:367-399` | read |
| `rag.chunking_strategy`, `rag.chunk_size`, `rag.semantic_threshold` | `restart` | `server.rs:1357-1368` | read |
| `rag.chunk_tokens` | `restart` | listed at `settings.rs:937`; boot reader not seen | UNVERIFIED |
| `rag.embedding_provider`, `rag.embedding_model` | `inert` | descriptive; writes rejected (`settings.rs:916`) | read |
| `memory.*` | `restart` | `server.rs:1475-1508` | read |
| `native_tools.*` | `restart` | assessment G3 | UNVERIFIED |
| `intent_classifier.*`, `skill_config.*`, `agent_config.*` | `next_turn` | assessment G3 ("read live") | UNVERIFIED |
| `provider.*` rows (Provider Overrides) | needs audit: does a settings write reach the live registry? | `manager.rs:975,1050` seed; registry path not read | UNVERIFIED, no default accepted |
| `llm.*` other than `model`, `default_provider`, `embedding.*` (`api_key`, `base_url`, `protocol`, `timeout_secs`, `cache`, `budget`, ...) | `inert` if boot YAML only | `server.rs:493` snapshot (`config.llm.clone()`) | UNVERIFIED per key; any key with a DB reader moves to `next_turn` |
| `llm.model` | `inert` | YAML boot only | analysis-sourced |
| `server.*`, `security.*`, `persistence.*`, `llm_failover.*`, `file_processing.*`, `sycophancy.*`, `acp.*`, `kreuzberg.*`, `unstructured.*`, `mistral_ocr.*` | `inert` | YAML boot (`analysis.md` classification table) | analysis-sourced; `acp`, `sycophancy`, `persistence` re-read here |
| `skill_evolution.*`, `context_management.*`, `sandbox.*`, `vision.*` | `inert` | not read at all | `skill_evolution` and `sandbox` re-read here |
| `models.*`, `context_strategy.*`, `knowledge_bases.*`, `user_settings`-type rows | needs audit | none read | UNVERIFIED, no default accepted |

The `llm.*` row matters for the UI. If `llm.api_key`, `base_url`, `protocol`, `timeout_secs` and
the others are boot-YAML only, the `llm` panel keeps only `embedding.*`, `cache`, `budget` and
whatever else audits as read. The decision text says "any key found to have a reader moves to
picker/wiring"; the inverse also holds: a key found without a reader leaves the UI.

### D4. The schema marker hides controls from data

Add a field-level extension `x-uar-ui-hidden: true` (plus `x-uar-ui-reason`: `"yaml-only"`,
`"not-read"` or `"managed-by-registry"`) to the settings-type schemas. Apply it to every property of
the 14 removed namespaces, to `llm.model` (`yaml-only`), to `llm.default_provider`
(`managed-by-registry`), and to `rag.embedding_*` (`not-read`) and to every `llm.*` key the audit
classes `inert`. The generic schema panel skips hidden properties. Hand-written panels for the
removed namespaces are deleted. Navigation also drops the 14 entries explicitly, so the removal
holds even when type metadata fails to load (`settings-page.tsx` treats empty metadata as
"everything available").

Hidden does not mean deleted. The types stay registered; `GET /api/uar/settings/{ns}` still works;
a PUT still persists and reports `not_applied`. The schema upsert at boot (`stats.types_upserted`,
`manager.rs:228,338`) carries the marker to existing databases; that propagation is UNVERIFIED for
databases whose type row already exists and needs a boot-time check in task 1.4.

### D5. Console shows status from a typed domain hook

New domain `platform/entities/settings-apply` exporting `useSettingApplyStatus(key)` (returns
`{ apply, status, saved, effective }` as primitives) and `useNamespaceRestartPending(namespace)`
(returns the list of keys pending restart). Components never read the `Setting` entity slice
directly. Subscription boundary: one indicator component per row subscribes to one key's status;
the namespace banner subscribes to one derived primitive (the count of restart-pending keys), so a
keystroke in one control does not rerender the other rows.

Ingestion: `settings-store.load` currently upserts each row in a loop
(`frontend/src/features/settings/model/settings-store.ts`, grandfathered per-row writes). This
change does not touch that; it reads the fields already spread onto each `Setting` entity
(`...(setting as unknown as Record<string, unknown>)`). Moving settings load to one atomic ingestion
is owned by the graph-ownership changes; the interface this change needs is "the `Setting` entity
carries `apply` and `application_status`".

`SettingWithMeta` is rewritten to the flattened shape the API sends (`saved`, `effective`,
`revision`, `apply`, `application_status`, `source`, `is_drift`). The nested `meta` type is removed;
the only readers are test fixtures (`ai-settings-panels.test.tsx:92-128`), which move with it.

Display rules: a row whose status is `restart_required` shows a text label "Saved. Takes effect
after restart." (never colour alone). A namespace save summary lists each key by status. `pending`
shows "Saved, applying". `effective` shows nothing extra: the quiet state is the normal state.
`not_applied` never appears for a retained control; it exists for API clients and for keys
accidentally left visible, where it renders "Saved, but nothing reads this value".

### D6. One editing surface per record type

The assessment lists six duplicate settings surfaces. Reading the panels shows the list overstates
it. Decision per surface:

| Settings surface | Verdict | Action | Precondition |
|---|---|---|---|
| `agent_config` ("Agent Configuration": one entry per agent, `governance-settings-panels.tsx:687-760`) | duplicate of the Agents page | Replace the panel with a link row to `/admin/agents` | Parity audit: every control in the panel exists in the agent editor. UNVERIFIED. If a control is missing, add it to the agent editor first. |
| `skill_config` ("Skill Configuration": one entry per built-in skill, `governance-settings-panels.tsx:926-956`) | duplicate of the Skills page | Replace with a link row to `/admin/skills` | Same parity audit. UNVERIFIED. |
| `skill_evolution`, `llm_failover`, `file_processing`, `kreuzberg`, `unstructured`, `mistral_ocr` | no reader | Removed (D4) | none |
| `provider` ("Provider Overrides") vs Providers, Credentials | The editor is certified by six requirements in `frontend-configuration-surfaces` (bounded default model, accessible context, async outcomes, draft protection, narrow widths, masks). | **Keep.** The regroup change moves it as the Connection editor inside the Models & providers area. Not deleted here. | n/a |
| `models` ("Model Files": tokenizer and embedding model paths) | configuration, not model records | Keep; regroup places it | Reader audit (D3 row) |
| `memory`, `knowledge_bases` | configuration of the memory and knowledge systems, not their content | Keep; regroup places them under Knowledge & memory | none |
| `presentation_policy` | instance-wide assignment ceiling, not template records | Keep; regroup places it | UNVERIFIED: panel source not read |
| Embedding choice in three places | `llm.embedding.*` is the only operative selector (`manager.rs:2952-2958` comment; `settings.rs:916`) | `rag.embedding_*` fields removed (D7); `knowledge_bases.default.embedding_*` stays only if its reader audits as read, else hidden; `memory.embedding_*` stays (separate memory store, read at `server.rs:1483-1488`) | KB default reader UNVERIFIED |

### D7. RAG co-edits stop being lost

Remove the "Embedding" card (provider and model inputs) from `RagPanel`
(`ai-settings-panels.tsx:504-518`). The backend rejection stays as is: it is the explicit rejection
contract for descriptive keys. With the fields gone, the dirty set can no longer contain a rejected
key, so a bulk save of `chunk_size`, `chunking_strategy` and `semantic_threshold` persists together
and the response reports `restart_required` for each (`rag.*` is `restart` class).

Considered and rejected: change the bulk path to persist the valid keys and report the rejected one.
It would change the documented "rejects the whole request before anything persists" contract for
every namespace, and the frontend would still show a control that cannot save.

### D8. Unreachable pages: relink, do not remove

All eight pages have a rendering component. The default is to relink each from its natural parent,
plus a palette entry. Removal is the alternative for any page whose boundary acceptance fails
because its backend is absent.

| Page | Relink target | Notes |
|---|---|---|
| `models` | Providers: secondary tab "Models" | The "set default" control on this page is the live system default control; the system-default picker change places the final control. |
| `credentials` | Providers: secondary tab "Credentials" | Provider ID is a free-text field with a datalist (`credentials-page.tsx:226`); the picker change owns that. |
| `mcp-health` | MCP & tools: tab "Health" | Already in `matchPaths` for highlighting. |
| `compiler` | Skills: tab "Compiler" | Already in `matchPaths`. |
| `memory` | Knowledge: tab "Memory" | Nav copy for Knowledge already says "memory". |
| `cost` | Runtime settings area: "Cost" | The regroup change moves it under Governance (budget) per `docs/design/governance.html`. |
| `auth` | Runtime settings area: "Access keys" | Regroup places it under Governance (access). |
| `protocols` | Runtime settings area: "Protocols" | UNVERIFIED target; the prototypes have no protocols page. Operator confirms or removes. |

URLs stay `/admin/<section>` here. The regroup change owns the new route table and the redirects.

### D9. Providers banner reads the configured default model

`ProviderDetail`'s default banner reads the system default model through the existing
`useSystemDefaultModelStatus()` hook (`features/providers/model/use-provider-default.ts`), which
already projects `ProviderMeta.default_model`. States: available (prints `provider/model`),
unavailable (default provider has no default model: warning text, not `models[0]`), loading, error.
The banner subscribes to the status primitives, not the model list. The `useProviderModels` call
remains for the model table; the banner no longer needs `models`.

### D10. Reload: the report is the config change's; the console screen is this change's

An earlier draft of this change added an `x-uar-reload` object to `POST /.well-known/uar-config/reload`.
It is withdrawn. `uiux-config-boot-safety-and-live-reload` (design D7) makes that route perform the
reconcile and adds `POST /api/uar/settings/reload` returning
`{status, reloaded_at, applied, retained_with_drift, restart_required, skipped}`. Two reports for one
action would disagree. This change consumes that route (D14) and adds one rule to the shared
contract:

- A YAML edit of a key that is read from YAML at boot (the `inert` DB class, for example
  `server.shutdown_timeout_secs`) is reported `restart_required`: the new value takes effect at the
  next start. A DB edit of the same key is `not_applied`, because no start will read it. The config
  change's reconcile must classify the YAML case from the table in D3, not from the retired
  three-way `setting_apply_mode`.

### D11. Order against sibling changes

- `uiux-system-model-live-default` ships the system-default picker. This change hides the text
  field; the Providers page "Set as default" already calls the live route
  (`features/providers/api/providers-api.ts:71`), so no window exists without a system control.
- `uiux-schema-controls-and-pickers` also edits `generic-schema-panel.tsx`. One writer per file
  applies; the lead sequences (see proposal).
- `uiux-config-boot-safety-and-live-reload` owns the boot-status, drift, resolve and reload routes and
  the reload consumers. This change owns the console screens that consume them (D14), depends on the
  response shapes in that change's design D6-D8, and relies on the flattened `SettingWithMeta` here.
  Inert rows are excluded from the drift screen (their authority is YAML, so "DB differs from YAML"
  has no meaning).
- `uiux-design-context-and-settings-regroup` follows this change.

### D12. Only the live route writes `llm.default_provider`

The realtime change refreshes every replica's registry default from the `llm.default_provider` row on
each change event, and every write returns through the change feed (analysis, delta 1). If the
settings API could write the row, a plain `PUT` would become a live system-model change on every
replica, a second write path that decision 2 rules out. So: add `llm.default_provider` to
`rejected_unsupported` (`settings.rs:916-927`). That function already guards the single PUT, the bulk
PUT and the DELETE reset, and it answers 422 `unsupported_setting` with a message. The message names
`POST /api/uar/providers/{id}/default`. The live route and boot seeding write through
`SettingsManager` directly and are unaffected. GET still returns the row.

Cost: a bulk `llm` namespace save that includes `default_provider` is rejected whole, by the existing
explicit-rejection contract. The console never submits it, because the field is hidden (D4).
Alternative: a separate key written only by the route. Rejected because the row already exists, the
boot reader and the drift rules reference it by name, and a second key would duplicate state.

### D13. Secrets and the masked round trip

The mask behavior exists (`mask_sensitive`, `preserve_masked_sensitive_value`,
`settings.rs:509-535, 1497-1534`; two certified requirements). What is missing is evidence that it
holds end to end, and a rule about logs. Two limits in the available tooling shape the scenario:

- A length-preserving mask cannot show retention. A stored literal mask would read back as the same
  number of mask characters as the secret it replaced. Retention has to be observed where the secret
  is used: the `Authorization` header the provider request carries.
- `stub-llm` records no headers, and the provider test route accepts a loopback provider with no key,
  so neither shows retention today. Task 1.8 adds header recording to `stub-llm` (test support, not
  production code) and serves it at `GET /__recorded-auth`.

Task 1.7 audits the settings write path for any log or trace line that includes a setting value, and
removes it. The scenario then greps the instance output for the sentinels. UNVERIFIED: whether any
such line exists today.

### D14. Console screens for boot status, drift and reload

One panel, "Configuration status", hosts the three screens, plus a persistent banner from the shell.

| Screen | Source | Behavior |
|---|---|---|
| Boot banner | `GET /api/uar/settings/boot-status` (config design D8) | Shown on every console page when `config.mode != "primary"`. Names the rejected file and the first 12 hex characters of `failed_source_digest`. No values, no parser text. |
| Boot detail | same | Fallback class, last-known-good saved time, embedding values ignored (key, reason), provider rows skipped (key, reason). |
| Drift list | `GET /api/uar/settings/drift` | Rows for keys whose class is not `inert`: key, saved value, configured value (masked if secret). Provider drift: field names. |
| Resolve | `POST /api/uar/settings/drift/{key}/resolve` `{"resolution":"use_config"}` | One button per resolvable row. 404 shows "no longer drifting". No button for `llm.default_provider` or `governance.*` (the API answers 409). |
| Reload | `POST /api/uar/settings/reload` | Button; shows status and the four lists with keys and reasons. 422 shows rejected with the class. |

Interface gap, stated rather than papered over: the boot status shape in the config change carries
digests and a YAML key path but not the rejected file's name. The operator-visible requirement "a
warning naming the rejected file" needs a `failed_source_path` field on `config.fallback`. This change
asks for it; until the config change adds it, the banner can show the digest only and SCN-SET-010 cannot
pass.

Graph and hooks (project rule): boot status and drift are server-confirmed state, so they enter the
entity graph through one atomic ingestion action each (no per-row loops) and reach components through
typed hooks exported by `@/platform/entities` (`useBootStatus`, `useConfigDrift`). The reload result is
workflow output held in local component state, not a business entity. Each row subscribes at its own
boundary.

Admin key: the existing `settingsAdminHeaders()` attaches `X-UAR-Admin-Key` only when the build sets
`VITE_UAR_ADMIN_KEY`. The scratch configuration turns the guard off (`fixtures/scratch-config.yaml`), so
no scenario supplies a key.

Placement: a settings section (`config_status`) now, a `governance/config-status` section after the
regroup change. The shell banner file is created here (`app/shell/boot-status-banner.tsx`); the regroup
change mounts it.

## Risks / Trade-offs

- **Over-warning.** Defaulting unknown keys to `restart` can show "restart required" on a key that
  applies live. The table audit (task 1.1) is the mitigation; the boundary scenario that enumerates
  every seeded key is the check.
- **Hidden fields hide a key someone relies on.** An operator who edits `sandbox.*` in the console
  today gets no effect, which is why it is removed. The loss is real for the keys that are honored
  from YAML: they are configurable only in YAML. That is the decision's stated cost.
- **Schema marker is an extension key.** Older consoles ignore it and keep showing hidden fields
  (status then says `not_applied`, which is accurate).
- **Parity audit may fail.** If the Agents or Skills pages lack a control the Settings panel has, the
  Settings panel cannot be replaced yet. The task order puts the audit first.
- **The uncomfortable part.** After this change the console tells operators that most of what
  `server`, `security` and `persistence` used to offer was never working. Some operators will have
  edited those rows believing they applied. Their stored values stay in the DB, differ from YAML,
  and are ignored. The console should say so once, in the release note, not silently.

## Frozen oracle fixtures

Written before implementation. They are the expected-value sources for the scenarios below; no
implementer edits them, and no scenario takes an expected value from a classifier table, route table,
audit file or register written during the phase (rule R1). Paths are relative to this change.

| File | Used by | Content |
|---|---|---|
| `fixtures/removed-settings.json` | SCN-SET-002, 003 | The operator decision's 15-item list, the 14 navigation labels, one probe key per namespace (with seed line), the removed fields, the expected `inert` / `not_applied` result, and the expected 422 for `llm.default_provider` |
| `fixtures/retained-settings.json` | SCN-SET-001, 002 | The 16 retained navigation labels, the restart probe (`rag.chunk_size`), live probes, invalid-value controls, the chunk-count oracle |
| `fixtures/ingest-document.txt` | SCN-SET-001 | A 6000-character document uploaded unmodified |
| `fixtures/admin-pages.json` | SCN-SET-005 | The eight pages, their headings and data requests, and the unknown-section control |
| `fixtures/secret-roundtrip.json` | SCN-SET-009 | Sentinel secrets and lengths, captured endpoints, retention and replacement oracles, drift probe, the 404 control |
| `fixtures/config-screens.json` | SCN-SET-007, 010 | The broken YAML, drift values, reload edits and expected reports |
| `fixtures/scratch-config.yaml` | all | Scratch YAML with `security.settings_mutation_auth_required: false` so no scenario supplies a credential |

The classifier table itself is not an oracle. Its entries are checked through behavior: SCN-SET-001
observes a real chunk count before and after a restart, and the live probes observe the next request.

## Acceptance

Common prerequisites for every table below: an isolated UAR instance from the completed production
boundary on a lead-assigned port, a scratch data directory, behind the existing external identity
proxy; `fixtures/scratch-config.yaml` as its YAML; the console bundle from the same compile; no
credentials, tokens or login steps in any scenario (operator rule 2026-10-26); the local stub model
server `stub-llm` for every model route (rule R4). Prerequisites for the lead to turn into commands
with receipts: the production server binary, the `stub-llm` binary (`[[bin]] stub-llm`, feature
`server`, `Cargo.toml:128-130`, after task 1.8), and the console bundle. No scenario needs a baseline
compile of its own; the runner-integrity gate (lead-owned) builds its pre-phase baseline under
`~/.claude/worktrees/` (project rule, rule R5). `stub-llm` invocation:
`STUB_LLM_PORT=<port> stub-llm <fixtures.json>`, serving `http://127.0.0.1:<port>/v1`. Gate commands
are proposals; the lead pins them. Evidence is pending for every row.

### B1. A boot-only setting is reported as restart required, then takes effect after restart

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-settings-truthful-status-and-removals#5.1`; SCN-SET-001 |
| Production entry point | Console Settings, RAG & Chunking: edit Chunk Size to 200, Save; then restart the instance |
| Real collaborators | UAR server with embedded SurrealDB in a scratch dir; built console; Chromium via Playwright; the real ingestion worker; `fixtures/ingest-document.txt` |
| Boundary exercised | `PUT /api/uar/settings/rag`, `GET /api/uar/settings/rag`; browser UI; process restart; document upload and chunk listing |
| Observable result | Before restart the row shows "Saved. Takes effect after restart." and the API row has `application_status: "restart_required"`; the uploaded fixture yields at most 8 chunks. After restart the row is `effective` and a fresh upload yields at least 25 chunks (bounds in `fixtures/retained-settings.json`) |
| Negative control | `rag.chunk_size` = 10 (below the schema minimum of 64) is rejected with a non-2xx response and the row is unchanged. Live probe: `prompt_caching.enabled` edited in the same session reports `effective` at once and the next prompt-caching read reflects it |
| Isolation | Scratch data dir, lead-assigned port, browser profile per sample; instance stopped and directory deleted by the runner; the installed-runtime port 1906 is not used |
| Prerequisites | Production binary; `fixtures/*`; embedding backend usable offline for ingestion (UNVERIFIED which; the runner author reads the existing `knowledge-rag-real` setup) |
| Final local gate | Proposed: `pnpm -C frontend exec playwright test -c playwright.installed-settings-truth.config.ts -g SCN-SET-001` (new config, execute stage) |
| Evidence | Pending |
| Limitations | Embedded Surreal only; one process; chunk bounds depend on the chunker's recursive strategy |

### B2. Settings with no reader are absent from the console and report `not_applied`

| Field | Declaration |
|---|---|
| Canonical identity | `...#5.2`; SCN-SET-002 |
| Production entry point | Console Settings navigation and the `llm` panel; direct API client for the probes |
| Real collaborators | Same instance |
| Boundary exercised | Rendered navigation text; `GET /api/uar/settings/types`; `GET` and `PUT /api/uar/settings/{key}` |
| Observable result | None of the 14 labels in `fixtures/removed-settings.json` appears in navigation; all 16 labels in `fixtures/retained-settings.json` appear; the `llm` panel shows none of the four removed field labels; for each of the 14 probe keys the same-value `PUT` returns `apply: "inert"` and `application_status: "not_applied"`, before and after a restart; the sandbox runner chosen at boot is unchanged by the row (`yamlStillHonored`) |
| Negative control | `PUT llm.default_provider` returns 422 with the body in the fixture. `PUT resilience.retry_max_attempts` = `"abc"` is rejected non-2xx; `PUT` of its valid value reports `effective`. With `GET /types` blocked, the 14 labels are still absent |
| Isolation | As B1 |
| Prerequisites | Production binary |
| Final local gate | Proposed: `... -g SCN-SET-002` |
| Evidence | Pending |
| Limitations | Does not prove each removed key's YAML reader still works; that code is unchanged |

### B3. The system default model has one control and it is live

| Field | Declaration |
|---|---|
| Canonical identity | `...#5.3`; SCN-SET-003 |
| Production entry point | Console Providers, "Set as default" on the second of two providers; direct `PUT /api/uar/settings/llm.default_provider` |
| Real collaborators | Instance with two providers whose base URLs point at two `stub-llm` listeners; the real registry |
| Boundary exercised | `POST /api/uar/providers/{id}/default`; `GET /api/uar/providers`; settings API |
| Observable result | After "Set as default" the provider list shows the chosen provider as default at once, `GET llm.default_provider` reports `effective`, and the `llm` panel has no field for it |
| Negative control | Direct `PUT llm.default_provider` returns 422 naming the default route, and the registry default and the row are unchanged. `POST .../providers/nosuch/default` returns 404 and nothing changes |
| Isolation | As B1; two `stub-llm` ports |
| Prerequisites | `stub-llm` binary; fixture file for the stub (no inference is requested) |
| Final local gate | Proposed: `... -g SCN-SET-003` |
| Evidence | Pending |
| Limitations | The final picker UI belongs to `uiux-system-model-live-default` |

### B4. The Providers default banner names the configured default model

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.1`; SCN-SET-004 |
| Production entry point | Console Providers, detail of the default provider |
| Real collaborators | Instance with a stub-backed provider offering models `m-a`, `m-b`, default model `m-b` |
| Boundary exercised | Rendered banner; `GET /api/uar/providers` |
| Observable result | Banner text is `<provider>/m-b`, not `<provider>/m-a` |
| Negative control | The same provider with `default_model` cleared: the banner shows the unavailable warning and prints neither `m-a` nor `m-b` |
| Isolation | As B1 |
| Prerequisites | Stub provider with two models |
| Final local gate | Proposed: `... -g SCN-SET-004` |
| Evidence | Pending |
| Limitations | One provider shape |

### B5. Every stable admin page is reachable from the console

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.2`, `#4.3`; SCN-SET-005 |
| Production entry point | Console from `/threads`: clicks and command-palette search only |
| Real collaborators | Console bundle, instance |
| Boundary exercised | Browser navigation; the page data requests |
| Observable result | For each of the eight entries in `fixtures/admin-pages.json`: the heading text is visible and the listed data request returns 2xx, or the page shows its own explicit empty or disabled state |
| Negative control | `/admin/nonexistent` renders the cockpit (`data-testid="admin-section-runtime"`) |
| Isolation | As B1 |
| Prerequisites | None |
| Final local gate | Proposed: `... -g SCN-SET-005` |
| Evidence | Pending |
| Limitations | A page whose backend is absent reports that state; the scenario records it |

### B6. RAG co-edits persist

| Field | Declaration |
|---|---|
| Canonical identity | `...#3.4`; SCN-SET-006 |
| Production entry point | Console RAG & Chunking, Save with Chunk Size, Chunking Strategy and Semantic Threshold dirty |
| Real collaborators | As B1 |
| Boundary exercised | Bulk `PUT /api/uar/settings/rag`; `GET` afterward |
| Observable result | The three keys read back as entered; the panel has no embedding provider or model input |
| Negative control | A direct bulk `PUT` that includes `rag.embedding_model` and a valid `rag.chunk_size` is rejected (422) and neither key changes |
| Isolation | As B1 |
| Prerequisites | None |
| Final local gate | Proposed: `... -g SCN-SET-006` |
| Evidence | Pending |
| Limitations | None beyond B1 |

### B7. The console shows what a reload did

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.7`; SCN-SET-007 |
| Production entry point | Console Configuration status, "Reload configuration", after editing the scratch YAML per `fixtures/config-screens.json` |
| Real collaborators | Instance and its YAML file; the config change's `POST /api/uar/settings/reload` |
| Boundary exercised | HTTP and file edit; rendered report |
| Observable result | `resilience.requests_per_second` appears under applied and `server.shutdown_timeout_secs` under restart-required in the panel; the next resilience read shows the new value; the server's listener setting is unchanged |
| Negative control | Replace the YAML with `brokenYaml`, press Reload: the panel shows the rejected state with a failure class, no success text, and `GET resilience.requests_per_second` is unchanged |
| Isolation | Scratch YAML only |
| Prerequisites | The config change's reload route implemented |
| Final local gate | Proposed: `... -g SCN-SET-007` |
| Evidence | Pending |
| Limitations | `applied` coverage depends on the config change's consumers |

### B8. One editing surface for agent and skill records

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.4`; SCN-SET-008 (not mandatory) |
| Production entry point | Console Settings, Agents and Skills entries |
| Real collaborators | Instance with one agent and one built-in skill |
| Boundary exercised | Rendered link rows; the Agents and Skills APIs |
| Observable result | Each Settings entry contains a link and no input; following it reaches the owning page where the former control saves |
| Negative control | On the owning page, saving an agent with an empty name is rejected with a visible validation error and nothing is persisted |
| Isolation | As B1 |
| Prerequisites | Parity audit complete (task 4.4) |
| Final local gate | Proposed: `... -g SCN-SET-008` |
| Evidence | Pending |
| Limitations | Depends on the audit result |

### B9. A secret survives a masked round trip and never leaks

| Field | Declaration |
|---|---|
| Canonical identity | `...#5.9`; SCN-SET-009 |
| Production entry point | Console Providers: configure `openai` with the sentinel key and the stub base URL; edit the default model with the key field left as the mask; replace the key |
| Real collaborators | UAR instance, `stub-llm` recording Authorization headers (task 1.8), console, Chromium |
| Boundary exercised | The endpoint list in `fixtures/secret-roundtrip.json`; instance log capture; the provider request received by `stub-llm` |
| Observable result | No captured response body or log line contains the sentinel; the read-back mask has 29 mask characters; after the unchanged-mask save the stub's recorded header is `Bearer sk-stub-SENTINEL-7f3a9c1e5b2d`; after replacement it is the replacement, with a 33-character mask; the drift probe holds only masks |
| Negative control | `PUT provider.nosuch` with a mask placeholder returns 404 and creates nothing |
| Isolation | As B1; secrets are fixtures for a local stub, not UAR credentials |
| Prerequisites | Task 1.8 (header recording) and task 1.7 (log audit) done |
| Final local gate | Proposed: `... -g SCN-SET-009` |
| Evidence | Pending |
| Limitations | Proves absence in the captured endpoints and logs, not in every possible sink; at-rest encryption is another change |

### B10. The console shows the boot fallback, drift and its one-click resolution

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.5`, `#4.6`; SCN-SET-010 |
| Production entry point | Console after (a) a start from a rejected YAML and (b) a start where YAML differs from an operator edit, then the "Use YAML value" action |
| Real collaborators | Instance, scratch YAML, the config change's boot-status, drift and resolve routes, console, Chromium |
| Boundary exercised | Process start; the three routes; rendered banner and panel |
| Observable result | (a) A banner on every page names `config.yaml` and the first 12 characters of the sha256 of the broken bytes, which the runner computes itself; the panel shows `source_parse` and the last-known-good time. (b) The drift list shows `resilience.requests_per_second` with 7 and 5; after the action the API value is 5 and the entry is gone |
| Negative control | `POST .../drift/llm.default_provider/resolve` returns 409 and the console shows no action for it; resolving the already resolved key returns 404 and the console shows "no longer drifting", not success; the banner contains no text from the broken file |
| Isolation | As B1; the config file lives in the scratch dir |
| Prerequisites | The config change's routes implemented, including a `failed_source_path` field on boot-status (requested; see D14) |
| Final local gate | Proposed: `... -g SCN-SET-010` |
| Evidence | Pending |
| Limitations | The fallback behavior itself is certified by the config change's scenarios |

## Scenario matrix rows

All rows: substituted boundaries none; restoration is to stop the scratch instance(s) and `stub-llm`,
delete the scratch directory and config, release the ports. The runner reads expected values only from
`fixtures/`. Rows are split in two tables for width.

| Field | SCN-SET-001 | SCN-SET-002 | SCN-SET-003 | SCN-SET-004 | SCN-SET-005 |
|---|---|---|---|---|---|
| Name | Boot-only edit reports restart required | Unread settings leave the console | System default has one live control | Providers banner names the default model | Stable admin pages are reachable |
| Actor | Console operator | Console operator; API client | Console operator | Console operator | Console operator |
| Useful outcome | Knows a saved RAG value is not yet active and gets it after restart | Is never offered a control that does nothing | Changing the system default takes effect now and has no second, boot-only path | Sees which model is the default | No working page is stranded |
| Source requirement | `openspec:settings-apply-status#A saved value is reported effective only when a reader applied it` | `openspec:frontend-configuration-surfaces#Settings that no code reads are absent from the console` | `openspec:settings-apply-status#The default provider is changed only through the live route` | `openspec:remaining-admin-surface-certification#The Providers default banner shows the configured default model` | `openspec:remaining-admin-surface-certification#Every stable admin page is reachable from the console` |
| Reason now | Decision 6, item 10 | Decision 6 | Critical 3, decision 2, realtime C2 | Review warning | G5 |
| Likelihood | High | Certain | Moderate | Certain when default differs from first model; frequency UNKNOWN | Certain |
| Mandatory | true | true | true | true | true |
| Nondeterministic | true | false | true | false | true |
| Stochastic | samples 3; grader: all assertions in the Observable result hold in the sample; stoppingRule: fixed N=3, all must pass | n/a | samples 3; grader: provider list and row read show the new default within one request after the click; stoppingRule: fixed N=3, all must pass | n/a | samples 3; grader: every page heading visible and its data request 2xx or explicit empty state; stoppingRule: fixed N=3, all must pass |
| Production entry | Settings, RAG & Chunking, Save; restart | Settings navigation; `PUT` probes | Providers, Set as default; `PUT llm.default_provider` | Providers detail | Clicks and palette from `/threads` |
| Real collaborators | UAR, embedded Surreal, console, Chromium, ingestion worker | UAR, console, Chromium | UAR, registry, two `stub-llm` listeners | UAR, registry, `stub-llm` | Console, UAR |
| Oracle: observation | Row label; API status; chunk counts before and after restart | Navigation text; probe `PUT` responses before and after restart | Provider list default; row status; 422 body | Banner text | Heading text; data request status |
| Oracle: truth source | `fixtures/retained-settings.json` (restart probe, chunk oracle) and `fixtures/ingest-document.txt` | `fixtures/removed-settings.json` and `fixtures/retained-settings.json` | `fixtures/removed-settings.json` (`defaultProviderWrite`); the clicked provider id | The stub provider's configured `default_model` | `fixtures/admin-pages.json` |
| Oracle: comparator | Exact status strings; counts within the fixture bounds | Absence and presence of listed labels; exact `inert` / `not_applied` | Exact id equality; exact status and body fields | String equality with `m-b`; inequality with `m-a` | Heading equality; 2xx |
| Negative control: input | `rag.chunk_size` = 10; live probe `prompt_caching.enabled` | `llm.default_provider` write; `"abc"` for an integer key; blocked `/types` | Settings `PUT` of the other provider; default route for `nosuch` | Provider with no `default_model` | `/admin/nonexistent` |
| Expected rejection | Non-2xx, row unchanged; live key `effective` at once | 422 with fixture body; non-2xx; removed labels still absent | 422, registry and row unchanged; 404, nothing changed | Unavailable warning, no model id printed | Cockpit test id rendered |
| Fixtures | `fixtures/*`, scratch dir | `fixtures/*`, scratch dir | Scratch dir, two stub listeners | Scratch dir, stub with two models | `fixtures/admin-pages.json` |
| Evidence required | Trace, API bodies, chunk counts, restart log | Trace, navigation text, probe bodies | Trace, provider and row reads, 422 and 404 bodies | Trace, banner text | Navigation trace, request log |
| Substituted boundaries | none | none | none | none | none |
| Limitations | Embedded Surreal; one process | YAML readers not re-proven | Picker UI is another change | One provider shape | Page backends may be absent |
| Invalidation keys | `manager.rs`, `api/settings.rs`, `features/settings/**`, `server.rs:1353-1368`, ingestion chunker, `fixtures/retained-settings.json` | same plus settings-type schemas, `fixtures/removed-settings.json` | `api/providers.rs`, `provider_sync.rs`, `api/settings.rs`, `features/providers/**` | `providers-page.tsx`, `use-provider-default.ts` | `nav-destinations.ts`, `admin-page.tsx`, command palette, `fixtures/admin-pages.json` |
| Restoration | Stop instance, delete scratch | same | same, stop stubs | same | same |
| Prerequisites | Production binary; offline-usable embedding backend (UNVERIFIED) | Production binary | Production binary; `stub-llm` binary | same as 003 | Production binary |
| Owner task(s) | 1.1-1.3, 1.5, 3.1-3.3, 5.1 | 1.2, 1.4, 1.6, 2.1, 2.2, 5.2 | 1.3, 1.6, 2.2, 5.3 | 4.1, 5.4 | 4.2, 4.3, 5.5 |

| Field | SCN-SET-006 | SCN-SET-007 | SCN-SET-008 | SCN-SET-009 | SCN-SET-010 |
|---|---|---|---|---|---|
| Name | RAG co-edits persist | Console shows what a reload did | One editing surface for agent and skill records | Secret survives a masked round trip and never leaks | Boot fallback, drift and one-click resolution are visible |
| Actor | Console operator | Console operator | Console operator | Console operator; API client | Console operator |
| Useful outcome | A valid RAG change is never lost to a rejected sibling | Knows what applied, what needs a restart, and whether the reload was rejected | An agent or skill is edited in one place | A key entered once keeps working, and cannot be read back | Knows the system started on an older configuration and can adopt the YAML value |
| Source requirement | `openspec:frontend-configuration-surfaces#No settings control writes a key the backend rejects` | `openspec:frontend-configuration-surfaces#The console shows what a configuration reload did` | `openspec:frontend-configuration-surfaces#Each record type has one editing surface` | `openspec:frontend-configuration-surfaces#A secret setting never leaves the server in clear text` | `openspec:frontend-configuration-surfaces#The console shows how the runtime booted`; `...#The console shows configuration drift and resolves it in one action` |
| Reason now | G5 | Build item 8 UI; review warning | G5 | Non-waivable safety check, review critical 3 | Review critical 8; calibration example 4 |
| Likelihood | Certain | Moderate | Moderate | High: every provider edit | Moderate |
| Mandatory | true | true | false | true | true |
| Nondeterministic | false | true | false | true | true |
| Stochastic | n/a | samples 3; grader: report lists match the fixture; stoppingRule: fixed N=3, all must pass | n/a | samples 3; grader: all oracles in the Observable result hold; stoppingRule: fixed N=3, all must pass | samples 3; grader: banner, panel and drift assertions all hold; stoppingRule: fixed N=3, all must pass |
| Production entry | RAG panel Save; bulk `PUT` | Configuration status, Reload | Settings entries | Providers configure and edit; API reads | Console after rejected-YAML start; drift action |
| Real collaborators | UAR, console | UAR, YAML file, config reload route | UAR, console, Agents and Skills APIs | UAR, `stub-llm` with header recording, console | UAR, scratch YAML, config routes, console |
| Oracle: observation | Read-back of three keys; rejection of the API-direct write | Panel lists; next resilience read; listener value | Link target; saved value on the owning page | Captured bodies and logs; stub-recorded Authorization header; mask lengths | Banner text; drift list text; API value after resolve |
| Oracle: truth source | Values the runner enters | `fixtures/config-screens.json` | The Agents and Skills pages' own save results | `fixtures/secret-roundtrip.json` | `fixtures/config-screens.json`; digest computed by the runner from the bytes it wrote |
| Oracle: comparator | Equality; zero keys persisted on rejection | List membership exactly as in the fixture | Value equality | Substring absence; exact header equality; exact length | Substring presence and absence; exact value equality |
| Negative control: input | Bulk `PUT` with `rag.embedding_model` | `brokenYaml` then Reload | Agent saved with an empty name | Mask placeholder `PUT` for a missing row | Resolve for `llm.default_provider`; resolve twice; banner text scan |
| Expected rejection | 422, no partial write | Rejected state shown, values unchanged | Visible validation error, nothing persisted | 404, nothing created | 409 and no action shown; 404 and "no longer drifting"; no broken-file text in the banner |
| Fixtures | Scratch dir | `fixtures/config-screens.json` | Scratch dir | `fixtures/secret-roundtrip.json` | `fixtures/config-screens.json` |
| Evidence required | Trace, API bodies | Trace, report text, API reads | Trace | All captured bodies, log, recorded headers | Trace, boot-status and drift bodies, screenshots of banner and panel |
| Substituted boundaries | none | none | none | none | none |
| Limitations | None beyond 001 | `applied` depends on the config change | Audit-dependent | Proves absence in captured sinks only | Needs `failed_source_path` from the config change |
| Invalidation keys | `ai-settings-panels.tsx`, `api/settings.rs:916` | `features/settings/**` config screens, config reload route | `governance-settings-panels.tsx`, Agents and Skills pages | `api/settings.rs` mask code, `stub-llm`, provider save path | boot-status, drift and resolve routes, banner and panel files |
| Restoration | Stop instance, delete scratch | same | same | same, plus stop stub | same |
| Prerequisites | Production binary | Production binary; config reload route | Production binary | `stub-llm` with task 1.8 | Production binary; config routes; `failed_source_path` |
| Owner task(s) | 3.4, 5.6 | 4.7, 4.8, 5.7 | 4.4, 5.8 | 1.7, 1.8, 5.9 | 4.5, 4.6, 4.8, 5.10 |
