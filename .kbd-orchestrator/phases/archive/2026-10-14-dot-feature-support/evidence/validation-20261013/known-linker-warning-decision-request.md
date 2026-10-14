# Dot candidate linker-warning decision

The completed phase implementation builds with zero Rust warnings. Candidate attempt06 completed Cargo with exit0 in2m07s against authorized inventory07; its only remaining diagnostic is Apple ld reporting that __eh_frame exceeds the16MiB offset range of the compact-unwind table, with possible exception-performance impact. The raw log and source-bound observation are retained alongside this request.

Proposed operator exception: allow the planned serial local database and runtime integration gates to proceed using this candidate while retaining this exact linker diagnostic visibly. Keep all Rust and other linker warnings subject to the zero-warning rule. Do not change panic behavior, unwind flags, compiler warning levels, tests, assertions, or source fingerprints. This exception does not certify exception performance, waive runtime scenarios, authorize paid resources, or grant final release acceptance.

The uncomfortable thing: the compiler and native linker successfully created a binary, but that does not prove panic recovery or exception performance. Existing planned functional and recovery gates remain necessary; no false-positive conclusion is claimed. Upstream has an unresolved report of the same diagnostic: https://github.com/rust-lang/rust/issues/159105 .

Policy requiring the decision: .claude/rules/rust.md, Zero warnings. Fix immediately, or annotate #[expect(lint, reason=...)]. An attempted scoped expectation was reported as unfulfilled and was removed completely; no suppression was substituted. Database qualification and all subsequent gates remain unstarted pending this decision.
