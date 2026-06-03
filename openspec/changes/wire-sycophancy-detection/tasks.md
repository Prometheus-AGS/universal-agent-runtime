# Tasks — wire-sycophancy-detection

## 0. Bootstrap

- [x] 0.1 Confirm `sycophancy-core` API: `Detector::new(SkillConfig)`, `detect(&str, &[String], &Strictness) -> DetectionResult`, `Strictness::{Permissive,Standard,Strict}` (ref: prometheus-cli `evaluate.rs`)
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Metrics (metrics.rs)

- [x] 1.1 `record_sycophancy_score(score: f64)` → `uar_sycophancy_score` histogram
- [x] 1.2 `record_sycophancy_flagged()` → `uar_sycophancy_flagged_total` counter

## 2. Event model (events.rs + sse.rs)

- [x] 2.1 Add `NormalizedEvent::SycophancyFlagged { run_id, sycophancy_score: f32, has_critical: bool, correction_mandatory: bool, classifications: Vec<SycophancyClassification> }`
- [x] 2.2 Add `pub struct SycophancyClassification { pattern_id: String, severity: String, rationale: String }`
- [x] 2.3 Map `SycophancyFlagged` → `agui.quality.sycophancy` in `to_agui_event` (sse.rs)

## 3. Detection helper (pure, testable)

- [x] 3.1 `fn strictness_from(s: &str) -> Strictness` — total, defaults to Standard
- [x] 3.2 `fn should_flag(result: &DetectionResult, auto_correct_threshold: f32) -> bool` — score ≥ threshold OR has_critical
- [x] 3.3 Unit tests: strictness mapping (permissive/standard/strict/unknown), flag decision (above/below/critical)

## 4. Wire at the chat post-stream seam (server.rs)

- [x] 4.1 After the terminal `RunDone`/`RunDoneWithUsage` arm, when `sycophancy.enabled` and assistant text non-empty: build `Detector::new(SkillConfig::default())`, run `detect(text, &[], &strictness)`
- [x] 4.2 Record score metric; if `should_flag`, record flagged counter, emit `SycophancyFlagged` via `state.run_manager.emit_to_run(...)`, and `tracing::warn!`
- [x] 4.3 Contain errors (best-effort; never affect the already-streamed response). Exclude full response text from the event.
- [x] 4.4 Map classifications → `SycophancyClassification` (pattern_id, severity string, short rationale)

## 5. Validation (gate)

- [x] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 5.2 `cargo clippy` — no new warnings in touched files
- [x] 5.3 `cargo test --features postgres-backend --lib` — existing pass + new strictness/flag tests
- [ ] 5.4 Manual: a sycophantic response triggers `agui.quality.sycophancy` + metric (pending live env — document if not runnable here)
- [x] 5.5 `openspec validate wire-sycophancy-detection --strict`; update `.kbd-orchestrator` progress

## Notes

- **Detection only** — automatic correction/regeneration is DEFERRED (proposal §scope). `auto_correct_threshold` is used as the flag threshold; `log_only`/`reflect_threshold` corrective behavior is reserved for the follow-up.
- Coverage: streaming chat path only (the user-facing one); other run paths are a follow-up.
- Config: read from the chat-seam snapshot (matches `llm_config`); live settings re-read deferred.
- Security: event carries score/pattern-id/severity/short rationale — never the full response text or secrets.
- Detection is sync + rule-based (no LLM/network); runs post-terminal so no added stream latency.
