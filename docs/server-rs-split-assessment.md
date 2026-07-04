# `src/server.rs` split assessment (CH-20)

**Date:** 2026-07-04. **Current size:** 5,068 lines (up from 4,922 measured
at the start of this phase's assessment — the file keeps growing every
phase that touches the API surface, which is itself evidence the split is
overdue rather than optional).

This is an **assessment and recommendation**, not an executed split. A
mechanical move of ~5,000 lines of axum handler code across module
boundaries is exactly the kind of change that should be its own dedicated,
narrowly-scoped effort with full regression testing between each step —
not something to rush through as one item inside a larger phase. Per Rule
31 ("Prefer Small, Reviewable Changes") and Rule 8 ("Minimize Irreversible
Actions"), the responsible scope for this item, given the size of the rest
of this phase, is to produce the evidence and the plan, not to execute a
high-blast-radius refactor without room for careful verification.

## Current structure (by line range)

| Range | Contents | Approx. size |
|---|---|---|
| 1–74 | Module doc + imports | 74 |
| 75–1301 | Server bootstrap: `start_server`, `start_server_sidecar`, `serve_on_listener`, `bind_companion_listener`, CORS layer construction, legacy base-URL normalization, tool-call-approval handler. This is where the axum `Router` is assembled and every `.route(...)` call lives (43 total). | ~1,227 |
| 1304–2110 | Admin/API handlers: resilience-policy resolution (`load_global_resilience_policy`, `resolve_effective_resilience_policy`), title generation, `/api/models`, `/api/catalog`, `/api/route-model`, `/api/v1/models/:id`, `/api/context-stats`, `/api/skills/reload`, legacy route stubs, static-file/SPA serving, 404 handler. | ~806 |
| 2113–3450 | **Anthropic Messages API compatibility shim**: request/response structs (`AnthropicMessagesRequest`, `AnthropicMessageInput`, `AnthropicContentBlockInput`, etc.), format-conversion functions (`convert_anthropic_messages_to_openai`, `convert_anthropic_tools_to_openai`, `inject_anthropic_cache_control`, `anthropic_image_to_openai_part`, ...), the `api_messages` handler itself (~600 lines), and Anthropic-flavored SSE event emission. | ~1,337 |
| 3451–5068 | **OpenAI Chat Completions compatibility shim**: request/response structs (`OpenAiChatCompletionResponse`, `OpenAiChunk`, `OpenAiDelta*`, ...), session-id extraction/validation, multipart content building, model resolution, and the main chat-completions handler. | ~1,617 |

The two compatibility shims (Anthropic + OpenAI) together are **~2,954
lines — 58% of the file** — and are the least entangled with the rest:
each is a self-contained request→internal-format→response translation
layer with its own struct set, referenced from exactly one route
registration each in the bootstrap section.

## Recommended target layout

```
src/server/
  mod.rs              -- bootstrap only: start_server, start_server_sidecar,
                          serve_on_listener, bind_companion_listener, CORS,
                          Router assembly (imports handlers from siblings)
  admin_api.rs         -- models/catalog/route-model/context-stats/
                          skills-reload/title-generation/legacy stubs/
                          static+SPA serving/404, resilience-policy resolution
  anthropic_compat.rs  -- the full Anthropic Messages API shim
  openai_compat.rs      -- the full OpenAI Chat Completions shim
```

`src/server.rs` becomes `src/server/mod.rs` (a directory module) — the
smallest structural change axum/Rust requires; no route paths, request/
response shapes, or behavior change for any client.

## Recommended sequencing (4 independent, reviewable PRs)

1. **Extract `openai_compat.rs`** (lines 3451–5068, largest and most
   self-contained — touches the fewest shared helpers outside its own
   range). Move the OpenAI structs + handler + its private helpers
   (`extract_cookie_session_id`, `validate_uuid_session_id`,
   `resolve_session_id`, `build_multipart_content`, `resolve_requested_model`,
   `model_known`) verbatim; `pub(crate)` the handler fn, `use` it back into
   `mod.rs`'s router assembly. Verify: `cargo check`, `cargo test --lib`,
   plus a live smoke test of `/v1/chat/completions` (streaming and
   non-streaming) against a real or stub model — this is exactly the kind
   of wiring regression compile success alone won't catch (a route
   registered against the wrong re-exported handler still compiles).
2. **Extract `anthropic_compat.rs`** (lines 2113–3450) the same way.
   Verify: same checklist, smoke-testing `/v1/messages` (Anthropic-shaped
   request, both streaming and non-streaming, tool-call round-trip since
   this shim has the most complex conversion logic —
   `convert_anthropic_tools_to_openai`/`ensure_toolu_id`/`ToolTrack`).
3. **Extract `admin_api.rs`** (lines 1304–2110). Lower risk (simpler
   handlers, less state), but still needs a smoke pass over `/api/models`,
   `/api/catalog`, `/api/route-model` since these are exercised by the
   admin frontend built across `uar-next-harness` (CH-07/CH-10 dashboards
   depend on `/api/models`' `benchmarks` field specifically).
4. **Rename `server.rs` → `server/mod.rs`**, leaving only bootstrap +
   Router assembly (~1,227 lines — a 76% reduction from the current
   5,068). This is the safest step since by this point every handler
   already lives elsewhere; it's a pure `mod.rs` reorganization.

Each step should land as its own commit/PR with its own compile + test +
live-smoke checkpoint — never batch multiple extractions into one
unverified change, since a wrong re-export is a silent behavior change
(the server still starts; only a specific route breaks).

## What this assessment does NOT recommend

- Splitting `admin_api.rs` further (models/catalog/routing are small
  enough individually that a fourth-level split would be over-engineering
  for the current size).
- Extracting shared response-building helpers (`anthropic_error_response`,
  `openai_error_response`) into their own module — each is small, used
  only within its respective compat shim, and moving them separately adds
  a PR for no real benefit.
- Doing this work as part of a large multi-change phase (like this one) —
  recommend a dedicated, single-purpose follow-up phase/change so the
  4-step sequence above gets the checkpoint discipline it needs.
