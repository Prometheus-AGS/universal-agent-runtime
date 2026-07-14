## Why

Assessment H5/M7: the README has no mermaid diagrams, no fabric context, no
SDK/skills/deployment sections and no docs-site link; the docs site lacks
SDK/skills/deployment/security pages; OpenAPI reports 0.1.0 and a fraction
of routes; provider counts contradict.

## What Changes

- Rewrite README.md for the whole package with mermaid architecture, flow
  and scenario diagrams including the fabric (flint-realtime-fabric,
  flint-gate, flint-forge, flint-platform-agent) relationships.
- Add docs-site pages (SDKs, skills, deployment, security, architecture);
  set onBrokenLinks to throw; align OpenAPI version/coverage; fix provider
  count; clean repo root.

## Capabilities
### New Capabilities
- `customer-documentation`

## Impact
README.md, website/, openapi.rs, repo root hygiene.
