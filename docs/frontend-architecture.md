# Frontend Architecture

UAR's first-party interface is a React 19 + TypeScript application in `frontend/`. Rust/Axum owns server APIs and streaming transports. This document is the canonical frontend ownership contract; [ADR-007](adr/ADR-007-react-first-frontend.md) records the decision.

## Dependency direction

```text
Component → Hook/View Model → Store/Entity Domain → Service → API
                                      ↑                 │
                                      └── state/events ─┘
```

### Components

Components and pages under `frontend/src/components`, `frontend/src/admin`, `frontend/src/pages`, and feature `.tsx` files may:

- render accessible UI;
- hold ephemeral presentation state such as an open dialog or selected tab;
- call hooks and invoke the actions returned by hooks.

They must not call `fetch`, import `frontend/src/services`, import Zustand stores directly, mutate the entity graph, or implement persistence/workflow rules.

### Hooks and view models

Hooks compose presentation state, subscribe to stores, select derived values, and expose store actions. Hooks must not call `fetch` or import service modules. A hook is not a loophole for hiding I/O.

### Stores and entity domains

Stores under `frontend/src/stores` and explicitly owned entity-domain actions are the application-state boundary. They own:

- loading, success and error state;
- service calls;
- optimistic updates and rollback;
- realtime event reconciliation;
- cache and persistence coordination.

Entity hooks remain subscription façades. Fetchers and mutation helpers must be invoked by the owning store/domain action, not pages.

### Services

Services under `frontend/src/services` are thin, typed I/O adapters. They may call `fetch`, open streams, upload files, or access external APIs. They must not render UI or own application state.

### Infrastructure exceptions

Low-level asset and transport bootstrap modules may perform I/O outside feature services when the behavior is intrinsically infrastructural—for example loading a PGlite WASM asset or establishing the shared realtime adapter. Exceptions are explicit in the boundary checker and cannot include feature components or hooks.

## Feature example

```text
ProvidersPage
  → useProvidersPage()
    → providerStore.configureProvider()
      → providersService.configureProvider()
        → POST /api/uar/providers
```

The response updates the provider store/entity graph. `ProvidersPage` rerenders from subscribed state; it does not manually refresh or write graph records.

## Protocol ownership

- The AG-UI adapter converts versioned wire events into normalized application events exactly once. Chat and Runtime Console consume the same normalized stream.
- The A2UI renderer validates a declared protocol profile and approved component catalog, then renders native React components. It never executes model-provided HTML or JavaScript.
- PGlite stores local thread/message state. Server synchronization occurs through typed services and realtime adapters, not direct component database calls.

## Enforcement and migration

`scripts/check-frontend-boundaries.mjs` reports dependency violations. `scripts/frontend-boundary-allowlist.txt` contains the exact pre-existing set during migration. A change may remove entries but may not add one without an architecture decision. `close-react-boundary-gate` removes the production allowlist before GA.
