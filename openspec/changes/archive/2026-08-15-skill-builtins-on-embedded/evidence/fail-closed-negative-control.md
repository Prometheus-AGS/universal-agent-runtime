# B3 fail-closed negative control

Date: 2026-08-14
Profile scope: `server-full` only. This evidence transfers to no other profile.

The retained positive `src/embedded.rs` diff hashed to
`7e3832efea6157e77889adabca3a973012b9627b56936b2fe1c8f4eb2de6c7f3`
before inversion. The four-line `self.seed_defaults` registration block was
then removed with `apply_patch` while the discovery import was deliberately
left in place.

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features --features server-full --test builtin_db_registration embedded_runtime_seeds_persists_and_deduplicates_builtins -- --exact --test-threads=1
```

Observed failing output (exit 101):

```text
running 1 test
test embedded_runtime_seeds_persists_and_deduplicates_builtins ... FAILED

failures:

---- embedded_runtime_seeds_persists_and_deduplicates_builtins stdout ----

thread 'embedded_runtime_seeds_persists_and_deduplicates_builtins' (2114432) panicked at tests/builtin_db_registration.rs:398:9:
B3 seed child failed
stdout:

running 1 test
test embedded_runtime_seeds_persists_and_deduplicates_builtins ... FAILED

failures:

---- embedded_runtime_seeds_persists_and_deduplicates_builtins stdout ----

thread 'embedded_runtime_seeds_persists_and_deduplicates_builtins' (2114434) panicked at tests/builtin_db_registration.rs:94:9:
assertion `left == right` failed: fresh embedded registry must contain builtin builtin::learn-practice exactly once
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    embedded_runtime_seeds_persists_and_deduplicates_builtins

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.22s


stderr:

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    embedded_runtime_seeds_persists_and_deduplicates_builtins

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.30s

error: test failed, to rerun pass `-p universal-agent-runtime --test builtin_db_registration`
```

The registration block was restored with `apply_patch`. The restored diff hash
was exactly
`7e3832efea6157e77889adabca3a973012b9627b56936b2fe1c8f4eb2de6c7f3`.
The same command then produced exit 0:

```text
running 1 test
test embedded_runtime_seeds_persists_and_deduplicates_builtins ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.91s
```
