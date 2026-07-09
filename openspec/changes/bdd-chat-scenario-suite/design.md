## Context

The BDD harness already exists and works:
- Runner: `tests/bdd.rs` — a `harness = false` `[[test]]` binary named `bdd` that calls `World::run("tests/features")` under `#[tokio::main]` (cucumber-rs 0.23).
- Features: `tests/features/*.feature`. Today only `librefang_and_agui.feature`.
- Backing: real UAR server boot via `tests/integration/live/harness.rs::boot_test_server` + a `stub_llm.rs` fixture server. Scenarios exercise the production code path, not a mock.
- `World` holds `pending_fixtures`, `stub`, `base_url`, `response_status`, `response_body`. Fixtures are accumulated by `Given` steps and the server is booted lazily on the first `When` (`ensure_server_booted`), because the stub's fixtures are fixed at construction.

Existing reusable step vocabulary:
- `Given a running UAR server with a stub LLM`
- `Given the stub LLM responds to "<msg>" with the content "<content>"`
- `Given the stub LLM responds to "<msg>" with a call to tool "<tool>" then the content "<content>"`
- `When I send a bare OpenAI-shaped chat completion request with message "<msg>" to "<path>"`
- `When I send a streaming chat completion request with stream_mode "<mode>" and message "<msg>"`
- `Then the response status should be successful`
- `Then the response body should be an OpenAI chat.completion with content "<content>"`
- `Then the response should contain the event "<name>"` / `should not contain ...` (+ legacy variants)

## Goals / Non-Goals

**Goals:**
- Add `tests/features/chat.feature` covering: single-turn, multi-turn, tool-call round trip, streaming, malformed-request handling.
- Maximize reuse of the existing `World`, fixtures, and steps. Add step definitions only for genuinely new vocabulary.
- Keep the suite green under the existing `cargo test --test bdd` path (already run by `cargo test --all-features` in CI).

**Non-Goals:**
- No new test runner, dependency, feature flag, or CI job.
- No browser/`@ui` scenarios — this is an `@api`-style suite on the HTTP surface.
- No change to production runtime, API, or UI code.
- No refactor of the existing `librefang_and_agui.feature` or its steps.

## Decisions

- **Reuse existing steps wherever possible.** Single-turn and tool-call scenarios are already expressible with the current vocabulary (`content`/`tool-call` fixtures + `bare OpenAI-shaped` request + `chat.completion with content` assertion). These scenarios add coverage without new step code.
- **Add three new step definitions** for uncovered vocabulary:
  1. A `When` that sends a chat request with an explicit multi-message conversation (prior user+assistant turns plus a new user message), so the multi-turn scenario can assert the latest turn is answered.
  2. A `When` that sends a structurally malformed request (JSON body omitting `messages`).
  3. A `Then` asserting the response status is a 4xx client error (and not 5xx).
- **Streaming assertion reuses `contains the event`.** The streaming default-mode scenario asserts on the stub content plus a terminal completion signal already present in the stream body; the existing `contains the event` / content-substring steps cover this, so a substring/event assertion step is reused rather than adding a stream parser.
- **Multi-turn fixture keying.** The stub fingerprints on the last user message, so the multi-turn `Given` reuses the existing content-fixture step keyed to the latest user turn; the new multi-message `When` simply includes the prior turns in the `messages` array it posts.

## Risks / Trade-offs

- **Risk:** the stub's `RequestFingerprint` (model + last_user_message + has_tools/has_tool_result) may not match a multi-message request if extra context changes the fingerprint. **Mitigation:** the fingerprint keys on the *last* user message only, so prior turns don't affect matching; verify by running the suite. If a mismatch appears, adjust the fixture message to the exact last user turn.
- **Risk:** malformed-request handling might currently return 5xx rather than 4xx, exposing a real defect. **Mitigation:** that is a legitimate finding — if the scenario fails, report it rather than weakening the assertion; the requirement is that malformed input is a client error.
- **Trade-off:** substring/event assertions on the streamed body are coarser than a full SSE parse. **Accepted:** matches the existing suite's established assertion style and keeps the change small; a structured stream parser is out of scope.
