---
sidebar_position: 1
title: Python SDK
description: Typed asynchronous HTTP and SSE access from the Python SDK source package.
source_records:
  - sdks/python/README.md
current_authority: /docs/sdk-python/intro
---

# Python SDK

## Boundary statement

**The Python SDK is a source-available network client.** It does not embed UAR,
own server configuration, or prove that a PyPI artifact or hosted generated
reference exists.

## Client surface

`Client` uses `httpx` and `httpx-sse`; Pydantic validates typed responses. It
supports non-streaming and streaming chat, tool requests, structured output,
embeddings, run creation/stream/cancel/checkpoint/resume, knowledge-base CRUD and
search, document upload, and ingestion.

The client is asynchronous and supports Python 3.10 or newer. `api_key` becomes
a bearer header. Use an async context manager or call `close()` when the client
owns its HTTP session.

## Source checkout

Use the checked-in lock and package source:

```bash
cd sdks/python
uv sync --locked
UAR_BASE_URL=http://127.0.0.1:1906 uv run python examples/chat.py
```

The examples directory also covers streaming, tool calls, structured output,
agent runs, and retrieval. Network examples require a configured UAR server and
valid credentials.

## Local Sphinx reference

The repository includes Sphinx sources in `sdks/python/docs` and the development
dependencies needed to build them locally:

```bash
cd sdks/python
uv sync --locked --extra dev
uv run sphinx-build -W -b html docs docs/_build/html
```

That output is not staged by the current Pages assembler. The public portal
therefore provides this narrative guide and source links, not a generated
Python API tree.

## Registry publication

`pyproject.toml` names `universal-agent-runtime-sdk` version `1.0.0`. Confirm the
project, exact version, files, and publisher on PyPI before using a registry-only
installation. Local metadata and Sphinx configuration are not availability
evidence.

## Profile limits

This SDK calls an HTTP/SSE server built as `minimal`, `server-full`, or a named
custom server composition. It does not call the transport-free
`embedded-mobile` profile directly. A successful SDK request verifies only its
route, server version, provider, and credentials.

Next: [HTTP compatibility](../protocols/http-compatibility.md).
