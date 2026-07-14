# Architecture Decision Records

This directory contains the architecture decision records (ADRs) for the Universal Agent Runtime project. ADRs document significant decisions that affect the project's structure, dependencies, or public interface.

## Index

| ADR | Title | Date | Status |
|---|---|---|---|
| [ADR-0001](0001-record-architecture-decisions.md) | Record architecture decisions | 2026-07-14 | Accepted |
| [ADR-0002](0002-dual-license-agpl-mit.md) | Dual-license the runtime under AGPL and the SDKs under MIT | 2026-07-13 | Accepted |
| [ADR-0003](0003-coverage-baseline-60-percent.md) | Set a 60% coverage baseline on day one | 2026-07-13 | Accepted |
| [ADR-0004](0004-central-uar-error-enum.md) | Introduce a central `UarError` enum | 2026-07-13 | Accepted |
| [ADR-0005](0005-config-rs-schemars-secrecy.md) | Migrate configuration to `config-rs` with `schemars` and `secrecy` | 2026-07-13 | Accepted |
| [ADR-0006](0006-slsa-l3-supply-chain.md) | Adopt SLSA Level 3 and OSV/Grype for supply-chain security | 2026-07-13 | Accepted |
| [ADR-0007](0007-sdks-1-0-0-mit.md) | Release the SDKs as 1.0.0 under MIT | 2026-07-13 | Accepted |
| [ADR-0008](0008-rag-citation-stream.md) | Emit a RAG citation stream | 2026-07-13 | Accepted |
| [ADR-0009](0009-a2ui-vendor-webcore.md) | Vendor `@a2ui/web_core` and `@a2ui/react` and build a UAR-owned renderer | 2026-07-13 | Accepted |
| [ADR-0010](0010-a2ui-renderer-on-webcore.md) | Build the A2UI renderer on webcore with React and shadcn/ui | 2026-07-13 | Accepted |
| [ADR-0011](0011-docs-visual-regression.md) | Host a unified docs portal with Docusaurus and add visual regression | 2026-07-14 | Accepted |

## Process

1. When a significant architectural decision is needed, draft an ADR using the template in ADR-0001.
2. Open the ADR for review in the same PR that implements the decision.
3. Mark the ADR as `Accepted` when the PR is merged.
4. If a decision is superseded, mark the old ADR as `Superseded` and link to the new ADR.
