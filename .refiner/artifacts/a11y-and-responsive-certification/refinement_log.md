# C-15 refinement log

## 2026-08-08 — iteration 1

- **Specify:** Bound the final change to WCAG automation, representative keyboard and
  semantic checks, the 320/768/1024/1440 light/dark matrix, high contrast, honest KnowMe
  acceptance classification, and protected-scope integrity.
- **Plan:** Capture failing evidence before editing, remediate only reproduced defects,
  reconcile browser profiles, run the consolidated frontend gates once, and submit a
  self-contained packet to a fresh critic.
- **Execute:** Added a serial axe/interaction/matrix configuration; fixed reproduced token,
  focus, heading, landmark, and focus-return defects; removed accessibility suppressions;
  separated deterministic, performance, and real-server Playwright profiles; and made the
  Vitest unit project selectable for valid coverage measurement.
- **Reflect — delta first:** The first matrix was 1/14, the Flat 2.0 exact-string baseline
  became stale after focus remediation, two replay assertions described old presentation,
  and combined Storybook coverage distorted a timing gate. Each defect now has a minimal
  correction and a repeatable receipt.
- **Persist:** 14/14 accessibility, 331/331 frontend tests, 42 default browser tests, 2 real
  browser tests, 217,622/250,000 bundle bytes, and all three performance budgets pass.
  Coverage is 33.54% lines versus the 19.45% baseline and known 60% target. Fresh isolated
  review remains the convergence condition.

## 2026-08-08 — iteration 2

- **Reflect — delta first:** Fresh artifact-only review blocked on seven auditability and
  coverage gaps: ephemeral receipts, dark-only secondary surfaces, no fail-closed
  suppression scan, shallow keyboard/focus coverage, an incomplete target selector,
  compound acceptance labels, and a non-reproducible protected digest.
- **Execute:** Added permanent JSON reporters and manifests, certified chat/settings and
  representative focus controls in both themes, added a suppression scanner with a
  rejecting fixture, expanded keyboard traversal and focus return, included anchors in
  target sizing, split acceptance classifications, and retained exact per-path protected
  status/diff hashes.
- **Observed remediation:** The stronger checks reproduced 50%-opacity Base UI focus rings
  and route-remount focus loss. Production Base UI focus tokens are now solid, and the
  persistent shell restores focus after route changes.
- **Persist:** 16/16 accessibility tests, 331/331 full frontend tests, 42 default browser
  passes with 3 explicit skips, 2/2 real-server passes, 217,630/250,000 bundle bytes,
  and all three retained final performance budgets pass. Coverage is 33.68% lines; the
  inherited 60% threshold remains honestly red. Re-review remains the convergence gate.

- **Converge:** Final artifact-only re-review passed with zero critical/high findings. The
  remaining performance-receipt limitation is disclosed, and the reviewer-requested
  top-level receipt-manifest timestamp was refreshed after the final evidence run.
