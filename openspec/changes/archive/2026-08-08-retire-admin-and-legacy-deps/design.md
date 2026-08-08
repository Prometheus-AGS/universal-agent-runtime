## Context

C-14a moved thirteen configuration pages and their domain ownership into `features/*`; C-14b decomposed settings. The remaining `src/admin/` tree is an inner navigation shell plus two surfaces. The top-level C-10 application shell already owns navigation, breadcrumbs, command discovery, compact behavior, and responsive overlays, so retaining a second shell duplicates navigation and keeps the terminal theme alive. C-12 requires the standalone A2UI tester to remain development-only, so deleting its admin path cannot mean deleting that capability.

There are no direct `@radix-ui/*` imports in product source. The live manifest contains 26 direct declarations (the plan's 27 count was stale), so all 26 are eligible for removal while Radix packages required transitively by `cmdk`, Assistant UI, or other packages must remain lockfile-resolved. TanStack Query has one provider-only call site and highlight.js has no surviving source import after Shiki landed in C-09.

## Goals / Non-Goals

**Goals:**

- Render every retained `/admin/*` route directly inside the shared application shell.
- Preserve the development-only A2UI tester and MCP health behavior under feature ownership.
- Delete the legacy admin tree, terminal theme, dead technical-layer files, and unused direct dependencies.
- Enforce the binding §6.3 matrix—platform/shared cannot import features/app, features cannot import app—plus cross-feature public-entry imports with deterministic negative proofs.

**Non-Goals:**

- Redesigning a migrated page, changing route URLs, or changing backend/provider/realtime contracts.
- Removing transitive Radix packages that retained dependencies still require.
- Migrating unrelated chat/application stores and services beyond the two remaining admin-owned clusters.

## Decisions

### Direct route composition replaces the inner admin shell

`AdminPage` resolves the path to the existing feature page and mounts the runtime feed hook from `features/runtime/model`. The outer C-10 shell remains the single navigation and responsive owner. Keeping a reduced inner shell was rejected because it would preserve duplicate navigation and the terminal-theme contract.

### A2UI and MCP surfaces move rather than disappear

The A2UI tester, its hook/store/API, and the MCP health page, hook/store/API/entity adapter move into `features/a2ui` and `features/tools`. This preserves the C-12 development-only contract and the observed MCP health route while deleting the `admin/`, top-level store/hook/service, and last legacy entity-fetcher ownership.

### Dependency retirement follows direct-import and lockfile evidence

All direct Radix declarations, TanStack Query, and highlight.js are removed in one dependency-manager operation only after source inspection proves zero required imports. Post-removal `pnpm why` and lockfile inspection must show that any remaining Radix packages are transitive rather than direct.

### Extend the existing boundary gate instead of adding a lint dependency

The repository already has a deterministic import parser and negative-fixture harness. It will reject upward layer imports and cross-feature implementation imports while allowing same-feature imports and deliberate root, `api`, or `model` index entries. Adding `eslint-plugin-import` solely for equivalent path zones was rejected as unnecessary dependency and bundle-independent toolchain churn.

## Risks / Trade-offs

- [Route mapping drift] → Keep one typed route map, preserve every retained key, and test representative/default/development-only resolution.
- [Removing direct dependencies breaks a transitive consumer] → Regenerate the frozen lockfile, retain transitive packages, and run type/lint/build gates at completion.
- [A broad runtime barrel pulls page code into the initial graph] → Import the feed hook through the narrow runtime model module as required by the C-14a handoff.
- [Boundary rules outlaw intentional cross-domain coordination] → Permit only explicit public root, `api`, and `model` entries and remediate current implementation-path imports before enabling the gate.

## Migration Plan

1. Capture route, dependency, direct-import, retired-file, and protected-path baselines.
2. Re-home A2UI and MCP ownership, replace the inner shell, and remove terminal-theme routing.
3. Remove verified-unused direct dependencies and regenerate the lockfile.
4. Install boundary checks and negative fixtures, then run scoped and completion gates.
5. Record canonical completion, sync the capability delta, and archive the change.

Rollback is the ordinary source diff: the change does not migrate durable data or alter a backend contract.

## Open Questions

None. The C-12 development-only A2UI contract resolves the apparent ambiguity in “delete `src/admin/`”: ownership moves, while production discovery remains disabled.
