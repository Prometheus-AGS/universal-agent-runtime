# Caller-owned HTTP cancellation preservation control

Profile: `server-full`
Product behavior: pre-fix baseline

## Command

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases::l4_c12_persistence_round_trip \
  -- --exact --test-threads=1 --nocapture
```

This was one focused integration test, not the phase-level live suite.

## Observed result

```text
test live::capability_cases::l4_c12_persistence_round_trip ... ok
test result: ok. 1 passed; 0 failed; 92 filtered out; finished in 31.87s
```

Before the parent delivered SIGTERM, the updated harness observed all of the following:

- caller-owned cancellation stopped the HTTP listener;
- the child process remained alive;
- captured stderr did not contain `UAR_SHUTDOWN outcome=deadline_enforced`;
- the later parent-delivered SIGTERM completed process-scoped shutdown with exit code 0;
- a fresh child reopened the same SurrealKV path and read the identical persisted resource, proving the process boundary released the embedded database lock.

The optional `surreal_memory` readiness and absent `kreuzberg` warnings were observed during startup but did not fail or weaken these assertions.
