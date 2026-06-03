# guardrail-pii-block

## Why

HP6 added input guardrails with an opt-in **injection** block (`server.rs:3732`, `block_on_injection`), but **PII findings are always flag-only** — there is no way to reject inputs containing secrets/PII before the LLM call. This is goal **S3** of `uar-safety-and-evals`: add an opt-in PII block mode, mirroring the injection block.

## What Changes

- Add `GuardrailsConfig.block_on_pii: bool` (**default false** — detect-only preserved; decision D4).
- Extend the chat-input block condition (`server.rs:3732`) to also reject when `block_on_pii` is set AND the finding category is `Pii`, reusing the existing guardrail error response + `GuardrailFlagged` emit. The combined condition: block when `(block_on_injection && Injection) || (block_on_pii && Pii)`.
- Update the `block_on_injection` doc note (PII is no longer "always flag-only").

Default behavior is unchanged (both block flags default false ⇒ detect-only). Out of scope: stronger PII heuristics, ML detection, output-side PII redaction.

## Capabilities

### Modified Capabilities
- **`request-guardrails`** — delta `specs/request-guardrails/spec.md`. Adds an opt-in PII-block requirement alongside the existing injection block; detection-only remains the default for both categories.

## Impact

- **Affected code:** `src/config.rs` (`block_on_pii` field + default), `src/server.rs` (block condition). No new dependency, no API change.
- **Behavior preservation:** default false ⇒ identical behavior; opt-in only.
- **Security (Rule 33):** lets operators reject inputs carrying secrets/PII before they reach the model. PII detection remains the existing heuristic (false negatives accepted).
- **UX caveat:** blocking PII can reject a user sharing their own data — hence default-off and operator opt-in.
- **KBD workflow state:** YES — S3 of `uar-safety-and-evals`.
