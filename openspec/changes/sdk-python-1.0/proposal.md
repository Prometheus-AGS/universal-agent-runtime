# sdk-python-1.0

## Why

The existing Python package is an unpublished alpha client whose chat methods target disabled legacy routes and whose public API covers only a fraction of the runtime. Python users need a stable, typed, async 1.0 client matching the Rust SDK capability surface.

## What Changes

- Release the MIT-licensed Python package at version 1.0.0.
- Add typed async APIs for chat, streaming chat, tool calls, structured outputs, embeddings, the complete run lifecycle, knowledge-base CRUD/search/document upload, and ingestion.
- Use `httpx-sse` for SSE and Pydantic v2 for public request/response models.
- Add six runnable examples, focused tests, and Sphinx API documentation suitable for ReadTheDocs or GitHub Pages.

## Capabilities

### New Capabilities

- **`sdk-python-1.0`** — stable typed Python client for the UAR HTTP and SSE APIs.

## Impact

- Affected code is confined to `sdks/python/` plus this OpenSpec change.
- No server routes or other SDKs are modified.
- The package requires Python 3.10 or newer.

