# PLAN: classify-readiness-diagnostic-websocket-frame

**Project:** Universal Agent Runtime
**Date:** 2026-09-05
**OpenSpec available:** YES (installed CLI 1.10.0)
**Changes to implement:** 1 bounded diagnostic change with 4 sequential tasks
**Baseline:** assessment at source HEAD `226d4a0a`; project implementation 113/121 before registration.

## Scope and decisions

Plan one OpenSpec change, `classify-readiness-diagnostic-websocket-frame`, modifying the existing `readiness-latency-diagnostics` capability. The change produces a child-local diagnostic collector, one separately authorized header-only observation, a bounded classification, and independent acceptance. It does not modify the prior collector, product code, dependencies, configuration, database records, or service lifecycle.

The historical rejected frame remains unclassifiable because its header bytes were discarded. Execute may classify only the first corresponding frame from one new authenticated WebSocket attempt after identity and configuration checks. That result must be labeled as a new observation, never as proof of the historical frame.

The sanitized frame schema extends the goal's FIN/opcode/MASK/length list with RSV1–RSV3 and length-encoding form. Without RSV and extension-negotiation state, the report cannot assess even header-level RFC 6455 validity. This is evidence capture, not broadened protocol handling.

No readiness endpoint, namespace/database selection, query, inference request, product test suite, restart, retry, or recovery action is authorized by this plan. Optional Analyze is skipped: no library selection, architecture choice, dependency introduction, or linked evolver plan is present.

## CHANGE LIST (ordered)

1. **classify-readiness-diagnostic-websocket-frame:** Capture and classify one newly observed rejected frame from structural metadata only.
   - **Scope:** child-local diagnostic source and evidence; OpenSpec/KBD records.
   - **Depends on:** accepted Assess artifact; explicit `/kbd-execute classify-readiness-diagnostic-websocket-frame` authorization before any connection.
   - **Recommended agent:** Codex.
   - **Est. complexity:** M.
   - **Customer value:** HIGH — removes the parser ambiguity blocking the database-readiness diagnosis without changing runtime behavior.
   - **Details:** Build a single-purpose observer that reuses the established target/authentication form but retains no reply payload or credentials. Classify only what the captured header proves, compare it with the existing collector predicate, and deliver the minimum justified next action after fresh-context review.

## Task order and acceptance

| Task | Depends on | Deliverable and exit criteria |
|---|---|---|
| **1. Establish continuity and prepare the observer** | None | Reconfirm source HEAD, installed UAR and database executable digests, current PIDs/start times, exact listener, nonsecret endpoint/config hashes, authentication form, and prior evidence hashes. Create a dependency-free observer under this child only. Its receive state machine copies at most the first 10 framing bytes needed for base/extended length, records no raw header bytes, never reads a server mask key or payload intentionally, never parses a reply, and emits only the allowlisted schema below. A syntax-only check passes. Identity/config disagreement, unsafe credential provenance, or inability to enforce the header boundary blocks Task 2 and is recorded rather than bypassed. |
| **2. Capture one structural observation** | 1 | Only after explicit Execute authorization, make at most one WebSocket upgrade to the confirmed `/rpc` target, request no extensions, negotiate the existing `json` subprotocol, and send exactly one existing root-signin RPC with credentials held only in memory. Read the base header plus only the 0/2/8 extended-length bytes required to determine declared length, immediately destroy the socket, and retain no response payload. Deliver `evidence/frame-observation.json` and sanitized execution commands/outcomes. Stop on first timeout, transport/auth/identity/shape error; do not retry or continue readiness. |
| **3. Classify and recommend** | 2 | Deliver `classification.md` mapping every observed structural field to (a) RFC 6455 header-level constraints and (b) the old collector's exact acceptance predicate. Distinguish `header-valid`, `header-invalid`, and `indeterminate without payload/fragment sequence`; do not claim full message validity. Identify the exact mismatch that triggered collector rejection if the new observation reproduces it. Recommend only the smallest next action for that exact observed form, or report inconclusive evidence. Do not implement protocol support or resume the readiness diagnostic. |
| **4. Phase-end verification and independent acceptance** | 3 | At the end of implementation, run synthetic diagnostic-parser fixtures and schema/privacy checks; no product suite. A fresh-context artifact-only critic receives the spec, observer digest, sanitized evidence, and classification without generation history. Resolve critical findings or retain a blocked/qualified disposition. Deliver `verification.md` and `handoff-out.md`, record actual attempted/skipped counts, and accept the change only if evidence boundaries and claims match. Then run `kbd-status`; Reflect remains a separate command. |

Task order is strictly 1 → 2 → 3 → 4. No parallel operational work is permitted. A prerequisite failure converts later operational work to explicit SKIPPED outcomes; it does not permit a fallback client, another connection, or enlarged limits.

## Observation contract (Execute only)

### Request and time budget

- One active socket and one total WebSocket upgrade.
- One total sign-in RPC; zero `use`, query, health, readiness, inference, or mutation requests.
- 15 seconds for TCP/upgrade, 15 seconds for the first response header, and 30 seconds total active wall time.
- Stop on the first error, timeout, identity/config drift, unexpected handshake, operator interruption, or evidence-cap violation.
- No automatic or manual retry inside this change.

### Retained schema

The observation may retain only:

- UTC timestamp, source HEAD, executable/config digests, PID/start-time comparison, target host/port/path, and hashes of inherited evidence;
- handshake HTTP status, negotiated subprotocol, `extensionsRequested: false`, and whether any extension response header was present, without retaining raw response headers;
- `fin`, `rsv1`, `rsv2`, `rsv3`, numeric `opcode`, allowlisted opcode class, `masked`, length encoding (`7-bit`, `16-bit`, or `64-bit`), declared payload length as a decimal string, and framing bytes consumed (2, 4, or 10);
- result category, stop reason, elapsed milliseconds, and counters proving one-or-zero attempts, zero payload bytes intentionally read, zero payload bytes retained, zero credentials retained, zero database queries, and zero readiness requests.

The observer must not retain raw framing bytes, mask keys, payload bytes, response text, JSON reply content, credentials, tokens, record identifiers, environment dumps, process arguments, or raw configuration. Evidence files are capped at 64 KiB. A 64-bit declared length beyond JavaScript's safe integer range remains a decimal string; no payload-size allocation follows from it.

The socket API may deliver payload bytes in the same transient network chunk as the header. The observer must copy only the needed framing prefix into its bounded state and discard the remainder without parsing, hashing, logging, or serializing it. Therefore the defensible claim is `payloadBytesRetained: 0`, not that no payload byte ever entered an operating-system or runtime buffer.

### Classification boundary

Use RFC 6455 sections 5.2–5.6. Because the client requests no extensions, nonzero RSV bits make the newly observed header invalid for this handshake. Server-to-client masking, reserved/unknown opcodes, non-final control frames, control-frame declared lengths above 125, and non-minimal extended-length forms are header-level findings. Text/binary continuation sequencing and UTF-8/application-message validity remain indeterminate without payload and prior-fragment state.

Compare those findings separately with the prior collector, which accepts only `FIN=1`, `RSV1–3=0`, `opcode=1`, and `MASK=0` at its first check. A standard-valid control or fragmented/data frame can therefore be collector-incompatible without being proven protocol-invalid.

## Verification and records

Tier 0 after each artifact edit: `git diff --check`, JSON parsing, and syntax-only validation for the observer. These are not product tests. Synthetic parser fixtures, evidence-schema assertions, privacy scans, OpenSpec strict validation, and independent artifact review run only in Task 4 at the child phase boundary. No build, install, service probe, product test suite, or GitHub Actions work is planned.

OpenSpec artifacts:

- `openspec/changes/classify-readiness-diagnostic-websocket-frame/proposal.md`
- `openspec/changes/classify-readiness-diagnostic-websocket-frame/design.md`
- `openspec/changes/classify-readiness-diagnostic-websocket-frame/specs/readiness-latency-diagnostics/spec.md`
- `openspec/changes/classify-readiness-diagnostic-websocket-frame/tasks.md`

KBD registers one change and four tasks. Registration changes the project implementation denominator from 121 to 122; completing this diagnostic would move the counter from 113/122 to 114/122. It does not itself close the remaining original implementation entries.

## EXECUTION ROUND ORDER

- **Round 1:** Task 1, static preparation only.
- **Round 2:** Task 2, the sole operational observation after explicit Execute authorization.
- **Round 3:** Task 3, offline classification and recommendation.
- **Round 4:** Task 4, phase-end fixture/schema/privacy validation and independent review.

## COMMANDS TO RUN

`/opsx:new classify-readiness-diagnostic-websocket-frame` — emitted during Plan.
`/kbd-execute classify-readiness-diagnostic-websocket-frame` — next command; separately authorizes the bounded observation contract.

## Trade-off and deferred work

Reading only the framing header protects content but prevents full message validation. If the new observation differs from the prior rejection or stops before a header, this change must remain inconclusive. Handling ping/pong, fragmentation, compression, binary codecs, or any other frame form is deferred to a separately scoped product or diagnostic change after the exact observed form is reviewed.

## PLAN COMPLETE

Plan completes when the OpenSpec artifacts validate strictly, the KBD change/tasks are registered, and the waypoint points to explicit Execute authorization. No operational observation begins in Plan.
