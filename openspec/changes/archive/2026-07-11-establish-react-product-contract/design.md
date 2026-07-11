## Context

The shipped first-party browser and Tauri interfaces are implemented in React 19 and TypeScript, while parts of the repository still describe HTMX and Web Components as the current product architecture. The React application also contains pre-existing dependency-direction violations across components, hooks, stores, and services. This change establishes one canonical frontend contract and a mechanical gate that prevents the violation set from growing while later vertical certifications remove it.

## Goals / Non-Goals

**Goals:**

- Record React 19 and TypeScript as the canonical first-party frontend.
- Define Component → Hook/View Model → Store/Entity Domain → Service → API as the mandatory dependency direction.
- Inventory live product surfaces without promoting uncertified behavior to GA support.
- Add a deterministic, dependency-free boundary checker with an exact, shrinking legacy allowlist.

**Non-Goals:**

- Refactor every existing frontend boundary violation in this change.
- Certify individual routes, providers, protocols, persistence backends, or platforms.
- Change runtime behavior or introduce a new frontend framework or dependency.

## Decisions

### Canonical architecture is documented in an ADR and living guide

The ADR records the durable product decision; `docs/frontend-architecture.md` defines operational ownership and import rules for contributors. Keeping both separates decision history from the living implementation contract. Relying on repository guidelines alone was rejected because they do not resolve contradictory public and historical documentation.

### Surface maturity is explicit and conservative

The inventory maps live routes to owners, backend contracts, maturity, and the focused certification change that must provide executable evidence. Uncertified surfaces remain Preview or Experimental. Treating code presence or catalog presence as support evidence was rejected because it overstates customer-ready behavior.

### Enforcement uses a repository-local static checker

`scripts/check-frontend-boundaries.mjs` scans production TypeScript sources for prohibited direct fetches and imports. It has no external parser dependency and compares findings with an exact allowlist. A new violation or stale allowlist entry fails. Adding an ESLint plugin was rejected for this baseline because the repository already has the needed Node runtime and the rules are narrow; the checker can later be replaced if syntax coverage requires an AST.

### Existing debt is frozen, not normalized

Every detected pre-existing violation is recorded as `path|rule`. Entries may only be removed unless a later architecture decision explicitly changes the contract. This enables a blocking no-regression gate now while `close-react-boundary-gate` drives the count to zero before GA.

## Risks / Trade-offs

- [Regex-based import detection can miss unusual syntax] → Keep rules narrow, prove representative negative fixtures, and migrate to an existing parser if real syntax escapes the gate.
- [The initial inventory can become stale as routes change] → Certification changes update the inventory and support matrix alongside executable evidence.
- [An allowlist can become permanent debt] → Fail on stale entries and make zero production entries an explicit dependency of GA certification.
- [Historical documents can continue to contradict the ADR] → `reconcile-product-documentation` will label or archive historical material and add truth gates.

## Migration Plan

1. Land the ADR, architecture guide, inventory, and support-matrix skeleton.
2. Capture the current violation set and enable the checker in local and CI grep gates.
3. Remove allowlist entries during vertical feature certifications.
4. Make the production allowlist empty in `close-react-boundary-gate` before release certification.

Rollback is limited to reverting the documentation and checker commit; this change does not alter runtime behavior or persisted data.

## Open Questions

- Whether the final zero-violation checker should remain standalone or be consolidated into ESLint after migration.
- Which currently Preview surfaces will meet GA evidence requirements versus remain explicitly Experimental.
