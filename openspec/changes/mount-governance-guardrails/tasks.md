# Tasks — mount-governance-guardrails

## 0. Bootstrap

- [ ] 0.1 Confirm engine in AppState (`lib.rs:98`), boot permit-all (`policies/default.cedar`), `governance_layer` unmounted, `call_llm` action exists, chat input seam (`server.rs` `api_chat_completion`)
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Config

- [ ] 1.1 Add `GuardrailsConfig { input_screening_enabled: bool = true, block_on_injection: bool = false }` to `config.rs`; expose `pub guardrails: GuardrailsConfig` on `AppConfig` (defaults preserve behavior)

## 2. Governance action vocabulary

- [ ] 2.1 Add `pub const VALIDATE_OUTPUT: &str = "validate_output";` to `policy.rs` `actions` (`call_llm` already exists)

## 3. Guardrails module (pure, testable) — src/uar/guardrails.rs

- [ ] 3.1 `GuardrailCategory` (Injection | Pii) + `GuardrailFinding { category, reason }`
- [ ] 3.2 `screen_input(text, cfg) -> Option<GuardrailFinding>` — injection substring/regex set + PII/secret shaped patterns; reason is a short label (never the matched value); pure, no IO
- [ ] 3.3 Register `pub mod guardrails;` in `src/uar/mod.rs`
- [ ] 3.4 Unit tests: injection positives, PII/secret positives, clean negatives, disabled no-op

## 4. Event + metric

- [ ] 4.1 `NormalizedEvent::GuardrailFlagged { run_id: Option<String>, category: String, reason: String }` (events.rs); map to `agui.guardrail` (sse.rs)
- [ ] 4.2 `record_guardrail_flagged(category: &str)` → `uar_guardrail_flagged_total{category}` (metrics.rs)

## 5. Wire screening at chat input seam (server.rs)

- [ ] 5.1 After `extract_input_message`, before `start_run`: when `guardrails.input_screening_enabled`, call `screen_input`
- [ ] 5.2 On finding: record metric, `warn!`, emit `GuardrailFlagged` (via `emit_to_run` when a run exists; for the block path, return it in the error)
- [ ] 5.3 If `block_on_injection` and finding is `Injection`: reject before `start_run` with a guardrail error (no run started). PII is flag-only.
- [ ] 5.4 Never log/emit the raw input or matched secret value

## 6. Mount the Cedar governance layer (server.rs)

- [ ] 6.1 Add `.layer(from_fn_with_state(state.clone(), governance::middleware::governance_layer))` after auth in the router stack
- [ ] 6.2 Confirm permit-all default ⇒ existing behavior unchanged (chat path has no `X-Agent-Id`; anonymous passes through)

## 7. Validation (gate)

- [ ] 7.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 7.2 `cargo clippy` — no new warnings in touched files
- [ ] 7.3 `cargo test --features postgres-backend --lib` — existing pass + new guardrails unit tests
- [ ] 7.4 Manual: an injection input emits `agui.guardrail` (+ blocks when enabled); a clean input is unaffected; restrictive policy → 403 for an `X-Agent-Id` request (pending live env — document if not runnable here)
- [ ] 7.5 `openspec validate mount-governance-guardrails --strict`; update `.kbd-orchestrator` progress

## Notes

- **Safe by default:** permit-all `default.cedar` + detect-only screening (default) ⇒ no behavior change out of the box. Blocking + restrictive policies are opt-in.
- **In-house only (R4):** regex/substring heuristics; no external moderation service; no new dependency.
- **Security:** events/logs carry only category + short reason — never the raw input or secret value.
- **Deferred:** replacing `tool_requires_approval` with Cedar `is_tool_allowed` at the tool loop; PII-blocking mode; ML detection; output moderation beyond HP4 sycophancy.
- **Blocking scope:** Injection blocks (when enabled); PII is flag-only (blocking a user's own PII is user-hostile).
