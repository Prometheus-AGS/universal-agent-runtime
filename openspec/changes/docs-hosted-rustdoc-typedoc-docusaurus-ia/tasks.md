## 1. Reorganize Docusaurus information architecture

- [x] 1.1 Create `website/docs/{architecture,configuration,sdk-rust,sdk-python,sdk-typescript,rag,a2ui,governance,supply-chain,contributing}` directories.
- [x] 1.2 Add stub `_category_.json` and `intro.md` (or `index.md`) files for each section so Docusaurus autogenerates the sidebar.
- [x] 1.3 Update `website/sidebars.ts` to expose the new IA order.
- [x] 1.4 Update `website/docusaurus.config.ts` navbar and footer to reference the new sections.

## 2. Wire API reference links

- [x] 2.1 Add `website/docs/api/index.md` describing the hosted API references.
- [x] 2.2 Add `/docs/api/rust` and `/docs/api/typescript` links to the navbar and API index page.
- [x] 2.3 Document the rustdoc/typedoc generation commands in the API index page (placeholders for full generation).

## 3. npm scripts

- [x] 3.1 Add `docs:start` to root `package.json` (`pnpm -C website start`).
- [x] 3.2 Add `docs:build` to root `package.json` (`pnpm -C website build`).
- [x] 3.3 Add `docs:lint` to root `package.json` (`pnpm -C website lint`).
- [x] 3.4 Add `lint` script to `website/package.json` that runs `vale` on `docs/` and `README.md`.

## 4. Vale configuration and UAR style rules

- [x] 4.1 Create `.vale.ini` pointing to the UAR style rule set.
- [x] 4.2 Add `.github/styles/UAR/` directory with rule files for terminology, inclusive language, and sentence structure.
- [x] 4.3 Add a `vale` devDependency to `website/package.json` so the lint script works without a global install.
- [x] 4.4 Add a `pnpm-lock.yaml` update via `pnpm install --frozen-lockfile` after modifying workspace deps.

## 5. Architecture decision records

- [x] 5.1 Add `docs/adr/0001-record-architecture-decisions.md` using the standard ADR template.
- [x] 5.2 Add `docs/adr/index.md` listing all ADRs.
- [x] 5.3 Add 10 ADRs covering the grade-A decisions:
  - 0002: Dual-license (AGPL runtime + MIT SDKs)
  - 0003: 60% coverage baseline
  - 0004: Central `UarError` enum
  - 0005: Config-rs with schemars and secrets
  - 0006: SLSA L3 + OSV/Grype supply-chain attestations
  - 0007: SDKs released under MIT with 1.0.0 versioning
  - 0008: RAG citation stream
  - 0009: A2UI vendor `@a2ui/web_core` and own renderer
  - 0010: A2UI renderer on webcore with React + shadcn/ui
  - 0011: Hosted docs portal with Docusaurus and visual regression

## 6. GitHub Pages workflow

- [x] 6.1 Create `.github/workflows/docs.yml` that triggers on pushes to `main`.
- [x] 6.2 Build the Docusaurus site with `pnpm docs:build`.
- [x] 6.3 Run `pnpm docs:lint` in the workflow.
- [x] 6.4 Add placeholder steps for rustdoc and typedoc generation.
- [x] 6.5 Deploy the `website/build` directory to the `gh-pages` branch using `actions/deploy-pages` or `peaceiris/actions-gh-pages`.

## 7. Verification

- [x] 7.1 Run `pnpm install --frozen-lockfile`.
- [x] 7.2 Run `pnpm docs:build` to confirm the Docusaurus site builds.
- [x] 7.3 Run `pnpm docs:lint` to confirm Vale runs (may report existing issues; fix new issues introduced by this change).
- [x] 7.4 Run `cargo check --locked --no-default-features --features server-full` if Rust files were touched (none expected).
- [x] 7.5 Validate `openspec validate docs-hosted-rustdoc-typedoc-docusaurus-ia`.

## 8. Remaining work

- [ ] 8.1 Operator: configure a custom domain in `website/docusaurus.config.ts` if desired (currently uses `prometheus-ags.github.io/universal-agent-runtime`).
- [ ] 8.2 Operator: enable GitHub Pages from the `gh-pages` branch once the workflow is merged.
- [ ] 8.3 Follow-up changes: populate each IA section with full content and generate rustdoc/typedoc/sphinx outputs end-to-end.
