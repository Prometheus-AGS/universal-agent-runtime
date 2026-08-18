## Why

Assessment H5/M7: the customer documentation remains incomplete and internally
inconsistent. The README has substantial architecture material but still lacks
SDK and skills orientation, a fabric relationship diagram, and a docs-site
entrypoint. The site lacks skills, deployment, security, and SDK overview
pages. OpenAPI reports 0.1.0, covers only the legacy skills reload endpoint,
and omits the current refresh endpoint and main customer route groups. Tracked
scratch test outputs remain at repository root.

## What Changes

- Complete README.md with SDK, skills, deployment, and docs-site orientation,
  plus a Mermaid view of the Flint service boundaries.
- Add customer docs for SDKs, skills, deployment, security, and architecture.
- Make Docusaurus render Mermaid and fail on broken site and Markdown links.
- Derive OpenAPI version metadata from the package and document the main
  customer route groups using paths verified against the Axum router.
- Remove five reviewed, tracked scratch outputs from repository root.

## Capabilities
### New Capabilities
- `customer-documentation`

## Impact
`README.md`, `website/docs/`, `website/docusaurus.config.ts`,
`website/package.json`, `website/package-lock.json`,
`src/uar/api/openapi.rs`, and five named root scratch artifacts.
