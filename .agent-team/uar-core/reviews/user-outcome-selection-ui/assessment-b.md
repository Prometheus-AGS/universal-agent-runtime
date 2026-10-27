# Assessment B — source integrity evidence

2026-10-25. Active phase: `uar-post-lifecycle-verification-readiness`, stage `user-outcome-implementation`; inspected waypoint revision 432 (144/168 implementation items). Verification CLOSED. Classification: source inspection and authorized review-document writing only. Target: `.agent-team/uar-core/handoffs/user-outcome-selection-ui-design.md`, grounded in the existing frontend event-to-chat path. Isolated assessment: no Assessment A or other assessment output read.

This is a manual source/design assessment, not a completed detector/browser Impeccable critique. The operator lock overrides skill instructions requiring detector and browser execution. No scores, detector counts, runtime PASS, GUI acceptance or overlay evidence are claimed.

## Integrity verdict

The proposed design correctly represents a routing decision using the incumbent chunk surface, but the source path currently cannot display the event. The most uncomfortable scenario is a correct backend selection and a successful answer with no visible notice: adding the catalog type and renderer alone still permits exactly that outcome.

## Findings and corrections

### P1 — Explicit chat ingestion is required, beyond normalizer/renderer wiring

Evidence: `frontend/src/platform/agui/runtime-chunk-projection.ts:110` defaults unknown custom names to hidden `raw`. `agui-adapter.ts:111` maps an unrecognized official custom name to `agui.custom`. In `frontend/src/stores/chat-stream-store.ts:1111`, that case only extracts A2UI envelopes; a selection payload has no corresponding chat insertion. The existing `addRuntimeChunk` calls at lines 954–974 handle approval and denial only. `chunk-renderer.tsx:33` hides raw/step chunks. This is an observed source gap against the explicit visible-selection requirement.

Correction: specify the exact adapted event branch and domain insertion for selection, so the event reaches the streaming message before output. Preserve canonical event identity on the notice. Do not make generic unknown custom events visible, use text deltas to impersonate a notice, or reclassify selection as skill activation. Add source fixtures covering event → adapter → chat message → data-part renderer, not solely a component fixture.

### P1 — Legacy and canonical paths require different normalization entry points

Evidence: `src/uar/api/sse.rs:521` emits `agui.agent.selected` with `name: uar.agent.selected`, nested `value`, and `request_id`. The official adapter emits CUSTOM. `UarAguiAdapter.ingest` treats `agui.*` as legacy first; it only attaches `runtimeChunk/eventRow` when `isUarAguiEvent(payload)` succeeds. The existing native legacy selection payload is not an official profile frame. Adding `customChunk` support therefore does not automatically cover legacy/native delivery.

Correction: document and implement a shared selection decoder called by both supported envelope paths. Carry authoritative `runId` or legacy `request_id` rather than substituting the current UI run when a wire run is available. Keep native/legacy display explicitly in the acceptance inventory; do not infer it from the canonical fixture.

### P1 — Replay identity needs durable-message coverage, not a general no-duplicate assertion

Evidence: `agui-adapter.ts:212–216` deduplicates by eventId and rejects regressing sequence within its adapter instance. `chat-stream-store.ts:722` constructs one adapter before the stream attempts. However, `chat-message-store.ts:338–356` implements `addRuntimeChunk` as an unconditional push, and legacy frames without IDs use `${event}:${sseId ?? this.seen.size}`. The source only establishes per-instance replay suppression; it does not establish duplicate suppression after adapter recreation, persisted message restoration or absent stable legacy cursor. Official normalization preserves eventId, sequence and runId in the chunk base; use them rather than regenerated IDs or title-based comparison.

Correction: define one notice per durable selection event across reconnect/restore, while retaining two distinct selection event IDs even when the agent title/ID is identical. Scope any duplicate protection to event/run identity at the existing ingestion boundary. Do not collapse legitimate repeated selections by agent ID. Author fixtures for same ID replay, increasing distinct IDs with same title, equal sequence/different ID behavior, fresh adapter/restored message and legacy cursor identity. A legacy stream without durable identity cannot support an unconditional replay guarantee; state that limit or establish the actual authoritative identity source.

### P2 — Catalog acceptance inventory must include every dependent record and visible fixture

Evidence: `frontend/src/shared/content/chunk.ts` defines the discriminated union plus `CHUNK_PHASE`, `CHUNK_BUBBLE_VISIBLE`, `CHUNK_RENDERER`, and `CHUNK_TRACE` as exhaustive records. Renderer dispatch is exhaustive via `assertNever`. `features/chat/model/chunk-catalog.test.ts:27–64` maintains `ALL_KINDS` and `CHUNK_FIXTURES`. `ui/chunks/chunk-catalog.stories.tsx` says every bubble-visible discriminant is present. `rich-data-renderers.tsx` derives data-part registrations from the visibility table.

Correction: add the new kind to union, renderer-name catalog, context phase, visibility, renderer and trace records, renderer dispatch, exhaustive fixture table/list and visible catalog story. Choose trace disposition intentionally and retain metadata in the existing event inspector. Source fixtures should assert context, not skill; visible registration and renderer dispatch; and unchanged hidden raw/step behavior. Avoid changing portable ContentBlock merely to transport a runtime-specific notice unless actual persistence requires it.

### P2 — Specify malformed-input behavior and text wrapping concretely

Evidence: `agui-schema.ts:26` validates CUSTOM name/value presence, not selection identity fields. `runtime-chunk-projection.ts` helpers distinguish strings and records but do not validate this new payload. `ChunkSurface` renders its children and uses existing theme tokens; no selection renderer exists yet. The proposed strict identity and plain-text treatment are appropriate trust-boundary rules, but remain unimplemented requirements.

Correction: require nonblank string agent ID; accept a nonblank title or display the real ID. Do not synthesize an identity for missing/non-string IDs. Keep malformed payloads diagnostic/raw, including original payload where the existing inspector permits. Preserve `sourceAgentId`, `selectedThreadId`, `rosterRevision`, routing method and envelope identity as metadata; do not interpret arbitrary strings as URLs, HTML or Markdown. Unknown methods must not invent “intent-based routing.” Explicitly include `min-w-0` and an appropriate long-token wrap rule on the text-bearing flex child if using a flex row; ordinary whitespace wrapping alone does not address an unbroken ID. Decorative icon stays aria-hidden. A single notice live region is preferable to duplicate hidden and visible announcements.

Required fixtures: whitespace-only/missing/non-string IDs; blank/missing title fallback; title containing HTML/Markdown, quotes, emoji, RTL and a long unbroken token; malformed `agent` array/null; absent optional metadata; unknown method; valid intent method. These are authored acceptance fixtures for later execution, not claims of current runtime defects.

## Strengths to preserve

- Backend `NormalizedEvent::AgentSelected` is a routing-metadata event, explicitly excluding classifier reasoning and private prompts (`src/uar/domain/events.rs:57`). `delegation.rs:384` emits it after child creation and before waiting for child output. The notice should retain this precise meaning; success, completion, duration and atomic roster guarantees are unwarranted.
- `src/uar/api/adapters.rs:137`, `api/sse.rs:521`, and `api/agui.rs:650` provide consistent semantic fields across projections. Frontend work can consume the real event without a new endpoint or fake business entity.
- Existing shared ChunkSurface tokens and pure renderer props support a narrow incumbent extension. No new store, fetch or agent-entity duplication is necessary for this display.

## Later evidence needed to prove the user outcome

Source inspection cannot prove visible timing, contrast, wrapping, announcement behavior or reconnect persistence. Following full-phase completion and exact operator approval, capture the actual production stream and UI: submit a request selecting a registered specialist; show the notice appearing while the answer is in progress, before terminal completion; correlate displayed identity to the durable event; reconnect/replay without duplicate notice; restore/reopen the persisted thread if required by the delivery contract; and demonstrate a subsequent ordinary answer still renders normally. Inspect long-name/ID wrapping at narrow width and zoom, both themes, and actual assistive-technology announcement behavior without focus movement. A static catalog story or mocked JSON alone does not prove live production delivery.

Deferred machine/runtime work: detector scan and mutable browser overlay, CLI engine/context probe, browser/server launch, tests, imports, typecheck, lint, format, builds and all acceptance scripts. No service/provider commands started. The complete Impeccable mandatory detector/browser run remains pending; this document must not be described as a passed critique. Later suggested command progression, subject to the phase gate: `$impeccable harden` for identity/text edge cases, `$impeccable audit` for the implemented scope, and `$impeccable polish` last.

## Run notes and handoff

Only this assessment file changed. Source inspections used `cat`, `sed` and `rg`; no verification command ran. Some search operands did not exist (`packages`, `frontend/src/features/chat/runtime`); subsequent inspected paths supplied the actual evidence. `.impeccable/critique/ignore.md` was absent. Target slug was not generated because invoking the skill engine is deferred under the closed gate. Detector: deferred, not attempted and not claimed unavailable. Browser visibility, overlay injection and console findings: deferred. No live server, browser tab or temporary runtime file was created, so no server or runtime cleanup was needed. Unresolved acceptance decisions are the durable legacy identity guarantee and persisted-message restoration coverage. This is a source snapshot of the shared checkout during implementation, not a frozen source identity or acceptance inventory.

Questions skipped: the requested user outcome is fixed; this assessment supplies source corrections without reopening product preferences.
