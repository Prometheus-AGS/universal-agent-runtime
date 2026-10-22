# Caller-owned tools across UAR interfaces

## Why
The operator reported that silently ignoring OpenAI tools is unacceptable and explicitly requested the full tool loop across OpenAI Chat Completions, AG-UI, Anthropic Messages, A2A and ACP. Frozen Kubo release1.2.0 must not change.

## What changes
Add typed caller declarations/choice/parallel controls, full conversation input, host/caller ownership in a frozen model-call snapshot, durable caller handoff and exact result continuation/replay. Preserve host governance, graph cursor, protected transcript and cumulative budget accounting across restart. Each transport projects its native tool contract. No caller schema becomes executable host authority.

## Capabilities
- caller-tool-continuation: authenticated durable handoff/claim/replay and retained host state.
- openai-compatible-client-tools: JSON and streaming wire contract plus native projections on other supported interfaces.

## Uncomfortable limitations
No new behavior is compiled or accepted while the operator verification gate is CLOSED. A claimed continuation interrupted after side effects is not automatically re-executed. Unsupported provider controls return explicit errors. Release1.2.1 publication needs separate authorization and a new digest;1.2.0 stays fixed.
