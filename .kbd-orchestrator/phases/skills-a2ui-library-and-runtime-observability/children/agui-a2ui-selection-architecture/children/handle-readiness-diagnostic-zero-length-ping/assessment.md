# ASSESSMENT: handle-readiness-diagnostic-zero-length-ping

Project: Universal Agent Runtime  
Date: 2026-09-06  
Stage: Assess  
Codebase baseline: source HEAD `226d4a0af89811975662cf3f203c699f70e8cebc`; the accepted prior observer and current evidence hashes match their recorded acceptance values.  
Cross-tool progress: no changes or tasks are registered in this child; canonical implementation is 0/0.

## IMPLEMENTATION STATUS

- **Accepted prerequisite frame classification: DONE WITH QUALIFICATION.** The prior child records a newly observed final, RSV-clear, server-unmasked, minimally encoded zero-length Ping (`opcode=9`). Its current observer, frame evidence, continuity evidence and old-collector hashes recompute to the accepted values. The discarded historical frame and collection-time pre-correction artifacts remain unavailable.
- **Existing restricted RPC collector: PARTIAL.** `collector.mjs:271-303` can send a masked Text RPC and parse one nonempty final unmasked Text response. Its first predicate requires `header[0] === 0x81 && (header[1] & 0x80) === 0`, so the observed `0x89 0x00` structure fails. Its later predicate requires `length > 0 && length <= MAX`, so zero would also fail if the first predicate were bypassed. It has no control-frame response path and must not be modified in place.
- **Header-only structural observer: PARTIAL AS INPUT ONLY.** `observer.mjs` has bounded connect, handshake, sign-in-write and frame-prefix helpers, plus exact structural classification. Its collection mode intentionally destroys the socket immediately after one header and never reads an application response, so it is not the requested Ping/Pong collector.
- **Exact zero-length Pong sender: MISSING.** No child-local implementation exists. RFC 6455 requires a Pong in response to Ping and requires every client-to-server frame to be masked. For the observed empty Ping, the minimum response is therefore a final masked Pong with opcode `0xA`, declared payload length zero, a fresh four-byte masking key and no application payload—not an unmasked two-byte `0x8A 0x00` frame.
- **Bounded Ping-then-sign-in receiver: MISSING.** No implementation admits one exact zero-length Ping, sends its Pong, then waits for one sign-in Text response under the same non-resetting deadline. A loop accepting unlimited control frames would exceed the evidence-backed scope.
- **Child evidence and acceptance: MISSING.** This child has no collector, evidence, tests, OpenSpec delta, independent review or operational receipt. No network attempt has been made.

## CROSS-TOOL PROGRESS

- `handle-readiness-diagnostic-zero-length-ping`: no registered change, no tasks, no blocker and no other-tool implementation activity.
- The generated child evidence, certification and publication dimensions are correctly `PENDING`; they do not inherit the preceding child's qualified acceptance.
- The `assess:before` memory-mirror write failed non-blockingly. Repository artifacts remain authoritative; no memory-server result is treated as evidence.

## SPEC GAP SUMMARY

- **A new spec delta is required.** The canonical `Rejected frame observation is single-attempt and header-only` requirement says the observer immediately ends the connection after deriving one header. It cannot be stretched to authorize sending Pong or reading a later Text response. Plan must add a separate exact-frame continuation requirement and scenario.
- **Client masking must be explicit.** RFC 6455 sections 5.2–5.3 require all client-to-server frames to carry the mask bit and a four-byte masking key. A zero-length payload does not remove that framing requirement.
- **Pong content equivalence is satisfiable only for the observed form.** RFC 6455 sections 5.5.2–5.5.3 require a Pong response and identical application data. Because the observed Ping length is zero, the diagnostic can comply without reading or retaining payload content. Any nonzero Ping must remain unsupported in this child.
- **The response sequence needs a hard cardinality bound.** The minimum behavior is either one accepted Text response directly, or one exact zero-length Ping followed by one accepted Text response. A second Ping, Pong, Close, fragment, binary frame, reserved opcode, masked server frame, extension-marked frame, non-minimal length or oversized Text response must stop the attempt without retry.
- **The deadline must not reset after Ping.** The canonical single-attempt frame-observation requirement supplies both a 15-second first-frame-header limit and a 30-second total active cap. Plan should justify strengthening the former into one shared 15-second sign-in-exchange deadline that starts before the sign-in write and covers first-header read, Pong write and the single following response. The 30-second cap remains the outer bound; the new delta must carry both values explicitly because the existing requirement otherwise ends at the first header.
- **Successful sign-in is the terminal boundary.** This child should validate the sign-in response shape and retain only status, byte counts and timings. It should not send `use`, queries, readiness, health or inference requests. Reaching sign-in moves the readiness investigation to a separately planned and authorized child.

Primary protocol basis: [RFC 6455 sections 5.2–5.5](https://www.rfc-editor.org/rfc/rfc6455.html). The standard identifies Ping as opcode `0x9`, Pong as `0xA`, requires a Pong response with identical application data, requires control frames to be final and no larger than 125 bytes, and requires all client-to-server frames to be masked.

## MINIMUM SAFE IMPLEMENTATION BOUNDARY FOR PLAN

1. Copy the diagnostic collector into this child; preserve the prior collector and accepted observer unchanged.
2. Reuse the accepted observer's bounded handshake, structural frame-prefix parsing, deadline-aware write pattern, allowlist-only continuity evidence and exclusive receipt creation.
3. Add one exact branch for `FIN=1`, `RSV1–3=0`, `opcode=9`, server `MASK=0`, minimal 7-bit encoding and declared length `0`.
4. Construct a six-byte client Pong frame: `FIN/opcode=0x8A`, `MASK/length=0x80`, followed by a fresh four-byte masking key. Send no payload and scrub the frame/key buffers after the write settles.
5. Permit at most one Ping/Pong exchange. The allowance applies to the first frame made available after the sign-in write settles, including a frame that arrived and was buffered while that write was pending. If the write itself exhausts the shared deadline, stop without reading or sending Pong. Otherwise continue to one subsequent final, RSV-clear, unmasked Text response under the original deadline. If Text is the first response, accept the existing path without sending Pong.
6. Parse only a Text response whose declared length is 1–262,144 bytes, match the RPC ID and determine sign-in success/error. Do not retain token, response text, payload bytes, credentials, raw frames or raw identity values. Retained evidence remains capped at 65,536 bytes.
7. Reuse the reviewed literal-configuration credential gate: accept only one occurrence of each required persistence field, reject interpolation and vault references, require the confirmed provider/endpoint/namespace/database/user form, keep the password only in memory and clear credential-bearing buffers on every completion path. Any duplicate, nonliteral, substituted or target-mismatched value is an unsafe credential source and stops before connection.
8. Stop at sign-in success/error or the first different frame, deadline expiry, continuity drift, credential-gate failure, declared Text length above 262,144 bytes, evidence serialization above 65,536 bytes, or `SIGINT`/`SIGTERM`. An interrupt must destroy the active socket and retain only its sanitized stop code. Never retry.
9. Run syntax/static checks during implementation and the diagnostic-only synthetic suite at this child phase boundary. Required cases are: direct Text; zero-length Ping then Text; exact masked-Pong byte shape; repeated Ping; nonzero Ping; unexpected Pong; Close; fragmented Text/control; Binary; reserved opcode; masked server frame; extension-marked frame; non-minimal length; oversized Text; stalled Pong write; non-resetting 15-second exchange deadline within the 30-second total cap; interrupt cleanup; and receipt privacy, size and cardinality assertions.

## BUILD HEALTH

- build check: **UNKNOWN** — Assess ran no build or product test. The scoped product-path status is empty, but that does not certify repository-wide build health.
- prior diagnostic artifact check: **PASS AS HISTORICAL EVIDENCE** — the preceding child recorded `node --check` and 7/7 synthetic observer cases. Those cases did not implement or test Pong-and-continue behavior.
- known violations: the stage-gate's implicit phase-directory resolver selected the top-level phase for this grandchild and reported a false canonical mismatch. Supplying the explicit child directory passed the same gate. No waypoint or projection was hand-edited.
- test coverage: **NONE for this child** — no collector or child test exists yet. Coverage percentage is not measured.

## CONSTRAINT CHECK

- AGENTS.md violations: **NONE OBSERVED in Assess.** No product source, configuration, dependency, database record, service lifecycle, UI, build workflow or kernel capability was changed. Scoped repository status is evidence only for the named product paths, not a universal external-activity audit.
- child scope: **COMPLIANT.** Writes are limited to this child and KBD-generated state. OpenSpec paths are not currently authorized; Plan must request or record a scope extension before creating `openspec/changes/handle-readiness-diagnostic-zero-length-ping/**`.
- capability inversion: **NOT TOUCHED.** The proposed work is a host-run diagnostic artifact, not an agent-kernel write path.
- constraints.md: **N/A.** No phase-local constraints file exists.
- verification tier: **COMPLIANT.** Assess performed read-only inspection and format checks only. Product tests remain deferred to their proper boundary; diagnostic synthetic checks belong at this child phase boundary after implementation.

## GOAL PROGRESS

1. **PARTIAL** — the exact minimum collector boundary is assessed, including the masked six-byte Pong and one-Ping cardinality limit, but no Plan or collector exists.
2. **NOT MET** — no Execute authorization or network attempt occurred, and no sign-in response was reached.
3. **PARTIAL** — privacy, stop, evidence and independent-review boundaries are identified, but no child evidence or acceptance exists.

## RISKS AND UNCOMFORTABLE FINDING

The Pong looks like a two-byte empty control frame, but that would be wrong for a client: masking makes the actual empty Pong six bytes. The more dangerous implementation error is an apparently harmless receive loop that resets its timeout after every control frame. That would turn one bounded, evidence-specific exception into generalized frame handling and allow a peer to extend the diagnostic indefinitely. The Plan should permit exactly one observed zero-length Ping and should carry one deadline across sign-in write, Pong and response.

Even a successful sign-in would only remove this collector compatibility boundary. It would not identify the discarded historical frame, prove database-readiness root cause, show recovery, establish sustained readiness or close any of the eight currently remaining project implementation entries.

## SYCOPHANCY SELF-CHECK

- S-02: the proposed Ping handling is not accepted because it was recommended previously; it is constrained by the observed frame, current source and RFC requirements.
- S-03: the assessment names missing implementation, missing spec authority, missing tests, the masking trap, the deadline-extension risk and unavailable historical provenance.
- S-06: no conclusion relies on “clearly” or “obviously.”
- The `detect_sycophancy` MCP tool is unavailable in this session. No detector score is fabricated. Two isolated `k3` adversarial reviews passed with no critical findings; their warnings and resolutions are retained under `review/assess/`.

## ASSESSMENT COMPLETE

Assess supports proceeding to Plan, not Execute. Plan must create one new OpenSpec delta, register bounded tasks, keep the prior artifacts immutable and require a separate Execute command before any socket connection. The final review warnings were corrected after the configured two-review cap and were not independently re-vetted; Plan must reverify the quoted collector predicates, deadline provenance and buffered-first-frame rule before registering work.
