# Verification — `fix-container-rust-toolchain-pin-consistency`

Date: 2026-08-22

Tested implementation SHA:
`64bea08695c341843e79e829ee0d4dfc0c210c3c`

These results are limited to the profile named in each row. The macOS ARM64
probe does not establish a Linux result. The Linux production-image result
does not establish any other runtime profile, target, provider, deployment, or
operational-resilience claim.

| requirement | command | observed output | limit | source SHA | profile |
|---|---|---|---|---|---|
| Docker and repository dated pins agree; backend selects the declared value. | `scripts/verify-runtime-image-toolchain-pin.sh Dockerfile rust-toolchain.toml nightly-2026-07-18` | `runtime-image toolchain pin consistent: docker=nightly-2026-07-18 repository=nightly-2026-07-18 effective=nightly-2026-07-18`; exit 0. | All three values must equal exactly; backend source must contain `cargo +"${RUST_TOOLCHAIN}" build`. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Source contract only. |
| Docker/repository mismatch fails closed. | Run the validator against `/tmp/uar-toolchain-pin-controls-20260822/Dockerfile.mismatch`, whose default is `nightly-2026-08-22`, plus repository `rust-toolchain.toml` and effective `nightly-2026-08-22`. | `Dockerfile RUST_TOOLCHAIN=nightly-2026-08-22 does not match repository channel=nightly-2026-07-18`; exit 1. | Must exit nonzero before Docker begins. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Source contract negative control only. |
| Floating backend nightly fails closed. | Run the validator against `/tmp/uar-toolchain-pin-controls-20260822/Dockerfile.floating`, whose backend command is `cargo +nightly build --release`, plus repository `rust-toolchain.toml` and effective `nightly-2026-07-18`. | `backend build selects the unqualified moving nightly channel`; exit 1. | Must exit nonzero before Docker begins and identify the floating selector. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Source contract negative control only. |
| Effective candidate build argument mismatch fails closed. | `scripts/verify-runtime-image-toolchain-pin.sh Dockerfile rust-toolchain.toml nightly-2026-08-22` | `effective RUST_TOOLCHAIN=nightly-2026-08-22 does not match repository channel=nightly-2026-07-18`; exit 1. | Must exit nonzero before Docker begins. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Source contract negative control only. |
| Dockerfile is valid for the pinned Linux ARM64 candidate command. | `docker buildx build --check --platform linux/arm64 --build-arg RUST_TOOLCHAIN=nightly-2026-07-18 -f Dockerfile .` | `Check complete, no warnings found.`; exit 0. | Docker check must exit 0 with the recorded effective channel and platform. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Dockerfile check, `linux/arm64`. |
| Repository channel accepts the locked ARM64 dependency. | Record `uname -m`, fixture hashes, and `rustc +nightly-2026-07-18 -Vv`; with a fresh external `CARGO_TARGET_DIR`, run `cargo +nightly-2026-07-18 check --locked --manifest-path openspec/changes/fix-container-rust-toolchain-pin-consistency/fixtures/diskann-wide-probe/Cargo.toml --target aarch64-apple-darwin`. | Host `arm64`; rustc `1.99.0-nightly (b6839f4d0 2026-07-17)`; `diskann-wide v0.54.0` and the probe compiled; exit 0 in 12.98s. Fixture hashes before/after: manifest `d3bb4591e0ff01aed64baefa3a7f306fe2ed470f03f7364d9a75d438fe4cbf5f`, lock `a8009d91b3100b85708c89e032c7a330dd8bd64bd674c732dcec0c692a184f20`, source `6b3c4600b724a2c490a63cc7e7d0403e54554a3f75431d20edb7f7652b67e296`; fixture status empty. | Exact dated channel, exact locked fixture, exact target, unchanged hashes, and exit 0. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Cargo probe, `aarch64-apple-darwin`. |
| The observed incompatible channel is the failing negative control. | Record `rustc +nightly-2026-08-22 -Vv`; with a different fresh external `CARGO_TARGET_DIR`, run `cargo +nightly-2026-08-22 check --locked --manifest-path openspec/changes/fix-container-rust-toolchain-pin-consistency/fixtures/diskann-wide-probe/Cargo.toml --target aarch64-apple-darwin`. | rustc `1.100.0-nightly (c656540d6 2026-08-21)`; exactly three E0283 diagnostics at `diskann-wide` ARM64 lines 348, 363, and 379; exit 101. The three fixture hashes matched the positive run and fixture status remained empty. | Exact incompatible dated channel, identical locked inputs/target, exactly three E0283 diagnostics, and nonzero exit. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Cargo probe negative control, `aarch64-apple-darwin`. |
| Complete production image compiles from a clean implementation commit. | From the clean detached checkout: `docker build --platform linux/arm64 --build-arg RUST_TOOLCHAIN=nightly-2026-07-18 -t uar:toolchain-pin-64bea086 .` | Toolchain install selected `nightly-2026-07-18`; `diskann-wide v0.54.0` compiled; Rust release finished in 14m21s with the three pre-existing warnings; exit 0. Image `sha256:07a9dca99e084bbe132855a196e51ff443ae18273ce04a1e6821c00d92c77b4f`, Linux ARM64, 5,142,539,787 bytes. Detached status remained empty. | Exact clean implementation SHA, `linux/arm64`, explicit dated build argument, complete image export, and exit 0. | `64bea08695c341843e79e829ee0d4dfc0c210c3c` | Production Docker image, `linux/arm64`, backend features `server-full,postgres-backend`. |
| Evidence-only handoff remains non-circular. | `git rev-parse HEAD^`; classify every path from `git show --format= --name-only HEAD` against the evidence-only allowlist. | Parent printed `64bea08695c341843e79e829ee0d4dfc0c210c3c`; 31 evidence paths classified with no product or build path; exit 0. Canonical KBD resolves the final commit SHA externally after the commit exists. | Evidence commit must have tested implementation SHA as its parent; parent must rebuild the resolved handoff SHA and restart the 10,800-second certification from zero. | Tested source: `64bea08695c341843e79e829ee0d4dfc0c210c3c`; final handoff SHA intentionally resolved outside this self-referential file. | KBD source binding only; parent certification not run. |

## Local gates

- `bash -n scripts/verify-runtime-image-toolchain-pin.sh` exited 0.
- Workspace `Cargo.toml`, root `Cargo.lock`, `rust-toolchain.toml`, and
  `.github/**` had no diff. The `Dockerfile` diff was exactly one removed and
  one added selector line.
- `openspec validate fix-container-rust-toolchain-pin-consistency --strict`
  reported `Change 'fix-container-rust-toolchain-pin-consistency' is valid` and
  exited 0 before Execute and at completion.
- The contained artifact-refiner gate reported schema PASS for its manifest and
  constraints, file PASS for its one non-empty referenced receipt, constraint
  PASS for its five blocking constraints at their stated limits, and consistency
  PASS for its iteration log, decision, and converged state.
- The artifact-refiner workflow dispatcher failed twice before trigger
  evaluation because its quoted heredoc passed a literal `$EVENT_PAYLOAD` to
  `json.loads`. This artifact has no workflow triggers, so no configured action
  or validation was skipped; the imported skill was not patched in this child.
- Global `git diff --check` reports pre-existing trailing blank lines in six
  unrelated KBD task projections. The child-permitted scoped diff check exits 0,
  and those unrelated files are excluded from this evidence.

The parent 10,800-second certification has not run on the final handoff SHA and
must restart from zero after this child.
