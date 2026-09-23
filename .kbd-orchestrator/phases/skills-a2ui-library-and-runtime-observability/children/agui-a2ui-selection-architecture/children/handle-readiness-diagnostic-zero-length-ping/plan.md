# PLAN: handle-readiness-diagnostic-zero-length-ping

**Project:** Universal Agent Runtime  
**Date:** 2026-09-06  
**Stage:** Plan  
**Producer model:** gpt-6-astra (frontier)  
**OpenSpec available:** YES (installed CLI 1.10.0)  
**Changes to implement:** 1 bounded diagnostic change with 4 sequential tasks  
**Baseline:** assessment at source HEAD `226d4a0af89811975662cf3f203c699f70e8cebc`; project implementation 114/122 and child implementation 0/0 before registration.

## Scope and decisions

Plan one OpenSpec change, `handle-readiness-diagnostic-zero-length-ping`, modifying the existing `readiness-latency-diagnostics` capability. The change copies the prior restricted collector into this child and adds only the exact behavior needed by the newly observed final, RSV-clear, server-unmasked, minimally encoded, zero-length Ping. Prior accepted collectors, observers and evidence remain immutable.

The receive state machine permits exactly two successful sequences: a direct final unmasked Text sign-in response, or one exact zero-length Ping followed by one final unmasked Text sign-in response. The Ping path sends a six-byte client frame—`0x8A`, `0x80`, and a fresh four-byte masking key—with no payload. It does not introduce a general receive loop, reusable WebSocket client, control-frame retry, fragmentation, compression, binary codec or product behavior.

One monotonic 15-second sign-in-exchange deadline starts immediately before the sign-in write and covers that write, the first frame, any Pong write and the single following Text response. Its provenance is the canonical `openspec/specs/readiness-latency-diagnostics/spec.md` requirement `Rejected frame observation is single-attempt and header-only`, which sets both a 15-second first-frame-header limit and a 30-second total active-observation limit. This change deliberately strengthens that existing 15-second first-frame budget to cover the whole sign-in exchange without enlarging it. The separate TCP/upgrade limit is also 15 seconds, so the upgrade and sign-in-exchange maxima fit within the unchanged 30-second outer cap. A frame already buffered while the sign-in write is pending is eligible as the first response only after the write settles and only if time remains. If the sign-in write exhausts the shared deadline, the attempt stops without consuming the response or sending Pong.

The old collector predicates were reverified from the accepted source: it first requires `header[0] === 0x81 && (header[1] & 0x80) === 0`, then requires `length > 0 && length <= MAX`. The observed `0x89 0x00` header fails both the Text-only form and the nonzero-length condition. This change replaces neither predicate in the accepted collector; it copies and narrows the state machine inside the current child.

No UI, provider compatibility, realtime entity state, product API, dependency, configuration, database record or service lifecycle changes are planned. No readiness, health, namespace/database selection, query, inference, retry, restart, build, install or deployment operation is authorized. Optional Analyze is skipped because no library selection, new dependency, architecture alternative or evolver plan is involved.

## CHANGE LIST (ordered)

1. **handle-readiness-diagnostic-zero-length-ping:** Answer one exact empty Ping during sign-in and retain bounded structural/timing evidence.
   - **Scope:** child-local diagnostic source, fixtures and evidence; OpenSpec/KBD records.
   - **Depends on:** accepted Assess handoff; explicit `/kbd-execute handle-readiness-diagnostic-zero-length-ping` before any socket connection.
   - **Recommended agent:** Codex using Astra, per operator override.
   - **Est. complexity:** M.
   - **Complexity score:** Medium—four sequential tasks and a bounded protocol state transition, with no product-module boundary.
   - **Model class:** frontier by operator override (intrinsic apply routing class: medium).
   - **Customer value:** HIGH—removes the exact protocol incompatibility that currently prevents the readiness diagnostic from reaching a sign-in result.
   - **Details:** Build a one-shot collector that either accepts Text directly or answers one exact empty Ping with a masked empty Pong and then accepts one Text response. Preserve credential, identity, deadline, privacy and cardinality gates; stop at sign-in instead of resuming database-readiness work.

## Task order and acceptance

| Task | Depends on | Deliverable and exit criteria |
|---|---|---|
| **1. Prepare the child-local collector** | None | Copy the accepted restricted collector into this child, preserving its target/credential/continuity gates and accepted artifacts unchanged. Implement a one-shot state machine for direct Text or one exact empty Ping→masked empty Pong→Text. The Pong uses a fresh four-byte mask key, the sign-in/Pong/response share one non-resetting 15-second deadline, and the 30-second active cap remains outermost. Enumerate every terminal path and confirm by source review that each path contains the required clearing calls for credentials, response content, mask keys and frame bytes. Perform only Tier 0 format and syntax/static checks; runtime verification of clearing remains explicitly deferred to the Task 4 suite. |
| **2. Make the single authorized sign-in attempt** | 1 | Only after explicit Execute authorization and fresh continuity/credential checks, make at most one WebSocket upgrade and one sign-in RPC to the confirmed existing target. Permit zero or one Pong write and zero or one subsequent Text response; stop on the first unsupported frame, deadline, write failure, continuity drift, unsafe credential source or operator interrupt. Deliver sanitized `evidence/signin-observation.json` plus exact content-free command/outcome records. No retry or readiness continuation. |
| **3. Classify the result and preserve the boundary** | 2 | Deliver `classification.md` and `execution.md`. Record whether the attempt was skipped, stopped, returned Text directly, or completed Ping→Pong→Text; validate RPC ID and sign-in success/error only in bounded memory. Retain status, sizes, timing and request/cardinality counters, not credentials, tokens, response text, raw frames, identity values or application payloads. State explicitly that sign-in does not establish database-readiness cause or recovery. |
| **4. Run phase-boundary verification and independent acceptance** | 3 | At the end of this child Execute phase, run the complete diagnostic-only synthetic suite, evidence schema/privacy/size/cardinality checks, syntax/static checks and strict OpenSpec validation. A fresh-context artifact-only critic reviews the spec, source digest, test output, sanitized evidence and classification. Resolve critical findings or retain a blocked/qualified disposition. Deliver `verification.md` and `handoff-out.md`, run KBD status, and leave Reflect as a separate command. No product test suite is run. |

Task order is strictly 1 → 2 → 3 → 4. No operational task is parallelized. A Task 1 or continuity failure makes Task 2 skipped and Tasks 3–4 document the limitation; it never authorizes a fallback client, guessed credential, broader parser, second connection or enlarged deadline.

## Exact collector contract (Execute only)

### Preconditions and budgets

- Accept exactly one occurrence of each required literal persistence field from the reviewed configuration source. Reject duplicates, interpolation, vault references, substitution, target mismatch or an unconfirmed provider/endpoint/namespace/database/user form before connecting.
- Reconfirm source, installed executable, process, listener, nonsecret target, authentication form, configuration and inherited evidence continuity. Any disagreement ends the attempt before the socket opens.
- Use one active socket, one upgrade, one sign-in write and no retry. Cap TCP/upgrade at 15 seconds, the sign-in exchange at one non-resetting 15 seconds, and total active observation at 30 seconds.
- A `SIGINT` or `SIGTERM` destroys the active socket and records only an allowlisted interrupt stop code.

### Accepted sequence and stop rules

1. Start the sign-in-exchange deadline immediately before writing the masked Text sign-in RPC. Do not inspect any response frame until that write settles. If it settles after the deadline, stop without reading or responding.
2. Parse the first complete frame from bytes already buffered or subsequently received within the original deadline.
3. Accept a direct Text frame only when it is final, RSV-clear, server-unmasked, minimally encoded and declares 1–262,144 bytes.
4. Otherwise accept only one first-frame Ping with `FIN=1`, RSV1–3 clear, opcode `0x9`, server `MASK=0`, 7-bit minimal encoding and declared length `0`.
5. Construct and completely write one masked empty Pong: first byte `0x8A`, second byte `0x80`, fresh four-byte mask key, no payload. Clear frame/key buffers after the write settles. Do not reset the deadline.
6. Parse at most one following frame. It must satisfy the same accepted Text conditions. Validate the expected RPC ID and sign-in success/error in memory, then terminate.
7. Stop immediately on a repeated or nonzero Ping, unexpected Pong, Close, continuation, fragmented Text/control frame, Binary, reserved opcode, masked server frame, nonzero RSV bit, non-minimal length, oversized Text, stalled/partial Pong write, malformed RPC response, evidence-cap violation, deadline or interrupt.

There is no general frame loop. The maximum observed frame count is two; the maximum Pong count is one; the maximum sign-in response count is one.

### Evidence and privacy

Retained evidence is capped at 65,536 bytes and may contain only allowlisted identity digests/comparisons, sanitized handshake state, structural frame classes, declared byte counts, monotonic timings, result/stop categories, and request/privacy/cardinality counters. It must not retain raw framing bytes, masking keys, payload bytes or hashes, response text, decoded tokens, credentials, raw identity values, record identifiers, environment dumps, process arguments or raw configuration.

The Text response buffer is capped at 262,144 bytes. Transient payload bytes may enter runtime buffers because sign-in must be parsed, but they are cleared and never logged, hashed or serialized. The defensible evidence claim is zero payload bytes retained, not zero payload bytes received.

### Phase-boundary suite

Task 4 covers direct Text; empty Ping then Text; exact masked-Pong byte shape; repeated Ping; nonzero Ping; unexpected Pong; Close; fragmented Text and fragmented control; Binary; reserved opcode; masked server frame; extension-marked frame; non-minimal length; oversized Text; stalled/partial Pong write; a deadline that does not reset after Ping; buffered first-frame handling after write settlement; write-deadline exhaustion without read/Pong; interrupt cleanup; and receipt privacy, 64-KiB size and cardinality assertions.

## OpenSpec and KBD records

OpenSpec artifacts:

- `openspec/changes/handle-readiness-diagnostic-zero-length-ping/proposal.md`
- `openspec/changes/handle-readiness-diagnostic-zero-length-ping/design.md`
- `openspec/changes/handle-readiness-diagnostic-zero-length-ping/specs/readiness-latency-diagnostics/spec.md`
- `openspec/changes/handle-readiness-diagnostic-zero-length-ping/tasks.md`

KBD registers one change and four tasks after the reviewed plan and strict OpenSpec validation. The registration should move project implementation from 114/122 to 114/123 and child implementation from 0/0 to 0/1; canonical runtime output, not this estimate, is authoritative.

## EXECUTION ROUND ORDER

- **Round 1:** Task 1, offline collector implementation and Tier 0 checks.
- **Round 2:** Task 2, sole operational attempt after explicit Execute authorization.
- **Round 3:** Task 3, offline classification and evidence finalization.
- **Round 4:** Task 4, all diagnostic tests and independent acceptance at the phase boundary.

## COMMANDS TO RUN

- `/opsx:new handle-readiness-diagnostic-zero-length-ping` — emitted during Plan.
- `/kbd-execute handle-readiness-diagnostic-zero-length-ping` — next command and the separate authority for the bounded socket attempt.

## Trade-off and deferred work

This exact exception can reach sign-in without importing a general WebSocket implementation, but it intentionally rejects every other control-frame sequence. A different frame or a second Ping remains unsupported even if RFC 6455 might permit it. Successful sign-in only removes this collector boundary; database-readiness attribution, recovery, sustained readiness and the remaining project implementation entries stay unresolved and require separate planning.

## Sycophancy self-check

- **S-02:** Feasibility is grounded in the accepted `0x89 0x00` observation, the old collector predicates and RFC 6455 masking/control-frame requirements; no broader compatibility is assumed.
- **S-07:** Product code, generalized frame support, retries and readiness continuation are excluded.
- **S-03:** The plan preserves the uncomfortable outcome that the one attempt can stop inconclusively and cannot establish readiness recovery.

## PLAN COMPLETE

Plan completes when independent adversarial review has vetted this document, the OpenSpec delta and task artifacts validate strictly, the KBD change/tasks are registered, and the waypoint points to explicit Execute authorization. Plan performs no socket connection or product test.
