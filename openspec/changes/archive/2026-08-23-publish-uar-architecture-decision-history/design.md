## Context

The checkout contains 18 ADR files, 546 retained `.prometheus` files, 51
top-level KBD phase directories, 48 KBD reflections, and 184 OpenSpec change
directories at the start of this change. Those counts describe the inspected
checkout, not a permanent repository invariant. The raw corpus includes
machine-local context, event records, conversations, tentative plans, and
superseded claims, so direct publication would be both unsafe and misleading.

## Goals / Non-Goals

**Goals:**

- Give readers a dated, source-traceable account of the decisions that shaped
  the current runtime and documentation.
- Make supersession explicit and preserve the cost or limitation of each choice.
- Explain how ADR, OpenSpec, KBD, and append-only history differ as evidence.
- Make missing sources, status drift, raw-wiki sourcing, and omitted corrections
  deterministic local failures.

**Non-Goals:**

- This change does not publish raw logs, complete conversations, KBD event JSON,
  or wiki copies.
- It does not claim every historical proposal shipped or every accepted ADR
  remains current.
- It does not rewrite current architecture, runtime behavior, testing policy, or
  release status.
- It does not run the production build, browser, accessibility, or deployment
  gate before the whole documentation phase is complete.

## Decisions

### 1. Publish five focused history guides

The History category contains an overview, an architecture-decision index, a
timeline, a corrections ledger, and a provenance guide. The later testing-history
change owns the sixth guide, `history/testing-methodology`.

### 2. Keep traceability in a reviewed manifest

`docs/publication/architecture-history.json` records the inspected corpus and the
selected decisions, dates, dispositions, present authorities, and source files.
Public pages name reviewed record IDs and repository-facing ADR/OpenSpec records;
they do not expose raw log bodies. This permits audit without treating private
working memory as publication-ready prose.

### 3. Treat supersession as first-class data

A superseded record names its replacement. The selected correction ledger must
include licensing, frontend architecture, visual authority, JWT crypto,
verification location, inference evidence, and placeholder publication. A page
that lists the old position without its replacement is invalid.

### 4. Separate process evidence from product authority

ADRs explain architectural intent. OpenSpec expresses behavioral contracts and
change deltas. KBD records lifecycle, order, and reflection. `.prometheus`
retains decisions, incidents, and lessons. Current source and canonical specs
remain the authority for present-tense product behavior.

## Risks / Trade-offs

- **Selection bias:** a curated history cannot reproduce every retained thought.
  The manifest states the corpus denominator and selection rule.
- **Stale accepted ADRs:** accepted status can outlive implementation. Public
  entries link to current authority and do not equate acceptance with delivery.
- **Traceability versus privacy:** raw paths are useful to maintainers but raw
  content can be unsafe. The manifest carries internal references; public prose
  carries reviewed identifiers and summaries.
- **Narrative neatness:** preserving reversals makes the project look less linear.
  That is intentional; erasing corrections would destroy the useful history.

## Migration Plan

1. Add the history manifest and local fail-closed validator.
2. Reconcile the retained ADR index and publish the five history guides.
3. Run isolated negative controls only after content is complete.
4. Validate OpenSpec, publication composition, TypeScript, and scoped diff.
5. Record row-form evidence, transition KBD, and commit independently.
