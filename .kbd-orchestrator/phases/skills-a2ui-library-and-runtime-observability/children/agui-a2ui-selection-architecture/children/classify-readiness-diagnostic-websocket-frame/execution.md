# EXECUTION: classify-readiness-diagnostic-websocket-frame

**Project:** Universal Agent Runtime
**Date:** 2026-09-05
**Selected backend:** openspec
**Dispatched to:** Codex SELF
**Backend rationale:** The change is already represented by a strict-valid OpenSpec delta and four ordered tasks; self-execution keeps the one-attempt operational boundary and KBD progress inspectable.
**Backend entrypoint:** `/kbd-execute classify-readiness-diagnostic-websocket-frame`
**OpenSpec available:** YES
**Source plan:** `.kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/agui-a2ui-selection-architecture/children/classify-readiness-diagnostic-websocket-frame/plan.md`

## EXECUTION SCOPE

- `classify-readiness-diagnostic-websocket-frame`: prepare a child-local observer, make at most one authorized sign-in/header observation, classify it offline, and independently review the evidence.

## DISPATCH CONTRACT

- Canonical task progress: `openspec/changes/classify-readiness-diagnostic-websocket-frame/tasks.md`.
- Canonical workflow progress: this child's generated `progress.json`, updated only through `prometheus kbd` commands.
- Work strictly in task order `1.1 → 2.1 → 3.1 → 4.1`.
- Run `kbd-status` after every completed task, the completed change, and the completed Execute stage.
- Preserve unrelated working-tree changes and never modify the accepted prior collector.

## APPROVAL GATES

- The user's `/kbd-execute classify-readiness-diagnostic-websocket-frame` command authorizes the plan's one WebSocket upgrade and one root-signin/header observation after Task 1.1 passes.
- It does not authorize readiness, health, namespace/database selection, queries, inference, retries, restarts, product edits, configuration changes, dependency changes, or database-record mutations.
- Any continuity mismatch or inability to enforce the header-only boundary stops the operational attempt.

## FALLBACK CONDITIONS

- No alternate backend, WebSocket library, client, credential source, endpoint, retry, or expanded evidence budget is permitted.
- A prerequisite failure produces explicit SKIPPED evidence and an inconclusive handoff.

## VERIFICATION REQUIREMENTS

- During Tasks 1–3: Tier 0 syntax, JSON parsing, bounded-file, and artifact-format checks only; no product tests.
- At Task 4 only: synthetic observer/parser fixtures, evidence schema/size/privacy checks, strict OpenSpec validation, and fresh-context artifact-only review.
- Verify retained counters show at most one upgrade/sign-in and zero query, readiness, payload-retention, credential-retention, product-test, and service-mutation operations.

## PROGRESS LEDGER

- [DONE] `classify-readiness-diagnostic-websocket-frame` — Codex SELF — 4/4 tasks complete; accepted with a collection-provenance qualification.

## OUTPUTS

- Child-local observer and sanitized continuity/frame evidence.
- `classification.md`, `verification.md`, and `handoff-out.md`.
- OpenSpec and generated KBD progress receipts.

## BLOCKERS

- None before Task 1.1. The active attempt remains conditional on continuity and observer checks.

## REFLECTION HANDOFF

- Compare the exact plan with attempted/skipped operations, new-frame classification, reviewer findings, retained uncertainty about the historical frame, and any separately authorized next action. Do not equate diagnostic acceptance with readiness recovery.

## EXECUTION READY

Task 1.1 is the only current task. No network request begins until its continuity and observer-preparation exit criteria pass.

## TASK RECORDS

### Task 1.1 — complete

- `observer.mjs` was created under this child and passed `node --check`.
- Passive preparation made zero network requests and wrote `evidence/continuity.json` (3,190 bytes).
- Source HEAD `226d4a0af89811975662cf3f203c699f70e8cebc`, both service PIDs/start times, executable digests, listeners, four configuration digests, and the two inherited evidence hashes matched the accepted baseline.
- The exact forbidden-key/value scan was clean. No credential value or raw configuration was retained.
- Task 2.1 is enabled for the single authorized collection attempt.

### Task 2.1 — complete

- The observer ran exactly once under the user's Execute authorization and exited successfully without retry.
- The retained receipt records one WebSocket upgrade, one root-signin request, one frame header, and zero database query, readiness, health, inference, record-mutation, service-mutation, product-test, or retry operations.
- The negotiated handshake was HTTP 101 with `json` subprotocol and no requested or returned extension.
- The observed two-byte framing prefix classified structurally as a final, unmasked ping control frame (`opcode 9`) with all RSV bits clear, 7-bit minimal length encoding, and declared length zero.
- The allowlist-minimized receipt is 3,368 bytes. It retains only identity digests/comparisons, the nonsecret host/port/path and authentication form, bounds, structural handshake/header fields, elapsed time, stop reason, and counters. It retains no raw PID, start time, executable/configuration path, listener, namespace/database name, payload byte, raw framing byte, credential value, authorization label, or historical-frame assertion. The observer intentionally read no payload and destroyed the socket after the header.
- This new observation does not identify the discarded historical frame; Task 3.1 will classify only the retained observation and state that uncertainty explicitly.

### Task 3.1 — complete

- `classification.md` maps each retained handshake and frame field to RFC 6455 sections 5.2–5.6 and, separately, to the prior collector's `0x81`/unmasked predicate.
- The new frame is header-valid and collector-incompatible: opcode `9` is a defined Ping while the restricted collector accepts only opcode `1` Text at its first check; if that first gate were bypassed, its separate `length > 0` check would also reject this zero-length control frame.
- The classification preserves the unknown historical frame and leaves payload, later frame sequence, sign-in response, readiness latency, root cause, recovery, and sustained readiness unproved.
- The minimum next action is a separate diagnostic child that handles only the exact observed zero-length Ping by sending the required zero-length Pong before continuing the existing bounded sign-in wait. No fix or readiness attempt was performed here.

### Task 4.1 — complete

- Phase-end diagnostic checks passed: observer syntax; 7/7 synthetic cases including a stalled sign-in-write deadline; exact observation and continuity schemas; 64-KiB caps; privacy/counter assertions; strict OpenSpec validation; whitespace and scoped diff checks.
- Initial independent review blocked on stale digests and a missing stalled-write deadline. A second review found the old collector's later positive-length gate, the narrower 15-second budget gap, and raw identity fields outside the retained allowlist. All blocking findings were corrected.
- Final fresh-context artifact-only review reported no remaining findings and returned `QUALIFIED PASS`.
- The retained qualification is that the collection-time observer and pre-minimization receipt were superseded, so their historical hashes and exact field-removal delta cannot be independently reconstructed.
- `verification.md`, `handoff-out.md`, and both review receipts record the accepted result, limits, and next-authority boundary.
- Scoped product-path status was empty and the observer receipt records zero service/database mutations and zero product tests. This is not a complete audit of external activity.
- The generic Artifact Refiner validator was not fabricated: this child has no PMPO manifest/constraints/iteration state and its approved scope excludes `.refiner/`; the required plan-specific independent critic supplied the applicable QA gate.
