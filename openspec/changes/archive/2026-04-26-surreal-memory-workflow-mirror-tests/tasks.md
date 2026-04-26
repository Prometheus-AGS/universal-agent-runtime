## 1. Workflow Bootstrap

- [x] 1.1 Confirm `mcp.json` exposes the `surreal_memory` MCP entry for the UAR `/mcp/memory` endpoint.
- [x] 1.2 Confirm `.kbd-orchestrator/` remains the documented source of truth for workflow state.
- [x] 1.3 Identify or add shared workflow fixture data for Codex, Claude Code, Cursor, and OpenCode writes.
- [x] 1.4 Document the exact local validation commands for mirror round-trip and runtime replay evidence.

## 2. Workflow Mirror Contract

- [x] 2.1 Add a workflow mirror record type or helper that normalizes `workflow_kind`, `workflow_id`, `phase`, `change`, `source_tool`, `updated_at`, and `source_path` metadata.
- [x] 2.2 Add validation for the supported workflow kinds: project, phase, openspec_change, task, waypoint, assessment, plan, blocker, and verification_result.
- [x] 2.3 Add a deterministic content format for mirrored workflow records that excludes provider keys, environment files, and raw secret-bearing logs.
- [x] 2.4 Add recovery selection logic that chooses the newest record by `updated_at` for matching `workflow_kind` and `workflow_id`.
- [x] 2.5 Preserve all candidate records during conflict evaluation unless an explicit deletion operation is requested.

## 3. Memory Integration

- [x] 3.1 Implement a small adapter or script that reads KBD/OpenSpec workflow metadata and writes mirror records through the memory service or `/mcp/memory` boundary.
- [x] 3.2 Ensure mirror writes do not mutate `.kbd-orchestrator/` files.
- [x] 3.3 Ensure recovery/query operations return candidate state without automatically overwriting authoritative workflow files.
- [x] 3.4 Add support for task/global memory scope selection for workflow mirror records.
- [x] 3.5 Add structured logging or result output that includes source tool, workflow id, selected record id, and conflict resolution outcome.

## 4. Deterministic Mirror Tests

- [x] 4.1 Add in-process test coverage for creating and retrieving a mirrored workflow record.
- [x] 4.2 Add in-process test coverage for updating mirrored workflow metadata and content.
- [x] 4.3 Add conflict-resolution test coverage proving newest `updated_at` wins for the same `workflow_kind` and `workflow_id`.
- [x] 4.4 Add test coverage proving the winning record preserves `source_tool`.
- [x] 4.5 Add test coverage proving older conflict candidates remain available for audit.
- [x] 4.6 Ensure mirror tests run without live LLM provider credentials, live model calls, or external provider network access.
- [x] 4.7 Add optional endpoint smoke validation for `/mcp/memory` if a running UAR server is available.

## 5. Runtime Replay And Entity Sync

- [x] 5.1 Extend runtime replay fixtures with a memory event whose action is `workflow_mirror`.
- [x] 5.2 Include workflow mirror metadata in the replayed memory event payload.
- [x] 5.3 Add unit coverage proving replay ingestion creates a runtime memory event entity for `workflow_mirror`.
- [x] 5.4 Add visual or integration coverage proving the runtime console exposes workflow mirror memory activity without refresh.
- [x] 5.5 Verify the frontend does not write authoritative workflow mirror records directly to Surreal Memory.

## 6. Cross-Tool Workflow Evidence

- [x] 6.1 Validate mirror fixture records for Codex, Claude Code, Cursor, and OpenCode.
- [x] 6.2 Record verification evidence in `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json`.
- [x] 6.3 Add or update a phase verification report for `surreal-memory-workflow-mirror-tests`.
- [x] 6.4 Run `openspec validate surreal-memory-workflow-mirror-tests --strict`.
- [x] 6.5 Run the targeted mirror tests and runtime replay tests.
- [x] 6.6 Run `git diff --check`.
