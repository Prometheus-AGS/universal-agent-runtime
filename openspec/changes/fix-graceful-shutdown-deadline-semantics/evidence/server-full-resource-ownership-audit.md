# Server-full resource ownership audit

Date: 2026-08-22

Profile: `server-full`

Purpose: determine whether OpenSpec task 3.5 can be satisfied without changing
the approved child write surface or weakening a cleanup guarantee.

## SQLx

Exact-package negative query:

```bash
cargo tree --locked --no-default-features --features server-full \
  -e normal -i sqlx@0.8.6
```

Observed output and status:

```text
error: package ID specification `sqlx@0.8.6` did not match any packages

help: a package with a similar name exists: `shlex`
exit=101
```

`server-full` does not activate the optional `postgres-backend` feature, so no
SQLx pool exists in the certified profile. A SQLx pool-release test cannot
exercise a configured `server-full` resource.

Full-graph command:

```bash
set -o pipefail
cargo tree --locked --no-default-features --features server-full -e normal \
  | rg '(^|[[:space:]├└│])sqlx v'
```

Observed output and status:

```text
<no output>
pipeline_exit=1
```

The full resolved normal-dependency graph contains no SQLx package at any
version.

## Redis

Command:

```bash
cargo tree --locked --no-default-features --features server-full \
  -e normal -i redis@1.2.1
```

Observed output and status:

```text
redis v1.2.1
└── opendal-service-redis v0.57.0
    └── opendal v0.57.0
        └── liter-llm v1.12.0
            └── universal-agent-runtime v1.0.0
exit=0
```

Command:

```bash
rg -n -i 'redis' src --glob '*.rs'
```

Observed output:

```text
src/config.rs:37:    /// Enable external cache (Redis)
src/uar/compiler/registries.rs:5://! (Redis, etcd, external schema registries) can be plugged in by implementing
```

Redis is present only through a transitive OpenDAL service selected by
`liter-llm`. UAR's server composition owns no Redis client or connection, so it
has no Redis closure operation to invoke or test.

Configuration-to-composition commands:

```bash
rg -n 'external_cache_enabled' src --glob '*.rs'
rg -n 'redis::|RedisClient|Client::open.*redis' src --glob '*.rs'
rg -n -i 'external_cache_enabled|redis' src/server.rs
```

Observed results:

```text
external_cache_enabled appears only in src/config.rs,
src/uar/settings/manager.rs, and one API response fixture.

No redis::, RedisClient, or Redis Client::open call exists in src/. exit=1
No external_cache_enabled or redis reference exists in src/server.rs. exit=1
```

The toggle is represented in configuration and settings metadata but is not
composed into the runtime. The only resolved Redis package belongs to
`opendal-service-redis` behind `liter-llm`, not to a UAR-owned server resource.

## MCP

UAR does own configured MCP `RunningService` instances in
`src/mcp/registry.rs`. The installed `rmcp` 2.2.0 source exposes explicit
`close`, `close_with_timeout`, and consuming `cancel` operations. Its `Drop`
implementation only triggers asynchronous cancellation and explicitly says to
call `close()` or `cancel()` for guaranteed cleanup.

Relevant installed-source command:

```bash
rg -n 'pub async fn (close|cancel)|impl.*Drop for RunningService' \
  ~/.cargo/registry/src/index.crates.io-*/rmcp-2.2.0/src/service.rs
```

Observed definitions:

```text
773:    pub async fn close(&mut self) -> Result<QuitReason, tokio::task::JoinError> {
793:    pub async fn close_with_timeout(
818:    pub async fn cancel(mut self) -> Result<QuitReason, tokio::task::JoinError> {
825:impl<R: ServiceRole, S: Service<R>> Drop for RunningService<R, S> {
```

Context7 independently returned the same lifecycle contract from the current
`rmcp::service::RunningService` documentation: explicit close/cancel waits for
background transport cleanup; drop cancellation alone is not the guaranteed
path.

## Resolved stop condition

The operator approved widening `scope.json` to include
`src/mcp/registry.rs`. The authorized implementation is a crate-private
registry shutdown operation plus focused proof that a stdio child observes
transport closure before normal UAR process completion.

The corrected OpenSpec requirement and task name the active SurrealDB and MCP
resources and record SQLx and Redis as `server-full` profile exclusions. This
preserves the active cleanup guarantees without claiming tests for resources
that the certified profile does not configure or own.

The SurrealDB release proof keeps the original helper process alive after its
dedicated UAR server thread and Tokio runtime have joined. A second UAR process
must become ready on the identical SurrealKV path before the parent permits the
first helper process to exit. This proves resource release before process exit
rather than relying on process teardown.
