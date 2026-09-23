## Context

See `proposal.md` for why. Facts that shape the design:

- Reference: `/Users/gqadonis/Projects/references/compass/.github/workflows/compass-release.yml` — `metadata` job (tag equals workspace version), a qualification job, a six-target `build` matrix on native runners with `fail-fast: false`, per-archive `.sha256`, and a `release` job gated on the tag that publishes with `softprops/action-gh-release` pinned by commit SHA. Compass's `[profile.release]` (`compass/Cargo.toml:143-147`): `codegen-units = 1`, `lto = "thin"`, `panic = "abort"`, `strip = "symbols"`.
- UAR's AGENTS.md "GitHub Actions policy": deployment-only; builds must use explicitly test-disabled commands; run `pnpm github-actions-policy:validate` before and after editing workflows or the scripts they call; never weaken the validator.
- The validator (`scripts/validate-github-actions-policy.mjs`) allowlists workflow file names with required markers (`:10-13`) and bans `cargo test|check|clippy|fmt|bench|audit|mutants` and JS test commands (`:15-20`). It passed on this worktree on 2026-09-23 (run directly with `node`, see tasks 0.4).
- `uar-sidecar` requires feature `server` (`Cargo.toml:38-41`); `minimal = ["server", "surreal-backend"]` (`:145`).
- `rust-toolchain.toml` pins `nightly-2026-07-18` and lists iOS, Android and wasm targets; rustup installs them on each runner.
- Path dependencies live in git submodules (`Cargo.toml:263`, `:341`, `:481`, `:504`); `deploy.yml` checks out with `submodules: true` and runs `scripts/update-submodules.sh` (`deploy.yml:50-57`).
- `FastEmbedBackend` loads model assets from disk and never downloads (`src/uar/rag/embeddings/fastembed.rs:11-17`, `:49-70`); the assets are `src/uar/runtime/matching/models/` (33 MB).

## Goals / Non-Goals

**Goals:** reproducible tag-to-release for six targets; nothing in Actions that is not build, package or publish; sizes recorded; breakage found locally first.

**Non-Goals:** signing, attestation, SBOMs, installers, universal macOS binaries, cross-compilation, running the binary in Actions, a size limit.

## Decisions

### D1 — Keep unwinding (deviation from D4; decided)
Add `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`; leave `panic` at its default, `unwind`. UAR's run finalizer, sandbox, graph-tool and terminal paths rely on `catch_unwind` (sites listed in `proposal.md`); `panic = "abort"` turns a contained tool panic into a sidecar crash that leaves sandboxes and terminals undrained. *Alternative:* copy compass exactly. Rejected: compass is a CLI with no in-process isolation to lose.
*Decision 2026-09-23 (technical decision by the coordinating session):* `panic = "unwind"` is the accepted design. D4's `panic = "abort"` is not adopted for UAR. This is closed; reopening it requires removing the `catch_unwind` cleanup paths first.

### D2 — Trigger and version gate
`on: push: tags: ["uar-sidecar-v*"]` and nothing else — no `workflow_dispatch` (compass has one; D4 says tag only). A `metadata` job compares `GITHUB_REF_NAME` with `uar-sidecar-v` + the `universal-agent-runtime` package version from `cargo metadata --no-deps`. A prerelease tag therefore needs a prerelease package version; tags containing `-` are published with `prerelease: true`. The `uar-sidecar-v` prefix avoids the existing `v*` product tags (`docs/release-verification.md:21-22`).

### D3 — Build step
Per matrix entry: checkout with submodules and `scripts/update-submodules.sh` (as `deploy.yml`); toolchain from `rust-toolchain.toml` plus the matrix target; `Swatinem/rust-cache` keyed per target; then
`cargo build --release --locked --bin uar-sidecar --no-default-features --features minimal,local-models,document-intelligence,wasm-runtime --target <target>`.
`cargo build` of one `--bin` compiles no test targets. No other cargo subcommand runs except `cargo metadata` in D2. Compass's `cargo test`, `test_release_scripts.sh` and `code-graph-qualification` are not copied.

### D4 — Packaging without running
A Node packaging script (runners all have Node) writes `uar-sidecar-<target>/` with the binary, `models/` (the five asset files), `LICENSE`, `LICENSE-CC-BY-4.0.md`, then `.tar.gz` (Unix) or `.zip` (Windows), the `.sha256`, and a `size-<target>.json` with binary and archive bytes. Verification in Actions is limited to recomputing the checksum; compass's `--version`/`--help` step executes the product and is dropped. Whether ONNX Runtime is linked statically or needs a shared library beside the binary is unknown; the local pre-tag build (D6) checks it and, if needed, the library joins the archive.

### D5 — Publish
A `release` job, `needs: build`, `permissions: contents: write` (other jobs `contents: read`), downloads all artifacts, renders a size table from the six `size-*.json` files into the notes body, and calls `softprops/action-gh-release` pinned by SHA with `generate_release_notes: true`. Because `needs: build` requires every matrix leg, one failed target publishes nothing. Dropping a target (D4 falsifier 1) is an explicit matrix edit recorded in the notes.

### D6 — Local pre-tag build
`docs/release-verification.md` gains a section with the exact D3 command for the maintainer's host target, the packaging command, and a check of the binary's dynamic library dependencies (`otool -L`, `ldd`, `dumpbin /dependents`). The result is recorded with a date before tagging. This is where the binary is run, locally.

### D7 — Validator extension (operator decision 2026-09-23: approved)
Add `sidecar-release.yml` to `allowedWorkflows` with markers (`uar-sidecar-v`, `--bin uar-sidecar`, the exact feature string, `softprops/action-gh-release@`, `sha256`) and a per-workflow prohibition list: `branches:`, `pull_request`, `workflow_dispatch`, `schedule:`, any execution of the built `uar-sidecar` binary, and `--features` values other than the exact string. Existing checks stay unchanged.
`docs/release-verification.md` gains one exception sentence beside its rule that GitHub Actions are not a release build runner: tag-triggered, test-disabled release builds for the sidecar are deployment, not development verification. The operator approved both the allowlist entry and the sentence on 2026-09-23 (task 0.3).

## Risks / Trade-offs

- [Native C/C++ dependencies on some runners] → `aws-lc-sys`, `onig_sys`, `esaxx-rs`, Tesseract via `kreuzberg-tesseract`, and ONNX Runtime prebuilt download for `aarch64-pc-windows-msvc`. D4 falsifier 1: up to 4 recorded working days of fixes per failing target; then the target is dropped and reported; a failing Windows target reopens D4.
- [Built but never run on four of six platforms] → The workflow cannot run the binary. "Working" on a target is proved only where someone runs it (maintainer host, the-boss integration). Report that plainly.
- [Nightly toolchain and extra targets] → Longer setup per runner; nightly availability for the `aarch64-pc-windows-msvc` host is unverified.
- [Profile applies to all release builds] → The Docker image build (`Dockerfile:225`) gets slower and needs more link memory.
- [Archive size] → The model assets add about 33 MB per archive; recorded, not limited.
- [Submodule access] → Private SSH submodules need `SUBMODULES_TOKEN`, as in `deploy.yml`.

## Migration Plan

Merge the workflow and validator change; build locally (D6); the operator pushes a prerelease tag and checks the six archives and checksums; the operator pushes the release tag. Rollback: delete the release and tag (operator action) and revert the workflow. No runtime data is affected.

## Open Questions

- Should the release also publish a single manifest file (compass publishes `compass-release.json` last) so the-boss can read version and checksums in one fetch? Deferrable; the per-archive `.sha256` already meets D4.
