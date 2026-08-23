# Specification — `uar-developer-reference-documentation-review`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: evaluate the fifteen UAR API, protocol, tool, SDK, configuration,
  installation, deployment, and upgrade guides as a bounded source artifact.
- Deterministic execution: required for manifest structure, source authority,
  profile limits, publication safety, link boundaries, and negative controls.
- Inputs: `docs/publication/developer-reference.json`, the fifteen guides, four
  compatibility pages, current source authorities, local validators, and the
  scoped working-tree diff.

## Target state

- API and protocol pages distinguish router behavior, generated summaries,
  compatibility adapters, event families, discovery, authorization, and state.
- Tool documentation keeps execution in the trusted host and rejects the JWT
  proxy as a production gateway.
- SDK pages separate checked-in source, generated reference staging, and
  independently observed registry publication.
- Configuration binds anonymous operation to loopback and identifies runtime
  schema discovery as the exact build-specific authority.
- Installation, deployment, upgrade, health, data compatibility, and rollback
  are separate claims with explicit profile limits.

## Unknowns and evidence limits

- Production build, rendered diagrams, browser navigation, responsive behavior,
  accessibility, search, and deployed Pages routes remain deferred.
- No protocol-conformance, registry, installation, runtime-health, migration,
  rollback, provider, or inference execution is part of this source review.
- The full publication gate is expected to remain red until later route,
  README, and historical-source reconciliation changes finish.

## Uncomfortable fact

The repository contains deploy examples with development credentials, an
authentication-disabled default, floating container tags, and platform-specific
storage assumptions. The public guide exposes those boundaries instead of
turning the examples into a production recommendation.
