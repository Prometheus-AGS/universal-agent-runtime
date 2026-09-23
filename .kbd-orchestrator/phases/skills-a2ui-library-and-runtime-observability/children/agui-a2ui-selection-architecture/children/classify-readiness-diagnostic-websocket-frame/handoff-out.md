# HANDOFF: classify-readiness-diagnostic-websocket-frame

Date: 2026-09-05

From: Execute task 4.1

Disposition: current diagnostic artifacts accepted with provenance qualification

## Delivered

- Dependency-free, single-attempt header observer: `observer.mjs`
- Passive comparison/digest continuity receipt: `evidence/continuity.json`
- Allowlist-minimized frame receipt: `evidence/frame-observation.json`
- Field-by-field RFC and old-collector analysis: `classification.md`
- Initial and final independent review receipts under `review/`
- Requirement acceptance and exact check results: `verification.md`

## Finding

The separately authorized new observation produced a final, RSV-clear, server-unmasked Ping frame with opcode 9 and declared length zero. That header is valid under the observable RFC 6455 rules. The old diagnostic collector would reject it first because it requires a final Text frame (`0x81`), and its later positive-length predicate would also reject zero if the first gate were bypassed.

This finding explains the new observer/old-collector incompatibility. It does not identify the discarded historical frame and does not establish database-readiness root cause, recovery, current readiness, or sustained readiness.

## Next authority boundary

The next lifecycle command is `/kbd-reflect classify-readiness-diagnostic-websocket-frame`. Reflection must preserve the plan-to-delivery delta: independent review found and corrected deadline, allowlist, digest, and collector-comparison defects; collection-time provenance remains independently unreconstructable.

If the operator later chooses to continue diagnosis, create a new diagnostic child. Limit its collector copy to the exact observed frame form: final, RSV-clear, unmasked Ping with declared length zero. Send the RFC-required zero-length Pong, then continue the existing bounded wait for the sign-in response. Require separate authorization for one attempt and preserve the same content-free evidence limits.

Do not generalize to arbitrary Ping/Pong payloads, fragmentation, compression, binary frames, or product-runtime support from this evidence. Do not rerun this observer, resume readiness, restart services, modify configuration/dependencies/database records, build/install, commit/push, or sync/archive OpenSpec during this Execute handoff.

## Residual risk

The collection-time observer and original receipt are unavailable as separate artifacts, so their hashes and exact minimization delta cannot be independently recomputed. Reviewer inspection cannot prove historical network counts, runtime byte handling, service continuity, credential handling, or activity outside the bounded attempt. Current observer/evidence hashes, current privacy shape, synthetic deadline behavior, and classification are verified.
