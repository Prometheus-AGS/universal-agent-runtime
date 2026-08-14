# Decision — standard JWT provider

> **SUPERSEDED 2026-08-13 by JWT-CRYPTO-002.** This document preserves the
> original server-full-only AWS-LC decision as historical evidence. The active
> decision is RustCrypto; see `re-evaluation-2026-08-13.md` and `handoff-out.md`.

## Decision

For UAR's `server-full` profile, retain the direct `jsonwebtoken` version requirement at `11.0.0`, keep the lock at 11.0.0, and standardize on the built-in `aws_lc_rs` feature:

```toml
jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }
```

The parent A0 change owns implementation. The spike does not alter `Cargo.toml`, `Cargo.lock`, runtime source, or tests.

## Decisive evidence

- Read-only Cargo feature simulation succeeded with `--locked` and left `Cargo.toml` and `Cargo.lock` unchanged.
- Baseline and `jsonwebtoken/aws_lc_rs` each resolve 918 active normal/build packages under `server-full`; `jsonwebtoken/rust_crypto` resolves 940. The 22-package RustCrypto set difference and exact commands are recorded in `research-evidence.md`.
- Exact pinned source inspection found the same JWT algorithm branches in both built-ins; `research-evidence.md` records the command and set. AWS-LC documents upstream build/test support for the native release targets, but this spike built UAR only on its current aarch64 macOS host.
- The RustCrypto RSA advisory is secondary: repository search found no current RS*/PS* private-key signing path.

## Assumptions

- The supported result is limited to the `server-full` feature profile on the current aarch64 macOS host. Upstream AWS-LC platform documentation is compatibility evidence, not a UAR cross-target build verdict.
- UAR has no present `no_std`, bare-WASM, C/C++-free, HSM/KMS, or FIPS certification requirement for this dependency.
- No performance advantage is assumed for AWS-LC or RustCrypto.
- The manifest feature is unconditional and can affect non-`server-full` graphs, but this phase certifies only `server-full`. Embedded/mobile and other profiles require separate evidence and receive no verdict from this decision.
- Every build host that compiles this ordinary non-FIPS backend provides a C/C++ compiler. Per upstream documentation, CMake, bindgen, and Go are not required for non-FIPS `aws-lc-sys`.
- The prior `knowme-embedded-mobile-cargo-gating-scope-assessment` allowed existing server dependencies when Android/iOS already compiled and warned against speculative wide gating. This decision is consistent only in that narrow sense: it does not claim embedded/mobile success after the new feature, and any observed regression must be handled by the phase that certifies that profile.

## Falsifier

Reject or reopen this decision if the parent manifest edit introduces a new package; if the exactly-one-provider tree assertion fails; if an unpatched advisory affects the locked `jsonwebtoken` or AWS-LC version; if the `server-full` graph no longer otherwise activates AWS-LC; or if a certified `no_std`, bare-WASM, C/C++-free, HSM/KMS, or FIPS requirement becomes real. A patched stable RustCrypto RSA release also reopens the comparison, although it does not erase the separately measured active-package difference. Non-`server-full` and cross-target behavior is outside this verdict rather than a passively certified falsifier.

## Rejected alternative

`rust_crypto` remains a valid upstream backend but is rejected as UAR's standard for this profile. It activates 22 packages beyond the baseline, including `rsa 0.9.10`, while its portability benefits do not satisfy a current exit criterion. The RSA timing advisory has no patched version but is not treated as evidence of a currently exercised UAR private-key path.

## Parent verification commands

After the manifest edit, parent A0 must run:

```bash
git diff -- Cargo.toml Cargo.lock
cargo check --locked --no-default-features --features server-full
cargo clippy -p universal-agent-runtime
cargo tree --locked --no-default-features --features server-full \
  -e features -i jsonwebtoken@11.0.0
cargo test --locked --no-default-features --features server-full \
  --lib test_resolve_user_context -- --nocapture
openspec validate fix-jwt-crypto-provider --strict
```

The existing `test_resolve_user_context_valid_token` is the HS256 round trip. Parent A0 must add `test_resolve_user_context_rejects_token_signed_with_wrong_secret` before the focused command. In a provider-disabled scratch checkout it must also run:

```bash
cargo test --locked --no-default-features --features server-full \
  --lib uar::security::middleware::tests::test_resolve_user_context_valid_token \
  -- --exact --nocapture
```

That negative control must be observed to fail with the missing-provider message. Cross-target and non-`server-full` checks are not silently claimed; this phase's reporting contract limits all results to `server-full`.

The tree output must contain `aws_lc_rs` and must not contain `rust_crypto`. The owner of any future `Cargo.lock`, `jsonwebtoken`, provider-feature, or release-target update must repeat `cargo info jsonwebtoken@11.0.0`, the tree assertion above, and the repository's prescribed advisory verification before carrying this decision forward.
