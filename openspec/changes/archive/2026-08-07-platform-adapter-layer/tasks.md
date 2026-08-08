## 1. Workflow and inventory

- [x] 1.1 Record C-04 as in progress in canonical KBD state.
- [x] 1.2 Create and strictly validate the proposal, design, and architecture-boundary delta.
- [x] 1.3 Inventory AG-UI files, PGlite consumers, and all direct PEM imports.

## 2. Entity-management facade

- [x] 2.1 Add one explicit `platform/entities` facade for the used PEM runtime values and types.
- [x] 2.2 Rewrite production PEM consumers to import from `@/platform/entities`.
- [x] 2.3 Rewrite test and setup PEM consumers to import from `@/platform/entities`.
- [x] 2.4 Confirm the facade is the sole direct package import site.

## 3. AG-UI adapter ownership

- [x] 3.1 Move the AG-UI schema and adapter to `platform/agui/`.
- [x] 3.2 Move their focused tests and update all AG-UI consumer imports.
- [x] 3.3 Run the moved schema and adapter tests.

## 4. PGlite adapter ownership

- [x] 4.1 Move `lib/db.ts` and its asset loader to `platform/pglite/` without behavioral edits.
- [x] 4.2 Update the React provider, stores, and test mocks to the moved client path.
- [x] 4.3 Confirm no retired `@/lib/db` or `protocols/agui-*` entry point remains.

## 5. Boundary enforcement

- [x] 5.1 Add the platform-adapter gate for sole PEM ownership, retired paths, and React-free platform files.
- [x] 5.2 Add negative fixtures that prove each platform-adapter rule rejects a violation.
- [x] 5.3 Wire the platform-adapter gate into the existing CI grep-gate entry point.

## 6. Validation

- [x] 6.1 Run frontend typecheck and lint.
- [x] 6.2 Run architecture-boundary, Flat 2.0, and platform-adapter gates with negative fixtures.
- [x] 6.3 Strictly validate the OpenSpec change and scoped diff.
