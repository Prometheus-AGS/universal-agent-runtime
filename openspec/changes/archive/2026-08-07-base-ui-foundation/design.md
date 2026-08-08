## Context

UAR's frontend primitive wrappers were regenerated onto Base UI so product code could
retain the existing shadcn-style wrapper API while replacing the underlying Radix
implementations. The delivered tree uses `@base-ui/react` from production primitive
files, and `frontend/components.json` selects the `base-vega` style with the `neutral`
base color.

This archive reconciliation documents the implemented architecture. The current
manifest still contains unused Radix package declarations, but production source has no
direct Radix imports. Their transitive audit and removal belong to the later
`retire-admin-and-legacy-deps` change, where `cmdk`, assistant-ui, and Storybook can be
checked together.

## Goals / Non-Goals

**Goals:**

- Make Base UI the implementation foundation for interactive production primitives.
- Preserve stable local wrapper imports for application code.
- Keep shadcn generator metadata aligned with the Base UI `base-vega` preset.

**Non-Goals:**

- Rewrite `asChild` composition call sites; `base-ui-composition-patterns` owns that work.
- Complete icon migration; `base-ui-icon-migration` owns that work.
- Remove every Radix manifest declaration; C-14c owns the final transitive audit and prune.

## Decisions

### Use Base UI behind local wrappers

Interactive primitives import Base UI inside `frontend/src/components/ui/`. Feature and
page code continues importing the local wrappers, which isolates the application from
headless-library API details.

At archive verification time, 34 wrapper files imported Base UI directly. The larger
regeneration count also includes layout, styling, and native-element wrappers that do
not require a headless Base UI primitive; the gate therefore requires a non-empty Base
UI-backed set plus zero direct production Radix imports instead of freezing that count.

**Alternative considered:** migrate every consumer directly to Base UI. Rejected because
it would spread primitive API coupling across the product and make the migration harder
to stage.

### Use the shadcn `base-vega` generator profile

`frontend/components.json` records `base-vega` and `neutral`, allowing regenerated
wrappers to remain consistent with the selected foundation.

**Alternative considered:** hand-maintain all wrappers without generator metadata.
Rejected because later regeneration could silently return the tree to a different
primitive family.

### Defer package pruning until transitive consumers are audited

Production code contains no direct Radix imports, but dependency declaration cleanup is
deferred to C-14c so packages used transitively by `cmdk`, assistant-ui, or Storybook are
not removed blindly.

## Risks / Trade-offs

- **Mixed composition APIs during the staged migration** → C-03b owns the complete
  `asChild`-to-`render` conversion and related tests.
- **Unused Radix declarations remain temporarily** → C-14c performs the transitive
  dependency check and final manifest prune.
- **Generated wrappers may change across CLI releases** → committed wrapper source and
  the pinned Base UI dependency remain the reviewable runtime authority.

## Migration Plan

1. Regenerate the local primitive wrappers using the Base UI profile.
2. Repair Base UI API differences within the wrapper boundary.
3. Complete composition and icon follow-up changes independently.
4. Remove obsolete declarations only after the final transitive dependency audit.

Rollback is a normal source-control revert of the wrapper and manifest changes; no
persistent data migration is involved.

## Open Questions

None for the foundation. Composition, icon, and final dependency-pruning work are tracked
by their named follow-up changes.
