## 1. Typed Frontend Normalization

- [x] 1.1 Add the validated-frame normalizer, phase mapping, message chunk, event row, and terminal timing types under `frontend/src/platform/agui/`.
- [x] 1.2 Extend the per-stream adapter with normalized projections, opaque RAW passthrough, and deterministic unit coverage for deduplication and all three consumers.

## 2. Consumer Integration

- [x] 2.1 Route normalized message chunks and event rows through the chat stream store without re-extracting official content payloads.
- [x] 2.2 Add typed `phase_timings` to runtime runs and upsert the terminal timing projection through the entity graph.

## 3. Attach and Replay Parity

- [x] 3.1 Reconstruct cursor-consistent state and assistant-message snapshots from retained run history and prepend them to `agui_spec` attach/resume streams.
- [x] 3.2 Synthesize exactly one official `TOOL_CALL_START` before replayed args/end frames and extend focused Rust plus live-seam coverage.

## 4. Verification and Absorption

- [x] 4.1 Pass frontend typecheck, lint, architecture gates, focused frontend/Rust tests, strict OpenSpec validation, and diff-integrity checks.
- [x] 4.2 Pass the Wave 2 full frontend test/build boundary and record verification evidence.
- [x] 4.3 Archive `complete-agui-event-parity` as superseded without applying its obsolete capability delta.
