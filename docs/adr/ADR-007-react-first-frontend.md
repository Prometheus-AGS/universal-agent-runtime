# ADR-007: React is the canonical first-party frontend

- Status: Accepted
- Date: 2026-07-11
- Owners: UAR maintainers

## Context

UAR's shipped browser interface and administrative console are implemented in React 19 and TypeScript under `frontend/`. Older documents describe an HTML/HTMX/Web Components application and sometimes state that React or SPA frameworks are intentionally avoided. Those descriptions no longer match the product.

The repository also contains several state and I/O paths that bypass the required frontend dependency direction. That makes a page capable of rendering while its mutations, errors, or realtime reconciliation remain disconnected.

## Decision

React 19 + TypeScript is the canonical first-party web UI and the frontend used by the Tauri desktop shell. The mandatory dependency direction is:

```text
React Component → Hook/View Model → Store/Entity Domain → Service → API
```

- Components render state, collect user intent, and call hooks.
- Hooks subscribe to stores and expose actions or presentation state.
- Stores/entity domains own application state, optimistic transactions, loading/error state, and service calls.
- Services own HTTP, streaming, persistence, and external I/O.
- State returns to components through React subscriptions and events.

A2UI is declarative data validated and rendered through approved React components. AG-UI is an event protocol between the frontend and agent runtime; it is not a UI framework.

HTMX and Web Components may remain in dated research, historical designs, or separately scoped embeddable artifacts, but they are not the primary UAR interface and must not be described as current product architecture.

## Consequences

- Existing violations are tracked in a finite legacy allowlist and removed by vertical feature certifications.
- CI rejects new violations immediately and eventually requires a zero-entry production allowlist.
- Public product documentation must identify React as primary.
- Mobile remains experimental until platform packaging and tests exist; sharing React source does not prove identical platform behavior.

## Verification

Run:

```bash
pnpm run frontend:boundaries
```

The release gate ultimately requires no allowlisted production violations.
