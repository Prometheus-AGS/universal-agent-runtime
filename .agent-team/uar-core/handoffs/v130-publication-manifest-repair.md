# v1.3.0 publication manifest repair — 2026-10-24

## Observed failure and source cause

Lead-reported deployment run `40087378392`, ARM64 job `120852969939`, source
`3d9253fc`, tag `v1.3.0`, failed loading the vendored Parking Lot core manifest:
`error inheriting edition from workspace root workspace.package.edition undefined`.
This handoff does not claim the deployment log was independently fetched by this writer.

Source inspection found UAR excludes the vendored facade workspace root but also
has a direct path dependency on its nested `prometheus-parking-lot-core` package.
All seven nested packages inherit package metadata, dependencies and lints without
an explicit package workspace pointer. UAR has package edition `2024` but no
`workspace.package.edition`; the vendored workspace defines that inherited field.
The reported failure is consistent with nested core being treated as a UAR member
instead of using the vendored workspace for inheritance.

## Authorized repair

Added exactly `workspace = "../.."` immediately below `[package]` in:

- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-core/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-runtime-tokio/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-pg/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-embedded/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-iggy/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-wasm/Cargo.toml`
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-ffi/Cargo.toml`

Updated `vendor/git/README.md` to document this adaptation and replace the previous
claim that nested manifests are byte-identical. Upstream Rust implementation source
was not edited. The pre-existing root vendored manifest adaptation omitting dev-only
bench/example targets remains. No root Cargo manifest, lockfile, scripts, workflow,
tag or publication state was changed by this writer.

## Local documentation basis

Inspected installed Cargo 1.95 documentation (without invoking Cargo):
`/Users/gqadonis/.rustup/toolchains/1.95-x86_64-apple-darwin/share/doc/rust/html/cargo/reference/manifest.html`,
section `the-workspace-field`, documents `[package] workspace = "path/to/workspace/root"`.
`reference/workspaces.html`, member/exclusion discussion, states the key overrides
automatic workspace search and explains automatic membership of path dependencies.
Existing local sycophancy member manifests also use `workspace = "./../.."`.

## Command classification and unverified risk

Commands were source inspection (`rg`, `cat`, `sed`, `ls` of source/docs) or authorized
implementation/documentation writing (Python targeted edits and this handoff). The
write command reported:
`Authored explicit vendored workspace ownership in seven nested manifests and updated provenance.`

Verification remains **CLOSED**. No Cargo command, tests, checks, compiler probes,
formatters, lint, builds, packaging validation, diff checks or review gates ran.
No commit, push, tag mutation or workflow rerun was performed. No guards or runtime
fallbacks were introduced; the manifest pointer directly addresses the observed
inheritance failure.

The patch has not proved resolution, lockfile completeness, successful manifest
loading, compilation or deployment. The uncomfortable risk is that fixing this first
manifest-loading failure may expose a separate dependency-resolution or compiler
failure; the original publication attempt did not reach those stages. The lead must
present the new inventory and obtain any required publication/tag amendment approval
because public `v1.3.0` already exists. Earlier acceptance or publication authorization
does not establish verification for this new source edit.
