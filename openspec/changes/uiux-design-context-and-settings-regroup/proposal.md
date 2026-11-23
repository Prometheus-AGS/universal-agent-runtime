## Why

The operator authorized a redesign of the operator console on 2026-11-19 and named `docs/design/`
the visual authority. Three things stand in the way, all verified in the repository:

- **The design context contradicts the phase.** `frontend/PRODUCT.md` says "It does not redesign
  the application or promote development tooling into production" (Operating Context). No
  `DESIGN.md` exists in `frontend/` or at the repository root. Two other documents still claim
  authority over visuals: `docs/UI_DESIGN.md` ("Material 3 Flat 2.0", retired by decision) and
  `docs/admin-aesthetic-spec.md` ("Terminal / CRT", status "authoritative", driven by the
  open change `setup-design-skills-and-playwright`). `docs/ui-design-authority.md` lists the
  authority order but names neither `docs/design/` nor the 2026-11-19 redesign decision. The
  skill engine is pinned to 0.1.17 by `IMPECCABLE_BIN` in the operator's own harness settings
  (`~/.claude/settings.json:6`, read, not edited), while the installed CLI is newer (analysis
  cand-011).
- **Settings is 30 flat sections.** `NAV_ITEMS` has 30 entries in 5 categories; the active
  section is `useState("provider")` in `settings-page.tsx:12`, so a section has no URL and cannot
  be linked, reloaded or reached with back/forward. Progressive disclosure is nearly absent:
  `AdvancedSection` is used in two places (assessment G9) and every schema-driven panel renders
  every field. `docs/design/` groups the same material into seven settings areas (Models &
  providers, Tools & MCP, Skills, Knowledge & memory, Governance, Appearance, Desktop;
  `docs/design/README.md`, `assets/settings.js:61-70`), URL-addressable
  (`react-implementation-plan.md` section 6).
- **Primary navigation points at the wrong places.** The "Runtime settings" destination claims
  `/admin/runtime`, `/admin/approvals`, `/admin/protocols`, `/admin/settings`, `/admin/auth`,
  `/admin/cost` and `/admin/memory` through `matchPaths` (`nav-destinations.ts`), although the
  cockpit is a dashboard and half of those pages have no link at all (assessment G5).

This change covers build items 6 and 23 of the analysis and the Impeccable engine pin finding.

## What Changes

- **Design context.** Run `impeccable update` (skills) and `impeccable init` to rewrite
  `PRODUCT.md` (the "does not redesign" statement goes; confirmed product truth stays). Write
  `DESIGN.md` from `docs/design/` (`brand-spec.md`, `react-implementation-plan.md`, the shell
  and settings stylesheets, the prototypes) and its `.impeccable/design.json` sidecar. Use
  Impeccable `document` only afterwards, as a drift check against the shipped code; its findings
  become work items, never edits that make `DESIGN.md` describe the code instead of the design.
- **Authority documents.** `docs/ui-design-authority.md` names `docs/design/` and `DESIGN.md` and
  reproduces the 2026-11-19 redesign decision (the document's portability rule requires it).
  `docs/UI_DESIGN.md` and `docs/admin-aesthetic-spec.md` get a retirement notice naming
  `docs/design/`; they are not deleted (the authority document keeps historical artifacts as
  context).
- **Impeccable engine pin.** Operator-owned. This change records the mismatch in its handoff and
  stops at the first command that needs a different engine. It does not edit user or harness
  settings.
- **Quality gate.** Impeccable critique by two isolated subagents plus a fresh-context
  adversarial review (already required by the `agent-ui-design-workflow` capability; this change
  schedules and records it, it does not restate it) over `DESIGN.md`, `PRODUCT.md` and the regrouped Settings diff (project UI
  rule), recorded under `.impeccable/critique/`.
- **Settings regroup.** One route table (`/settings/<area>/<section?>`) feeds the settings
  navigation, the command palette and the compact Configure hub. The surviving settings
  namespaces and the relinked admin pages (from `uiux-settings-truthful-status-and-removals`)
  land in seven areas. Sections gain URLs; primary controls show first and the rest sit under
  "Advanced". Old `/admin/<section>` URLs for moved pages redirect.
- **Primary navigation.** The Configure group becomes one Settings destination (plus
  Presentations and the dev-only A2UI entry). Runtime cockpit, approvals and protocols move under
  the Runs destination's match set.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-design-authority`: `docs/design/` is the visual authority; `DESIGN.md` derives from it;
  retired documents; the engine pin is not a repository concern.
- `runtime-console-ux`: Settings areas with deep links, progressive disclosure, one route table,
  primary navigation. One existing `frontend-app-shell` requirement is modified (see Impact).
- `frontend-app-shell`: modifies `Configure uses the shared compact sheet host` because its
  scenario lists Providers, MCP & tools, Skills, A2UI and Runtime settings as hub items. Added to
  the assigned list of capabilities because the regroup cannot leave that requirement false.

## Impact

- Docs and design context: `frontend/PRODUCT.md`, `frontend/DESIGN.md` (new),
  `.impeccable/design.json` (new), `docs/ui-design-authority.md`, `docs/UI_DESIGN.md`,
  `docs/admin-aesthetic-spec.md`, `.impeccable/critique/*` (new files).
- Frontend: `frontend/src/app/shell/nav-destinations.ts`, `command-palette.tsx`,
  `mobile-sheet-host.tsx`, `breadcrumb-header.tsx`, `frontend/src/App.tsx` (routes and
  redirects), `frontend/src/pages/admin-page.tsx`, `frontend/src/features/settings/**`
  (navigation, layout, route table, disclosure), `AdvancedSection`, e2e specs and
  `nav-destinations.test.ts` that assert the old inventory.
- No change to the primary shell layout (rail, top bar, bottom targets). The single sidebar with a
  tabbed inspector that `docs/design/` shows is **not** implemented here: no change in this phase
  owns it (see Unresolved decisions).
- Depends on `uiux-settings-truthful-status-and-removals` (surviving inventory) and provides the
  header slot that `uiux-system-model-live-default` fills with the system-default picker.
  Shares `settings-navigation.tsx` and `settings-page.tsx` with that change; order is enforced by
  tasks, one writer per file.

## Related open changes

Suggested dispositions. The operator decides in Plan (decision 7). Ledger counts are checkbox
counts, not verified against code.

| Open change | Suggested | Reason |
|---|---|---|
| `setup-design-skills-and-playwright` (0/16) | supersede | Installs skills and writes `docs/admin-aesthetic-spec.md` (terminal/CRT), the visual contract this change retires. Mark superseded, do not delete. |
| `migrate-models-page-direct-and-redesign`, `migrate-knowledge-page-direct-and-redesign`, `migrate-memory-page-direct-and-redesign`, `migrate-skills-page-direct-and-redesign`, `migrate-compiler-page-direct-and-redesign` (all 0 done) | supersede | Each "adopts the terminal aesthetic" for a page the redesign regroups. Their data-layer parts (direct reads, optimistic writes) overlap `uiux-model-identity-graph-ownership`; split per change when dispositioned. |
| `migrate-settings-mutations-and-retire-stores` (0/15) | supersede | Mandates the terminal aesthetic for settings (assessment section 0.5). Disposition also listed in the status change. |
| `impeccable-uiux-audit`, `uiux-remediation-wave-1` (empty shells) | archive | No artifacts. |

## Unresolved decisions

1. **Impeccable engine pin.** `IMPECCABLE_BIN` points at engine 0.1.17 in
   `~/.claude/settings.json`. Whether to move it is the operator's call. This change records
   the observed mismatch and does not edit that file or any harness setting.
2. **No change owns the shell redesign.** The single collapsible sidebar with a tabbed right
   inspector (`docs/design/README.md`, `ui-research-findings.md` section 3) is the headline of the
   authorized redesign, yet none of the ten assigned changes implements it. This change documents
   it in `DESIGN.md` as the target and regroups Settings inside the current shell. Either add a
   change for the shell, or accept that the phase delivers the design record and the settings
   regroup only.
3. **Appearance and Desktop scope.** `appearance.html` shows density, motion and text-size
   controls; the shipped console has a theme toggle only (`components/ThemeToggle.tsx`,
   `stores/theme-store.ts`). Default here: the Appearance area holds the existing theme control and
   nothing new; Desktop (Tauri) appears only when running in the desktop host and only with
   controls that already exist. Operator may widen.
4. **Creative North Star.** `brand-spec.md` states the system in one sentence but names no
   metaphor. Default: use that sentence and label any named metaphor provisional until the operator
   supplies one. Impeccable's document flow asks for it.
5. **"About seven" areas.** `ui-research-findings.md` section 3 lists nine settings pages
   (General, the seven, About); the prototype route table has General disabled
   (`assets/settings.js:62`). Default: seven areas; About stays at `/about`; there is no General
   area.
