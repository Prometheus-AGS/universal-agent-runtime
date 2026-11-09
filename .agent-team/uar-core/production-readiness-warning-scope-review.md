# Warning repair scope review

Phase: production-readiness-476-baseline, phase-exit source review.
Scope: changes relative to `8e0602eecbd37504e8cdf8945f2f1c5537ff6701` in `src/bin/uar-sidecar/inherited_input.rs` and `tests/support/approval_recovery_process.rs`, plus the current warning-repair spec delta. No actionable findings.

## Source evidence

- `inherited_input.rs:79-82` replaces only deprecated `AsyncFd::new(self.file)` with supported unsafe `AsyncFd::register(self.file)` and documents the descriptor invariant. The private `Input` owns a `File`; `into_async` transfers it into the private `AsyncInput` wrapper. Its read method uses `get_ref`, and its close method drops the wrapper. No code replaces the registered file or closes its descriptor independently. Existing nonblocking configuration and read loop are unchanged. All Windows branches are unchanged.
- The actual cached Tokio 1.54.1 source, `src/io/async_fd.rs:226-239`, requires an open, stable descriptor for the registration lifetime. `register` configures the same readable/writable interest as the deprecated wrapper. At lines 911-912 the old `new` implementation is exactly `unsafe { Ok(Self::register(inner)?) }`. Thus the new caller preserves the old result conversion. Lines 1482-1485 convert `AsyncFdRegisterError<T>` by returning its `cause`, dropping the remaining owned `inner`; lines 304-310 and 978-981 deregister before dropping the owned file. The new safety comment accurately describes these paths.
- `approval_recovery_process.rs:3-8` places a reasoned `#[expect(dead_code)]` on this target's inclusion of the shared resume helper module. It does not alter the shared source, crate-wide lint policy, test entrypoint, assertions, scenarios, or runtime control flow. The helper exposes a separate `qualify` and readback entrypoint while approval recovery calls its own `process::qualify` and `process::run_child`. The annotation is target-local and lint-specific, not blanket warning suppression.
- The current `rust-test-baseline` spec delta explicitly preserves sole ownership, nonblocking/read behavior, error propagation, and Windows behavior; the two source changes match that narrow requirement. It also requires a warning-free compile gate, which source inspection cannot establish.

## Limits and command classification

All commands executed for this review were source inspection (`cat`, `sed`, `rg`, and scoped `git diff`) or creation of this review document. No tests, lint, format, typecheck, Cargo, rustc, compiler probe, build, packaging check, `git diff --check`, external mutation, or indirect gate ran. Runtime behavior and warning-free compilation remain unverified here. No production edits, guards, or unrequested changes were added by this reviewer.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: APPROVE — source scope only; no execution acceptance claimed.
