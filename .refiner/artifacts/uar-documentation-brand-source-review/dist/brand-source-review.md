# UAR documentation brand source review

## Decision

The branding source is ready for the documentation-content changes. This is not
rendered-site, accessibility, deployment, or publication certification.

## Constraint results

| Constraint | Result | Evidence boundary |
|---|---|---|
| Shipped brand identity | Satisfied | Five canonical asset pairs are byte-identical; required UAR tokens are present; stock assets and tutorial markers are absent. |
| Local search and fonts | Satisfied | npm resolves local search 0.55.3 and all three font packages 5.3.0; controls reject hosted search and remote fonts. |
| Semantic static homepage | Satisfied | Source controls require ordered headings, labelled sections, explicit image dimensions, module-level data, and existing internal routes; TypeScript passes. |
| Flat accessible source contract | Satisfied | Controls reject stock green, gradients, decorative shadows/borders, invisible focus, and missing reduced motion. |
| Bounded branding change | Satisfied | Strict OpenSpec passes; only `website/package-lock.json` is a changed lockfile; prohibited product-source diff is empty. |

## Current Web Interface Guidelines source audit

- Interactive actions use links with meaningful text; no div click handlers or
  invented keyboard behavior were introduced.
- The page has one H1 followed by section H2s and card/panel H3s.
- Image dimensions are explicit; decorative brand imagery is hidden from the
  accessibility tree.
- Focus is visible, headings have scroll margin, copy uses balanced/pretty
  wrapping, tables remain horizontally scrollable on narrow screens, and motion
  has a reduced-motion override.
- Static arrays remain module-level and the homepage adds no client state,
  effects, runtime fetches, or broad application imports.

## Evidence limits

- Production build, rendered keyboard behavior, contrast measurement,
  screenshots, browser console/network diagnostics, local-search interaction,
  and deployed route checks are deferred to `certify-and-publish-uar-docs`.
- npm reported 20 high-severity transitive advisories during installation. No
  advisory remediation or dependency migration was authorized in this change.

## Uncomfortable fact

The visual design has not yet been seen in a browser. The source is coherent and
fail-closed, but the final certification must still be allowed to reject it.
