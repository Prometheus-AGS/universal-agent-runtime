## Why

**Every JWT sign and verify in the runtime panics.** This is a live
authentication defect on `main`, not a test-harness problem.

`Cargo.toml:393` declares `jsonwebtoken = "11.0.0"` with default features. That
crate's manifest reads:

```toml
[features]
aws_lc_rs = ["dep:aws-lc-rs"]
default = ["use_pem"]
rust_crypto = ["dep:ed25519-dalek", "dep:hmac", "dep:p256", "dep:p384", "dep:rand", "dep:rsa", "dep:sha2"]
```

`default` enables **neither** crypto backend. With both features off,
`CryptoProvider::from_crate_features()` falls through to:

```rust
static INSTANCE: CryptoProvider = CryptoProvider {
    signer_factory:   |_, _| panic!("{}", NOT_INSTALLED_ERROR),
    verifier_factory: |_, _| panic!("{}", NOT_INSTALLED_ERROR),
    key_utils: KeyUtils::new_unimplemented(),
};
```

`grep -rn "install_default\|CryptoProvider" src/` returns **nothing**, so no
provider is installed at runtime either. Both call sites therefore panic:
`src/uar/security/middleware.rs:48` (`decode`) and
`src/uar/security/api_keys.rs:265` (`encode`).

### Why it looks like it works

`Cargo.lock` resolves `ed25519-dalek`, `hmac`, `rsa`, `p256` under
`jsonwebtoken 11.0.0` — pulled in by *other* crates. The dependencies are
present; the **feature flag** that selects them is not. Code compiles and links
cleanly and fails only when a token is actually signed or verified.

A second copy, `jsonwebtoken 10.4.0`, resolves via `surrealdb-core 3.2.4` **with
`aws-lc-rs`**. That copy has a provider. Ours does not. Comparing the two
lockfile entries is what makes the diagnosis unambiguous.

### How it was found

An executor working `gap-02-jwks-token-verifier` reported being blocked: A1 task
1.2 requires the existing HS256 middleware tests to pass unchanged before any
JWKS work, and they panic. **That precondition is correct and must not be
amended** — it detected a real defect exactly as intended. This change removes
the defect so 1.2 passes on its own terms.

## What Changes

- `Cargo.toml:393` — enable a crypto backend explicitly:
  `jsonwebtoken = { version = "11.0.0", features = ["rust_crypto"] }`.
  `rust_crypto` is chosen over `aws_lc_rs` because its dependencies are already
  in the tree, so this adds no new transitive crates.
- Add a test that signs and verifies a token through the real code path. A
  compile check cannot catch this class of defect: the failure is a runtime
  panic in a function pointer, so only an executed round-trip proves it.

## Capabilities

### New Capabilities
- `jwt-hardening`

## Impact

`Cargo.toml`, `Cargo.lock`, plus one test module. No source logic changes.

**Ordered before `gap-02-jwks-token-verifier` (A1).** A1 builds a JWKS verifier
on `jsonwebtoken`; building it over a panicking provider would produce a verifier
that cannot be exercised.

## Non-goals

- The JWKS/RS256 lane — A1's scope.
- Changing which algorithms are accepted. This restores the ability to execute
  the algorithms already configured; it does not alter policy.
