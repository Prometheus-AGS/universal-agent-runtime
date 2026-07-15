# API Reference

UAR publishes generated API references alongside the narrative documentation.

## Hosted references

- [Rust API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/rust) — rustdoc for the runtime and Rust SDK
- [TypeScript API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript) — TypeDoc for the TypeScript SDK
- Python API reference — Sphinx autodoc (published alongside the site)

## Generating locally

```bash
# Rust (runtime + SDK)
cargo doc --no-deps --workspace --features server-full

# TypeScript SDK
pnpm -C sdks/typescript typedoc

# Python SDK
pnpm -C sdks/typescript build:docs
# or directly: sphinx-build sdks/python/docs _build/python
```

The CI workflow in `.github/workflows/docs.yml` builds these references and copies them into the Docusaurus `build/` directory before deploying to GitHub Pages.

## Note on paths

The paths above (`/docs/api/rust` and `/docs/api/typescript`) are relative to the deployed Docusaurus site. When running locally, the generated HTML is served from the Docusaurus static directory.
