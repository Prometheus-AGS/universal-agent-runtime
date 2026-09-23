# ASSESSMENT: classify-readiness-diagnostic-websocket-frame

**Project:** Universal Agent Runtime

**Date:** 2026-09-05

**Stage:** Assess

**Baseline:** `226d4a0a`
**Canonical phase:** `skills-a2ui-library-and-runtime-observability::agui-a2ui-selection-architecture::classify-readiness-diagnostic-websocket-frame`

## IMPLEMENTATION STATUS

### Established

- The preceding diagnostic preserved immutable evidence for the failed attempt. Its observation artifact has SHA-256 `dbcac041a81e21775302e56a629daac3dd06eb7fdccb6229b928bfcef83abf09`; the collector has SHA-256 `5dbbe09b51e705681d4d9c4be0da8315500df2172095ed094c14888a9dd0f467`.
- The WebSocket opening handshake completed in 353.701 ms and negotiated subprotocol `json`.
- The local collector then rejected the first frame read during sign-in after 0.748 ms with `unsupported_websocket_frame`.
- The rejection occurred before any authentication reply was decoded and before namespace selection, database selection, queries, or `readyz` measurement.
- The existing collector accepts only an unmasked, final text-frame first byte pair at its initial check: `header[0] === 0x81 && (header[1] & 0x80) === 0`. Length decoding and payload reading happen only after that check.

### Missing

- No FIN, RSV, opcode, MASK, or declared-length fields were retained for the rejected historical frame.
- No sanitized classifier artifact exists.
- No evidence establishes which structural condition caused the collector rejection.
- No independent review of an actual frame classification exists.
- No evidence establishes that a future connection would emit the same frame form as the historical connection.

The uncomfortable finding is that the exact historical frame cannot be classified from the retained evidence. A separately authorized observation could classify a newly observed corresponding frame; it could not retroactively prove the structure of the discarded historical frame.

## CROSS-TOOL PROGRESS

- The preceding child `diagnose-database-readiness-latency` completed one diagnostic change with five tasks and was accepted as `PASS_QUALIFIED_INCONCLUSIVE`.
- The current child has no registered OpenSpec change and no implementation tasks.
- Project-level implementation remains 113/121.
- Existing project evidence, certification, or publication projections do not certify this new child.

## SPEC GAP SUMMARY

The canonical `readiness-latency-diagnostics` specification permits a bounded, explicitly authorized diagnostic attempt and requires stop-on-first-error, response/privacy caps, no automatic retry, no product mutation, and independent review. It does not yet define a header-only WebSocket frame-classification artifact.

The child goal names FIN, opcode, MASK, and length. That set is insufficient for a protocol-validity classification because RFC 6455 framing also assigns semantic meaning to RSV1–RSV3, and validity can depend on negotiated extensions. A future plan must either:

1. extend the sanitized schema to FIN, RSV1–RSV3, opcode, MASK, and declared length, while retaining no payload, credentials, raw application message, or header bytes; or
2. narrow the result to collector-compatibility classification and explicitly avoid claiming full RFC validity.

The first option is the minimum evidence needed for the stated protocol-classification goal. It remains structural metadata and does not broaden protocol handling.

A new observation would be an operational retry, even if it stops immediately after reading structural header fields. Assess does not authorize that connection. The Plan stage must define the exact one-attempt bound, identity-continuity checks, artifact caps, stop condition, and independent-review input before requesting Execute authorization.

Protocol reference: RFC 6455 sections 5.2–5.6, <https://www.rfc-editor.org/rfc/rfc6455>.

## BUILD HEALTH

- No build, product test, readiness probe, database query, or service restart was run during Assess.
- Current build health is not re-certified by this assessment.
- The prior collector and observation hashes are inherited evidence, not newly executed verification.
- Coverage for a proposed header classifier is unknown because no implementation exists.
- Per project policy, product tests remain deferred to a phase boundary.

## CONSTRAINT CHECK

- The assessment stayed within the current child write scope.
- No product code, configuration, dependency, database, service, or prior diagnostic artifact was changed.
- No readiness attempt or automatic retry was performed.
- No defensive guard or broadened WebSocket behavior was introduced.
- Registering a spec-driven OpenSpec change will require a narrow scope extension for `openspec/changes/classify-readiness-diagnostic-websocket-frame/**`.
- There is no child-specific `constraints.md`; inherited project and parent constraints remain controlling.

## GOAL PROGRESS

1. **Capture sanitized structural metadata — NOT MET.** The historical attempt did not retain it, and no new observation is authorized.
2. **Classify the observed frame — PARTIAL.** The exact collector rejection predicate and the classification fields are known; the actual frame values are not.
3. **Produce reviewed evidence and a minimum next action — NOT MET.** No new evidence or independent classification review exists.

## ASSESSMENT COMPLETE

The next justified action is `/kbd-plan classify-readiness-diagnostic-websocket-frame`. The plan must preserve the historical-frame uncertainty, define a child-local collector copy rather than mutate the prior collector or product, register a narrow spec delta, capture only structural fields required for classification, stop before payload retention or readiness continuation, and require independent review. Execute must remain separately authorized.
