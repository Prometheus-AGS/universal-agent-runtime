# 2. Dual-license the runtime under AGPL and the SDKs under MIT

Date: 2026-07-13

## Status

**Superseded** by [ADR-0017](0017-relicense-runtime-to-mit.md) (2026-08-07), which
relicensed the entire repository — runtime, SDKs, and tools — under `MIT` and removed
the commercial dual-license and the CLA-lite clause. The record below is retained as
the historical rationale and no longer describes the project's licensing.

## Context

The runtime was originally AGPL-3.0-only. Competitor SDKs are predominantly permissive, which limits commercial adoption of UAR SDKs. The runtime must remain copyleft to preserve the open-source moat, while the SDKs need a permissive license to match market expectations.

## Decision

- Keep the runtime server and its source under `AGPL-3.0-only` plus a commercial dual-license.
- License the Rust, Python, and TypeScript SDKs under `MIT`.
- License documentation under `CC-BY-4.0`.
- Add a CLA-lite forward-going clause to `CONTRIBUTING.md`.

## Consequences

- SaaS deployers can purchase a commercial license for the runtime; SDK consumers face no copyleft obligations.
- The open-letter process for SDK contributors is simplified because no third-party contributors were found in the SDK history.
- `tools/license-check.sh` validates the declared licenses in CI.

## Alternatives considered

- MIT for everything: rejected because it removes the copyleft moat for the runtime.
- GPL for SDKs: rejected because it matches zero named competitors and would block adoption.
