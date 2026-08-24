## Context

See `proposal.md` for motivation. The Agents page currently derives status from `ProviderMeta:current`, but its own load path does not hydrate provider metadata, and the Providers store writes `default_id` without the selected provider's `default_model`. The component consequently interprets an absent client projection as an absent runtime route.

The frontend architecture requires business state to remain in the normalized entity graph, I/O to remain behind stores/actions, and components to consume typed domain hooks. The existing application-level tooltip provider and Base UI tooltip primitive already provide hover and focus behavior.

## Goals / Non-Goals

**Goals:**

- Make provider-default projection complete and available wherever model status is rendered.
- Expose a typed hook that distinguishes loading, available, unavailable, and initial-load failure without duplicating provider state in component-local state.
- Compose a tooltip around each applicable agent row so the row remains the single focus target.

**Non-Goals:**

- Changing provider routing, agent persistence, backend APIs, or model-selection policy.
- Adding a second status store or importing entity-management internals into components.
- Refactoring the broader provider graph hydration paths.

## Decisions

### Hydrate provider metadata from one registry response

The Providers store will project both `default_id` and the matching provider's `default_model` into `ProviderMeta:current` in the same graph transaction. This keeps the resolution inputs from one server snapshot and fixes the path already used by provider administration. Replacing all provider loaders with a new abstraction was rejected because it expands a small observed defect into an adjacent migration.

### Load and classify through a typed provider-domain hook

The provider-default hook will request the existing Providers store load on mount and subscribe narrowly to provider metadata plus load state. The Agents page receives only the status and effective route. Component-local React state and direct graph/store imports are rejected because provider routing is shared business state, not view mechanics.

### Make the row the tooltip trigger

The existing agent-row button will be composed as the tooltip trigger. This provides hover and keyboard-focus explanations without nesting a second focusable button inside the row or adding render-driven state setters. Resolved inheritance uses a neutral information icon; only confirmed missing routes and unverifiable registry state use amber indicators.

## Risks / Trade-offs

- [A successful projection can become stale if a later refresh fails] → Keep the last confirmed route visible rather than converting a working agent into a false warning; provider administration already exposes refresh failure separately.
- [Tooltip text alone is not visible to touch-only users] → Preserve concise accessible labels on the status icon and attach the tooltip to the entire row focus target.
- [Two provider hydration functions remain in the codebase] → Limit this fix to the actively used store path and record consolidation as out of scope; both paths will now project the same default fields.

## Migration Plan

Build and deploy the frontend bundle with no data migration. Rollback is the preceding static bundle; provider and agent records are unchanged.
