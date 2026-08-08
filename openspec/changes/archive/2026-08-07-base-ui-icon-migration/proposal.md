# Change: Finish the Base UI Icon Migration

## Why

The Base UI foundation regeneration already migrated the 16 application wrappers
named by this change to `lucide-react` icons or icon-free Base UI indicators. A
live 2026-08-07 audit finds no `@radix-ui/react-icons` or direct `radix-ui`
imports under `frontend/src`, but the obsolete icon package remains a direct
frontend dependency and is still represented in both maintained pnpm lockfiles.

The change therefore needs reconciliation, dependency-graph cleanup, and focused
semantic verification rather than a second source rewrite.

## What Changes

- Audit all 16 named wrappers against their regenerated Lucide or native-indicator
  implementations.
- Preserve close-button labels, directional chevrons, checked/selected state,
  sizing, hit areas, and current styling.
- Remove `@radix-ui/react-icons` from `frontend/package.json`.
- Regenerate both `frontend/pnpm-lock.yaml` and the root `pnpm-lock.yaml` so each
  maintained workspace graph excludes the unused direct dependency.
- Keep inline SVG product artwork in `KnowMeLogo.tsx` and `lib/db-context.tsx`;
  generic interface-icon rules do not apply to logos or provider marks.

## Audited Wrapper Set

- `frontend/src/components/ui/pagination.tsx`
- `frontend/src/components/ui/resizable.tsx`
- `frontend/src/components/ui/accordion.tsx`
- `frontend/src/components/ui/input-otp.tsx`
- `frontend/src/components/ui/command.tsx`
- `frontend/src/components/ui/navigation-menu.tsx`
- `frontend/src/components/ui/sheet.tsx`
- `frontend/src/components/ui/dialog.tsx`
- `frontend/src/components/ui/carousel.tsx`
- `frontend/src/components/ui/breadcrumb.tsx`
- `frontend/src/components/ui/radio-group.tsx`
- `frontend/src/components/ui/sidebar.tsx`
- `frontend/src/components/ui/dropdown-menu.tsx`
- `frontend/src/components/ui/context-menu.tsx`
- `frontend/src/components/ui/menubar.tsx`
- `frontend/src/components/ui/checkbox.tsx`

## Acceptance Criteria

- `frontend/src` has zero imports from `@radix-ui/react-icons` or direct
  `radix-ui`.
- `frontend/package.json`, `frontend/pnpm-lock.yaml`, and the root
  `pnpm-lock.yaml` contain no `@radix-ui/react-icons` entry.
- Frozen installs succeed from both maintained workspace roots.
- Focused primitive tests demonstrate close, expansion, checked, and selected
  semantics without relying on icon implementation details.
- Frontend typecheck, lint, architecture-boundary, Flat 2.0, and strict OpenSpec
  gates pass.

## Out of Scope

- Renaming current Lucide imports only to remove an `Icon` suffix.
- Restyling primitives or changing their interaction areas.
- Replacing product logos or provider artwork with generic Lucide icons.
- Removing other Radix packages; those belong to later migration work.
