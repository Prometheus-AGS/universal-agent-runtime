# Specification — `uar-documentation-publication-contract`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: verify that the publication-contract OpenSpec bundle and its local
  validators form a complete, bounded, fail-closed foundation for the later
  documentation changes.
- Deterministic execution: required.
- Inputs: the proposal, design, delta specs, tasks, publication manifests,
  validator controls, current-tree validation output, and scoped git diff.

## Target state

- Every selected documentation source resolves to exactly one declared
  publication disposition.
- Every product surface resolves to one planned document ID or explicit
  exclusion.
- Private history can inform reviewed synthesis but cannot be copied into the
  public source or built site.
- The contract detects zero or multiple source matches, route omissions,
  invalid provenance, unsafe content, child-validator failures, and competing
  Pages publishers.
- The current repository's known missing pages and competing publishers remain
  visible failures owned by later registered changes.
- Strict OpenSpec and the scoped diff audit pass without changing runtime or UI
  behavior.

## Unknowns and evidence limits

- `versions.toml` is named by the standing project rules but is absent from this
  checkout, so it cannot be cited as inspected evidence.
- This foundation does not prove the final portal builds or deploys; later
  changes own content, branding, workflow repair, and final publication.

## Uncomfortable fact

A publication validator that passes this checkout now would be dishonest. The
tree still has two Pages publishers and most planned pages do not exist; this
foundation is correct only if it reports those conditions as failures.
