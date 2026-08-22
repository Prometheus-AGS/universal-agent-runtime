# Baseline process-test negative control

Profile: `server-full`
Source SHA before product behavior change: `32afa53d510c8b840b3e98b2be9d9f5dee149531`

## Command

```bash
cargo test --locked --no-default-features --features server-full --lib shutdown_process_ -- --test-threads=1 --nocapture
```

## Observed output

```text
running 7 tests
test server::tests::shutdown_process_child ... ok
test server::tests::shutdown_process_held_registered_cleanup_exits_at_deadline ... FAILED
  held cleanup child did not enforce its deadline
test server::tests::shutdown_process_held_sse_exits_at_deadline ... FAILED
  held SSE child did not enforce its deadline
test server::tests::shutdown_process_idle_sigterm_and_sigint_exit_within_one_second ... FAILED
  idle TERM did not exit within one second
test server::tests::shutdown_process_real_sse_completes_and_both_listeners_refuse ... FAILED
  primary listener accepted connections after SIGTERM
test server::tests::shutdown_process_stderr_backpressure_does_not_block_deadline ... FAILED
  backpressured stderr blocked forced exit
test server::tests::shutdown_process_stderr_lock_does_not_block_deadline ... FAILED
  ordinary stderr lock blocked forced exit

test result: FAILED. 1 passed; 6 failed; 0 ignored; 612 filtered out; finished in 9.69s
```

The passing entry is the environment-gated child fixture invoked without its child variable; it intentionally returns without serving. All six behavioral parent cases failed against the baseline behavior. The fixture uses real child processes and real SIGTERM/SIGINT delivery. The SSE cases issue a real HTTP `text/event-stream` request, and the held-cleanup case blocks an actual registered cleanup callback.
