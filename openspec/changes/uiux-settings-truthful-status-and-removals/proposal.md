## Why

The operator console lets operators edit settings that nothing reads, then tells them the edit is
"effective". Both halves are verified in source:

- **Edits with no reader.** The Settings navigation offers 14 namespaces whose database values no
  code reads (`server`, `security`, `persistence`, `skill_evolution`, `llm_failover`,
  `context_management`, `file_processing`, `sycophancy`, `sandbox`, `vision`, `acp`, `kreuzberg`,
  `unstructured`, `mistral_ocr`) plus the `llm.model` field inside the `llm` namespace
  (`frontend/src/features/settings/ui/settings-navigation.tsx`; classification table in
  `analysis.md`, "Settings namespace classification"). Eleven are honored from YAML at boot only;
  four (`skill_evolution`, `context_management`, `sandbox`, `vision`) are not read at all
  (`runtime/manager.rs:1436` uses `SkillEvolutionConfig::default()`; sandbox runner comes from env
  `UAR_SANDBOX_RUNNER`, `sandbox/platform.rs:21`; `config.sycophancy` and `config.acp` come from the
  boot snapshot, `server.rs:2878,8717`).
- **A false status.** `setting_apply_mode` labels every namespace except `governance`, `server`,
  `security` and `persistence` as `NextTurn` (`src/uar/settings/manager.rs:73-80`). For those,
  `set_value` writes the in-memory "effective" map (`manager.rs:666-671`), so the settings API
  reports `application_status: "effective"` (`src/uar/api/settings.rs:575-582`) even for
  boot-only keys: `rag.chunk_size`, `rag.chunking_strategy` and `rag.semantic_threshold` are read
  once at boot (`server.rs:1357-1368`) and `memory.*` once at boot (`server.rs:1475-1508`). The
  mutation response is partly honest (`restart_required_activation`, `settings.rs:932-940`), the
  per-row status is not. The frontend reads none of it: `SettingWithMeta` declares only a nested
  `meta` (`frontend/src/types/index.ts:320-341`) and no file outside `types/index.ts` reads
  `application_status`, `apply` or `is_drift`.
- **A second system-model control.** `llm.default_provider` is still a schema field with
  `x-control: text` (`manager.rs:2851-2856`). It takes effect only at boot
  (`provider_sync.rs:21-23`); only `POST /api/uar/providers/{id}/default` swaps the live default
  (`src/uar/api/providers.rs:367-399`). Operator decision 2 (2026-11-19) made the registry default
  the one system control. This is unresolved critical finding 3.
- **A misleading banner and a no-op endpoint.** The Providers detail banner prints
  `provider.id/models[0].id` instead of the provider's `default_model`
  (`frontend/src/features/providers/ui/providers-page.tsx:368-371`).
  `POST /.well-known/uar-config/reload` returns 200 with the config JSON schema
  (`server.rs:3743`) although request handlers read the boot snapshot (`server.rs:490`).
- **Unreachable and broken surfaces.** Eight admin sections (`protocols`, `auth`, `cost`,
  `memory`, `compiler`, `credentials`, `models`, `mcp-health`) exist in `pages/admin-page.tsx:32-52`
  with no navigation entry, palette entry or link (assessment G5). The RAG panel's embedding
  provider/model fields write `rag.embedding_*`, which the backend rejects
  (`api/settings.rs:916-927`); the bulk save rejects the whole request before anything persists
  (`api/settings.rs:1353-1361`), so valid co-edits (`chunk_size`, `chunking_strategy`,
  `semantic_threshold`) are lost.

Operator decision 6 (2026-11-19) removes settings with no reader from the UI and keeps the YAML
keys. Operator decision 2 makes the registry default the system level. This change implements both
and makes the console show the truth about every setting that remains.

## What Changes

- Add one classifier of how each persisted setting is applied (`live`, `next_turn`, `restart`,
  `inert`) with a reader citation per entry. Report `application_status` from it. Never report
  `effective` for a key whose reader has not applied the saved value. Add `not_applied` for
  `inert` keys. Unclassified keys default to `restart`, not to `effective`.
- Stop `set_value` writing the effective map for every non-restart key. The component that actually
  applies a value records it (the live default-provider route, the embedding hot-swap).
- Mark the removed namespaces and the `llm.model` / `llm.default_provider` fields as not rendered,
  in the settings-type schema, so the console hides them from data and from an explicit navigation
  list. YAML keys, boot readers and stored DB rows stay.
- Show apply status in the console: a typed domain hook behind `@/platform/entities`, a per-row
  indicator, and a save summary naming restart-required keys. Fix the `SettingWithMeta` type to
  the flattened fields the backend sends.
- Remove the RAG embedding fields. One owner per embedding choice.
- Replace the Settings "Agents" and "Skills" record editors with links to the owning pages,
  after a parity audit (per-row decisions in design.md).
- Relink the eight unreachable admin pages from their natural parent surface and the command
  palette.
- Providers banner reads `default_model`.
- The settings API rejects writes to `llm.default_provider` (422, naming the live default route), so
  the registry default has exactly one writer. This also removes the conflict the realtime review
  found: replicas refresh from that row, and a plain `PUT` must not become a live system-model change.
- A masked-secret round-trip requirement and an audit that no log or response carries a secret
  value; `stub-llm` records Authorization headers so retention can be observed.
- Console screens that consume the config change's routes: a boot-fallback banner naming the
  rejected file, a drift list with one-click "Use YAML value", and a reload report (a
  "Configuration status" panel).
- Add `settings-apply-status` capability; modify `frontend-configuration-surfaces`; add to
  `remaining-admin-surface-certification`.

## Capabilities

### New Capabilities

- `settings-apply-status`: classification of persisted settings by how they are applied, honest
  status in API responses, the schema marker for controls that must not render, and the single
  writer rule for the default provider.

### Modified Capabilities

- `frontend-configuration-surfaces`: removes settings with no reader, displays apply status, one
  editing surface per record type, no control writes a rejected key, one system model control,
  masked-secret round trip, and the boot-status, drift and reload screens.
  Two existing requirements are modified (`Settings behavior remains compatible through
  decomposition`, `Settings namespace reads use canonical backend slugs`) because they freeze the
  navigation inventory and use `context_management` and `server` as examples of retained
  namespaces.
- `remaining-admin-surface-certification`: reachability of stable admin pages and the Providers
  default banner. (The reload report moved to a console requirement; the report itself is the
  config change's.)

## Impact

- Rust: `src/uar/settings/manager.rs` (classifier, `set_value`, schema markers),
  `src/uar/api/settings.rs` (status, activation, rejection of `llm.default_provider`),
  `src/uar/api/providers.rs` (record effective after live default swap), `src/bin/stub-llm.rs` and
  `tests/integration/live/stub_llm.rs` (test support: record Authorization headers).
- Frontend: `frontend/src/features/settings/**` (navigation, panels, generic schema panel, API,
  model hooks), `frontend/src/types/index.ts`, `frontend/src/platform/entities/**` (new settings
  status domain), `frontend/src/features/providers/ui/providers-page.tsx`,
  `frontend/src/app/shell/nav-destinations.ts`, `frontend/src/pages/admin-page.tsx`,
  command palette, e2e specs that name removed namespaces (`settings-routes-installed`,
  `settings-decomposition`).
- No dependency pin change. No change to YAML keys or boot readers.
- Order: this change lands before `uiux-design-context-and-settings-regroup`, which regroups the
  surviving inventory. Shares `generic-schema-panel.tsx` with `uiux-schema-controls-and-pickers`;
  the lead must assign one writer per file (see handoff).

## Related open changes

Suggested dispositions. The operator decides in Plan (decision 7). Ledger states were counted from
`tasks.md` checkboxes, not verified against code.

| Open change | Suggested | Reason |
|---|---|---|
| `migrate-settings-mutations-and-retire-stores` (0/15) | supersede | Mandates a terminal aesthetic and keeps `settings-types-meta-store`; conflicts with the redesign and with the schema-marker approach here. |
| `rewrite-use-settings-hook`, `retire-settings-store-and-bus-source`, `audit-doc-flip-settings-direct` (no open tasks) | archive | Ledgers hold no open work; reconcile against code before archiving. |
| `add-settings-fetcher-and-hook` (0/3), `add-settings-form-cache` (0/5), `migrate-settings-reads-transitional` (0/7) | archive | The fetcher, form cache and hook exist in `features/settings/model` and `api`; remaining tasks describe finished work. UNVERIFIED per task. |
| `config-hot-reload-vault` (28/30) | continue, folded into `uiux-config-boot-safety-and-live-reload` | Claims handlers read the live config; they read the boot snapshot (`server.rs:490`). The reload report here depends on that change's consumers. |
| `make-config-authoritative-on-boot` | archive as superseded | R3 was reverted deliberately (decision 2026-11-19). Owned by the CFG writer; listed for completeness. |
| `explain-agent-model-status` | continue | Provider-default projection; the Providers banner fix reads the same `default_model`. |
| `admin-agent-provider-first-model-picker`, `impeccable-uiux-audit`, `uiux-remediation-wave-1` (empty shells) | archive | Contain no artifacts. |
| `providers-page-direct-reads` (8/10), `providers-page-direct-mutations` (9/12) | continue or archive after the Providers banner task lands | Touch `providers-page.tsx`; check remaining tasks against code before deciding. |

## Unresolved decisions

1. **Stored rows of removed namespaces.** Default proposed here: leave them inert. The rows stay
   in the `settings` table, the types stay registered, the API still reads them and now reports
   `not_applied`. Alternative: delete the rows in a one-time migration that touches only those
   namespaces and never any other key (`.prometheus/gotchas.md`, 2026-08-25 entry: a migration must
   not overwrite explicit operator values). Deletion also removes the permanent "DB differs from
   YAML" drift on those rows. Operator confirms in Plan.
2. **API writes to inert keys.** Default: accept and report `not_applied`. Alternative: reject with
   a validation error naming the YAML key, as `rejected_unsupported` already does for
   `rag.embedding_*` (`api/settings.rs:916`). Rejection could break existing API clients and
   behavior tests. Operator confirms.
3. **Four per-key reader audits** are still UNVERIFIED by this writer (see design.md, "Classifier
   table"). Execute task 1.1 must close them before the table is committed.
4. **Interface asks of `uiux-config-boot-safety-and-live-reload`.** (a) A `failed_source_path`
   field on the boot-status `fallback` object (its design D8 has digests and a key path only; the
   operator-visible requirement is a warning that names the rejected file). (b) A YAML edit of a
   YAML-at-boot key is reported `restart_required` in the reload report, while a DB edit of the
   same key is `not_applied`. Without (a), SCN-SET-010 cannot pass.
5. **Test-support ownership.** Recording `Authorization` headers in `stub-llm` touches
   `src/bin/stub-llm.rs` and `tests/integration/live/stub_llm.rs`, which other changes' scenarios
   also use. The lead assigns one writer.
6. **Reload route wording.** The earlier draft of this change put a report on
   `POST /.well-known/uar-config/reload`; that is withdrawn in favor of
   `POST /api/uar/settings/reload` from the config change, so there is one report, not two.
