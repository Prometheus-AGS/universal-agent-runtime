## Context

See `proposal.md` for motivation and the delta spec for the behavior contract. The accepted prior collector can write one masked Text RPC and parse one nonempty final unmasked Text response. Its first response predicate requires `header[0] === 0x81 && (header[1] & 0x80) === 0`, and its length predicate requires `length > 0 && length <= MAX`; the independently classified `0x89 0x00` structure therefore cannot pass. The prior collector, observer and evidence are immutable inputs.

This design applies only to a host-run diagnostic artifact inside the active KBD child. A later Execute command is the authority boundary for its sole connection. The canonical diagnostic spec supplies the 15-second upgrade, 15-second first-frame-header and 30-second total active limits; this change reuses the 15-second response budget as a stricter whole-sign-in-exchange deadline.

## Goals / Non-Goals

**Goals:**

- Cross the exact observed zero-length Ping boundary while preserving one-attempt, one-response semantics.
- Keep protocol, credential and application content out of retained evidence.
- Make deadline, buffering, terminal-path clearing and cardinality behavior explicit enough to verify.
- Stop at a sanitized sign-in result and preserve database-readiness uncertainty.

**Non-Goals:**

- General WebSocket control-frame handling, fragmentation, compression, binary codecs or reusable protocol infrastructure.
- Product code, dependency, configuration, database, service lifecycle, UI, provider or realtime-state changes.
- Namespace/database selection, queries, health/readiness probes, inference, retries, recovery or release certification.

## Decisions

### Copy the collector and use a finite one-shot state machine

Copy the accepted restricted collector into the new child, then replace its one-Text response assumption with explicit states: `writing-signin`, `waiting-first`, `writing-pong`, `waiting-text`, and `terminal`. Only `waiting-first → terminal` for direct Text and `waiting-first → writing-pong → waiting-text → terminal` for the exact Ping are successful paths. There is no receive loop after `waiting-text`.

Alternative: modify the accepted collector or import a full WebSocket library. Rejected because the former rewrites accepted diagnostic history and the latter adds dependency and behavior surface unrelated to the one observed frame.

### Encode the empty Pong as a masked six-byte client frame

The Pong consists of `0x8A`, `0x80`, and four fresh random mask-key bytes. A zero-length payload has nothing to XOR, but client masking still requires the mask bit and key. The writer must settle the complete six-byte frame before transitioning to `waiting-text`; a partial or stalled write is terminal.

Alternative: send the visually simpler unmasked `0x8A 0x00`. Rejected because client-to-server WebSocket frames must be masked, including empty control frames.

### Carry one monotonic deadline through the entire sign-in exchange

Create the exchange deadline immediately before the sign-in write and pass remaining time to every write/read operation. Never create a new response timeout after Ping. A callback may buffer response bytes while the sign-in write is pending, but parsing is gated on successful write settlement and positive remaining time. If the write deadline expires, destroy the socket without interpreting or responding to buffered bytes.

Alternative: assign separate 15-second budgets to sign-in write, first response, Pong and second response. Rejected because progress could extend a one-shot observation well beyond its evidence-backed bound.

### Separate transient parsing from retained evidence

Bound Text payload accumulation at 262,144 bytes, parse only enough JSON in memory to match the RPC identifier and classify sign-in success/error, then clear it. Keep credential extraction behind the previously reviewed exact-literal gate. Enumerate every terminal path and its sensitive buffers during Task 1 source review; Task 4 verifies clearing behavior under synthetic terminal cases.

Alternative: retain redacted response text or payload hashes for debugging. Rejected because redaction can miss secrets and hashes still create a durable derivative of content that the diagnostic does not need.

### Run the behavioral suite only at the child phase boundary

Tasks 1–3 use format, syntax/static and content-free artifact checks only. Task 4 runs all synthetic behavior, privacy, size and cardinality cases together, followed by fresh-context independent review. This follows the operator's test timing constraint while preserving a complete exit gate.

Alternative: run focused synthetic tests after Task 1. Rejected for this child because the operator explicitly requires tests only at phase completion. The trade-off is that the single bounded observation occurs before behavioral fixture execution; source review, strict state/cardinality limits and the non-mutating protocol boundary mitigate but do not eliminate that risk.

## Risks / Trade-offs

- **A valid but different control-frame sequence is rejected** → keep the exact evidence-backed exception and require another separately assessed change rather than silently generalizing.
- **Buffered bytes are consumed before the sign-in write succeeds** → gate parsing on write settlement and remaining original deadline; test both buffered-success and write-timeout cases at Task 4.
- **A peer extends the attempt with repeated Ping frames** → admit at most one Ping and one following Text; any second control frame is terminal.
- **Sensitive transient buffers survive an error path** → enumerate terminal paths in source review, centralize terminal cleanup and verify each case at the phase boundary.
- **The live attempt precedes synthetic behavioral testing** → keep the attempt single, non-mutating and statically reviewed; a failure remains evidence, not a reason to retry.
- **Sign-in succeeds and is mistaken for readiness recovery** → terminate before `use` or queries and require every result artifact to state the narrower claim.

## Migration Plan

No deployment, migration or rollback is required because no product or running-service state changes are made. Keep the child collector and sanitized receipts as diagnostic history. If Task 4 fails, do not repeat the live attempt; correct the offline implementation and fixtures, preserve the original observation receipt, and qualify acceptance accordingly. Enter Reflect only after Execute acceptance.

## Open Questions

None. A different observed frame or the next database-readiness action would change scope and therefore requires a new assessed child rather than an open decision inside this design.
