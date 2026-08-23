# Specification — `uar-security-operations-documentation-review`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: evaluate the eleven UAR authentication, credential, tenancy,
  governance, approval, runtime-console, run, observability, realtime, cost,
  and recovery/shutdown guides as a bounded source artifact without claiming a
  rendered site, deployed Pages site, runtime security certification, restore
  certification, or cross-profile readiness.
- Deterministic execution: required for provenance, structure, profile limits,
  source classification, publication safety, and negative-control evidence.
- Inputs: `docs/publication/security-operations.json`, the eleven public
  guides, four compatibility pages, classified source records, current
  implementation authorities, local validators and controls, and the scoped
  working-tree diff.

## Target state

- Authentication distinguishes verified identity, anonymous mode, API keys,
  probe exceptions, JWKS behavior, and RustCrypto provider conflict.
- Credential docs distinguish encrypted user records, operator fallback, and
  process- or database-owned durability.
- Tenancy states the verified-construction boundary and limits the present
  partition claim to A2A tasks and contexts.
- Governance exposes the `server-full` Cedar boundary, `minimal` and
  `embedded-mobile` exclusions, and the current permit-all policy-load-error
  fallback.
- Approvals cannot override effective-policy or Cedar denial and document all
  reject, close, cancellation, timeout, and single-use outcomes.
- Operations separate browser projections, process state, provider/model
  signals, UAR-owned signals, reconnect, cost estimates, billing, shutdown,
  and restore proof.
- Public pages exclude raw private evidence, credentials, keys, machine paths,
  and raw event/session payloads.

## Unknowns and evidence limits

- The complete Docusaurus production build, rendered Mermaid diagrams, browser
  navigation, responsive behavior, accessibility, search, and deployed Pages
  routes are intentionally deferred to the final phase gate.
- No runtime authentication attack, tenant-isolation test, provider call,
  backup, restore, or process-signal test is part of this documentation-source
  review.
- Present-tense statements resolve against current implementation and canonical
  OpenSpec. Older narrative material is provenance, not present authority.

## Uncomfortable fact

The current `server-full` composition falls back to permit-all when Cedar
policy files fail to load. Hiding that fact would turn a useful governance
guide into a dangerous deployment claim. The documentation names the fallback
and scopes the claim; it does not repair the runtime behavior.
