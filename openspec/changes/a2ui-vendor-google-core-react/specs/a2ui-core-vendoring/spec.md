# A2UI core vendoring

## Purpose

Make Google's A2UI project libraries (`@a2ui/web_core`, `@a2ui/react`)
available to the UAR frontend workspace as pinned, license-disclosed
dependencies, behind a stable internal import path, so later A2UI
changes (Change 17 onward) have a known-good foundation to build on.

## ADDED Requirements

### Requirement: `@a2ui/web_core` is vendored as an exact-pinned dependency
`frontend/packages/a2ui-core/` SHALL exist as a `pnpm` workspace
package (matched by `frontend/pnpm-workspace.yaml`'s `packages/*`
glob) whose only runtime dependency is `@a2ui/web_core`, pinned to an
**exact** version (no `^`/`~`/range) in `package.json`. The package
SHALL re-export `@a2ui/web_core`'s public surfaces (at minimum the
default export and `./v0_9`) under the internal package name
`@prometheus-ags/a2ui-core`.

#### Scenario: A later change imports the vendored core library
- **WHEN** code elsewhere in `frontend/` needs A2UI core rendering
  primitives (message processing, data/component/surface models)
- **THEN** it imports from `@prometheus-ags/a2ui-core` (or its `/v0_9`
  subpath), not directly from `@a2ui/web_core`
- **AND** `pnpm -C frontend install` resolves the exact pinned
  version recorded in `frontend/packages/a2ui-core/package.json`

#### Scenario: The upstream version is bumped
- **WHEN** a maintainer updates the pin in
  `frontend/packages/a2ui-core/package.json`
- **THEN** the change is a single exact-version string edit (not a
  multi-file diff of vendored source), and `UPSTREAM.md`'s pinned
  version table is updated in the same change
- **AND** `pnpm -C frontend install` regenerates `pnpm-lock.yaml` to
  match

### Requirement: `@a2ui/react` is vendored as reference-implementation-only
`frontend/packages/a2ui-react/` SHALL exist as a `pnpm` workspace
package whose only runtime dependency is `@a2ui/react`, pinned to an
exact version. The package's `package.json` `description`, its
`README.md`, and the top-of-file comment in its entry module MUST
each state explicitly that it is a reference implementation and MUST
NOT be imported from UAR product code.

#### Scenario: A developer considers importing the reference package into product code
- **WHEN** a developer looks at `frontend/packages/a2ui-react/`
- **THEN** the package's `package.json` description, `README.md`
  banner, and `src/index.ts` header all state it is reference-only
  and point to the UAR-owned renderer package
  (`@prometheus-ags/a2ui-uar`, delivered by a separate change) as the
  package product code should depend on instead

#### Scenario: A future change cross-tests the UAR renderer against the reference
- **WHEN** the UAR-owned A2UI renderer (delivered by a later change)
  needs to verify semantic parity against Google's official renderer
- **THEN** it imports `@prometheus-ags/a2ui-react` for that
  cross-test only, using the same exact-pinned version discipline as
  `@prometheus-ags/a2ui-core`

### Requirement: Upstream Apache-2.0 license and copyright are preserved
Both vendored packages MUST include a verbatim copy of the upstream
`LICENSE` file (Apache-2.0). Every re-export module MUST carry the
upstream `Copyright 2025 Google LLC` / Apache-2.0 header. The
packages MUST NOT strip, edit, or relicense any upstream file
resolved through `node_modules`.

#### Scenario: A license audit inspects the vendored packages
- **WHEN** a license audit tool or reviewer inspects
  `frontend/packages/a2ui-core/` and `frontend/packages/a2ui-react/`
- **THEN** each package directory contains a `LICENSE` file with the
  full Apache License, Version 2.0 text
- **AND** each `src/*.ts` re-export module's header block states
  `Copyright 2025 Google LLC` and the Apache-2.0 grant text
- **AND** neither package's `package.json` declares a license other
  than `Apache-2.0`

### Requirement: The pinned version and update procedure are documented
`frontend/packages/a2ui-core/UPSTREAM.md` MUST record: the exact
pinned version of `@a2ui/web_core`, how that version was confirmed
(command run, date), the npm tarball's shasum, the rationale for
vendoring via a pinned dependency rather than a source copy, and a
numbered procedure for bumping the pin in the future.

#### Scenario: A maintainer needs to update the vendored core library
- **WHEN** a maintainer opens `frontend/packages/a2ui-core/UPSTREAM.md`
- **THEN** they find the currently pinned version, the exact npm
  commands to check for a newer release, and the steps to bump the
  pin, regenerate the lockfile, and re-verify typecheck
- **AND** they do not need to consult any source outside this repo
  to understand why the package is structured as a pinned dependency
  rather than a copied source tree

### Requirement: Vendored packages do not regress workspace verification
Adding the two vendored packages MUST NOT break
`pnpm -C frontend typecheck` or `pnpm -C frontend lint` for the rest
of the workspace, and each vendored package MUST independently pass
its own `typecheck` script.

#### Scenario: CI runs the standard frontend verification commands
- **WHEN** `pnpm -C frontend install`, `pnpm -C frontend typecheck`,
  and `pnpm -C frontend lint` run after this change lands
- **THEN** all three commands succeed with the two new packages
  present in `frontend/packages/`
- **AND** `pnpm --filter @prometheus-ags/a2ui-core typecheck` and
  `pnpm --filter @prometheus-ags/a2ui-react typecheck` each succeed
  independently
