# Proposal

## Why

MCP transport Ok can classify isError=true as execution success. This change supplies the R3 acceptance guarantee for the always-on assistant runtime; source findings are not yet reproduced failures.

## What Changes

Propagate application failure and original content through registry, ToolEnd, host admission and durable connector receipts. Keep transport failure distinct from post-dispatch outcome unknown; only successful receipts advance success dependencies. Reuse existing effect classification/reconciliation and never broaden retry authority.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `tool-call-protocol`: Tool execution preserves application and uncertain outcomes. Add a requirement without removing existing scenarios.

## Impact

Code ownership: `src/mcp/registry.rs`, `src/llm/orchestrator.rs`, `src/uar/runtime/tool_admission`, `src/uar/compiler/collaboration/connector_effect`. Dependencies: dot-02-principal-credentials. Runtime UX changes are truthful API/event outcomes and durable state; no new UI is included. Provider compatibility preserves existing routing and secret reattachment; no new provider integration. Realtime consumers must observe the same committed outcomes, not inferred in-memory success. KBD task state changes only through the phase's typed task lifecycle during Execute; this proposal does not implement behavior.

