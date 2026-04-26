## Why

The runtime console cannot be accepted while `bun run lint` fails, because lint errors currently hide real regressions in React effect usage, unused state, and hot-refresh boundaries. This change turns frontend lint into a clean hardening gate before further runtime-console visual, realtime, and provider compatibility work builds on top of the UI.

## What Changes

- Fix all current frontend ESLint errors without suppressing meaningful rules.
- Resolve or intentionally isolate all current frontend ESLint warnings so the lint command exits cleanly.
- Refactor React effect patterns that synchronously set state where derived state or async callbacks are more appropriate.
- Remove unused variables and stale props from frontend tests and provider/chat components.
- Preserve the existing frontend layering rules: components render and call hooks, hooks expose store state/actions, stores call services, and services own HTTP/SSE I/O.
- Keep runtime console behavior unchanged except where fixes are needed to make the UI safer for realtime state updates and provider/status inspection.
- Update KBD progress for `runtime-console-validation-hardening` after lint is verified.

## Capabilities

### New Capabilities

- `frontend-validation-gate`: Defines the frontend lint/typecheck gate required before runtime-console UI, realtime entity graph, and provider compatibility changes can be treated as accepted.

### Modified Capabilities

- None.

## Impact

- Affected frontend files include the current lint failure locations in `frontend/e2e/`, `frontend/src/components/`, `frontend/src/features/chat/`, `frontend/src/pages/`, and `frontend/src/admin/pages/`.
- The change affects frontend implementation and test hygiene only; it does not introduce backend API changes, data migrations, or provider routing changes.
- Runtime UX impact is reduced risk: the runtime console and chat/provider surfaces should avoid avoidable rerender cascades and stale unused state before visual and live-update testing begins.
- Provider compatibility impact is indirect: provider status screens and chat selectors become easier to verify once lint no longer blocks the frontend validation gate.
- Realtime state impact is indirect: React effect cleanup reduces the chance that live AG-UI/A2UI/entity graph updates are masked by avoidable cascaded renders.
- KBD workflow state must be updated when the lint gate moves from `change_created` to implementation and verification states.
