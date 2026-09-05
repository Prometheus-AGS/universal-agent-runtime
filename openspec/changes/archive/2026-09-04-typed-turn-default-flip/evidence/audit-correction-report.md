# Phase-end audit correction verification — 2026-09-04

This supplements the earlier green phase receipt. Independent review found
five untested defects after that run; host-path tests then exposed one related
remote contract routing defect. Historical results are not rewritten.

## Delivered corrections

| Files | Change |
|---|---|
| `src/uar/runtime/thread/service.rs`, `cost_budget.rs` | Release only host-proven never-dispatched remote reservations; retain conservative dispatched accounting. |
| `src/uar/runtime/thread/policy_intersection.rs` | A named remote child uses Agent mode even when its parent is the default UAR router; inherited ceilings remain. |
| `src/llm/orchestrator.rs`, `src/uar/runtime/manager.rs` | Production host-admitted read-only concurrency, ordered confirmation fallback, no duplicate admission charge; metadata-only idle retry until semantic output. |
| `src/uar/runtime/skills/catalog.rs` | Compact titles and suggestions survive pressure; a space separator avoids measured punctuation overhead; omission remains counted. |
| `src/server.rs`, `docs/realtime/chat-replay.md` | Resume the authorized original run, format-tagged frame cursors, projection seeding, no duplicate primary execution; evicted prefix returns410. |

No dependencies, operator pins, workflows, publication or unrelated feature was
added in this correction pass. Runtime guards trace to the audited ownership,
admission, dispatch, retry and replay boundaries.

## Targeted phase-end results

All commands use `--locked --no-default-features --features server-full`.

| Command after `cargo test` | Actual result |
|---|---|
| `--lib uar::runtime::thread::service::tests -- --nocapture` | 3 passed,0 failed;1.48s |
| `--test model_path_resiliency` | 12 passed,0 failed;1.36s |
| `--test skill_activation_runtime --test tool_call_protocol` | Skills8 passed,0 failed;1.50s. Tools11 passed,0 failed;0.77s. |
| `--test integration live::chat_replay_cases -- --nocapture` | 1 passed,0 failed;19.55s; graceful_complete shutdown |

The model target ran as the first target in a combined command that subsequently
failed the initial catalog test. Its12/12 result is a target pass, not an exit0
claim for that initial combined command. The corrected skills/tools command
exited0. All other final commands above exited0.

Observed failures retained: unavailable in-memory fixture backend at compile;
default-root UAR mode copied into an Agent-only child contract;1982/2000 catalog
entries under the token cap with the old colon separator; and provider404 from
a new fixture base URL missing/v1. Each was fixed without disabling a test or
weakening its original acceptance assertion. Independent artifact-only reviews
accepted the final production corrections and strengthened regression assertions.

`openspec validate <change> --strict` passed for fail-closed-tool-arguments,
model-path-resiliency, progressive-skill-runtime and thread-native-subagents.
`git diff --check` passed without diagnostics.

## Full phase gate — passed

```sh
cargo check --locked --no-default-features --features server-full &&
cargo fmt --all -- --check &&
cargo test --locked --no-default-features --features server-full
```

Owned session72074 exited0. T0 passed1.07s, zero compiler warnings; fmt passed;
test build2m26s. Selected actual output:

```text
Library: test result: ok. 713 passed; 0 failed; 1 ignored; finished in 9.08s
BDD: 9 scenarios (9 passed); 49 steps (49 passed)
Broad integration: test result: ok. 94 passed; 0 failed; 1 ignored; finished in 863.63s
MCP projection: 9 passed; 0 failed
Model resiliency: 12 passed; 0 failed
Settings persistence: 47 passed; 0 failed
Skill activation: 8 passed; 0 failed
Tool protocol: 11 passed; 0 failed
Shadow parity: 1 passed; 0 failed
Typed assembly: 3 passed; 0 failed
World-state diff: 3 passed; 0 failed
Doctests: test result: ok. 26 passed; 0 failed; 17 ignored; finished in 0.36s
```

Every executed target passed, including the new replay test inside the broad
matrix and the remote host regressions inside the library target. No second
Cargo writer or test-disabled workaround was used. Canonical revision2298
records10/10 implementation and111/120 overall. Phase-close verification,
archive authorization and reflection are separate from this passing local gate;
nothing was published by the test run.

## Limits

The HTTP fixture runs in the existing loopback-only mode and reports inactive
Cedar/run-policy/risk gates; its owner/tenant route checks are exercised, but
it is not a governance-enabled deployment certification. Existing SurrealKV
shutdown warnings are present. Real remote-peer enforcement/cancellation,
provider-side billing termination, enabled memory/quality replay side effects,
cancelled-run HTTP replay, and the explicitly deferred real-provider429
observation remain outside these passing fixtures. The phase suite has ignored
tests and must disclose its final totals. No archive approval is inferred.
