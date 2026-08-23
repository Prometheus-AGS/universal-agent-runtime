## Why

UAR ships several HTTP and streaming adapters, three SDK source packages,
native and MCP tool surfaces, layered configuration, and multiple deployment
profiles, but the public portal currently mixes stale endpoint summaries with
unsupported publication claims. Developers and operators need one
source-grounded path from interface selection through configuration and
deployment, with generated-reference and registry availability stated exactly.

## What Changes

- Publish current guides for UAR HTTP/SSE, OpenAI- and Anthropic-compatible
  adapters, AG-UI/A2UI, MCP, A2A, native tools, and generated API references.
- Reconcile the Rust, Python, and TypeScript SDK guides against their source
  packages, examples, build metadata, and actual Pages staging; remove the
  unsupported claim that a Python generated reference is hosted.
- Consolidate configuration authority, installation, deployment, packaging,
  profile limits, version support, upgrade, and rollback guidance.
- Add a classified developer-reference manifest plus bounded local validation
  and observed negative controls for missing interfaces, invented endpoints,
  unsupported publication claims, unsafe configuration, profile transfer, and
  deployment-only workflow drift.
- Keep the production build, browser/accessibility pass, Pages deployment, and
  public-route checks deferred until all phase content changes are complete.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Require the portal to document the delivered APIs,
  protocols, SDK source packages, tool surfaces, configuration authority, and
  deployment paths with current-source provenance and explicit publication,
  profile, security, and compatibility limits.

## Impact

- Documentation: API, protocol, tool, SDK, configuration, installation,
  deployment, and upgrade pages beneath `website/docs/`, with concise
  compatibility treatment for older duplicate entry points.
- Publication tooling: a developer-reference authority manifest and bounded
  local validator/control scripts composed into the existing publication gate.
- Runtime UX: documents the shipped protocol, tool, provider/configuration, and
  settings surfaces; no React or runtime behavior changes.
- Provider compatibility and realtime state: documents provider-neutral model
  addressing and the current SSE/event adapters without changing routing or
  persistence.
- KBD: transition this registered change through Execute after row-form evidence
  passes, then advance to `reconcile-uar-readme-estate`.
- Dependencies and public APIs: unchanged.
