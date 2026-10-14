# Proposal

## Why

Legacy HTTP, tools and background memory paths need one complete ownership contract. This change supplies the D3 isolation acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Apply the verified principal namespace to write/search/direct-ID/update/delete/history and context retrieval through HTTP, MCP/native and background paths. Reuse MemoryService and existing index. Migrate only proven owner mappings, quarantine ambiguous legacy memory and ensure retrieved text never carries authority.

**BREAKING for unsafe legacy behavior:** callers can no longer rely on subject-only ownership, volatile production authority or successful no-op durability; migration and explicit unsupported outcomes are required.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `multi-tenant-isolation`: Assistant memory is scoped by verified complete owner. Preserve the full existing per-user isolation requirement under MODIFIED and add the new memory boundary requirement.

## Impact

Code ownership: `src/uar/memory/service.rs`, `src/uar/memory/scopes.rs`, `src/uar/memory/mcp_server.rs`, `src/uar/api/memory.rs`, `src/uar/tools/memory.rs`, `src/uar/memory/context_builder.rs`. Dependencies: dot-02-principal-credentials. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

