## Why

UAR ships a working cucumber-rs BDD harness (`tests/bdd.rs`, features in `tests/features/`, backed by a real server boot + stub LLM), but the only feature file is `librefang_and_agui.feature` — it covers the LibreFang seam and AG-UI event vocabulary. The core end-user chat path (`/v1/chat/completions` and the streaming `/api/chat/completion` surface) has no behavior-level, human-readable scenario coverage. Regressions in multi-turn chat, the tool-call round trip, or error handling would only be caught (if at all) by lower-level integration tests, not by an outside-in suite that documents expected behavior. This change adds a focused chat-behavior feature suite on the existing harness.

## What Changes

- Add a new Gherkin feature file `tests/features/chat.feature` describing core chat behaviors as outside-in scenarios against a running UAR server with a stub LLM:
  - Single-turn non-streaming completion returns an OpenAI `chat.completion` with the expected content.
  - Multi-turn conversation: prior assistant turn is carried in `messages` and the next user turn is answered.
  - Tool-call round trip: the stub requests a tool, the server runs it, and the final assistant content reflects the tool result.
  - Streaming default mode emits token deltas and a terminal completion event.
  - Graceful handling of a malformed request (missing `messages`) returns a client error status, not a panic/5xx.
- Add step definitions to `tests/bdd.rs` only for vocabulary not already present (e.g. multi-message conversation setup, malformed-request send, client-error assertion). Reuse the existing `World`, stub-LLM fixtures, and server-boot harness — no parallel test infrastructure.
- Keep the suite runnable through the existing `cargo test --test bdd` / `cargo test --all-features` path already exercised in CI (`.github/workflows/ci.yml`). No new runner, dependency, or CI job.

## Capabilities

### New Capabilities
- `chat-scenario-coverage`: Behavior-level BDD coverage of the core chat completion surface (single-turn, multi-turn, tool-call round trip, streaming, and malformed-request handling) running on the existing cucumber-rs harness against a real server boot with a stub LLM.

### Modified Capabilities
<!-- None. This adds new test coverage; it does not change any existing product requirement. The existing librefang/AG-UI feature and its step vocabulary are reused, not altered. -->

## Impact

- **Affected code:**
  - Added: `tests/features/chat.feature`
  - Modified: `tests/bdd.rs` (new step definitions for uncovered vocabulary; existing steps and `World` reused)
- **Runtime UX:** None. This is test-only; no production runtime, API, or UI code changes.
- **Provider compatibility:** None directly changed, but the suite *guards* the OpenAI-compatible `/v1/chat/completions` contract and the streaming `/api/chat/completion` surface against regression.
- **Realtime state:** None changed; streaming scenarios assert on emitted event/token output only.
- **Dependencies:** None added. Uses the already-present `cucumber` 0.23 dev-dependency and the existing `stub_llm` / `boot_test_server` harness.
- **KBD workflow state:** Tracked as change 7/9 (Round 3) of phase `uar-production-ready-uiux-2026-07`; `progress.json` and the waypoint are advanced by `/kbd-apply` per task. No other KBD state changes required.
