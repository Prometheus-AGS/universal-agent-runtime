## Context

Build items 19 and 20 and round-2 critical finding 2 of `analysis.md` (schema panel CSV,
JSON and bare-number rendering; the hand-written free-text controls at assessment lines
211-223; the path-picker evaluation). Scenario area `PICK`. **Source** means read at file:line
in this Spec stage; **runtime** means observed by running something. Nothing was run, so every
defect is source-derived.

Out of scope by lead instruction: the vision, summarization, RAG embedding and OCR free-text
sites. `uiux-settings-truthful-status-and-removals` removes those panels. Also out:
`llm.default_provider` and `llm.model` (critical 3, that change), `skill_evolution.*` and
`llm_failover.*` (namespaces removed).

### Shared file, single writer assigned in Plan

`frontend/src/features/settings/ui/generic-schema-panel.tsx` is edited by this change and by
`uiux-settings-truthful-status-and-removals` (SET). SET adds the markers `inert`,
`not_applied`, `x-uar-ui-hidden` and `x-uar-reload` and fixes `SettingWithMeta` to the
flattened API fields. This change **consumes those markers and does not redefine them**: a
control the registry would render is hidden when `x-uar-ui-hidden` is set, and read-only
with the status text when `inert` or `not_applied` is set. Plan assigns one writer; the
suggested order is SET first, then this change.

### Verified at source

| Claim | Location |
|---|---|
| Panel renders by JSON type; array is CSV text, object raw JSON, number bare | `features/settings/ui/generic-schema-panel.tsx:165-206,209-293` |
| Backend declares more control kinds than the panel reads | grep of `"x-control"` values in `src/uar/settings/manager.rs`: toggle 40, slider 24, select 23, text 18, number 18, password 9, url 5, tag-input 4, multi-select 2, json 2, textarea 1, switch 1, pricing-table 1, model-card 1, capabilities-grid 1; the panel reads `json` and `textarea` only |
| Slider bounds exist; no unit hint exists | `x-slider-min`, `x-slider-max` (4 each); no `x-unit` in `manager.rs` (grep) |
| 13 call sites use the panel | `settings-panel-registry.tsx:34-125` (12) and `settings-page.tsx:88` |
| `llm.embedding.model` is `x-control: text` | `src/uar/settings/manager.rs:2864-2869` |
| Two model pickers | `features/models/ui/model-selector.tsx:51`; `settings-primitives.tsx:244` (threshold `:238`) |
| Add Model dialog uses two plain selects | `features/models/ui/models-page.tsx:460-531` |
| `ModelSelector` has four external callers | `chat/session-config-panel.tsx:22,82`; `agents/ui/agent-editor.tsx:16,362,378`; `teams/ui/binding-model-step.tsx:2,50`; `skills/ui/skills-page.tsx:21,397` |
| Combobox chips primitives exist | `components/ui/combobox.tsx` (`ComboboxChips`, `ComboboxChip`, `ComboboxChipsInput`) |
| Virtualizer already used | `features/chat/ui/run-trace-timeline.tsx:2` |
| Skill import posts a typed path; server reads that directory | `features/skills/ui/skill-import-dialog.tsx:119`; `features/skills/api/skills-api.ts:67-76`; `src/uar/api/skills.rs:508-518` |
| Import by path is host-controlled; `import-content` has no catalog entry | `src/uar/api/administration_capabilities.rs:947-952`; grep for `import-content` in that file finds nothing |
| Catalog has no embedding capability flag | `src/llm/catalog.rs:126-176` |
| Workspace is an id header, not a path | `platform/entities/teams/api/teams-transport.ts:47-51` |
| `llm_failover.fallback_models`, not `resilience` | `src/uar/settings/manager.rs:3011-3023` (inside `make_type("LLM Failover", "llm_failover", ...)`) |

### UNVERIFIED

- How Base UI 1.9.0 Combobox attaches an external virtualizer. Candidate cand-003 cites the
  autocomplete documentation page; the page was not read in this stage. Task 1.1 reads it first.
- Whether embedding models appear in the catalog at all, and under which modality or family.
- Whether `POST /api/uar/skills/import-content` carries the same host-controlled authority
  class as `skills.import`; the capability catalog has no entry for it.
- Whether the client can reproduce the server's frontmatter parsing (`parse_frontmatter`) well
  enough for a pre-import preview.
- The count of numeric schema fields that have a unit in the title; the unit rule is a heuristic.
- Which team binding references have an enumerable source.

## Rules applied (task-specific summary)

Vercel Composition Patterns: `architecture-avoid-boolean-props` and
`architecture-compound-components` shape the picker (a compound `ComboboxPicker` with parts,
and separate `SinglePicker` and `MultiPicker` variants, not `multi`, `grouped` or `searchable`
flags); `patterns-explicit-variants` gives one named control component per schema kind
(`SliderControl`, `StepperControl`, `ChipListControl`, and so on) instead of one component
with a `mode` prop; `state-decouple-implementation` keeps option sources behind domain hooks.
Vercel React Best Practices: `rerender-split-combined-hooks` and `rerender-derived-state` (each
control subscribes to one field; a `ModelPicker` subscribes to its option list, not to the
settings draft), `rerender-memo` (items memoized under about 1,000), `rendering-content-visibility`
and virtualization beyond it, `rerender-use-deferred-value` for the search query on the 8k
list, `rerender-no-inline-components` (registry entries are module-level components),
`bundle-barrel-imports` (named narrow entry `features/models/pickers.ts`). Prometheus Entity
Management and the project rule: provider, model, agent and tool options are graph-owned
records read through typed hooks from `@/platform/entities`; the picker never fetches or
imports a store; draft values stay in the settings draft, which is UI-local mechanics.

## Decisions

**D1. Registry dispatch, no new form engine.** `features/settings/ui/schema-controls/` holds an
ordered list of `{ matches(prop, key, namespace), Control }` entries. Order: `x-control` kind,
then an explicit namespace-and-key binding for provider and model fields, then JSON type.
`@rjsf/core` stays rejected (cand-004: second form state and theme layer). `@jsonforms/react`
is read for its tester and renderer shape only (cand-005, reference). Unknown kinds fall back
to the JSON-type entry, so a backend that adds a kind never blanks a field.

**D2. Picker family.** A presentational compound `ComboboxPicker` over the existing Base UI
Combobox wrapper (cand-002, adopt). Items are grouped by provider with `Combobox.Group`.
Options past about 1,000 are virtualized with `@tanstack/react-virtual` (cand-003, adopt, the
`run-trace-timeline.tsx` precedent). Domain wrappers live in `features/models` behind a named
narrow entry. The simple-select path below eight options is kept because the spec
"Provider default models use bounded selection" requires it. The stored-but-unavailable value
renders as an invalid option, matching the behavior `ModelSelector` already has
(`model-selector.tsx:91`).

**D3. Option sources.** Session, agent, skill and team model choices keep reading
`ConfiguredModel` entities (enabled configured models, route `provider/model`). The Add Model
dialog and the system default picker need catalog models, which come from the `Model` entities
owned by `uiux-model-identity-graph-ownership`. Provider choices read `Provider` or
`ConfiguredProvider`. Tool choices read the tool catalog (`entities/hooks/use-tools.ts`); agent
choices read agents (`features/agents/model/use-agents.ts`). No picker fetches.

**D4. Embedding model option source (blocker, needs a decision).** The catalog has no
embedding flag, so a picker cannot list embedding models from current data. Options:
(a) the state or runtime owner adds a server-declared option list per embedding backend and
the schema points at it (for example an `x-options-source` hint), then the frontend binds
generically; (b) the frontend infers embedding models from catalog names or families (rejected:
guesswork that breaks silently); (c) accept a typed value behind an "Other" entry (rejected:
free text through a side door, contrary to G7). Recommendation: (a). Until it exists the field
shows the stored value read-only with a stated reason (spec scenario "No option source is
available for a field"). This reduces what operators can do today, which is the cost of not
guessing; the operator may prefer to keep typed entry for these three fields and keep them in the inventory as textboxes.

**D5. Skill import and the path-picker evaluation.**

| Option | What it is | For | Against |
|---|---|---|---|
| A. Browser file chooser plus `import-content` | the dialog reads `SKILL.md`, previews frontmatter, posts `skill_markdown` | no server path; works in browser and desktop; endpoint exists (`skills.rs:66`) | skill directories (`references/`, `scripts/`, `assets/`) are not carried; preview is client-side; authority class of `import-content` unconfirmed; the existing path import is host-controlled |
| B. Server-side directory browser | new read-only endpoint listing directories under configured skill roots | keeps directory import and server preview | new backend endpoint and a new filesystem read capability; trusted-host and state owners; not in this change |
| C. Tauri dialog plugin | native folder dialog in The Boss | native feel | plugin not installed (`package.json:51-52`); a dependency change this change must not make; desktop only; still produces a client path, not a server path, unless the server is local |
| D. Keep the typed path as an inventory exception | status quo | no loss of directory import | violates the goal for the one control most likely to be mistyped |

Recommendation: A for the default web and desktop path, **conditional on the trusted-host owner
(`uar-trust-tools`) confirming `import-content` authority**. If they decline, D with a
`server-path` inventory entry and the host-controlled gate unchanged. Directory import keeps
working through D or B in either case. C is not adopted here; adopting it is an operator and
dependency decision.

**D6. Frozen free-text inventory.** `fixtures/free-text-inventory.json` in this change lists
every control that must not be a textbox, every numeric control that must be bounded, and
every textbox that may remain with a reason from a closed set (`prompt`, `content`, `name`,
`secret`, `url`, `new-identifier`, `search-filter`, `server-path`). It is written now and is
not edited by the implementer; the sweep reads it and never writes it. The first draft's
implementer-written register is dropped. Disposition of every site named in assessment G6 and G7
and found in this stage (the inventory carries the same rows with file and line):

| Site | Disposition |
|---|---|
| Generic panel arrays, objects, numbers (13 call sites; lasting users `llm`, `context_strategy`, `models`, `native_tools`, fallback) | converted (D1) |
| `llm.embedding.model` | picker, blocked on D4 |
| KB default embedding provider and model (`ai-settings-panels.tsx:548-566`) | picker, blocked on D4 for the model |
| Memory embedding model (`memory-settings-panel.tsx:142`) | picker, blocked on D4 |
| Credentials provider id (`credentials-page.tsx:226`) | picker on create, read-only on rotate |
| Retryable status codes (`resilience-settings-panel.tsx:430`) | multi-select over the standard list |
| Agent allowed tools (`governance-settings-panels.tsx:796`) | tool multi-select |
| Approval action checkbox list (`governance-settings-panels.tsx:583`) | shared checkbox or toggle-group primitive |
| Resilience and governance numbers (16 `type="number"` inputs) | bounded steppers with units |
| Skill keywords (`skills-page.tsx:370`) | chips (content-like tags) |
| Skill preferred tools (`skills-page.tsx:384`) | tool multi-select |
| Memory agent id filter (`memory-page.tsx:204`) | agent picker |
| Memory user id filter (`memory-page.tsx:198`) | inventory textbox, `search-filter`, until an option source is confirmed |
| Skill import path (`skill-import-dialog.tsx:119`) | D5 |
| Team launch input JSON (`launch-sheet.tsx:155`) | inventory textbox, `content` |
| Team binding references (`binding-reference-fields.tsx:26`) | inventory textbox, `new-identifier`, pending a confirmed source (UNVERIFIED) |
| Team workspace id (`workspace-control.tsx:26`) | inventory textbox, `new-identifier`; recent workspaces may be offered as suggestions (`teams-location.ts:38-40`) |
| Presentation template source (`presentation-editor.tsx:66`) | inventory textbox, `content` |
| Provider base URL (`providers-page.tsx:279`) | inventory textbox, `url`, validated, prefilled from the catalog |
| `native_tools.file_allowed_paths`, `models` file locations | inventory textbox, `server-path` (host filesystem paths, not enumerable from the browser) |
| `native_tools.web_fetch_allowed_domains` | chips |

**D7. Sequencing.** SET removes controls first; this change then converts what remains, so no
effort is spent on controls that are leaving. `generic-schema-panel.tsx` is the shared file
(see Context). `ai-settings-panels.tsx` and `file-processing-settings-panels.tsx` are SET's
removal targets; this change edits only `ai-settings-panels.tsx` lines 537-566 (knowledge
bases) and `:227` (the existing picker) and must rebase on SET's removals.

## Interfaces and cross-change contracts

- **Consumes** from `uiux-model-identity-graph-ownership`: `useConfiguredModels`,
  `useConfiguredProviders`, `useCatalogProviders`, `useCatalogModels`,
  `useCatalogModelsForProvider`.
- **Provides** to `uiux-system-model-live-default`: `CatalogModelPicker` and `ProviderPicker`
  (via `@/features/models/pickers`) for the system default control, and the stale-value
  behavior it needs to show an unavailable default.
- **Consumes** from `uiux-settings-truthful-status-and-removals`: the `inert`, `not_applied`,
  `x-uar-ui-hidden`, `x-uar-reload` markers and the flattened `SettingWithMeta`.
- **Consumes** from `uiux-design-context-and-settings-regroup`: the new settings grouping and
  deep links; the sweep visits sections through whatever navigation exists then.

## Frozen oracle fixtures

Written now, before implementation, so no expected value comes from anything the implementer
writes during the phase or from the system under test (revision rule R1). The implementer does
not edit these files; a change needs the lead.

| Fixture | Path (under `openspec/changes/uiux-schema-controls-and-pickers/fixtures/`) | Holds |
|---|---|---|
| Free-text inventory | `free-text-inventory.json` | The audited surfaces; 12 controls that must not be textboxes with their option sources; 19 numeric sites that must be bounded (18 with frozen bounds in `numeric-bounds-oracle.json`); 40 controls that may be textboxes with a closed reason each; structural allowances. Built from assessment.md lines 211-223 and a grep audit of every input and textarea in the settings, providers, credentials, skills, memory, agents, teams, presentations and knowledge features. This replaces the implementer-written register of the first draft. |
| Schema controls oracle | `schema-controls-oracle.json` | Expected control kind, bounds and out-of-range inputs for 11 fields in `llm`, `native_tools`, `models` and `context_strategy`, read from `src/uar/settings/manager.rs` by line. |
| Picker oracle | `picker-oracle.json` and `picker-config.yaml` | The committed catalog snapshot digest, provider `nano-gpt` with 597 catalog models, expected search counts (`claude` 44, `gpt` 54, `llama` 28, `qwen` 85, `zzzz-no-match` 0), and the first and last ten sorted ids. The config configures that one provider against the stub model server. |
| Provider options oracle | `provider-options.json` | The 343 provider ids of the committed catalog snapshot (sha pinned), the frozen choices and the frozen expected stored values for the credentials and knowledge-base provider pickers. |
| Numeric bounds oracle | `numeric-bounds-oracle.json` | 18 numeric settings with frozen minimum and maximum (read from `manager.rs` by line, source digest pinned), the frozen out-of-range inputs and five frozen in-range saves. |
| Skill import fixtures | `skill-import/valid/SKILL.md`, `skill-import/invalid/SKILL.md` | A valid manifest (`name: pick-import-fixture`, version 1.0.0) and a file with no frontmatter. The duplicate case imports the valid file twice. |
| Baseline revision | `f2c9e36efe088e06a17c5d880ada877e25f2ad52` | The pre-phase tree; its frontend bundle is the negative control where a row names it. |
| Thresholds | this document | Long task above 200 ms fails; virtualize above 1,000 options; at most 200 option elements in the DOM when virtualized. |

If the committed catalog snapshot digest differs from the fixtures, the runner reports BLOCKED.

## Acceptance

Common real collaborators: the runtime binary with its production static bundle, a scratch
persistence directory per run, the repository stub model server (`src/bin/stub-llm.rs`, started
as `target/debug/stub-llm <fixtures.json>` with `STUB_LLM_PORT` set, as
`frontend/playwright.real-server.config.ts:35-41` does with `tests/bdd/fixtures/bdd-chat.json`),
and Playwright Chromium. No scenario calls a paid model route. Identity comes from the existing
external proxy; tests never mint, carry or send credentials or JWTs. Ports are scratch ports,
never the live `1906` service. Unit, mocked and structural checks are supplemental only. A
mandatory scenario whose negative control did not run is BLOCKED, never PASS.

Common prerequisites (the lead adds the build commands): the runtime and `stub-llm` binaries;
the settings change's markers in place; for scenarios naming the baseline, a worktree created
with `scripts/worktree-new.sh uiux-pick-baseline --base f2c9e36efe088e06a17c5d880ada877e25f2ad52`
(under `~/.claude/worktrees/`) and its frontend bundle; removed afterwards with
`scripts/worktree-rm.sh`. How a bundle is selected at run time is UNVERIFIED and bound by the lead.

### Behavior P1: Schema-driven fields use structured controls and round-trip

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-schema-controls-and-pickers` / `uiux-schema-controls-and-pickers#9.1` |
| Source identity | repository HEAD plus working tree; bundle digest recorded |
| Production entry point | The operator opens the `llm`, `native_tools`, `models` and `context_strategy` settings sections |
| Real collaborators | runtime, `GET /api/uar/settings/types`, `GET` and `PUT /api/uar/settings/<namespace>`, browser |
| Boundary exercised | settings HTTP API; rendered controls and their ARIA roles |
| Observable result | For each of the 11 fields in `schema-controls-oracle.json`: the control has the expected kind (slider or spinbutton with the frozen min and max, chip list, or Advanced disclosure), shows the unit where stated, and after an edit, save and reload the value equals the `GET` response. The server's types response must agree with the frozen rows or the runner reports BLOCKED |
| Negative control | Each frozen out-of-range input (601 for `llm.timeout_secs`, -1 for `llm.max_retries`, 51 for `session_search_max_results`) is not accepted: the control rejects it or Save is disabled with the range stated, and the stored value is unchanged. Invalid JSON in `context_strategy.config` blocks Save |
| Isolation | scratch data dir and port; no other settings touched |
| Prerequisites | runtime and stub binaries; SET's markers |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/schema-controls.spec.ts`, 3 samples |
| Evidence | pending |
| Limitations | Proves 11 frozen fields and the four namespaces, not every schema field. Units only where the title or an `x-unit` states one. Nondeterministic (rendering): 3 samples, all must pass |

### Behavior P2: The model picker is searchable and keyboard operable at provider-catalog scale

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.2` |
| Source identity | as P1, plus the catalog snapshot digest |
| Production entry point | The operator opens the Add Model dialog on `/admin/models` for the configured provider `nano-gpt` (config `picker-config.yaml`) and opens its model control |
| Real collaborators | runtime with the bundled catalog, browser |
| Boundary exercised | graph-backed option list; DOM; keyboard |
| Observable result | The control lists 597 options in sorted order beginning and ending with the frozen first and last ten ids; typing `claude`, `gpt`, `llama`, `qwen` leaves exactly 44, 54, 28 and 85 options; selecting with the keyboard stores that model; the longest task while opening and typing stays under 200 ms |
| Negative control | Typing `zzzz-no-match` shows the empty state and selecting nothing stores nothing; separately, the baseline bundle must fail the first check because its model control is a plain unsearchable select (no search input), otherwise the scenario cannot reject and is BLOCKED |
| Isolation | as P1 |
| Prerequisites | as P1; `picker-config.yaml` loaded; the baseline worktree |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/model-picker.spec.ts`, 5 samples |
| Evidence | pending |
| Limitations | 597 options is below the 1,000-option virtualization point, so this scenario does not exercise virtualization. The 8,170-option cross-provider picker is mounted by the system default control owned by `uiux-system-model-live-default`; its scale scenario belongs to that change, and until it exists the virtualization requirement has no scenario (reported to the lead). Nondeterministic: 5 samples, grader is all four counts equal and the maximum long task under 200 ms, all must pass |

### Behavior P3: Embedding and provider fields accept only listed values

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.3` |
| Source identity | as P1, plus the digest of `provider-options.json` |
| Production entry point | The operator creates a credential, and edits the knowledge-base default embedding provider |
| Real collaborators | runtime settings and credentials APIs, browser |
| Boundary exercised | credentials and settings APIs; picker options |
| Observable result | The credentials provider picker and the knowledge-base default embedding provider picker each list exactly the 343 provider ids frozen in `provider-options.json` (computed from the committed catalog snapshot; `/api/catalog` lists every catalog provider without filtering, `src/server.rs:4091-4111`). Choosing `anthropic` in the credentials picker with the fixed fake key creates a credential for `anthropic`; choosing `openai` for the knowledge-base default saves `openai`. Both are read back with `GET /api/uar/credentials` and `GET /api/uar/settings/knowledge-bases` and compared with the frozen expected values. The API is only the reader of what was stored; the expected values are frozen, so a dropped or altered save fails |
| Negative control | Typing `not-a-provider` into either picker selects nothing: no credential is created and the knowledge-base value is unchanged from its frozen starting value |
| Isolation | as P1 |
| Prerequisites | as P1. The embedding model fields (C-002, C-004, C-005 in the inventory) have no frozen option set because no source exists (decision D4); those rows are BLOCKED and never passed |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/embedding-and-provider-pickers.spec.ts`, 3 samples |
| Evidence | pending |
| Limitations | The embedding-model half is not exercised until D4 names a source; this is the largest hole in this change and the lead must narrow the functional claim until then. Nondeterministic: 3 samples, all must pass |

### Behavior P4: No unlisted free-text control remains

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.4` |
| Source identity | as P1, plus the digest of `free-text-inventory.json` |
| Production entry point | The sweep opens each audited surface in the production UI and enumerates controls by ARIA role |
| Real collaborators | runtime, production bundle, browser, the frozen inventory and the live option sources |
| Boundary exercised | rendered DOM of the audited surfaces; option endpoints (`/api/catalog`, `/api/uar/providers`, `/api/tools`, `/api/agents`) |
| Observable result | Every textbox matches one `mayBeTextbox` entry by surface and accessible name with a reason from the closed set; every `mustNotBeTextbox` entry has a non-textbox role and options equal to its named source; every `mustBeBoundedNumber` site has min and max, equal to the schema where the schema declares them; no surface has an unlisted textbox |
| Negative control | The baseline bundle must fail: it has textboxes for the 12 `mustNotBeTextbox` entries that the candidate must not have, so the baseline run reports at least 10 of 12 as violations. If it reports fewer than 10, the inventory does not discriminate and the result is BLOCKED |
| Isolation | read-only against a scratch runtime with the stub model server and at least one provider, one tool and one agent present |
| Prerequisites | as P1; deep links or section navigation from the regrouping change; the baseline worktree |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/free-text-sweep.spec.ts`, 3 samples |
| Evidence | pending |
| Limitations | Matches by accessible name, so an unlabeled textbox fails (intended). Entries marked `blockedOn` D4 or D5 report BLOCKED. The sweep reads the inventory; it never writes it. Nondeterministic (rendering): 3 samples, all must pass |

### Behavior P5: A skill is imported without typing a server path

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.5` |
| Source identity | as P1 |
| Production entry point | Skills page, Import dialog, file chooser, with `skill-import/valid/SKILL.md` |
| Real collaborators | runtime skills API, browser file chooser (`setInputFiles`), skill store |
| Boundary exercised | the import endpoint; `GET /api/uar/skills` |
| Observable result | The dialog shows name `pick-import-fixture`, description and version 1.0.0 before import; after confirming, `GET /api/uar/skills` lists skill id `pick-import-fixture` with version 1.0.0 |
| Negative control | `skill-import/invalid/SKILL.md` shows the server's `skill_markdown_invalid` reason and `GET /api/uar/skills` gains no skill; importing the valid file a second time shows the conflict (`skill_id_conflict`) and leaves the first skill unchanged |
| Isolation | scratch skill store; fixtures from this change's `fixtures/skill-import/` |
| Prerequisites | decision D5 recorded by the trusted-host owner; otherwise BLOCKED |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/skill-import.spec.ts`, 3 samples |
| Evidence | pending |
| Limitations | References, scripts and assets of a skill directory are not carried by content import. Nondeterministic: 3 samples, all must pass |

### Behavior P6: Multi-value fields are chosen, not parsed

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.6` |
| Source identity | as P1 |
| Production entry point | Skill preferred tools, agent allowed tools, retryable status codes and the memory agent filter |
| Real collaborators | runtime agents, skills, tools and settings APIs, browser |
| Boundary exercised | HTTP save and read-back |
| Observable result | The status-code options equal the frozen `httpStatusCodes` list. The memory agent filter options contain the two agents the harness seeds with frozen ids `pick-agent-a` and `pick-agent-b`, and every option is an id present in `GET /api/agents`. The tool options are a subset of the names in `GET /api/tools`. Saved selections equal the frozen selections (`pick-agent-a` filter; status codes 429 and 503; tool names chosen from the first two options in `GET /api/tools` sorted order) |
| Negative control | Status codes `99` and `600` are not offered and cannot be added; a tool name that is not in `GET /api/tools` cannot be selected |
| Isolation | as P1, with the two seeded agents and at least two tools present |
| Prerequisites | as P1. Why runtime truth is used for the tool and agent lists: the built-in tool set is defined by compiled code and feature flags, and no committed file lists it, so the option sets are bounded on one side by frozen values (the seeded agents, the status list) and on the other by the runtime's own lists. This is a stated exception to the frozen-truth rule |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/multi-value-controls.spec.ts`, 3 samples |
| Evidence | pending |
| Limitations | A picker that drops a legitimate tool is not caught by the subset check. Nondeterministic: 3 samples, all must pass |

### Behavior P7: Resilience, governance, memory and rag numbers are bounded

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-schema-controls-and-pickers#9.7` |
| Source identity | as P1, plus the digest of `src/uar/settings/manager.rs` recorded when the fixture was written (a changed digest makes the fixture stale: BLOCKED) |
| Production entry point | Resilience, intent classifier, rag and memory settings panels |
| Real collaborators | runtime settings API, browser |
| Boundary exercised | settings save and read-back |
| Observable result | Each of the 18 rows in `numeric-bounds-oracle.json` is a slider or spinbutton whose minimum equals the frozen minimum and whose maximum equals the frozen maximum where one is declared. The five frozen in-range saves (`resilience.retry_max_attempts` 3, `resilience.request_timeout_ms` 45000, `memory.vector_weight` 0.6, `rag.chunk_size` 2048, `intent_classifier.topk` 7) read back as exactly those values from `GET /api/uar/settings/<namespace>`. The expected bounds and values are frozen in the fixture; the live types response is never an expected value |
| Negative control | Each frozen out-of-range input (`retry_max_attempts` 11, `retry_backoff_multiplier` 5.1 and 1.0, `request_timeout_ms` 999, `burst_size` 0, `accept_threshold` 1.1, `topk` 0, `chunk_size` 63, `vector_weight` 1.1) is not enterable or leaves Save disabled with the range stated, and the stored value is unchanged |
| Isolation | as P1 |
| Prerequisites | as P1 |
| Final local gate | same spec file as P1 |
| Evidence | pending |
| Limitations | The per-agent override numbers (governance-settings-panels.tsx:828-883) and the knowledge-base default chunk size have no frozen bound (`notFrozen` in the fixture); the runner reports whether they are bounded controls but does not assert values. Nondeterministic: 3 samples, all must pass |

## Scenario matrix rows

For every row: fixtures are the frozen files above plus a scratch data directory and scratch port
with the stub model server; restoration is to stop the runtime and stub, delete the scratch
directory, close the browser context and remove any baseline worktree. All rows are
nondeterministic (rendering and timing) and carry a stochastic block. Owner tasks refer to
`tasks.md` of this change.

| Field | SCN-PICK-001 | SCN-PICK-002 | SCN-PICK-003 | SCN-PICK-004 |
|---|---|---|---|---|
| name | Array, number and object settings use structured controls | Model picker is searchable and keyboard operable | Embedding and provider fields accept only listed values | No unlisted free-text control remains |
| actor | Operator editing settings | Operator choosing a model | Operator choosing a provider or embedding model | Maintainer running the sweep |
| useful outcome | No CSV, raw JSON or bare number for editable fields | Fast, searchable choice from 597 models | A typo cannot become a provider id | Regressions to free text are caught |
| source requirement | `openspec:frontend-configuration-surfaces#Array, object and numeric settings use structured controls` | `openspec:frontend-configuration-surfaces#Provider and model choices use one picker family` | `openspec:frontend-configuration-surfaces#Provider and model fields do not accept typed values` | `openspec:frontend-configuration-surfaces#Enumerable settings values are never free text` |
| reason now | `generic-schema-panel.tsx:209-293` renders by JSON type | two pickers, an unsearchable Add Model select | embedding fields are `x-control: text` | 56 text inputs in settings alone (assessment G7) |
| likelihood | likely | UNKNOWN | likely | likely |
| mandatory | true | true | true | true |
| nondeterministic | true | true | true | true |
| stochastic | samples 3; grader: all 11 fields have the frozen kind and values round-trip; stoppingRule fixed N, all must pass | samples 5; grader: counts 597/44/54/28/85 and max long task under 200 ms; stoppingRule fixed N, all must pass | samples 3; grader: options equal the union and typed non-options select nothing; stoppingRule fixed N, all must pass | samples 3; grader: bijection with the inventory and options equal their sources; stoppingRule fixed N, all must pass |
| production entry | UI settings sections | UI Add Model dialog | UI credentials and settings | production UI, all audited surfaces |
| real collaborators | runtime settings API, browser | runtime catalog, browser | runtime settings, credentials, catalog and providers APIs, browser | runtime, bundle, browser, option endpoints |
| oracle | observation: control roles, bounds, saved values; truth source: `schema-controls-oracle.json` (frozen) and `GET /api/uar/settings/<ns>`; comparator: equality | observation: option counts and order, long task; truth source: `picker-oracle.json` (frozen) and the 200 ms threshold; comparator: equality, below threshold | observation: option set and stored values; truth source: `provider-options.json` (343 ids from the catalog snapshot) and the frozen choices and expected stored values; comparator: set equality and value equality (API only reads back) | observation: textbox list, roles, option sets; truth source: `free-text-inventory.json` (frozen) and option endpoints; comparator: bijection and set equality |
| negative control | input: out-of-range values and invalid JSON; expected: rejected, stored value unchanged | input: no-match query; baseline bundle; expected: empty state, baseline fails the search check (else BLOCKED) | input: typed `not-a-provider`; expected: nothing selected, no credential created, knowledge-base value unchanged | input: baseline bundle; expected: at least 10 of 12 must-not-be-textbox entries violated (else BLOCKED) |
| fixtures | `schema-controls-oracle.json` | `picker-oracle.json`, `picker-config.yaml` | `provider-options.json`, scratch dir | `free-text-inventory.json` |
| evidence required | report, API responses | report, long-task list, option dump | report, API responses | report, baseline violation list, inventory digest |
| substituted boundaries | none | none | none | none |
| limitations | 11 fields | below the virtualization point | embedding half BLOCKED on D4 | name-based matching |
| invalidation keys | `features/settings/ui/schema-controls/**`, `generic-schema-panel.tsx`, `src/uar/settings/manager.rs` schemas | `components/ui/**` picker, `features/models/**`, `catalog/provider_catalog.json` | `settings-panel-registry.tsx`, embedding schema fields, D4 source | `free-text-inventory.json`, any audited surface, `nav-destinations.ts` |
| owner tasks | `#9.1` with 3.x, 4.x | `#9.2` with 1.x, 2.x | `#9.3` with 5.x | `#9.4` with 5.x, 6.x |

| Field | SCN-PICK-005 | SCN-PICK-006 | SCN-PICK-007 |
|---|---|---|---|
| name | Skill import without a typed server path | Multi-value fields are chosen | Resilience and governance numbers are bounded |
| actor | Operator importing a skill | Operator editing tools, agent filter, status codes | Operator tuning retries and budgets |
| useful outcome | Import works with a file chooser | Values come from catalogs, not parsed strings | Out-of-range values cannot be saved |
| source requirement | `openspec:frontend-configuration-surfaces#Skill import does not require a typed server path` | `openspec:frontend-configuration-surfaces#Enumerable settings values are never free text` | `openspec:frontend-configuration-surfaces#Array, object and numeric settings use structured controls` |
| reason now | the dialog posts a typed absolute path (`skill-import-dialog.tsx:119`) | CSV at `skills-page.tsx:384`, `governance-settings-panels.tsx:796`, `resilience-settings-panel.tsx:430` | bare number inputs across the resilience, memory, rag and intent classifier panels (18 frozen in numeric-bounds-oracle.json) |
| likelihood | UNKNOWN | likely | likely |
| mandatory | false (blocked on D5) | true | true |
| nondeterministic | true | true | true |
| stochastic | samples 3; grader: valid imports once, invalid and duplicate rejected; stoppingRule fixed N, all must pass | samples 3; grader: options equal sources and saved arrays equal choices; stoppingRule fixed N, all must pass | samples 3; grader: bounds equal schema and out-of-range rejected; stoppingRule fixed N, all must pass |
| production entry | UI skills import dialog | UI skills, agents, resilience, memory | UI resilience, intent classifier, memory panels |
| real collaborators | runtime skills API, browser file chooser | runtime tools, agents, skills and settings APIs | runtime settings API |
| oracle | observation: preview and listed skill; truth source: `skill-import/valid/SKILL.md` front matter (frozen) and `GET /api/uar/skills`; comparator: field equality | observation: options and saved selections; truth source: frozen status list, frozen seeded agents `pick-agent-a` and `pick-agent-b`, and the runtime tool list as a stated bound (the built-in tool set has no committed source); comparator: equality, subset | observation: control bounds and read-back values; truth source: `numeric-bounds-oracle.json` (18 rows, source digest pinned) and its five frozen saves; comparator: equality (live types response is never an expected value) |
| negative control | input: `invalid/SKILL.md` and a second import of the valid file; expected: `skill_markdown_invalid` shown, then `skill_id_conflict` shown, no extra skill | input: status 99 and 600, an unknown tool; expected: not offered, not selectable | input: frozen out-of-range values (11, 5.1, 999, 0, 1.1, 63); expected: not enterable, range stated, stored value unchanged |
| fixtures | `skill-import/**` | scratch dir with one agent and two tools | `numeric-bounds-oracle.json`, scratch dir |
| evidence required | report, API responses | report, API responses | report, API responses, list of keys without a schema bound |
| substituted boundaries | none | none | none |
| limitations | directory assets not carried; BLOCKED until D5 | tool list depends on the scratch runtime | per-agent override numbers and knowledge-base chunk size are reported, not asserted |
| invalidation keys | `skill-import-dialog.tsx`, `skills-api.ts`, `src/uar/api/skills.rs`, `skill-import/**` | `skills-page.tsx`, `governance-settings-panels.tsx`, `resilience-settings-panel.tsx` | `resilience-settings-panel.tsx`, `governance-settings-panels.tsx`, schemas |
| restoration | scratch skill store removed | scratch dir removed | scratch dir removed |
| owner tasks | `#9.5` with 7.x | `#9.6` with 4.x, 6.x | `#9.7` with 4.x |

## Risks and trade-offs

- **Registry fallback hides a new backend kind.** An unknown kind renders by JSON type, so a
  kind that needs a special control looks acceptable and is wrong. The changed-path inventory
  lists the three declared kinds left on fallback (`pricing-table`, `model-card`,
  `capabilities-grid`).
- **Reduced editing for embedding fields (D4).** Until an option source exists the three
  embedding fields become read-only. That is a regression for operators who change them.
- **Skill import loses directory assets (D5).** Option A trades references, scripts and assets
  for removing the typed path.
- **Unit heuristic.** Reading units from titles is fragile; an `x-unit` hint from the backend
  replaces it.
- **The uncomfortable part.** This change makes the console stricter than the backend: the
  schema still lets any string through for `llm.embedding.model`. The picker is a UI guard,
  not validation. A direct API caller can still store a bad value, and no server-side check is
  specified here.

## Unresolved decisions for the lead and operator

1. D4: embedding option source (recommended: server-declared option list; owners state and
   runtime). Operator may instead keep typed entry for three fields and keep them in the inventory as textboxes.
2. D5: authority class of `import-content`, to be confirmed by `uar-trust-tools`; fallback is
   option D.
3. Plan assigns the single writer for `generic-schema-panel.tsx` (SET and this change).
4. Whether a backend `x-unit` hint and `x-options-source` hint are added (state and runtime
   owners; not edited here).
5. Whether the 200 ms picker budget is accepted or replaced.
6. Decide D4 (embedding option source) before Plan: until then three in-use embedding fields are BLOCKED and would be read-only, which removes a control operators use today.
7. The 8,170-option picker scale check needs a surface that mounts `CatalogModelPicker` (the system default control). Name that scenario in `uiux-system-model-live-default`, or the virtualization requirement has no scenario.
8. Contract level (lead): one disposition per overlapping open change; this change says `migrate-settings-mutations-and-retire-stores` is "continue, narrowed", the design and persistence proposals differ.
