# Offline and reproducible builds

UAR release builds perform no network access. The provider catalog and local
model assets are committed, versioned inputs; unpublished Rust dependencies are
vendored with provenance, while published dependencies are exact or lockfile
pinned crates.io releases.

The React admin application is built by the frontend workflow and its committed
`static/` output is packaged with UAR; Rust `build.rs` never installs Node
packages. PDF text extraction uses the offline-safe `lopdf` path, while
Kreuzberg handles the remaining enabled document formats without downloading
PDFium binaries.

## Build from an offline source archive

Create the release-grade archive while dependency registries are available:

```bash
scripts/package-offline-source.sh dist/uar-offline-source.tar.gz
```

The archive includes recursive-checkout source, committed catalog/model inputs,
unpublished dependency sources, all registry crate sources selected by
`Cargo.lock`, `.cargo/config.toml` source replacement, and license files. On a
machine with no network access:

```bash
tar -xzf uar-offline-source.tar.gz -C uar-source
cd uar-source
cargo build --locked --offline --features minimal
```

CI performs this build from two independent extractions. It remaps source paths,
sets `SOURCE_DATE_EPOCH`, and compares the declared release binary and catalog
digest byte-for-byte.

## Refresh reviewed inputs

Catalog refresh reads the checked-out `vendor/git/liter-llm` provider and model
schemas and is never invoked by `build.rs`. Update the pinned `models.dev` and
`liter-llm` submodules and the Cargo lockfile before refreshing:

```bash
node scripts/refresh-provider-catalog.mjs
git diff -- catalog/provider_catalog.json
shasum -a 256 catalog/provider_catalog.json
```

Update `catalog/SNAPSHOT.md` with the date, digest, entry count, and both source
commits only after reviewing the provider/model diff and running the bounded
catalog/routing checks required by the active phase.

The current reviewed runtime-input baseline is also recorded in
`versions.toml`: Liter 1.18.2 at
`c5c6caac617eb931cd5009146a70831422ec236c`, the curated Surreal Memory source
at `432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8`, and the Skill System parent at
`ad5c82c6c16145637c589a3ddfa06e0f20d603e7`. Each commit is reachable from its
authoritative remote `main` branch before UAR records it.

Local model refresh is similarly explicit:

```bash
scripts/refresh-local-model.sh
cargo test --lib --features local-models
```

Review the binary provenance, digest changes, and embedding behavior before
updating `catalog/SNAPSHOT.md`. Burn source generation, when needed for
maintainer analysis, remains separate:

```bash
cargo run --manifest-path tools/uar-model-builder/Cargo.toml
```

## Dependency policy

- New Git dependencies are prohibited in release manifests.
- Published dependencies use crates.io plus `Cargo.lock`.
- Unpublished dependencies must be copied under `vendor/git`, reduced to
  build-relevant source, and recorded in `vendor/git/README.md` with immutable
  commit, source URL, and license.
- `cargo build --locked --offline` is the release acceptance gate.
