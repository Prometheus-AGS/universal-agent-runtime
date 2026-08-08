## Context

The target frontend dependency direction ends in a React-free `platform/`
layer, but the current tree exposes AG-UI adapters from `protocols/`, the PGlite
client from `lib/`, and the entity-management package directly from 68 import
statements. C-04 is the physical adapter-boundary step. It must preserve runtime
behavior and avoid prematurely enforcing the broader feature/service zones that
C-14c owns.

## Goals / Non-Goals

**Goals:**

- Give AG-UI transport normalization, PGlite persistence, and entity graph
  integration explicit infrastructure ownership under `frontend/src/platform/`.
- Make one file the only application-owned importer of
  `@prometheus-ags/prometheus-entity-management`.
- Keep consumer-facing symbol names and behavior stable while import paths move.
- Add a deterministic, negative-fixture-tested gate for these three adapter
  boundaries.

**Non-Goals:**

- Move `lib/db-context.tsx`; it is a React provider and cannot belong to the
  React-free platform layer.
- Rename or redistribute the 23 REST service clients.
- Enforce the complete `app → features → shared → platform` zone graph before
  C-14c has moved the existing callers.
- Change database schema, AG-UI wire semantics, PEM APIs, dependencies, or
  user-visible UI.

## Decisions

### Preserve adapter APIs while moving ownership

Move the existing AG-UI files to `platform/agui/` with their current filenames
and move `lib/db.ts` to `platform/pglite/client.ts`. Update consumers directly;
do not leave forwarding modules at the retired paths.

Alternative considered: keep compatibility shims in `protocols/` and `lib/`.
Rejected because the change exists to make the ownership boundary real, and
shims would keep the retired entry points valid.

### Use one explicit PEM facade

`platform/entities/index.ts` will explicitly re-export only the runtime values
and types used by this repository. All application and test consumers will
import from `@/platform/entities`; no glob re-export is introduced.

Alternative considered: one wrapper file per entity feature. Rejected for this
change because it would mix the mechanical boundary move with API redesign and
duplicate package integration logic before the C-14 feature migration.

### Add a narrow adapter-boundary gate

Add `scripts/check-platform-adapters.mjs` plus negative fixtures. It will reject
retired AG-UI/PGlite paths and direct PEM package imports outside the sole
facade, and it will run from the existing CI grep-gate entry point. It will not
add service/store/component zone rules.

Alternative considered: extend the broad frontend boundary checker with all
target zones. Rejected because the plan records 46 live service import sites
that C-14 must move before those zones can be enforced.

## Risks / Trade-offs

- **Risk:** A mechanical rewrite misses a multiline or test-only PEM import. →
  Scan all TypeScript/TSX source and make the gate include tests, not only
  production files.
- **Risk:** Moving AG-UI files changes aliases or fixture resolution. → Move the
  focused tests with the adapters and run both test files after updating paths.
- **Risk:** Moving PGlite changes singleton or migration behavior. → Move the
  implementation verbatim and update all three known consumers/mocks; typecheck
  verifies its contract remains intact.
- **Trade-off:** The facade re-exports package hooks without redesigning them. →
  This is intentional staging; C-14 owns feature-local APIs and final zones.

## Migration Plan

1. Add the explicit entity-management facade.
2. Move AG-UI adapter/schema files and tests into `platform/agui/`.
3. Move the PGlite client to `platform/pglite/client.ts`.
4. Rewrite every application and test import to the new platform entry points.
5. Add and run the narrow adapter-boundary gate and negative fixtures.
6. Run moved focused tests, frontend typecheck/lint, existing boundary and Flat
   2.0 gates, and strict OpenSpec validation.

Rollback reverses the path moves and import rewrites. There is no persistent
data migration, dependency change, or protocol version change.

## Open Questions

None. C-14c remains the explicit owner of broader zone enforcement and service
redistribution.
