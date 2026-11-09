# Warning repair security and scope review

Phase: `production-readiness-476-baseline`, phase-exit source review only.
Verdict: **No actionable security or scope findings in the reviewed delta.**

## Scope and source identity

Reviewed the working-tree diff against `8e0602eecbd37504e8cdf8945f2f1c5537ff6701` for:

- `src/bin/uar-sidecar/inherited_input.rs` (diff index `ec605815..fe56f7d4`).
- `tests/support/approval_recovery_process.rs` (diff index `fa0b9c12..238a96d6`).
- `openspec/changes/restore-exact-tree-rust-baseline/specs/rust-test-baseline/spec.md` (diff index `30de417e..e5437cd0`).

Read surrounding sidecar startup and shared-fixture source only where needed to assess these changes. Did not read writer handoffs or generation history. No conclusion is offered about the remaining cohort or whole-phase acceptance.

## Evidence

1. `inherited_input.rs:65-79` keeps the existing `F_GETFL`/`F_SETFL` nonblocking setup and its immediate I/O error propagation. Only the deprecated registration call changes, with a concrete ownership justification at the unsafe boundary. `Input` owns a `std::fs::File`, consumes itself during transfer, and `AsyncInput` retains that file inside its private `AsyncFd` field. Its read implementation obtains only a shared file reference, does not replace or close the descriptor, and its Unix close path drops the wrapper. The startup call at `src/bin/uar-sidecar.rs:327` runs under the enabled Tokio runtime shown at lines 308-316.
2. `Cargo.lock` pins Tokio `1.54.1`. Read that exact installed registry source at `/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.54.1/src/io/async_fd.rs`: `register` at lines 211-242 requires a valid open stable descriptor for the registration lifetime; `register_with_interest` at lines 267-284 retains the input on either success or failure; `take_inner` at lines 303-311 deregisters before returning the owned input; `Drop` at lines 978-982 calls it; conversion at lines 1482-1486 returns `cause`, dropping the remaining owned input. The deprecated `new` at lines 903-914 already delegates to the same registration and conversion. Upstream rationale is linked by that source to https://github.com/tokio-rs/tokio/pull/8587. The new call therefore exposes and justifies the existing unsafe contract without adding a new descriptor owner or error fallback.
3. The diff leaves the entire Windows implementation, Unix read/readiness loop, token ingestion, and cleanup behavior unchanged. It adds no authority, credential handling, logging, dependency, or external action.
4. `approval_recovery_process.rs:3-8` adds a reasoned `expect(dead_code)` only to this target's shared `resume` module inclusion. The including target imports the shared process helpers and has its own `qualify` function at line 1288; the included source also defines its separate qualification/readback functions. The expectation does not alter fixture code, guards, assertions, feature conditions, or execution. It is neither a crate-wide allowance nor a compiler flag.
5. The spec delta records supported registration, ownership, nonblocking/read semantics, error propagation, unchanged Windows behavior, and warning-free compile acceptance. These requirements fit the actual production change; the existing fixture-construction requirement already covers the scoped expectation.

## Limits and unverified state

The uncomfortable limit is that source equivalence does not prove warning-free compilation or operating-system behavior. Those claims remain unverified on this candidate. No tests, Cargo/rustc commands, compiler probes, lint/format checks, builds, package checks, security scans, policy validators, or `git diff --check` ran. The review used only `pwd`, `rg`, `cat`, `sed`, `nl`, `head`, and read-only `git diff` source inspection; an attempted read of `src/bin/uar-sidecar/main.rs` reported that the file did not exist, after which the actual `src/bin/uar-sidecar.rs` was inspected.

Only this report was authored. No production or test source was edited, no guards were added, and no unrelated work was introduced. This artifact is a scoped source-review result, not gate authorization or phase acceptance. The operator verification lock remains in force for every command outside the explicitly authorized source review.
