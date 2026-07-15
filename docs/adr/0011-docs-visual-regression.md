# 11. Host a unified docs portal with Docusaurus and add visual regression

Date: 2026-07-14

## Status

Accepted

## Context

UAR has accumulated documentation in `docs/` and a Docusaurus `website/`, but they are not integrated into a unified, hosted developer portal. The grade-A release requires public documentation with generated API references, a clear information architecture, and a place to publish visual regression results.

## Decision

- Host the Docusaurus site on GitHub Pages at `prometheus-ags.github.io/universal-agent-runtime/`.
- Reorganize `website/docs/` into sections: architecture, configuration, sdk-rust, sdk-python, sdk-typescript, rag, a2ui, governance, supply-chain, and contributing.
- Wire generated rustdoc and TypeDoc outputs into `/docs/api/rust` and `/docs/api/typescript`.
- Use `vale.sh` with a UAR-specific style configuration for prose linting.
- Maintain ADRs in `docs/adr/` and publish them as part of the portal.
- Add Storybook 8 with Chromatic for visual regression (Change 25) and publish the Storybook as a static site linked from the portal.

## Consequences

- There is a single canonical URL for all UAR documentation.
- SDK users can navigate from narrative docs to generated API references without leaving the site.
- Prose quality is enforced by Vale in CI.
- ADRs provide a transparent record of architectural decisions.

## Alternatives considered

- Use a separate docs host (ReadTheDocs, Vercel): rejected because GitHub Pages is zero-cost and integrates cleanly with the existing GitHub Actions workflows.
- Keep the current flat `docs/` directory without Docusaurus: rejected because discoverability is poor and API references are not linked.
