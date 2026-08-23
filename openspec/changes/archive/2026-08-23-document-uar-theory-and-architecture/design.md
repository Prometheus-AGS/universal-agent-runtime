## Context

See `proposal.md` for motivation and `specs/dev-portal-2026/spec.md` for the
observable contract. The existing `website/docs/architecture/intro.md` combines
three useful diagrams with a short narrative, but it cannot serve as current
authority for the trust model, lifecycle, profile differences, event/state
model, protocol boundaries, or graph delegation.

The implementation must derive present-tense claims from the current checkout.
The most relevant authorities are the profile definitions in `Cargo.toml`, the
runtime manager and graph engine, `NormalizedEvent`, the persistence trait and
embedded builder, current security/governance source, and canonical OpenSpec.
`versions.toml` is named by standing rules but absent from this checkout, so no
page may cite it as inspected evidence. Private `.prometheus` and KBD history is
reserved for later reviewed synthesis and will not be copied into these guides.

## Goals / Non-Goals

**Goals:**

- Establish a stable seven-page conceptual spine that later product, security,
  protocol, API, decision-history, and testing-history pages can link to.
- Make authority flow legible in prose even when Mermaid is unavailable.
- Make profile and provenance omissions deterministic local failures.
- Keep each page focused enough that readers can enter from search without
  reading the entire section first.

**Non-Goals:**

- This change does not document provider setup, skills, knowledge workflows,
  security operations, API endpoint catalogs, SDK usage, deployment, ADR history,
  or testing history; later registered changes own those subjects.
- It does not change runtime, React, realtime, provider, persistence, protocol,
  or build-profile behavior.
- It does not turn proposals or private history into present-tense product truth.
- It does not run the full production site build or browser/a11y certification
  before all documentation content is complete.

## Decisions

### 1. Use seven stable pages rather than one long architecture essay

The Architecture category will expose:

1. `architecture/intro` — why UAR exists and the map of the system;
2. `architecture/trust-boundary` — capability inversion and host authority;
3. `architecture/execution-lifecycle` — request, turn, step, tool, and terminal flow;
4. `architecture/state-and-events` — normalized events, persistence, and inspectable state;
5. `architecture/profiles` — `server-full`, `minimal`, and `embedded-mobile`;
6. `architecture/protocols` — typed entrances and their shared runtime boundary;
7. `architecture/delegation` — current graph execution and its limits.

A single large page was rejected because it produces weak search results,
unstable anchors, and no clear current authority for downstream guides. More
pages were rejected because provider, governance, and API details already have
separate registered owners.

### 2. Add a machine-readable architecture authority manifest

`docs/publication/architecture.json` will map every required guide to its file,
public route, applicable profiles, canonical specification records, current
source authorities, and required conceptual markers. It is distinct from
`docs/publication/routes.json`: that existing manifest maps product inventory
surfaces one-for-one and cannot honestly absorb non-UI architecture routes.

Each public guide will retain publication frontmatter with `source_records`
limited to classified OpenSpec/documentation records and a
`current_authority` route. The architecture manifest may additionally point to
Rust, Cargo, and frontend sources; a dedicated validator checks those paths
without pretending application source is itself a publication record.

Embedding all source paths only in prose was rejected because drift would be
invisible. Extending the product-route manifest was rejected because its
inventory-label cardinality is load-bearing.

### 3. Separate normative boundaries from illustrative flow

Every page will lead with a compact boundary statement, then explain mechanics,
then state profile limits and related guides. Mermaid diagrams illustrate the
same nodes and arrows described immediately in prose. Diagrams will not be the
sole carrier of authority, ordering, or failure semantics.

This favors repeated boundary callouts over a terse diagram-only site. The
repetition is intentional because architecture pages are search entry points,
not chapters that readers must consume serially.

### 4. Describe the current runtime, not the proposed 1.1 architecture

The narrative will distinguish:

- an agent/model producing intent from a host completing an effect;
- normalized events from durable storage;
- the simple tool loop from graph-driven orchestration;
- HTTP/SSE/A2A server composition from transport-free embedded calls;
- a configured persistence provider from conversational or process-local state;
- current graph delegation from the deferred subagent-provider architecture.

The attached remediation proposals and future-work documents are not current
authority. Planned typed IDs, session logs, spill stores, receipts, and component
host work must not appear as delivered merely because they are architecturally
desirable.

### 5. Validate structure and claims with a bounded local control

`scripts/validate-documentation-architecture.mjs` will validate the manifest,
required files/routes, profile enum, existing source authorities, frontmatter,
heading order, Mermaid plus explanatory prose, and required boundary markers.
`scripts/test-documentation-architecture.mjs` will copy the bounded source into
temporary fixtures and observe failures for a missing page, source, profile,
provenance record, profile limit, trust-boundary marker, and diagram explanation.

The control will be composed into the final publication validator but will not
run in GitHub Actions as a routine test. A production Docusaurus build remains
deferred to the phase-level certification change.

## Risks / Trade-offs

- **[Risk] Source paths can remain present while semantics change** → Require
  focused concept markers now and rely on later content/history reviews to catch
  semantic drift; do not describe path existence as full truth proof.
- **[Risk] Seven pages repeat boundary language** → Keep a stable one-sentence
  boundary block per page and cross-link instead of duplicating detailed prose.
- **[Risk] The governance source contains permissive constructors and evolving
  policy behavior** → Architecture pages describe only the host authorization
  position; the dedicated security/governance change owns operational defaults
  and fail-closed claims.
- **[Risk] `minimal` includes the server transport today** → State this exact
  feature composition and avoid using “minimal” as a synonym for library-only or
  transport-free; only `embedded-mobile` owns that boundary.
- **[Risk] Mermaid can hide ambiguity behind polished diagrams** → Every diagram
  receives adjacent node-and-flow prose that the validator requires.
- **[Trade-off] Final rendered evidence is delayed** → This change can establish
  content and source contracts but cannot claim browser quality until the
  complete portal is built and inspected.

## Migration Plan

1. Add the architecture authority manifest and its isolated fail-closed controls.
2. Rewrite the introduction and add the six focused architecture guides with
   classified provenance and explicit profile limits.
3. Reconcile category metadata and architecture links without changing unrelated
   portal navigation owned by later content slices.
4. Run the bounded architecture/publication controls, strict OpenSpec, and scoped
   source audit after content is complete; record row-form evidence.
5. Transition the registered KBD change and commit it independently. If the final
   site build later exposes a structural problem, revert this change commit or
   correct the affected guide before publication rather than weakening the gate.
