# Handoff: uar-providers, issue #380 item 3 (Anthropic SSE byte-partition acceptance tests)

Date: 2026-10-30. Branch: fix/batch3-identity. Role: uar-providers.

## Verification status
NOTHING WAS COMPILED OR RUN. No cargo, test, clippy, fmt, or `git diff --check` command was executed (verification lock CLOSED). All claims below are source inspection only. The lead must run the gate.

## Changed paths
1. `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/llm/anthropic_sse.rs`
   - Appended `#[cfg(test)] mod byte_partition_tests` (starts at line ~117, to end of file; file grew from 116 to ~667 lines). Existing `tests` module untouched.
2. `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/llm/anthropic_driver.rs`
   - Behavior-preserving extraction (lines ~410-545). The `stream! { ... }` loop body is moved verbatim out of `AnthropicDriver::stream` into
     `pub(super) fn decode_anthropic_sse<S, B>(byte_stream: S, mut state: StreamState, metrics_model: String) -> Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>`
     with `S: Stream<Item = Result<B, reqwest::Error>> + Send + 'static`, `B: AsRef<[u8]> + Send + 'static`.
   - `stream()` now ends with `Ok(decode_anthropic_sse(response.bytes_stream(), state, metrics_model))`.
   - Only textual deltas inside the loop: `framer.push(&chunk)` -> `framer.push(chunk.as_ref())` (Bytes -> generic B). `let mut state` became `let state` at the call site (the `mut` moved to the helper parameter).
   - No public API change (`pub(super)` = visible in `crate::llm` and descendants). No other file touched.

## Interface effect
New crate-internal fn `crate::llm::anthropic_driver::decode_anthropic_sse`. Production behavior, error messages, event order, metrics calls, and terminal semantics are intended identical. Compile risk to check first: generic bounds / `Send` of the `stream!` future, and type inference of `yield Err(... .into())` (return type is spelled as the boxed dyn stream, same coercion site as before).

## Tests (all `#[tokio::test]` in `llm::anthropic_sse::byte_partition_tests`)
Positive, vs. one fixed fixture (LF + CRLF + bare-CR delimiters, `: keepalive` comment, `event:`/`data:` with no space after colon, ignored `id:` field, 2-line `data:` field, `ping`, unknown `future_event`, text delta with JSON `é` escape + literal "é" + 4-byte emoji, tool_use start + 4 `input_json_delta` chunks (one empty) + stop, `message_delta` with usage, `message_stop`):
- `byte_partition_single_chunk_yields_expected_normalized_sequence`
- `byte_partition_every_two_chunk_split_matches_single_chunk` (all offsets 0..=len, including empty head/tail)
- `byte_partition_one_byte_at_a_time_matches_single_chunk`
- `byte_partition_fixed_multi_chunk_partitions_match_single_chunk` (chunk sizes 2/3/5/7/11/13/64/1000, irregular 1/2/3/5/8/13/21, split before and after every CR/LF, same with empty chunks, split before every non-ASCII byte)
- `byte_partition_valid_terminal_then_eof_emits_exactly_one_done`
Negative / terminal conditions (each over single chunk, 1-byte chunks, 7-byte chunks):
- `byte_partition_malformed_complete_json_fails_even_before_a_later_message_stop`
- `byte_partition_invalid_utf8_fails_even_before_a_later_message_stop`
- `byte_partition_truncated_utf8_codepoint_at_eof_fails_without_done`
- `byte_partition_oversized_event_fails_without_done` (1 MiB + payload vs pinned liter 1 MiB limit; single chunk and 64 KiB chunks)
- `byte_partition_missing_message_stop_at_eof_fails_without_done`
- `byte_partition_final_unterminated_event_at_eof_fails_without_done` (tail with and without trailing newline)

Selector: `cargo test --locked --no-default-features --features server-full byte_partition_` (substring filter; all 11 tests share the prefix; or full path `llm::anthropic_sse::byte_partition_tests`).

## What each test exercises
Production path covered (the real code, not a copy): `AnthropicSseFramer::push/finish` -> `liter_llm::SseDecoder::push/next_event/finish` -> `serde_json` parse -> type/event-name check -> unknown-event skip -> `StreamEvent` deserialize -> `StreamState::process_event` -> usage metrics calls -> deferred `Done` after `framer.finish`/`decoder.finish`/`saw_terminal`. Assertions compare full `NormalizedEvent` sequences (request ID fixed to `req-byte-partition`), failure kind (`ProviderErrorKind::Stream`), message fragment, absence of `Done`, and that the failure is the last stream item.

NOT covered:
- `reqwest::Response::bytes_stream()` itself and the HTTP request/status handling in `AnthropicDriver::stream` (tests inject `futures::stream::iter(Vec<Ok(Vec<u8>)>)`). The issue's "mock-HTTP-driver test proving production bytes_stream() uses this decoder" is NOT done; `stream()` -> `decode_anthropic_sse` wiring is verified only by source inspection. The existing axum stub in the driver tests could host it, but real TCP does not guarantee chunk boundaries.
- `Err(reqwest::Error)` transport-failure arm (a `reqwest::Error` cannot be constructed in a unit test).
- Issue item 1 / full-harness truncated-Anthropic test (orchestrator/manager scope, not providers).
- Initial UTF-8 BOM partitions (covered only by the existing framer unit tests).
- "Data after message_stop" error arm.

## Unresolved / for the gate
- Exact `Display` text of failures asserted by fragment: "Malformed Anthropic SSE event", "invalid UTF-8", "incomplete event or UTF-8 codepoint", "exceed", "ended without message_stop" (derived from driver strings and vendored liter source; not executed).
- Failure-case event prefixes differ by partition for invalid-UTF-8 and oversize (a failing `push` drops frames in the same chunk); tests assert only failure + no Done + no corrupted text there, and exact prefixes for JSON/missing-stop/unterminated cases.
- ~12 lines exceed 100 columns (raw JSON string literals); rustfmt does not split them. `cargo fmt --check` may still want to reformat other spots in the new module; unverified.
