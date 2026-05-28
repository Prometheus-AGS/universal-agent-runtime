## Why

UAR currently launches its own copies of helper binaries (MCP servers under `mcp.json`, optionally `surreal-memory-server`, `liter-llm` proxy variants, `forge` MCP enrichment). In a developer environment those binaries are frequently already running (e.g. the user already has surreal-memory-server in Docker on port 23001 and SurrealDB on 28000). Double-spawning wastes resources, fights over ports, and leaves orphaned children when UAR exits.

## What Changes

- Introduce `src/uar/orchestrator/process_supervisor.rs` with `ManagedBinary { name, expected_port, health_url, pidfile }` and a `Supervisor` that:
  1. Probes `expected_port` (TCP connect + optional GET on `health_url`).
  2. If reachable: marks the binary as **adopted**, records the endpoint, **does not spawn**, injects env vars pointing UAR at the adopted endpoint.
  3. If not reachable: spawns, writes a pidfile to `$XDG_RUNTIME_DIR/uar/<name>.pid` (fallback `~/.uar/run/`), takes ownership.
- On shutdown only owned (non-adopted) processes get a SIGTERM.
- Adopted endpoints feed back into `AppConfig` overrides so the rest of UAR is oblivious to the distinction.
- Apply to: MCP stdio/HTTP servers from `mcp.json`, optional `liter-llm` sidecar, optional `forge` enrichment server, optional embedded `surreal-memory-server` when not already running.

## Acceptance

- With `docker compose ps` showing the user's SurrealDB and surreal-memory-server already running, restarting UAR logs `adopted surrealdb at ws://localhost:28000 (existing instance)` and never tries to spawn its own.
- Killing UAR leaves the adopted Docker containers running; orphans only the supervisor-owned processes (none in this scenario).
- Version drift between adopted and would-have-spawned versions is surfaced as a `warn` log with both versions printed.
