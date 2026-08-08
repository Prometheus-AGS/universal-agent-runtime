# App Shell and Navigation Refinement Log

## 2026-08-07 — Iteration 1

- **Specify:** Bound the artifact to C-10's typed navigation, one-tree responsive shell, Base UI palette/sheet, delivered brand, and accessibility requirements. C-14-owned feature migrations and legacy command consumers remain outside scope.
- **Plan:** Validate source architecture first, then focused interactions, exact breakpoint metrics, visual hierarchy, brand inventory, and repository gates.
- **Execute:** Exercised 20 focused tests and rendered the live shell at 1440, 901, 900, 390, and 320 CSS pixels. Removed two obsolete Flat 2.0 filename allowlist entries after retiring the PascalCase shell/logo files.
- **Reflect — delta first:** The initial render exposed two polish gaps: high-contrast mode selected the light fixed-color brand asset, and compact command/theme controls were below the 44px touch target. Both were corrected before convergence. No further blocking delta remained.
- **Persist:** Stored the deterministic receipt and convergence decision under the named `app-shell-and-navigation` artifact.

## 2026-08-07 — Iteration 2

- **Specify:** Reopened the loop for the first isolated review's five warnings and three suggestions, separating actual product gaps from one packet-scope false positive.
- **Plan:** Add direct keyboard/current-state/dialog-stacking coverage, correct semantic labels and metadata, then rerun the deterministic and independent gates.
- **Execute:** Added keyboard Enter navigation coverage, suppressed the global palette while another dialog is open, mapped system breadcrumbs explicitly, exposed Configure's current state, added an unconditional favicon fallback, stopped claiming a nonexistent maskable icon variant, and aligned the design artifact's `shellSheet` name.
- **Reflect — delta first:** The first keyboard test selected the already-current Chat command because the palette correctly auto-highlights its first result; narrowing the query to Knowledge proved Base UI's documented keyboard-to-`onClick` behavior. The registry warning was disproved by the existing `.refiner/artifacts/markdown-lazy-blocks` directory, which the review packet's pruned tree did not show.
- **Persist:** Updated the verification receipt to 22 focused tests and queued a corrected isolated review packet.

## 2026-08-07 — Iteration 3

- **Specify:** Reopened after the corrected review found one critical route-state defect and four actionable shell warnings.
- **Plan:** Separate exact root matching from prefix matching, assert a single current rail destination, preserve browser focus for the reserved shortcut, expose collapsed readiness semantics, and settle favicon ordering.
- **Execute:** Added `exactMatchPaths`, moved bare `/admin` and `/about` to exact matching, added one-current-link coverage, prevented the browser default for the reserved Control/Meta+K chord without opening over editable/dialog contexts, added `role="status"`, moved the unconditional favicon fallback after media-specific choices, and hardened metadata filtering.
- **Reflect — delta first:** The critical arose because `findDestinationForPath` used inventory ordering but individual rail links did not. The new matcher makes correctness local to each destination instead of relying on consumer order. The manifest maskable warning remains intentionally unimplemented because no delivered safe-zone-aware maskable asset exists; claiming one would be false metadata.
- **Persist:** Updated the verification receipt to 23 focused tests and queued the final isolated review.

## 2026-08-08 — Iteration 4

- **Specify:** Process the non-blocking route-corrected review without undoing the explicit outside-editable shortcut contract or inventing maskable artwork.
- **Plan:** Toggle the shell palette with its own shortcut, suppress only open modal dialogs, add a second install-icon size, and retain global reduced-motion ownership.
- **Execute:** Narrowed dialog detection to active `aria-modal` surfaces, restored browser/editor defaults inside editable controls, added same-shortcut close behavior, added a delivered 256px PWA icon alongside 512px, and documented the shared 1ms reduced-motion override.
- **Reflect — delta first:** Two consecutive shortcut warnings pulled in opposite directions. The binding rule settles the conflict: editable controls are not intercepted; the shell chord is prevented only when it opens or closes the shell palette. No delivered safe-zone-aware maskable asset exists, so the manifest remains truthfully `purpose: any`.
- **Persist:** Final deterministic evidence is 25 focused tests plus all implementation-tier gates; queue the final independent receipt.

## 2026-08-08 — Iteration 5

- **Specify:** Reopened for the final isolated review's favicon-precedence blocker and its concrete typography, editable-target, responsive-dialog, announcement, download, and shortcut-label findings.
- **Plan:** Put the unconditional favicon fallback before media-specific choices, preserve the supplied wordmark geometry while rendering its type in the host document, close compact modal state on the desktop transition, and cover the two newly observed interaction edges.
- **Execute:** Corrected favicon order, inlined the delivered wordmark/app-icon vector geometry so page fonts and theme classes apply without duplicate image downloads, recognized all valid editable `contenteditable` states, closed the Configure sheet at 901px, made the About wordmark decorative, rendered the platform-correct shortcut hint, exposed Runs/About through the persistent compact command trigger, and generated a UAR 180px touch icon from the delivered app icon.
- **Reflect — delta first:** The fallback link intended for old browsers was last, which made it the winning icon in modern browsers. CSS hiding also concealed a still-modal dialog after a viewport transition. Ordering and state ownership are now explicit, with focused regressions for both the favicon contract and compact-to-desktop teardown.
- **Persist:** Deterministic evidence is now 28 focused tests plus all implementation-tier gates; queue a final independent receipt against the corrected artifact.
