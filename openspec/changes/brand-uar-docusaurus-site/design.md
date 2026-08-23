## Context

See `proposal.md` for motivation and `specs/dev-portal-2026/spec.md` for the
observable contract. The source site is the Docusaurus 3.10.2 classic preset
with stock green tokens, tutorial homepage/components, sample artwork, and no
search. The shipped React application already defines the brand authority in
`frontend/src/shared/theme/tokens.css`, `frontend/src/index.css`, and
`frontend/public/brand/`.

Memory recall ran during assessment but the memory endpoint was unreachable.
The local UI/UX Pro Max, Impeccable, and `ux-designer` commands are unavailable
in this harness. The implementation therefore uses the available
`build-branded-docusaurus`, `frontend-design`, Vercel React/performance,
composition, and current Web Interface Guidelines, with final browser/a11y
certification deferred to the phase gate.

## Goals / Non-Goals

**Goals:**

- Make every public site surface recognizably UAR without changing the React app.
- Preserve the app's dark-first four-step ladder and accessible light equivalent.
- Provide a concise, product-specific homepage and deterministic local search.
- Keep the implementation mostly CSS and static composition; avoid fragile theme
  swizzles and unnecessary client state.

**Non-Goals:**

- This change does not write the 20 missing guide routes or rewrite retained docs.
- It does not introduce hosted search, analytics, external AI search, or telemetry.
- It does not add the branded-Docusaurus skill's optional container because the
  approved phase plan explicitly cuts that delivery target.
- It does not run the final production build, screenshots, or accessibility gate
  before all documentation content is complete.

## UI/UX Guidance Distillation

The site will use an **industrial editorial runtime instrument** direction: terse
mono eyebrows, Space Grotesk display type, Geist body copy, JetBrains Mono code,
large calm fields, and a controlled ember signal with cyan reserved for protocol
and live-flow concepts. The visual hierarchy comes from the exact app surface
ladder (`#0B0F14` → `#111620` → `#161D29` → `#1C2535`, with its light
counterpart), spacing, typography, and filled active states—never gradients,
decorative shadows, borders, or separator rules. Static React composition avoids
new client state and broad imports. Semantic links/buttons, visible
`:focus-visible`, heading order, explicit image dimensions, responsive reflow,
touch targets, `text-wrap`, `scroll-margin-top`, and reduced-motion overrides come
from the current Web Interface Guidelines. The stock search theme's shadows and
borders are explicitly neutralized to preserve Flat 2.0.

## Decisions

### 1. Map the app tokens directly into stable Infima variables

`custom.css` defines UAR semantic tokens first, then maps Docusaurus/Infima
variables to them for light and dark modes. Stable public theme classes are
styled directly. No component is swizzled unless the homepage cannot express the
required structure through ordinary React/CSS modules.

An independent docs palette was rejected because it would make the portal a
second product identity. A decorative maximalist treatment was rejected because
it conflicts with the app's explicit Flat 2.0 contract.

### 2. Use reviewed static copies of the shipped SVG identity

The UAR favicon, mark, and light/dark wordmarks are copied from
`frontend/public/brand/` into `website/static/img/brand/` with their canonical
source recorded in this design. SVGs provide crisp, dimensioned assets without
adding runtime image code. Stock crocodile, mountain, React, and tutorial assets
are removed.

### 3. Keep homepage composition static and explicit

The homepage uses semantic sections for the runtime proposition, trust boundary,
surface ladder, protocol matrix, profile limits, and 4 reader paths. Data arrays
remain module-level constants; no effects, runtime fetches, or client stores are
introduced. This follows the available React guidance to hoist static content and
avoid unnecessary hydration work.

### 4. Pin local search and fonts as site-only dependencies

Pin `@easyops-cn/docusaurus-search-local` `0.55.3` as a theme with hashed English
docs/pages indexes, blog indexing disabled, and Ask AI omitted. Pin Fontsource
Geist Variable, Space Grotesk, and JetBrains Mono `5.3.0` and import them locally;
the portal does not depend on remote font or search services. The versions and
Docusaurus 3/React 19 peer compatibility were verified from the current npm
registry and upstream plugin documentation.

### 5. Keep motion sparse and removable

Only initial hero/content opacity and transform transitions are permitted, using
the app's timing curves. `prefers-reduced-motion` reduces all nonessential motion
to effectively immediate state changes. No continuous decorative animation is
introduced.

## Risks / Trade-offs

- **[Risk] Docusaurus internal class changes can break cosmetic overrides** → Use
  stable documented theme/Infima classes, keep selectors shallow, and validate the
  pinned production build at the final gate.
- **[Risk] Local search brings default shadows/borders** → Override its published
  CSS variables and inspect light/dark keyboard behavior during final certification.
- **[Risk] Static brand copies can drift from the app** → Record their canonical
  source and compare hashes in the branding controls.
- **[Risk] Missing guide routes limit homepage destinations now** → Link only to
  existing foundational pages in this change; later content changes update
  navigation as their routes land.
- **[Trade-off] The final visual evidence is deferred** → This change can prove
  source contracts and asset identity only; it cannot claim rendered quality until
  the complete portal is built and inspected.

## Migration Plan

1. Pin the search/font dependencies and update the npm lockfile.
2. Replace stock assets, homepage/components, CSS tokens, and navigation.
3. Add source-level controls for forbidden stock identity, asset parity, theme
   tokens, local search, semantic structure, and reduced-motion coverage.
4. Run strict OpenSpec and bounded source review; do not run the phase-level build.
5. Roll back as one change if the final complete-content build later exposes a
   structural incompatibility.
