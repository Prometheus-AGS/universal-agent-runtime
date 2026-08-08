## Why

The application has a minimal three-link rail and bottom bar, but it does not implement the binding UAR shell contract: work and Configure destinations are not modeled once, the rail cannot collapse to 60px, narrow layouts lack the Configure hub and shared sheet host, route context has no app-level breadcrumb header, and the shipped UI still uses the retired KnowMe mark instead of the delivered Slash Gate assets. The existing `migrate-cross-cutting-pages` change is the plan-assigned C-10 vehicle; its stale entity-store migration tasks are superseded by the later C-14 feature migration and must not be executed here.

## What Changes

- Replace the minimal shell with the target `app/shell/` composition: 240px/60px desktop rail, compact bottom tabs, breadcrumb header, shared mobile sheet host, and one destination inventory.
- Model Chat, Knowledge, Agents, and Runs as work destinations; model Providers, MCP & tools, Skills, A2UI, Runtime settings, and About as Configure/system destinations. On compact layouts, the Configure tab opens the shared hub sheet instead of squeezing desktop navigation into unexplained icons.
- Resolve cand-011 by using the installed Base UI `Autocomplete` plus Base UI `Dialog` for the app command palette. Existing `cmdk` consumers remain unchanged until their later migration/retirement owner; the new shell does not introduce another `cmdk` surface.
- Add app-shell presentation state behind the existing hook/store boundary for rail collapse, command-palette visibility, and the active mobile sheet.
- Copy the delivered Slash Gate assets from `docs/ui/logo/` into `frontend/public/brand/`, replace the retired KnowMe React mark with `shared/ui/uar-logo.tsx`, and install light/dark favicon links.
- Add focused tests for destination matching, desktop collapse, compact tabs and Configure sheet behavior, breadcrumbs, keyboard palette access, navigation, brand usage, and accessible state labels.
- Preserve provider payloads, AG-UI events, durable entities, feature stores, service contracts, and the C-14-owned admin/entity migrations.

## Capabilities

### New Capabilities

- `frontend-app-shell`: responsive shell composition, navigation inventory, command access, brand projection, and shell accessibility behavior.

### Modified Capabilities

None.

## Impact

- Affected code: `frontend/src/app/shell/`, the app composition root, the existing UI state hook/store, current brand consumers, `frontend/public/brand/`, and focused shell tests.
- Dependencies: no additions. The installed `@base-ui/react` 1.6.0 provides the documented command-palette primitives. `cmdk` remains declared for existing non-shell consumers and is not removed in C-10.
- Routing: current `/threads`, `/admin/*`, and `/about` routes remain valid; the shared destination model projects selected admin sections as first-class work or Configure destinations without moving their feature implementation.
- Accessibility: semantic navigation landmarks, skip navigation, visible 3px focus treatment, text-plus-color readiness, accessible collapsed labels, keyboard palette access, reduced-motion-safe transitions, and 44px compact targets.
- Security/runtime boundaries: unchanged. Navigation targets are static application routes; no user-provided URL, HTML, provider payload, or credential enters the shell command model.
