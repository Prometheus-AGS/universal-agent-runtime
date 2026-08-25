# ASSESSMENT: provider-settings-search-then-accessibility

Project: universal-agent-runtime
Date: 2026-08-25
Current phase: Spec
Requested order: B (searchable provider model inventories), then A (accessibility and dirty-state protection)

## IMPLEMENTATION STATUS

- B — searchable model inventories: **PARTIAL**. `ProviderPanel` already derives a bounded, deduplicated list of enabled models from each provider and renders the existing shadcn/Base UI `SettingSelect`. Large inventories have no search path.
- B — inventory validity: **DONE**. `getProviderModelOptions` rejects malformed and disabled entries, preserves provider order, falls back to the model id for blank labels, and deduplicates by id.
- B — empty and stale states: **DONE**. Zero-option and unavailable-current-model states already have explicit visual treatment.
- A — programmatic labels: **PARTIAL**. The Default Model select has a provider-specific accessible name, but visible `Field` labels are not associated with controls; provider switches and API-key reveal buttons are not provider-specific.
- A — status semantics: **PARTIAL**. Loading, saved, and error copy is visible but lacks consistent live-region semantics.
- A — dirty-state protection: **PARTIAL**. `useSettings("provider")` exposes a durable in-session dirty map and preserves it across `reload()`, but the provider Save action is enabled while clean and the page gives no visible unsaved-state feedback or browser-unload protection.
- A — responsive provider cards: **PARTIAL**. The provider editor uses an unconditional two-column grid instead of stacking at narrow widths.

## DESIGN ASSESSMENT

- Preserve the simple select for one through seven enabled models and use a dedicated searchable shadcn/Base UI Combobox for eight or more. Eight is the first inventory size beyond the 5–7 recognition boundary cited by the Impeccable critique, and the named threshold keeps the policy testable.
- Search must match both display label and raw model id with case-insensitive, trimmed literal substring matching. Configuration order remains authoritative and free-form values remain impossible.
- The searchable and simple controls must share the same closed-state dimensions, label, invalid state, placeholder, and provider-owned value path.
- A must follow B so programmatic labels, descriptions, keyboard semantics, and responsive constraints can cover both control variants once.
- Refresh currently preserves unsaved drafts. A confirmation that claims Refresh discards edits would be false unless a new explicit discard operation is introduced. The minimum honest protection is visible dirty state, Save disabled while clean, browser-unload protection while dirty, and Refresh copy that does not imply discard.
- Shared primitive extensions must be optional so existing settings panels keep their current API and behavior.

## SPEC GAPS

- `frontend-configuration-surfaces` does not define when a provider model inventory becomes searchable, what fields are searched, or the 7/8 behavior boundary.
- It does not require every provider control to have a programmatic label and associated help/error description.
- It does not require loading/save/error announcements or visible provider dirty state.
- It does not require browser-unload protection for unsaved provider edits or a responsive single-column layout at narrow widths.

## ACCEPTANCE BOUNDARY

### B — provider-model-search

- Seven enabled models render the simple shadcn select; eight render the searchable shadcn/Base UI combobox.
- Search matches label and id case-insensitively after trimming query whitespace, preserves provider order, treats punctuation literally, and shows `No matching models` when empty.
- Pointer and Enter selection write exactly one valid provider model id through the existing settings draft, close the popup, and do not permit arbitrary text.
- Disabled, malformed, empty-id, and duplicate-id models remain unavailable; duplicate labels expose the raw id for disambiguation.
- Existing zero-option and stale-model behavior remains intact.

### A — provider-settings-accessibility-dirty-state

- Provider cards and every Base URL, Protocol, API Key, Default Model, and Enabled control have stable programmatic names; help and invalid text is referenced with `aria-describedby`.
- API-key reveal buttons and switches include provider context.
- Loading and save success use polite status semantics; errors use alert semantics.
- Save is disabled while clean, enabled while provider drafts exist, and disabled again after successful save. Dirty providers receive visible text feedback.
- Browser unload is prevented while provider drafts exist and allowed while clean. Refresh continues to preserve drafts and therefore does not present a discard confirmation.
- Provider fields stack at narrow widths and retain the incumbent two-column desktop composition.

## BUILD HEALTH

- Focused Vitest baseline: **PASS** — 2 files, 5 tests.
- Frontend typecheck: **PASS**.
- Frontend lint: **PASS**.
- Settings structure gate: **PASS** — 11 modules; largest 549/600; 29 keys preserved.
- Browser-level responsive and focus behavior: **UNVERIFIED** until implementation is complete and the built provider panel is exercised at narrow and desktop widths.

## RISKS AND SCOPE CUTS

- P1: a generic Command/Popover composition can lose form-combobox semantics. Use the repository's Base UI Combobox wrapper and test keyboard selection and accessible roles.
- P1: incompatible shared primitive changes could regress other settings panels. Add optional props and run focused primitive plus provider tests.
- P2: search by label alone hides raw-id workflows and duplicate labels. Search both and render id metadata when useful.
- P2: whole-provider dirty drafts can remain dirty after a user manually returns every field to its original value. Structural draft reconciliation is not required by the request and is deferred unless the existing hook exposes a small, verified path.
- P2: live inventory changes can cross the 7/8 threshold while focused. Do not auto-select or save; preserve the bounded current value and cover the threshold statically in tests.
- Scope cut: no catalog fetch, virtualization, fuzzy ranking, new entity state, new dependency, or visual redesign.

## ASSESSMENT COMPLETE

The minimum coherent delivery is two ordered OpenSpec changes: first an adaptive bounded model picker for large provider inventories, then provider-panel accessibility, live status, dirty-state visibility/unload protection, and responsive layout. The uncomfortable case is a Refresh confirmation that promises data loss even though current code preserves drafts; this phase will keep Refresh honest instead of inventing a discard workflow outside the user's request.
