# API Reference

UAR publishes generated API references alongside the narrative documentation.

## Hosted references

- [Rust API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/rust) — rustdoc for the runtime and Rust SDK
- [TypeScript API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript) — TypeDoc for the TypeScript SDK

The Python SDK is documented in the narrative SDK guide. UAR does not advertise
a hosted Python API reference because the repository does not yet contain a
pinned generator and staged artifact for one.

## Generating locally

```bash
# Rust (runtime + SDK)
cargo doc --locked --no-deps --workspace --features server-full

# TypeScript SDK
npm --prefix sdks/typescript ci
npm --prefix sdks/typescript run docs

# Narrative portal and assembled reference artifact
npm --prefix website ci
npm --prefix website run build
node scripts/stage-documentation-references.mjs
```

The deployment workflow in `.github/workflows/docs.yml` runs these same
generation and staging commands before it uploads the one Pages artifact.

## Note on paths

The paths above (`/docs/api/rust` and `/docs/api/typescript`) are relative to the
deployed Docusaurus project site. Locally assembled output lives beneath
`website/build/docs/api/`.
