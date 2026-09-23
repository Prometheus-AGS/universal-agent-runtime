## Context

See `proposal.md` for motivation and the delta spec for the behavioral contract. The preceding collector performs a valid WebSocket upgrade and sends a masked text sign-in frame, but its response path reads two bytes and immediately rejects anything other than `0x81` followed by an unmasked length byte. It then discards the header, so the historical rejection cannot be reconstructed.

This design operates only inside the active KBD child. The running services and prior diagnostic artifacts remain immutable. A later Execute request is the authority boundary for the sole network attempt.

## Goals / Non-Goals

**Goals:**

- Capture enough sanitized structure to classify one newly observed frame at the header level.
- Preserve credentials and application payloads by construction rather than by post-processing logs.
- Separate RFC 6455 header findings, old-collector compatibility, and properties that require payload or fragment history.
- Leave a reviewed minimum next action without changing protocol handling.

**Non-Goals:**

- Recover or assert the discarded historical frame.
- Decode a response, validate JSON/UTF-8, follow fragmentation, answer control frames, or resume authentication/readiness measurement.
- Add a WebSocket dependency, edit the prior collector, or change product/runtime behavior.
- Restart, repair, tune, or certify the database or UAR service.

## Decisions

### Use a child-local, dependency-free observer

Copy only the established connection/authentication mechanics into a new child-local script and replace the response decoder with a bounded framing-prefix state machine. This prevents a diagnostic need from becoming product or reusable-protocol code.

Alternative: modify the prior collector or use a full WebSocket library. Rejected because either changes accepted diagnostic history or adds broader frame handling and dependency surface before the actual frame form is known.

### Gate the attempt on exact continuity evidence

Immediately before connecting, compare the current source revision, installed executable digests, process IDs/start times, listener, relevant configuration-file digests, endpoint, and root-signin form with the inherited identity artifact. Record current values in a new allowlisted identity receipt. Any mismatch or unresolved credential provenance ends active execution before the socket opens.

Alternative: permit process restarts when binary/configuration digests match. Rejected because a restarted server may not reproduce the historical connection state, and this change has only one attempt.

### Retain derived fields, never raw framing or application bytes

The receive state machine copies only two base-header bytes plus zero, two, or eight extended-length bytes. It derives FIN, RSV bits, opcode, MASK, length encoding, declared length, and framing-byte count, then destroys the socket. If a runtime callback includes additional bytes, the callback ignores them and never places them in retained state. A server MASK bit is classifiable from the base header; the observer does not read a following mask key.

Alternative: retain the first frame or a hash for later inspection. Rejected because payload content and even a payload hash exceed the stated evidence goal and create unnecessary credential/business-content risk.

### Request no extensions and retain only extension presence

The upgrade sends no extension offer. Evidence records whether a response extension header was present, not its raw value. This makes nonzero RSV bits a header-level protocol finding for this handshake without expanding retained headers.

Alternative: retain or negotiate extension details. Rejected because extension support is not needed to classify the observed no-extension exchange and could change the server response.

### Keep protocol and collector classification separate

An offline classifier evaluates observable RFC 6455 header constraints and the old predicate independently. It records indeterminate properties for payload, UTF-8, application message, and continuation sequence. The exact observed mismatch may justify a recommendation, but implementation of that recommendation is outside this change.

Alternative: label every old-collector rejection a protocol error. Rejected because standard-valid control, fragmented, or non-text frames may still be incompatible with the restricted collector.

### Defer executable validation to the phase boundary

Tier 0 syntax and artifact checks follow edits. Synthetic framing fixtures, evidence-schema/privacy checks, strict OpenSpec validation, and independent artifact review run together after Tasks 1–3, satisfying the operator's phase-end test constraint. No product test suite runs.

## Risks / Trade-offs

- **[A different frame appears]** → Classify only the new evidence and retain an inconclusive disposition for the historical rejection; do not retry.
- **[Payload arrives in the same callback]** → Copy only the bounded framing prefix, discard the remainder without processing, and claim only zero payload bytes retained.
- **[Closing the socket does not cancel server work]** → Send only the existing sign-in operation, make no query/readiness request, and avoid claiming server-side cancellation.
- **[Header-only evidence is incomplete]** → Limit validity claims to observable framing constraints and mark payload/fragment-dependent properties indeterminate.
- **[Identity has drifted]** → Stop before connecting and request new authority instead of weakening continuity gates.
- **[Detailed evidence leaks secrets]** → Emit a fixed allowlist, cap files at 64 KiB, scan retained artifacts at phase end, and give the reviewer only sanitized artifacts.

## Migration Plan

There is no product deployment or data migration. Execute creates only child-local diagnostic artifacts. If the observer is rejected before use, no runtime rollback is needed. OpenSpec synchronization/archive and KBD phase closure occur only after Execute acceptance and Reflect, under their own workflow commands.
