# Research evidence receipt — jsonwebtoken crypto provider spike

All commands ran from `/Users/gqadonis/.claude/worktrees/uar-1-0-readiness` at baseline `a5978c03`. Results apply only to `--no-default-features --features server-full` unless a command says otherwise.

## Provider-disabled runtime control

Command:

```bash
cargo test --locked --no-default-features --features server-full \
  --lib test_resolve_user_context -- --nocapture
```

Observed result:

```text
running 4 tests
...anonymous_when_jwt_disabled_and_no_header ... ok
...anonymous_when_jwt_disabled_and_invalid_header ... ok
...unauthorized_when_jwt_required_and_no_header ... ok
...valid_token ... FAILED

Could not automatically determine the process-level CryptoProvider from jsonwebtoken crate features.
...make sure exactly one of the 'rust_crypto' and 'aws_lc_rs' features is enabled.

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 552 filtered out
```

The build portion finished in 12m42s. The test bodies finished in 0.00s. This is a negative control, not a performance comparison.

## Exact provider features

Command:

```bash
cargo info jsonwebtoken@11.0.0
```

Observed feature block:

```text
+default     = [use_pem]
 use_pem     = [dep:pem, dep:simple_asn1]
 aws_lc_rs   = [dep:aws-lc-rs]
 rust_crypto = [dep:ed25519-dalek, dep:hmac, dep:p256, dep:p384, dep:rand, dep:rsa, dep:sha2]
```

Version comparison command:

```bash
cargo info jsonwebtoken@10.4.0
```

Observed result:

```text
version: 10.4.0 (latest 11.0.0)
```

Both 10.4.0 and 11.0.0 expose `aws_lc_rs` and `rust_crypto`. Downgrading would not remove the missing-provider decision. It also would not remove the duplicate major from `server-full`: SurrealDB uses 10.4.0 while liter-llm uses 11.0.0. The current direct 11 API is already compiled, and no version defect was observed.

## Algorithm implementations

Command:

```bash
rg -n 'Algorithm::' \
  "$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jsonwebtoken-11.0.0/src/crypto/aws_lc" \
  "$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jsonwebtoken-11.0.0/src/crypto/rust_crypto"
```

Observed in both provider factories: HS256/384/512, ES256/384, RS256/384/512, PS256/384/512, and EdDSA signer and verifier branches. The binding decision only needs the currently exercised HS256 and planned public-key verification paths, but the broader equality claim is grounded in the exact pinned source.

## AWS-LC feature simulation without edits

Command:

```bash
cargo tree --locked --no-default-features \
  --features 'server-full,jsonwebtoken/aws_lc_rs' \
  -i aws-lc-rs@1.17.0
```

Observed reverse-tree addition:

```text
aws-lc-rs v1.17.0
├── jsonwebtoken v10.4.0
├── jsonwebtoken v11.0.0
├── rustls v0.23.43
└── rustls-webpki v0.103.13
```

The command exited 0 with `--locked`. `git diff -- Cargo.toml Cargo.lock` remained empty. This directly verifies that enabling the dependency feature can reuse the locked AWS-LC packages without a new package or lockfile change.

## RustCrypto feature simulation without edits

Command:

```bash
cargo tree --locked --no-default-features \
  --features 'server-full,jsonwebtoken/rust_crypto' \
  -i rsa@0.9.10
```

Observed reverse tree:

```text
rsa v0.9.10
└── jsonwebtoken v11.0.0
    ├── liter-llm v1.12.0
    └── universal-agent-runtime v1.0.0
```

## Measured active package counts

Method:

```bash
cargo tree --locked --no-default-features --features '<feature set>' \
  --edges normal,build --prefix none --format '{p}' \
  | sed 's/ (.*)$//' | sort -u | wc -l
```

Observed counts:

```text
server-full baseline                                918
server-full + jsonwebtoken/aws_lc_rs                918
server-full + jsonwebtoken/rust_crypto              940
```

The 22 RustCrypto-only active packages were:

```text
base16ct 0.2.0
crypto-bigint 0.5.5
curve25519-dalek 4.1.3
der 0.7.10
ecdsa 0.16.9
ed25519 2.2.3
ed25519-dalek 2.2.0
elliptic-curve 0.13.8
ff 0.13.1
group 0.13.0
hkdf 0.12.4
num-bigint-dig 0.8.6
p256 0.13.2
p384 0.13.1
pem-rfc7468 0.7.0
pkcs1 0.7.5
pkcs8 0.10.2
primeorder 0.13.6
rfc6979 0.4.0
rsa 0.9.10
sec1 0.7.3
spki 0.7.3
```

AWS-LC had no package in the corresponding set difference.

## Current UAR signing algorithms

Command:

```bash
rg -n 'Algorithm::(RS|PS)|EncodingKey::from_rsa|encode\(' \
  src tests tools/uar-jwt-proxy
```

Observed result: token creation exists in the security middleware, API-key code, integration baseline, and JWT proxy, but no `Algorithm::RS*`, `Algorithm::PS*`, or `EncodingKey::from_rsa*` use was found. Current UAR signing evidence is HS256/default-header only. The RustCrypto RSA advisory is therefore a conservative secondary consideration about the selected provider's exposed capability, not evidence of a currently exercised RSA private-key path.

## Timeline clarification

Research evidence gathering began during assessment before `assess.handoff.json` was written. The long baseline check/test and initial upstream inspection completed before the 07:04:11Z assess handoff. Analyze then ran the feature simulations, package counts, and candidate comparison; `library-candidates.json` was generated at 07:06:28Z. Its `generated_at` is artifact creation time, not research start time.
