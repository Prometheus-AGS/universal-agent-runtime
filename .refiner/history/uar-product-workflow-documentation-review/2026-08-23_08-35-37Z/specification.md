# Specification — `uar-product-workflow-documentation-review`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: evaluate the seven UAR provider, model, inference, agent, skill,
  knowledge, and memory workflow guides as a bounded source artifact without
  claiming a rendered site, fresh runtime execution, deployment, or
  cross-profile certification.
- Deterministic execution: required for provenance, structure, dependency-order
  navigation, source classification, publication safety, and negative-control
  evidence.
- Inputs: `docs/publication/product-workflows.json`, the seven public guides,
  the `/docs/skills` compatibility page, classified source records, current
  implementation authorities, local validators and controls, and the scoped
  working-tree diff.

## Target state

- Seven guides form one dependency sequence from provider configuration and
  model selection through inference, agents, skills, knowledge, and memory.
- Catalog metadata, configured availability, genuine inference, durable state,
  live events, and UI projections remain distinct.
- Skill provenance, scoped precedence, next-request binding, restart behavior,
  tombstone, and restore safety match the current canonical specifications.
- Knowledge and memory remain separate authorities, each with observable
  ingestion/recall boundaries and profile-specific limits.
- Retained genuine-model observations are reviewed summaries bounded to their
  provider/model, packaged boundary, source SHA, checkout/date, and
  `server-full` profile.
- Public pages exclude raw private evidence, credentials, machine paths, keys,
  and raw event/session payloads.

## Unknowns and evidence limits

- The complete Docusaurus production build, rendered Mermaid diagrams, browser
  navigation, responsive behavior, accessibility, search, and deployed Pages
  routes are intentionally deferred to the final phase gate.
- No fresh runtime or model request is part of this documentation-source review.
  Retained 1.0 observations cannot certify the current checkout or another
  profile.
- Existing narrative documents may be stale. Present-tense statements resolve
  against the current implementation and canonical OpenSpec records.

## Uncomfortable fact

Detailed workflow prose can still become dangerous when it outlives the code:
a correct guide today can turn a catalog row, UI selection, or stored record
into a false execution claim after an implementation change. The manifest and
negative controls detect named drift classes; they do not eliminate the need to
reconcile documentation with every future behavior change.
