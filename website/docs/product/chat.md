---
sidebar_position: 1
title: Chat
description: Start conversations, select effective agent configuration, and observe governed runs in the first-party React application.
source_records:
  - frontend/src/pages/chat-page.tsx
  - frontend/src/features/chat/chat-thread-view.tsx
  - docs/product-surface-inventory.md
current_authority: /docs/product/chat
---

# Chat

## Boundary statement

**The Chat screen is a client for UAR execution, not the model runtime.** A
selected label proves what the UI requested. The effective run configuration
and returned runtime events prove what UAR resolved and executed.

The first-party React application mounts Chat at `/threads`. The screen owns
conversation selection, agent and session controls, message composition, run
status, and normalized output. Provider calls, tools, retrieval, memory, policy,
and authoritative server state remain behind UAR's governed boundary.

## Start a conversation

1. Configure a provider and default model. Chat blocks with **No Model
   Configured** when UAR cannot resolve one.
2. Open `/threads` and select an existing thread or choose **New conversation**.
3. Select an agent or keep the default. Open session configuration to inspect or
   change the conversation-scoped model and resource choices.
4. Send a message. The client submits through UAR and renders the returned text,
   tool lifecycle, citations, status, and supported A2UI artifacts.
5. Inspect the effective configuration and run stream when the claim depends on
   a particular agent, model, skill, tool, or knowledge base.

Changing an agent or session setting affects a subsequent request; it does not
rewrite an in-flight or completed turn.

## State and reconnect behavior

Thread/message state can be retained in browser PGlite while server entities
remain authoritative. Unsent drafts are client-owned. Run streams may replay
events retained by the current process when a valid cursor is supplied; they do
not promise durable history after restart or eviction.

The responsive sidebar becomes an overlay on narrow viewports. Keyboard focus,
loading state, empty state, and route transitions remain part of the product UI
contract, not proof that a backend operation succeeded.

## Profile limits

- `server-full` packages this React screen and its REST/SSE services.
- `minimal` exposes relevant server paths but does not include the packaged UI
  as a profile claim.
- `embedded-mobile` supplies its own host presentation around transport-free
  runtime services; the React route does not transfer unchanged to that host.

Continue with [Create and run agents](/docs/agents/overview),
[Verify genuine inference](/docs/providers/inference), and
[Understand realtime state](/docs/operations/realtime).

