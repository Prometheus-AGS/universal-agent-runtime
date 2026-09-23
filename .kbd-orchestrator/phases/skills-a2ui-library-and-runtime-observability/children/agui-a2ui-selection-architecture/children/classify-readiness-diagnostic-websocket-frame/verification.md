# VERIFICATION: classify-readiness-diagnostic-websocket-frame

Date: 2026-09-05

Task: Execute 4.1

Result: ACCEPTED — QUALIFIED PASS

## Acceptance

The current artifacts independently support a header-valid, prior-collector-incompatible classification for the newly observed final, RSV-clear, server-unmasked, zero-length Ping frame. Acceptance covers the bounded diagnostic observer, allowlist-minimized evidence, RFC 6455 header analysis, exact old-collector comparison, historical uncertainty, and narrow next-authority boundary. It does not establish the discarded historical frame, database-readiness root cause, recovery, current or sustained readiness, or product-runtime requirements.

The initial review blocked on stale digests and a missing total-deadline guard around a stalled sign-in write. A second review found the old collector's later positive-length predicate, a narrower 15-second deadline gap, and raw identity values beyond the evidence allowlist. All blocking findings were corrected. Final re-review found no remaining findings and returned `QUALIFIED PASS`; receipts are `review/frame-classification-initial.md` and `review/frame-classification-final.md`.

The qualification is provenance-specific. The collection-time observer and pre-minimization receipt were superseded rather than preserved as separate files, so their historical hashes and exact field-level delta cannot be independently reconstructed. The current artifacts and hashes are verified.

## Requirement mapping

| Requirement | Evidence | Result and limit |
|---|---|---|
| Single attempt and deadlines | `observer.mjs`; `evidence/frame-observation.json` | PASS for the current observer and retained attempt counters. Exclusive receipt creation prevents rerun; one connection site and no retry loop exist. One shared 15-second deadline covers sign-in write plus first header, bounded by 30 seconds total. The recorded attempt completed in 978.377 ms. Historical execution counts remain receipt claims rather than independently observed reviewer facts. |
| Structural, content-free evidence | Both evidence JSON files | PASS for current artifacts. Files are 2,245 and 3,368 bytes, below 64 KiB. They retain identity digests/comparisons, the allowed nonsecret endpoint, bounds, handshake/header metadata, elapsed time, stop reason, and counters. Payload, raw framing bytes, raw process/configuration identity, and credential values are absent. Runtime buffer behavior cannot be reconstructed from an artifact-only review. |
| RFC and collector classification | `classification.md`; prior `collector.mjs` | PASS. RFC 6455 sections 5.2–5.6 classify the new `0x89 0x00` structure as a valid Ping header. The old collector would first reject opcode 9 at its `0x81` gate and, counterfactually, reject declared length zero at its later `length > 0` gate. This is collector incompatibility, not a server protocol violation. |
| Historical uncertainty and next action | `classification.md` | PASS. The discarded historical frame remains unknown. The recommendation is limited to a separate diagnostic-only collector copy handling this exact zero-length Ping with a zero-length Pong before continuing an independently bounded sign-in wait. No product support or readiness rerun occurred. |
| Independent acceptance | Review receipts | QUALIFIED PASS with no remaining findings. The qualification is limited to unavailable collection-time provenance and the intrinsic limits of artifact-only negative claims. |

## Phase-end checks

- `node --check observer.mjs`: exit 0, no output.
- `node observer.mjs --self-test`: 7/7 synthetic cases passed: short text, empty Ping, fragmented extended text, 64-bit binary length, masked server frame, non-minimal 16-bit length, and stalled sign-in-write frame deadline.
- Final `jq -e` observation schema/counter assertion: `true`.
- Final `jq -e` continuity comparison/digest assertion: `true`.
- Final `jq -e` privacy assertion, with the explicitly permitted endpoint `path` distinguished from prohibited raw configuration/executable paths: `true`.
- Evidence size assertions: pass at 2,245 and 3,368 bytes, each below 65,536 bytes.
- `openspec validate classify-readiness-diagnostic-websocket-frame --strict`: `Change 'classify-readiness-diagnostic-websocket-frame' is valid`.
- Scoped trailing-whitespace scan and `git diff --check`: no output.
- `git status --short -- src frontend Cargo.toml Cargo.lock package.json pnpm-lock.yaml`: no output.
- No product test suite ran. The seven synthetic cases validate only the diagnostic artifact at this planned child phase boundary.

The first privacy scan treated the allowed WebSocket endpoint key `target.path` as though it were a prohibited raw filesystem/configuration path and returned `false`. The corrected exact scan exempted only the two allowlisted `/rpc` endpoint fields and passed. This does not weaken the prohibition on raw executable, configuration, identity, or environment paths.

## Artifact Refiner applicability

The generic `refine-validate` skill was evaluated because KBD calls for per-change QA on multi-artifact work. Its schema requires an existing PMPO `artifact_manifest.json`, `constraints.json`, iteration log, and decisions record under `.refiner/`. This diagnostic child has no PMPO refinement state, and its approved write scope excludes `.refiner/`. The generic validator therefore cannot produce a valid schema/consistency result for this change and was not fabricated or routed around the scope boundary. The plan-specific fresh-context artifact critic is the applicable acceptance gate recorded above.

## Goal disposition

1. MET for the separately authorized new observation — sanitized structural metadata was captured without retained payload or credential values. The discarded historical frame remains unknown.
2. MET — the new frame is classified separately under RFC 6455 and the old collector behavior without implementing broader handling.
3. MET WITH QUALIFICATION — independent review accepted the current artifacts and minimum recommendation; historical provenance cannot be independently reconstructed.

No product guard was added. The diagnostic observer's continuity, file-size, one-attempt, deadline, framing-prefix, and privacy guards trace to the explicit plan and the prior unsupported-frame stop. No added guard is speculative.

The scoped checks do not prove universal absence of external activity. They support only the named repository paths and the retained bounded-attempt counters.
