# Android `embedded-mobile` verification evidence

Date: 2026-08-14
Worktree: `/Users/gqadonis/.claude/worktrees/uar-1-0-readiness`
Target: `aarch64-linux-android`
Profile: `embedded-mobile`

The installed Android NDK exposes API-suffixed compiler drivers rather than a
bare `aarch64-linux-android-clang`. The pre-existing `native-tls` dependency
also requires a target OpenSSL sysroot. A temporary vendored OpenSSL sysroot was
built under `/tmp`; it changed no repository manifest or lockfile.

Command, run literally from the worktree root:

```bash
PATH=/Users/gqadonis/Library/Android/sdk/ndk/28.2.13676358/toolchains/llvm/prebuilt/darwin-x86_64/bin:/Users/gqadonis/.cargo/bin:/usr/bin:/bin \
CC_aarch64_linux_android=/Users/gqadonis/Library/Android/sdk/ndk/28.2.13676358/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android35-clang \
AR_aarch64_linux_android=/Users/gqadonis/Library/Android/sdk/ndk/28.2.13676358/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar \
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=/Users/gqadonis/Library/Android/sdk/ndk/28.2.13676358/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android35-clang \
AARCH64_LINUX_ANDROID_OPENSSL_DIR=/tmp/uar-android-openssl-sysroot/build/aarch64-linux-android/debug/build/openssl-sys/aa411d62480924f6/out/openssl-build/install \
AARCH64_LINUX_ANDROID_OPENSSL_STATIC=1 \
RUSTC_WRAPPER= SCCACHE_DISABLE=1 CARGO_NET_OFFLINE=true \
cargo check --locked -p universal-agent-runtime --no-default-features \
  --features embedded-mobile --target aarch64-linux-android --lib
```

Observed exit: `0`

Observed output:

```text
warning: constant `MAX_BODY_BYTES` is never used
  --> src/uar/tools/fetch_guard.rs:54:7
   |
54 | const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
   |       ^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: constant `MAX_REDIRECTS` is never used
  --> src/uar/tools/fetch_guard.rs:56:7
   |
56 | const MAX_REDIRECTS: usize = 5;
   |       ^^^^^^^^^^^^^

warning: `universal-agent-runtime` (lib) generated 2 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 46s
warning: the following packages contain code that will be rejected by a future version of Rust: nix v0.31.3, redis v1.2.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
```

The two source warnings predate A0 and are outside its permitted surface. This
result applies only to `embedded-mobile` on `aarch64-linux-android`.
