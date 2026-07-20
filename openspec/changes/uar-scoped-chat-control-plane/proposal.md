## Why

UAR already exposes provider, agent, skill, MCP, knowledge, and context configuration, but conversation overrides are process-local and most selections never reach the run hot path. KnowMe therefore cannot truthfully offer global, agent, and conversation controls or let host-owned local models participate in UAR-governed execution.

## What Changes

- Add a typed, durable run policy with deterministic global → agent → conversation → turn resolution and an immutable effective-policy snapshot.
- Apply the resolved model, skills, MCP servers, knowledge bases, memory, context strategy, and tool-approval policy during real run construction.
- Promote the external LLM seam into a capability-aware local-model provider contract for embedded desktop and mobile hosts.
- Expose APIs for reading and updating scoped policies and for inspecting the effective policy used by a run.
- Preserve realtime visibility by emitting policy, retrieval, skill, MCP, model-routing, lifecycle, and failure events through the normalized stream.
- Enforce protected built-in agents and prevent lower scopes from re-enabling globally or agent-disabled resources.
- Update KBD/OpenSpec workflow state and downstream KnowMe bindings after the UAR contract is certified.

## Capabilities

### New Capabilities

- `scoped-chat-control-plane`: Typed durable resource selection, policy precedence, effective-policy inspection, and execution enforcement.
- `embedded-local-model-provider`: Host registration and lifecycle of local model providers that remain inside UAR agent, context, skill, MCP, and tool loops.

### Modified Capabilities

- `provider-model-settings-certification`: Enabled provider/model configuration and local provider capabilities must govern runtime model selection.
- `knowledge-rag-product-certification`: Knowledge-base selection must be scoped per run and affect retrieval and citations.
- `ag-ui-chat-conformance`: Normalized policy, skill, MCP, retrieval, model lifecycle, and error events must be observable and replayable.

## Impact

- UAR domain types, persistence providers and migrations, Axum APIs, run manager, provider routing, skill matching, MCP registry filtering, RAG retrieval, and event history.
- Backward-compatible request decoding for existing clients; KnowMe desktop/web/mobile adapters migrate to the typed policy contract.
- Provider compatibility expands to embedded host runtimes without bypassing Liter-LLM cloud routing or UAR governance.
- Runtime UX gains truthful inherited/overridden state, preflight diagnostics, and auditable run configuration.
