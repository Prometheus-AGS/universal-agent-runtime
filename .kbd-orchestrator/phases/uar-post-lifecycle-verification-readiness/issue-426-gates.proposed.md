# Issue #426 — proposed local acceptance packet

Status: **PROPOSED, NOT AUTHORIZED, NOT RUN**. The AGENTS.md operator verification lock remains CLOSED. This packet is not permission to run any command. The final implementation inventory and canonical whole-phase state must be reconciled first; exact commands then require explicit operator instruction. No separate source-identity, readiness, import or Docker preflight is implied.

## Finite behavioral gate

```sh
cargo test --locked --no-default-features --features server-full,test-probes --test ingestion_admission --test durable_ingestion --test lifecycle_owned_resources --test run_retention -- --test-threads=1
```

The new ingestion fixture uses embedded SurrealKV and a controlled embedding collaborator. The existing durable_ingestion target also requires its isolated shared SurrealDB fixture configuration, as documented in tests/support/api_key_process.rs. Configure only scoped test endpoints/credentials through its existing environment contract; do not read or change operator application configuration. Absence of a collaborator is BLOCKED/NOT RUN, never PASS. PostgreSQL functional evidence is separate and cannot be inferred from Surreal results.

Record admission/FIFO/queue-bound/oversized cases, actual worker lifetime across dropped waiters and shutdown, durable crash/publication fencing, engine ordering and SSE/owner-retention regressions independently. No sleep-only proof or upstream-only unit test substitutes for the adapter's production path. A failure stops the packet; repair and renew source inventory/approval before rerunning the failed gate.

## Supported-profile and customer artifacts

```sh
cargo build --locked --no-default-features --features minimal --bin universal-agent-runtime
cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
cargo build --locked --no-default-features --features embedded-mobile --lib
cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-apple-ios
cargo build --locked --no-default-features --features embedded-mobile --lib --target aarch64-linux-android
docker build --platform linux/amd64 --tag uar-issue-426:1.3.0 .
```

Run approved commands serially using the existing build target directory and pinned toolchain. Record each produced artifact by path and exact source identity. The existing Dockerfile's production Rust build selects server-full,postgres-backend; its local build above uses the repository root context and produces the named local image without pushing it. Cross-target library commands produce target-specific rlibs and require the existing SDK/toolchain environment; missing prerequisites remain BLOCKED, with no additional preflight command implicitly approved. Builds do not establish mobile runtime, PostgreSQL functional, WASM or ARM-container acceptance. No unrelated platform support claim is made.

This issue does not modify The Boss; no Boss build is introduced. If the active phase scope is expanded to include The Boss, the whole-phase inventory and requested gate packet must additionally include `pnpm build:mac:arm64` as required by AGENTS.md.

## Independent review

Once the operator opens the review boundary, provide a fresh-context artifact-only reviewer with the final diff, exact approved requirements, source identities and actual gate receipts. Rust/code review is pending while CLOSED; it has not been delegated early. Any corrective edit invalidates acceptance for changed source and requires a renewed inventory and operator instruction.

No GitHub Actions tests, paid providers, remote agents, pushes, PR publication, tags or deployment are part of this packet.
