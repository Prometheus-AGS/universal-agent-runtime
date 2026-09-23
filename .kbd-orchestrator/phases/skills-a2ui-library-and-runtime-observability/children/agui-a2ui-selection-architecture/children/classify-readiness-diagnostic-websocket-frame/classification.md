# WebSocket frame classification

**Observation:** 2026-09-05T21:30:26.441Z–2026-09-05T21:30:27.428Z

**Collection observer SHA-256 at the live attempt:** `2795b152624566ef0f78fdf99d10e17ea4e3d77d042868209fa9e407a911947c`

**Acceptance observer SHA-256 after deadline/privacy corrections:** `0d54cdbdcd409a68db07201f1c456d23297bf5708aa3e7febf6903e9894e1fbb`

**Evidence SHA-256:** `95c915f92378c6630451eeb5405df11519ca8b2176a6369c6f42005dd2caa58f`

**Protocol reference:** [RFC 6455 sections 5.2–5.6](https://www.rfc-editor.org/rfc/rfc6455#section-5.2)

## Disposition

The newly observed frame is **header-valid** under the observable RFC 6455 constraints and **collector-incompatible** with the prior diagnostic collector. It is a final, unmasked, zero-length Ping control frame. RFC 6455 defines opcode `0x9` as Ping, permits a server to send Ping after the connection is established, requires control frames to be final and no larger than 125 bytes, and requires server-to-client frames to be unmasked.

This is not a complete protocol-validity finding. The observer retained no application content and did not continue the frame sequence. It also does not recover or prove the discarded historical frame; that frame remains unknown.

The live attempt created a 4,598-byte receipt with SHA-256 `9a6a38ddec7b0ac1b5bddb23f23fceb613d3354a5e29a8910dde89eecaf3e2c2`. Phase-end review removed only non-allowlisted authorization/history labels and raw process/configuration identity values; it did not change the retained handshake, frame, counter, timestamp, limit, elapsed-time, stop-reason, or nonsecret endpoint fields. The current minimized receipt is 3,368 bytes and has the evidence digest above. The observer was then hardened for stalled-write deadlines and future allowlisted output, but the exclusive receipt prevents another live attempt.

## Retained handshake fields

| Retained field | Value | Classification | Basis |
|---|---:|---|---|
| `handshake.httpStatus` | `101` | Consistent with a successful WebSocket upgrade | Evidence receipt; RFC 6455 opening handshake |
| `handshake.negotiatedSubprotocol` | `json` | Negotiated application subprotocol; it does not restrict protocol control frames | Evidence receipt; RFC 6455 section 1.3 |
| `handshake.extensionsRequested` | `false` | No extension semantics requested | Evidence receipt |
| `handshake.extensionsResponsePresent` | `false` | No extension semantics apply to reserved bits | Evidence receipt; RFC 6455 sections 5.2 and 5.8 |

## Retained frame fields

| Retained field | Value | RFC 6455 header-level finding | Prior collector finding |
|---|---:|---|---|
| `frame.fin` | `true` | Valid and required for a control frame, which must not be fragmented | Matches the collector's required FIN bit |
| `frame.rsv1` | `false` | Valid with no negotiated extension | Matches the collector's required zero RSV bits |
| `frame.rsv2` | `false` | Valid with no negotiated extension | Matches the collector's required zero RSV bits |
| `frame.rsv3` | `false` | Valid with no negotiated extension | Matches the collector's required zero RSV bits |
| `frame.opcode` | `9` | Defined Ping control-frame opcode | **Mismatch:** the collector requires opcode `1` (Text) through `header[0] === 0x81` |
| `frame.opcodeClass` | `ping` | Consistent with opcode `9` | Unsupported by the collector's first-frame predicate |
| `frame.masked` | `false` | Valid for a server-to-client frame | Matches `(header[1] & 0x80) === 0` |
| `frame.lengthEncoding` | `7-bit` | Correct form for lengths 0–125 | Its decoder supports this form, but the actual run stopped at the earlier opcode gate |
| `frame.declaredLength` | `0` | Within the control-frame maximum of 125 | **Second incompatibility:** if the opcode gate were bypassed, `length > 0` would reject zero at collector line 293 |
| `frame.minimalLengthEncoding` | `true` | Satisfies the minimum-length-encoding requirement | The collector has no minimal-encoding check; this form would decode directly to zero before its positive-length check |
| `frame.lengthMostSignificantBitZero` | `true` | Consistent; the special 63-bit constraint is not engaged by 7-bit encoding | Not checked for this 7-bit frame; the collector's 64-bit branch uses only its own maximum-response bound |
| `frame.framingBytesConsumed` | `2` | Exactly the base header needed for a 7-bit, unmasked frame | Given the reconstructed `0x89 0x00` header, the old collector would consume these two bytes and stop at its first gate |

The new observer captured the header instead of applying the old collector's gate. Had the old collector processed this new frame, it would have stopped first at opcode `9` versus its required opcode `1`. Full collector compatibility has a second, counterfactual mismatch: even if the frame passed the first gate, its declared length `0` would fail the later `length > 0` response-size predicate. FIN, RSV1–3, and MASK match the first gate; the collector does not implement the RFC minimal-encoding and 64-bit-MSB checks as such.

## Bounds and indeterminate properties

The evidence records one WebSocket upgrade, one root-signin request, one observed header, and zero retries. It also records zero database queries, readiness or health requests, inference requests, record or service mutations, product tests, intentionally read payload bytes, retained payload bytes, retained raw framing bytes, and retained credential values.

The following remain indeterminate or outside what this header can prove:

- the structure and content of the discarded historical frame;
- whether a Pong response followed by continued bounded reading would produce the sign-in response;
- the sign-in response's frame type, payload, JSON, UTF-8, application validity, or latency;
- any preceding or following data-frame fragmentation state;
- database readiness latency, root cause, recovery, or sustained readiness;
- complete WebSocket connection validity beyond the retained handshake and one frame header.

Because the observed frame declares length zero, there is no Ping application body to echo. That fact comes from the retained header; the observer still intentionally read and retained no payload.

## Minimum evidence-justified next action

If the operator wants to continue the readiness diagnosis, create a separately scoped diagnostic child that changes only a copy of the diagnostic collector to accept this exact header-valid frame form: an unmasked, final Ping with all RSV bits clear and declared length zero. The diagnostic client should send the required zero-length Pong and continue its existing bounded wait for the sign-in response. It should retain the same privacy and one-attempt limits and stop again before any product change.

Do not add general Ping, Pong, fragmentation, compression, binary-frame, or product-runtime support from this evidence. Do not describe the Ping as the database-readiness root cause: it explains the new collector rejection, not the discarded historical rejection or the readiness latency.
