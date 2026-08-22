# Corrected process-test evidence

Profile: `server-full`
Base SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`
Product/test/script patch SHA-256: `01cdad66d261dec3910a26ce399d9160807246a9ac2386cf3c4bed29a3905dcd`

## Command

```bash
cargo test --locked --no-default-features --features server-full --lib shutdown_process_ -- --test-threads=1
```

## Observed output

```text
running 9 tests
test server::tests::shutdown_process_child ... ok
test server::tests::shutdown_process_held_registered_cleanup_exits_at_deadline ... ok
test server::tests::shutdown_process_held_sse_exits_at_deadline ... ok
test server::tests::shutdown_process_idle_sigterm_and_sigint_exit_within_one_second ... ok
test server::tests::shutdown_process_mcp_eof_precedes_held_cleanup_deadline ... ok
test server::tests::shutdown_process_real_sse_completes_and_both_listeners_refuse ... ok
test server::tests::shutdown_process_registered_cleanup_precedes_graceful_completion ... ok
test server::tests::shutdown_process_stderr_backpressure_does_not_block_deadline ... ok
test server::tests::shutdown_process_stderr_lock_does_not_block_deadline ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 615 filtered out; finished in 6.62s
```

The child fixtures use real OS signals and process exit observation. The active and held request fixtures are actual `text/event-stream` responses. The held-cleanup fixture blocks a registered cleanup callback. The stderr controls either hold Rust's ordinary stderr lock or fill the captured pipe; both still exit inside the one-second post-deadline tolerance.

The MCP/held-cleanup case runs an actual stdio MCP child and observes its EOF
marker while the ingestion-shaped blocking cleanup remains held. The process
then reaches the one-second internal deadline, exits 0, emits
`deadline_enforced`, and emits neither `graceful_complete` nor a cleanup-complete
marker.
