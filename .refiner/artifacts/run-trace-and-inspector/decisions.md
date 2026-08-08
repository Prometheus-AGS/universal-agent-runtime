# Run Trace and Inspector Refinement Decisions

## 2026-08-08 — Iteration 1

- **Delta:** Compact inspector controls were smaller than 44 CSS pixels, and the replay status text could claim validation before validation completed.
- **Correction:** Raised all new inspector interaction targets to at least 44px, applied the existing 3px ember focus contract, and made checkpoint, agent, replay, and resume feedback reflect their actual scoped states.
- **Decision:** Accept deterministic convergence for the artifact-refiner gate. Reopen refinement if isolated adversarial review finds a critical defect; retain full frontend Vitest and production build for the C-12 Wave 4 boundary.

## 2026-08-08 — Iteration 2

- **Delta:** The first isolated review correctly found that roving selection changed `tabIndex` without moving DOM focus, causing repeated Arrow keys to operate from the stale row. It also identified missing selected-checkpoint inspection, collapsed-root phase selection, resumed-run query persistence, and repeat-copy announcement behavior.
- **Correction:** Added controlled focus transfer for virtual and non-virtual rows with direct regressions, rendered selected checkpoint content inertly, reopened the root for phase selection, routed all run handoffs through `?run=`, and made each copy announcement text distinct.
- **Decision:** Retain the fixed 256px scroll-owner baseline because `flex-1` still governs growth and the explicit intrinsic height prevents the previously observed 22,000px virtual-spacer expansion. Retain strict response schemas as the existing fail-closed trust-boundary contract. Treat nested replay paths as disproved by `src/uar/a2ui/realtime.rs`, which intentionally emits only whole-surface, components, and data-model paths. Re-run deterministic and isolated gates.

## 2026-08-08 — Iteration 3

- **Delta:** PGlite subscription setup could reject outside the action-state model, leaving no local error UI and preventing independent remote requests. A stable selected row also recentered after every live reprojection.
- **Correction:** Added explicit snapshot loading/success/error ownership, fail-visible panel feedback, continued remote requests, changed-selection-only scrolling, and checkpoint-selection preservation with focused regressions.
- **Decision:** Treat the round-two dependency blocker as invalid packet attribution, not a code defect: C-11's exact manifest and lock additions are verified by a frozen install, while the other dependency changes belong to already accepted C-02/C-03/C-08/C-09. The final source-scoped receipt passes. Adopt its virtual-focus retry and pending-run warnings; retain phase/filter and teardown behavior as documented nonblocking tradeoffs with no observed failure.
