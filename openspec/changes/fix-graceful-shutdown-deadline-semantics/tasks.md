## 1. Workflow and Pre-Execution Gates

- [x] 1.1 Validate the corrected planning artifacts with `openspec validate fix-graceful-shutdown-deadline-semantics --strict` and resolve every reported error before Execute.
- [x] 1.2 Submit the corrected proposal, delta specification, design, and task list as new artifact-only packets to the independent critic and judge; save both verdicts under the child phase and resolve every blocking finding before Execute.
- [x] 1.3 Register `fix-graceful-shutdown-deadline-semantics` under the active child and enter its Execute stage through `prometheus kbd`; verify canonical status names this change and no other change as active.

## 2. Behavioral Negative Controls

- [x] 2.1 Preserve the immutable baseline command `scripts/certify-operational-resilience-local.sh certify` and its source-SHA-bound evidence showing Docker SIGKILL, exit 137, and the idle 30-second mandatory wait.
- [x] 2.2 Add real child-process tests for idle SIGTERM/SIGINT, active SSE completion, post-signal primary and companion refusal, held-SSE and held-cleanup deadline enforcement, ordinary-stderr-lock/backpressure tolerance, and synchronous outcome markers; run them against test-only changes on baseline behavior and record the behavioral failures before changing shutdown implementation.
- [x] 2.3 Extend the integration process harness assertion so caller-owned HTTP cancellation proves the child remains alive with no process deadline marker until a later OS signal; record the pre-fix result without running the phase-level live suite.

## 3. Runtime Shutdown Coordination

- [x] 3.1 Add a crate-private shutdown coordinator with process-started, cleanup-complete, and process-complete state; verify its focused normal-completion tests pass.
- [x] 3.2 On SIGINT/SIGTERM, cancel run work and both HTTP listeners before awaiting blocking cleanup, remove the Axum pre-drain sleep, and verify `cargo check --locked --no-default-features --features server-full` passes after the edit.
- [x] 3.3 Arm a standard-library watchdog thread at signal observation, independent of Tokio, and verify a focused blocked-async-work test observes deadline expiry on schedule.
- [x] 3.4 On watchdog expiry, make one non-blocking emergency write of `UAR_SHUTDOWN outcome=deadline_enforced` without acquiring the ordinary stderr lock, retain an independent hard-stop fuse, and exit 0; on normal completion emit only `graceful_complete`, and verify both marker branches plus locked/backpressured stderr controls in child-process tests.
- [x] 3.5 Mark normal process completion only after HTTP listeners, ingestion cleanup, ingestion-watcher join, live-query supervisor join, explicit MCP transport closure, A2A shutdown, and observed SurrealKV lock release finish; verify named normal-path tests for ingestion ordering, A2A completion, a second UAR becoming ready on the same SurrealKV path while the original helper remains alive at a pre-exit barrier, and MCP child stdin closure pass; prove MCP cancellation starts while held ingestion cleanup is still blocked, and preserve feature-graph evidence that SQLx and Redis are outside the `server-full` claim.
- [x] 3.6 Preserve caller-owned sidecar cancellation as HTTP-only and non-terminating, then prove subsequent SIGTERM and SIGINT still initiate process-scoped shutdown.

## 4. Focused Process Verification

- [x] 4.1 Verify idle SIGTERM and idle SIGINT each exit normally with code 0 within one second and emit `graceful_complete` without `deadline_enforced`.
- [x] 4.2 Verify a real `text/event-stream` SSE response completes inside the graceful window, both primary and companion listeners reject new connections after signal observation, and the child exits normally with code 0 without `deadline_enforced`.
- [x] 4.3 Verify a deliberately held SSE stream remains active until the short internal deadline, then its connection closes and the child exits 0 within one second without any parent/external kill; require `deadline_enforced` when stderr is writable and forbid graceful/cleanup-complete markers.
- [x] 4.4 Verify caller-owned HTTP cancellation alone leaves the process alive and unarmed, then a later OS signal exits normally and releases the embedded persistence lock.
- [x] 4.5 Verify deliberately held registered cleanup reaches the same bounded forced exit and never emits graceful or cleanup-complete evidence.
- [x] 4.6 Verify an ordinary stderr lock and a backpressured stderr pipe cannot delay forced exit beyond the one-second tolerance.

## 5. Container Certification Boundary

- [x] 5.1 Configure the local non-root container journey with an explicit 30-second UAR graceful deadline and a 35-second Docker stop deadline while retaining UID, writable persistence, health, and exit-code-zero assertions; verify `bash -n scripts/certify-release-candidate.sh` passes.
- [x] 5.2 Add a held-work request to the non-root journey so the runtime reaches its internal deadline, and record internal/external limits, monotonic elapsed time, terminated request, outcome marker, and exit code in `non-root-container.json`.
- [x] 5.3 Capture Docker event evidence for the held-work journey and verify UAR exits 0 before escalation with no SIGKILL event; do not use an idle container exit to claim the 30/35-second margin.

## 6. Local Verification and Evidence

- [x] 6.1 Run `cargo check --locked --no-default-features --features server-full` and `cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps`; record actual local outputs without a cross-profile claim.
- [x] 6.2 Run only the focused shutdown coordinator, idle/active/held-work process, sidecar, and named ingestion/A2A/SurrealDB/MCP cleanup tests locally; record the SQLx/Redis profile exclusions, commands, outputs, elapsed limits, outcome markers, and source SHA.
- [x] 6.3 Run `openspec validate fix-graceful-shutdown-deadline-semantics --strict`, the repository-required artifact-refiner validation path, `git diff --check`, `git diff --exit-code -- Cargo.toml Cargo.lock`, and an added-Rust-visibility diff inspection proving no public API was introduced; resolve every blocking result and record actual output.
- [x] 6.4 Write `verification.md` in the contract row format, pairing every bounded or fail-closed shutdown assertion with its observed negative control and reporting only `server-full`.
- [x] 6.5 Mark tasks complete only after their exact command output or artifact exists and all stated evidence passes.

## 7. Child Completion and Parent Handoff

- [x] 7.1 Complete the canonical change and child Execute stage through `prometheus kbd`, commit the bounded child surface as one change commit, and verify unrelated user changes remain unstaged and unmodified.
- [ ] 7.2 Reflect and close the child, restore the parent release denominator and certification tasks through the canonical runtime, and verify exact next work is fresh `certify-operational-resilience` on the new immutable candidate SHA.
- [ ] 7.3 Restart `scripts/certify-operational-resilience-local.sh certify` from zero in a clean detached checkout and verify all deterministic, 10,800-second soak, backup/restore, native restart, cleanup, and non-root held-work evidence belongs to the corrected SHA before supply-chain certification.
