## Context

Detection runs in a fire-and-forget post-stream task (`server.rs:~4344`) that captures `sycophancy_cfg`, `mgr` (run_manager), `rid`, `text`; it records metrics and emits `SycophancyFlagged` when `outcome.flagged`. `should_flag` uses `auto_correct_threshold`. The app holds `state.orchestrator: Arc<Orchestrator>` with `chat_non_streaming(Vec<Message>) -> Result<String>` (no tools, collects deltas). `SycophancyConfig` has `auto_correct_threshold`, `reflect_threshold`, `log_only` (the latter two inert today).

## Goals / Non-Goals
**Goals:** opt-in single corrective pass on flagged turns; emit as a follow-up; never delay the first stream; honor `auto_correct`/`log_only`/`auto_correct_threshold`.
**Non-Goals:** `reflect_threshold`/reflect-phase correction; iterative/multi-pass; rewriting thread history; correcting non-chat paths; per-run model selection for the pass (use the app orchestrator).

## Decisions
- **D1 — `auto_correct: bool` (default false)** added to `SycophancyConfig`. Opt-in; preserves detection-only behavior.
- **D2 — reuse `state.orchestrator.chat_non_streaming`** for the corrective pass (no tools, one shot). Capture an `Arc<Orchestrator>` clone into the post-stream task. Uses the app's configured model — acceptable for correction; avoids per-run driver construction.
- **D3 — pure `correction_messages(flagged_text) -> Vec<Message>`** in `quality.rs` (system instruction: rewrite to remove sycophancy, preserve correct/substantive content, output only the rewrite; user: the flagged text). Unit-test the shape (role/system-instruction present, contains the text), since the LLM call itself isn't unit-testable.
- **D4 — follow-up event `SycophancyCorrected { run_id, corrected_text }`** → `agui.quality.sycophancy_corrected`, emitted via `emit_to_run` after a successful non-empty correction. Distinct from `SycophancyFlagged`; the client renders it as a correction.
- **D5 — trigger:** correction runs when `outcome.flagged && auto_correct && !log_only`. `auto_correct_threshold` already governs `flagged`. Contained: errors/empty results are logged and dropped (no follow-up).

## Risks / Trade-offs
- **[Cost/latency]** an extra LLM call per flagged turn → Mitigation: opt-in, flagged-only, post-stream (no first-response latency); single pass.
- **[Correction quality]** the rewrite could still be imperfect → Mitigation: best-effort signal; emitted as a distinct follow-up the user/UI can choose to surface; not a guarantee.
- **[Behavior change]** none by default (auto_correct=false).
- **[Loop risk]** correcting a correction → Mitigation: single pass only; the corrective output is emitted as an event, not re-fed through detection.

## Migration Plan
1. Add `auto_correct` (default false) to config.
2. Add `correction_messages` helper + unit test in quality.rs.
3. Add `SycophancyCorrected` event + SSE mapping.
4. In the post-stream task: capture orchestrator; on flagged+auto_correct+!log_only, run `chat_non_streaming(correction_messages(text))`, emit `SycophancyCorrected` on success.
5. cargo check/clippy/tests; manual: enable auto_correct, force a flagged response → corrected follow-up.
- Rollback: additive; revert restores detection-only.

## Open Questions
- Should the corrected text also be persisted to the session as an assistant message? (No for v1 — emit as event only; thread-history rewrite is out of scope.)
