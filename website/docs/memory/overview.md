---
sidebar_position: 1
title: Use Agent Memory
description: Enable scoped memory, observe recall and capture, administer records, and distinguish durable memory from other runtime state.
source_records:
  - docs/MEMORY_SYSTEM.md
  - docs/CONVERSATION_MEMORY.md
current_authority: /docs/memory/overview
---

# Use agent memory

## Boundary statement

**Memory is opt-in durable recall, not an implicit property of a model or chat
window.** UAR creates the memory service only when memory is enabled and its
storage/embedding configuration initializes successfully. Without that service,
memory administration and recall must report disabled or unavailable state.

## Memory is not knowledge

Memory records hold captured or explicitly written facts at global, agent, user,
session, or task scope. Knowledge bases hold operator-supplied documents and
indexed chunks. Selected model context is the resolved input for one call. A
live event reports a recall or mutation. Process-local history retains messages
for conversation continuity. These are four different authorities.

| State | Storage authority | Lifetime/use |
|---|---|---|
| Memory record | The configured memory store | Durable until mutation, deletion, or owning retention behavior |
| Knowledge resource | UAR persistence for knowledge bases/documents | Retrieved when explicitly attached to execution |
| Selected model context | Effective run policy plus assembled prompt | One resolved model call |
| Live event | Run event stream/replay boundary | Observation of recall or mutation, not the stored record itself |
| Conversation history | Session/run state and configured persistence behavior | Multi-turn message continuity, not cross-session semantic memory |

## Enable and initialize

Memory is disabled by default. Enable the `memory.enabled` setting (or the
corresponding `UAR_MEMORY__ENABLED` environment setting), then configure:

- a memory database path or external SurrealDB endpoint;
- an embedding provider and model supported by the memory service;
- whether post-turn auto-capture is enabled;
- whether pre-call context injection is enabled and its token budget;
- whether the memory MCP HTTP surface is enabled and its path.

Restart after changing memory startup configuration. Do not assume a generic
configuration reload reconstructs `MemoryService` unless that deployment has
explicitly verified it. Startup logs must show successful service
initialization. If initialization fails, the server logs the error and continues
with memory disabled; do not infer readiness from the configured flag alone.

For `EmbeddedRuntime`, the host constructs a `MemoryService` with its chosen
store and embedding configuration and passes it to the builder. Its memory store
must not use the same locked directory as the runtime persistence store.

## Capture and context injection

When effective run policy enables memory and `inject_context` is active, UAR
searches relevant scoped records before the model call, assembles a bounded
memory block, injects it into selected model context, and emits a memory-recall
event for the returned hits.

When `auto_capture` is active, UAR examines the completed user/assistant turn
after the stream reaches a terminal run event. Newly captured records are
written asynchronously and reported as memory-mutation events. A completed
assistant response is therefore not proof that capture succeeded; observe the
mutation or query the store.

## Packaged UI workflow

1. Enable memory and restart/reload the server as required.
2. Open **Admin → Memory** at `/admin/memory`.
3. Confirm the memory count and scope breakdown load. Filter by user, agent, or
   session and search by meaning when inspecting existing records.
4. Create or update a record through the admin API or memory MCP surface; the
   packaged browser currently focuses on inspect/search/delete operations.
5. Select an agent/conversation whose effective policy enables memory, then send
   a new Chat request.
6. Inspect **Memory · read** for recall items and **Memory mutation** for a
   create/update/delete observation. Query the admin surface to confirm durable
   state after an asynchronous capture.
7. Use single-record or filtered bulk deletion deliberately. The UI describes
   deletion as permanent.

## Admin and API workflow

Admin memory routes require admin authorization and are mounted at
`/api/admin/memories`:

| Operation | Request |
|---|---|
| List/filter | `GET /api/admin/memories` with optional user, agent, and session filters |
| Statistics | `GET /api/admin/memories/stats` |
| Search | `GET /api/admin/memories/search?q=...` with optional scope filters |
| Create | `POST /api/admin/memories` |
| Inspect/update/delete | `GET`, `PATCH`, or `DELETE /api/admin/memories/{id}` |
| Bulk delete | `DELETE /api/admin/memories` with explicit scope filters |

A disabled service returns an empty disabled collection for the list boundary;
operations that require the service return unavailable. That distinction lets a
settings UI render “off” without representing mutation/search as successful.

## Memory MCP workflow

When `memory.mcp_http_enabled` is true and the service initialized, the
streamable HTTP memory MCP endpoint is mounted at the configured path (default
`/mcp/memory`). Its tools cover scoped add/get/update/delete/list/search,
history and compression, conversation extraction, knowledge-graph operations,
and task streams.

The memory MCP surface is a transport over the same storage authority. Tool-call
success is mutation/lookup evidence for that record; it is not proof that a
later run recalled or injected it.

## Realtime state and reload authority

- Startup settings and memory-service construction are reload/restart state.
- Created, updated, and deleted records are durable mutation state.
- Recall and mutation events are live run observations and may be replayed by
  the owning event boundary; they are not the database.
- Auto-capture happens after response completion, so admin state may reconcile
  after the visible answer.
- Conversation messages can maintain multi-turn context without creating a
  semantic memory. Conversely, a durable memory can be recalled in a later
  session when scope and relevance permit.

## Mutation and safety boundaries

Memory content can affect future prompts. Restrict admin and MCP access, scope
records correctly, and inspect recalled content rather than treating persistence
as endorsement. Single and bulk deletion are destructive. A process crash after
the assistant response but before asynchronous capture can leave the answer
complete without a new memory record.

No retained 1.0 functional closeout result certified memory. This page describes
delivered source behavior; a readiness claim requires a fresh capture/write,
recall/injection, mutation, and durable-store observation in the target profile.

## Profile limits

- `server-full` includes the packaged Memory UI and server admin/MCP/run
  composition when memory is explicitly enabled.
- `minimal` has the server base, but memory remains opt-in and the admin UI is
  outside the profile claim.
- `embedded-mobile` receives memory only when the host attaches a
  `MemoryService`; it has no packaged HTTP/UI surface and owns lifecycle,
  storage-path, and embedding integration.

Memory evidence does not transfer across profiles, stores, scopes, or embedding
providers. Review [runtime profiles](/docs/architecture/profiles) for the larger
composition boundary.
