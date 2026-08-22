# A0 negative-control evidence

Date: 2026-08-14
Worktree: `/Users/gqadonis/.claude/worktrees/uar-1-0-readiness`
Source baseline: `a5978c038a97d499c219d55dd4e43feea3268e91`

These controls ran in isolated scratch crates. Their manifests and test bodies
are included so the commands are replayable without relying on an untracked
temporary file as evidence. Exit `101` is expected for each deliberately false
acceptance assertion.

## Provider-disabled round trip

Scratch directory: `/tmp/uar-jwt-provider-disabled`

`Cargo.toml`:

```toml
[package]
name = "uar-jwt-provider-disabled"
version = "0.0.0"
edition = "2024"

[dependencies]
jsonwebtoken = { version = "=11.0.0", default-features = false }
serde = { version = "1", features = ["derive"] }
```

`src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    #[test]
    fn provider_disabled_round_trip_control() {
        let token = jsonwebtoken::encode(
            &Header::default(),
            &Claims {
                sub: "provider-disabled".to_string(),
                exp: usize::MAX,
            },
            &EncodingKey::from_secret(b"secret"),
        )
        .expect("provider-disabled negative control deliberately assumes encoding works");

        jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(b"secret"),
            &Validation::default(),
        )
        .expect("provider-disabled negative control deliberately assumes decoding works");
    }
}
```

Command:

```bash
cd /tmp/uar-jwt-provider-disabled
RUSTC_WRAPPER= SCCACHE_DISABLE=1 CARGO_NET_OFFLINE=true \
  cargo test tests::provider_disabled_round_trip_control -- --exact --nocapture
```

Observed exit: `101`

Observed output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.91s
Running unittests src/lib.rs (/Volumes/my-passport/cargo-build/51/7bec5b40e06eec/debug/build/uar-jwt-provider-disabled/89a3fdadbaa7b220/out/uar_jwt_provider_disabled-89a3fdadbaa7b220)

running 1 test

thread 'tests::provider_disabled_round_trip_control' (51557839) panicked at /Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/jsonwebtoken-11.0.0/src/crypto/mod.rs:124:40:

Could not automatically determine the process-level CryptoProvider from jsonwebtoken crate features.
Call CryptoProvider::install_default() before this point to select a provider manually, or make sure exactly one of the 'rust_crypto' and 'aws_lc_rs' features is enabled.
See the documentation of the CryptoProvider type for more information.

test tests::provider_disabled_round_trip_control ... FAILED

failures:
    tests::provider_disabled_round_trip_control

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

## AWS-LC preinstalled before the UAR guard

Scratch directory: `/tmp/uar-jwt-provider-conflict`

`Cargo.toml`:

```toml
[package]
name = "uar-jwt-provider-conflict"
version = "0.0.0"
edition = "2024"

[dependencies]
jsonwebtoken = { version = "=11.0.0", default-features = false, features = ["aws_lc_rs", "rust_crypto"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

`src/lib.rs`:

```rust
#[path = "/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/src/uar/security/jwt.rs"]
mod jwt;

#[cfg(test)]
mod tests {
    use super::jwt::{self, JwtError};
    use jsonwebtoken::crypto::{aws_lc, rust_crypto};

    fn install_aws_lc() {
        aws_lc::DEFAULT_PROVIDER
            .install_default()
            .expect("isolated process should accept AWS-LC first");
    }

    #[test]
    fn conflicting_aws_lc_provider_returns_structured_error() {
        install_aws_lc();
        assert!(matches!(
            jwt::ensure_rustcrypto_provider(),
            Err(JwtError::ProviderConflict)
        ));
    }

    #[test]
    fn negative_control_assumes_conflicting_provider_is_accepted() {
        install_aws_lc();
        jwt::ensure_rustcrypto_provider()
            .expect("negative control deliberately assumes AWS-LC is accepted");
    }

    #[test]
    fn identical_rustcrypto_preinstalled_returns_conflict() {
        rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .expect("isolated process should accept RustCrypto first");
        assert!(matches!(
            jwt::ensure_rustcrypto_provider(),
            Err(JwtError::ProviderConflict)
        ));
    }
}
```

Command:

```bash
cd /tmp/uar-jwt-provider-conflict
RUSTC_WRAPPER= SCCACHE_DISABLE=1 \
  cargo test tests::negative_control_assumes_conflicting_provider_is_accepted \
  -- --exact --nocapture
```

Observed exit: `101`

Observed output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 5.23s
Running unittests src/lib.rs (/Volumes/my-passport/cargo-build/b2/1d7de7a158f894/debug/build/uar-jwt-provider-conflict/3544afda79bfb652/out/uar_jwt_provider_conflict-3544afda79bfb652)

running 1 test

thread 'tests::negative_control_assumes_conflicting_provider_is_accepted' (51544036) panicked at src/lib.rs:28:14:
negative control deliberately assumes AWS-LC is accepted: ProviderConflict
test tests::negative_control_assumes_conflicting_provider_is_accepted ... FAILED

failures:
    tests::negative_control_assumes_conflicting_provider_is_accepted

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

error: test failed, to rerun pass `--lib`
```

The matching positive assertion ran separately:

```bash
RUSTC_WRAPPER= SCCACHE_DISABLE=1 \
  cargo test tests::conflicting_aws_lc_provider_returns_structured_error \
  -- --exact --nocapture
```

Observed output:

```text
running 1 test
test tests::conflicting_aws_lc_provider_returns_structured_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
```

## Wrong-secret acceptance

Scratch directory: `/tmp/uar-jwt-wrong-secret-control`

`Cargo.toml`:

```toml
[package]
name = "uar-jwt-wrong-secret-control"
version = "0.0.0"
edition = "2024"

[features]
server = []
default = ["server"]

[dependencies]
jsonwebtoken = { version = "=11.0.0", default-features = false, features = ["rust_crypto"] }
serde = { version = "1", features = ["derive"] }
thiserror = "2"
```

`src/lib.rs`:

```rust
#[path = "/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/src/uar/security/jwt.rs"]
mod jwt;

#[cfg(test)]
mod tests {
    use super::jwt;
    use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    #[test]
    fn negative_control_assumes_wrong_secret_is_accepted() {
        let token = jwt::encode(
            &Header::default(),
            &Claims {
                sub: "negative-control".to_string(),
                exp: usize::MAX,
            },
            &EncodingKey::from_secret(b"wrong-secret"),
        )
        .expect("token creation should succeed");

        jwt::decode::<Claims>(
            token,
            &DecodingKey::from_secret(b"correct-secret"),
            &Validation::default(),
        )
        .expect("negative control deliberately assumes wrong-secret acceptance");
    }
}
```

Command:

```bash
cd /tmp/uar-jwt-wrong-secret-control
RUSTC_WRAPPER= SCCACHE_DISABLE=1 \
  cargo test tests::negative_control_assumes_wrong_secret_is_accepted \
  -- --exact --nocapture
```

Observed exit: `101`

Observed output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 4.81s
Running unittests src/lib.rs (/Volumes/my-passport/cargo-build/02/a736933dd44abd/debug/build/uar-jwt-wrong-secret-control/3fa84bc2db6b90f9/out/uar_jwt_wrong_secret_control-3fa84bc2db6b90f9)

running 1 test

thread 'tests::negative_control_assumes_wrong_secret_is_accepted' (51545544) panicked at src/lib.rs:33:10:
negative control deliberately assumes wrong-secret acceptance: Token(Error(InvalidSignature))
test tests::negative_control_assumes_wrong_secret_is_accepted ... FAILED

failures:
    tests::negative_control_assumes_wrong_secret_is_accepted

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

## RustCrypto preinstalled before UAR

An identical RustCrypto provider preinstalled before UAR produces the same
public `install_default()` result as a foreign provider. The operator selected
a first-owner boundary on 2026-08-14: UAR must acquire the process slot itself,
and any earlier installation fails closed. This test is positive evidence for
that boundary; it does not claim that `jsonwebtoken` exposes provider identity.

Command proving the current behavior:

```bash
cd /tmp/uar-jwt-provider-conflict
RUSTC_WRAPPER= SCCACHE_DISABLE=1 \
  cargo test tests::identical_rustcrypto_preinstalled_returns_conflict \
  -- --exact --nocapture
```

Observed exit: `0`

Observed output:

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 19s
Running unittests src/lib.rs (/Volumes/my-passport/cargo-build/b2/1d7de7a158f894/debug/build/uar-jwt-provider-conflict/3544afda79bfb652/out/uar_jwt_provider_conflict-3544afda79bfb652)

running 1 test
test tests::identical_rustcrypto_preinstalled_returns_conflict ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

This is observed acceptance evidence for UAR's first-owner contract.

## Final ownership re-evaluation — supersedes the pointer-identity attempt

The intermediate pointer-identity guard compared the error from
`rust_crypto::DEFAULT_PROVIDER.install_default()` with the RustCrypto static.
That comparison accepted a process in which AWS-LC had already been installed.
The dual-provider scratch test observed the defect directly:

```text
running 1 test
thread 'tests::conflicting_aws_lc_provider_returns_structured_error' panicked at src/lib.rs:14:9:
assertion failed: matches!(jwt::ensure_rustcrypto_provider(), Err(JwtError::ProviderConflict))
test tests::conflicting_aws_lc_provider_returns_structured_error ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
```

The reason is visible in the pinned `jsonwebtoken` 11.0.0 source:
`install_default` calls `OnceLock::set(default_provider)`. `OnceLock::set`
returns the attempted value on failure, not the value already stored. The only
getter is `pub(crate)`. No public API can distinguish an identical preinstalled
RustCrypto provider from preinstalled AWS-LC.

The final rule therefore restores UAR first ownership: UAR-owned startup
installs RustCrypto and caches its own success; any earlier process owner returns
`ProviderConflict`. The final scratch source was:

```rust
#[path = "/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/src/uar/security/jwt.rs"]
mod jwt;

#[cfg(test)]
mod tests {
    use super::jwt::{self, JwtError};
    use jsonwebtoken::crypto::{aws_lc, rust_crypto};

    #[test]
    fn conflicting_aws_lc_provider_returns_structured_error() {
        aws_lc::DEFAULT_PROVIDER
            .install_default()
            .expect("isolated process should accept AWS-LC first");
        assert!(matches!(
            jwt::ensure_rustcrypto_provider(),
            Err(JwtError::ProviderConflict)
        ));
    }

    #[test]
    fn negative_control_assumes_preinstalled_aws_lc_is_accepted() {
        aws_lc::DEFAULT_PROVIDER
            .install_default()
            .expect("isolated process should accept AWS-LC first");
        jwt::ensure_rustcrypto_provider()
            .expect("negative control deliberately assumes AWS-LC is accepted");
    }

    #[test]
    fn preinstalled_rustcrypto_returns_structured_error() {
        rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .expect("isolated process should accept RustCrypto first");
        assert!(matches!(
            jwt::ensure_rustcrypto_provider(),
            Err(JwtError::ProviderConflict)
        ));
    }
}
```

Commands and observed output:

```bash
cd /tmp/uar-jwt-provider-conflict
cargo test --offline tests::preinstalled_rustcrypto_returns_structured_error -- --exact --nocapture
```

```text
running 1 test
test tests::preinstalled_rustcrypto_returns_structured_error ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
```

```bash
cargo test --offline tests::conflicting_aws_lc_provider_returns_structured_error -- --exact --nocapture
```

```text
running 1 test
test tests::conflicting_aws_lc_provider_returns_structured_error ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
```

```bash
cargo test --offline tests::negative_control_assumes_preinstalled_aws_lc_is_accepted -- --exact --nocapture
```

Observed exit: `101`

```text
running 1 test
thread 'tests::negative_control_assumes_preinstalled_aws_lc_is_accepted' panicked at src/lib.rs:26:14:
negative control deliberately assumes AWS-LC is accepted: ProviderConflict
test tests::negative_control_assumes_preinstalled_aws_lc_is_accepted ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
```
