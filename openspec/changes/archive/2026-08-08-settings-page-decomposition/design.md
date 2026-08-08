## Context

C-14a moved the settings ownership cluster into `features/settings`, but intentionally left `ui/settings-page.tsx` intact at 3,336 lines for C-14b. The file contains five separate responsibilities: the navigation inventory, reusable field/panel primitives, schema-driven rendering, domain-specific panels, and the route-level shell. It already follows the hook boundary and must retain its current `useSettings`, onboarding, metadata, JWT, realtime change-bus, provider/model, and save/reload behavior.

This is a structural refactor, not a visual redesign. The established compact utilitarian console treatment, responsive sidebar/content layout, semantic tokens, typography, interaction targets, and accessibility labels remain the design authority.

## Goals / Non-Goals

**Goals:**

- Reduce the route-level settings page to composition and navigation only.
- Keep every resulting settings UI module at or below approximately 600 lines.
- Establish explicit internal ownership for shared primitives, schema rendering, panel registry, and domain panel groups.
- Preserve the current route export, navigation order, availability filtering, default active panel, controls, validation, loading/error/saved states, save/reload behavior, and JWT gating.
- Add focused evidence that the decomposed registry and responsive composition still expose the same settings surface.

**Non-Goals:**

- Redesigning, restyling, renaming, adding, or removing settings controls.
- Moving model hooks, stores, APIs, schemas, realtime behavior, or backend contracts.
- Retiring the remaining admin shell, dependencies, or stores; C-14c owns that work.
- Installing the final feature boundary zones or changing the settings feature public root.

## Decisions

### 1. Split by stable UI responsibility and domain cohesion

The settings UI becomes:

- `settings-navigation.tsx`: typed navigation categories and items.
- `settings-primitives.tsx`: fields, toggles, selects, headers, and status banners.
- `generic-schema-panel.tsx`: namespace wrapper and metadata-driven controls.
- `panels/ai-settings-panels.tsx`: provider, vision, context, RAG, and knowledge-base panels.
- `panels/file-processing-settings-panels.tsx`: file processing, Unstructured, Mistral, and Kreuzberg panels.
- `panels/resilience-settings-panel.tsx` plus `resilience-preview.ts`: global resilience editor and pure effective-policy helpers.
- `panels/governance-settings-panels.tsx`: intent classifier, governance, agent, and skill configuration.
- `panels/memory-settings-panel.tsx`: memory configuration.
- `panels/caching-user-settings-panels.tsx`: prompt caching and JWT-gated user preferences.
- `settings-panel-registry.tsx`: custom and generic namespace resolution.
- `settings-page.tsx`: responsive sidebar, active selection, availability, and content composition.

Alternative considered: one file per individual panel. Rejected because many panels are small, share one domain vocabulary, and would turn a mechanical decomposition into dozens of low-value files. The selected groups remain below the binding size target while preserving coherent review units.

### 2. Preserve component bodies and contracts before cleanup

Each declaration moves mechanically with only export/import changes. JSX, class names, copy, constants, hook calls, validation, and event handlers remain unchanged. No opportunistic abstraction or stylistic cleanup is allowed in this change.

Alternative considered: normalize repeated save/loading wrappers while splitting. Rejected because that would combine behavior changes with ownership extraction and make regressions harder to attribute.

### 3. Keep panel resolution data-driven and internal

The panel registry remains a single key-to-renderer map. The page uses the same typed navigation inventory and fallback to `GenericSchemaPanel`. Neither panels nor registry are exported from the settings feature root; only `SettingsPage` remains public.

Alternative considered: route each namespace independently. Rejected because there is one existing settings route and C-14b must preserve query/navigation behavior.

### 4. Verify structure and behavior separately

A structural gate records module line counts and prevents a return to a monolith. Focused React tests verify default composition, navigation to a custom panel, unavailable item behavior, and the retained generic fallback with mocked model hooks. Existing model tests continue to cover the store/API seam.

## Risks / Trade-offs

- **[Risk] A declaration moves without one of its local dependencies.** → Extract in responsibility-sized checkpoints and run typecheck/lint after each group.
- **[Risk] Panel keys or navigation order drift.** → Keep the existing constants byte-for-byte and add focused registry/navigation assertions.
- **[Risk] Repeated wrappers invite unrelated refactoring.** → Preserve component bodies; abstraction changes are explicitly out of scope.
- **[Risk] Barrel exports widen the initial bundle.** → Keep all extracted modules internal to the settings UI and retain only `SettingsPage` at the feature root, honoring the measured C-14a budget constraint.
- **[Risk] The settings page exceeds the target through future recombination.** → Add an exact settings UI module-size check to the existing frontend validation harness.

## Migration Plan

1. Capture current declaration boundaries, navigation inventory, page export, line counts, and protected-path state.
2. Extract navigation, shared primitives, schema controls, and pure resilience helpers.
3. Move panel groups without changing bodies, then establish the internal registry.
4. Reduce `settings-page.tsx` to route-level composition and preserve the feature root export.
5. Add focused composition and module-size validation, then run cheap gates and focused tests.
6. At completion, run the consolidated frontend/bundle boundary appropriate to C-14b, strict OpenSpec validation, isolated review, canonical transition, sync, and archive.

Rollback is file-local: restore the original monolithic file and root import if a checkpoint fails. No data or backend migration exists.

## Open Questions

None. The KBD plan fixes the behavior-preserving scope, approximately 600-line ceiling, and strict C-14a → C-14b → C-14c order.
