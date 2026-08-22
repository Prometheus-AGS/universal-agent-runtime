# Post-fix caller cancellation and embedded restart evidence

Profile: `server-full`

## Command

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases::l4_c12_persistence_round_trip \
  -- --exact --test-threads=1
```

## Observed output

```text
test live::capability_cases::l4_c12_persistence_round_trip ... ok
test result: ok. 1 passed; 0 failed; 93 filtered out; finished in 17.63s
```

The harness first cancelled only the caller-owned HTTP token, observed the HTTP listener stop, proved the child remained alive, and proved stderr contained no `deadline_enforced` marker. The parent then delivered SIGTERM. The full child future, including the A2A transport, exited with code 0 and wrote `resources-released`. While that original helper process remained alive at a pre-exit barrier, a second UAR became ready on the same SurrealKV path and read the exact persisted resource. This proves the embedded lock was released by UAR lifecycle completion, not by process exit.

The separate registered-cleanup ordering command also passed:

```text
cargo test --locked --no-default-features --features server-full --lib shutdown_process_registered_cleanup_precedes_graceful_completion -- --test-threads=1 --nocapture
test result: ok. 1 passed; 0 failed; 615 filtered out; finished inside the focused process suite
```

The equivalent caller-cancellation control followed by SIGINT also passed:

```text
UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked --no-default-features --features server-full --test integration live::harness::tests::caller_owned_http_cancellation_remains_nonterminating_before_sigint -- --exact --test-threads=1 --nocapture
test result: ok. 1 passed; 0 failed; 93 filtered out; finished in 8.65s
```

## Paired negative control

```bash
UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH=1 \
  UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked --no-default-features --features server-full \
  --test integration live::capability_cases::l4_c12_persistence_round_trip \
  -- --exact --test-threads=1
```

Observed exit `101` at the intended final assertion:

```text
assertion `left == right` failed: C-12: resource did not survive the cold restart:
Knowledge base '<id>' not found
  left: 404
 right: 200
test result: FAILED. 0 passed; 1 failed; 93 filtered out; finished in 18.67s
```

Both runs emitted a SurrealKV teardown warning after the nested helper runtime
ended: `No runtime available for closing the store correctly`. That warning is
not treated as clean output. The stronger external assertion still passed: the
second UAR acquired the same path and became ready before the original process
was permitted to exit.
