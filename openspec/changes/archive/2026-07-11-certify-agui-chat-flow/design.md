## Context

UAR currently exposes two event dialects: legacy lower-case `agui.*` SSE names and an `agui_spec` mode that uses official upper-case event types. The latter is incomplete (notably deprecated `THINKING_*`, missing start/end message events, and legacy payload shapes), while Chat and Runtime Console each interpret portions of the stream directly.

The upstream AG-UI protocol remains pre-1.0 and evolves in place. UAR therefore pins a dated upstream vocabulary snapshot and gives its own compatibility surface an independent version.

## Decisions

### `uar.agui/1` is the stable UAR profile

`stream_mode: "agui_spec"` implements `uar.agui/1`, pinned to the official AG-UI core event vocabulary documented on 2026-07-11. The profile uses the official event `type` values and field names. The legacy `stream_mode: "agui"` remains temporarily available but is explicitly non-conformant and deprecated.

### Standard events carry standard semantics

Run, step, text message, reasoning, tool call/result, state snapshot/delta, message snapshot, raw, custom, and error events use the official lifecycle rules. `STATE_DELTA` is RFC 6902 JSON Patch. Thinking events are not emitted; provider thinking is exposed through the official reasoning lifecycle.

### UAR extensions use `CUSTOM`

Every UAR-specific signal is encoded as `{ type: "CUSTOM", name: "uar.<domain>.<event>", value: {...} }`. Extension names and payloads are versioned by `profile: "uar.agui/1"`; unknown extensions must be retained for inspection but may be ignored by renderers.

### One adapter owns client ingestion

A typed frontend adapter validates, orders, deduplicates, and reduces official events into Chat state and Runtime entities. Stores consume adapter outputs rather than independently switching on wire events.

## Risks / Trade-offs

- Upstream is pre-1.0 → the dated vocabulary pin prevents silent semantic drift.
- Legacy consumers may rely on `agui.*` → legacy mode remains during 1.0 but is documented as deprecated.
- JSON Patch can diverge → failed deltas mark the client unsynchronized until a fresh snapshot arrives.
- Replay can duplicate frames → stable event IDs/cursors and idempotent reduction are mandatory.

## Migration Plan

1. Publish the profile and extension registry.
2. Complete Rust mappings and shared golden fixtures.
3. Introduce the shared frontend adapter and migrate Chat/Console consumers.
4. Certify live, cancellation, replay, resume, interruption, and error behavior.
