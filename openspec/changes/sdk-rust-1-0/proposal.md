# Rust SDK 1.0

## Why

The existing Rust SDK advertises 1.0.0 but still targets disabled legacy chat
routes and lacks the production run, streaming, tool, structured-output,
embedding, checkpoint, and complete knowledge-base surfaces.

## What Changes

- Align the HTTP client with the committed UAR and OpenAI-compatible routes.
- Add typed SSE streaming and typed diagnostics.
- Complete runs, knowledge-base, document, ingest, tool, and embedding APIs.
- Add six runnable examples, migration notes, tests, and publishable rustdoc.

## Impact

All implementation is isolated to `sdks/rust/` plus this OpenSpec change.
The embeddings client targets the conventional `/v1/embeddings` contract; the
committed server does not yet mount that route, which remains an integration
dependency rather than SDK scope.
