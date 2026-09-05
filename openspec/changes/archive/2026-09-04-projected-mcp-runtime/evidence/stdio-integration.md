# Projected MCP real-stdio integration evidence

Date: 2026-09-04

The integration uses `tests/fixtures/mcp_stdio_server.py` as a real child
process over newline-delimited stdio MCP. The test asserts four process starts
and four matching process stops across initial discovery, lazy reconnect on the
first tool call, cancelled initialization, successful retry, and runtime
shutdown.

## Tier 0

Command:

```text
cargo check --locked --no-default-features --features server-full
```

Observed output:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.86s
```

## Focused real-stdio integration

Command:

```text
cargo test --locked --no-default-features --features server-full --test mcp_projection real_stdio_server_covers_lazy_reconnect_cancel_and_shutdown -- --nocapture
```

Observed output:

```text
running 1 test
test real_stdio_server_covers_lazy_reconnect_cancel_and_shutdown ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.50s
```

The uncomfortable limitation is that this local integration requires
`python3` on the captured `PATH`. The runtime itself remains language-neutral;
Python is only the protocol fixture used to make child lifecycle receipts
observable.
