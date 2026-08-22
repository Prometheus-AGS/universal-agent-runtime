## Why

The production Dockerfile declares and installs `nightly-2026-07-18`, but its
backend build invokes floating `cargo +nightly`. A clean Linux ARM64 image
build therefore resolved that alias to `nightly-2026-08-22`, failed compiling locked
`diskann-wide 0.54.0`, and blocked the immutable release candidate before any
operational-resilience assertion could run.

## What Changes

- Make the production backend build explicitly select the existing dated
  `RUST_TOOLCHAIN` value instead of the moving `nightly` alias.
- Add a local contract check that requires the Dockerfile pin to match
  `rust-toolchain.toml`, requires the backend build to select that pin, and
  rejects an unqualified floating-nightly selector.
- Preserve `nightly-2026-08-22` and its observed ARM64 failure as the immutable
  negative control using one committed locked probe fixture, record the
  effective candidate build argument, and require the dated-pin control plus a
  complete local production-image build to pass.
- Keep the dated channel and all dependency, feature, public API, deployment,
  and GitHub Actions surfaces unchanged.

## Capabilities

### New Capabilities

- `runtime-image-toolchain-consistency`: Defines how the production image
  declares, installs, selects, and locally verifies one dated Rust toolchain.

### Modified Capabilities

None.

## Impact

- Build surface: one backend selector in `Dockerfile`, one narrow local
  verification script under `scripts/`, and one locked dependency probe under
  this OpenSpec change's verification fixtures.
- Dependencies: no workspace `Cargo.toml`, root `Cargo.lock`, Rust channel, or
  crate version change; the isolated fixture pins the already-locked failing
  crate only for verification.
- Runtime UX: none; this changes build reproducibility, not served behavior.
- Provider compatibility: none; provider configuration and feature selection
  remain unchanged.
- Realtime state: none; no event, persistence, or protocol schema changes.
- Public APIs and deployment: none; no runtime API, manifest, or GitHub Actions
  change.
- KBD: build a clean implementation commit, record that result in a direct
  evidence-only commit, then return the parent to an exact-handoff local
  certification restart from zero.
