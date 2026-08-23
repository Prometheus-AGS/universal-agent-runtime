# UAR developer reference content review

## Scope

This review covers the fifteen guides declared in
`docs/publication/developer-reference.json` and four compatibility pages. It is
a `direct:content` source review. It does not certify the production build,
rendered diagrams, browser behavior, accessibility, deployment, protocol
conformance, package registries, runtime health, migration, rollback,
inference, or any runtime profile.

## Constraint evaluation

| Constraint | Result | Observed evidence | Limit |
|---|---|---|---|
| Complete classified developer guides | Satisfied | Direct validation reported `Documentation developer-reference validation passed (15 guides)`. Missing-guide and unclassified-authority controls rejected their fixtures before the complete source passed. | Source files, manifest, classification, frontmatter, markers, compatibility pages, and local links only. |
| Truthful API, protocol, and tool boundaries | Satisfied | Guides separate the router from OpenAPI, adapter subsets from protocol parity, live events from durable state, discovery from authorization, and trusted-host execution from the loopback JWT proxy. Five unsafe claim mutations were rejected. | No request, protocol-conformance suite, MCP server, A2A peer, tool, or authorization flow was executed. |
| Honest SDK and publication status | Satisfied | Rust and TypeScript generated reference staging is distinguished from Python's local Sphinx source, and version metadata is not treated as registry publication. Hosted-Python and registry-from-metadata mutations were rejected. | No registry lookup, package installation, SDK execution, or generated-reference deployment was observed. |
| Safe profile-bounded operator guidance | Satisfied | Anonymous mode is loopback-only; development defaults, pinned-image requirements, health/readiness, profile ownership, upgrade verification, and rollback are explicit. Unsafe-listener, profile-transfer, missing-health, missing-rollback, and routine-Actions-test mutations were rejected. | No container, cluster, datastore, health endpoint, backup, upgrade, or rollback was executed. |
| Bounded public-safe source evidence | Satisfied | TypeScript and all bounded source-control suites exited 0. The private-excerpt mutation was rejected, and the scoped public guides contain no machine path, credential, private key, raw payload, or raw private history. | The full publication gate remains intentionally red on later missing routes and unreconciled source material; no final-site claim is made. |

## Structural and technical review

- The guide chain moves from API selection through protocol translation, tools,
  SDKs, configuration, installation, deployment, upgrade, and rollback.
- OpenAPI is described as a generated summary rather than an exhaustive router
  contract, and every named adapter is limited to implemented fields/events.
- MCP discovery, catalog visibility, policy/risk/approval decisions, and host
  execution remain separate steps.
- SDK source presence, generated-reference staging, and registry publication
  are independent claims; Python has no hosted generated-reference claim.
- Deployment guidance identifies development Compose defaults, floating-image
  risk, platform-specific Helm assumptions, and deployment-owner boundaries.

## Deterministic checks observed

- `node scripts/test-documentation-developer-reference.mjs` rejected fifteen
  mutated defects, then passed the complete fixture.
- `npm --prefix website run typecheck` exited 0.
- Publication, architecture, brand, product-workflow, security/operations,
  developer-reference, staging, and GitHub Actions policy controls exited 0.
- `openspec validate document-apis-sdks-tools-and-deployment --strict` reported
  the change valid.

## Regression review

No shared Docusaurus configuration, global navigation, branding source, or
`docs/publication/routes.json` change belongs to this slice. No runtime, React
application, provider behavior, dependency, vendored source, README, lockfile,
raw private history, release, or deployment workflow belongs to this slice.

## Convergence

All five blocking content constraints have observed source evidence, and the
negative controls demonstrate that the required failure modes are detected.
The bounded content review converges in one iteration. Full publication,
production build, browser/accessibility, deployment, protocol, registry,
runtime-health, migration, rollback, inference, and cross-profile claims remain
explicitly deferred.
