# 12. Transport-free AG-UI adapter reachable from the embedded runtime

Date: 2026-07-22

## Status

Accepted

## Context

The canonical `NormalizedEvent → AG-UI protocol` mapping (`to_agui_spec_event`,
plus `enrich_agui_spec_payload`) lived in `src/uar/api/sse.rs`. That module is
gated `#[cfg(feature = "server")]` and imports `axum::response::sse::{Event, Sse}`
at the top, so the pure mapping function — though it uses only `NormalizedEvent`
and `serde_json` — was **unreachable to any build without the `server` feature**.

The embedded, in-process runtime (`embedded-mobile` feature; used by the KnowMe
mobile and desktop hosts) never enables `server` (no axum, no HTTP). To emit
AG-UI/A2UI events on the local path it therefore **hand-rolled a duplicate**
`NormalizedEvent → uar.agui/1 JSON` mapping in the downstream project
(`gen_ui_agent/src/uar.rs::embedded_stream_payload`). Two parallel encoders drift.

The UAR also already carried a third, older mapping (`api/adapters.rs::to_ag_ui`,
`token.delta`/`tool.call` naming) and a fourth legacy one (`sse::to_agui_event`,
`agui.*` names). Four `NormalizedEvent → wire` encoders, one canonical.

The AG-UI protocol's own reference SDKs (`@ag-ui/encoder`, `ag-ui-protocol`) ship
a standalone `EventEncoder` that converts events to the wire and is **separate
from the HTTP handler / transport**; guidance is that *agent runtimes should
expose AG-UI-compatible event streams to frontends*. The transport coupling in
UAR was incidental, not required by the protocol.

## Decision

- Move the canonical `to_agui_spec_event` and `enrich_agui_spec_payload` into the
  already-**ungated, axum-free** `src/uar/api/adapters.rs`. That module imports
  only `NormalizedEvent` + `serde_json`, so the encoder is reachable from every
  build, including `embedded-mobile`.
- `src/uar/api/sse.rs` **re-exports** them (`pub use super::adapters::{…}`) and
  keeps only the Axum SSE framing (`build_sse_response`). No behavior change on
  the server path; existing callers and tests are unaffected.
- Keep the encoder as a **free function, not a trait.** UAR's convention is free
  functions, and there is no second transport to abstract over. A `Transport` /
  `EventEncoder` trait would be speculative generality (YAGNI); introduce it only
  when a second wire format (WebSocket, protobuf) actually lands — the ungated
  location makes that a localized future change.
- Downstream (KnowMe) deletes its duplicate `embedded_stream_payload` and calls
  the shared UAR encoder, so cloud and local paths produce byte-identical AG-UI
  frames from one source of truth.

## Consequences

- The embedded runtime and the SSE server now share ONE canonical
  `NormalizedEvent → AG-UI` mapping. Local (on-device) agent runs emit the same
  AG-UI/A2UI events as cloud runs, enabling full-agentic operation against local
  models with no protocol re-implementation.
- The older `to_ag_ui` and legacy `to_agui_event` encoders remain for their
  existing consumers but are now clearly non-canonical; a follow-up may retire
  them.
- Adding a non-SSE transport later is a matter of adding a framing wrapper over
  the shared encoder, at which point extracting a trait becomes justified.

## Note

This change was made under an explicit operator override of the repository's
active production-completion execution lock (phase
`uar-final-production-hardening-2026-07`, "advance BossFang changes 20–24 only").
The operator authorized this AG-UI adapter consolidation as a separate,
self-contained change on its own branch; it does not touch the BossFang
`server-full` sidecar work.
