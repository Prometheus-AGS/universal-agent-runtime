# Tasks — sycophancy-auto-correct

## 0. Bootstrap
- [ ] 0.1 Confirm seams: post-stream sycophancy task (`server.rs:~4344`), `state.orchestrator.chat_non_streaming`, `SycophancyConfig`, `quality::detect`
- [ ] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Config
- [ ] 1.1 Add `auto_correct: bool` (`#[serde(default)]`, default false) to `SycophancyConfig` + init in `Default`

## 2. Correction prompt helper (quality.rs, pure + tested)
- [ ] 2.1 `pub fn correction_messages(flagged_text: &str) -> Vec<crate::llm::Message>` — system instruction (rewrite to remove sycophancy, preserve correct/substantive content, output only the rewrite) + user message with the flagged text
- [ ] 2.2 Unit test: returns a system + user message; user message contains the flagged text

## 3. Event + mapping
- [ ] 3.1 `NormalizedEvent::SycophancyCorrected { run_id, corrected_text }` (events.rs)
- [ ] 3.2 Map → `agui.quality.sycophancy_corrected` in `to_agui_event`; `to_runtime_entity_event` returns None (exhaustive)

## 4. Corrective pass (server.rs post-stream task)
- [ ] 4.1 Capture `Arc::clone(&state.orchestrator)` into the sycophancy task
- [ ] 4.2 After the `SycophancyFlagged` emit, when `sycophancy_cfg.auto_correct && !sycophancy_cfg.log_only`: call `orchestrator.chat_non_streaming(quality::correction_messages(&text)).await`
- [ ] 4.3 On Ok(non-empty): emit `SycophancyCorrected` via `mgr.emit_to_run`. On Err/empty: log + drop (contained)

## 5. Validation (gate)
- [ ] 5.1 `cargo check --features postgres-backend` clean; zero new warnings
- [ ] 5.2 `cargo clippy` — no new warnings in touched files
- [ ] 5.3 `cargo test --features postgres-backend --lib` — existing pass + correction_messages test
- [ ] 5.4 Manual: enable auto_correct, force a flagged response → `agui.quality.sycophancy_corrected` follow-up; default → detection-only (pending live env)
- [ ] 5.5 `openspec validate sycophancy-auto-correct --strict`; update `.kbd-orchestrator` progress

## Notes
- Opt-in (default false); post-stream, flagged-only, single pass — no first-response latency. log_only suppresses; auto_correct_threshold gates (via flagged). reflect_threshold/reflect-phase deferred. Uses the app orchestrator model. Errors contained.
