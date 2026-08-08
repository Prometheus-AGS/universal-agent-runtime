## Why

The migrated configuration pages still run through a legacy inner admin shell and retain obsolete theme, dependency, and technical-layer ownership. C-14c removes that compatibility layer now that C-14a and C-14b have established feature ownership, while preserving all production feature routes and the development-only A2UI tester required by C-12.

## What Changes

- Replace the legacy admin shell with direct route-to-feature composition under the shared application shell.
- Remove the terminal/CRT theme override and the now-unused TanStack Query, highlight.js, and direct Radix dependency declarations.
- Re-home the development-only A2UI tester and MCP health surface with their API/model ownership under the corresponding features, then delete `frontend/src/admin/` and the retired technical-layer files.
- Move the runtime feed subscription through its narrow model entry and enforce the §6.3 upward-import and cross-feature public-entry boundaries with negative fixtures.
- Keep runtime UX, provider/model compatibility, realtime entity hydration, backend routes, and live A2UI rendering unchanged.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `frontend-architecture-boundaries`: require one shared shell, feature-owned configuration surfaces, explicit dependency retirement, and mechanically enforced downward/public-entry imports.

## Impact

Frontend route composition, navigation inventory, A2UI/tools feature ownership, architecture gates, dependency manifests, and the terminal-theme CSS are affected. Backend, provider/model, protocol, persistence, and realtime wire contracts are unchanged. KBD canonical state advances from C-14c in progress to complete only after verification and independent review.
