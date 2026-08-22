# Decisions — `fix-container-rust-toolchain-pin-consistency`

## Iteration 1

- Decision: terminate.
- Blocking violations remaining: 0.
- Rationale: the dated selector, paired fail-closed controls, locked ARM64
  controls, full clean production image, strict OpenSpec, and bounded evidence
  requirements have observed support at their explicit limits.
- Deferred proof: the parent must rebuild the final evidence handoff SHA and
  restart its 10,800-second certification from zero.
- Dispatcher degradation: two attempts failed in the imported script before
  trigger evaluation; the empty trigger list made this a no-op failure rather
  than a missed validation or mutation.
