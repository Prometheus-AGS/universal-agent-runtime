# wire-sycophancy-detection

## Why

`sycophancy-core` is a declared-but-dead dependency (`Cargo.toml:111`): a full `SycophancyConfig` exists (`config.rs:1693` — `enabled`, `strictness`, `auto_correct_threshold`, `reflect_threshold`, `log_only`) with a REST surface to edit thresholds (`api/settings.rs:241`), but **nothing in `src/` ever imports or invokes the detector** — the config is inert. The detector is purely rule-based and local (`Detector::detect(content, prior_completions, &Strictness) -> DetectionResult` — sync, no LLM call, no network), so running it on completed responses is cheap. This is HP4 of `uar-harness-parity`: make the response-quality guardrail actually run.

## What Changes

- After an assistant response completes on the chat path (`server.rs` `api_chat_completion`, where the full `assistant_text_for_capture` + `run_id` + config are in scope), when `sycophancy.enabled`:
  - Construct `sycophancy_core::skill::detector::Detector::new(SkillConfig::default())` and call `detect(&assistant_text, &[], &strictness)`, mapping `sycophancy.strictness` (`"permissive"|"standard"|"strict"`) to `Strictness`.
  - Record the score as a metric (`uar_sycophancy_score` histogram) and, when flagged, increment `uar_sycophancy_flagged_total`.
  - When `sycophancy_score >= auto_correct_threshold` OR `has_critical`: emit a new terminal-adjacent `SycophancyFlagged` normalized event (run_id, score, has_critical, correction_mandatory, and a compact list of pattern classifications) via the existing `emit_to_run` path (so it lands in the replay buffer / Runtime Console), and log at `warn`.
- Add the `SycophancyFlagged` event variant + SSE mapping (`agui.quality.sycophancy`).

**Explicitly NOT in scope (deferred):** automatic *correction*/regeneration of the response (a much larger feature requiring a second LLM pass). HP4 is detection + surfacing only. `log_only` and `auto_correct_threshold` are honored as the *flag* threshold; actual auto-correction is a follow-up. `reflect_threshold` (a lower bar for reflection-phase outputs) is reserved for that follow-up and not applied on the chat path here.

## Capabilities

### New Capabilities
- **`sycophancy-detection`** — `specs/sycophancy-detection/spec.md`. Detection runs on completed responses when enabled; strictness mapping; threshold-based flagging; the `SycophancyFlagged` event + metric; no LLM call and no added user-visible latency on the stream; graceful no-op when disabled or text is empty.

## Impact

- **Affected code:** `src/server.rs` (invoke detection at the post-stream seam, emit event, log), `src/uar/domain/events.rs` (new `SycophancyFlagged` variant + a small `SycophancyClassification` struct), `src/uar/api/sse.rs` (map to `agui.quality.sycophancy`), `src/uar/telemetry/metrics.rs` (score histogram + flagged counter). No new dependency (`sycophancy-core` already present); detection is sync/local.
- **APIs:** no HTTP API changes; one new SSE event type. The existing `GET/PUT /api/settings/sycophancy` thresholds become meaningful.
- **Provider compatibility:** unaffected — detection is provider-agnostic, runs on the final text.
- **Runtime/UX:** flagged responses surface a quality event in the stream/Runtime Console; no blocking, no regeneration, no added stream latency (detection runs after the terminal event).
- **Realtime state:** the `SycophancyFlagged` event flows through the existing emit/replay path.
- **Config liveness:** detection reads the config snapshot available at the chat seam (matches the existing `llm_config` snapshot pattern). Live re-read of edited thresholds via the settings manager is a possible follow-up; not required for HP4.
- **Security:** the event carries the score, pattern ids (S-01…S-08), severities, and rationales — NOT the full response text or user PII beyond what the rationale quotes; keep rationales short and avoid echoing secrets.
- **KBD workflow state:** YES — HP4 of `uar-harness-parity`.
