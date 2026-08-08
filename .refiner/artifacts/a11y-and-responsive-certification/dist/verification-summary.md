# C-15 artifact-only review packet

## Review objective

Determine whether `a11y-and-responsive-certification` satisfies its four blocking
constraints without overstating WCAG, coverage, acceptance, or scope evidence. Report
findings by severity with exact file/line evidence. A critical finding blocks completion.

## Authoritative artifacts

- `openspec/changes/a11y-and-responsive-certification/proposal.md`
- `openspec/changes/a11y-and-responsive-certification/design.md`
- `openspec/changes/a11y-and-responsive-certification/specs/frontend-accessibility-certification/spec.md`
- `openspec/changes/a11y-and-responsive-certification/accessibility-report.md`
- `openspec/changes/a11y-and-responsive-certification/responsive-matrix.md`
- `openspec/changes/a11y-and-responsive-certification/acceptance-checklist.md`
- `openspec/changes/a11y-and-responsive-certification/verification.md`
- `openspec/changes/a11y-and-responsive-certification/isolated-review.md`
- `openspec/changes/a11y-and-responsive-certification/protected-path-baseline.txt`
- `openspec/changes/a11y-and-responsive-certification/protected-path-closeout.md`
- `openspec/changes/a11y-and-responsive-certification/protected-path-manifest.json`
- `openspec/changes/a11y-and-responsive-certification/receipts/manifest.json`
- `openspec/changes/a11y-and-responsive-certification/receipts/performance-attempts.json`
- `openspec/changes/a11y-and-responsive-certification/receipts/storybook-suppression-gate.json`
- `openspec/changes/a11y-and-responsive-certification/files.txt`
- `frontend/e2e/a11y-responsive-certification.spec.ts`
- `frontend/playwright.accessibility.config.ts`
- `frontend/playwright.performance.config.ts`
- `frontend/playwright.real-server.config.ts`
- `frontend/playwright.config.ts`
- `frontend/vitest.config.ts`
- `frontend/package.json`
- `.refiner/artifacts/a11y-and-responsive-certification/constraints.json`

## Deterministic evidence

- Accessibility profile: **16/16 passed**, covering eight viewport/theme cells, both-theme
  chat/settings landmarks, high contrast, desktop/compact keyboard operation, computed
  focus across native/shell/Base UI/dialog/feature controls, status, and reduced motion.
- The certified accessibility command runs a TypeScript-AST Storybook suppression scan
  first; its negative gate rejects unquoted, quoted, computed, and assignment forms.
- Full frontend: **69 files / 331 tests passed**, including fail-closed Storybook.
- Default browser: **42 passed / 3 explicit skips / 0 failed**.
- Dedicated real server: **2/2 passed** for knowledge RAG and provider routing.
- Bundle: **217,630 / 250,000 gzip bytes**.
- Retained final performance: **942.2/1,000ms**, **14.1/100ms**, and **137/250ms**; one
  immediately preceding 1008.8ms cold-start variance sample is bound to the same performance
  input digest and disclosed in the attempt-history receipt.
- Coverage: **33.68% lines**, above the **19.45%** baseline but below the intentionally
  retained **60%** target; the coverage command exits red only on those thresholds.
- Protected digest: entry and closeout both
  `07e74ad94dc137e9574e411bc99d6f0fcd631879c5a0e52a1b87ca999cf43dc4`.
- Typecheck, lint, boundaries, settings decomposition, CI grep gates, production build,
  and strict OpenSpec validation passed.
- The receipt manifest binds exact commands, timestamps, exit codes, profile-input hashes,
  and JSON receipt hashes; the protected-path manifest retains equivalent per-path proof.

## Known non-claims

- Axe and semantic assertions are not presented as a manual screen-reader usability study.
- The UAR migration has no Flutter application surface; Flutter rows are not applicable.
- The operator-approved Base UI divergence is not presented as Shadcn compliance.
- The Flat 2.0 gate has 376 tracked legacy findings and 0 new findings; the packet does not
  claim that the entire estate has zero visible borders.
- Coverage remains below 60%; only non-regression against the 19.45% phase baseline is
  claimed.

## Required critic output

Return `PASS` only if no critical or high-severity correctness/auditability defect remains.
Otherwise list each finding with severity, evidence, impact, and the smallest correction.

## Final critic result

Fresh artifact-only re-review returned **PASS with zero critical/high findings** after the
receipt, structural suppression, keyboard/focus, target-selector, acceptance, and
protected-manifest corrections. Two transparent warnings remain: the original failed
performance reporter file was overwritten before preservation, and a stale receipt
manifest timestamp was corrected after review.
