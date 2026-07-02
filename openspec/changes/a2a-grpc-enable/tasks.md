## 1. Compile the parked gRPC service

- [ ] 1.1 Uncomment `pub mod grpc;` in `src/uar/api/a2a/mod.rs:23`
- [ ] 1.2 `cargo build` and fix the tonic-0.14 API errors surfaced in
      `src/uar/api/a2a/grpc.rs` (service trait signatures, `tonic::Request`/
      `Response` wrapping, any streaming/codec type changes). No proto
      codegen — it's a manual service builder.

## 2. Config + mount

- [ ] 2.1 Add `grpc_port` to `ServerConfig` (`src/config.rs`) with a default
      (e.g. 50051) and `UAR_SERVER__GRPC_PORT` env override
- [ ] 2.2 Uncomment + wire the mount in `start_server`
      (`src/server.rs:1055-1064`): bind the grpc addr, build
      `GrpcAgentService::new(a2a_state).into_server()`, serve concurrently
      with the HTTP listener (mirror the dual-listener `tokio::try_join!`
      pattern already used for the IPv4/IPv6 companion)
- [ ] 2.3 Graceful shutdown: fold the gRPC serve future into the existing
      `CancellationToken` shutdown path

## 3. Verify

- [ ] 3.1 tonic-client integration test: start the server on an ephemeral
      grpc port, connect a tonic client, round-trip one A2A method
      (SendMessage or GetTask), assert the response
- [ ] 3.2 `cargo test` green; existing A2A JSON-RPC tests still pass
      (no regression to the HTTP binding)
- [ ] 3.3 Add the `CH-01` row to `tests/integration/live/MATRIX.md`
      (gRPC task round-trip) per plan A2.3

## 4. Spec + docs

- [ ] 4.1 Update the `a2a-grpc` delta spec: transport is now
      compiled/exported/mounted/tested (was defined-but-disabled)
- [ ] 4.2 Note the grpc_port in deployment docs

## Notes

Design/specs artifacts are light for this change — it's "finish wiring
already-written code," not new architecture. Draft them via
`/opsx:continue a2a-grpc-enable` if the schema requires them before apply;
otherwise this proposal + tasks are enough to execute.
