## 1. Contract and package

- [x] 1.1 Define the complete typed public API and 1.0 package metadata.
- [x] 1.2 Add `miette` diagnostics and typed central UAR error decoding.

## 2. Client implementation

- [x] 2.1 Implement chat, tool-call, structured-output, and embeddings methods.
- [x] 2.2 Implement create/stream/cancel/resume/checkpoint run lifecycle.
- [x] 2.3 Implement knowledge-base CRUD, document CRUD/upload, search, and ingest.
- [x] 2.4 Implement reconnectable typed SSE parsing.

## 3. Developer experience

- [x] 3.1 Add six runnable examples.
- [x] 3.2 Add README, BREAKING.md, and complete rustdoc.
- [x] 3.3 Add focused request/response and stream tests.

## 4. Verification

- [x] 4.1 Pass fmt, check, tests, examples, rustdoc, and strict OpenSpec validation.

## Integration note

The SDK exposes embeddings through the conventional `/v1/embeddings` route,
but base `b9a85515` does not mount that endpoint. Runtime support must land
before the embeddings method can succeed against the first-party server.
