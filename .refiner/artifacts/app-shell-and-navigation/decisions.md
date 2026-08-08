# App Shell and Navigation Refinement Decisions

## 2026-08-07 — Iteration 1

- **Delta:** High-contrast brand selection and compact header target sizing required correction after the first rendered audit.
- **Correction:** High-contrast mode now selects the delivered dark asset variant, and compact command/theme controls are 44px while desktop sizes remain unchanged.
- **Decision:** Terminate the refinement loop after deterministic gates and rendered measurements satisfy all four blocking constraints. Full-suite/build certification remains at the Wave 4 boundary rather than being run early.

## 2026-08-07 — Iteration 2

- **Delta:** The first isolated review identified missing direct keyboard evidence, dialog-stacking protection, system/current semantics, and conservative browser/PWA metadata.
- **Correction:** Remediated every actionable finding and added focused coverage. Retained Base UI Item `onClick` as the documented pointer-and-Enter selection callback; did not add the judge's incorrect input `onValueChange` suggestion.
- **Decision:** Re-run deterministic validation and isolated review before final convergence.

## 2026-08-07 — Iteration 3

- **Delta:** A broad `/admin` prefix made Runtime settings active alongside specific work/configure links; the finding correctly blocked convergence.
- **Correction:** Bare route roots now use explicit exact matching, and focused coverage proves only Knowledge owns `/admin/knowledge` in the rail.
- **Decision:** Keep the PWA icon purpose at `any` until a real safe-zone-aware maskable asset is delivered; do not manufacture capability metadata to silence a review warning. Re-run the independent gate for final convergence.

## 2026-08-08 — Iteration 4

- **Delta:** The non-blocking review identified over-broad dialog presence detection and no same-shortcut close path, while reversing its earlier recommendation for editable controls.
- **Correction:** Only active modal dialogs suppress the global opener; an open shell palette toggles closed; editable controls retain native/browser behavior exactly as the spec states.
- **Decision:** Treat the existing shared reduced-motion rule and Wave 4 static rebuild as established upstream/downstream contracts, retain truthful non-maskable manifest metadata, and seek one final independent receipt.

## 2026-08-08 — Iteration 5

- **Delta:** The unconditional favicon fallback followed the media-specific links, so modern browsers could prefer it in light mode; a compact dialog could also retain modal state after becoming visually hidden at the desktop breakpoint.
- **Correction:** Fallback metadata now comes first, and the sheet closes when `(min-width: 901px)` becomes true. The same pass embeds the supplied brand vector geometry so its typography participates in the host font context and closes the remaining concrete accessibility/interaction findings.
- **Decision:** Preserve Base UI's documented `Item.onClick` selection contract instead of adding an incorrect input-value handler; make that contract explicit beside the handler and in focused Enter-key evidence. Keep all routes available through the compact header palette, and converge only after the corrected artifact receives a final isolated review receipt with no critical finding.
