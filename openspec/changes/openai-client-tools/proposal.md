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

## Finite delivery boundary — 2026-10-22
The operator requested a KBD-derived finish contract. The existing feature now uses `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/client-tools-exit-contract.md` (CT-1–CT-7). Local built-in AgentNode team delegation is in scope; remote federation, arbitrary graph certification, new providers and broader estate checks are excluded. Stop on passing immutable-image evidence, or report the defined no-progress blocker without waiving functional failures.
