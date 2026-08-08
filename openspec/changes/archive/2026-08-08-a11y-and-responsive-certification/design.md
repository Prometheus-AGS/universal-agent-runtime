## Context

C-15 is the final gate in a 21-change React migration. The shell already supplies a skip
link, named desktop and compact navigation landmarks, a main landmark, text-bearing
readiness state, polite live regions, and a reduced-motion media query. The remaining
claims are not certified: the global focus rule renders a 2px ring, feature controls can
override it with 1–2px indicators, Storybook disables accessibility tests for known token
contrast failures, and no repeatable 320/768/1024/1440 light/dark matrix exists.

The authoritative inputs are the C-15 plan row, KnowMe standard §§11–12,
`docs/ui/uar-frontend-migration-plan.md` §11, and the `a11y-gate` checklist. This is a
React/Tauri-web certification change. The repository has no UAR Flutter application UI;
the nested entity-management Flutter SDK is a separate package and is not a surface of
this migration.

## Goals / Non-Goals

**Goals:**

- Produce deterministic, reviewable evidence for WCAG 2.2 AA automation and the manual
  keyboard/accessibility checks that automation cannot replace.
- Certify the shared shell and representative chat, runtime, and settings surfaces across
  320, 768, 1024, and 1440 CSS pixels in light and dark themes.
- Establish one 3px ember focus contract and remediate only reproduced focus, contrast,
  naming, target-size, status, live-region, reduced-motion, or overflow failures.
- Apply the KnowMe §12 checklist and distinguish verified, not-applicable, backend-bound,
  and separately owned claims.

**Non-Goals:**

- Redesigning application surfaces or changing their data flow.
- Adding a Flutter application or certifying the nested entity-management SDK as UAR UI.
- Claiming screen-reader usability from axe alone.
- Repairing backend-dependent behavior that is not an accessibility or responsive defect.

## Decisions

### Reuse the installed axe-core version in Playwright

The workspace already resolves `axe-core@4.12.1` through the Storybook accessibility
tooling. The frontend will declare that exact package as a direct development dependency
and inject its browser bundle from a dedicated Playwright certification spec. This avoids
adding a second wrapper or a new transitive graph while making the test's runtime
dependency explicit. Storybook remains the component-level fail-closed gate.

Alternative considered: rely only on Storybook. Rejected because isolated component
stories do not exercise application landmarks, responsive navigation, or integrated route
states.

### Separate the responsive matrix from focused interaction proofs

Every required width/theme pair will assert the shell mode, main content, zero horizontal
page overflow, no shell/content overlap, and zero axe violations on deterministic routes.
Focused tests will separately cover skip navigation, Tab/Shift-Tab order, Enter and Escape,
dialog focus return, computed 3px focus presentation, target dimensions, polite live
regions, non-colour status text, and reduced-motion computed durations.

Alternative considered: repeat the full keyboard walk on every matrix cell. Rejected
because it duplicates the same DOM interaction contract without increasing responsive
coverage; breakpoint-specific navigation is still walked once in each shell mode.

### Make the global focus rule authoritative

The global `:focus-visible` rule will provide a 3px ember ring for native and Base UI
controls. Feature-level 1–2px focus overrides identified by the certification audit will
be removed or upgraded so they cannot weaken the contract. Intentional `ring-0` rules may
remain only where the same element supplies an explicit 3px ember outline.

Alternative considered: patch only currently visible controls. Rejected because the plan
requires a consistent application-wide focus contract and the existing shared rule is the
correct ownership boundary.

### Treat evidence classifications as part of certification

The final report will not convert missing infrastructure into a pass. Real-backend flows,
Flutter-only statements, and non-applicable architecture clauses will be named separately
from verified React evidence. Any failure in a requirement owned by this change blocks
canonical completion.

## Risks / Trade-offs

- [Axe cannot prove complete screen-reader usability] → Pair it with semantic inspection,
  live-region assertions, and documented keyboard walks.
- [Mocked APIs can hide backend failures] → Limit certification claims to rendered UI and
  interaction contracts; retain real-backend checks as separately classified evidence.
- [Global focus styling can conflict with component styles] → Measure computed focus on
  native, Base UI, shell, and feature controls and retain explicit 3px overrides only when
  needed.
- [Dense UIs contain legitimate small inline links] → Apply the 24px target assertion to
  standalone controls; document WCAG spacing/inline exceptions rather than producing a
  false pass.
- [Large matrix can become flaky] → Use deterministic route mocks, one Chromium worker,
  fixed viewports, and state setup before navigation.

## Migration Plan

1. Capture protected-path and accessibility baselines, then add the certification spec.
2. Run focused probes to reproduce failures before editing source.
3. Correct only reproduced token, focus, semantics, or responsive defects and rerun cheap
   type/lint/boundary checks during implementation.
4. At phase completion run Storybook, Vitest, Playwright certification, the complete E2E
   classification, build/budget, coverage, and acceptance-checklist review once.
5. Package receipts for isolated review, transition C-15 through canonical KBD, and archive
   the OpenSpec change. Rollback is the ordinary source/test/lockfile diff; no persistent
   data migration is involved.

## Open Questions

None. The plan and binding standards define the target; baseline probes determine the
minimal remediation set.
