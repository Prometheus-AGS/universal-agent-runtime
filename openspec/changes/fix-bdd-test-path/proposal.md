# fix-bdd-test-path

## Why

`cargo check --test bdd` failed: `couldn't read
tests/live/integration/live/harness.rs`. Pre-existing, unrelated to any
tracked change in the prior phase, discovered alongside
`fix-uar-integration-test`.

## Root cause

`tests/bdd.rs` declared:

```rust
mod live {
    #[path = "integration/live/harness.rs"]
    pub mod harness;
    #[path = "integration/live/stub_llm.rs"]
    pub mod stub_llm;
}
```

Rust resolves a nested `#[path]` relative to the *module directory*
rustc computes for the enclosing `mod live` — and since `mod live` had
no `#[path]` of its own, that directory defaults to `tests/live/` (the
convention for an out-of-line `mod live;`), even though `live` is
actually declared inline in this file. The nested paths therefore
resolved to `tests/live/integration/live/harness.rs`, which doesn't
exist.

## What changed

Moved the `integration/live` prefix onto the outer `mod live` itself:

```rust
#[path = "integration/live"]
mod live {
    #[path = "harness.rs"]
    pub mod harness;
    #[path = "stub_llm.rs"]
    pub mod stub_llm;
}
```

Now `live`'s own directory is `tests/integration/live/`, and the child
`#[path]`s resolve relative to *that*.

## Verification

- `cargo check --test bdd`: clean (`Finished` profile). Only pre-existing,
  harmless `unused_imports`/`dead_code` warnings inside
  `tests/integration/live/harness.rs`'s and `stub_llm.rs`'s own internal
  `#[cfg(test)] mod tests` blocks — those items are used by the main
  `tests/integration.rs` binary's own test suite, just not by this
  narrower `bdd` binary target; out of scope for this path-only fix.
