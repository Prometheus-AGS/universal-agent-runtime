# Proposal

## Why

Conditional terminal enforcement and native per-chunk decoding permit false completion. This change supplies the R4 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Define terminal semantics for each actually selected provider protocol and enforce them on graph and legacy routes. Incrementally frame bytes/SSE/JSON including UTF-8 and CRLF in the native path; reuse inspected Liter framing without gratuitous vendored rewrites. Surface malformed/unfinished streams as errors, preserving cancellation and tool-call semantics.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `model-path-resiliency`: Supported streams require protocol completion evidence. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/llm/liter_driver.rs`, `src/llm/anthropic_driver.rs`, `src/llm/orchestrator.rs`, `vendor/git/liter-llm`. Dependencies: dot-01-release-provenance. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

