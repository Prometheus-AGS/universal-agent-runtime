## Why

The A2A (Agent2Agent) gRPC transport is fully written but **disabled**: the
manual tonic service exists (`src/uar/api/a2a/grpc.rs`, 358 lines) but its
module is commented out (`src/uar/api/a2a/mod.rs:23` → `// pub mod grpc;`) and
the server mount is commented out (`src/server.rs:1055-1064`). So UAR ships A2A
JSON-RPC only; the gRPC binding — the second wire binding the LF A2A v1.0 spec
normatively defines — is unreachable. This blocks gRPC-based multi-agent
deployments and the LibreFang integration story (CH-18 depends on it).
Dependencies are already in place (`tonic = "0.14"`, `tonic-build = "0.14"`,
`prost = "0.13"`).

## What Changes

- Export the gRPC module: `pub mod grpc;` in `src/uar/api/a2a/mod.rs`.
- Make `grpc.rs` compile under tonic 0.14 (the code was written against an
  earlier API and parked when 0.14 changed things — fix the surfaced errors;
  it's a manual service builder, no codegen required).
- Mount the gRPC service in `start_server` (`src/server.rs:1055-1064`,
  currently commented): bind `config.server.grpc_port`, add
  `GrpcAgentService::into_server()`, serve alongside the HTTP listener.
- Add a `grpc_port` field to `ServerConfig` (default e.g. 50051) if not present.
- Add a tonic-client integration test that starts the gRPC server on an
  ephemeral port and round-trips at least one A2A method (SendMessage /
  GetTask), proving the transport actually serves.
- **Deferred (optional):** re-enable proto auto-generation in `build.rs`
  (`tonic_build::compile_protos`) — NOT required, since the service is
  hand-written; only do this if the proto and the manual types drift.

## Capabilities

- **Modified Capabilities:**
  - `a2a-grpc` (existing spec at `openspec/specs/a2a-grpc/`) — the gRPC
    transport moves from defined-but-disabled to compiled, exported, mounted,
    and integration-tested.

## Impact

- **Affected code:** `src/uar/api/a2a/mod.rs` (export), `src/uar/api/a2a/grpc.rs`
  (tonic 0.14 fixes), `src/server.rs` (mount), `src/config.rs` (grpc_port),
  new `tests/` gRPC client test.
- **Dependencies:** none new — tonic/prost already present.
- **Live matrix:** per plan A2.3, add a `CH-01` row to
  `tests/integration/live/MATRIX.md` (gRPC task round-trip) when this lands.
- **Risk:** the manual `grpc.rs` may need non-trivial tonic-0.14 API updates
  (service trait signatures, `Request`/`Response` wrapping, streaming types) —
  surfaced only at compile time; iterate against `cargo build`.

## Implementation Note (2026-07-02, post-landing)

The actual implementation deviated from "no codegen required" above: it adds
`proto/a2a.proto` and generates the service/message types via
`tonic_prost_build::compile_protos` in `build.rs` (`tonic::include_proto!("a2a")`
in `grpc.rs`), rather than hand-writing the tonic service against the
pre-existing manual types. This added a new runtime dependency,
`tonic-prost = "0.14"` (the codegen output needs the `tonic-prost` crate, not
just the `tonic-prost-build` build-dependency already present) — the
"Dependencies: none new" line above no longer holds. Landing this also
required bumping `prost` from `"0.13"` to `"0.14"` to match the version
`tonic-prost` 0.14 pulls in (two incompatible `prost::Message` traits in the
graph otherwise). See `tasks.md` for what shipped vs. what remains open
(shutdown wiring on the mount is now done as of the 2026-07-02 verification
pass; spec/deployment-docs update is still open).
