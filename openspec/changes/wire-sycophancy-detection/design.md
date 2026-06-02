## Context

`sycophancy-core` (`Cargo.toml:111`) is present but never invoked. The detector is `sycophancy_core::skill::detector::Detector::new(SkillConfig)` with `detect(content: &str, prior_completions: &[String], strictness: &Strictness) -> DetectionResult { sycophancy_score: f32, classifications: Vec<HeuristicMatch>, has_critical: bool, correction_mandatory: bool }` — sync, rule-based, no LLM/network. `HeuristicMatch { pattern_id (S-01..S-08), severity (Low/Medium/High/Critical), location, rationale }`. UAR `SycophancyConfig { enabled, strictness: String, auto_correct_threshold: f32 (0.5), reflect_threshold: f32 (0.3), log_only }` (`config.rs:1693`) is editable via `GET/PUT /api/settings/sycophancy` but inert. The reference wiring is `prometheus-cli`'s `evaluate.rs` (`SkillConfig::default()` + `detect(text, &[], &Strictness::Standard)`).

The chat path's post-stream seam (`server.rs` `api_chat_completion`, ~4235) has the full `assistant_text_for_capture`, the `run_id`, the app `state` (config + `run_manager`), and already emits post-terminal events via `state.run_manager.emit_to_run(...)` (memory auto-capture uses this exact pattern).

## Goals / Non-Goals

**Goals:** run the detector on completed chat responses when enabled; map strictness; flag on threshold/critical; surface a `SycophancyFlagged` event + metrics; zero added stream latency; never break the response.

**Non-Goals:** automatic correction/regeneration of responses (a second-LLM-pass feature — deferred); applying `reflect_threshold` on the chat path (reserved for the reflection-phase follow-up); live re-read of settings-API threshold edits into in-flight runs (snapshot is acceptable, matches `llm_config`); detection on non-chat paths (graph runs, ingestion).

## Decisions

### D1 — Detect at the server.rs post-stream seam, not in the manager
Run detection where the full assistant text + run_id + `state` (config, `emit_to_run`) are all in scope, after the terminal `RunDone`/`RunDoneWithUsage` is observed. This adds no stream latency (the user already received the response) and reuses the memory-auto-capture emit pattern.
- **Why not the manager spawned task:** it has only a config snapshot and no easy `emit_to_run`/AppConfig access; the server seam is cleaner. *Trade-off:* detection only covers the streaming chat path (the user-facing one) — acceptable for HP4; other paths are a follow-up.

### D2 — Detection + surfacing only; correction deferred
Honor `auto_correct_threshold` as the *flag* threshold and `has_critical` as an override. Do NOT regenerate/correct the response. `log_only` and `reflect_threshold` are accepted config but their corrective behavior is deferred; document this so the config isn't misread as fully active.
- **Why:** auto-correction needs a second LLM pass + response-mutation plumbing — out of proportion for "wire the detector". Detection + visibility delivers the safety signal now.

### D3 — New `SycophancyFlagged` event, no response text
Add `NormalizedEvent::SycophancyFlagged { run_id, sycophancy_score, has_critical, correction_mandatory, classifications: Vec<SycophancyClassification> }` where `SycophancyClassification { pattern_id, severity, rationale }`. Map to `agui.quality.sycophancy`. Emit only when flagged (keeps the stream quiet for clean responses).
- **Why not reuse Error:** sycophancy is a quality signal, not a failure. *Security:* exclude the full response; rationales are short detector-authored strings.

### D4 — Strictness mapping is total and forgiving
`match strictness.to_lowercase().as_str() { "permissive" => Permissive, "strict" => Strict, _ => Standard }`. Unknown values default to Standard (no error), matching the config default.

### D5 — Best-effort, contained
Detection runs after the response is delivered; wrap it so a detector panic/error is logged and never affects the request. Score metric recorded whenever detection runs; flagged counter only when flagged.

## Risks / Trade-offs

- **[Coverage gap]** only the streaming chat path is instrumented → Mitigation: documented; the seam is the user-facing path; other paths are a follow-up.
- **[Config staleness]** settings-API threshold edits won't affect in-flight runs (snapshot) → Mitigation: documented; matches `llm_config`; live re-read is a follow-up.
- **[Misleading config]** `auto_correct_threshold`/`log_only`/`reflect_threshold` imply correction that isn't built yet → Mitigation: proposal + tasks explicitly state correction is deferred; the threshold is used for flagging.
- **[PII/secret leakage in rationale]** detector rationales quote snippets → Mitigation: keep classifications to pattern_id/severity/short rationale; never attach full text; review.
- **[Latency]** detection is sync on the response thread → Mitigation: it runs post-terminal (after streaming), rule-based and fast; if ever heavy, move to `tokio::task::spawn_blocking`.

## Migration Plan

1. Add metrics recorders (score histogram, flagged counter).
2. Add the `SycophancyFlagged` event variant + `SycophancyClassification` + SSE mapping.
3. Wire detection at the server.rs post-stream seam (strictness map, threshold flag, emit, log) behind `enabled`.
4. Validate: unit-test the strictness mapping + flag decision (pure helper); compile + clippy + existing tests.
- **Rollback:** additive (new event variant, new metrics, gated detection). Revert restores the inert state.

## Open Questions

- **Auto-correction follow-up:** should it live in this phase or `uar-safety-and-evals`? (Leaning the latter — it pairs with guardrails/evals.)
- **Settings liveness:** is live threshold editing wanted soon enough to wrap `SycophancyConfig` in `Arc<RwLock<...>>` now, or defer? (Defer for HP4.)
