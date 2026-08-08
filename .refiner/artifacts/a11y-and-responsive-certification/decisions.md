# C-15 refinement decisions

## 2026-08-08 — certification pass

- **Delta:** The initial 14-test probe passed only one case. It reproduced light/dark
  contrast defects, a 2px shared focus rule plus weaker feature overrides, missing heading
  semantics, nested main landmarks, focus-return failure, and disabled Storybook a11y
  checks.
- **Correction:** Adjust only the failing tokens and semantics, establish a solid 3px
  focus contract, restore command-palette trigger focus, remove the suppressions, and
  retain deterministic transport mocks for the matrix.
- **Decision:** Treat the eight-cell matrix, focused keyboard proofs, high-contrast axe,
  full Storybook suite, broad browser profiles, and protected digest as the blocking C-15
  evidence. Treat manual screen-reader/text-scaling review and the absent Flutter surface
  honestly in the acceptance classification.

## Coverage runner correction

- **Delta:** Running coverage over both Vitest projects instruments a timing assertion and
  produced 282ms against a 250ms non-instrumented browser budget, despite the dedicated
  performance run passing at 130.4ms.
- **Correction:** Name the non-browser project `unit` and scope `test:coverage` to it.
  Storybook accessibility remains in the normal full-suite gate and performance remains in
  its dedicated uninstrumented gate.
- **Decision:** Record 33.54% line coverage as an improvement over 19.45%, while preserving
  and reporting the known red 60% threshold.
