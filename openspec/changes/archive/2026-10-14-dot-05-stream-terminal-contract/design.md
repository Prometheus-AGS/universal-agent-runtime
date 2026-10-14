# Design

## Context

See proposal.md for motivation and ../../../../.kbd-orchestrator/phases/dot-feature-support/analysis.md for dated source/research evidence. Baseline is main 2cc9a182. Conditional terminal enforcement and native per-chunk decoding permit false completion.

## Goals / Non-Goals

Goal: Supported streams require protocol completion evidence. Required profiles are local durable SurrealKV and shared durable networked SurrealDB with at least two runtimes. Preserve existing Postgres behavior and test modified credential/key/accounting paths there; do not claim missing Postgres observer/checkpoint parity. Direct embedded SDK certification and new assistant UI are outside scope.

## Decisions

Define terminal semantics for each actually selected provider protocol and enforce them on graph and legacy routes. Incrementally frame bytes/SSE/JSON including UTF-8 and CRLF in the native path; reuse inspected Liter framing without gratuitous vendored rewrites. Surface malformed/unfinished streams as errors, preserving cancellation and tool-call semantics.

Keep all mutations in trusted host services, retain complete verified owner scope, and expose durable business state through explicit inspectable records. Use fallible typed persistence outcomes; do not mask an unsupported backend or outage with success. No kernel mutation authority, no automatic replay of unknown effects, and no new lease expiry replacing exclusion.

Alternatives: existing UAR services are preferred over an independent queue/workflow authority because admission, receipt and owner transactions must agree. Apalis/Temporal remain analysis references, not dependencies. Rebuilding existing approvals/observer/credential machinery is excluded.



## Candidate evidence

### cand-007: Pinned Liter 2.1.1 SSE parser and UAR provider boundary — reference

Reuse proven framing ideas from exact checked-out source while enforcing protocol completion at the host provider boundary. A library upgrade alone does not resolve UAR's conditional success.

- Tier 1: [source](https://github.com/GQAdonis/liter-llm/blob/a6047386cc9fae4258b4a8577f6a016022095586/crates/liter-llm/src/http/streaming.rs) — Checked-out parser buffers UTF-8/lines and rejects incomplete buffered EOF, but empty-buffer EOF returns no error without a terminal sentinel.
- Tier 1: [source](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/2cc9a1829167411e01a8198072f60892d70a233d/src/llm/liter_driver.rs) — UAR requires terminal completion conditionally; the native Anthropic parser is a separate route.

Risks: Do not replace provider-specific message/event parsing with a generic data-line parser without compatibility checks. No wire-level reproduction or complete provider matrix ran. versions.toml policy pins still require an explicit reconciliation change.

## Risks / Trade-offs

The uncomfortable thing: a persisted row or green isolated test does not prove cross-process authority and restart correctness. Validate the actual production boundary and retain negative controls. New retries/guards must trace to the reported failure or real owner/effect/consent boundary; do not harden unrelated code.

No unresolved artifact-review prerequisite specific to this change; dependencies and real infrastructure availability still apply.

## Migration Plan

Keep provider routing and interrupted-history semantics. Deploy adapter changes against reconciled exact sources; rollback keeps truthful interrupted state and does not re-execute partially emitted turns.

Dependencies: dot-01-release-provenance. Preserve other active changes and their evidence status. Follow the phase plan's explicit reconciliation of credential-store, cost-history, run-scoped resources and team-admission ownership. No wholesale migration or rollback may erase history.

## Verification

Through production orchestrator plus controlled protocol endpoints, split every relevant boundary and test empty-buffer EOF without terminal, trailing partial event, malformed event, cancellation and valid terminal. Inventory every supported route; unsupported routes must not advertise completion guarantees.

Author relevant integration fixtures and API/operator documentation with the implementation. Execute no tests, builds, compiler checks, lint, format, validators or product review gates until the lead records that every production item in the expanded Dot phase (changes 01–20, including the Dot17 runner) is implemented. A completed change never opens that boundary. Afterward run the planned real-path gates serially and locally; do not put product suites in Actions. The operator’s latest full-phase instruction also defers the workflow-policy validator when a necessary workflow-invoked script is edited: retain it as required final-gate work, never report it skipped or passed. Record exact newly added commands only once their targets exist. Historical evidence for 01–06 remains bound to its original source.

