## ADDED Requirements

### Requirement: Flat 2.0 syntax is mechanically gated

The frontend SHALL configure ESLint `no-restricted-syntax` selectors that reject border and divider utilities, one-pixel rings, layout shadows, backdrop blur, background gradients, and JSX `variant="outline"` usage according to the approved Flat 2.0 contract.

#### Scenario: New product source introduces prohibited visual separation

- **WHEN** a contributor adds a prohibited Flat 2.0 syntax finding to TypeScript or TSX product source under `frontend/src`
- **THEN** the style gate fails and identifies the file and rule

### Requirement: Frontend filenames converge on kebab-case

The frontend SHALL configure `unicorn/filename-case` to require kebab-case filenames and directories for TypeScript and TSX product source under `frontend/src`, subject only to exact legacy findings tracked during migration.

#### Scenario: New source uses a non-kebab-case path

- **WHEN** a contributor adds a TypeScript or TSX product-source path under `frontend/src` that violates the kebab-case contract
- **THEN** the style gate fails before the path can expand the legacy baseline

### Requirement: Legacy style debt is exact and shrinking

The style gate SHALL compare unsuppressed ESLint diagnostics with an explicit allowlist, preserve duplicate finding counts, and fail for both unexpected diagnostics and stale allowlist entries.

#### Scenario: A violation is added inside an already-allowlisted file

- **WHEN** an allowlisted legacy file gains another prohibited syntax finding
- **THEN** the unsuppressed checker reports a new diagnostic even though normal ESLint uses an exact-file migration override

#### Scenario: Migration removes a legacy finding

- **WHEN** a downstream change resolves an allowlisted finding
- **THEN** the style gate fails until the stale allowlist entry is removed

### Requirement: Gate behavior is proven and integrated

The repository SHALL include negative fixtures for prohibited Flat 2.0 syntax and filename casing and SHALL run the positive and negative style checks from the existing root CI grep-gate harness.

#### Scenario: CI runs architectural grep gates

- **WHEN** the root CI grep-gate script executes
- **THEN** it verifies the current baseline and proves representative invalid fixtures are rejected
