# C07 production integration gate

Date: 2026-09-28. Repository: `Prometheus-AGS/universal-agent-runtime`. The gate ran in the `codex/agent-fabric-convergence-c07` worktree on macOS Apple Silicon against the pinned SurrealDB Rust SDK 3.3.0 and a real UAR child process with persistent SurrealKV.

Command: `cargo test --test agent_fabric_c07_gate --features afc-c07-gate -- --test-threads=1`

Final result: **2 passed, 0 failed**, 127.87 seconds. One case is the child-process harness helper; the other is `live::observer_cases::c07_observers_complete_live_boundary`. The latter exercised two independent subscriptions, source and conversation exclusions, C06 monitor command receipts, pause and restart catch-up, revocation before dispatch, separate committed occurrences despite a one-event C06 UI ring, and outbox pruning after 1,026 actual C06 source cancellations. It confirmed an exact retention gap and operator-only acknowledgement.

Two earlier gate attempts failed on gate setup/assertion timing rather than production code: the restart initially used a different HTTP port from the endpoint-pinned C06 deployment binding, and an assertion read the second committed restart occurrence before the observer's next scheduler cycle. The final gate retained the same port and waited for both occurrences. An earlier compiler attempt stopped before tests and its four compile errors were fixed before the production gate ran. These failures are not counted as passing evidence.

Scope limits: this gate did not inject a dead-letter failure or an uncertain external effect. C07 exposes finite retry/dead-letter state, but those paths are supported by source inspection rather than this gate. The ordinary-run compatibility path was exercised by the preceding C06 phase gate, not repeated here. This gate is local macOS/SurrealKV evidence, not installed Windows x64 or Mac Apple Silicon release acceptance.
