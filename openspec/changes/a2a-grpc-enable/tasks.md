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
      default port 50051, `UAR_SERVER__GRPC_PORT` env override
- [x] 2.2 gRPC server is mounted and serving in `start_server`
      (`src/server.rs`) — binds `grpc_addr`, builds
      `GrpcAgentService::new(a2a_state).into_server()`, serves via
      `serve_with_shutdown`. Not a literal `tokio::try_join!` with the HTTP
      dual-stack listeners (different shapes: one future vs. two), but
      equivalent in spirit — runs concurrently via `tokio::spawn`, and
      `start_server` now awaits the returned `JoinHandle` after the HTTP
      listener(s) finish, so a gRPC panic surfaces (logged) instead of being
      silently dropped.
- [x] 2.3 Graceful shutdown: the gRPC serve future is now
      `serve_with_shutdown(addr, run_cancellation_root.clone().cancelled())`
      — it shares the same root `CancellationToken` the HTTP shutdown path
      uses, so it starts draining at the same instant in-flight runs are
      cancelled. Verified 2026-07-02.

## 3. Verify

- [x] 3.1 tonic-client integration test added:
      `tests/test_a2a_grpc.rs` — starts a real `GrpcAgentService` on an
      ephemeral port, round-trips `MessageSend` → `TaskGet`, and asserts a
      `NotFound` status for an unknown task ID. 2/2 passing.
- [x] 3.2 `cargo test --lib` green: 276/276 (verified 2026-07-02 alongside
      the build fix above); existing A2A JSON-RPC tests unaffected
- [x] 3.3 `CH-01` row added to `tests/integration/live/MATRIX.md`

## 4. Spec + docs

- [x] 4.1 `openspec/specs/a2a-grpc/spec.md` describes transport *behavior*
      (MessageSend/TaskGet/MessageStream scenarios), not the manual-vs-codegen
      implementation choice, so it did not need changing. The plan-vs-reality
      gap (manual builder → proto codegen, new `tonic-prost` dependency) is
      now recorded in `proposal.md`'s "Implementation Note" instead.
- [x] 4.2 Added `UAR_SERVER__GRPC_PORT` row to `docs/configuration.md`

## Notes

Design/specs artifacts are light for this change — it's "finish wiring
already-written code," not new architecture. Draft them via
`/opsx:continue a2a-grpc-enable` if the schema requires them before apply;
otherwise this proposal + tasks are enough to execute.

**2026-07-02 verification pass (part 2):** all tasks above are now complete
— code compiles, 276/276 lib tests + 2/2 new gRPC integration tests green,
shutdown is wired, MATRIX row + docs added. This change is ready for the
artifact-refiner QA gate and archival (`/opsx:verify` → `/opsx:archive`).
