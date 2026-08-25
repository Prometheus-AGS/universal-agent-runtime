## Context

See `proposal.md` for motivation. The provider panel already owns model eligibility and draft updates. The installed settings primitives expose the project's shadcn/Base UI Select and Combobox surfaces, and focused tests already cover bounded short-list selection.

## Goals / Non-Goals

**Goals:**

- Add efficient search without weakening the provider-owned validity boundary.
- Keep the closed control visually and semantically consistent across the 7/8 threshold.
- Reuse the existing settings hook and avoid new business state.

**Non-Goals:**

- Fuzzy ranking, model catalog fetching, virtualization, free-form creation, or provider API changes.
- A redesign of provider cards or unrelated settings controls.

## Decisions

### Use a dedicated model-picker wrapper with a named threshold

`SEARCHABLE_MODEL_THRESHOLD` is eight. The wrapper renders the existing `SettingSelect` below the threshold and the installed Base UI Combobox at or above it. Keeping this policy provider-specific avoids surprising every generic settings select.

Alternative considered: make every `SettingSelect` searchable. Rejected because short enumerations do not benefit from a search input and other settings callers have not requested hybrid behavior.

### Use Base UI Combobox rather than Command plus Popover

The form Combobox owns input, listbox, selected-value, focus, and keyboard semantics as one primitive. It receives the same `{label, value}` option objects as `SettingSelect`, with filtering based on normalized label and raw value.

Alternative considered: compose Command and Popover. Rejected because Command is an action-filter surface and would require custom work to restore value-control semantics, focus return, and selected state.

### Preserve provider order and use literal substring filtering

The picker normalizes the trimmed query and candidate fields with locale-insensitive lowercase comparison, then preserves the original option order. Results display raw IDs as subordinate metadata when labels differ, which also resolves duplicate labels.

Alternative considered: fuzzy scoring. Rejected because it changes ordering without observed usage evidence and complicates deterministic tests.

### Keep `getProviderModelOptions` as the validity boundary

No search result is synthesized from text. Selection can only emit an option already returned by the existing provider eligibility helper, and the existing settings hook remains the only draft mutation path.

## Risks / Trade-offs

- [Inventory changes across the threshold while the control is focused] → Never auto-select or save; React may remount the variant only when the provider data actually changes, and static 7/8 tests protect the policy.
- [Combobox keyboard or focus behavior regresses] → Add focused tests for input discovery, Enter selection, popup closure, and bounded emitted values.
- [Duplicate labels are indistinguishable] → Render the raw model id as secondary result text whenever it differs from the label.
- [Hundreds of options cost more to render] → Keep the popup scroll-bounded; defer virtualization until an observed performance failure exists.

## Migration Plan

Ship as a frontend-only additive control path. Rollback removes the searchable variant and returns all inventories to the existing bounded select; persisted values and API contracts are unchanged.
