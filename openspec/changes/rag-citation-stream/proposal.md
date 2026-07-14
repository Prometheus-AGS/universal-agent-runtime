## Why

The grade-A plan's Change 13 assumed the server "currently streams
RAG-augmented chat responses" with citations already threaded through
that stream. That assumption was wrong: an audit of `src/uar/rag/`
(`retrieval.rs`, `pipeline.rs`) and `src/uar/runtime/manager.rs` found
that RAG retrieval and the streaming chat pipeline were two disconnected
systems. `HybridRetriever::build_context` (in `retrieval.rs`) builds a
`Citation`-bearing `RAGContext`, but nothing in the codebase called it
outside its own module — `RAGContext` was dead beyond `retrieval.rs`.
The actual RAG-augmented chat path lives in `RunManager::execute_run`
(`src/uar/runtime/manager.rs`, ~line 724): when `artifact.memory.kb.enabled`,
it runs a raw vector search and appends unnumbered bullet content
(`"- {chunk.content}"`) to the system prompt — with no citation markers,
no SSE event, and no client-visible attribution at all. The existing
`NormalizedEvent::CitationAdded` / `NormalizedEvent::Citation` events
(both `src/normalized.rs` and `src/uar/domain/events.rs`) carry LLM-native
*web* citations (URL, title, snippet) sourced from the LLM protocol
stream — a different mechanism entirely from RAG chunk attribution, and
not populated by the RAG pipeline either.

This change corrects that gap: it builds numbered citation markers for
the RAG chunks that are actually injected into the prompt, and wires
them into the one real RAG-augmented chat code path, so `[1]`, `[2]`
markers the model cites back are attributable to a real retrieved chunk
on both the backend event stream and the frontend transcript.

## What Changes

- New `CitationStream` type in `src/uar/rag/citation_stream.rs`: builds
  1-based numbered citation markers from retrieval results
  (`KnowledgeMatch` or an assembled `RAGContext`), renders the numbered
  `[1] ... [2] ...` block injected into the system prompt, and converts
  to the new `NormalizedEvent::RagCitations` wire event.
- New `NormalizedEvent::RagCitations { run_id, citations: Vec<RagCitation> }`
  variant + `RagCitation` struct in `src/uar/domain/events.rs`, distinct
  from the existing LLM-native-web-citation `Citation` variant.
- `RunManager::execute_run`'s RAG block (`src/uar/runtime/manager.rs`)
  rewired: resolves per-chunk document names (best-effort, via
  `PersistenceLayer::get_document`), builds a `CitationStream`, injects
  its numbered prompt block instead of the old unnumbered bullets, and
  emits the `RagCitations` event on the run's broadcast channel before
  the model streams its answer.
- `src/uar/api/sse.rs`: `to_agui_event` and `to_agui_spec_event` both
  map `RagCitations` to `agui.rag_citations` (legacy) / `CUSTOM
  uar.rag_citations` (AG-UI spec profile) respectively.
- Frontend: `agui-adapter.ts` maps `uar.rag_citations` → `agui.rag_citations`;
  `chat-stream-store.ts` parses it and calls a new
  `chat-message-store.addRagCitations` action, which attaches a
  `rag-citations` content block to the streaming message.
- New hook `frontend/src/hooks/use-message-citations.ts` (selects a
  message's citation markers from the store) and new component
  `frontend/src/components/citations/citation-hover-panel.tsx`
  (`CitationBadge` — one `[n]` marker with a `HoverCard` showing
  document name / relevance / snippet; `MessageCitations` — the ordered
  strip of badges for one message), wired into
  `frontend/src/components/assistant-ui/enhanced-thread.tsx`'s
  `AssistantMessage` so every RAG-augmented reply shows its sources.
- `tests/bdd/features/rag-citation.feature` + `tests/bdd/steps/rag-citation.steps.ts`,
  reusing the existing `kb-retrieval` / `no-kb` step vocabulary.

## Capabilities

### New Capabilities

- `rag-citation-ux`: numbered RAG citation markers threaded from
  retrieval through the SSE event stream to a hover-to-source panel in
  the chat transcript.

## Impact

- **`src/uar/domain/events.rs`**: additive enum variant + new struct;
  `NormalizedEvent` is `#[non_exhaustive]`-equivalent in practice
  (matched exhaustively in only two places, both updated in this
  change: `src/uar/api/sse.rs`'s `to_agui_event` and
  `to_agui_spec_event`; other match sites — `adapters.rs`,
  `openai/routes.rs` — already have wildcard arms).
- **`src/uar/runtime/manager.rs`**: RAG block behavior change — the
  system prompt now gets a numbered `[RELEVANT KNOWLEDGE]` block
  instead of unnumbered bullets. This is a prompt-content change for
  any agent with `memory.kb.enabled`; response wording may shift
  slightly as models cite `[n]` markers, but retrieval inputs/outputs
  (which chunks, how many, in what order) are unchanged.
- **No new runtime dependencies.** Reuses `PersistenceLayer::get_document`
  (already on the trait) for document-name resolution.
- **Frontend**: additive content-block type (`RagCitationsContentBlock`)
  and store action; no existing content-block behavior changes.

## Out of scope

- **A2UI surfaces (Changes 18-20) actually consuming `CitationStream`.**
  Those changes don't exist yet in this worktree. `CitationStream` is
  deliberately transport-agnostic (built from retrieval results, no SSE
  or UI code inside it) so a future A2UI renderer can consume
  `CitationStream::markers()` directly without going through SSE at
  all — but no A2UI integration code is written here.
- **Migrating `src/uar/api/knowledge.rs`'s `RagRetrievalPipeline` (the
  CH-11-hardened decompose/dedup/verify pipeline) into the chat path.**
  `RunManager::execute_run` currently uses raw
  `PersistenceLayer::search_knowledge[_scoped]` calls, not
  `RagRetrievalPipeline`/`HybridRetriever`. Swapping the chat path onto
  the hardened pipeline is a real, valuable follow-up but is a
  retrieval-quality change orthogonal to citation streaming; doing both
  in one change would conflate two different risk surfaces. Tracked as
  follow-up.
- **Rendering `[n]` markers inline inside the streamed message text**
  (e.g. superscript-linking the literal `[1]` substring the model
  emits mid-sentence to its `CitationBadge`). This change renders the
  citation set as an ordered "Sources" strip below the message instead
  — simpler, robust to a model that never emits `[n]` verbatim or
  paraphrases the citation, and avoids fragile streamed-markdown regex
  parsing. Inline-linking is a plausible follow-up UX polish pass.
- **Deduplicating repeated `RagCitations` events within one run.** The
  event is emitted once per RAG retrieval call (currently once per
  `execute_run`, since retrieval runs once per user turn); multi-turn
  or multi-retrieval-per-turn agents are out of scope for this change.
- **A dedicated PGlite migration for citation markers.** Not needed:
  `db.insertMessage` (`frontend/src/lib/db.ts`) persists the whole
  `content: ContentBlock[]` array via `JSON.stringify`, so the new
  `rag-citations` block round-trips through PGlite for free, the same
  as every other content-block type — confirmed by reading
  `insertMessage`, not just assumed.
