# Change 23: Hosted rustdoc + TypeDoc + Docusaurus IA + ADRs

## Why

UAR has accumulated a large `docs/` directory and a Docusaurus `website/`, but the two are not integrated into a unified, hosted developer portal. The grade-A release requires a public documentation site that includes:

- Generated API references (rustdoc for Rust, TypeDoc for TypeScript, Sphinx for Python)
- A clear Docusaurus information architecture
- Prose linting with a project-specific Vale configuration
- Architecture decision records documenting the major grade-A choices

This change delivers the minimum viable portal infrastructure so that subsequent changes can publish SDK cookbooks and Storybook visual regression results without re-inventing the hosting pipeline.

## What Changes

- Reorganize `website/docs/` into the required IA sections: architecture, configuration, sdk-rust, sdk-python, sdk-typescript, rag, a2ui, governance, supply-chain, and contributing.
- Update `website/docusaurus.config.ts` and `website/sidebars.ts` to expose the new IA and add links to generated API references.
- Add root-level npm scripts: `docs:start`, `docs:build`, and `docs:lint`.
- Add a Vale configuration (`.vale.ini`) and a UAR style rule set under `.github/styles/UAR/`.
- Add an ADR template at `docs/adr/0001-record-architecture-decisions.md` and 10 ADRs covering the grade-A decisions.
- Add `.github/workflows/docs.yml` to build the Docusaurus site, run Vale, and deploy to `gh-pages` on pushes to `main`.
- The workflow includes placeholders for rustdoc and TypeDoc generation; the wiring is in place even though the generated content is not fully produced in this change.

## Capabilities

### New Capabilities

- `dev-portal-2026`: the hosted, unified developer portal for UAR documentation and API references.

## Impact

- **Documentation:** New IA sections are visible on the Docusaurus site and linked from the README.
- **CI:** A new `docs.yml` workflow runs on every push to `main` and publishes to GitHub Pages.
- **Developer experience:** Contributors can run `pnpm docs:lint` to check prose style before committing.
- **Architecture governance:** ADRs are now the canonical location for recording grade-A decisions.

## Out of Scope

- Full content for every IA section (this change provides stubs and structure; content is populated by other changes).
- Generating all rustdoc/TypeDoc/Sphinx content end-to-end (the build pipeline is wired; content generation is owned by SDK and runtime changes).
- Storybook/Chromatic visual regression (Change 25).
- Custom domain setup beyond the default `*.github.io` URL (operator can configure `url` in `website/docusaurus.config.ts` later).
