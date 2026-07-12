## Context

The former release workflow diverged from ordinary CI: Node 18 and Bun replaced the pinned pnpm workspace, `--all-features -D warnings` ignored the supported bundle contract, Redis and test-only config were assumed, and archives were uploaded without proving they could be installed or started. The product matrix declares Linux, macOS, and Windows support for the Stable minimal bundle.

## Goals / Non-Goals

**Goals:**

- Reuse Node 22, pnpm 10.33.0, Cargo warning policy, and named feature bundles.
- Build native Stable archives for Linux x64/arm64, macOS x64/arm64, and Windows x64.
- Extract each archive outside the checkout, verify its version, start it on port 1906, and probe readiness.
- Retain archives and a complete offline source bundle as workflow evidence.

**Non-Goals:**

- Claim desktop, mobile, or browser-WASM certification.
- Publish a GA release during workflow development.
- Add supply-chain signing or provenance, which belongs to the next dedicated change.

## Decisions

1. Native GitHub-hosted runners are used instead of cross-compilation so the produced binary is actually executed on its advertised architecture. Current official labels are `ubuntu-24.04-arm`, `macos-15-intel`, `macos-15`, and `windows-latest`.
2. The Stable `minimal` bundle is the distributable binary baseline. Preview bundles compile in validation but are not mislabeled as platform-certified archives.
3. Archive smoke tests operate only on extracted content. This catches missing binaries/config/licenses and checkout-relative startup assumptions.
4. A `release-test-*` tag triggers the same non-publishing graph. Only semantic-version tags or an explicit publish input can attach draft release artifacts.

## Risks / Trade-offs

- [Hosted labels can change] → A static contract gate records the expected labels; runner selection is reviewed with the official runner-image inventory.
- [Native matrix is slower] → Jobs run concurrently and compile only the Stable bundle.
- [A placeholder provider key could be mistaken for a usable secret] → It is explicitly a smoke placeholder and no provider request is made.
- [Tag testing creates remote refs] → Use a clearly non-GA `release-test-*` tag and never enter the publish job.

## Migration Plan

Replace the workflow, run local structural/OpenSpec checks, commit, push a non-GA test tag, and retain the workflow run URL/result. Rollback is a workflow revert and removal of the test tag; no customer release is created.

## Open Questions

None.
