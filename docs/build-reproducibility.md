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
unpublished dependency sources, all registry and Git crate sources selected by
`Cargo.lock`, Cargo's generated source replacements in `.cargo/config.toml`,
and license files. On a machine with no network access:

```bash
tar -xzf uar-offline-source.tar.gz -C uar-source
cd uar-source
cargo build --locked --offline --features minimal
```

The release audit scans the full extracted archive recursively, including
lockfiles in pinned nested checkouts. Keep these lockfiles current even when
their workspace is outside the Linux runtime build. The root JavaScript
workspace uses the pinned pnpm lockfile for development and container builds.

Create the archive from a clean candidate with all recursive submodules at
their recorded commits. Record the source commit, recursive pins, archive
digest, and the build and scanner results with the release evidence. Run
release qualification locally. The offline minimal build exercises the
server artifact, while the recursive scan also covers desktop and
standalone tools carried in the archive.

The Linux desktop source uses GTK3 with WebKitGTK 4.1. Its Rust lock selects
GTK3 0.19 and GLib 0.22 through the reviewed Tauri and WebKitGTK source overlays.
Build that desktop target with Rust 1.92 or newer and the existing GTK3 and
WebKitGTK 4.1 development libraries. Linux desktop behavior requires its own
verification evidence.

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

The candidate runtime-input pins for Liter, the curated Surreal Memory
source, and Skill System are recorded in `versions.toml`. The release input
provenance binds those pins to the Git tree and nested source. Ensure the
release's submodule commits are available from their declared remotes so
another maintainer can reproduce the checkout.

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
