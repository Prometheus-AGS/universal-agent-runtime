## Why

Frontend infrastructure adapters currently live under generic `lib/` and
`protocols/` paths, while direct entity-management imports are spread across
application features, stores, tests, and helpers. This obscures ownership and
prevents the target `app → features → shared → platform` dependency direction
from becoming enforceable in later migration changes.

## What Changes

- Move the AG-UI schema, adapter, and their focused tests from `protocols/` to
  `platform/agui/` without changing event reduction or replay behavior.
- Move the PGlite client from `lib/db.ts` to `platform/pglite/client.ts` while
  leaving the React `db-context.tsx` outside the platform layer.
- Add one explicit `platform/entities` facade as the sole application-owned
  import site for `@prometheus-ags/prometheus-entity-management`, and route all
  current consumers through it.
- Add a focused adapter-boundary check that prevents the retired paths and
  direct package imports from returning.
- Do not move REST clients or install the full architectural zone rules; those
  remain sequenced for C-14c.

## Capabilities

### New Capabilities

- `frontend-architecture-boundaries`: Defines infrastructure-adapter ownership
  and the allowed dependency entry points for AG-UI, PGlite, and entity graph
  integration.

### Modified Capabilities

None.

## Impact

- **Frontend code:** import paths and physical ownership change across AG-UI,
  PGlite, entity helpers, stores, admin pages, and tests; runtime APIs remain
  unchanged.
- **Runtime UX and realtime state:** no user-visible behavior change; AG-UI
  ordering, replay, approval, lifecycle, and state-patch semantics are preserved
  by the moved focused tests.
- **Provider compatibility:** unchanged; provider and model stores retain the
  same entity-management APIs through the platform facade.
- **Dependencies and persistence:** no package or schema changes; the existing
  PGlite database name, migrations, singleton lifecycle, and localStorage
  migration remain intact.
- **KBD workflow:** C-04 start and completion are recorded through canonical
  `prometheus kbd change transition`; no product runtime consumes KBD state.
