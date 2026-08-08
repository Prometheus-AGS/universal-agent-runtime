# Run Trace and Inspector Refinement Log

## 2026-08-08 — Iteration 1

- **Specify:** Bound validation to the C-11 reactive run trace, typed service/store boundary, one responsive tree, accessible inspector, 500-event budget, inert raw-data handling, and per-change gates.
- **Plan:** Audit state and trust-boundary semantics first, then compact-target sizing, keyboard/ARIA behavior, wide/narrow composition, semantic theme usage, focused tests, Chromium performance, and repository gates.
- **Execute:** The audit found inspector tabs, copy buttons, resume, and replay refresh below the 44px compact-target requirement. It also found that idle/loading replay states displayed “Validated inert metadata only.” The controls now use 44px minimum targets with the 3px ember focus treatment; replay, checkpoint, and agent loading/error states are explicit and truthful.
- **Reflect — delta first:** Automated interaction coverage had proved the action paths but did not expose the target-size and status-language gaps. A new loading-state regression now prevents a non-success replay state from claiming validation. The responsive source retains one semantic tree, and all trace surfaces use shared semantic tokens for both themes rather than theme-specific hard-coded colors.
- **Persist:** Six focused files pass with 32 tests, the supported Chromium 500-event story passes, and typecheck, lint, architecture, Flat 2.0, strict OpenSpec, and diff checks pass. Full frontend tests and production build remain deferred to the Wave 4 boundary after C-12.

## 2026-08-08 — Iteration 2

- **Specify:** Reopened refinement for the first isolated review's critical roving-focus defect and its concrete checkpoint, phase-selection, resumed-run query, and repeat-announcement findings.
- **Plan:** Make focus ownership explicit for both virtual and non-virtual rows, expose selected checkpoint data as escaped inert content, reopen collapsed ancestors during phase selection, and persist every selected or resumed run in the existing `?run=` route contract.
- **Execute:** Keyboard selection now records a pending focus target and focuses the newly committed row after render; virtual rows participate in the same element map. The inspector renders selected checkpoint metadata/state/messages as deterministic text, phase selection reopens the root, run handoffs update search params, and copy announcements carry a monotonically changing count.
- **Reflect — delta first:** The original keyboard test asserted callbacks without rerendering a controlled projection, so it missed the broken roving-focus behavior. The replacement regression performs consecutive ArrowDown operations and proves DOM focus advances on each selection. Backend source proves replay paths are deliberately coarse-grained; existing source proves the alleged import/dead-code warnings are false, while singleton-consumer and additive-wire concerns remain hypothetical rather than observed C-11 defects.
- **Persist:** Six focused files pass with 35 tests, the supported Chromium story passes, and typecheck, lint, architecture, Flat 2.0, strict OpenSpec, and scoped diff checks pass. Submit a corrected complete-source packet for isolated review.

## 2026-08-08 — Iteration 3

- **Specify:** Reopened for round two's real local-subscription failure and live-scroll findings while separating them from a cumulative-diff packet attribution error.
- **Plan:** Give the local PGlite subscription its own state/error channel, continue independent remote loads after setup failure, prevent same-selection appends from recentring, preserve checkpoint choice, and scope the next packet to C-11 source plus exact dependency evidence.
- **Execute:** Snapshot setup now reports loading/success/error, catches initialization failure, renders an alert, and proceeds with checkpoint/replay/agent requests. Selection scrolling is keyed to a changed selected id or pending keyboard focus, and checkpoint refresh retains a still-valid selection. The frozen frontend install proves the exact 3.14.9 manifest/lock pair.
- **Reflect — delta first:** The action-state model covered four remote operations but omitted the local subscription that makes the trace usable. The cumulative `HEAD` package hunk also caused the judge to assign earlier accepted Wave 1-3 dependency changes to C-11; the next packet carries only complete C-11 source and exact current manifest/lock evidence.
- **Persist:** The final isolated review passes at 0 critical / 3 warnings / 1 suggestion with verified-distinct `k3` and anti-sycophancy score 0.0. Its virtual-focus retry and pending-run warnings were adopted with focused coverage; six focused files now pass with 39 tests, the supported Chromium story passes, and frozen install, typecheck, lint, boundaries, Flat 2.0, strict OpenSpec, and scoped diff checks pass.
