## Why

UAR now has current portal guides, but its repository entry points still form a
second, inconsistent documentation system: 39 tracked READMEs mix current,
placeholder, generated, historical, and vendored material, while five frozen
product routes still lack documents. This change makes every README and retained
navigation surface resolve to a declared current authority before history and
final-site certification proceed.

## What Changes

- Reconcile the root README hero, tagline, existing brand asset, badges,
  diagrams, quickstart, profile/support boundaries, GitHub Pages link, and
  customer navigation against current source and the canonical portal.
- Classify and reconcile the measured estate of 39 READMEs: the root, 31
  subordinate UAR-owned files, five iterative-evolver mirrors generated from
  their declared source, and two semantically untouched vendored exclusions.
- Replace placeholder or superseded subordinate guidance with concise local
  package/directory contracts and links to current portal authority; preserve
  retained history with dated supersession banners rather than rewriting it.
- Add the five documents required by the frozen route inventory for chat, A2UI
  artifacts, compiler, A2UI testing, and About; do not edit the frozen route
  manifest or shared navigation/theme.
- Add a machine-readable README authority manifest, deterministic sync and
  validation tools, isolated negative controls, and row-form verification.
- Keep GitHub Actions deployment-only and defer the production site build,
  browser/accessibility checks, live Pages validation, and repository homepage
  mutation to `certify-and-publish-uar-docs`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `readme-presentation`: Require the branded root entry point, complete README
  ownership/disposition inventory, generated-mirror equality, and unchanged
  vendored content.
- `documentation-truth-gate`: Extend present-versus-historical and
  source-authority validation to every tracked README and retained public
  document.
- `customer-documentation`: Require repository entry points and frozen product
  routes to resolve to the current portal without unsupported availability,
  certification, or cross-profile claims.
- `dev-portal-2026`: Materialize every required frozen product route while
  preserving the existing single-portal information architecture.

## Impact

- Affects `README.md`, the 31 subordinate UAR-owned READMEs, the five generated
  iterative-evolver README mirrors and their local sync contract, selected
  retained historical documents, five Docusaurus route pages, documentation
  manifests/validators, OpenSpec artifacts, refiner evidence, and KBD handoff.
- Does not change runtime UX, React behavior, provider compatibility, realtime
  state, APIs, dependencies, submodule pins, package publication, release
  artifacts, or deployment workflows.
- KBD workflow state must transition `reconcile-uar-readme-estate` through
  Execute and point next to `publish-uar-architecture-decision-history`.
