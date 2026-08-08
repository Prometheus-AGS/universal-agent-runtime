# C-14c migration inventory

## Remaining legacy admin ownership

- `frontend/src/admin/admin-shell.tsx`: duplicate inner navigation, command palette, responsive overlay, and runtime feed subscription.
- `frontend/src/admin/A2uiTestingPage.tsx`: C-12-required development-only utility; move to `features/a2ui`.
- `frontend/src/admin/McpHealthPage.tsx`: MCP health surface; move to `features/tools`.
- `frontend/src/pages/admin-page.tsx`: route composition and terminal-theme mutation; retain composition only.

## Technical-layer clusters to re-home

- A2UI: `hooks/use-a2ui-schemas.ts`, `stores/a2ui-schemas-store{,.test}.ts`, `services/a2ui-api.ts`.
- MCP: `hooks/use-mcp-health.ts`, `stores/mcp-health-store{,.test}.ts`, `services/mcp-api.ts`, `entities/fetchers/mcp-status.ts`, `entities/hooks/use-mcp-status.ts`.

## Dependency baseline

- Product source has zero direct `@radix-ui/*` imports.
- `@tanstack/react-query` has one provider-only import in `App.tsx`.
- `highlight.js` has zero surviving product-source imports after C-09/C-12.
- The live manifest has 26 direct `@radix-ui/*` declarations (the plan's 27 count is stale); transitive Radix ownership must be rechecked after removal.

## Store disposition

The admin-domain stores moved or deleted during C-14a remain deleted. The two surviving admin-owned stores are not behaviorally retired: their technical-layer paths are retired and their ownership moves into A2UI/tools features. Unrelated chat, thread, theme, health, upload, approval, and shell stores remain outside C-14c.

## Boundary baseline

The existing gate enforces component/hook/store/service layering but not directory direction. Current intentional cross-feature entry points include root, `api`, and `model` indexes; direct cross-feature implementation imports must be remediated before the new gate is enabled.
