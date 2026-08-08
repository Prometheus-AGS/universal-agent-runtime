## 1. Workflow and baseline

- [x] 1.1 Capture the C-15 protected-path hash and current Storybook, focus, contrast, landmark, live-region, reduced-motion, status, target-size, and responsive baselines.
- [x] 1.2 Strictly validate the proposal, design, capability delta, tasks, and UI-routing summary before implementation.
- [x] 1.3 Declare the already-resolved `axe-core@4.12.1` package as an explicit frontend development dependency without introducing a second accessibility runtime.

## 2. Automated accessibility certification

- [x] 2.1 Add a serial Playwright certification configuration and reusable axe injection/reporting helpers.
- [x] 2.2 Certify representative chat, runtime, and settings surfaces for axe, landmarks, accessible names, live regions, and non-colour status cues.
- [x] 2.3 Add desktop and compact keyboard walks for skip navigation, Tab/Shift-Tab, Enter, Escape, dialog focus containment, and trigger focus return.
- [x] 2.4 Add computed checks for the 3px ember focus indicator, standalone 24px targets, and reduced-motion behavior.
- [x] 2.5 Remove every Storybook `a11y.test = off` suppression and keep the component accessibility suite fail closed.

## 3. Observed remediation

- [x] 3.1 Correct only contrast failures reproduced in resolved light, dark, or high-contrast token pairs.
- [x] 3.2 Make the shared 3px ember focus rule authoritative and remove or upgrade reproduced 1–2px feature overrides.
- [x] 3.3 Correct any reproduced accessible-name, landmark, live-region, status-cue, target-size, media-alternative, reduced-motion, or responsive-overflow defects.
- [x] 3.4 Run the cheap TypeScript, lint, and frontend-boundary gates while implementation is active.

## 4. Responsive and acceptance certification

- [x] 4.1 Run the 320/768/1024/1440 light/dark matrix and retain route-, theme-, and viewport-specific axe and layout receipts.
- [x] 4.2 Run a representative high-contrast accessibility pass for the UAR-specific third theme.
- [x] 4.3 Complete the KnowMe §12 checklist with verified, failed, not-applicable, backend-bound, and separately owned classifications.
- [x] 4.4 Reconcile the broad E2E probe retained from C-14d so default-profile, dedicated-performance, and real-backend tests are invoked by the correct configurations.

## 5. Phase completion gates

- [x] 5.1 Run the full frontend Vitest and fail-closed Storybook suites.
- [x] 5.2 Run the complete default-profile Playwright suite plus dedicated accessibility and performance configurations, documenting any environment-bound exclusions without claiming a pass.
- [x] 5.3 Run typecheck, lint, boundary/settings/CI grep gates, production build, bundle budget, and coverage; prove coverage is no worse than the 19.45% baseline.
- [x] 5.4 Verify the protected-path hash is unchanged and inventory the C-15-owned files.

## 6. Evidence and closeout

- [x] 6.1 Write `verification.md`, the accessibility report, matrix receipt, acceptance checklist, and protected-path closeout.
- [x] 6.2 Package the artifact-refiner evidence and complete a fresh artifact-only adversarial review with no unresolved critical finding.
- [x] 6.3 Transition C-15 to complete through canonical KBD state and confirm revision/projection refresh.
- [x] 6.4 Archive `a11y-and-responsive-certification`, validate the resulting main capability, append the Prometheus session waypoint, and run the phase reflection.
