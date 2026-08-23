---
sidebar_position: 5
title: Agent-to-Agent Protocol
description: A2A task, context, tenant, JSON-RPC, registry, and gRPC boundaries in server-full.
source_records:
  - docs/A2A_PROTOCOL.md
current_authority: /docs/protocols/a2a
---

# Agent-to-Agent Protocol

## Boundary statement

**A2A peer input becomes a UAR task only inside the authenticated, tenant-aware
handler.** A peer-supplied task, context, or tenant-like string is not itself a
verified tenant identity or an authorization decision.

## Current surface

The current source identifies its JSON-RPC types as A2A RC v1.0 and mounts:

- `GET /.well-known/agent.json` — the compiler agent card;
- `POST /a2a/compiler` — JSON-RPC 2.0 dispatch;
- `/a2a/registry` routes — agent and skill discovery.

The dispatcher implements `message/send`, `tasks/get`, and `tasks/cancel`.
`contextId` maps to the compiler session used for a multi-turn interaction. A
completed compilation can attach a typed descriptor artifact.

`server-full` also enables the tonic gRPC transport on the configured gRPC
port. Its send/get/cancel operations share the same A2A state and task store.
The streaming method currently emits a status update rather than incremental
compiler artifacts.

## Verified tenant boundary

When JWT authentication is required, HTTP and gRPC calls must produce a
verified tenant claim before A2A task handling proceeds. The task store keys
task ids and context ids by that verified tenant. Cross-tenant lookup and
cancellation therefore return no matching task instead of mutating the owning
tenant's task.

Anonymous mode uses the unpartitioned anonymous scope and does not provide
tenant isolation. Tenant-aware A2A task/context storage is not a blanket claim
about every UAR table, cache, event, or API; see
[Tenancy](../tenancy/overview.md).

## Task and context lifecycle

`message/send` creates a compiler session and task when no matching context is
found, or appends a turn to the existing tenant-scoped task. `tasks/get`
returns its current state and artifacts. `tasks/cancel` changes only a
cancellable submitted, working, or input-required task and best-effort cancels
the corresponding compiler session.

The current task store is in-memory. Task and context lookup do not survive a
process restart, even when other UAR resources use durable persistence.

## Registry boundary

The A2A registry advertises known peer agents and skills. Registry discovery
does not authenticate a peer, authorize a task, or prove the remote agent is
healthy. Treat remote base URLs and advertised capabilities as operator-managed
metadata until an authenticated call succeeds.

## Compatibility limit

Current source is the present-tense authority where older narrative material
names a different A2A revision, route, or port. UAR implements the checked-in
types and transports; it does not claim every optional A2A operation, push
notification, or streaming behavior.

## Profile limits

The documented A2A transport is a `server-full` claim. The default `minimal`
profile does not enable `a2a-transport`, and `embedded-mobile` exposes no A2A
network listener. A custom additive build must be named and verified as its own
composition.

Next: [Tools and trusted-host execution](../tools/overview.md).
