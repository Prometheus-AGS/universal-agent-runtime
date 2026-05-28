## Why

`AgentMemorySection` grabs `patchAgent` directly off `useAgentsAdminStore`. After this change, the store is gone, so the patch logic must move to the page (or a page-scope helper) and apply optimistic graph patches directly. The pattern mirrors the Provider migration: snapshot → optimistic merge → call service → rollback on error.

`handleDelete` already calls `services/agents-api.ts::deleteAgent` directly today; we just add optimistic graph patching + rollback to it.

## What Changes

- Page-scope `patchAgent(agentId, body)` helper (or inline pattern inside `AgentMemorySection`):
  1. Snapshot the current entity from `useGraphStore.getState().entities["Agent"][agentId]`.
  2. Optimistically `upsertEntity("Agent", agentId, { ...snapshot, ...body })`.
  3. Call `services/agents-api.ts::patchAgent`.
  4. On rejection: re-upsert snapshot + surface error.
- `handleDelete`:
  1. Snapshot the entity.
  2. Optimistically `removeEntity("Agent", id)`.
  3. Call `deleteAgent` service.
  4. On rejection: re-upsert snapshot + surface error.
- Local `useState` already in place for `deleting`/`deleteError`; add `error` for the patch path.

## Acceptance

- Memory toggle in `AgentMemorySection` flips instantly; SSE reconciles ≤200 ms.
- Forced patch rejection rolls back to the prior state.
- Delete row disappears instantly; forced rejection re-upserts.
- `useAgentsAdminStore` no longer referenced from the page (admin store is now orphaned, ready for retirement in the next change).
