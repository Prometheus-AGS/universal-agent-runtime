## 1. Child-local collector preparation

- [x] 1.1 Copy the accepted restricted collector into the active KBD child and implement the exact direct-Text or zero-length-Ping→masked-Pong→Text state machine. Preserve the 15-second non-resetting sign-in-exchange deadline, 30-second outer cap, one-Ping/two-frame cardinality, exact literal credential gate and terminal cleanup. Verify completion with `git diff --check`, a syntax-only check, and a source-review receipt enumerating every terminal path and buffer-clearing call; do not run behavioral tests yet.

## 2. Single operational observation

- [x] 2.1 After explicit KBD Execute authorization, reconfirm source/executable/process/listener/target/configuration/authentication continuity and make at most one upgrade plus one sign-in attempt. Send at most one masked empty Pong only for the exact accepted Ping, stop at sign-in success/error or the first stop condition, and never retry. Verify completion with a parseable, content-free `evidence/signin-observation.json` recording exact attempted/skipped counts, frame/Pong/sign-in cardinality, deadlines, sizes and sanitized outcome.

## 3. Offline classification

- [x] 3.1 Produce `classification.md` and `execution.md` mapping the observation to the exact state-machine result and every enforced boundary. Verify the expected RPC identifier and sign-in success/error only in bounded memory, record no credential/frame/payload/response content, and state that sign-in does not prove database-readiness cause or recovery. Verify completion with format/JSON/privacy/size checks only; defer all behavioral tests to Task 4.

## 4. Phase-boundary acceptance

- [x] 4.1 At the end of the child Execute phase, run the full diagnostic-only synthetic suite for direct Text, empty Ping→Pong→Text, exact masked-Pong bytes, every named unsupported frame class, partial/stalled writes, non-resetting deadline, buffered-first-frame handling, write-timeout behavior, interrupt cleanup, buffer clearing, evidence privacy, 64-KiB size and cardinality limits. Run syntax/static checks and `openspec validate handle-readiness-diagnostic-zero-length-ping --strict`; obtain fresh-context artifact-only review of the source digest, actual test output, sanitized evidence and classification; deliver `verification.md` and `handoff-out.md`, resolve critical findings or retain a blocked/qualified result, and run KBD status without running product tests.

Task order is strictly 1.1 → 2.1 → 3.1 → 4.1. A prerequisite failure permits an explicitly skipped operational attempt and an inconclusive deliverable, never a fallback client, guessed credential, second connection, broader frame handler or enlarged deadline. No tool workflow code changes, so cross-harness validation is not applicable. Run KBD status after every completed task, change and phase.
