ASSESSMENT: uar-1-0-readiness/jsonwebtoken-crypto-provider-spike
Project: universal-agent-runtime
Date: 2026-08-13
Codebase baseline: `codex/uar-1-0-readiness` at `a5978c03`, equal to current `origin/main`, with no implementation changes.
Cross-tool progress: none

IMPLEMENTATION STATUS
- `jsonwebtoken` version: [DONE] — direct dependency resolves to 11.0.0, the current upstream release observed on 2026-08-13; a separate 10.4.0 copy is transitively owned by SurrealDB.
- Provider selection: [MISSING] — direct `jsonwebtoken = "11.0.0"` enables default `use_pem` only. Neither `rust_crypto` nor `aws_lc_rs` is active for 11.0.0.
- Provider installation: [MISSING] — no runtime source calls `CryptoProvider::install_default`; `CryptoProvider::from_crate_features` in `/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jsonwebtoken-11.0.0/src/crypto/mod.rs` therefore returns the fallback provider whose operations panic.
- Runtime verification path: [PARTIAL] — signing and verification call the correct library APIs, but both panic when first resolving the absent provider.
- Comparative decision evidence: [PARTIAL] — dependency, platform, advisory, upstream-maintenance, and measured baseline behavior facts are collected, but comparison and the binding verdict belong to analysis.

CROSS-TOOL PROGRESS
- NONE — no child implementation is owned by another tool.

SPEC GAP SUMMARY
- The A0 proposal inferred that RustCrypto adds no dependency cost because its crates occur in `Cargo.lock`. That does not establish build-graph activation. Under `server-full`, `aws-lc-rs 1.17.0` is active today, while `p256 0.13.2`, `p384 0.13.1`, `rsa 0.9.10`, and `ed25519-dalek 2.2.0` print no reverse dependency and would be newly activated by `rust_crypto`.
- Both providers implement every algorithm exposed by jsonwebtoken 11, so algorithm coverage does not decide the choice.
- The release matrix is Linux x86-64/aarch64, macOS x86-64/aarch64, and Windows x86-64. AWS-LC documents builds/tests for all five at https://aws.github.io/aws-lc-rs/platform_support.html; RustCrypto's portability advantage matters mainly for `no_std` and bare WebAssembly, neither of which is part of the certified `server-full` profile.
- `jsonwebtoken` documents that at most one built-in backend may be enabled. Enabling both is invalid as an automatic-selection strategy.
- Security evidence is not symmetric in kind. RustCrypto avoids C/FFI code, but enabling its backend activates `rsa 0.9.10`, whose Marvin timing advisory has no patched version (https://rustsec.org/advisories/RUSTSEC-2023-0071.html). AWS-LC adds C/assembly and FFI attack surface; its March 2026 `aws-lc-sys` advisories are patched at or before 0.38.0, while this graph pins 0.41.0 (for example https://rustsec.org/advisories/RUSTSEC-2026-0046.html). Selecting ordinary `aws_lc_rs` is non-FIPS and creates no FIPS certification claim.
- Measured evidence currently covers the actual provider-disabled failure and active dependency topology. No comparative performance benchmark has run. Performance is not a decision input in this spike because a representative release benchmark is forbidden during implementation by `.claude/rules/rust.md`; the binding decision must not claim a speed advantage for either backend.

BUILD HEALTH
- build check: [PASS] — `cargo check --locked --no-default-features --features server-full` exited 0 after 14m04s. It emitted three existing warnings (`MAX_BODY_BYTES`, `MAX_REDIRECTS`, and missing `Debug` for `WasmHostState`) plus the existing future-incompatibility notice for `nix 0.31.3` and `redis 1.2.1`.
- known violation: [OBSERVED] — `cargo test --locked --no-default-features --features server-full --lib test_resolve_user_context -- --nocapture` ran four focused tests: 3 passed and `test_resolve_user_context_valid_token` failed. The panic at `jsonwebtoken-11.0.0/src/crypto/mod.rs:124` states that it could not determine a process-level provider and requires exactly one of `rust_crypto` or `aws_lc_rs` (or manual installation).
- test coverage: [PARTIAL] — the unchanged middleware tests exercise the real HS256 path and now supply the provider-disabled negative control. Parent A0 still needs an explicit round-trip and wrong-secret rejection after selecting a provider.

CONSTRAINT CHECK
- AGENTS.md violations: NONE in child-owned files.
- constraints.md violations: N/A — file absent.
- scope compliance: PASS — the child denies writes to `Cargo.toml`, `Cargo.lock`, `src/**`, and `tests/**`; A0 retains implementation ownership.
- tier discipline: PASS — no Tier 2 command has run.

GOAL PROGRESS
- Compare both providers: [PARTIAL] — official/current and repository facts collected; analysis remains.
- Record one binding decision: [NOT MET] — intentionally deferred to analysis.
- Return exact parent configuration and checks: [NOT MET] — depends on the decision.

UNCOMFORTABLE FACT
- Choosing RustCrypto for a pure-Rust preference would expand the active authentication graph and activate the `rsa` crate. The current RustSec advisory for its Marvin timing attack states that no patched version exists. UAR's immediate JWKS use verifies with public keys rather than decrypting, but standardizing the runtime provider also standardizes future signing capability; this cannot be dismissed as irrelevant without narrowing the supported algorithm contract.

ASSESSMENT COMPLETE
