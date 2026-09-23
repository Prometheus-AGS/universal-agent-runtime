## Why

the-boss will ship `uar-sidecar` inside its installer for six platforms and pin each release by version and checksum (the-boss decision D4, revision 3). UAR has no way to produce those binaries. `Cargo.toml` has no `[profile.release]` (only `[profile.dev]`, `Cargo.toml:558-560`), and `.github/workflows/` holds only `deploy.yml` and `docs.yml`. The policy validator allows exactly those two workflows (`scripts/validate-github-actions-policy.mjs:10-13`), so any new workflow fails it today. `docs/release-verification.md:8-9` says GitHub Actions are not a build runner for releases; D4 makes one exception for tag-triggered, test-disabled sidecar builds. That exception must be written into the policy, not assumed.

## What Changes

- Add `[profile.release]` with compass's `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`.
- **Deviation from D4, decided 2026-09-23 (task 0.2):** D4 also copies `panic = "abort"`. UAR contains panics at run, tool and sandbox boundaries with `catch_unwind` — for example the run finalizer that drains sandboxes and terminals after a panic (`src/uar/runtime/manager.rs:5482-5500`), sandbox creation and execution (`src/sandbox/execution.rs:476`, `:509`, `:539`), graph tools (`src/uar/runtime/graph/tools.rs:126`, `:215`, `:234`), graph hosts (`src/uar/runtime/thread/graph_host.rs:143`) and terminal processes (`src/uar/tools/terminal_process.rs:344`). Under `panic = "abort"` every one of those becomes a sidecar crash that skips cleanup. The release profile keeps `panic = "unwind"`. This is the accepted design (technical decision by the coordinating session, 2026-09-23), not an open question. The cost: a panic outside those boundaries still unwinds rather than aborting, and the binary carries unwind tables.
- Add `.github/workflows/sidecar-release.yml`, modeled on compass's `compass-release.yml` but deployment-only: it runs only when a `uar-sidecar-v*` tag is pushed; builds `uar-sidecar` on six native runners (`macos-15-intel`, `macos-15`, `ubuntu-24.04`, `ubuntu-24.04-arm`, `windows-2025`, `windows-11-arm`) with `--no-default-features --features minimal,local-models,document-intelligence,wasm-runtime` and a `cargo build` command that compiles no tests; packages one archive per target with the binary, the on-device embedding model assets and the license files, plus a `.sha256` per archive; and publishes them to a GitHub Release whose notes record the binary and archive size per platform. Compass's `cargo test`, script-test and qualification steps and its run-the-binary verification are not copied.
- Extend the policy validator's allowlist with `sidecar-release.yml` and markers that enforce the above (operator decision 2026-09-23: approved, task 0.3). AGENTS.md forbids weakening the validator; this adds one allowed workflow with stricter per-workflow checks and loosens no existing rule.
- State the exception in `docs/release-verification.md` — tag-triggered, test-disabled release builds for the sidecar are deployment, not development verification — and add local pre-tag build guidance that runs the same build command the workflow runs.

Not in scope: signing, attestation, SBOMs (the existing local evidence flow in `docs/release-verification.md` keeps them), installers, the-boss's manifest, and any claim that a built binary works on a platform nobody ran it on.

## Capabilities

### New Capabilities
- `sidecar-release-distribution`: the release profile, the tag-only deployment workflow, the six targets and feature set, archive contents and checksums, recorded sizes, and the local pre-tag build.

### Modified Capabilities
(none — `modular-release-capabilities` and `offline-reproducible-build` keep their requirements; the sidecar feature set is an additive product of existing features.)

## Impact

- **Code/config:** `Cargo.toml` (release profile), new `.github/workflows/sidecar-release.yml`, `scripts/validate-github-actions-policy.mjs` (allowlist and markers), new `scripts/validate-sidecar-release.mjs` and `scripts/test-sidecar-release.mjs`, `docs/release-verification.md`.
- **Every release build changes:** the profile also applies to `cargo build --release` in `Dockerfile:225` and to local release builds — slower links, more link-time memory.
- **Provider compatibility, runtime UX, realtime state:** unchanged at runtime; the binary is the same program.
- **Native-dependency risk (unverified on the six runners):** `aws-lc-sys` 0.41.0, `onig_sys` 69.9.3, `esaxx-rs` 0.1.10, Tesseract via `kreuzberg-tesseract` 4.10.2 (kreuzberg `ocr`), and ONNX Runtime fetched by fastembed's `ort-download-binaries` (`ort-sys` 2.0.0-rc.13) — notably on `aarch64-pc-windows-msvc`. Versions are from `Cargo.lock`; whether each is in the sidecar's feature closure is not yet checked.
- **KBD workflow state:** YES. D4 falsifier 1 is tracked per failing target as recorded working days in the task log of child phase `the-boss-universal-agent-runtime`; a failing Windows target reopens D4.
