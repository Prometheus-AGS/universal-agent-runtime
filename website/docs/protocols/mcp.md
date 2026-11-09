---
sidebar_position: 4
title: Model Context Protocol
description: MCP discovery, namespacing, health, reconnect, and trusted-host execution boundaries.
source_records:
  - docs/TOOL_NAMING.md
current_authority: /docs/protocols/mcp
---

# Model Context Protocol

Use the [canonical MCP recovery/effect row](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/product-support-matrix.md#execution-and-recovery-guarantees)
for current scope. This guide describes source roles; each advertised
role/transport/version still needs independent pinned-counterpart acceptance on
the selected artifact. A working client tool does not certify full server feature
parity, reconnect does not replay an uncertain effect, and unqualified optional
features remain partial. The [final checklist](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/releases/v1.4.0-final.md)
records that pending evidence.

## Boundary statement

**MCP discovery is not authorization.** A connected server can advertise a tool
and schema, but UAR still filters the effective catalog and executes a selected
call through its trusted host, capability, governance, risk, and approval
boundaries.

## MCP roles in UAR

UAR participates in MCP in three ways:

- it connects as a client to configured stdio or streamable-HTTP servers;
- it exposes runtime operations through `/mcp/uar`;
- when memory and its HTTP server are enabled, it exposes memory operations at
  the configured path, defaulting to `/mcp/memory`.

The server configuration API under `/api/uar/mcp/servers` persists enabled
entries through UAR settings. `/api/uar/mcp/health` reports connection and tool
counts. Environment maps expose key names in public views, not secret values.

## Discovery and namespacing

At connection time the registry reads each server's tool list and builds a
namespaced identity from server and raw tool name. Public names use a sanitized
`server__tool` form, such as `time__now`. Native tools use the `native__` prefix
inside the same registry.

Names prevent collisions and make provenance visible. They do not grant the
model or agent permission to call the tool. An effective run can see only the
filtered catalog its host and policies allow.

## Execution boundary

A selected call resolves the namespaced identity back to its registered server
and raw tool name. UAR applies the effective tool set and runtime decisions,
then the host calls the MCP transport. Tool arguments and results enter the
normal runtime lifecycle and event stream.

Transport failure, an unknown name, a timeout, policy denial, approval
rejection, and tool-returned error are distinct outcomes. A discovered schema
does not prove that a call will be authorized or succeed.

## Health and reconnect

Configured stdio servers are child processes; remote servers use streamable
HTTP. Configuration placeholders must resolve before a remote URL is accepted.
On a call failure caused by a closed transport, the registry can reconnect the
configured service for a subsequent call. It does not replay the failed tool
call automatically. During shutdown, new reconnect installation is blocked and
the registry waits for active transport closure within the process shutdown
contract.

The MCP health page and API show current connection state. They are operational
views, not durable availability history or proof that every advertised tool was
executed.

After cancellation, keep the exact invocation and its known or unknown effect
disposition. Closing the transport cannot settle an unknown external effect.
A claimed effect can prevent a clean shutdown until the trusted host has
authoritative evidence. See [RC3 protocol checks](../release-candidate.md#run-tools-team-tasks-and-protocol-peers).

## Security and configuration

Treat a remote MCP URL and a stdio command as privileged operator
configuration. Put secrets in referenced environment variables rather than in
committed JSON. A stdio server runs with the permissions of its configured
process unless an explicitly supported sandbox boundary applies.

See [Tools and trusted-host execution](../tools/overview.md) for native-tool
defaults and approval, and [Configuration](../configuration.md) for settings
authority.

## Profile limits

`minimal` and `server-full` expose the server MCP endpoints and configured
registry. `embedded-mobile` has no HTTP MCP endpoint by default; an embedding
host can supply optional MCP integration in process and owns its transport,
credentials, lifecycle, and evidence. MCP success does not certify A2A or model
inference.

Next: [Agent-to-Agent Protocol](./a2a.md).
