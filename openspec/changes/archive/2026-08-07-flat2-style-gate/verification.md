# Verification: flat2-style-gate

Date: 2026-08-07
Phase change: C-03

## Summary

| Dimension | Status |
|---|---|
| Completeness | 10/10 tasks complete; 4/4 requirements mapped |
| Correctness | 5/5 specified scenarios have executable evidence |
| Coherence | Implementation follows the exact, shrinking-allowlist design; no component source changed |

## Acceptance evidence

1. `frontend/eslint.config.js` enables the approved `no-restricted-syntax` selectors and `unicorn/filename-case` for TypeScript/TSX source.
2. `frontend/eslint-flat2-contract.js` is the single rule-option source imported by both normal lint and the unsuppressed baseline configuration.
3. The explicit allowlist contains 400 unique findings: 384 Flat 2.0 syntax diagnostics and 16 filename diagnostics. The published census command remains 630 border idioms; C-03 changes no component source.
4. Normal lint disables a rule only for exact paths represented by that rule in the legacy allowlist. The standalone checker scans those paths without suppression and compares exact source-fragment diagnostics, including deterministic duplicate occurrence numbers.
5. The negative proof exercises normal-config rejection, unsuppressed syntax and filename detection, an added diagnostic inside an already-allowlisted file, and a resolved/stale allowlist entry.
6. Generated coverage, test-result, Storybook, package-workspace, and deliberate fixture outputs are excluded from normal product lint; `pnpm -C frontend lint` now passes.
7. `eslint-plugin-unicorn` is pinned to 73.0.0. Its Node >=22 and ESLint >=10.4 requirements are satisfied by Node 24.16.0 and frontend ESLint 10.7.0, and both maintained lockfiles accept frozen installation.
8. The root CI grep-gate harness runs both the positive baseline and negative proof and passes in full.

## Review-scope provenance

This repository intentionally carries the completed but uncommitted C-00/C-02 phase work in the same worktree. Consequently, the raw tracked `frontend/package.json` and lockfile diff against `HEAD` contains Tailwind 4, Vite, and Chromatic hunks owned and verified by archived changes `2026-08-07-tailwind4-css-first-tokens` and the C-00 archive set. C-03's dependency delta is only exact `eslint-plugin-unicorn` 73.0.0 plus its lockfile resolution. Its corrected review packet uses the C-03 file inventory and current artifact snapshots so those earlier accepted hunks are not misattributed to this gate.

## Scenario mapping

- **New product source introduces prohibited visual separation:** stdin-based normal ESLint proof rejects a new kebab-case TSX path containing `border`; the baseline negative fixture rejects prohibited literal and template-literal utilities plus both literal and expression-container outline variants.
- **New source uses a non-kebab-case path:** stdin-based normal ESLint proof rejects `src/NewFlat2Surface.tsx`, and the baseline fixture reports `PascalFixture.tsx`.
- **A violation is added inside an already-allowlisted file:** the negative test supplies a partial fixture allowlist and confirms the second diagnostic in that same file is reported as new.
- **Migration removes a legacy finding:** the negative test adds a resolved entry to a complete fixture allowlist and confirms the gate reports it as stale.
- **CI runs architectural grep gates:** `bash scripts/ci-grep-gates.sh` runs the style checks alongside existing architecture/aesthetic checks and reports all gates passed.
- **Legacy diagnostic multiplicity is preserved:** normalized entries include an occurrence suffix after sorting; the production allowlist has no duplicate lines, and the fixture verifies two syntax diagnostics in one file.
- **Unparseable source fails closed:** the negative proof creates a temporary malformed TSX fixture and confirms the checker exits 2 with the fatal parser diagnostic instead of dropping it from the baseline.

## Commands run

```text
node scripts/check-flat2-style.mjs
node scripts/test-flat2-style-negative.mjs
pnpm -C frontend lint
pnpm -C frontend typecheck
node scripts/check-frontend-boundaries.mjs
node scripts/test-frontend-boundaries-negative.mjs
bash scripts/ci-grep-gates.sh
pnpm install --frozen-lockfile --lockfile-only
pnpm -C frontend install --frozen-lockfile --lockfile-only
openspec validate flat2-style-gate --strict
git diff --check
```

All listed implementation checks pass against the final snapshot. Full Vitest, Playwright, and production builds are intentionally deferred to the Wave 1 boundary under the phase tier discipline.

## Adversarial review

The first isolated packet was incomplete: Git omitted untracked C-03 files while exposing cumulative tracked C-00/C-02 package hunks. The judge correctly blocked that packet with 2 critical, 3 warning, and 1 suggestion findings, but both critical claims belonged to completed C-02 and disappeared once the review scope contained the actual C-03 inventory.

The corrected packet contained 25,012 bytes of C-03-owned source/config/spec snapshots, the exact Unicorn package delta, allowlist integrity evidence, and executable gate output. Judge `k3` reviewed it against producer `openai/gpt-5` through the REST gateway with `cross_model_check: verified-distinct`. It returned `PASS` with 0 critical, 4 warning, and 1 suggestion findings; the strict anti-theater gate passed at score 0.0.

Disposition:

- Narrowed the formal baseline scenarios to product source under `frontend/src`, matching the published census and default checker scope. Normal frontend lint remains broader.
- Fatal ESLint parser diagnostics now fail closed with exit 2 and have a temporary-fixture proof.
- Added TemplateElement and JSX expression-container selectors plus negative fixtures.
- Rejected duplicate directory enforcement: `unicorn/filename-case` 73 documents that it checks directory names by default (`checkDirectories: true`); the configured rule does not disable it.
- Retained the nonblocking malformed-internal-flag suggestion. A missing flag value already terminates nonzero and cannot produce a false pass; those arguments are only issued by repository-owned tests.

Post-remediation style, negative, lint, typecheck, boundary, root CI, frozen-lockfile, strict OpenSpec, and whitespace gates all pass.

Review receipt SHA-256 values:

- Incomplete round one: `a39272ee2c03cc9e4a775312abd1a34edbc0a8df1cc021d19ad03aab4a0ed12b`
- Corrected round: `e87a548e95c1f75edc31f8d20804b753b5a67e4530ec7125f9caf232ae995cb4`

## Issues by priority

### CRITICAL

None found by deterministic verification.

### WARNING

None unresolved.

### SUGGESTION

- Internal `--allowlist` / `--fixture-dir` calls could report a clearer usage error when their value is omitted; current malformed invocations fail nonzero.

## Final assessment

The implementation satisfies its owned gate-only contract, deterministic checks, and isolated review gate. It is ready for canonical C-03 completion and archive.
