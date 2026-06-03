## Context

`GuardrailsConfig { input_screening_enabled, block_on_injection }` (`config.rs`); the chat input seam blocks at `server.rs:3732` only when `block_on_injection && finding.category == Injection`. `guardrails::screen_input` already returns `Pii` findings; they are flagged (event + metric) but never block.

## Goals / Non-Goals
**Goals:** opt-in PII block mirroring injection; default-off; reuse the existing block path + event.
**Non-Goals:** better PII heuristics; output redaction; ML detection.

## Decisions
- **D1 — add `block_on_pii: bool` (default false)** to `GuardrailsConfig`; behavior-preserving.
- **D2 — combined block condition:** `(block_on_injection && Injection) || (block_on_pii && Pii)`. Keep the single existing guardrail-error response + the post-`start_run` `GuardrailFlagged` emit (flagging is unchanged; only the block gate widens).
- **D3 — keep PII detection as-is** (heuristic); this change only adds the block decision.

## Risks / Trade-offs
- **[False-positive blocking]** PII heuristic could reject legitimate user data → Mitigation: default-off, operator opt-in, documented.
- **[Behavior change]** none by default (both flags false).

## Migration Plan
1. Add `block_on_pii` (default false) to config.
2. Widen the block condition in `server.rs`.
3. `cargo check`/`clippy`/tests; manual: enable `block_on_pii`, send PII input → rejected; default → flagged-only.
- Rollback: additive; revert restores injection-only block.

## Open Questions
- None (bounded change).
