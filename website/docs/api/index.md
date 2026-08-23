---
sidebar_position: 1
title: API Reference Map
description: Choose between UAR's generated, narrative, and runtime API references without confusing their coverage.
source_records:
  - docs/API_CHAT_COMPLETION.md
current_authority: /docs/api
---

# API reference map

## Boundary statement

**UAR has several API reference layers, and none silently stands in for the
others.** The running router is the authority for the routes compiled into one
server. The embedded OpenAPI document is a maintained summary. Generated
language references describe source APIs. Narrative guides explain workflows,
state, and compatibility limits.

## OpenAPI summary

`server-full` includes the `api-docs` feature and exposes:

- `/api/openapi.json` — the embedded OpenAPI 3.1 summary;
- `/api/docs` — Swagger UI backed by that summary.

The summary covers the principal health, chat, model, metric, MCP-health, run,
provider, skill, knowledge, authentication, and realtime routes. It is not an
exhaustive inventory of every route mounted by `src/server.rs`. Use it for
client discovery, then use the relevant narrative guide for lifecycle and
security behavior.

`minimal` includes the HTTP server but not the `api-docs` feature by default.
An additive custom build can enable it, but that custom composition needs its
own evidence.

## Generated references

The Pages artifact stages two source-generated references:

- [Rust API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/rust/) — workspace rustdoc, including the runtime and Rust SDK;
- [TypeScript API reference](https://prometheus-ags.github.io/universal-agent-runtime/docs/api/typescript/) — TypeDoc from `sdks/typescript`.

The repository contains a Python SDK and Sphinx configuration, but the Pages
assembler does not stage a Python generated reference. See the
[Python SDK guide](../sdk-python/intro.md) for the local source workflow.

## Narrative references

Start with the boundary you intend to call:

- [Protocol overview](../protocols/overview.md)
- [HTTP compatibility](../protocols/http-compatibility.md)
- [Events, AG-UI, and A2UI](../protocols/events-and-ui.md)
- [MCP](../protocols/mcp.md)
- [A2A](../protocols/a2a.md)
- [Tools and trusted-host execution](../tools/overview.md)
- [Authentication](../security/authentication.md)
- [Runs](../operations/runs.md)

For exact request and response structures that are not in the embedded OpenAPI
summary, inspect the versioned SDK types and current server source for the
release you deploy.

## Publication status

Source packages, a `1.0.0` value in package metadata, a Git tag, and a generated
reference are four separate facts. This portal confirms the repository sources
and the two references staged by the Pages contract. It does not infer crates.io,
PyPI, npm, image-registry, or release-asset availability from local metadata.

## Profile limits

The network API belongs to server builds. `server-full` carries the documented
release composition and generated Swagger UI; `minimal` carries the smaller
HTTP/SSE composition without API docs by default. `embedded-mobile` is
transport-free and calls in-process services instead, so HTTP route evidence
does not transfer to it.

Next: [Protocol overview](../protocols/overview.md).
