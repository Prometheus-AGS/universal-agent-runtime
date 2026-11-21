## Context

Phase `uar-uiux-architecture-2026-11`, build items 6 (Impeccable, PRODUCT.md, DESIGN.md, critique
gate) and 23 (Settings regroup, deep links, progressive disclosure). Binding decisions of
2026-11-19: redesign authorized; `docs/design/` is the visual authority and the source of
`DESIGN.md`; `docs/UI_DESIGN.md` retired; `docs/ui-design-authority.md` stays in force where it
does not conflict, including the Base UI exception D1; Settings grouped into about seven areas;
`PRODUCT.md` may be rewritten.

Method: source and document reading only; nothing built or run. The Impeccable CLI was not
invoked (it is an execute-stage step).

### Verified here

| Claim | Evidence (read) |
|---|---|
| PRODUCT.md says "does not redesign the application" | `frontend/PRODUCT.md`, section Operating Context |
| No DESIGN.md in the frontend | `ls frontend/DESIGN.md`: no such file |
| `IMPECCABLE_BIN` is the 0.1.17 engine in operator settings | `~/.claude/settings.json:6` (read only) |
| `docs/UI_DESIGN.md` is "Material 3 Flat 2.0" | `docs/UI_DESIGN.md:1-8` |
| `docs/admin-aesthetic-spec.md` is "Terminal / CRT", status "authoritative" | `docs/admin-aesthetic-spec.md:1-12` |
| `docs/ui-design-authority.md` orders operator decisions, phase goals, "delivered design comps", the KnowMe standard; it names neither `docs/design/` nor the redesign decision | `docs/ui-design-authority.md:5-14` |
| Seven settings pages in the prototype route table plus a disabled General | `docs/design/assets/settings.js:61-70`; `docs/design/README.md` Screens table |
| Brand tokens, type, rules | `docs/design/brand-spec.md` (full file) |
| Shell tiers, patterns, routes | `docs/design/react-implementation-plan.md` sections 2-7 |
| Tokens already carry Inter body, Space Grotesk display | `frontend/src/shared/theme/tokens.css:12-14` |
| Settings has 30 nav items, state in `useState("provider")` | `settings-navigation.tsx`; `settings-page.tsx:12` |
| `AdvancedSection` used twice | `ai-settings-panels.tsx:346`; `resilience-settings-panel.tsx:339` |
| "Runtime settings" destination claims seven path prefixes | `app/shell/nav-destinations.ts` |
| Configure hub lists `CONFIGURE_DESTINATIONS` | `app/shell/mobile-sheet-host.tsx:51-53` |
| Routes: `/threads`, `/admin/*` (one lazy page keyed by path segment), `/about` | `frontend/src/App.tsx:37-69`; `pages/admin-page.tsx:32-52` |
| The critique gate is already a requirement | `openspec/specs/agent-ui-design-workflow` ("UI design decisions receive independent critique") |

## Goals / Non-Goals

**Goals**

- A design record (`DESIGN.md`) that says what the console should look like, derived from the
  authority the operator named, and a product record (`PRODUCT.md`) that no longer forbids the work.
- One URL-addressable Settings with about seven areas; every retained setting reachable once.
- Progressive disclosure that measurably cuts what an operator sees first.

**Non-Goals**

- The single-sidebar-plus-inspector shell (see proposal, Unresolved decision 2).
- Wiring or changing any control's behavior (the status, picker and boot-safety changes own that).
- Moving the Impeccable engine pin.
- New Appearance controls (density, motion, text size) or a Desktop feature set beyond what exists.
- Deleting retired documents.

## Decisions

### D1. Order of work: design record first, regroup second, critique last

1. Read-only: `impeccable context` to learn which directory it resolves for PRODUCT.md and
   DESIGN.md (UNVERIFIED: PRODUCT.md sits in `frontend/`, the `.impeccable/` directory at the
   repository root). Write each file where the CLI resolves it.
2. `impeccable update`, then `impeccable doctor`. If doctor reports an engine/skill version
   mismatch or a stale directive that needs a different engine, stop, record it, and ask the
   operator. Do not edit `~/.claude/settings.json`, `IMPECCABLE_BIN`, or any harness setting.
3. `impeccable init` to rewrite PRODUCT.md (operator interview; see D2).
4. Write DESIGN.md and its sidecar from `docs/design/` (D3).
5. Run `impeccable document` in scan mode **only** to list drift between DESIGN.md and the
   shipped code; keep DESIGN.md as written (D4).
6. Update authority documents (D5).
7. Regroup Settings (D6-D8).
8. Critique gate over the artifacts and the diff (D9).

Steps 1-5 need no implementation of the regroup, so a designer's evidence (DESIGN.md) exists before
the diff it judges.

### D2. PRODUCT.md: remove the contradiction, keep the product truth

Facts to keep (all confirmed in the current file): the platform is web; the product is governed
agent execution; an assignable Presentation is a reusable UI template, separate from the
development-only A2UI tester; presentation management persists records and supports assignment to
governed runs; existing clients keep their current rendering unless they negotiate otherwise;
business entities and drafts live in the normalized entity graph behind typed domain hooks; agent
kernels do not write, the trusted host owns mutations; a template is declarative UI, not
executable code; no commercial claims or synthetic usage statistics are authorized.

Facts to change: Operating Context states that the work extends the console and does not redesign
it. Replace with the 2026-11-19 decision: the console is being redesigned; `docs/design/` is the
visual authority; dev tooling stays dev-only. `impeccable init` interviews the operator; the answer
mechanism must be probed before inferring (init reference). Any fact inferred without an answer is
labelled inferred. Init does not write visual direction, so the PRODUCT.md change carries no
palette or type.

### D3. DESIGN.md is written from `docs/design/`, not extracted from code

Sources and what each supplies:

| DESIGN.md section | Source in `docs/design/` |
|---|---|
| Frontmatter `colors` | `brand-spec.md` six core tokens and extended ladder, in the OKLCH values the file gives (canonical format: OKLCH; state colors and the light scene kept as listed) |
| Frontmatter `typography` | `brand-spec.md` Type: Space Grotesk display, Inter body and UI, JetBrains Mono for measurement |
| Frontmatter `rounded`, `spacing` | `brand-spec.md` rule 5 (12px cards and panels, 8px controls, pill only on small status chips); spacing from `assets/shell.css` (values read at write time; none invented) |
| Overview | `brand-spec.md` one-sentence definition; `ui-research-findings.md` section 3 for the shell target |
| Colors, Named Rules | `brand-spec.md` rules 1, 2, 6 (four-step ladder, one accent two jobs, reserved phase colors) |
| Typography, Named Rules | rule 4 (mono means measurement) |
| Elevation & Depth | rule 1: regions separate by fill, not stroke; the documented exception for data-viz axes; the high-contrast theme as the one stroke-drawing theme (`react-implementation-plan.md` section 3) |
| Layout | `react-implementation-plan.md` section 5 (272px sidebar, 60px rail, inspector 360-760px, 760px and 1180px breakpoints) and `ui-research-findings.md` section 3 |
| Shapes | rule 5; focus ring 2px ember with a 2px canvas gap |
| Components | patterns in `react-implementation-plan.md` section 4 (Button, StatusChip, RuleRow, MasterDetail, SummaryStrip, Wizard, InlineConfirm, EmptyState); component CSS in `assets/shell.css` and `assets/settings.css` |
| Do's and Don'ts | Flat 2.0 contract (no borders, rings, layout shadows, blur, gradients, outline variants), status never by colour alone, mono only for measurement |
| Sidecar `.impeccable/design.json` | Motion tokens (120/200/320 ms; `cubic-bezier(0.2,0,0,1)` and `cubic-bezier(0.16,1,0.3,1)`), shadow vocabulary (none), breakpoints, full component snippets |

Two tensions the file must state rather than hide:

- `brand-spec.md` says the shipped frontend "loads Geist and switches in the first implementation
  PR". `tokens.css:12-14` already loads Inter. DESIGN.md records Inter as the standard; the doc
  note in `brand-spec.md` is stale and is corrected in the authority update (D5).
- `--border` is `transparent` in the default themes, while 327 `border` and 123 `border-border`
  classes remain (assessment G9) and the high-contrast theme depends on a visible `--border`
  (`index.css:166`). DESIGN.md states "Flat 2.0, with an explicit high-contrast stroke exception".

The Creative North Star: the `brand-spec.md` one-sentence definition is used verbatim as the
Overview opening. A named metaphor is not invented; the operator supplies one or the file labels
it provisional.

### D4. `document` is a drift check

After DESIGN.md exists, run `impeccable document` in scan mode and record the differences it finds
between DESIGN.md and the shipped tokens and components (for example token values in
`tokens.css`, the 327 `border` classes, uncovered components). The output is a work list under
`.kbd-orchestrator/phases/<phase>/` for later changes. DESIGN.md is not regenerated or edited to
match the code. If `document` offers to overwrite, the answer is no. The sidecar is regenerated only
together with DESIGN.md.

### D5. Authority documents

`docs/ui-design-authority.md`:

- Add `docs/design/` (and the `DESIGN.md` derived from it) to the authority order directly after
  the operator decisions and phase goals, above the KnowMe standard for visual matters.
- Add a decision row for the 2026-11-19 redesign (scope: console redesign, `docs/design/` visual
  authority, Settings in about seven areas, one sidebar plus inspector as the target shell) with
  its rationale reproduced, per the document's own portability rule, and the KBD path as plaintext
  provenance only.
- List the retired documents. D1 (Base UI) is untouched.

`docs/UI_DESIGN.md` and `docs/admin-aesthetic-spec.md`: add a first-paragraph retirement notice
naming `docs/design/` and the date. No other edit. `docs/design/brand-spec.md`: correct the Geist
sentence (the only edit in `docs/design/`; the README says copies come from the design project, so
the edit is also queued for that project, UNVERIFIED how the operator syncs it).

### D6. One route table, seven areas

A single module `frontend/src/features/settings/model/settings-routes.ts` declares
`area -> sections` with: label, icon, path, and either a settings namespace key or a page
component. Settings navigation, the command palette, breadcrumbs and redirects derive from it. The
React Router tree gains `/settings/*`; `SettingsLayout` reads `:area` and `:section` from the URL
instead of `useState`.

| Area (`/settings/<area>`) | Sections (`/<section>`) | Source |
|---|---|---|
| `models` Models & providers | `providers` (Provider Overrides editor and catalog), `models` (Models page), `credentials`, `llm-defaults` (`llm` panel: embedding, cost, cache, budget; whichever keys the status change leaves visible), `model-files` (`models`), `routing` (`resilience`, `intent_classifier`), `context` (`context_strategy`), `caching` (`prompt_caching`) | `provider`, `llm`, `models`, `resilience`, `intent_classifier`, `context_strategy`, `prompt_caching` namespaces; Providers, Models, Credentials pages |
| `tools` Tools & MCP | `tools`, `mcp-health`, `native-tools` | Tools page, MCP health page, `native_tools` |
| `skills` Skills | `skills`, `compiler` | Skills page, Compiler page; `skill_config` is a link row (status change) |
| `knowledge` Knowledge & memory | `defaults` (`rag`), `knowledge-bases` (`knowledge_bases`), `memory` (`memory` configuration), `memory-records` (Memory page) | namespaces and Memory page |
| `governance` Governance | `enforcement` (`governance`), `access-keys` (Auth page), `budget` (Cost page), `presentation-policy`, `config-status` (the Configuration status panel from `uiux-settings-truthful-status-and-removals`: boot status, drift, reload) | namespaces, pages and the status change's panel |
| `appearance` Appearance | `theme` (existing theme control), `preferences` (`user_settings`) | `components/ThemeToggle.tsx`, `user_settings` |
| `desktop` Desktop | none until a Tauri-only control exists | shown only when `isTauri()` is true and a section exists |

`agent_config` is a link to the Agents work page. The 14 removed namespaces appear nowhere.
Redirects (client-side `Navigate replace`): `/admin/settings` to `/settings`; `/admin/providers`,
`/admin/models`, `/admin/credentials` to their `models` sections; `/admin/tools`,
`/admin/mcp-health` to `tools`; `/admin/skills`, `/admin/compiler` to `skills`; `/admin/memory` to
`knowledge/memory-records`; `/admin/auth`, `/admin/cost` to `governance`. Work pages
(`/admin/knowledge`, `agents`, `teams`, `runs`, `presentations`, `approvals`, `runtime`,
`protocols`) keep their URLs. Unknown area redirects to `/settings/models`; unknown section
redirects to the area's first section; `desktop` outside Tauri redirects to `models`.

Protocols moves to the Runs side: `protocols`, `approvals` and `runtime` join the Runs
destination's match set. This supersedes the interim relink of `protocols` in
`uiux-settings-truthful-status-and-removals` (design D8) and resolves its open question.

Unresolved here: how the Provider Overrides editor and the Providers catalog page share the
`models/providers` section. Default: the section is the Providers master-detail page, and the
Provider Overrides editor (certified by six existing requirements) is its Connection editor,
reused without change.

### D7. Primary navigation

`nav-destinations.ts`: the `providers`, `mcp-tools`, `skills` and `runtime-settings` destinations
are replaced by one `settings` destination (`/settings`, `matchPaths: ["/settings"]`), keeping the
`presentations` destination in the Configure group and the dev-only A2UI destination. The `runs`
destination gains `/admin/runtime`, `/admin/approvals`, `/admin/protocols` in `matchPaths`.
Consequences already handled by existing code shape: the rail, the compact hub
(`mobile-sheet-host.tsx`) and the palette read the inventory, so they follow. The compact Configure
hub therefore lists Settings and Presentations (and A2UI in development); the Settings page
provides its own area navigation at compact width. Existing requirement
`frontend-app-shell#Configure uses the shared compact sheet host` names the five old hub items in
a scenario and is modified here.

The command palette adds one entry per section (from the route table), so "Providers" and "Cost"
remain one keystroke away.

The shell mounts `app/shell/boot-status-banner.tsx` (created by the settings status change) above the
route content on every page. The Settings header exposes a slot above each area's content for a status strip. The system-default
picker from `uiux-system-model-live-default` mounts in the `models` area's slot, which is how that
change satisfies "reachable from primary navigation" (decision 2).

### D8. Progressive disclosure

Rule: a section's first render shows the controls an operator needs for that section's main job;
everything else sits under one "Advanced (n)" disclosure using the existing `AdvancedSection`.
Proposed measurable limit: at most 8 visible controls on first render per section (operator
confirms the number). Mechanics:

- Schema-driven panels read an `x-uar-ui-advanced: true` field marker from the settings-type schema
  (additive; same namespace of markers as `x-uar-ui-hidden` from the status change).
- Bespoke panels wrap their own advanced groups with `AdvancedSection`.
- A raw JSON editor, where one remains after the picker change (an object with no declared
  properties), appears only inside Advanced and is labelled as such. Comma-separated text fields do not
  exist after that change and are not handled here.
- Disclosure state is not persisted across sessions.

An audit task assigns each retained control primary or advanced and records the result so a reviewer
can see what was demoted. The audit is a work product, not an oracle: SCN-DSN-003 counts controls in
the rendered DOM against the frozen limit and never reads the audit. Audit result for any panel is
UNKNOWN until the task runs.

### D9. Critique gate

Per the `agent-ui-design-workflow` requirement and the project UI rule: run Impeccable critique
twice with isolated subagents (A: design assessment, B: detector assessment, as in the existing
`.impeccable/critique/` files, which record `total_score`, `p0_count`, `p1_count`), then one
fresh-context adversarial review that sees only the artifact: `DESIGN.md` + sidecar + PRODUCT.md
as one artifact, and the Settings regroup as a diff. Results are new files under
`.impeccable/critique/`. P0 findings block acceptance; P1 findings are resolved or recorded with an
operator decision. The reviewers receive the artifact, not the generation history. Pass criteria are
counted by the runner from the reviewers' finding lists under `fixtures/critique-gate.json`; a file's
own `p0_count` header is not read, because an author's or critic's summary of itself is not independent
evidence.

## Risks / Trade-offs

- **DESIGN.md claims what the code does not do.** It describes the target. The drift list is the
  honest record of the gap, and DESIGN.md's Overview states that the code lags.
- **Folding the Configure group changes muscle memory and deep links.** Redirects cover old URLs;
  the compact hub changes shape. The two e2e specs and unit tests that assert the old inventory
  change with it.
- **Skill update without engine move.** A newer skill pack may call verbs the 0.1.17 engine lacks.
  D1 step 2 stops rather than works around it.
- **Eight-control limit is arbitrary.** It is a proposal for the critique to challenge, not a
  derived number.
- **The uncomfortable part.** The regroup rearranges a surface whose controls are being removed,
  retyped and reclassified by three other changes in the same phase. If any of them slips, this
  change's route table lists a section whose content is half-migrated. The route table is data, so a
  slipped section can be hidden by one entry; the acceptance scenario for completeness (SCN-DSN-002)
  fails loudly instead of passing on a smaller inventory.

## Frozen oracle fixtures

Written before implementation. They are the expected-value sources for the scenarios below. No
implementer edits them and no scenario reads an expected value from a route table, audit file or
critique summary written during the phase (rule R1). Paths are relative to this change.

| File | Used by | Content |
|---|---|---|
| `fixtures/settings-routes.json` | SCN-DSN-001, 002, 008 | The seven areas and their sections, the namespace-to-section map, link-outs, the removed namespaces, every redirect, the fallback controls, the palette entries, the primary-navigation matches and deep-link samples. Truth sources: decision 5, `docs/design/README.md`, `assets/settings.js:61-70`, and the frozen lists in `uiux-settings-truthful-status-and-removals/fixtures/` |
| `fixtures/section-controls.json` | SCN-DSN-003 | The 8-control limit, the counting selector and rules, the Advanced disclosure pattern, the over-limit control |
| `fixtures/critique-gate.json` | SCN-DSN-007 | The artifact input lists, forbidden reviewer inputs, the finding-marker counting rule, the pass rule, the isolation check |
| `fixtures/design-context.json` | SG-DSN-004..006 | Brand token values copied from `docs/design/brand-spec.md`, PRODUCT.md and authority-document strings |

## Acceptance

Common prerequisites for the UI scenarios: the console bundle served by an isolated UAR instance on a
lead-assigned port, scratch data directory, behind the existing external identity proxy, with
`openspec/changes/uiux-settings-truthful-status-and-removals/fixtures/scratch-config.yaml` as its YAML;
no credentials in any step (operator rule 2026-10-26); Chromium via Playwright. No scenario here calls
a model; where the instance needs a provider it points at the local `stub-llm` (rule R4). The
supplemental gates need the Impeccable CLI at the pinned engine; the lead names its invocation (rule
R5). The runner-integrity baseline is lead-owned and lives under `~/.claude/worktrees/`. Gate commands
are proposals. Evidence is pending for every row.

### B1. Settings sections are addressable by URL

| Field | Declaration |
|---|---|
| Canonical identity | `uiux-design-context-and-settings-regroup#3.1`, `#3.2`; SCN-DSN-001 |
| Production entry point | Browser at each `deepLinkSamples` URL, each `redirects[].from` URL, each palette entry |
| Real collaborators | Console bundle, UAR instance (the settings, cost and memory endpoints) |
| Boundary exercised | Router, command palette, browser history |
| Observable result | For every sample URL a cold load ends at that URL and shows the section heading; reload keeps it; back and forward move between two visited sections; each palette entry in the fixture navigates to its section URL; each redirect lands exactly on its `to` URL with the target page's heading rendered |
| Negative control | The four `fallbackControls`: `/settings/nope`, `/settings/models/nope`, `/settings/desktop` in a browser, and `/admin/nonexistent` end at the fixture's `expectedFinal` or render the cockpit test id. An URL the fixture does not list, `/settings/models/providers/extra`, renders the section without an error page |
| Isolation | Scratch instance and port; one browser profile per sample |
| Prerequisites | Regroup implemented; the status change's relinked pages present |
| Final local gate | Proposed: `pnpm -C frontend exec playwright test -c playwright.installed-settings-areas.config.ts -g SCN-DSN-001` (new config, execute stage) |
| Evidence | Pending |
| Limitations | Tauri desktop host not exercised |

### B2. Every retained setting is reachable exactly once

| Field | Declaration |
|---|---|
| Canonical identity | `...#3.1`, `#3.3`; SCN-DSN-002 |
| Production entry point | Settings navigation walked by the runner |
| Real collaborators | Console bundle; `GET /api/uar/settings/types` from the instance |
| Boundary exercised | Rendered navigation versus the instance's type list |
| Observable result | The set of namespaces from `/types` minus `removedNamespaces` equals the keys of `namespaceToSection` plus `linkOuts` plus the namespaces the fixture lists as pages; each is found in exactly the fixture's section; no namespace appears in two sections; every removed namespace appears nowhere |
| Negative control | A scratch console whose route table omits `rag` fails the comparison naming `rag`; a scratch console that lists `sandbox` fails naming `sandbox` |
| Isolation | Scratch instance; the scratch consoles are built from a copy of the source in the scratch directory |
| Prerequisites | Status change implemented (the removed list is final) |
| Final local gate | Proposed: `... -g SCN-DSN-002` |
| Evidence | Pending |
| Limitations | Pages (Providers, Cost and the like) are matched by heading, not by namespace |

### B3. Settings shows less first

| Field | Declaration |
|---|---|
| Canonical identity | `...#3.4`; SCN-DSN-003 |
| Production entry point | Each section in `fixtures/settings-routes.json` on first render, then Advanced opened by keyboard |
| Real collaborators | Console bundle and instance |
| Boundary exercised | Rendered DOM, counted by the rule in `fixtures/section-controls.json` |
| Observable result | The runner's count of visible controls is at most 8 for every section before Advanced is opened; the Advanced button matches the label pattern, opens with Enter, sets `aria-expanded` and reveals further controls in DOM order |
| Negative control | The scratch console with a section forced to render 9 visible controls fails, naming that section |
| Isolation | Scratch instance |
| Prerequisites | Regroup complete |
| Final local gate | Proposed: `... -g SCN-DSN-003` |
| Evidence | Pending |
| Limitations | Counts controls; whether the right ones are primary is a critique judgment (SCN-DSN-007) |

### B7. The critique gate ran with independent reviewers

| Field | Declaration |
|---|---|
| Canonical identity | `...#4.1`-`#4.3`; SCN-DSN-007 |
| Production entry point | Impeccable critique through two isolated subagents, then a fresh-context adversarial reviewer, per artifact in `fixtures/critique-gate.json` |
| Real collaborators | Impeccable skill, independent reviewer agents, the built artifacts |
| Boundary exercised | Reviewer packet manifests; files under `.impeccable/critique/` |
| Observable result | For each artifact: two critique files and one adversarial review file exist; the runner, not the files' headers, counts `[P0]` and `[P1]` markers under Priority Issues; P0 count is 0 in every file; each P1 appears in a resolution file with an operator decision id |
| Negative control | A reviewer packet whose manifest lists `.kbd-orchestrator/phases/` or any other forbidden input is rejected and its review does not count; a critique file whose body has a `[P0]` marker while its header says `p0_count: 0` fails the count |
| Isolation | Reviewers receive only the manifest's files |
| Prerequisites | Design context and regroup complete |
| Final local gate | Proposed: the lead names the invocation |
| Evidence | Pending |
| Limitations | Reviewers are model agents; same model family unless the external judge is available, recorded as `cross_model_check` |

### B8. Settings works at narrow width and in high contrast

| Field | Declaration |
|---|---|
| Canonical identity | `...#3.5`; SCN-DSN-008 |
| Production entry point | The seven areas' first sections at 320, 768, 1024 and 1440 px in dark, light and high-contrast themes |
| Real collaborators | Console bundle, instance, axe checks in the existing accessibility Playwright config |
| Boundary exercised | Layout and accessibility tree |
| Observable result | No horizontal page scroll at any width; area navigation operable by keyboard; zero axe violations; in high contrast the stroke exception renders visible borders; every status shown carries text |
| Negative control | A scratch console with a forced 400 px wide element at 320 px width fails the overflow check |
| Isolation | Scratch instance |
| Prerequisites | Regroup complete |
| Final local gate | Proposed: `pnpm -C frontend exec playwright test -c playwright.accessibility.config.ts -g "settings areas"` |
| Evidence | Pending |
| Limitations | Screen-reader speech is not exercised |

## Supplemental gates (not acceptance)

SCN-DSN-004, 005 and 006 check files and CLI output, not product behavior. The spec packet says
structural checks cannot establish acceptance, so they are classified `supplemental`: they run in the
final batch, are reported separately, and a failure never cascades into a product-failure claim. No
operator exception is requested. Expected values are in `fixtures/design-context.json`.

| Gate | Check | Negative control |
|---|---|---|
| SG-DSN-004 (SCN-DSN-004) | `impeccable context` loads PRODUCT.md and DESIGN.md with no missing or stale directive; every colour, radius, font and motion value in DESIGN.md and the sidecar equals the fixture; DESIGN.md is byte-identical before and after `impeccable document` | Scratch copy with one token changed makes the comparison report that token |
| SG-DSN-005 (SCN-DSN-005) | PRODUCT.md lacks every `productMdMustNotContain` string and contains every `productMdMustContain` string | The pre-phase PRODUCT.md fails the first check |
| SG-DSN-006 (SCN-DSN-006) | `docs/ui-design-authority.md` contains every `authorityDocMustContain` string; both retired documents open with a notice containing `docs/design`; no document outside the retired two and `docs/design/` cites them as authority | The pre-phase tree fails each check |

## Scenario matrix rows

All rows: substituted boundaries none; restoration is to stop the scratch instance, delete the scratch
directory and any scratch console build, release the port. Files under `docs/` and `frontend/` are the
product and are never restored by a scenario. The runner reads expected values only from `fixtures/`.

| Field | SCN-DSN-001 | SCN-DSN-002 | SCN-DSN-003 | SCN-DSN-007 | SCN-DSN-008 |
|---|---|---|---|---|---|
| Name | Settings sections have URLs | Every retained setting is reachable once | Settings shows less first | Critique gate ran independently | Settings at narrow width and high contrast |
| Actor | Console operator | Console operator | Console operator | Phase reviewer | Operator on a small screen or in high contrast |
| Useful outcome | Can link, reload and navigate back to a section | No setting is lost or duplicated by the regroup | The first screen of a section is short | Independent evidence for the new design | The regroup stays usable and accessible |
| Source requirement | `openspec:runtime-console-ux#Settings are organized into seven areas with deep links` | same | `openspec:runtime-console-ux#Settings sections disclose advanced controls on request` | `openspec:agent-ui-design-workflow#UI design decisions receive independent critique` | `openspec:frontend-accessibility-certification#Responsive accessibility is certified across the required matrix` |
| Reason now | G9, decision 5 | G9 | G9 | Project UI rule | Web testing rules; G9 |
| Likelihood | Certain | Certain | Certain | Certain | Moderate |
| Mandatory | true | true | true | true | true |
| Nondeterministic | true | false | true | true | true |
| Stochastic | samples 3; grader: all URL, heading and history assertions hold in the sample; stoppingRule: fixed N=3, all must pass | n/a | samples 3; grader: counter and keyboard assertions hold; stoppingRule: fixed N=3, all must pass | samples 3 independent adversarial reviews per artifact; grader: runner-counted P0 is 0 and isolation check passes; stoppingRule: fixed N=3, all must pass | samples 3; grader: no overflow, no axe violation, text on every status; stoppingRule: fixed N=3, all must pass |
| Production entry | Browser URLs and palette | Rendered navigation and `/types` | Section first render | Impeccable critique and reviewer agents | Playwright at four widths and three themes |
| Real collaborators | Console bundle, instance | Console bundle, instance | Console bundle, instance | Impeccable skill, independent reviewer agents | Console bundle, instance, axe |
| Oracle: observation | Final URL and rendered heading | Set of namespaces in the navigation | Runner-computed count of visible controls | Marker counts in reviewer files; packet manifests | Overflow, axe, contrast |
| Oracle: truth source | `fixtures/settings-routes.json` | `fixtures/settings-routes.json`; `GET /api/uar/settings/types` as the namespace universe | `fixtures/section-controls.json` | `fixtures/critique-gate.json` | WCAG 2.2 AA via the repository's accessibility config; `fixtures/settings-routes.json` for the section list |
| Oracle: comparator | URL and heading equality | Set equality and exactly-once | Count <= 8 | P0 = 0; manifest equality; each P1 resolved | Zero violations; zero overflow |
| Negative control: input | The four fallback controls and an unlisted deeper URL | Scratch route table without `rag`; scratch table listing `sandbox` | Scratch section with 9 visible controls | Packet with a forbidden path; file whose body disagrees with its header | Scratch element 400 px wide at 320 px |
| Expected rejection | Redirect to the fixture's final URLs; no error page | Comparison names `rag` and `sandbox` | Counter names the section | Review not counted; count fails | Overflow check fails |
| Fixtures | Scratch instance | Scratch instance and scratch consoles | Scratch instance and scratch console | Artifact packets only | Scratch instance |
| Evidence required | Trace and URL log | Navigation snapshot and `/types` response | Counter output and trace | Reviewer files, manifests, runner counts | Playwright report, axe output |
| Limitations | No Tauri | Pages by heading | Count only | Same model family unless external judge works | No screen reader |
| Invalidation keys | `fixtures/settings-routes.json`, `App.tsx`, `nav-destinations.ts`, `command-palette.tsx` | same plus the status change's removed list | `fixtures/section-controls.json`, `AdvancedSection`, settings panels | `fixtures/critique-gate.json`, reviewed artifacts | settings UI files, `tokens.css` |
| Restoration | Stop instance, delete scratch | same, delete scratch consoles | same | none beyond deleting scratch packets | same as 001 |
| Prerequisites | Regroup and status change implemented | same | regroup implemented | design context and regroup complete | regroup implemented |
| Owner task(s) | 3.1, 3.2, 5.1 | 3.1, 3.3, 5.2 | 3.4, 5.3 | 4.1-4.3, 5.6 | 3.5, 5.7 |

| Field | SCN-DSN-004 (supplemental) | SCN-DSN-005 (supplemental) | SCN-DSN-006 (supplemental) |
|---|---|---|---|
| Name | DESIGN.md derives from docs/design | PRODUCT.md no longer forbids the redesign | Authority documents agree |
| Actor | Designer or agent producing UI | Agent reading project context | Contributor resolving a design conflict |
| Useful outcome | New screens are generated on the authorized design | The loaded context matches the phase | One answer to what governs visuals |
| Source requirement | `openspec:frontend-design-authority#DESIGN.md derives from docs/design` | `openspec:frontend-design-authority#Design context is refreshed without changing operator settings` | `openspec:frontend-design-authority#Design authority names the redesign sources and retires superseded documents` |
| Reason now | Goal 2 | Goal 2 | Decision 5 |
| Likelihood | Certain | Certain | Certain |
| Mandatory | false | false | false |
| Classification | supplemental (file and CLI check, not product behavior) | supplemental | supplemental |
| Nondeterministic | false | false | false |
| Production entry | Impeccable CLI at the pinned engine; repository files | `frontend/PRODUCT.md` read by the CLI | File reads and searches |
| Real collaborators | Impeccable CLI; files | Impeccable CLI | Files |
| Oracle: observation | CLI output; frontmatter values; file bytes | Text presence and absence | Text presence; search results |
| Oracle: truth source | `fixtures/design-context.json` | `fixtures/design-context.json` | `fixtures/design-context.json` |
| Oracle: comparator | Value equality; byte identity across `document` | Substring checks | Substring checks; empty citation search |
| Negative control: input | Scratch DESIGN.md with one token changed | Pre-phase PRODUCT.md | Pre-phase tree |
| Expected rejection | Comparison reports the token | Contradiction found | Old claims found |
| Fixtures | Scratch copies | Read only | Read only |
| Evidence required | CLI output, diff, hashes | CLI output, text diff | Search output |
| Substituted boundaries | none | none | none |
| Limitations | Loadability and derivation, not quality | Facts are the operator's | Textual search |
| Invalidation keys | `docs/design/brand-spec.md`, `frontend/DESIGN.md`, `.impeccable/design.json`, engine pin | `frontend/PRODUCT.md` | the three docs |
| Restoration | Delete scratch copies | none | none |
| Prerequisites | Impeccable CLI at the pinned engine | same | none |
| Owner task(s) | 1.4, 1.5, 5.4 | 1.3, 5.4 | 2.1-2.3, 5.5 |
