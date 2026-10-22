# Changelog

All notable changes to Universal Agent Runtime are documented here. UAR follows
Semantic Versioning for Stable HTTP, event, configuration, and artifact
contracts identified in the product support matrix.

## 1.2.0 — 2026-10-22

- Recorded the existing multi-platform image from source
  `035bbdc702ba20daaa1add7294b35f5783ed6e0e` with canonical index digest
  `sha256:dec5284e97b78ff65a66ef19eb43632154e23ea946f05a1d241cf979dd09566d`.
  The release publishes `1.2.0` and its `v1.2.0` alias to this same index.
- Added streaming OpenAI budget-error terminals, MCP catalog bootstrap and
  retained transport cleanup changes, credential tenancy/migration APIs and
  authenticated credential capture, and retained AG-UI terminal projection.
- Documented Kubo integration limits: OpenAI client-supplied tools are not
  forwarded; AG-UI rejects frontend tools. See the
  [v1.2.0 integration contract](docs/releases/v1.2.0-kubo.md).
- Image build/publication succeeded; functional acceptance remains pending.
  The source tag is retrospective and does not establish a tag-triggered build.
  v1.1.0 acceptance does not certify this image.

## 1.0.0 — 2026-07-12

- Established the Stable minimal server bundle with embedded SurrealDB.
- Published versioned OpenAI-compatible, AG-UI, A2UI, MCP, configuration, and
  release-evidence contracts.
- Added dependency-owned capability bundles and reproducible offline builds.
- Added provider evidence tiers, governed tool execution, React Runtime
  Console certification, and end-to-end knowledge/RAG coverage.
- Added cross-platform archive validation, operational resilience testing,
  SBOMs, checksums, signatures, provenance, and non-root container evidence.

Preview and Experimental capabilities remain governed by
[`docs/product-support-matrix.md`](docs/product-support-matrix.md) and do not
receive Stable compatibility guarantees.
