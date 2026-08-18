# Re-evaluation — workspace JWT provider

The original child spike selected AWS-LC for `server-full`. That decision is
superseded. Its evidence remains on disk as history; this record explains the
new observation that changed the result.

## New workspace evidence

`tools/uar-jwt-proxy/Cargo.toml` already selected `rust_crypto`, while the
pending root edit selected `aws_lc_rs`. The observed command

```bash
cargo tree --locked --workspace -e features -i jsonwebtoken@11.0.0
```

showed both provider features active on the same `jsonwebtoken` 11.0.0 package.
Pinned crate source selects a built-in automatically only when exactly one is
active; both or neither yields a process provider whose JWT operations panic.
The server-only package-count comparison therefore did not answer the
workspace architecture question.

## Superseding decision

Pin every UAR-owned dependency to exactly `jsonwebtoken` 11.0.0 with default
features disabled and only `rust_crypto` enabled. RustCrypto completed isolated
`aarch64-apple-ios` and `aarch64-linux-android` checks. Its pure-Rust build
avoids the native C/assembly and cross-toolchain cost that applies immediately
to UAR's embedded-mobile profile.

RUSTSEC-2023-0071 is recorded but is not treated as proof against UAR's current
operations: it concerns observable RSA private-key operations, while A0 uses
HS256 signing and A1 adds RSA/EC public-key verification. UAR does not add
RSA/PS private-key signing in this phase.

Because downstream Cargo features are additive, the workspace manifest is not
the complete safety boundary for an embeddable crate. UAR therefore installs
the selected provider explicitly before its own JWT operations, accepts the
same installed provider idempotently, and returns a structured error for a
different provider.

## Falsifiers

Re-evaluate when UAR requires FIPS, introduces remotely observable RSA
private-key operations, or either provider gains an unpatched advisory that
applies to operations UAR actually performs. The same applicability and patch
availability rule applies to both providers.

## External review

Two isolated K3 adversarial-review rounds rejected the AWS-LC recommendation.
The first found that workspace-only feature selection did not protect
downstream embedders. The second found that, once the RSA advisory was scoped to
actual UAR operations, the present portability and operational criteria favored
RustCrypto. Both reports passed the strict sycophancy screen.
