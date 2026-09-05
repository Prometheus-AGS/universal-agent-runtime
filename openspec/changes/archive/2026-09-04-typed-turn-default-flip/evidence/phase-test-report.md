# New-default phase verification — 2026-09-04

Executed locally after the Typed default, schema, compatibility test, and
test-startup-bound adjustment were implemented:

```sh
cargo check --locked --no-default-features --features server-full && cargo fmt --all -- --check && cargo test --locked --no-default-features --features server-full
```

Overall exit status: **0** (tool session97691). One Cargo invocation at a time.
Tier0 completed in6.13s; formatting passed without output; the test build
completed in25.40s. Selected exact result lines from that run:

```text
test config::tests::harness_defaults_to_typed_and_retains_legacy_rollback ... ok
test result: ok. 710 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 10.91s
9 scenarios (9 passed)
49 steps (49 passed)
test result: ok. 93 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 921.95s
test result: ok. 26 passed; 0 failed; 17 ignored; 0 measured; 0 filtered out; finished in 0.29s
```

The710-test result is the library target,93 is the broad integration target,
and26 is doctests. All other executed targets also passed, including:

| Target | Passed |
| --- | ---: |
| context_history_integrity | 14 |
| mcp_projection | 9 |
| model_path_resiliency | 9 |
| project_instructions | 4 |
| prompt_assembly | 6 |
| settings_persistence | 47 |
| skill_activation_runtime | 7 |
| turn_shadow_parity | 1 |
| typed_turn_assembly | 3 |
| world_state_diff | 3 |

Strict validation separately exited0: `Change 'typed-turn-default-flip' is valid`.
The pre-flip real-provider evidence is in `live-shadow-report.json` and its
command receipt in `README.md`. This run revalidates the checked-in corpus with
the production default set to Typed; it is not another real-provider smoke.

## Failure and correction retained

The first new-default full run exited101 at BDD:8/9 scenarios passed; the
multi-turn scenario exceeded a30-second server-readiness wait before sending
its request. The shared test helper's startup wait is now120s, with an enclosing
child-process wait of180s. The existing health/request assertions and production
timeouts are unchanged. The rerun passed the formerly failing scenario.

## Limits

Ignored tests remain ignored; exit0 does not prove their scenarios. Runtime
output still included the existing local-governance and SurrealKV shutdown
warnings. No warning-free runtime claim is made. Live parity remains a narrow
two-case k3 smoke plus a three-case local corpus; live provider429, remote-peer
parity, and cancellation after emitted child text remain unverified.
