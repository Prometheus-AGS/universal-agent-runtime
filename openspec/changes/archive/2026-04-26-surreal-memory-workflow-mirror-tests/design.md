## Context

UAR already exposes long-term memory through `MemoryService` and the in-process memory MCP server. `mcp.json` registers `surreal_memory` against the UAR `/mcp/memory` endpoint, and KBD project metadata declares `.kbd-orchestrator/` as authoritative with Surreal Memory as a secondary mirror.

The missing piece is not the general memory subsystem. The gap is a deterministic workflow mirror contract: project, phase, OpenSpec change, task, waypoint, assessment, plan, blocker, and verification records need a predictable representation, round-trip validation, and conflict behavior that can be used by Codex, Claude Code, Cursor, and OpenCode without replacing file-based KBD state.

## Goals / Non-Goals

**Goals:**

- Define a workflow mirror record shape that can be stored as memory content plus structured metadata.
- Add or document a small adapter/script that reads KBD/OpenSpec state and writes workflow mirror records through `/mcp/memory` or the in-process `MemoryService` boundary.
- Validate create, retrieve, update, and newest-`updated_at` conflict resolution.
- Preserve `source_tool` on every mirrored record so cross-tool writes are auditable.
- Keep `.kbd-orchestrator/` as the source of truth and Surreal Memory as a secondary recovery/query store.
- Produce KBD progress and verification evidence for the mirror path.

**Non-Goals:**

- Do not change the underlying Surreal Memory storage engine.
- Do not make Surreal Memory authoritative over `.kbd-orchestrator/` files.
- Do not require live provider credentials, model calls, or embedding-provider network calls for the validation path.
- Do not redesign the runtime console memory UI in this change.
- Do not introduce a new external service; prefer the existing `/mcp/memory` endpoint or in-process service boundary.

## Decisions

### 1. Represent workflow state as memory records with typed metadata

Mirrored workflow records should use ordinary memory records with deterministic categories and metadata instead of a new storage table. The content should be human-readable, while metadata carries machine fields:

- `workflow_kind`: `project`, `phase`, `openspec_change`, `task`, `waypoint`, `assessment`, `plan`, `blocker`, or `verification_result`
- `workflow_id`: stable identifier for the entity
- `phase`: KBD phase name
- `change`: OpenSpec change name when applicable
- `source_tool`: `codex`, `claude-code`, `cursor`, `opencode`, or another explicit tool id
- `updated_at`: ISO-8601 timestamp used for conflict resolution
- `source_path`: KBD/OpenSpec path that produced the mirror record

Rationale: this keeps the mirror compatible with the existing memory MCP tools and avoids schema migration risk. The alternative was a new workflow-specific SurrealDB table, but that would expand the storage contract before the mirror behavior is proven.

### 2. Use task/global scope for workflow mirror records

Workflow mirror records should use a non-user-private scope, preferably `task` when a phase/change id is available and `global` for project-level state.

Rationale: KBD workflow state is tool/process state rather than user conversational memory. User-scoped memory would make cross-tool recovery brittle and dependent on authentication context.

### 3. Keep file state authoritative with newest-write recovery semantics

The adapter should never overwrite `.kbd-orchestrator/` files automatically. It should be able to produce a recovery/query result that selects the newest mirrored record by `updated_at`, while preserving the original `source_tool`.

Rationale: automatic reverse-sync from memory into files risks corrupting the canonical KBD ledger. The mirror is useful for search, recovery, and cross-tool lookup, not for silent file mutation.

### 4. Prefer in-process tests before endpoint smoke tests

The deterministic test path should start at the `MemoryService` or storage-backed helper boundary so it can run without a live HTTP server. If an MCP endpoint smoke test is practical, add it as a separate targeted check.

Rationale: in-process tests are faster, less flaky, and do not require the frontend or backend server to be running. Endpoint checks can be added once the data contract is covered.

### 5. Emit runtime memory events only as observable evidence

If mirror operations need to appear in runtime UI or replay tests, represent them as `RuntimeMemoryEvent` entities with action `workflow_mirror`. Do not make frontend UI state responsible for mirror persistence.

Rationale: persistence belongs in backend/workflow tooling. The runtime console should observe memory activity, not own workflow synchronization.

### 6. Validate through deterministic fixtures

Tests should use synthetic workflow records for Codex, Claude Code, Cursor, and OpenCode and avoid real API keys or provider calls. Conflict tests should create two records with the same `workflow_kind` and `workflow_id` and assert the newer `updated_at` wins.

Rationale: the mirror is a workflow reliability feature. Its validation must be repeatable in CI and local development without depending on credentials.

## Risks / Trade-offs

- [Risk] Memory creation may invoke embeddings depending on storage configuration. -> Mitigation: use local/default embedding configuration or a storage/test boundary that does not require network-backed providers.
- [Risk] Memory search is semantic and may not be exact enough for conflict tests. -> Mitigation: query by structured metadata or list/filter records in the adapter where available; use semantic search only as an operator convenience.
- [Risk] Multiple tools could write conflicting records with clock skew. -> Mitigation: use explicit `updated_at` from the source workflow record in tests and preserve all candidates for audit rather than deleting older records.
- [Risk] MCP endpoint tests can be flaky if the UAR server is not running. -> Mitigation: keep endpoint smoke checks optional or separately gated; require the in-process round-trip test for acceptance.
- [Risk] Mirrored workflow state could include secrets if raw files are copied. -> Mitigation: mirror only KBD/OpenSpec workflow metadata and summaries, and explicitly exclude provider keys, environment files, and raw secret-bearing logs.

## Migration Plan

1. Add the workflow mirror adapter/script and deterministic fixtures without changing existing memory APIs.
2. Add in-process tests for create, retrieve, update, conflict resolution, and `source_tool` preservation.
3. Optionally add an endpoint smoke test for `/mcp/memory` when the server is available.
4. Record verification evidence in KBD progress.
5. Leave existing KBD files untouched unless a human/tool explicitly chooses to recover from mirrored data.

## Open Questions

- Should the implementation live as a Rust backend helper, a repository script, or both?
- Should endpoint smoke validation be mandatory in CI or documented as an operator check requiring a running UAR server?
- Should mirrored workflow records be visible in the runtime console immediately, or should this change only prove persistence and recovery behavior?
