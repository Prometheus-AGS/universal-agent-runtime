## ADDED Requirements

### Requirement: Migrated non-admin surfaces consume semantic color tokens

The frontend SHALL express the C-05 non-admin color call sites through complete
semantic `--color-*` values and SHALL preserve the rendered role and alpha of
each replaced legacy HSL-channel expression.

#### Scenario: Migrated source is inspected

- **WHEN** the shared stylesheet, assistant thread, shared admin components,
  and KnowMe logo are checked after C-05
- **THEN** the measured migration file set contains no `hsl(var(` call site and
  consumes semantic color values instead

#### Scenario: An alpha color is migrated

- **WHEN** a legacy channel call site included an alpha component
- **THEN** the semantic replacement preserves the same alpha percentage

### Requirement: Admin-page token migration remains staged

C-05 SHALL NOT rewrite legacy HSL-channel call sites under
`frontend/src/admin/pages/`; those call sites SHALL remain owned by C-14a while
shared non-admin admin components may migrate through scoped semantic aliases.

#### Scenario: C-05 scope is verified

- **WHEN** the token codemod is complete
- **THEN** no `frontend/src/admin/pages/` source file is changed by C-05 and the
  deferred legacy occurrences remain available for C-14a

### Requirement: Token migration regression is mechanically rejected

The repository SHALL run a deterministic C-05 scope check from the root CI
grep-gate harness and SHALL keep the exact Flat 2.0 baseline synchronized with
source strings changed by the token migration.

#### Scenario: Legacy syntax returns to a migrated file

- **WHEN** a contributor adds `hsl(var(` to one of the six C-05 migration files
- **THEN** the root grep-gate harness fails and identifies the remaining file

#### Scenario: A changed string retains unrelated Flat 2.0 debt

- **WHEN** token syntax changes inside a still-allowlisted border string
- **THEN** the baseline records the new exact source string without hiding or
  inventing a diagnostic
