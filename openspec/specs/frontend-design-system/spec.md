# frontend-design-system Specification

## Purpose
TBD - created by archiving change tailwind4-css-first-tokens. Update Purpose after archive.
## Requirements
### Requirement: Tailwind uses the CSS-first Vite integration

The frontend SHALL use exact `tailwindcss` 4.3.3 and `@tailwindcss/vite` 4.3.3 packages, register the Tailwind Vite plugin, and import Tailwind through the shared CSS token source without a JavaScript Tailwind config or PostCSS config.

#### Scenario: Frontend tooling is inspected

- **WHEN** a contributor inspects the frontend build configuration
- **THEN** Tailwind is configured through the Vite plugin and `frontend/src/shared/theme/tokens.css`, and neither deleted legacy config file exists

### Requirement: The design token ladder is CSS-first and stable

The frontend SHALL define the KnowMe-aligned complete-color surface, text, brand, status, run-phase, typography, radius, easing, and duration tokens in `frontend/src/shared/theme/tokens.css` using Tailwind CSS-first theme directives.

#### Scenario: A downstream surface selects design roles

- **WHEN** a downstream UI change needs canvas, chrome, surface, raised, text, brand, status, or run-phase styling
- **THEN** the corresponding stable CSS theme token is available without adding JavaScript Tailwind configuration

### Requirement: Theme and legacy utility behavior is preserved during staging

The CSS-first foundation SHALL preserve dark, light, high-contrast, current semantic utility aliases, explicit A2UI source coverage, and live animation utility behavior while C-05 and C-14a retain ownership of legacy HSL-channel call-site conversion.

#### Scenario: Existing frontend source is compiled during the staged migration

- **WHEN** existing source still uses semantic utilities or legacy HSL-channel variables
- **THEN** C-02 supplies compatible generated utilities and variables without rewriting those deferred call sites

#### Scenario: Reduced-motion user loads the frontend

- **WHEN** the user prefers reduced motion
- **THEN** the token foundation preserves the reduced-motion duration override

### Requirement: Deleted configuration references do not dangle

All live frontend and CI configuration SHALL stop referencing `frontend/tailwind.config.ts` and `frontend/postcss.config.js`; Storybook visual regression SHALL trigger on the shared token source, and the component generator SHALL use the Tailwind 4 empty config-path value.

#### Scenario: Token-only change is proposed

- **WHEN** a pull request or main-branch push changes `frontend/src/shared/theme/tokens.css`
- **THEN** the Storybook visual-regression workflow is eligible to run

#### Scenario: Component generator reads frontend configuration

- **WHEN** the component generator reads `frontend/components.json`
- **THEN** it finds an empty Tailwind config path and the current CSS entry path

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

