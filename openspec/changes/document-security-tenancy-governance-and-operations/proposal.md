## Why

UAR exposes authentication, credential, governance, approval, realtime, and
operational surfaces, but the public portal does not explain how those controls
compose or where their guarantees stop. Operators need source-grounded guidance
that distinguishes `server-full`, `minimal`, and `embedded-mobile`, especially
where current behavior is permissive or process-local rather than a complete
security or durability boundary.

## What Changes

- Publish security guides for JWT/JWKS authentication, RustCrypto provider
  ownership, API keys, scoped provider credentials, and the exact tenant
  boundaries currently enforced.
- Publish governance and approval guides that distinguish Cedar denial from
  human approval, document the five-minute fail-closed approval timeout, and
  state the enabled/disabled profile behavior and permissive policy-load
  fallback without overstating enforcement.
- Publish operations guides for the runtime console, run inspection and
  cancellation, metrics/logging/tracing, cost estimates and process-local
  budgets, realtime/SSE reconnect behavior, graceful shutdown, persistence,
  backup, and recovery.
- Add a classified security/operations authority manifest and bounded local
  validation with observed failing controls for missing boundaries, unsafe
  credential examples, unsupported governance claims, and omitted profile or
  durability limits.
- Keep the production site build, browser/accessibility pass, deployment, and
  public-route validation deferred until every phase content slice is complete.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Require the public portal to document the delivered
  security, isolation, governance, approval, observability, cost, realtime,
  shutdown, persistence, and recovery contracts with current-source provenance
  and explicit profile and durability limits.

## Impact

- Documentation: new pages beneath `website/docs/security/`,
  `website/docs/tenancy/`, `website/docs/governance/`, and
  `website/docs/operations/`, plus compatibility treatment for existing
  security, governance, backup, and troubleshooting pages where needed.
- Publication tooling: a security/operations authority manifest and bounded
  local validator/control scripts composed into the existing publication gate.
- Runtime UX: documents the shipped `/admin/auth`, `/admin/credentials`,
  `/admin/approvals`, `/admin/runtime`, `/admin/runs`, and `/admin/cost`
  surfaces; no React or runtime behavior changes.
- Provider compatibility and realtime state: provider behavior is unchanged;
  the guides describe masked credential metadata and the current multiplexed
  SSE/reconnect path without claiming durable replay where none exists.
- KBD: transition this registered change through Execute after row-form evidence
  passes, then advance to `document-apis-sdks-tools-and-deployment`.
- Dependencies and public APIs: unchanged.
