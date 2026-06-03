# sycophancy-auto-correct

## Why

HP4 wired sycophancy *detection*: a flagged response emits `SycophancyFlagged` (post-stream) but nothing corrects it — `auto_correct_threshold` is only a flag threshold, and `log_only`/`reflect_threshold` are inert. This is goal **S2** of `uar-safety-and-evals`: act on a flagged response with an opt-in corrective pass.

## What Changes

- Add `SycophancyConfig.auto_correct: bool` (**default false** — opt-in; decision D3).
- In the existing post-stream sycophancy task (`server.rs:~4344`), when a response is flagged AND `auto_correct` is on AND NOT `log_only`: run **one** corrective LLM pass via the app `Orchestrator::chat_non_streaming` (no tools) with a correction prompt (rewrite to remove sycophancy, preserve substantive/correct content), and emit the result as a **follow-up** `SycophancyCorrected` event. The original stream is never blocked or delayed (correction runs after the terminal event, fire-and-forget, like detection).
- Add `NormalizedEvent::SycophancyCorrected { run_id, corrected_text }` → `agui.quality.sycophancy_corrected`.
- `auto_correct_threshold` now meaningfully gates correction (it is the flag threshold); `log_only=true` suppresses correction (detect/flag only).

Out of scope (deferred): `reflect_threshold` / reflect-phase correction (not on the chat path); multi-pass / iterative correction; correcting non-chat run paths; replacing the original response in the thread (we append a correction, not rewrite history).

## Capabilities

### Modified Capabilities
- **`sycophancy-detection`** — delta `specs/sycophancy-detection/spec.md`. Adds an opt-in auto-correction requirement: a flagged response may trigger one corrective LLM pass emitted as a follow-up; detection-only remains the default.

## Impact

- **Affected code:** `src/config.rs` (`auto_correct` field + default), `src/uar/quality.rs` (pure `correction_messages(text)` helper, unit-tested), `src/server.rs` (corrective pass in the post-stream task; capture `state.orchestrator`), `src/uar/domain/events.rs` (`SycophancyCorrected`), `src/uar/api/sse.rs` (mapping). No new dependency.
- **Cost/latency:** one extra LLM call **only on flagged turns when enabled**, and only after the user already received the original stream (no added latency to the first response).
- **Behavior preservation:** default false ⇒ detection-only, exactly as today.
- **Security:** the corrective pass sends the flagged assistant text (not secrets); the event carries the corrected text.
- **KBD workflow state:** YES — S2 of `uar-safety-and-evals` (final change of the phase).
