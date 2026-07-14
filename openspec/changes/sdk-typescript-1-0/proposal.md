## Why

KBD Change `sdk-typescript-1.0` requires the TypeScript SDK to become a stable,
runtime-validated 1.0 client with the same public capabilities as the Rust SDK.

## What Changes

- Expand the client across chat, tools, structured output, embeddings, run
  lifecycle, knowledge-base CRUD, documents, search, and ingest.
- Add SSE streaming through `@microsoft/fetch-event-source` and runtime
  response validation through `zod`.
- Add tests, six runnable examples, TypeDoc, and a GitHub Pages workflow.

## Capabilities

### New Capabilities

- `sdk-typescript-1-0`: stable TypeScript SDK 1.0 public contract.

### Modified Capabilities

None.

## Impact

Changes are confined to `sdks/typescript/`, its focused documentation workflow,
and this OpenSpec artifact. Package publication is explicitly out of scope.
