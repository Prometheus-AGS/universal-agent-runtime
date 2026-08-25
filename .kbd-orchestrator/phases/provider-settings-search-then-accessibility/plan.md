# PLAN: provider-settings-search-then-accessibility

Project: universal-agent-runtime
Date: 2026-08-25
OpenSpec available: YES
Changes to implement: 2

## CHANGE LIST (ordered)

1. `provider-model-search`: Add adaptive search to large provider model inventories
   - Scope: UI
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Files: `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx`; `frontend/src/features/settings/ui/settings-primitives.tsx` or a narrowly owned sibling primitive; `frontend/src/features/settings/ui/panels/ai-settings-panels.test.tsx`; focused primitive tests if the shared wrapper changes.
   - Details: Keep the simple shadcn select for one through seven enabled models and use the installed Base UI Combobox for eight or more. Search display labels and raw IDs with trimmed, case-insensitive literal matching while preserving provider order and the existing bounded settings draft.
   - Acceptance: The 7/8 boundary, label/id filtering, no-match state, duplicate-label disambiguation, pointer/Enter selection, popup closure, and invalid-value rejection have focused evidence. Existing empty and stale states remain intact.
   - Trade-off / scope cut: no fuzzy ranking, virtualization, catalog fetch, dependency, free-form model creation, or visual redesign.

2. `provider-settings-accessibility-dirty-state`: Complete provider accessibility and protect dirty drafts
   - Scope: UI | settings hook presentation state
   - Depends on: `provider-model-search`
   - Recommended agent: Codex
   - Est. complexity: M
   - Customer value: HIGH
   - Files: `frontend/src/features/settings/ui/panels/ai-settings-panels.tsx`; `frontend/src/features/settings/ui/settings-primitives.tsx`; `frontend/src/features/settings/model/use-settings.ts` only if actual background-loading state is not already exposed; focused provider and primitive tests.
   - Details: Associate every repeated provider control and hint, add live status/error semantics, derive visible modified state from the existing dirty map, disable Save while clean, disable Refresh while dirty/busy with explanatory text, and install a dirty-only browser-unload guard. Stack provider fields at narrow widths while retaining the desktop two-column layout.
   - Acceptance: Provider-specific names/descriptions, stale-model association, status/alert behavior, clean/dirty/save failure states, protected Refresh, beforeunload cancellation, and responsive structure have focused evidence; shared primitive consumers remain compatible.
   - Trade-off / scope cut: no discard dialog, field-level merge, structural draft reconciliation, persistence change, or unrelated settings redesign. Refresh is disabled while dirty because current `reload()` preserves drafts and can otherwise move the remote baseline underneath a whole-provider draft.

## EXECUTION ROUND ORDER

Round 1 (sequential): `provider-model-search`

Round 2 (sequential, after Round 1 verification): `provider-settings-accessibility-dirty-state`

## VERIFICATION ORDER

1. B focused: provider panel and affected shared primitive Vitest files.
2. B cheap gates: frontend typecheck, lint, and settings structure.
3. B completion: strict OpenSpec validation, frontend build, and full frontend tests before A starts.
4. A focused: provider panel and affected shared primitive Vitest files.
5. A cheap gates: frontend typecheck, lint, and settings structure.
6. Phase completion: both strict OpenSpec validations, frontend build, full frontend tests, one final Impeccable detector, and fresh-context adversarial review.

## COMMANDS TO RUN

OpenSpec changes already exist and are strictly valid:

`/opsx:apply provider-model-search`

Then, only after it verifies:

`/opsx:apply provider-settings-accessibility-dirty-state`

## PLAN COMPLETE
