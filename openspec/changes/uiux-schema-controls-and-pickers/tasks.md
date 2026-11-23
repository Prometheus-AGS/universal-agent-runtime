> Slice S1 plan and task model assignments: `.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/plan.md` ("Task model assignments", rows keyed by this change ID). Tasks not listed there are outside S1.

## 1. Picker family (reuse Base UI Combobox and @tanstack/react-virtual)

- [ ] 1.1 Read the Base UI 1.9.0 Combobox documentation for grouped items and virtualized lists (`docs/src/app/(docs)/react/components/combobox/page.mdx` and the autocomplete page cited in candidate cand-002 and cand-003) before writing code; record in the changed-path inventory how the virtualizer attaches (UNVERIFIED in this Spec stage).
- [ ] 1.2 Add a presentational `ComboboxPicker` under `frontend/src/components/ui/` as a compound component (root, trigger, search, list, group, empty, status parts) over the existing `components/ui/combobox.tsx` wrapper. No boolean mode props; variants are separate named components (`SinglePicker`, `MultiPicker`).
- [ ] 1.3 Add virtualization with `@tanstack/react-virtual` for option counts above about 1,000, following the existing use at `features/chat/ui/run-trace-timeline.tsx:2`; below that threshold render all items memoized. Keep the "fewer than eight options renders a simple select" behavior from `SettingModelPicker` (`settings-primitives.tsx:238`).
- [ ] 1.4 Implement the unavailable-stored-value state (invalid, described by recovery text, never auto-cleared) and the loading, error and empty states with accessible names for every part.
- [ ] 1.5 Add domain wrappers in `frontend/src/features/models/ui/`: `ModelPicker` (value is the `provider/model` route), `ProviderPicker` (value is the provider id) and `ProviderModelPicker` (provider fixed, value is the model id) and `CatalogModelPicker` (every catalog model across providers, grouped by provider; consumed by `uiux-system-model-live-default` for the system default). Options come from `@/platform/entities` hooks defined by `uiux-model-identity-graph-ownership` (`useConfiguredModels`, `useCatalogProviders`, `useCatalogModelsForProvider`).
- [ ] 1.6 Export the wrappers through a named narrow entry `frontend/src/features/models/pickers.ts`, as `features/models/model-selector.ts` does today, so consumers do not pull the models page into the initial bundle.

## 2. Replace the two existing pickers

- [ ] 2.1 Migrate the four `ModelSelector` call sites to `ModelPicker`: `features/chat/session-config-panel.tsx:82`, `features/agents/ui/agent-editor.tsx:362,378`, `features/teams/ui/binding-model-step.tsx:50`, `features/skills/ui/skills-page.tsx:397`. Keep each call site's value, clear and placeholder semantics.
- [ ] 2.2 Migrate `SettingModelPicker` callers (`features/settings/ui/panels/ai-settings-panels.tsx:227`) to `ProviderModelPicker`, preserving the provider default-model requirement already in the spec (search over display name and raw id, ambiguity disclosure, stale value reporting).
- [ ] 2.4 Replace the Add Model dialog's two plain selects (`features/models/ui/models-page.tsx:460-531`, provider select and an unsearchable model select over every catalog model of that provider) with `ProviderPicker` and `ProviderModelPicker`.
- [ ] 2.3 Delete `features/models/ui/model-selector.tsx`, `features/models/model-selector.ts`, `SettingModelPicker` and `SEARCHABLE_MODEL_THRESHOLD` from `settings-primitives.tsx` after the last caller moves; update their tests to the new components.

## 3. Schema control registry

- [ ] 3.1 Create `features/settings/ui/schema-controls/` with a registry of `{ matches(prop, key, namespace), Control }` entries ordered by `x-control` kind, then namespace-and-key binding, then JSON type (dispatch shape borrowed from `@jsonforms/react`, reference only; no dependency, `@rjsf/core` stays rejected).
- [ ] 3.2 Implement separate control components, not mode flags: `ToggleControl`, `EnumControl`, `SliderControl` (reads `x-slider-min`, `x-slider-max`), `StepperControl` (unbounded numbers, with unit), `ChipListControl` (string arrays), `MultiSelectControl` (enum arrays and `x-control: multi-select`), `ObjectFieldsControl` (declared `properties`, one nesting level), `UrlControl`, `SecretControl`, `TextareaControl`, `AdvancedJsonControl` (objects without `properties`, inside a collapsed disclosure, validated before save).
- [ ] 3.3 Derive a unit label from an `x-unit` hint when present, otherwise from a trailing parenthetical in the title such as "(seconds)" or "(KB)"; show none when neither exists. Coverage of units across current schemas is UNVERIFIED; record the count of numeric fields without a unit in the changed-path inventory.
- [ ] 3.4 Consume, do not redefine, the markers added by `uiux-settings-truthful-status-and-removals` (`inert`, `not_applied`, `x-uar-ui-hidden`, `x-uar-reload`, flattened `SettingWithMeta`): registry controls render a hidden field as absent and an inert or not-applied field read-only with the status text. Land after that change in `generic-schema-panel.tsx`.
- [ ] 3.5 Rewrite `GenericSettingControl` (`generic-schema-panel.tsx:209-293`) to call the registry; keep `GenericSchemaPanel`'s props, dirty tracking, Reset, conflict banner and save flow unchanged.
- [ ] 3.6 Honor the control kinds the backend already declares but the panel ignores (`slider`, `tag-input`, `multi-select`, `url`, `switch`, `select`); leave `pricing-table`, `model-card` and `capabilities-grid` on the JSON-type fallback and list them in the changed-path inventory.
- [ ] 3.7 Do not add bindings for `llm.default_provider`, `llm.model`, `skill_evolution.*`, `llm_failover.*`, the `rag.embedding_*` fields or any key in a namespace removed by `uiux-settings-truthful-status-and-removals`; read that change's removal list before adding a binding.

## 4. Bespoke panels use the structured controls

- [ ] 4.1 `resilience-settings-panel.tsx`: replace the retryable-status CSV (`:430`) with a multi-select over the standard HTTP status list, and the bare millisecond and count fields with bounded steppers with units.
- [ ] 4.2 `governance-settings-panels.tsx`: replace agent allowed tools CSV (`:796`) with a multi-select over the tool catalog (`entities/hooks/use-tools.ts`), the native checkbox list (`:583`) with the shared checkbox or toggle-group primitive, and the budget number fields with bounded steppers.
- [ ] 4.3 Add the multi-select-from-catalog control as one reusable component (`ToolMultiSelect`) so the agents and skills editors share it.

## 5. Provider and model fields

- [ ] 5.1 Bind `llm.embedding.model` in the registry to a `ProviderModelPicker` scoped to the value of `llm.embedding.backend`; hold the option source as a decision (design.md D4) and render the stored value read-only with a stated reason until a source exists.
- [ ] 5.2 Convert the knowledge-base default embedding provider and model (`ai-settings-panels.tsx:548-566`) and the memory embedding model (`memory-settings-panel.tsx:142`) to `ProviderPicker` and `ProviderModelPicker` with the same option-source rule. Note that `memory.*` needs a restart (assessment G3); keep that status visible.
- [ ] 5.3 Convert the credentials provider id (`features/credentials/ui/credentials-page.tsx:226`) to `ProviderPicker` over catalog and configured providers on create; show it read-only on rotate (existing `disabled` behavior at `:229`).

## 6. Page-level free text

- [ ] 6.1 Skills editor: skill keywords become chips; preferred tools become `ToolMultiSelect` (`features/skills/ui/skills-page.tsx:370,384`). Keep the serialized form state shape so existing request builders are unchanged.
- [ ] 6.2 Memory page: the agent id filter becomes an agent picker over the graph's agents (`features/agents/model/use-agents.ts`); the user id filter keeps free text only if no option source is confirmed, and is then an inventory textbox with reason "search-filter" (already listed in the frozen inventory) (`features/memory/ui/memory-page.tsx:198,204`).
- [ ] 6.3 Teams: classify the launch input JSON (`features/teams/ui/launch-sheet.tsx:155`), the binding reference fields (`binding-reference-fields.tsx:26`) and the workspace id (`workspace-control.tsx:26`). Convert each whose option source Plan confirms (a known-workspaces list from the existing local memory of recent workspaces is the only source found, `teams-location.ts:38-40`); the rest are already listed in the frozen inventory with their reason. The team launch input is content for the team and stays text.
- [ ] 6.4 Presentations: the template source editor (`features/presentations/ui/presentation-editor.tsx:66`) is declarative JSON content and stays text (listed in the frozen inventory). Providers page base URL (`features/providers/ui/providers-page.tsx:279`) stays a validated URL input prefilled from the catalog default (listed in the frozen inventory).

## 7. Skill import without a typed path

- [ ] 7.1 Wait for the trusted-host owner's decision on the `import-content` endpoint (design.md D5) before changing the import call.
- [ ] 7.2 Replace the directory-path input in `features/skills/ui/skill-import-dialog.tsx:119` with a file chooser for `SKILL.md`; show name, description and version parsed from its frontmatter before confirming; post to the confirmed endpoint; map the server's `skill_markdown_invalid` and `skill_id_conflict` responses to visible messages.
- [ ] 7.3 Remove the typed-path request from `features/skills/api/skills-api.ts:67-76` only when no caller remains, and state in the changed-path inventory that references, scripts and assets in a skill directory are not carried by the content import.

## 8. Sweep and spec authoring (the inventory is frozen; no register is written)

- [ ] 8.1 Author the sweep `frontend/e2e/free-text-sweep.spec.ts`: visit each audited surface listed in `openspec/changes/uiux-schema-controls-and-pickers/fixtures/free-text-inventory.json`, enumerate controls by ARIA role, and compare with the inventory in both directions (an unlisted textbox fails; a `mayBeTextbox` entry with no control fails as stale; each `mustNotBeTextbox` entry must have a non-textbox role with options equal to its named source; each `mustBeBoundedNumber` site must have min and max). The sweep reads the inventory and never writes it. Authoring only; do not run before the phase boundary.
- [ ] 8.2 Author the other boundary specs named below (`schema-controls`, `model-picker`, `embedding-and-provider-pickers`, `skill-import`, `multi-value-controls`) against the frozen fixtures. Authoring only.
- [ ] 8.3 Do not create `frontend/test-fixtures/free-text-register.json`. If the implementation adds a control the inventory does not list, report it to the lead; do not add it to the inventory.

## 9. Boundary acceptance (runs only at the phase boundary)

These tasks describe what the final integration gate exercises. They must not be run early. Expected values come only from the frozen fixtures in design.md. Provider upstreams use the stub model server (`target/debug/stub-llm`); no scenario calls a paid model route. A mandatory scenario whose negative control does not run is BLOCKED.

- [ ] 9.1 SCN-PICK-001: for the 11 frozen fields, check control kind and bounds against `schema-controls-oracle.json`, edit, save, reload and compare with `GET /api/uar/settings/<namespace>`; enter each frozen out-of-range value and assert it is rejected. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/schema-controls.spec.ts`, 3 samples.
- [ ] 9.2 SCN-PICK-002: with `picker-config.yaml`, open the Add Model dialog for `nano-gpt`; assert 597 options in order, the frozen search counts (44, 54, 28, 85, 0), keyboard selection, and long tasks under 200 ms; run the baseline bundle and require its model control to lack search. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/model-picker.spec.ts`, 5 samples.
- [ ] 9.3 SCN-PICK-003: compare the credentials and knowledge-base provider option sets with the 343 ids frozen in `fixtures/provider-options.json`; choose `anthropic` (credentials, fixed fake key) and `openai` (knowledge-base default) and compare the read-back with the frozen expected values; type `not-a-provider` and assert nothing is selected or created. The embedding model fields are BLOCKED until D4 names a source. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/embedding-and-provider-pickers.spec.ts`, 3 samples.
- [ ] 9.4 SCN-PICK-004: run `e2e/free-text-sweep.spec.ts` against the frozen inventory on the candidate and on the baseline bundle; require at least 10 of the 12 `mustNotBeTextbox` entries to fail on the baseline. 3 samples.
- [ ] 9.5 SCN-PICK-005: import `skill-import/valid/SKILL.md` through the chooser, `skill-import/invalid/SKILL.md`, and the valid file a second time; compare the skill list with `GET /api/uar/skills`. BLOCKED until the authority decision in task 7.1 is recorded. 3 samples.
- [ ] 9.6 SCN-PICK-006: compare the status-code options with the frozen list, the agent filter options with the two seeded agents (`pick-agent-a`, `pick-agent-b`) bounded above by `GET /api/agents`, and the tool options bounded above by `GET /api/tools`; save the frozen selections and compare; attempt status codes 99 and 600. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/multi-value-controls.spec.ts`, 3 samples.
- [ ] 9.7 SCN-PICK-007: for the 18 rows of `fixtures/numeric-bounds-oracle.json`, compare each control's minimum and maximum with the frozen values; enter each frozen out-of-range input and assert rejection; save the five frozen in-range values and compare the read-back with them. If the source digest of `manager.rs` differs from the fixture, report BLOCKED. 3 samples.

## 10. Prerequisites for the boundary runner (the lead adds the commands)

- [ ] 10.1 Runtime binary and `stub-llm` binary compiled under the serialized single-writer rule.
- [ ] 10.2 Baseline worktree created with `scripts/worktree-new.sh uiux-pick-baseline --base f2c9e36efe088e06a17c5d880ada877e25f2ad52` (under `~/.claude/worktrees/`) and its frontend bundle produced; removed with `scripts/worktree-rm.sh` after the run.
- [ ] 10.3 The settings-truth change's markers and removals present, and the regrouping change's navigation or deep links, so the sweep can reach every audited section.
- [ ] 10.4 For the cross-provider (8,170 option) picker scale check: the system default control from `uiux-system-model-live-default` mounted; otherwise the virtualization requirement has no scenario (reported to the lead).
