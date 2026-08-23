# UAR security and operations content review

## Scope

This review covers the eleven source guides declared in
`docs/publication/security-operations.json` and the four compatibility pages.
It is a `direct:content` source review. It does not certify the production
build, rendered diagrams, browser behavior, accessibility, deployment, runtime
security, tenant isolation at runtime, backup/restore execution, process-signal
behavior, or any runtime profile.

## Constraint evaluation

| Constraint | Result | Observed evidence | Limit |
|---|---|---|---|
| Complete classified boundary guides | Satisfied | Direct validation reported `Documentation security/operations validation passed (11 guides)`. Missing-guide and unclassified-record controls rejected their mutated fixtures before the complete fixture passed. | Source files, manifest, classification, frontmatter, headings, markers, diagrams-as-prose, compatibility pages, and links only. |
| Verified security and tenancy claims | Satisfied | Authentication states that identity follows signature and claim checks; credentials are write-only and masked; tenancy is limited to the A2A task/context partition. Unsafe-credential, unverified-identity, and blanket-isolation mutations were rejected. | Documentation/source correspondence only; no live authentication, key rotation, or two-tenant request was run. |
| Truthful governance and approval limits | Satisfied | The guide names `server-full` Cedar coverage, profile exclusions, empty-policy denial, the current permit-all load-error fallback, and final effective-policy/Cedar denial. Universal-fail-closed, approval-override, and missing-timeout mutations were rejected. | No policy engine or approval flow was executed; logs/events are not represented as an immutable audit ledger. |
| Bounded operational state and recovery | Satisfied | The guides separate server execution, browser projections, external telemetry owners, transport reconnect, cost estimates/provider billing, signal shutdown, and functional restore read-back. Durable-realtime, authoritative-billing, missing-deadline, missing-read-back, and missing-state-owner mutations were rejected. | No production signal, scrape, collector, provider invoice, cold backup, or restored process was observed. |
| Public-safe source-only evidence | Satisfied | The private-excerpt mutation was rejected. TypeScript plus architecture, brand, product, security, and composed publication controls exited 0. The public-source scan found no machine path, credential, private key, raw event/session payload, raw private history, or inspected dependency-pin claim. | No production build, browser, accessibility tree, visual comparison, search interaction, deployment, or cross-profile evidence. |

## Structural and technical review

- The guide chain moves from authentication and credential custody through
  tenant scope, governance and approval, live operation, observability,
  realtime, cost, and recovery.
- HS256 and RS256/JWKS selection, registered claims, key-cache refresh, API-key
  behavior, probe exceptions, and RustCrypto provider conflict remain distinct.
- Tenant identity is constructed only from verified claims, but the current
  partition is explicitly not generalized beyond A2A tasks and contexts.
- `minimal` and `embedded-mobile` are exclusions from the `server-full` Cedar
  claim; the current policy-load error fallback is disclosed rather than
  normalized into a fail-closed claim.
- Human approval follows policy and cannot convert denial into permission.
- Run-manager history, browser PGlite, live entity buses, metrics, logs, traces,
  persisted cost entries, provider bills, datastore backups, and host-owned
  embedded lifecycle are described as separate authorities.

## Deterministic checks observed

- `npm run docs:security-operations:validate` exited 0 with eleven guides.
- The fourteen isolated negative controls printed `PASS`, followed by `PASS
  positive control: complete security/operations source`.
- `npm --prefix website run typecheck` exited 0.
- Architecture, brand, product-workflow, security/operations, and composed
  publication controls exited 0.
- `openspec validate document-security-tenancy-governance-and-operations
  --strict` reported the change valid.

## Regression review

No shared Docusaurus configuration, global navigation, branding source, or
`docs/publication/routes.json` change was introduced. No runtime, React
application, provider/security behavior, dependency, vendored source, README,
lockfile, raw private history, or deployment workflow belongs to this content
change. The final scoped diff gate remains the authority for that claim before
commit.

## Convergence

All five blocking content constraints have observed source evidence, and the
negative controls demonstrate that the required failure modes are detected.
The bounded content review converges in one iteration. Final-site build,
browser/accessibility, deployment, runtime-security, restore, process-signal,
and cross-profile claims remain explicitly deferred.
