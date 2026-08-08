# C-14d dependency audit

Run date: 2026-08-08

## Application command implementation

- Before remediation: `cmdk@1.1.1` was a direct frontend dependency and declared `@radix-ui/react-compose-refs`, `@radix-ui/react-dialog`, `@radix-ui/react-id`, and `@radix-ui/react-primitive`.
- Resolution: the stable local `Command*` facade now uses `@base-ui/react/autocomplete`; `cmdk` is absent from `frontend/package.json`, both workspace lockfiles, and both resolved graphs.
- `pnpm remove cmdk` reported three resolved packages removed. No dependency was added because Base UI 1.6.0 was already installed.
- The repository-root `pnpm-lock.yaml` is authoritative for release/CI because its workspace includes `frontend`; isolated review found it stale, so it was regenerated and both root and nested frozen installs were rechecked.

## Accepted installed transitives

The root and nested `pnpm why` graphs report:

- The nested graph resolves `@radix-ui/react-dialog@1.1.15` → `vaul@1.1.2`;
  the authoritative root graph deduplicates that owner onto 1.1.19.
- `@radix-ui/react-dialog@1.1.19` → `radix-ui@1.6.2` → `@assistant-ui/react@0.14.26` (also consumed by `@assistant-ui/react-markdown@0.14.5`).
- `@radix-ui/react-tooltip@1.2.12` → `radix-ui@1.6.2` → `@assistant-ui/react@0.14.26`.

## Current-version check

Registry metadata reports `@assistant-ui/react@0.15.10` as current. It still declares `radix-ui@^1.6.7` and focused Radix helper dependencies, so upgrading would not remove the audited ownership. The installed 0.14.26 pin remains unchanged.

Official Assistant UI documentation supports Base UI-flavored generated components through `base-*` registry styles, but the published runtime package metadata remains the authoritative proof for its internal dependency graph.

## Prometheus Entity Management

Installed `@prometheus-ags/prometheus-entity-management@3.0.0-alpha.0` declares entity graph, TanStack virtual, clsx, immer, Lucide, tailwind-merge, and Zustand dependencies/peers. It declares no Radix package, and the `pnpm why` Radix paths do not name it.
