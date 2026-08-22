# Verification — `fix-graceful-shutdown-deadline-semantics`

Date: 2026-08-22

Profile scope: local UAR `server-full` only. These results transfer to no other
runtime profile, platform, provider, or deployment. The parent phase's full
10,800-second certification is intentionally not represented by this child
verification; it must restart from zero on the committed immutable candidate.

| requirement | assertion observed | negative control observed | command | result |
|---|---|---|---|---|
| Server handles SIGTERM gracefully. | Real child processes observed immediate HTTP drain for SIGTERM/SIGINT, normal idle and completed-SSE exit 0 within 1 second, both listeners refusing after signal, and held SSE/cleanup forced exit 0 within the 1-second post-deadline tolerance. Locked and backpressured stderr did not extend the bound. Caller-owned HTTP cancellation left the host alive and unarmed before later SIGTERM/SIGINT. A healthy non-root UID-65532 container with a real held SSE exited 0 after 30,489 ms under a 30-second UAR/35-second Docker boundary, emitted only `deadline_enforced`, terminated curl with exit 18, and produced a Docker `die` event without SIGKILL. | Against baseline SHA `32afa53d510c8b840b3e98b2be9d9f5dee149531`, the same process command observed 6 intended failures: held cleanup and held SSE did not enforce the deadline, idle shutdown exceeded 1 second, the primary listener still accepted connections, and stderr lock/backpressure blocked exit. The immutable candidate's earlier operational run separately recorded Docker SIGKILL and exit 137. | `cargo test --locked --no-default-features --features server-full --lib shutdown_process_ -- --test-threads=1`; baseline control adds `--nocapture`; container control runs a release image from source-only control commit `a9b50220995d11d4cbb944e00cc3ed2274f355ae`, holds `/api/uar/sync/stream`, then runs `docker stop --time 35`. | Positive: exit 0, 9 passed, 0 failed, 615 filtered, 6.62s. Container: Docker health `healthy`, UID 65532, elapsed 30,489 ms, curl 18, UAR exit 0, one deadline marker, zero graceful markers, one `die`, zero SIGKILL. Negative: exit nonzero, 1 passed and 6 failed in 9.69s at the intended behavioral assertions. |
| Resource cleanup on shutdown. | MCP shutdown cancelled shared transports, waited for a real stdio child's EOF, blocked reconnect and new upsert, and began before held ingestion-shaped cleanup completed. Live-query shutdown cancelled and joined retained topic supervisors idempotently. Composition tests joined ingestion/A2A work and released SurrealKV ownership before `resources-released`; a second UAR became ready on the same database path while the original helper remained alive. The server-full graph excludes SQLx/Postgres and contains no UAR-owned Redis client. Deadline fixtures emitted neither graceful nor cleanup-complete evidence when cleanup remained held. | `UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH=1` changed only the restart path; the same C-12 assertion then failed 404-versus-200, exit 101, proving the positive same-path result depended on release of the actual resource. A separate pre-exit baseline probe failed to acquire the same SurrealKV path while its owning lifecycle remained active. | `cargo test --locked --no-default-features --features server-full --lib mcp::registry::tests:: -- --test-threads=1`; `cargo test --locked --no-default-features --features server-full --lib uar::realtime::surreal_bus::tests::shutdown_is_idempotent_and_joins_topic_supervisors -- --exact --test-threads=1`; `UAR_LIVE_INTEGRATION_BACKEND=recorded cargo test --locked --no-default-features --features server-full --test integration live::capability_cases::l4_c12_persistence_round_trip -- --exact --test-threads=1`; rerun the last command with `UAR_L4_NEGATIVE_CONTROL_DIFFERENT_PATH=1`. | MCP: exit 0, 4 passed, 0 failed, 1.60s. Live query: exit 0, 1 passed, 0 failed, 0.17s. Same-path C-12: exit 0, 1 passed, 0 failed, 17.63s. Different-path control: exit 101, 0 passed, 1 failed at the intended 404-versus-200 assertion, 18.67s. Both C-12 runs disclosed the SurrealKV teardown warning; the external same-path readiness assertion still passed before original-process exit. |

## Local gates

- `cargo check --locked --no-default-features --features server-full` exited
  0. It reported three pre-existing warnings; this change does not claim a
  warning-free build.
- `cargo clippy --locked -p universal-agent-runtime --no-default-features
  --features server-full --lib --no-deps` exited 0 and reported the existing
  pedantic warning inventory.
- `bash -n scripts/certify-release-candidate.sh` exited 0.
- `openspec validate fix-graceful-shutdown-deadline-semantics --strict
  --no-interactive` exited 0 with `Change
  'fix-graceful-shutdown-deadline-semantics' is valid`.
- The scoped diff check emitted no error. `Cargo.toml` and `Cargo.lock` have no
  diff. Added Rust visibility contains only two `pub(crate)` shutdown methods;
  no public API was introduced.

Detailed commands and observed outputs are retained under `evidence/`. The
artifact-refiner converged after its three primary schemas, five progressive
checkpoints, five blocking constraint identities, manifest reference,
finalized registry, and 13-file active/history byte identity all passed.
