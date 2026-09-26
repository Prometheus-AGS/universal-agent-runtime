# Illustrative protocol and recovery traces

These eight JSONL documents are **authored expected behavior**, not captured traffic, integration results or certification. No application was executed to produce them. IDs, timestamps, task outcomes and fixture authority statements are fictional. They contain no real grants, keys, user content or published issue.

Each line follows [trace-row.schema.json](../schemas/trace-row.schema.json): `scenario`, sequential fixture `step`, `channel`, object `frame` and optional `notes`. The wrapper is not a protocol envelope and MUST NOT be sent over AG-UI, A2UI or A2A. Channel selection means:

| Channel | Meaning of frame |
|---|---|
| `ag-ui` | Proposed AG-UI event body; CUSTOM events follow the package's custom-event schema |
| `a2ui` | Reserved for a standalone accepted A2UI message body; these traces transport such bodies inside `surface.message` instead |
| `a2a` | Proposed A2A 1.0 JSON-RPC request or response body; streaming response is the SSE `data` body |
| `uar-action` | Proposed trusted-host action request, following surface-action.schema.json; not an upstream A2UI message |
| `internal` | Explanatory transition or assertion, not a public wire message, API, runtime snapshot or additional generated schema |

An internal `fresh-authority-issued` row asserts a fixture precondition; it is not a new approval endpoint or token. Likewise `snapshot-installed` is an assertion about consuming a snapshot, not a substitute for the runtime-snapshot schema. The document validation boundary checks wrapper/schema agreement. Runtime assertions need real integration evidence in later phases.

## Coverage and expected authoritative result

| File | Protocol/profile | Requirement and outcome |
|---|---|---|
| [ag-ui-interleaving.jsonl](ag-ui-interleaving.jsonl) | AG-UI base plus negotiated standard subagents and uar.team.agui/1 | Two workers interleave distinct messages; design subteam has separate identity; parent waits for children; integration runs only after all production dependencies finish |
| [ag-ui-base-client.jsonl](ag-ui-base-client.jsonl) | AG-UI base only | A durable approval wait outlives a finished segment; resumed segment has a new run ID; no unknown enhanced event reaches the base client |
| [a2ui-concurrent-actions.jsonl](a2ui-concurrent-actions.jsonl) | uar.a2ui/1, v0.9.1, approved catalog | Concurrent surfaces in two teams; action targets server-owned member; stale revision and conflicting team hint do not dispatch |
| [a2a-ordinary-client.jsonl](a2a-ordinary-client.jsonl) | Released A2A specification v1.0.1, wire1.0, JSONRPC | Opaque team task streams an artifact; cancel settles before terminal status; GetTask preserves result without revealing member identities |
| [reconnect-retention-gap.jsonl](reconnect-retention-gap.jsonl) | UAR administrative event projection | Retention gap requires authorized snapshot and watermark; duplicate replay reduces once; revoked authority blocks subsequent replay |
| [restart-pending-approval.jsonl](restart-pending-approval.jsonl) | UAR custom events plus internal recovery | Durable approval intent survives; executable old authority does not; new attempt requires current trusted approval |
| [stale-ownership-and-budget.jsonl](stale-ownership-and-budget.jsonl) | UAR custom events plus internal admission | Stale epoch cannot dispatch an effect; root aggregate budget prevents another turn; only authorized administrator raises limit |
| [duplicate-message-and-uncertain-effect.jsonl](duplicate-message-and-uncertain-effect.jsonl) | UAR receipt/task/recovery events | Duplicate message returns existing receipt; processed receipt does not imply effect success; lost issue-create response is reconciled without creating another issue |

The custom-event `sequence` is scoped to team identity, not fixture step. Duplicate replay deliberately repeats event IDs/sequences. Two separate teams may both emit sequence 1. Internal assertions between frames do not consume a public sequence. Filtered projections may omit events; these abbreviated narratives are not full journal exports.

No trace mixes upstream A2A 0.3/RC field conventions with 1.0. A2UI preserves UAR's existing v0.9.1 field/catalog restrictions rather than copying upstream 1.0 examples. Standard subagent fields follow the source documented in [AG-UI contract](../protocols/ag-ui.md); the future implementation must pin the exact compatible upstream SDK/schema before claiming enhanced conformance.

## Validation scope and later integration

The once-per-completed-artifact validation checks JSON parsing, schema examples, local links and trace-envelope agreement. An internal state assertion is not mechanically proven by JSON validity. Standard protocol payloads require exact upstream schema validation and actual clients during I3; task recovery/effect outcomes require real adapters and persistence during I2/I4. Do not interpret a schema-passing trace as a runtime test result.

The uncomfortable case is an attractive trace that omits a real crash window. Recovery traces intentionally expose dispatch uncertainty and fresh authorization, but only execution against the selected persistence/effect adapters can establish that those invariants hold.
