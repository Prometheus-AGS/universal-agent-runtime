# UAR architecture documentation content review

## Scope

This review covers the seven public architecture guides declared by
`docs/publication/architecture.json`. It evaluates source structure, provenance,
current-checkout alignment, profile boundaries, navigation, and publication
safety. It does not evaluate rendered output or runtime behavior.

## Constraint evaluation

| Constraint | Result | Observed evidence |
|---|---|---|
| Classified authority | Pass | The architecture validator accepted seven guides and found every classified record and current authority. Missing-source and missing-provenance controls failed as intended. |
| Coherent trust model | Pass | The sequence consistently places effects behind host identity, policy, and capability checks; request, turn, step, tool, terminal event, live event, and persistence are distinguished. Missing-boundary controls failed as intended. |
| Profile-bounded claims | Pass | The exact vocabulary is `server-full`, `minimal`, and `embedded-mobile`; every guide includes profile limits and explicitly rejects evidence transfer. Invalid-profile and missing-profile-limit controls failed as intended. |
| Delivered protocol and delegation | Pass | Protocols are adapters to shared execution control. Delegation documents the current simple loop and built-in graph while naming a general subagent-provider architecture as deferred. |
| Public-safe navigation | Pass | Routes are ordered from introduction through delegation, local architecture links resolve, every Mermaid diagram has a prose explanation, and the safety scan found no raw history, local paths, credentials, private payloads, or absent-version authority claims. |

## Deterministic evidence

- `node scripts/test-documentation-architecture.mjs` observed seven intended
  negative-control failures and one complete-fixture pass.
- `node scripts/validate-documentation-architecture.mjs` passed with seven
  guides.
- `npm --prefix website run typecheck` exited successfully.
- `node scripts/validate-documentation-brand.mjs` passed.
- `node scripts/test-documentation-publication.mjs` passed every publication
  control, including preservation of child-validator failures.

## Content assessment

The sequence answers the architectural questions in dependency order. The
introduction states why the runtime exists before naming components. The trust
guide fixes the authority boundary. The lifecycle and state guides distinguish
execution from observation and durability. The profile guide prevents feature
or evidence transfer. Protocol and delegation guides then explain extension
points without granting them independent authority.

The uncomfortable limitation is temporal: this material is accurate against
the reviewed checkout, but future runtime changes can make prose stale. The
manifest and local controls make that drift detectable only when maintainers run
them and update source markers responsibly.

## Deferred evidence

The complete Docusaurus build, rendered Mermaid output, browser navigation,
responsive layout, accessibility tree, and deployed GitHub Pages routes remain
unverified here. Runtime behavior and all cross-profile readiness claims also
remain unverified. Those gates belong to `certify-and-publish-uar-docs` after
the documentation estate is complete.

## Decision

Terminate the bounded content review. No blocking source-content constraint
remains, and no rendered, deployed, runtime, or cross-profile claim is made.
