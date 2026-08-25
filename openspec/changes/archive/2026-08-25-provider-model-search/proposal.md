## Why

Provider model selection is bounded to valid configured models, but long provider inventories are difficult to scan because the current shadcn select has no search path. The model control should stay simple for short lists and become efficiently searchable before large inventories turn recognition into trial and error.

## What Changes

- Keep the existing shadcn select for provider inventories with one through seven enabled models.
- Render a searchable shadcn/Base UI combobox when a provider has eight or more enabled models.
- Match literal, case-insensitive search terms against both model display names and raw model identifiers while preserving provider order.
- Preserve the existing eligibility, stale-value, empty-inventory, and settings-draft behavior without accepting free-form models.
- Add focused threshold, filtering, keyboard-selection, and bounded-value coverage.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-configuration-surfaces`: Define adaptive searchable provider model selection for large valid model inventories.

## Impact

- Runtime UX: large provider inventories become searchable while short inventories retain the simpler control.
- Provider compatibility: provider-owned enabled model IDs remain the sole valid value set; no provider API or catalog contract changes.
- Realtime state: no new entity or transport is introduced; selection continues through the existing provider settings hook and draft cache.
- Code: provider settings panel, a shared settings combobox primitive or wrapper, and focused frontend tests.
- Dependencies: none; use the installed shadcn/Base UI components.
- KBD: register this as the first ordered change in `provider-settings-search-then-accessibility`.
