## Context

The Base UI foundation regeneration already converted the 16 wrappers named by
the proposal to `lucide-react` icons or removed icons that were no longer needed.
A 2026-08-07 live audit found zero `@radix-ui/react-icons` or direct `radix-ui`
imports under `frontend/src`. The obsolete icon package remains declared in
`frontend/package.json` and represented in both `frontend/pnpm-lock.yaml` and
the root `pnpm-lock.yaml`, so the maintained dependency graphs still carry a
direct Radix package with no application consumer.

## Goals / Non-Goals

**Goals:**

- Verify the regenerated source covers every file and icon mapping in the
  original 28-task change.
- Remove the unused direct `@radix-ui/react-icons` dependency through pnpm so
  package metadata and the lockfile remain coherent.
- Preserve existing Lucide icon names, accessible labels, directionality, state
  meaning, sizing, and visual output.

**Non-Goals:**

- Replace custom product/logo SVG artwork with generic icons.
- Rename current Lucide imports merely to drop their `Icon` suffix.
- Restyle icon buttons, change hit areas, or migrate transitive dependencies
  owned by third-party packages.
- Introduce another icon package.

## Decisions

### Treat the regenerated source as verified prerequisite work

The 16 listed wrapper files will be audited against their current Lucide imports
and runtime tests, not mechanically rewritten. Source mutation is limited to the
observed unused dependency declaration and lockfile nodes.

Alternative considered: replay every original mapping edit. Rejected because it
would churn already-correct generated source and could alter established icon
semantics or styling.

### Remove the dependency with pnpm

`pnpm -C frontend remove @radix-ui/react-icons` will update
`frontend/package.json` and the frontend lockfile using the configured package
manager. A root lockfile-only install will then reconcile the root workspace
graph. Frozen-lockfile validation from both roots proves each graph is internally
consistent.

Alternative considered: hand-edit the manifest and lockfile. Rejected because
the package manager is the authoritative lockfile writer and the live task
explicitly requires package removal.

### Scope the zero-Radix gate to application ownership

Acceptance checks scan `frontend/src`, the frontend manifest, and the frontend
lockfile for the removed package. Transitive implementation details inside
`node_modules` are not application source and remain controlled by their owning
packages.

## Risks / Trade-offs

- **Risk:** A listed file retained a Radix icon through an import form missed by
  the old proposal. → Scan both package names and `radix-ui` across all TypeScript
  and TSX application source.
- **Risk:** Removing the dependency leaves a stale lockfile graph. → Use pnpm for
  removal and run a frozen-lockfile install check.
- **Trade-off:** The root and nested frontend workspaces maintain independent
  lockfiles and can resolve different compatible transitive versions. → Validate
  both graphs with frozen installs; run frontend behavior/type/lint gates against
  the nested frontend graph that owns the application package.
- **Risk:** Visual behavior changed during the earlier regeneration. → Preserve
  current source and use the existing focused primitive interaction suite plus
  explicit source inspection; Wave 1 owns full visual-regression execution.

## Migration Plan

1. Reconcile the proposal and 28-task checklist with the live icon census.
2. Add the `frontend-component-primitives` delta for icon-family ownership.
3. Remove `@radix-ui/react-icons` with pnpm and reconcile both lockfiles.
4. Add the source/package exclusion rules to the existing CI grep gate.
5. Run source/package exclusion checks, frozen-lockfile validation, frontend
   typecheck/lint, boundary and Flat 2.0 gates, focused primitive tests, and
   strict OpenSpec validation.

Rollback restores the manifest dependency and regenerated lockfile nodes; no
runtime data or protocol migration exists.

## Open Questions

None. The live source census and installed package-manager contract resolve the
stale proposal assumptions.
