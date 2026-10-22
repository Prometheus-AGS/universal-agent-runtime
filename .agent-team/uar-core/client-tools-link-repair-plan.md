# Client tools macOS link repair proposal

Phase: `uar-post-lifecycle-verification-readiness`, Execute.
Verification gate: **CLOSED**. This handoff records source inspection only.

## Observed failure

The lead supplied the once-authorized command and result:

`CARGO_TARGET_DIR=target/uar-post-lifecycle-main cargo build --locked --no-default-features --features server-full,postgres-backend --bin universal-agent-runtime`

Exit 101 at linking after source compilation; linker diagnostic:
`ld: unknown options: -no_warn_eh_frame_too_large`.
This teammate did not rerun the command or independently inspect its full output.

## Source diagnosis

`build.rs:9-17` contains a comment describing a macOS debug test-image warning
suppression, followed by a condition on target OS and debug profile only. Its
emitted argument is `cargo:rustc-link-arg=-Wl,-no_warn_eh_frame_too_large`.
There is no test-only selection in that block. The observed binary link rejects
the exact option emitted there. The source and supplied diagnostic identify a
concrete repair without a compiler or linker probe.

## Minimal proposed production edit

Delete the entire comment and conditional flag-emission block at `build.rs:9-17`.
Retain the two preceding rerun directives and the following protocol-generation,
catalog-copy, and stub-model code. Do not change Cargo dependencies, profiles,
target directories, unwind settings, or other linker arguments.

Do not replace the option with an inferred alternative, move the same rejected
option to test links, or add linker-version detection without new evidence.
The uncomfortable consequence is that the diagnostic this block intended to
suppress may reappear on a later applicable test link. That test-link behavior
is unverified; removing a rejected suppression does not establish that the full
binary link will succeed or that no other errors remain.

## Execution constraints and proposed gate

No production edit was made. The lead assigned this report only because the
required pre/post GitHub Actions policy validator is not currently authorized
for the build-script edit. Keep the verification gate closed. Resolve that
authorization boundary before editing `build.rs`; do not silently skip either
the policy rule or the operator verification lock.

After the authorized production edit, reconcile the whole-phase inventory and
seek explicit operator instruction for the exact applicable policy-validator
commands and the failed production build command above. No other check,
preflight, linker probe, format command, or test is authorized by this report.
Successful source compilation in the prior command earns no link acceptance.

## Handoff

- Owned file changed: `.agent-team/uar-core/client-tools-link-repair-plan.md` only.
- Source state inspected: current `build.rs` includes the failing flag block;
  waypoint source revision 157 / plan revision 91 reports 144 of 166
  implementation items complete and explicitly says verification CLOSED.
- Unresolved decision: operator authorization needed for the required validator
  boundary and subsequent exact gate commands.
- Added guards: none. No runtime behavior or production source changed.
- Verification run by this teammate: none. All repair effects remain unverified.
- Other writers share the checkout; their changes were preserved.
