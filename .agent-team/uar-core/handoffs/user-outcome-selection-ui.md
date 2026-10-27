# Shipped selection notice source handoff

2026-10-25. Active phase uar-post-lifecycle-verification-readiness; implementation, verification CLOSED. This is source authoring, not UI or runtime acceptance.

## Files and behavior

- frontend/src/shared/content/chunk.ts: typed agent-selection chunk and exhaustive phase, visibility, renderer and trace mappings. Actual child-thread identity correlates the host routing event; no completion status is inferred.
- frontend/src/shared/content/agent-selection.ts: shared validated host/persisted parser, stable child identity, portable reserved artifact, non-announcing restore, malformed reserved metadata diagnostic and recovery from a corrupt preferred chunks cache when canonical content is valid.
- frontend/src/shared/content/to-chunks.ts: portable reserved artifact projection through the normal persisted content path.
- frontend/src/platform/agui/{runtime-chunk-projection,agui-normalizer,agui-adapter}.ts: actual official CUSTOM and mounted legacy agent-selected event projection; context phase. Only actual wire sequences advance official replay ordering; a sequence-less legacy display fallback cannot poison later finite official sequence values.
- frontend/src/features/chat/ui/chunks/{agent-selection-chunk,chunk-renderer}.tsx: small incumbent ChunkSurface, selected name/intent method, stable data-agent-selection selector, wrapped long names, no focus movement or invented completion.
- frontend/src/features/chat/use-selection-announcement.ts: ephemeral document-local accessibility consumption only; one fresh polite notice, no stored replay or remount duplicate. This set owns no agent/domain data.
- frontend/src/stores/chat-message-store.ts: single-flight awaited history load, late hydration cannot replace a live stream; selected-child dedup before creating a new assistant bubble; validated live notice plus portable canonical content.
- frontend/src/platform/pglite/client.ts: sanitize both insert and preferred nonempty stored-chunks restoration; no persisted live announcement request is trusted.
- frontend/src/features/chat/use-chat-runtime.ts: per-thread history-load effect with explicit failed-load handling.
- frontend/src/stores/chat-stream-store.ts: root authored await-hydration before dispatch and actual legacy-selected runtime chunk insertion. Exclusive ownership transferred to manager_integrate for A2UI continuation extension; these branches must be preserved.
- Source-only coverage authored in shared/content/agent-selection.test.ts, platform/agui/agui-adapter.test.ts, stores/chat-message-store.test.ts, stores/chat-history-hydration.test.ts, features/chat/use-selection-announcement.test.tsx and features/chat/model/chunk-catalog.test.ts. No tests executed.

## Design and constraints

Small selection UI direction underwent two isolated source/design critiques and fresh artifact-only adversarial review plus correction at reviews/user-outcome-selection-ui/. Instrumented skill audits, browser and code review gates remain deferred. Existing platform/content projection and persisted message flow are retained; this bounded change does not certify a wholesale migration of legacy chat Zustand state into the entity graph. No new provider/agent store or component-side fetch was introduced.

Observed guards address malformed persisted identity, replay duplicates, legacy sequence poisoning, delayed history overwrites and duplicate accessibility announcements. They trace to required live/replay/installed-user behavior. No unsolicited feature was added. Every original dirty file and receipt remains preserved.

## Unverified / integration

No test, import, formatter, lint, typecheck, build, browser, screenshot or machine review command ran. Type compatibility, actual dual SSE delivery, PGlite durability, live timing and keyboard/screen-reader behavior remain unverified. Browser worker authored U24 actual shipped UI selectors and evidence, but no browser ran. Lead must integrate standard A2UI callback/continuation, freeze installed assets and executable closure, complete all remaining phase implementation and obtain exact renewed operator authority before any gate. No phase task or review is marked PASS here.
