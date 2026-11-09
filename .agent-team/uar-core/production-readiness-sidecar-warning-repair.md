# Sidecar inherited-input warning repair handoff

Phase: `production-readiness-476-baseline`, Execute. Verification lock: **CLOSED**.
Lead-supplied source base: `8e0602eecbd37504e8cdf8945f2f1c5537ff6701`.
Other writers share the checkout. Only the owned source file and this handoff were
changed; no teammate edits, dependencies or configuration were modified.

## Changes and observed cause

- `src/bin/uar-sidecar/inherited_input.rs`: replace the single Unix
  `tokio::io::unix::AsyncFd::new(self.file)?` with
  `unsafe { tokio::io::unix::AsyncFd::register(self.file) }?`, accompanied by the
  local ownership/error/drop safety justification. No surrounding formatting,
  nonblocking flag handling, readiness/read loop or Windows branch was changed.
- `.agent-team/uar-core/production-readiness-sidecar-warning-repair.md`: this
  required handoff.

The observed deprecation at inherited_input.rs:76 is preserved in
`target/production-readiness-476/approval-dev-compile-003.log`: the safe constructor
is deprecated for missing I/O safety and names `register` as its replacement.
No deprecation expectation/allow was added. Upstream rationale identified directly
by that source diagnostic: https://github.com/tokio-rs/tokio/pull/8587 .

## Pinned API and safety reasoning

`Cargo.lock:12806` resolves Tokio1.54.1. Inspected exact local dependency source:
`/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.54.1/src/io/async_fd.rs`.

- Lines224–240: `pub unsafe fn register(inner: T) -> Result<Self,
  AsyncFdRegisterError<T>>`. Descriptor must remain valid/open and identify the
  same file description, with stable `as_raw_fd`, for the AsyncFd lifetime.
- Lines239–241: registration retains READABLE|WRITABLE interests. This is the same
  registration performed by the deprecated constructor at lines908–912; the
  change does not narrow or expand readiness behavior.
- Lines267–287: success owns `inner` inside AsyncFd; failure owns it in the error.
- Lines310–317 and978–982: `take_inner` deregisters using the still-owned file
  before returning/dropping the inner; Drop calls this path.
- Lines1452–1493: the error's `From<AsyncFdRegisterError<T>> for io::Error` returns
  its original cause and drops the remaining original object. The existing `?`
  thus preserves the previous error cause and releases the File on failure.

The existing `Input::open` clones process stdin into its own independently owned
descriptor and converts that owner to a concrete `std::fs::File`. `into_async`
consumes `self`; successful fcntl calls establish nonblocking mode before moving
that File into registration. The borrowed numeric descriptor does not acquire
ownership. No descriptor replacement, raw close or mutable File exposure occurs
in this module's private `AsyncInput` wrapper; reads borrow its owned File through
readiness guards. Moving the wrapper does not change the File's descriptor.
Closing the separate original stdin descriptor does not close this owned clone.
The new unsafe call states these existing invariants rather than bypassing a
borrow-checker error. `run_sidecar` calls `into_async` inside its Tokio runtime,
preserving the existing current-reactor prerequisite and panic behavior.

## Work performed and verification limits

Every command was classified as source inspection (`cat`, `tail`, `head`, `sed`,
`rg`, plain `git diff -- src/bin/uar-sidecar/inherited_input.rs`) or authorized
implementation/documentation writing (`apply_patch`). Consulted current waypoint,
amended cohort plan, AGENTS, gotchas, Rust rules, workspace routing, Rust best
practices and async-patterns guidance plus pinned API/error/drop implementation.
`rust-router` and `unsafe-checker` were not found in the searched installed roots;
no behavior from an unavailable skill was invented.

No tests/checks, Cargo/rustc, formatter, lint, build, compiler probe,
`git diff --check`, independent review gate, external mutation or paid call was
run or delegated. No source/config dependency version changed. No new guard or
unrequested feature was introduced. No unresolved implementation decision remains
within this assignment.

Compilation, warning elimination, formatting, Unix runtime behavior and Windows
behavior remain unverified. The explicit unsafe contract is supported by current
ownership/source reasoning, not a runtime safety certificate. Lead must collect
both cohort handoffs, record the renewed exact-source inventory and obtain the
applicable gate authorization before independent review and serial acceptance.
