# Resource cleanup controls

Date: 2026-08-22

Profile: `server-full` only. These results transfer to no other profile.

Base SHA: `32afa53d510c8b840b3e98b2be9d9f5dee149531`

Product/test/script patch SHA-256:
`01cdad66d261dec3910a26ce399d9160807246a9ac2386cf3c4bed29a3905dcd`

## MCP transport ownership

Command:

```bash
cargo test --locked --no-default-features --features server-full \
  --lib mcp::registry::tests:: -- --test-threads=1
```

Observed output, exit `0`:

```text
running 4 tests
test mcp::registry::tests::reconnect_replacement_is_shared_without_widening_filtered_views ... ok
test mcp::registry::tests::shutdown_blocks_new_server_upsert ... ok
test mcp::registry::tests::shutdown_waits_for_stdio_eof_and_blocks_filtered_view_reconnect ... ok
test mcp::registry::tests::upsert_reconnect_uses_new_config_in_an_existing_filtered_view ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 620 filtered out; finished in 1.60s
```

The fixture is a real Python stdio child. Shutdown cancels the transport, waits
for the child to observe stdin EOF, prevents a pre-existing filtered view from
reconnecting, and rejects a newly configured server after terminal shutdown.
The process-suite MCP case separately proves this cancellation starts before a
held blocking cleanup reaches the deadline.

## SurrealDB live-query ownership

Command:

```bash
cargo test --locked --no-default-features --features server-full \
  --lib uar::realtime::surreal_bus::tests::shutdown_is_idempotent_and_joins_topic_supervisors \
  -- --exact --test-threads=1
```

Observed output, exit `0`:

```text
running 1 test
test uar::realtime::surreal_bus::tests::shutdown_is_idempotent_and_joins_topic_supervisors ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 623 filtered out; finished in 0.17s
```

The bus runs against a temporary SurrealKV database. Its terminal operation is
idempotent, cancels every topic supervisor, and joins every retained task.

## Composition-level ingestion and A2A completion

The C-12 process helper writes `resources-released` only after the real
`start_server_sidecar` future returns. Reaching that barrier requires both the
A2A gRPC task join and the retained ingestion-watcher abort/join. The positive
same-path C-12 result and its wrong-path negative control are recorded in
`post-fix-caller-cancellation.md`.

SQLx and Redis exclusions are recorded separately in
`server-full-resource-ownership-audit.md`.
