# wasmtime-disposition

## Why

`wasmtime`/`wasmtime-wasi` were pinned to `41.0.3` (resolving to
`41.0.4`), with 2 critical aarch64 Winch-backend sandbox-escape CVEs and
a `wasmtime-wasi` `path_open(TRUNCATE)` permission-bypass CVE open
against that version range. `wasm-runtime` is an opt-in Cargo feature
(not in `default = ["surreal-backend"]`), so exposure was already
limited to deployments that explicitly enable it — but the user asked
for the latest version specifically, which settles the
bump-vs-disposition question `plan.md` left open.

## What changed

Bumped `wasmtime`/`wasmtime-wasi` from `41.0.3` to `46` (latest,
resolving `46.0.1`) — satisfies all open alerts, which needed
`wasmtime >= 42.0.2`/`>= 43.0.2` and `wasmtime-wasi >= 44.0.2`.

The bump broke the build in exactly one way: `wasmtime::Error` no
longer implements `std::error::Error` in wasmtime 46, so `anyhow`'s
blanket `Context` impl no longer applies to `Result<T, wasmtime::Error>`
— 6 call sites across `src/uar/runtime/skills/wasm_runtime.rs` (3) and
`src/uar/runtime/wasm/sandbox.rs` (3). wasmtime 46 ships its own
`wasmtime::error::Context` trait for exactly this case; swapped the
import in both files (all `.context()`/`.with_context()` calls in each
file were on wasmtime results, so no disambiguation between the two
`Context` traits was needed).

Also removed `sandbox.rs`'s `engine_config.async_support(true)` call —
deprecated in wasmtime 46 ("no longer has any effect"; async support is
no longer an opt-in `Config` toggle).

## Verification

- `cargo check --features wasm-runtime`: clean (was 6 `E0599` errors
  before the `Context` trait fix).
- `cargo check --tests --features wasm-runtime`: clean.
- `cargo test --lib --features wasm-runtime`: 367/367 green (4 more than
  the feature-off baseline of 363 — the wasmtime-gated test modules).
- `cargo clippy --features wasm-runtime`: zero new warnings at the lines
  this change touched (the import swap, the `async_support` removal);
  pre-existing pedantic warnings elsewhere in both files (unnested
  or-patterns, `map().unwrap_or()`, undocumented unsafe block, redundant
  closure) are unrelated, at different lines, out of scope for this
  disposition-turned-upgrade.

## Note on scope

`plan.md` scoped this as "bump or explicitly document residual risk."
The user asked specifically for the latest version once the compile
errors were fully diagnosed and shown to be a small, well-understood,
2-file fix — so this landed as a real upgrade, not the documented-risk
fallback.
