## Context

See `proposal.md` for the observed candidate-build failure. The production
Dockerfile already declares `RUST_TOOLCHAIN=nightly-2026-07-18`, consumes that
argument in its `toolchain` stage, and installs the declared channel. Its
backend builder derives from that stage, so Docker's documented ARG scoping
rules make `RUST_TOOLCHAIN` available there. The defect is limited to the
backend command overriding that inherited value with the moving `nightly`
alias.

The repository channel, dependency graph, runtime behavior, and deployment
configuration are immutable inputs to this release child. Verification must be
local and source-bound because GitHub Actions are reserved for deployment.

## Goals / Non-Goals

**Goals:**

- Bind the backend image compilation to the dated channel already installed by
  the image.
- Make a mismatch or floating backend selector fail a cheap local contract
  check before an expensive image build.
- Preserve both sides of the observed compiler-compatibility control and prove
  the complete production image from the replacement source commit.

**Non-Goals:**

- Selecting a newer Rust channel or changing any crate version or feature.
- Reworking Docker stages, build caching, image contents, or deployment
  workflows.
- Reconciling the older, broader `gke-deployment` specification in this release
  child.

## Decisions

### Select the inherited Docker argument explicitly

The backend command will use `cargo +"${RUST_TOOLCHAIN}"` rather than
`cargo +nightly` or plain `cargo`. The explicit selector makes the compilation
consume the same value that the toolchain stage installs and keeps the
Dockerfile's declared pin visible at the point of use. Docker documents that an
ARG consumed in a parent stage is inherited by stages based on that parent, so
the existing `toolchain` to `builder` relationship supplies the value.

Plain `cargo` was considered. It would currently select the installed default,
but it weakens the source-level contract: later changes to the default or build
context could silently separate installation from selection. Redeclaring a
second literal in the builder was rejected because duplicate pins can drift.

### Validate the relationship with one narrow shell contract

`scripts/verify-runtime-image-toolchain-pin.sh` will accept optional Dockerfile,
`rust-toolchain.toml`, and effective-channel inputs, defaulting to the
repository files and Dockerfile default. It will:

1. read the production Dockerfile's `RUST_TOOLCHAIN` default;
2. read the repository's TOML `channel` value;
3. fail unless both non-empty values are equal;
4. fail unless the effective candidate channel equals the repository channel;
5. require the backend compilation to select `${RUST_TOOLCHAIN}` explicitly;
6. reject an unqualified `cargo +nightly` selector.

Optional paths make negative controls possible against temporary mutated
copies without editing the repository. The complete candidate command will
pass the same repository channel to the check and to Docker with
`--build-arg RUST_TOOLCHAIN=<dated-channel>`, recording the effective value.
A Bash script was chosen because the contract is a small source-text invariant
and introducing a parser dependency would exceed this child. A Docker-only
build check was rejected because it detects the failure only after costly
downloads and compilation.

### Use layered evidence rather than substituting one check for another

The contract script proves source and effective-argument consistency, the
isolated locked-dependency pair proves the compiler-compatibility boundary,
and a complete local production-image build proves the actual artifact. None
substitutes for the others. Both isolated checks run on the same ARM64 host and
record `rustc -Vv`; `nightly-2026-08-22` is the immutable failing control, while
`nightly-2026-07-18` must pass the same locked `diskann-wide 0.54.0` check. The
complete image command explicitly targets `linux/arm64`.

The isolated pair uses a committed fixture at
`fixtures/diskann-wide-probe/`: a minimal non-published Cargo package whose
only dependency is `diskann-wide = "=0.54.0"`, its generated `Cargo.lock`, and
an empty library source. The fixture is generated once with the repository
channel, then committed before either control. Both checks use `--locked`, the
same `aarch64-apple-darwin` target, separate target directories, and the same
recorded SHA-256 hashes for the manifest, lockfile, and source. A clean
fixture-scoped git status before and after both runs proves neither toolchain
rewrote the inputs.

An ephemeral manifest was rejected because its content and resolved lock graph
could not be independently reproduced from the evidence. Adding this fixture
to the workspace was also rejected; it remains an isolated OpenSpec
verification artifact and cannot affect workspace dependency resolution.

The uncomfortable case is that the one-line repair exposes a different image
failure after Rust compilation begins. That is not evidence this contract is
wrong, but it is outside the observed defect; execution stops instead of
silently widening this child.

## Risks / Trade-offs

- [The inherited argument is absent or empty in the builder] -> The explicit
  selector fails the image build, while the contract and full build prevent
  candidate acceptance.
- [Text validation becomes stale after a legitimate Docker command rewrite] ->
  The script fails closed and must be updated in the same reviewed change as
  the Dockerfile.
- [A caller supplies a mismatched Docker build argument] -> The source-bound
  candidate command passes and records the repository channel explicitly, and
  the contract's effective-channel input fails before Docker when it differs.
- [The older `gke-deployment` capability remains inconsistent with the current
  multi-stage image] -> Record the conflict for later reconciliation and do not
  expand a release-blocking defect fix into a broad specification rewrite.
- [A full local build consumes substantial disk and time] -> Retain useful
  BuildKit cache, check available storage before the run, and treat only the
  complete source-bound image build as candidate evidence.

## Migration Plan

1. Add and exercise the contract check against unchanged inputs and mutated
   negative-control copies.
2. Replace the single floating backend selector with the inherited dated
   selector and rerun the contract.
3. Commit the implementation and planning artifacts, then reproduce the
   dated-versus-incompatible compiler control and build the complete production
   image from a clean detached checkout of that exact implementation commit.
4. Add observed results in one direct evidence-only commit naming the tested
   implementation commit. The evidence commit changes no product/build input.
5. Hand the evidence commit to the parent, whose certification rebuilds that
   exact final handoff commit and restarts the 10,800-second soak from zero.

This two-commit sequence avoids the impossible requirement for a tracked file
to contain its own commit SHA. Rollback is the single Dockerfile selector
reversal plus removal of the local contract script and change artifacts. That
rollback restores the observed moving-nightly failure and therefore cannot be
promoted as a candidate.
