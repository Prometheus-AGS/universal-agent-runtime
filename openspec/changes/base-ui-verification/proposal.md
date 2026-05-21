# Change: Base UI Verification — cmdk Audit, assistant-ui Audit, E2E Smoke Tests

## Why

After the three prior changes (foundation swap, icon migration, composition patterns), the
project's own code should have zero Radix UI imports. However, two third-party packages
carry risk of reintroducing Radix as transitive dependencies:

1. **`cmdk`** — the Command palette library (`command.tsx`) has a known Radix peer dependency
   in v1.x. If it still imports `@radix-ui/*` internally, `command.tsx` must be replaced with
   the Base UI native `Combobox` component.

2. **`@assistant-ui/react`** — wraps its own Radix internals. If it reintroduces Radix
   transitively, this is accepted (we cannot control third-party packages). We document it
   and file an upstream issue if Base UI–compatible versions are available.

This change performs the final audit and runs full E2E smoke tests to confirm the migrated
frontend is functionally equivalent to the pre-migration state.

## What Changes

### If cmdk audit fails (cmdk still imports Radix)
- Remove `cmdk` from `frontend/package.json`
- Rewrite `frontend/src/components/ui/command.tsx` using Base UI native `Combobox`:
  ```tsx
  import * as Combobox from '@base-ui/react/combobox';
  ```
  The new `Command` component must maintain the same external API:
  `Command`, `CommandInput`, `CommandList`, `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandSeparator`
  so that all callers remain unchanged.

### assistant-ui audit result (documentation only)
- If `pnpm why @radix-ui/react-dialog` shows `@assistant-ui/react` as the source:
  - Accept — transitive from third-party
  - Create `frontend/docs/third-party-radix-note.md` explaining the situation
  - Check `@assistant-ui/react` releases for Base UI–compatible version; upgrade if available

### E2E Smoke Tests
- Run existing Playwright suite: `pnpm test:e2e`
- Manual smoke tests for flows not yet covered by E2E:
  - Chat message send via model-selector
  - Agent selector dropdown opens and selects
  - Admin settings form submit
  - Sidebar open/close on mobile viewport

## Acceptance Criteria

- [ ] `pnpm why @radix-ui/react-dialog` output documented (pass or transitive-only)
- [ ] `grep -r "@radix-ui" frontend/src/` returns zero results in **own code**
- [ ] `cmdk` either removed or confirmed Radix-free
- [ ] If `cmdk` replaced: `command.tsx` external API unchanged (no callers broken)
- [ ] `pnpm typecheck` passes
- [ ] `pnpm lint` passes
- [ ] `pnpm test:e2e` passes (or failures documented as pre-existing)
- [ ] Manual smoke: chat, agent-selector, admin form, sidebar all functional
- [ ] `frontend/src/components/ui-radix-backup/` removed (cleanup after confidence)
