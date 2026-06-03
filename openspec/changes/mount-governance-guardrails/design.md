## Context

`GovernanceEngine` (`engine.rs:30`) is complete and in `AppState` (`lib.rs:98`, built at `server.rs:451` from `policies/` with a permit-all `default.cedar`; falls back to `with_default_permit()` on load failure). `governance_layer` (`middleware.rs:48`) extracts `X-Agent-Id` + a path→action mapping, calls `is_allowed`, returns `403 GOVERNANCE_DENIED` on deny, and passes anonymous requests through — but it is **never mounted**. Layers are mounted in `server.rs` ~936-1038 (auth via `from_fn_with_state`, rate-limit, timeout, CORS, trace). Cedar actions (`policy.rs:119`) already include `EXECUTE_TOOL`, `CALL_LLM`, `SKILL_MUTATE`, etc. — but no `validate_output`. The chat handler `api_chat_completion` (`server.rs:3684`) has the user input pre-LLM (`extract_input_message`, ~3696) before `start_run` (~3876); HP4 added sycophancy detection at the post-stream RunDone seam (~4276). No governance/guardrail config exists in `config.rs`.

## Goals / Non-Goals

**Goals:** mount the governance layer safely (permit-all default preserves behavior); add the `validate_output` action; add in-house heuristic input screening (injection + PII) on the chat path, detect-only by default with opt-in blocking; surface flags as a metric + event without leaking content.

**Non-Goals:** replacing the `tool_requires_approval` heuristic with Cedar at the tool loop (separate change); any external moderation/guard model (R4); ML-based detection; output content moderation beyond HP4 sycophancy; per-route policy authoring (ship the permit-all default + the seam).

## Decisions

### D1 — Mount `governance_layer` after auth, unconditionally, relying on permit-all default
Add `.layer(from_fn_with_state(state.clone(), governance_layer))` in the server.rs layer stack just after auth. Safe because: `default.cedar` permits all; the chat path sends no `X-Agent-Id` (so it always passes through); only `X-Agent-Id` requests under restrictive policies are gated — which is the intended behavior.
- **Why not behind a config flag:** the task is "mount the layer"; the default is already permissive, so mounting changes nothing until an operator authors restrictive policies. A flag would add a dead toggle. *Alternative considered:* flag-gated mount — rejected as redundant given permit-all.

### D2 — Add `validate_output` action constant
One line in `policy.rs` `actions` (`pub const VALIDATE_OUTPUT: &str = "validate_output";`). Completes the vocabulary; inert until a policy uses it.

### D3 — In-house screening in a pure, testable module `src/uar/guardrails.rs`
`screen_input(text, config) -> Option<GuardrailFinding>` where `GuardrailFinding { category, reason }`. Categories: `Injection` (regex/substring set: "ignore previous instructions", "disregard (the|your) (system|previous)", "you are now", "developer mode", "reveal your (system )?prompt", etc.) and `Pii`/`Secret` (shaped patterns: `sk-`/`AKIA` API keys, 16-digit card with Luhn-ish spacing, SSN `\d{3}-\d{2}-\d{4}`). Pure (no IO); unit-tested with positive + negative cases. Reason is a short label — never echoes the matched secret value.
- **Why heuristics:** R4 (in-house, no external service); fast, deterministic, no dependency. Accepts false negatives — it is a first-line signal, not a guarantee (documented).

### D4 — Wire at the chat input seam; detect-only default, opt-in block
After `extract_input_message`, before `start_run`: if `guardrails.input_screening_enabled`, call `screen_input`. On a finding: record `uar_guardrail_flagged_total{category}`, `warn!`, and emit a `GuardrailFlagged` event. If `guardrails.block_on_injection` and the finding is `Injection` (not PII — blocking on a user's own PII is hostile), return a guardrail error response before `start_run` (no run started). Otherwise proceed.
- **Why block only Injection by default-capable path:** blocking a user sharing their own email/SSN is user-hostile; injection is the adversarial case. PII is flag-only. *Trade-off:* documented; a stricter mode is a follow-up.

### D5 — `GuardrailFlagged` event, content-safe
`NormalizedEvent::GuardrailFlagged { run_id: Option<String>, category: String, reason: String }` (run_id optional because a blocked input has no run). Map to `agui.guardrail`. Carries only category + short reason. Emitted via `emit_to_run` when a run exists; for a blocked pre-run input, the error response conveys it (no run stream to emit into).
- **Why Option run_id:** the block path has no run id; the detect-only path does.

### D6 — Minimal `GuardrailsConfig`
`{ input_screening_enabled: bool = true, block_on_injection: bool = false }`. Screening is non-blocking by default ⇒ default-on is behavior-preserving (only adds events/metrics). Blocking is opt-in.

## Risks / Trade-offs

- **[Mounting breaks deployments with restrictive policies]** an operator who already authored restrictive `.cedar` files would suddenly see HTTP denials → Mitigation: default is permit-all; only `X-Agent-Id` requests are gated; documented in the proposal; this is the intended governance behavior.
- **[Heuristic false positives/negatives]** regex screening is imperfect → Mitigation: detect-only default (no blocking), Injection-only blocking when enabled; documented as first-line, not a guarantee.
- **[Content leakage in events/logs]** flag could echo the input/secret → Mitigation: events/logs carry only category + short reason; never the matched value; explicit spec requirement + review.
- **[Latency]** screening is sync regex on the input → Mitigation: tiny/bounded; runs once pre-LLM; if ever heavy, precompile the regex set with `LazyLock`.
- **[Middleware ordering]** governance before/after auth/rate-limit → Mitigation: mount after auth (so identity context exists), before rate-limit (deny before quota) — matches the explorer's recommendation.

## Migration Plan

1. Add `GuardrailsConfig` to `config.rs` (defaults preserve behavior).
2. Add `validate_output` action const.
3. Add `src/uar/guardrails.rs` (pure screen + unit tests).
4. Add `GuardrailFlagged` event + SSE mapping + metric recorder.
5. Wire screening at the chat input seam (detect-only; opt-in block).
6. Mount `governance_layer` in the router.
7. Validate: cargo check + clippy + tests (incl. guardrails unit tests); confirm default behavior unchanged (permit-all, screening non-blocking).
- **Rollback:** additive (new middleware mount with permit-all default, new event/metric/config, new module). Revert restores the prior state.

## Open Questions

- **Block scope:** block only `Injection`, or also a configurable PII-block mode? (Lean: Injection-only for HP6; PII-block is a follow-up.)
- **Tool-loop Cedar gating:** wire `is_tool_allowed` into the orchestrator tool loop now or as the separate `tool_requires_approval`-replacement change? (Defer — keep HP6 bounded.)
- **Precompiled regex:** `LazyLock<Vec<Regex>>` vs simple substring matching? (Start with substring + a few shaped regexes; optimize only if needed.)
