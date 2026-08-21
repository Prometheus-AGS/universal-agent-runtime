## Context

See `proposal.md` for motivation and `specs/mcp-reconnect-recovery/spec.md` for observable behavior.

`McpRegistry` currently stores `Arc<DynClientService>` values inside an outer shared map. `filtered` and `merge` copy those values into new maps so each view can retain a distinct server/tool policy. When a call fails, reconnect inserts a new value into only the calling view's map. `RunManager` creates a new filtered view for the next request from `global_mcp`, so that request clones the original dead service value.

The correction must preserve the existing authorization boundary, avoid replaying a possibly completed operation, introduce no dependency, and never hold a synchronous lock across an asynchronous MCP call.

## Goals / Non-Goals

**Goals:**

- Give each configured server one replaceable service identity shared by every registry view that already contains that server.
- Keep each view's server map, server configuration, tool index, tool descriptions, and native tools independently filtered.
- Make reconnect replacement atomic at the service-pointer boundary and observable by later requests.
- Prove the real subprocess sequence and streamed failure events locally.

**Non-Goals:**

- Retrying or replaying the failed tool operation.
- Changing the 30-second call timeout, MCP protocol, public API, tool names, provider routing, or UI.
- Adding reconnect backoff, process pooling, a background supervisor, or GitHub Actions testing.
- Solving simultaneous reconnect storms beyond ensuring that any completed replacement leaves a valid service pointer for later calls.

## Decisions

### Use a shared replaceable slot per server

The service map value becomes a private shared slot equivalent to `Arc<RwLock<Arc<DynClientService>>>`. The outer registry map remains independently copied for `filtered` and `merge`; cloning a permitted entry clones its slot, not merely the service pointer it currently contains.

This is preferred over sharing the entire service map because a whole-map share would allow a filtered view to observe later-added or explicitly excluded servers. It is preferred over updating both a child view and a remembered parent because merged registries can have multiple origins and parent propagation would be incomplete and brittle.

### Clone the current service pointer before awaiting

Call lookup acquires the slot's read lock only long enough to clone its current `Arc<DynClientService>`, then releases the lock before `call_tool(...).await`. Reconnect creates the replacement without holding the slot lock and acquires a write lock only to swap the pointer.

This preserves synchronous `std::sync::RwLock` use for tiny pointer operations and prevents blocking unrelated work for the duration of a tool call or process handshake. Replacing the lock with an async lock would add scheduling overhead without providing a benefit for these non-awaiting critical sections.

### Preserve slot identity during upsert and registry projection

Initial connection creates a new slot. `filtered` clones only allowed slots; `merge` preserves the current collision behavior while carrying the selected slot; reconnect swaps the existing slot. An upsert for an existing server updates that server's existing slot so previously authorized views do not retain a dead handle, while its configuration and advertised-tool projections continue to follow the existing registry update behavior.

Removing a server deletes it from the target registry's map and indexes. Existing already-created views may still own the slot they were previously authorized to use; this is unchanged snapshot-view behavior and is not broadened by reconnect.

### Separate event evidence from final assistant text

The local installed-artifact certifier requests streaming responses for the destructive MCP calls. It requires exactly one `success: false` tool-result entry for crash and timeout, then validates a fixture-side JSONL trace with modes `echo, crash, echo, timeout, echo` and process-identifier transitions. Final assistant text and HTTP 200 status are not accepted as proof of tool success or failure.

This is preferred over non-streaming response inspection because non-streaming chat output intentionally retains the assistant's final text rather than the normalized tool-result event.

### Test structure

Focused Rust tests exercise slot identity across independently created filtered and merged views, pointer replacement, and continued exclusion of denied servers/tools. The local installed-artifact certifier remains the authoritative process-boundary proof because it covers the packaged binary, request-to-request registry reconstruction, actual subprocess exit, the full timeout, normalized streamed events, and process identifiers in one boundary test.

Negative controls mutate a failed event into success and duplicate a fixture trace entry; both must make the evidence validator exit nonzero.

## Risks / Trade-offs

- [A synchronous slot lock is poisoned by a panic] → Recover the inner guard with the repository's existing poison-handling convention; never panic merely because a prior holder panicked.
- [Two failing calls reconnect concurrently] → Each failure remains non-replayed and each reconnect swaps only a service pointer; the last successful replacement becomes current. Reconnect serialization is deferred because it is not required by the observed sequential defect.
- [Sharing replacement state widens authorization] → Share only slots already present in a view; retain independent filtered maps and assert denied server/tool absence after replacement.
- [An old authorized view survives server removal] → This remains the existing snapshot-view behavior. Runtime policy creates a new filtered view for each run, so subsequent requests observe removal; changing revocation semantics is out of scope.
- [A repaired source invalidates prior release evidence] → Commit the child first, build a new immutable candidate, and rerun operational-resilience certification locally from the new source SHA.

## Migration Plan

1. Land the private registry representation change and focused tests with no public or persisted data migration.
2. Build a new candidate from the committed source and run the short local installed-artifact preflight.
3. If the preflight fails, retain the prior source tag as non-certifying evidence and correct the child before resuming the parent.
4. On success, reflect and archive this child change, then restart the parent operational-resilience certification from the new immutable source.

Rollback is a source revert before candidate promotion. No stored state or external protocol migration is required.
