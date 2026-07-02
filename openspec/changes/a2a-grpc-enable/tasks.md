## 1. Compile the parked gRPC service

- [x] 1.1 Uncomment `pub mod grpc;` in `src/uar/api/a2a/mod.rs:23`
- [x] 1.2 `cargo build` green (verified 2026-07-02 on handoff machine after
      fixing an unrelated tonic-prost/prost version mismatch, see main repo
      commit `2e1f153`). **Plan deviation:** implemented via proto codegen
      (`proto/a2a.proto` + `tonic_prost_build::compile_protos` in `build.rs`,
      `tonic::include_proto!("a2a")` in `grpc.rs`) instead of the manual
      service builder this task originally specified — proto codegen was
      judged less boilerplate-prone for the 4 RPC methods. Delta spec (4.1)
      still needs updating to describe the codegen approach.

## 2. Config + mount

- [x] 2.1 Added `grpc_port` to `ServerConfig` (`src/config.rs:218-232`),
      default port, `UAR_SERVER__GRPC_PORT` env override
- [x] 2.2 (partial) gRPC server is mounted and serving in `start_server`
      (`src/server.rs:1056-1072`) — binds `grpc_addr`, builds
      `GrpcAgentService::new(a2a_state).into_server()`. **Not** wired the way
      this task asked: it's a detached `tokio::spawn`, not joined into the
      `tokio::try_join!` used for the HTTP dual-stack listeners, so a gRPC
      bind failure or panic is silent (only `tracing::error!`, no propagation).
- [ ] 2.3 Graceful shutdown: NOT done — the spawned task has no
      `CancellationToken` wiring, so it will not stop on server shutdown.
      Open gap for next session.

## 3. Verify

- [ ] 3.1 tonic-client integration test — not written yet
- [x] 3.2 `cargo test --lib` green: 276/276 (verified 2026-07-02 alongside
      the build fix above); existing A2A JSON-RPC tests unaffected
- [ ] 3.3 `CH-01` row still not added to `tests/integration/live/MATRIX.md`
      (blocked on 3.1)

## 4. Spec + docs

- [ ] 4.1 Delta spec still describes the old "manual service builder" plan —
      needs updating to reflect the proto-codegen approach actually used
- [ ] 4.2 grpc_port not yet documented in deployment docs

## Notes

Design/specs artifacts are light for this change — it's "finish wiring
already-written code," not new architecture. Draft them via
`/opsx:continue a2a-grpc-enable` if the schema requires them before apply;
otherwise this proposal + tasks are enough to execute.

**2026-07-02 verification pass:** code compiles and lib tests are green, but
this change is NOT ready to archive — 2.3 (graceful shutdown) is unimplemented
and 3.1/3.3 (integration test + MATRIX row) are unwritten. Treat as
"code lands, verify-gate open" rather than done.
