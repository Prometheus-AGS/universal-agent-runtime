## Context

See `proposal.md` for motivation. The current checkout contains 39 tracked
READMEs: one root, 31 subordinate UAR-owned files, five identical
iterative-evolver copies, and two vendored files. The original phase assessment
counted 38 because `docs/publication/README.md` did not exist yet. Current
portal content covers most product routes, but the frozen manifest still lacks
documents for chat, A2UI artifacts, compiler, A2UI testing, and About. The
composed publication gate also exposes historical/current source files whose
raw paths or payload examples cannot be published directly.

The Docusaurus production build, browser/accessibility certification, Pages
deployment, and repository metadata mutation remain final-phase work. This lane
must not edit shared Docusaurus navigation/theme or the frozen route manifest.

## Goals / Non-Goals

**Goals:**

- Derive and validate the README denominator from Git rather than a copied
  count.
- Give every README one ownership, status, authority, and action record.
- Make the root README the branded repository entry point while keeping the
  portal authoritative for detailed behavior.
- Make subordinate READMEs concise local contracts rather than competing
  product manuals.
- Regenerate five mirror files from one pinned source and prove two vendored
  files are unchanged.
- Materialize the five remaining frozen product routes and make truth/link
  validation understand Docusaurus absolute routes.
- Normalize or reclassify retained source documents that the publication
  sanitizer currently rejects, without publishing raw private history.

**Non-Goals:**

- Changing runtime, React application, provider, realtime, protocol, package,
  dependency, submodule pin, release, or deployment behavior.
- Proving registry publication, runtime health, inference, installation,
  migration, rollback, rendered-site quality, or cross-profile readiness.
- Rewriting vendored documentation or historical decisions into current prose.
- Running the production build or browser/accessibility suite before all phase
  content is complete.

## Decisions

### 1. Use an exact README authority manifest

Add `docs/publication/readme-estate.json` with one entry per tracked README and
fields for path, class, status, owner, current authority, action, profiles, and
optional generated source or vendored baseline hash. A validator compares the
manifest to `git ls-files`, rejecting missing, duplicate, or extra entries.

The denominator is therefore 39 for this checkout but is never hard-coded as a
permanent estate fact. The validator reports the observed count.

**Alternative considered:** validate only basenames through `sources.json`.
Rejected because that proves classification rules overlap correctly, not that
each README received an editorial decision.

### 2. Keep the root README useful but bounded

The root uses the existing dark wordmark, current portal tagline, license,
version, and documentation badges, current Mermaid boundaries, and a short
source quickstart. Detailed API, security, deployment, SDK, skills, and history
material links to current portal authorities. Status badges use static facts or
the observed portal URL; no routine-test workflow badge or availability badge
is introduced.

**Alternative considered:** mirror the whole portal in the README. Rejected
because it recreates the drift this change is removing.

### 3. Reconcile subordinate READMEs by local ownership class

Current package/directory READMEs retain local build and contribution detail
that is verified from nearby manifests/source, add a current-authority link,
and state profile or publication limits where relevant. Placeholder README
content is replaced with an honest inventory/status. The standalone
code-interpreter, universal-plugin, and WebSocket-realtime document sets are
historical because the current source instead contains in-process sandbox
tools, an unwired WASM loader contract, and entity-change SSE. Historical
READMEs receive a dated supersession banner and current-authority link; their
historical body is otherwise preserved except for publication-safe path
normalization.

**Alternative considered:** replace all subordinate files with one-line portal
links. Rejected because local package commands and directory contracts are
useful and often more precise than product-level prose.

### 4. Generate iterative-evolver mirrors from the pinned skill source

Treat
`crates/prometheus-skill-system/skills/process/iterative-evolver/README.md` as
the canonical checked-out source. Add a deterministic local sync script that
copies its bytes to the five root tool mirrors and a check mode that performs no
writes. The submodule and its pin remain unchanged.

**Alternative considered:** edit all five mirrors independently. Rejected
because equality today would not prevent the next drift.

### 5. Protect vendored READMEs by immutable baseline hashes

Record SHA-256 values for `vendor/git/README.md` and the surreal-memory README
in the manifest. Validation checks those hashes and the permitted-surface audit
rejects vendor diffs. A future upstream pin change must update the baseline in
the change that owns that pin.

**Alternative considered:** add current-authority banners to vendor files.
Rejected because that would misattribute UAR editorial content to upstream
material.

### 6. Complete frozen navigation with five bounded pages

Add `product/chat`, `product/a2ui`, `compiler/overview`,
`product/a2ui-testing`, and `about`. These pages orient readers to shipped UI
surfaces, profile limits, and deeper current guides; they do not duplicate the
workflow, protocol, or architecture authorities. The frozen route manifest and
shared sidebars remain unchanged.

**Alternative considered:** change document IDs to point at nearby existing
pages. Rejected because the route inventory intentionally distinguishes product
screens from the protocols and concepts they expose.

### 7. Resolve retained-source publication failures by disposition

Current narrative sources that are useful publicly are normalized: machine
paths become neutral placeholders and historical material receives banners.
Raw assessment JSON or protocol/event exemplars that are unsuitable as direct
public pages are classified as private-synthesis-only or historical synthesis
inputs with a current portal authority. No source is made public merely to turn
the validator green.

**Alternative considered:** weaken the sanitizer for old paths and raw payload
keys. Rejected because those controls protect the final assembled site and raw
history boundary.

### 8. Compose local fail-closed controls only after content completion

Add a README-estate validator and isolated controls for denominator drift,
duplicate/missing ownership, missing authority, stale current guidance,
historical banner loss, mirror drift, vendor mutation, root hero/badge/link
loss, missing frozen route, unsafe public content, cross-profile claims, and
routine GitHub Actions tests. Extend the truth gate so `/docs/...` links resolve
through actual Docusaurus IDs/routes instead of the repository filesystem.

The final production build and browser/accessibility gates remain deferred.

**Alternative considered:** rely on editorial review and Docusaurus broken
links. Rejected because neither proves mirror/vendor invariants or observes the
unsafe failure modes.

## Risks / Trade-offs

- **[Risk] The README manifest becomes another stale inventory.** → Derive the
  checkout set with Git and fail on any missing, duplicate, or extra entry.
- **[Risk] Bulk link insertion makes local READMEs noisy.** → Use one concise
  authority block and retain only local details that are source-backed.
- **[Risk] The pinned skill source changes formatting or content.** → Byte-copy
  from the checked-out pin; any pin update creates an explicit five-file diff.
- **[Risk] Historical normalization could erase evidence.** → Add banners and
  neutralize only public-unsafe path/payload material; otherwise preserve the
  body and keep raw private evidence outside the site.
- **[Risk] New product pages duplicate existing guides.** → Keep them as short
  screen entry points with dependency links and profile limits.
- **[Trade-off] Source validation cannot prove rendered navigation.** → Defer
  build, visual, keyboard, accessibility, and deployed-route proof to the final
  certification change.

## Migration Plan

1. Add the exact README manifest, sync/check tool, validator, and controls.
2. Reconcile the root README and each subordinate UAR-owned README by manifest
   class; regenerate the five mirrors from the pinned source.
3. Record and verify vendored hashes without editing vendor content.
4. Add the five missing product-route pages and Docusaurus-aware truth-link
   resolution without changing frozen navigation contracts.
5. Normalize or reclassify the retained sources identified by the composed
   publication gate.
6. Run isolated controls, TypeScript and bounded documentation composition,
   strict OpenSpec, artifact-refiner content review, and scoped diff audit.
7. Record row-form evidence, transition KBD, and commit independently without
   pushing the partial phase.

Rollback is a revert of this documentation commit. No runtime state, public
API, dependency, submodule pin, package, image, or deployed site changes.
