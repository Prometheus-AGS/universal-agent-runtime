<!-- mirror of openspec/changes/projected-mcp-runtime/tasks.md -->
# Tasks — projected-mcp-runtime

scope: src/mcp/**, src/uar/runtime/manager.rs (MCP block), src/uar/api/mcp_admin.rs, src/uar/telemetry/metrics.rs, src/uar/runtime/native_skills/search_tools.rs, src/sandbox/os_native/** (conditional), tests/mcp_projection.rs

## 0. Decision

- [ ] 0.1 Evaluate porting Codex `sandboxing` (Seatbelt, Landlock, bwrap) for stdio children; record the decision and, if not ported, implement rejection of `sandboxed: true` at config load

## 1. Failing tests first

- [ ] 1.1 `tests/mcp_projection.rs`: two consecutive runs using the same skill-declared server reuse one connection; a config-hash change forces a new one
- [ ] 1.2 A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds
- [ ] 1.3 A required server that fails to start aborts preflight with an error naming the server; an optional one warns and its tools are absent
- [ ] 1.4 With 200 deferred tools, the eager set is bounded; `search_tools("calendar")` activates matching descriptors for the next step
- [ ] 1.5 A child scope cannot replace a global server definition or loosen its sandbox policy
- [ ] 1.6 Concurrent refreshes of one server produce one connection attempt; a cancelled refresh leaves the entry dirty and the next call refreshes
- [ ] 1.7 State transitions emit normalized events in order and `set_mcp_server_status` receives each
- [ ] 1.8 `sandboxed: true` either launches under the sandbox or fails config load; it is never a no-op

## 2. Catalog and projection

- [ ] 2.1 Add `src/mcp/catalog.rs` (`McpCatalog`, `ServerDefinition` with authority and config hash)
- [ ] 2.2 Add `src/mcp/projection.rs`: resolve a step to exact servers and tools; authority ordering enforced
- [ ] 2.3 Add `src/mcp/binding_cache.rs`: key by owner, hash, auth identity, environment; generation invalidation; single-flight

## 3. Lifecycle

- [ ] 3.1 Lazy start for skill-contributed and child-only servers gated on cached catalog completeness; `wait_until_ready` on the call path
- [ ] 3.2 Required versus optional failure handling in preflight
- [ ] 3.3 State machine and normalized state events; wire the existing MCP-status recorder
- [ ] 3.4 `search_tools` model-only tool and `Deferred`/`Hidden` exposure in the tool list

## 4. Manager integration

- [ ] 4.1 Replace per-run `from_config` with catalog resolution and binding lookup
- [ ] 4.2 Preserve reconnect generation guard, timeouts, and shutdown guarantees; extend existing tests

## 5. Verification

- [ ] 5.1 Tier 1: `cargo test --locked --no-default-features --features server-full --test mcp_projection`
- [ ] 5.2 Tier 2: fmt check and full test run
- [ ] 5.3 Local integration with a real stdio server: lazy start, reconnect, cancel, shutdown (record commands and output)
- [ ] 5.4 `openspec validate projected-mcp-runtime --strict`
