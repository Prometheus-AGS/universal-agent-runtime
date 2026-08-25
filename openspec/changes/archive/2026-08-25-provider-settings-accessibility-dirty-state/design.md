## Context

See `proposal.md` for motivation. `useSettings("provider")` already exposes a module-cached dirty map, overlays dirty values over graph values, and clears drafts only after successful saves. Its current `reload()` refreshes remote data without clearing drafts, so allowing Refresh while whole-provider drafts exist can move the remote baseline underneath a stale draft.

## Goals / Non-Goals

**Goals:**

- Make repeated provider controls and status changes complete for assistive technology.
- Make the existing provider dirty state visible and operationally meaningful.
- Protect drafts at the two observed boundaries: remote refresh and browser unload.
- Preserve shared primitive compatibility and the incumbent visual system.

**Non-Goals:**

- A new discard workflow, field-level merge/conflict resolution, or structural draft reconciliation.
- Changing persistence, provider payloads, realtime reconciliation, or unrelated settings panels.

## Decisions

### Extend shared primitives with optional association props

`Field` accepts `htmlFor`; input, masked input, toggle, select, and model-picker paths accept optional `id`, `aria-describedby`, and provider-specific accessible labels. Shared banners receive fixed status/alert semantics. Optional props preserve current consumers.

Alternative considered: provider-only invisible labels. Rejected because the visible labels should be the source of accessible naming and the same primitive gap would remain.

### Derive dirty feedback only from the provider settings hook

`Object.keys(dirty)` defines whether provider drafts exist and which provider cards are modified. Save is disabled when this set is empty. The page does not introduce duplicate local business state.

Alternative considered: component-level change tracking. Rejected because it can diverge from the authoritative draft cache across remounts and save failures.

### Disable Refresh instead of presenting a false discard dialog

Refresh stays available when clean and becomes disabled with explanatory text while dirty, saving, or reloading. The hook exposes actual namespace loading separately from initial-empty loading if necessary. This prevents a stale whole-provider draft from later overwriting an unseen refreshed baseline.

Alternative considered: confirm `reload()` as a discard. Rejected because `reload()` does not clear drafts and such a dialog would misstate behavior. An explicit hook-owned discard operation remains a separate feature.

### Install `beforeunload` only while drafts exist

The panel adds and removes a browser `beforeunload` listener with dirty state. Internal component remounts retain the module cache; the guard targets the actual unload boundary where that cache is lost.

Alternative considered: intercept every internal settings navigation. Rejected because drafts survive those transitions and a blanket prompt would interrupt safe navigation.

### Keep provider context visible and structural

Cards become named groups, dirty cards show a restrained `Modified` badge, and the field grid uses one column below the desktop breakpoint. Status is communicated in text rather than color alone.

## Risks / Trade-offs

- [Shared primitive signature changes regress other panels] → Add only optional props and run focused shared primitive plus settings tests.
- [Dirty feedback becomes a false field count] → Mark modified provider cards without reporting a numeric change count.
- [A user reverts a whole object but the draft cache remains dirty] → Preserve the current authoritative draft semantics; structural reconciliation is explicitly outside this change.
- [Browser unload prompts vary by browser] → Test event cancellation and rely on the browser-provided confirmation copy.
- [Refresh appears unavailable without explanation] → Pair the disabled action with visible and described `Save changes before refreshing` text.

## Migration Plan

Ship after `provider-model-search` so labels and descriptions cover both picker variants. Rollback removes the optional associations, dirty indicators, unload guard, and responsive class changes without altering stored settings or APIs.
