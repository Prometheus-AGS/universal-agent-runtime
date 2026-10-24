# v1.3.0 publication standalone manifest repair

Phase: Execute, active waypoint `uar-post-lifecycle-verification-readiness`.
Base source provided by lead: `f870a9c337a89142e56bd8945b89c652c3c1385d`.
Outcome: assigned standalone-manifest implementation complete in the shared checkout.

## Owned files changed

- `vendor/git/prometheus-parking-lot-rs/Cargo.toml`: removed nested workspace declaration and workspace inheritance tables; materialized package metadata, all normal/dev/target dependencies, and Rust/Clippy lints directly into facade manifest. Existing features, optional dependencies, metadata, include list and coverage profile retained.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-core/Cargo.toml`: removed explicit package workspace owner; materialized metadata, dependencies, target/dev dependencies, and lints.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-runtime-tokio/Cargo.toml`: same; local core dependency now `../parking-lot-core`.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-pg/Cargo.toml`: same; local core normal/dev dependency now `../parking-lot-core`; SQLx default-features=false and feature list retained.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-embedded/Cargo.toml`: same; local core normal/dev dependency now `../parking-lot-core`; fault-injection self-dependency retained.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-iggy/Cargo.toml`: same; local core dependency now `../parking-lot-core`; exact Iggy requirements retained.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-wasm/Cargo.toml`: same; local core dependency now `../parking-lot-core`; target-specific conditions retained.
- `vendor/git/prometheus-parking-lot-rs/crates/parking-lot-ffi/Cargo.toml`: same; local core and embedded dependencies now sibling-relative; unpublished/library crate configuration retained.
- `vendor/git/README.md`: replaces unsuccessful explicit-owner adaptation notes with actual standalone adaptation and observed deployment failure rationale.

All package versions remain 0.3.0, edition 2024, minimum Rust 1.95. Inherited dependency features are merged with per-crate features rather than replacing them. Upstream Rust implementation files were not opened for mutation or written. No root Cargo manifest, lockfile, workflow, or other writer-owned file was edited.

## Source inspection and execution record

Commands were source inspection (`cat`, `rg --files`, `command -v python3.11`) and authorized implementation/documentation writing only. A Python 3.11 standard-library transformation read the original facade with `tomllib`, substituted its inherited declarations into all eight complete written manifests, and updated the notes. Output recorded each of the eight materialized manifests and the updated README. A subsequent source read displayed facade, PostgreSQL and Tokio manifests, including direct metadata, sibling core paths, explicit lints and preserved SQLx default features.

One initial source inspection referenced nonexistent `vendor/git/surreal-memory/Cargo.toml`; `cat` returned file-not-found. The corrected standalone precedent `vendor/git/surreal-memory-server/crates/surreal-memory/Cargo.toml` and sycophancy core manifest were then read. This was a source inspection error, not a verification result.

## Verification boundary and remaining work

Operator verification lock remains CLOSED for product tests and all unrelated phase gates. Lead packet states current user authorizes necessary publication source/build repairs and image publication; this teammate ran no Cargo/toolchain command, test, lint, formatter, compiler probe, metadata command, build, packaging command, or diff check. No acceptance credit is claimed.

Lead owns root workspace exclusion reconciliation, Cargo metadata/build commands, lock changes, commits, tags, and publication. The manifests have not been Cargo-loaded by this teammate. No added guards, speculative features or unrelated changes. No unresolved implementation decisions within assigned ownership. Review/build/publication results remain unverified here and must be recorded by lead from the authorized real path.
