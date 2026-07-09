## Context

Full trace confirmed the real round-trip mechanism, end to end:
1. An agent's tool call causes the backend to emit `NormalizedEvent::ArtifactInputRequest { run_id, artifact: ArtifactPayload }` (`src/uar/domain/events.rs`), serialized as `agui.artifact_input_request` on the run's SSE stream (`src/uar/api/sse.rs`).
2. `chat-stream-store.ts` handles that SSE event and adds a `__a2ui_input__` tool-call block to the thread's message store.
3. `enhanced-thread.tsx` renders that block as `<A2uiInputBlock>` (`frontend/src/features/chat/components/a2ui-artifact-block.tsx`), which renders a real confirm/select/text_input/form UI per `artifact_type`.
4. Submitting calls `submitArtifactResponse(runId, {artifact_id, response})` → real `POST /api/uar/runs/{run_id}/artifact-response` (`src/uar/a2ui/routes.rs`) → the backend emits a `ToolEnd` event back into the run, resuming the agent's tool-call loop.

Every piece of this is real and already correct. The only thing genuinely missing is a way to *originate* step 1 on demand, for testing, rather than only from a live agent tool call.

## Goals / Non-Goals

**Goals:**
- Let an operator trigger a real `ArtifactInputRequest` against a real, currently-active run, then observe and complete the round-trip through the actual production chat UI — no parallel/mock rendering path.
- Keep the new backend surface minimal: one endpoint, additive, no changes to the existing artifact-response flow.

**Non-Goals:**
- Building a way to test artifacts against a *synthetic* (non-running) session. If no run is currently active, the honest answer is "start a conversation first" — not inventing a fake run lifecycle just for this tool.
- Resolving `resolve-runtime-protocols-page-facade`'s dead AG-UI/model-route/A2UI-surface display panels (Round 2 of this phase) — that's the Runtime Console's read-only *display* of protocol events, a separate concern from this change's *trigger a real interactive round-trip* purpose. Both touch "A2UI" but are not the same problem.
- Auto-generating schema-conformant sample content for arbitrary user-defined schemas. The test page pre-fills a sensible example per `artifact_type` (confirm/select/text_input/form each have one), but the operator can edit the JSON directly — simpler and more honest than a content-generation engine that would itself need testing.

## Decisions

**1. New endpoint takes `{ artifact_type, title, content, metadata? }` directly, not `{ schema_id }` alone.**
Alternative considered: accept just a `schema_id` and have the backend derive sample content from the schema's `json_schema` field. Rejected — `ArtifactSchema.json_schema` describes the *shape* content must conform to, not an actual sample; auto-deriving valid sample content for arbitrary schemas (especially user-defined ones) is a non-trivial content-generation problem with its own correctness risk, disproportionate to this change's scope. The frontend still uses the schema browser to pick *which schema/artifact_type* to test, and pre-fills a known-good example per builtin `artifact_type` — the backend endpoint itself just needs a valid `ArtifactPayload` shape (minus `artifact_id`, which it generates) and doesn't need to know about the schema registry at all.

**2. "Go to thread" hands off to the real chat UI rather than rendering `A2uiInputBlock` inside the admin page.**
Alternative considered: render `<A2uiInputBlock>` directly inside `A2uiTestingPage` with the triggered run/artifact's data. Rejected as the primary path — it would require duplicating the SSE-subscription-to-tool-call-block wiring that `chat-stream-store.ts`/`enhanced-thread.tsx` already do correctly, doubling the surface that needs to stay correct. Handing off to `/threads` (the real chat page) means the round-trip is observed through the actual code path a real user would experience — closer to what "reuse the real chat components" means, and zero new rendering logic to maintain.

**3. Run targeting requires an active (`running`/`waiting`) run — no run creation from the test page.**
The test page doesn't start new agent conversations; it targets an existing one. This keeps the change additive and avoids conflating "test A2UI" with "start a chat" (a different, already-solid feature). If no active run exists, the page states that plainly.

## Risks / Trade-offs

- **[Risk]** `RunManager::emit_to_run` on a run with no listener currently subscribed (e.g. the operator hasn't navigated to that thread yet) — does the event get buffered for the next SSE connection, or lost? → **Mitigation**: confirmed during assessment that this project's runs already support resumable streaming with a 512-event buffer (per `uar-harness-parity`'s H5 delivery) — the same buffer that makes `GET /api/uar/runs/{id}/stream` resumable should deliver this event whenever the client next connects/reconnects to that run's stream. Verify this holds during implementation rather than assuming.
- **[Risk]** Triggering a synthetic artifact request on a run mid-way through genuine agent tool execution could interleave confusingly with real tool-call events in the transcript. → **Mitigation**: this is inherent to testing against a real run rather than a sandboxed one — acceptable given the goal is validating the real pipeline, and the artifact block is visually distinct (bordered `A2UI Input` panel) from ordinary tool calls.

## Migration Plan

Additive only. No rollback complexity — the new endpoint and reworked test page can be removed independently if needed without affecting the real artifact-response flow.

## Open Questions

None blocking.
