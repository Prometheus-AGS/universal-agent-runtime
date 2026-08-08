## Context

The binding UI contract requires Flat 2.0 surfaces separated by fill and spacing, not borders, dividers, layout shadows, backdrop blur, gradients, or outline variants. The migration also requires kebab-case filenames. The current frontend predates those rules, so enabling either rule without a baseline would create a flag day and block the dependency-ordered migration.

ESLint already supplies the approved AST-selector rule, and `eslint-plugin-unicorn` supplies path-aware filename enforcement. The repository's boundary gate establishes the local pattern for staged migrations: derive exact findings, compare them with an explicit allowlist, reject both additions and stale entries, and prove the rejection path with negative fixtures.

## Goals / Non-Goals

**Goals:**

- Make the approved Flat 2.0 syntax selectors and kebab-case filename rule part of the normal frontend ESLint configuration.
- Freeze the complete current finding set without changing component source.
- Fail when a new finding appears, including inside a file that already has legacy debt.
- Fail when migration work resolves an entry but the allowlist is not reduced.
- Use ESLint itself as the diagnostic engine for both normal lint and the baseline checker.

**Non-Goals:**

- Remove existing visual-style violations or rename existing files.
- Change shared primitives, variants, component markup, tokens, or runtime rendering.
- Add broader Unicorn recommended rules.
- Replace the existing frontend architecture boundary gate.

## Decisions

### Share one rule contract between normal lint and baseline measurement

The rule options live in one frontend config module. The normal ESLint config and a baseline-only ESLint config both import those options. The baseline checker invokes ESLint with the unsuppressed baseline config and compares normalized diagnostics with the allowlist. This avoids a second regex implementation whose coverage could drift from the configured AST selectors.

Alternative considered: scan class strings with a separate regular expression. Rejected because the gate could then disagree with `no-restricted-syntax`, especially around TypeScript/JSX AST shapes.

### Suppress by exact legacy file only in normal lint

The normal config enables both rules for all TypeScript source, then disables an individual rule only for exact file paths that currently contain that rule's allowlisted findings. The unsuppressed checker still scans every file and compares every diagnostic, so adding another violation inside an allowlisted file fails the style gate.

Alternative considered: use broad directory ignores or Unicorn's path-segment ignore option. Rejected because those forms can silently exempt new files.

### Normalize findings by source rather than line number

Flat 2.0 syntax entries use repository path, rule ID, and the exact AST source fragment, with deterministic occurrence numbering for duplicates. Filename entries use repository path and rule ID. This preserves exact counts while avoiding allowlist churn when unrelated lines move above a finding.

### Keep generated artifacts outside product lint

Coverage reports, Playwright/Chromatic results, and the deliberately-invalid gate fixtures are not product source. They are globally ignored by the normal frontend lint config; the negative-fixture test invokes the baseline config explicitly.

## Risks / Trade-offs

- **Risk:** File-level ESLint overrides could hide additions in a legacy file during normal lint. → **Mitigation:** The root style checker scans those files with the unsuppressed rules and is part of the same CI harness.
- **Risk:** An allowlist becomes permanent debt. → **Mitigation:** Stale entries fail, and downstream migration changes are required to shrink it as findings disappear.
- **Risk:** Diagnostic normalization loses duplicate findings. → **Mitigation:** Identical source signatures receive deterministic occurrence suffixes, preserving multiplicity.
- **Trade-off:** The allowlist is large. → It is generated, reviewable evidence of the existing debt and is preferable to a broad exemption.

## Migration Plan

1. Add the compatible Unicorn dependency and shared Flat 2.0 rule contract.
2. Add the unsuppressed baseline config, checker, exact allowlist, and negative fixtures.
3. Enable the rules in normal frontend lint with exact-file legacy overrides.
4. Wire the checker and negative proof into the root CI grep-gate harness.
5. Run targeted checks, frontend typecheck/lint, boundary gates, and strict OpenSpec validation.

Rollback removes the plugin, rule contract, allowlist/checker, fixtures, and CI hook. No runtime or persisted-data migration exists.

## Open Questions

None. The phase plan fixes the selectors, filename case, staged allowlist strategy, and downstream ownership.
