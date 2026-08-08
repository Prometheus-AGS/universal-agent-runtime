# C-15 accessibility certification report

Run date: 2026-08-08

## Verdict

The C-15-owned React accessibility contract passes its automated and keyboard checks.
The certified `test:a11y` command first ran the structural Storybook suppression gate and
its five-form rejecting fixture, then completed **16/16** serial Chromium tests with no
in-scope axe violations. The fail-closed Vitest/Storybook suite separately completed
**69 files / 331 tests**.

This is an automated WCAG 2.2 AA conformance check, not a claim that axe substitutes for
a manual screen-reader usability study. Manual assistive-technology evaluation remains a
release-level check and is classified explicitly in `acceptance-checklist.md`.

## Certified surfaces and assertions

- `/admin/runtime` at 320, 768, 1024, and 1440 CSS pixels in light and dark themes.
- `/admin/runtime` at 1440 CSS pixels in the UAR high-contrast theme.
- `/threads` and `/admin/settings` in light and dark themes for single-main-landmark and integrated axe checks.
- Desktop skip navigation and command-palette Enter/Escape/focus-return behavior.
- Compact Configure dialog keyboard containment, Escape close, and trigger focus return.
- Computed 3px light- and dark-theme focus presentation on native, shell, Base UI,
  dialog, and feature-owned controls, with at least 3:1 adjacent contrast.
- Standalone actionable buttons, links, inputs, selects, textareas, summaries, and
  role-based controls of at least 24 by 24 CSS pixels on the matrix route.
- Text-bearing readiness status and decorative status-dot semantics.
- `prefers-reduced-motion: reduce` collapsing non-essential transition duration.

## Observed failures and remediations

The initial 14-test baseline passed 1 test and failed 13. The reproduced defects were:

- light-theme ember contrast between 3.32:1 and 3.70:1;
- dark faint-text contrast between 3.18:1 and 3.97:1;
- missing page-level heading semantics on the shell breadcrumb;
- nested main landmarks on chat and settings routes;
- command-palette focus not returning to its trigger;
- Storybook accessibility suppressions;
- shared or feature focus indicators resolving to 1px or 2px.
- Base UI focus rings resolving through a 50%-opacity token; and
- route navigation remounting the command trigger after dialog-local focus restoration.

The minimal remediation darkened the light ember token to `#b93c1c`, raised dark faint
text to `#87909d`, made the focus tokens solid, established a 3px shared focus rule,
upgraded reproduced feature overrides and Base UI rings, exposed the current breadcrumb as
an `h1`, changed the nested chat/settings mains to named sections, moved command-palette
focus restoration to the route-aware shell, removed both Storybook suppression sites, and
added a fail-closed source scanner plus a negative fixture that proves suppressions are
rejected.

## Repeatable commands

```bash
pnpm -C frontend test:a11y
pnpm --filter uar-frontend test
```

The Playwright JSON reporter writes the complete durable run to
`openspec/changes/a11y-and-responsive-certification/receipts/accessibility-playwright.json`.
Exact commands, timestamps, exit codes, input hashes, and receipt hashes are indexed in
`receipts/manifest.json`; the scanner stdout/stderr and exit codes are retained in
`receipts/storybook-suppression-gate.json`.

## Limits

- Browser certification uses the supported Chromium profile; it is not a multi-engine
  browser compatibility claim.
- The target-size assertion covers standalone controls on the representative runtime
  matrix route. Inline links and WCAG spacing exceptions are not falsely reported as
  globally measured.
- Real backend behavior is covered by the dedicated provider-routing and knowledge-RAG
  browser tests, both of which passed, but those results do not replace accessibility
  inspection of every possible data state.
