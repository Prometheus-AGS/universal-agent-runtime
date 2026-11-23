## Why

Phase `uar-uiux-architecture-2026-11` goals G6 and G7: a picker everywhere a provider,
model or other enumerable value is chosen, and text only for prompts and content. Analysis
build items 19 and 20 and round-2 critical finding 2 found the work unspecified. Facts below
were re-read at source in this Spec stage (grep and file reads; nothing was run).

- **Two model pickers, neither complete.** `features/models/ui/model-selector.tsx` is a
  Popover plus Command list over `ConfiguredModel` entities (`:51-207`). `SettingModelPicker`
  (`features/settings/ui/settings-primitives.tsx:244`) is a Base UI Combobox that switches
  from a plain select to a searchable combobox at 8 options (`SEARCHABLE_MODEL_THRESHOLD`).
  Neither is virtualized. `ModelSelector` has four external callers (`chat/session-config-panel.tsx:22`,
  `agents/ui/agent-editor.tsx:16`, `teams/ui/binding-model-step.tsx:2`, `skills/ui/skills-page.tsx:21`).
- **The generic schema panel renders by type, not by control kind.**
  `features/settings/ui/generic-schema-panel.tsx:216-293` handles boolean, enum, number, array,
  object, `x-control: json`, `x-sensitive` and `textarea`. Arrays become one comma-separated
  text field (`:253-266`), objects a raw JSON textarea (`:165-206,268-270`), numbers a bare
  number input (`:240-250`), everything else monospace text (`:287-293`). The backend declares
  more control kinds than the panel reads: `toggle`, `slider`, `select`, `number`, `text`,
  `password`, `url`, `tag-input`, `multi-select`, `json`, `textarea`, `switch`, `pricing-table`,
  `model-card`, `capabilities-grid` (count by grep in `src/uar/settings/manager.rs`;
  `x-slider-min` and `x-slider-max` exist, no unit hint exists). The panel reads only `json`
  and `textarea`.
- **13 call sites use that panel.** Twelve registry entries
  (`features/settings/ui/settings-panel-registry.tsx:34-125`: `llm`, `context_strategy`,
  `models`, `server`, `persistence`, `sandbox`, `security`, `sycophancy`, `native_tools`,
  `skill_evolution`, `acp`, `llm_failover`) plus the fallback in `settings-page.tsx:88`. Eight
  of those namespaces leave the UI under operator decision 6 (change
  `uiux-settings-truthful-status-and-removals`), so the lasting users are `llm`,
  `context_strategy`, `models`, `native_tools` and any unmapped namespace.
- **Provider and model values are typed by hand in several places.** Operative ones that stay
  in the UI: `llm.embedding.model` is a schema field with `x-control: text`
  (`src/uar/settings/manager.rs:2864-2869`); the knowledge-base default embedding provider and
  model (`features/settings/ui/panels/ai-settings-panels.tsx:548-566`); the memory embedding
  model (`memory-settings-panel.tsx:142`); the credentials provider id with a datalist
  (`features/credentials/ui/credentials-page.tsx:226`). The vision, summarization, RAG embedding and OCR sites listed in assessment G6 and G7 are not part of this change: `uiux-settings-truthful-status-and-removals` removes those panels.
- **Multi-value and numeric fields are typed by hand.** Retryable HTTP status list
  (`resilience-settings-panel.tsx:430`, CSV parsed with `Number`), agent allowed tools as CSV
  (`governance-settings-panels.tsx:796`, in `AgentConfigPanel`), a native checkbox list
  (`governance-settings-panels.tsx:583`), skill keywords and preferred tools as CSV
  (`features/skills/ui/skills-page.tsx:370,384`), 16 `type="number"` inputs across the
  resilience and governance panels, memory page user and agent id filters
  (`features/memory/ui/memory-page.tsx:198,204`).
- **Server paths are typed.** Skill import takes an absolute directory path
  (`features/skills/ui/skill-import-dialog.tsx:119`) and posts it to `POST /api/uar/skills/import`
  (`features/skills/api/skills-api.ts:67-76`), which reads that directory on the server
  (`src/uar/api/skills.rs:508-518`). A browser cannot produce a server path. The capability
  catalog classes this endpoint as host-controlled (`src/uar/api/administration_capabilities.rs:947-952`).
  A content-based endpoint exists (`POST /api/uar/skills/import-content`,
  `src/uar/api/skills.rs:66,464-500`, takes `skill_markdown`) and the frontend does not call it.
  The catalog has no entry for it (grep); its authority class is UNVERIFIED. No Tauri dialog
  plugin is installed (`frontend/package.json:51-52` lists only `@tauri-apps/api` and
  `plugin-shell`).
- **No embedding option source exists in the catalog.** `ModelInfo` has capability flags
  `tool_call`, `reasoning`, `structured_output`, `attachment`, `temperature`, `streaming` and
  input and output modalities (`src/llm/catalog.rs:126-176`), and no embedding flag. Whether
  embedding models appear in the catalog at all is UNVERIFIED.
- **Assessment corrections.** The assessment calls the team workspace control a typed *path*
  (`features/teams/ui/workspace-control.tsx:26`); source shows a workspace *id* sent as the
  `x-uar-workspace-id` header (`platform/entities/teams/api/teams-transport.ts:47-51`). It
  lists `resilience.fallback_models` as JSON; the field is `llm_failover.fallback_models`
  (`manager.rs:3011-3023`, namespace `llm_failover`), which leaves the UI under decision 6.

## What Changes

- **A schema control registry** replaces type-only rendering in `generic-schema-panel.tsx`.
  It resolves each property to a control by `x-control` kind, then by an explicit
  namespace-and-key binding for provider and model fields, then by JSON type. Kinds it adds:
  chip list for string arrays, multi-select for enum arrays, grouped fields for objects with
  declared properties, a bounded slider or numeric stepper with a unit for numbers, and
  provider and model pickers. A raw JSON editor survives only inside an "Advanced"
  disclosure for an object with no declared properties. The dispatch pattern is borrowed
  from `@jsonforms/react` (reference only, no dependency).
- **One picker family** replaces `ModelSelector` and `SettingModelPicker`: a presentational
  Base UI Combobox composition (`components/ui/combobox.tsx` is the wrapper), grouped by
  provider, searchable, virtualized with `@tanstack/react-virtual` past about 1,000 items,
  showing a stored value that is no longer available as unavailable instead of dropping it.
  Domain wrappers (`ModelPicker`, `ProviderPicker`, a provider-scoped model picker) live in
  `features/models` behind a named narrow entry and read options through
  `@/platform/entities` hooks.
- **The hand-typed controls above are converted** or recorded as accepted free-text
  exceptions (prompts, content bodies, names, secrets, URLs, identifiers the operator is
  creating, server filesystem paths without an enumerable source). The accepted list is a
  frozen inventory, written before implementation, that an end-to-end sweep compares with every text control it finds.
- **Skill import stops asking for a typed server path.** Default: a browser file chooser
  reads `SKILL.md` and posts it to `import-content`. This is gated on a decision by the
  trusted-host owner and carries a stated limitation (see design.md D5).
- **No backend change in this change.** Embedding option sources and unit hints are listed as
  coordination items for the state and runtime owners.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-configuration-surfaces`: new requirements for typed schema controls, one
  provider and model picker family, no free text for enumerable values with a fixed
  inventory, and path-free skill import. The existing "Provider default models use bounded
  selection" requirement keeps its thresholds and is not modified.

## Impact

- Frontend source (owner `uar-console`): `frontend/src/features/settings/ui/**`
  (`generic-schema-panel.tsx`, `settings-primitives.tsx`, the panels listed above),
  `frontend/src/features/models/**` (picker wrappers, narrow entry),
  `frontend/src/components/ui/**` (presentational picker primitive and chip list),
  `features/skills/ui/**`, `features/memory/ui/memory-page.tsx`,
  `features/credentials/ui/credentials-page.tsx`, `features/teams/ui/**`,
  `features/presentations/ui/presentation-editor.tsx`, four `ModelSelector` call sites,
  `frontend/e2e/**`; the free-text inventory is a frozen fixture in this change (`fixtures/free-text-inventory.json`), not an implementer file.
- Shared file, single writer assigned in Plan: `features/settings/ui/generic-schema-panel.tsx`, also edited by `uiux-settings-truthful-status-and-removals`, which adds the markers `inert`, `not_applied`, `x-uar-ui-hidden` and `x-uar-reload` and fixes `SettingWithMeta` to the flattened API fields. This change consumes those markers and does not redefine them.
- Other files shared with changes in this phase: `ai-settings-panels.tsx`,
  `file-processing-settings-panels.tsx` and the settings registry are also edited by
  `uiux-settings-truthful-status-and-removals` (removals) and
  `uiux-design-context-and-settings-regroup` (regrouping). One writer per file per Plan wave;
  this change should follow the removals so it does not convert controls that are leaving.
- Dependency on `uiux-model-identity-graph-ownership`: model pickers read `Model`,
  `ConfiguredModel` and `Provider` entities through its domain hooks.
- No dependency pins change; no submodule is edited; no `src/**` edit.

## Related open changes

Not modified here; the operator decides in Plan (decision 7).

| Open change | State | Suggested disposition | Reason |
|---|---|---|---|
| `admin-agent-provider-first-model-picker` | empty shell (`.openspec.yaml` only) | archive, successor this change | No artifacts; the provider-first picker is specified here. |
| `migrate-settings-mutations-and-retire-stores` | 0 of 15 tasks | continue, narrowed | Retires the settings store and applies an older terminal aesthetic; the control work here does not depend on it, but the aesthetic part conflicts with `docs/design/` as visual authority and should be dropped. |
| `rewrite-use-settings-hook`, `retire-settings-store-and-bus-source` | empty task lists | archive as superseded | Placeholder proposals ("Per the plan's per-change synopsis") with no content. |
| `explain-agent-model-status` | 6 of 6 tasks | continue to archive | Complete; unaffected. |
| `add-settings-form-cache` | 0 of 5 tasks | continue, unrelated | Per-namespace dirty cache; no overlap with control kinds. |
| `uiux-remediation-wave-1`, `impeccable-uiux-audit` | empty shells | archive (design-owner changes decide) | No artifacts. Listed because they name UI remediation. |
