# Handoff out — jsonwebtoken crypto provider re-evaluation

**Parent:** `uar-1-0-readiness`
**Parent change:** `fix-jwt-crypto-provider` (A0)
**Decision:** binding workspace standard; target results remain target-specific.

## Binding choice

Define one workspace dependency and inherit it from every UAR-owned package:

```toml
jsonwebtoken = { version = "=11.0.0", default-features = false, features = ["rust_crypto"] }
```

The exact pin makes any upgrade deliberate. Do not enable `aws_lc_rs` in a UAR
package and do not add a direct backend dependency.

## Required runtime boundary

Cargo features are additive for downstream embedders. Before every UAR JWT
operation, explicitly install `jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER`.
UAR must acquire the process slot itself at the shared server-startup funnel;
cache that successful installation for idempotent reuse. Return a structured
provider-conflict error if any provider was installed first, including an
indistinguishable RustCrypto provider. The public v11 API exposes no installed
provider identity, so no pointer-comparison claim is made. The runtime wrapper
and proxy startup both enforce this first-owner boundary.

## Evidence and limits

- Pre-fix workspace tree: both `aws_lc_rs` and `rust_crypto` active.
- Provider-disabled round-trip: observed failure with the missing-provider panic.
- Isolated RustCrypto target checks: iOS and Android completed successfully.
- RUSTSEC-2023-0071 applies to observable RSA private-key operations, not the
  current HS256 signing or planned public-key verification paths.
- No FIPS, performance, runtime-wide, or cross-target transfer claim is made.

Full correction record: `re-evaluation-2026-08-13.md`. Historical AWS-LC
analysis remains available but is superseded.

## Parent boundary

A0 completes only after focused positive tests, the provider-conflict negative
control, singular workspace feature-tree evidence, Tier 0, proxy check, both
full embedded-mobile target checks, strict OpenSpec validation, and the
artifact-refiner gate. Tier 2 remains deferred until all six parent changes are
complete. Do not archive, push, or open a PR.
