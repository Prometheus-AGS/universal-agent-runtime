# Pre-repair evidence

Date: 2026-08-22
Candidate source: `487fb394006f4f1dbe0280b455d5107c576d7e99`
Host architecture: `arm64`

## Production selector

Command:

```bash
git show 487fb394006f4f1dbe0280b455d5107c576d7e99:Dockerfile \
  | rg -n 'cargo \+nightly build'
```

Observed output and exit:

```text
225:    && CARGO_NET_GIT_FETCH_WITH_CLI=true cargo +nightly build --release \
exit 0
```

## Clean production-image failure

Command recorded by the child assessment:

```bash
CARGO_TARGET_DIR=/Users/gqadonis/.claude/worktrees/uar-1-0-readiness/target \
  scripts/certify-operational-resilience-local.sh certify
```

Observed output and exit recorded by the assessment:

```text
native release build completed in 49.93 seconds
Docker backend cargo +nightly resolved Rust 1.100.0-nightly
diskann-wide 0.54.0 failed on Linux ARM64 with E0283 at
src/arch/aarch64/mod.rs:348, :363, and :379
error: could not compile `diskann-wide` (lib) due to 3 previous errors
exit 101
```

The failure occurred before any operational-resilience assertion or soak
interval began. It is a candidate-build failure, not a soak assertion result.

## Immutable isolated negative control

Commands:

```bash
uname -m
rustc +nightly-2026-08-22 -Vv
CARGO_TARGET_DIR=/tmp/uar-diskann-pin-probe.iNERxp/target-baseline \
  cargo +nightly-2026-08-22 check --locked \
  --manifest-path /tmp/uar-diskann-pin-probe.iNERxp/Cargo.toml \
  --target aarch64-apple-darwin
```

Observed identity:

```text
arm64
rustc 1.100.0-nightly (c656540d6 2026-08-21)
commit-hash: c656540d6467dee1381f0cbd882412d6bd1cd5ae
host: aarch64-apple-darwin
release: 1.100.0-nightly
LLVM version: 23.1.0
```

Observed failure and exit:

```text
Checking diskann-wide v0.54.0
error[E0283]: type annotations needed
  --> diskann-wide-0.54.0/src/arch/aarch64/mod.rs:348:82
error[E0283]: type annotations needed
  --> diskann-wide-0.54.0/src/arch/aarch64/mod.rs:363:44
error[E0283]: type annotations needed
  --> diskann-wide-0.54.0/src/arch/aarch64/mod.rs:379:44
error: could not compile `diskann-wide` (lib) due to 3 previous errors
exit 101
```
