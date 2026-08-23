## Why

UAR has a large documentation estate but no enforceable contract that separates current public guidance, retained history, generated material, private operational records, and vendored content. That gap has already allowed an incomplete portal, contradictory claims, unsafe raw-history publication risk, and two workflows competing to publish different artifacts to one GitHub Pages site.

## What Changes

- Define an authoritative source-classification manifest covering every tracked README and documentation path with `public`, `public-normalize`, `private-synthesis-only`, or `excluded` dispositions.
- Define the canonical public information architecture and required-route manifest for the branded Docusaurus portal.
- Require reviewed, source-linked synthesis for `.prometheus`, KBD, OpenSpec, and ADR history; raw logs, conversations, machine-local paths, secret-like material, and unreviewed wiki records never enter public output.
- Require present-tense claims to cite current authority and retained historical material to carry dated supersession metadata instead of being rewritten as current truth.
- Replace the earlier minimum-portal contract with one deployment-only GitHub Pages publisher and local ownership of prose, link, truth, privacy, accessibility, and completeness checks.
- Define how UAR-owned, generated-mirror, historical, and vendored README files are reconciled without editing generated mirrors or third-party sources independently.
- Supersede and reconcile the completed-but-unarchived `docs-hosted-rustdoc-typedoc-docusaurus-ia` change so its placeholder and GitHub Actions testing requirements cannot remain active authority.

## Capabilities

### New Capabilities

- `documentation-publication-contract`: Classification, provenance, route coverage, publication safety, and single-publisher rules for the public documentation estate.

### Modified Capabilities

- `customer-documentation`: Expand whole-product documentation behavior from a small guide set to the complete supported surface, with profile-scoped claims and an authoritative route inventory.
- `dev-portal-2026`: Replace the minimum portal and workflow-based routine testing contract with one complete portal, one Pages publisher, deterministic API-reference staging, and deployment-only Actions behavior.
- `documentation-truth-gate`: Extend current-versus-historical validation to estate-wide source classification, provenance, private-source rejection, and complete current-authority coverage.
- `readme-presentation`: Extend repository presentation behavior to a working public portal link and consistent disposition/navigation across UAR-owned and generated README files while excluding vendored content from semantic rewriting.

## Impact

- **Documentation and site:** Establishes the source manifest, route manifest, provenance records, historical banners, and validation contract consumed by every later change in `uar-branded-documentation-site`.
- **GitHub Pages:** Constrains later workflow work to one deployment publisher; routine documentation verification remains local.
- **KBD/OpenSpec:** KBD workflow state must advance this registered change through proposal, specs, design, tasks, execution, and verification before later documentation changes consume the contract. The obsolete portal change requires an explicit supersession disposition.
- **Runtime UX:** No runtime or React application behavior changes. The later portal will document the shipped UX and reuse its brand contract without altering application state or interaction flows.
- **Provider compatibility:** No provider, model, credential, or inference behavior changes; documentation must report those behaviors per supported profile and evidence source.
- **Realtime state:** No SSE, AG-UI, A2UI, entity-graph, or runtime event behavior changes; public documentation must distinguish live runtime behavior from examples and historical designs.
- **Dependencies and APIs:** No runtime dependency or public API change. Later portal implementation may adjust documentation-only dependencies and local validators under this contract.
