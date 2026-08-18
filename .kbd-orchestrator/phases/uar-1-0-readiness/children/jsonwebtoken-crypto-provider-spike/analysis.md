# Analysis — jsonwebtoken crypto provider

> **Historical server-full analysis.** Its AWS-LC conclusion was superseded on
> 2026-08-13 after the workspace feature graph revealed both providers active.
> See `re-evaluation-2026-08-13.md` for the binding workspace decision.

Date: 2026-08-13
Baseline: `a5978c038a97d499c219d55dd4e43feea3268e91`
Profile: `server-full` only; no result transfers to another feature profile or target class.

## Binding recommendation

Keep the direct dependency on `jsonwebtoken` major version 11, currently locked at 11.0.0, and select its `aws_lc_rs` built-in provider:

```toml
jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }
```

Do not enable `rust_crypto` at the same time. Do not add a manual `CryptoProvider::install_default` call. Do not add a direct `aws-lc-rs` dependency. This selection is the ordinary non-FIPS backend and makes no FIPS certification claim.

## Research budget and route

- Budget: at most 8 discovery queries per tier and 20 active research minutes; build/link wait time excluded.
- Tier 1, GitHub: 8 queries attempted. Four malformed repository-qualified searches failed without findings; four corrected searches plus upstream issue/PR inspection established current backend design and maintenance activity.
- Tier 2, documentation: Context7 was required by repository policy but is not available in this harness. The fallback used official `jsonwebtoken` source/docs, the AWS-LC user guide, and upstream security policies.
- Tier 3, package evidence: `cargo info`, `cargo tree`, the locked manifests, and the exact registry source for `jsonwebtoken 11.0.0` established versions, features, graph activation, and algorithm implementations.
- Tier 4, web/security: primary upstream documentation and RustSec advisories resolved platform and current vulnerability questions. No community benchmark or secondary comparison was used.
- Stopping rule met: measured runtime behavior and active-package topology distinguish the candidates. Comparative performance remains unknown and is not used as a decision input; no release benchmark was run during this implementation-controlled phase.

## Observed facts

### Version and provider contract

- `cargo info jsonwebtoken@11.0.0` reports 11.0.0, MIT, Rust 1.88, default `use_pem`, and exactly two built-in provider features: `aws_lc_rs` and `rust_crypto`.
- The current direct manifest entry enables only defaults. `cargo tree --locked --no-default-features --features server-full -e features -i jsonwebtoken@11.0.0` shows `default` and `use_pem`, but neither provider.
- The pinned source's `CryptoProvider::from_crate_features` selects a built-in only when exactly one provider feature is enabled. Neither or both returns a provider whose operations panic.
- The unchanged focused middleware command observed 3 passing cases and one failure. The valid-token test panicked at `jsonwebtoken-11.0.0/src/crypto/mod.rs:124` with the missing-provider message.
- `cargo info jsonwebtoken@10.4.0` reports `10.4.0 (latest 11.0.0)` and shows the same provider choice. Downgrading therefore does not solve the observed configuration defect. It also does not remove both majors: SurrealDB uses 10.4.0 while liter-llm uses 11.0.0. Keeping the already compiled/current direct 11 API is the minimum change.
- `jsonwebtoken` 11's provider abstraction was merged upstream in [PR 452](https://github.com/Keats/jsonwebtoken/pull/452).

### Functional coverage

The exact 11.0.0 source implements the same JWT algorithm set in both built-ins: HS256/384/512, ES256/384, RS256/384/512, PS256/384/512, and EdDSA, for signing and verification. Algorithm support is therefore a tie and does not decide the backend.

### Dependency topology

- `aws-lc-rs 1.17.0` and `aws-lc-sys 0.41.0` are already active under `server-full` through rustls 0.23.43 and SurrealDB's transitive `jsonwebtoken 10.4.0`.
- Reverse-tree queries for `p256 0.13.2`, `p384 0.13.1`, `rsa 0.9.10`, and `ed25519-dalek 2.2.0` print `nothing to print` in the current `server-full` graph. They occur in `Cargo.lock` but are not active for this build.
- Read-only feature simulation measured 918 active normal/build packages for the baseline, 918 with `jsonwebtoken/aws_lc_rs`, and 940 with `jsonwebtoken/rust_crypto`. The RustCrypto set difference contains 22 packages; the AWS-LC set difference is empty. Commands and the package list are in `research-evidence.md`.
- `cargo tree --locked --features 'server-full,jsonwebtoken/aws_lc_rs'` succeeded and `git diff -- Cargo.toml Cargo.lock` remained empty. This verifies before implementation that the AWS-LC feature reuses the locked packages. Parent A0 must still inspect its actual diff; any new package fires contract stop condition 1.

### Platform and build requirements

- UAR's release matrix is Linux x86-64/aarch64, macOS x86-64/aarch64, and Windows x86-64. This spike built UAR only on the current aarch64 macOS host and makes no UAR cross-target build claim.
- The [AWS-LC platform matrix](https://aws.github.io/aws-lc-rs/platform_support.html) reports builds and tests for each of those targets. For non-FIPS `aws-lc-sys`, it requires a C/C++ compiler but not CMake, bindgen, or Go.
- The [AWS-LC README](https://github.com/aws/aws-lc-rs/blob/main/aws-lc-rs/README.md) states that the crate uses AWS-LC through FFI and does not support `no_std`. RustCrypto's pure-Rust and `no_std` advantages are real, but neither is required by the certified `server-full` profile.
- AWS-LC's native toolchain cost is already paid by the present graph. This is a repository-specific fact, not a general recommendation for new Rust projects.
- The build-environment assumption is explicit: any host compiling this non-FIPS backend must provide a C/C++ compiler. CMake, bindgen, and Go are not required for non-FIPS `aws-lc-sys` according to the upstream guide.
- The selected dependency feature is unconditional in the direct manifest and may participate in non-`server-full` graphs. This spike certifies none of them. Embedded/mobile and other profiles must not inherit this result without their own tier-appropriate checks; an observed build regression there reopens the decision.

### Security posture

- RustCrypto avoids C, assembly, and FFI. Its selected RSA implementation would be `rsa 0.9.10`; [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), last modified 2026-04-25, describes a network-observable private-key timing attack and lists no patched version. Repository search found no current RS*/PS* signing path, so this is a conservative secondary consideration about exposed provider capability, not a currently exercised vulnerability in UAR.
- AWS-LC adds C/assembly and FFI attack surface. Its March 2026 `aws-lc-sys` advisories have patched ranges; for example [RUSTSEC-2026-0046](https://rustsec.org/advisories/RUSTSEC-2026-0046.html) is patched in 0.38.0, while this graph pins 0.41.0. Parent verification must still run the repository's normal advisory checks at their prescribed tier.
- `aws_lc_rs` is not `aws-lc-fips-sys`. A possible future FIPS requirement needs a separate design and certification review; it is not a hidden benefit of this choice.

### Maintenance

- `cargo info` reports `jsonwebtoken 11.0.0` as the current direct release and shows `aws-lc-rs 1.18.0` newer than the graph's 1.17.0. This spike does not upgrade the transitive backend; normal dependency review owns that later update.
- The `jsonwebtoken`, `aws-lc-rs`, and RustCrypto RSA repositories all showed current 2026 activity when inspected. Maintenance activity does not break the tie; the dependency graph and unresolved RSA advisory do.

## Candidate verdicts

| Candidate | Verdict | Reason within `server-full` |
|---|---|---|
| `jsonwebtoken 11.0.0` + `aws_lc_rs` | Adopt | Fixes the observed panic, covers the full algorithm set, supports every release target, and reuses an already active/patched native backend. |
| `jsonwebtoken 11.0.0` + `rust_crypto` | Reject as the standard backend | Functional and portable, but expands the active graph and activates an RSA implementation with an unresolved private-key timing advisory. Its portability benefits are outside this phase's certified profile. |
| Custom/manual `CryptoProvider` | Reject | No algorithm, hardware-key, or policy requirement needs a custom provider. It would add source-level process initialization and a new correctness surface when one built-in feature is sufficient. |

## Decision rationale

AWS-LC wins for this repository and profile primarily because it adds zero active packages while RustCrypto activates 22, covers the same algorithms, and supports all release targets. Its pinned `aws-lc-sys` is also beyond the patched ranges of the inspected 2026 advisories. RustCrypto's strongest benefits—pure Rust, `no_std`, and WebAssembly portability—do not satisfy a current `server-full` requirement. Its unresolved RSA advisory is secondary because UAR has no observed RSA private-key signing path today.

No performance claim supports this decision. No FIPS claim supports this decision.

## Parent A0 handoff

1. Change only the direct manifest entry to `jsonwebtoken = { version = "11.0.0", features = ["aws_lc_rs"] }`.
2. Run `git diff -- Cargo.toml Cargo.lock`. No lockfile change is predicted; stop if any new package appears.
3. Run Tier 0 exactly as the parent contract requires: `cargo check --locked --no-default-features --features server-full` and `cargo clippy -p universal-agent-runtime`.
4. Keep `uar::security::middleware::tests::test_resolve_user_context_valid_token` as the HS256 round-trip assertion. Add `test_resolve_user_context_rejects_token_signed_with_wrong_secret` and require an error rather than a panic.
5. Run both the new and unchanged middleware assertions with `cargo test --locked --no-default-features --features server-full --lib test_resolve_user_context -- --nocapture`.
6. In a provider-disabled scratch checkout, run `cargo test --locked --no-default-features --features server-full --lib uar::security::middleware::tests::test_resolve_user_context_valid_token -- --exact --nocapture` and record the expected missing-provider failure.
7. Assert provider exclusivity with `cargo tree --locked --no-default-features --features server-full -e features -i jsonwebtoken@11.0.0`: `aws_lc_rs` must be active and `rust_crypto` absent. Repeat this check whenever another workspace member changes its `jsonwebtoken 11` features because Cargo features are additive.
8. Run `openspec validate fix-jwt-crypto-provider --strict` after its verification record and tasks are complete.
9. Do not run Tier 2 until all six parent OpenSpec changes (`fix-jwt-crypto-provider`, `gap-02-jwks-token-verifier`, `gap-03-a2a-tenant-partitioning`, `skill-builtins-on-embedded`, `skill-scoped-governance`, and `skill-config-reconciliation`) reach phase completion. The pinned command is `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked --no-default-features --features server-full --test integration live::capability_cases -- --test-threads=1`.

## Re-evaluation triggers

Reopen this decision only if one of these becomes true:

- AWS-LC is no longer active elsewhere in the `server-full` graph.
- Another workspace member enables `rust_crypto` or another provider feature on `jsonwebtoken 11`; the exactly-one-provider tree assertion must then be restored before JWT use.
- UAR adds a certified `no_std`, bare-WASM, or C/C++-free distribution target.
- A supported release target loses upstream AWS-LC build/test coverage.
- A new unpatched AWS-LC advisory affects the locked version or an AWS-LC platform regression is observed.
- RustCrypto's RSA timing advisory gains a patched stable version; reopen the comparison, while recognizing that a patch does not erase the separately measured 22-package activation difference.
- UAR receives an actual FIPS requirement; that requires evaluation of `aws-lc-fips-sys`, toolchain requirements, module version, and certification evidence rather than assuming this non-FIPS choice transfers.
- A dependency update reports `jsonwebtoken 11.0.0` is no longer the intended current patch or introduces an advisory affecting the locked `jsonwebtoken` version.

## Independent-review warning resolution

- Comparative completeness was downgraded in `assessment.md`; the binding comparison is made here.
- Measured evidence includes the observed failure and exact active-package counts in `research-evidence.md`; performance is explicitly unknown and excluded from the rationale.
- Security risks are stated for both backends, including AWS-LC's native/FFI surface and patched advisory ranges.
- Registry source and AWS platform claims now have resolvable paths or URLs.
