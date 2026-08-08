## 1. Rule Contract and Dependency

- [x] 1.1 Add `eslint-plugin-unicorn@73.0.0` and preserve both maintained workspace lockfiles.
- [x] 1.2 Define the shared Flat 2.0 selectors and kebab-case filename rule options.
- [x] 1.3 Enable both rules in `frontend/eslint.config.js` with exact legacy-file overrides and generated-output ignores.

## 2. Shrinking Legacy Baseline

- [x] 2.1 Add an unsuppressed ESLint baseline config and a deterministic root checker that compares normalized findings with an explicit allowlist.
- [x] 2.2 Capture every current Flat 2.0 syntax and filename finding without modifying component source.
- [x] 2.3 Reject both new findings and stale allowlist entries, including duplicate findings in one file.

## 3. Gate Proof and CI Wiring

- [x] 3.1 Add negative fixtures proving prohibited style syntax and a PascalCase filename are rejected.
- [x] 3.2 Wire the style checker and negative-fixture proof into the existing root CI grep-gate harness and package scripts.

## 4. Validation

- [x] 4.1 Run the positive baseline gate, negative fixtures, frontend typecheck/lint, and the existing frontend boundary checks.
- [x] 4.2 Run strict OpenSpec validation, scoped diff checks, artifact refinement, and isolated adversarial review.
