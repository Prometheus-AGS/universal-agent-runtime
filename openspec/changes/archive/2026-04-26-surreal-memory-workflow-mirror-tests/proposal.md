## Why

KBD workflow files are the source of truth, but the phase cannot close until the Surreal Memory MCP mirror is proven as a reliable secondary recovery and query path. This matters now because Codex, Claude Code, Cursor, and OpenCode are expected to share workflow state while the runtime console continues to expose memory activity as live operational state.

## What Changes

- Add a deterministic workflow mirror validation path for KBD entities written through the UAR `/mcp/memory` endpoint or the equivalent in-process memory service boundary.
- Cover workflow entity types: project, phase, OpenSpec change, task, waypoint, assessment, plan, blocker, and verification result.
- Verify create, retrieve, update, and conflict-resolution behavior for mirrored workflow records.
- Enforce newest-`updated_at` conflict resolution while preserving `source_tool` audit metadata from Codex, Claude Code, Cursor, and OpenCode.
- Document or implement a small workflow mirror adapter/script that treats `.kbd-orchestrator/` files as authoritative and Surreal Memory as secondary recovery/query storage.
- Add validation evidence in KBD progress and verification artifacts.
- No breaking API changes are intended.

## Capabilities

### New Capabilities

- `surreal-memory-workflow-mirror`: KBD workflow state can be mirrored to and recovered from Surreal Memory MCP with deterministic round-trip and conflict-resolution validation.

### Modified Capabilities

- `runtime-event-replay-entity-sync`: Runtime memory events and workflow mirror activity remain observable as runtime/entity graph state when mirror operations are replayed or validated.

## Impact

- Affected backend/runtime areas may include `src/uar/memory/service.rs`, `src/uar/memory/mcp_server.rs`, `src/uar/api/memory.rs`, `src/uar/api/memory_admin.rs`, and focused tests or scripts for workflow mirror operations.
- Affected workflow files include `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json`, `.kbd-orchestrator/current-waypoint.*`, and the phase verification report.
- Runtime UX impact: operators get clearer confidence that memory activity shown in the runtime console can include workflow mirror events and can be recovered if a tool session is interrupted.
- Provider compatibility impact: none directly; this change must not require provider credentials or live model calls.
- Realtime state impact: mirror validation should produce deterministic memory/workflow events that can be surfaced or replayed without changing the source-of-truth KBD file model.
- KBD workflow state must be updated as this change advances, with `.kbd-orchestrator/` remaining authoritative and Surreal Memory remaining a secondary mirror.
