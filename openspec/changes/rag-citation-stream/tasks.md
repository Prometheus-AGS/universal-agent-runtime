## 1. Audit the real RAG-augmented chat path

- [x] 1.1 Read `src/normalized.rs` and `src/uar/domain/events.rs` — confirmed
  two separate `NormalizedEvent` enums (LLM-protocol-facing vs.
  domain/SSE-facing) and that both already have a `Citation` variant
  for LLM-native *web* citations (URL/title/snippet), not RAG chunks.
- [x] 1.2 Read `src/uar/rag/retrieval.rs`, `src/uar/rag/pipeline.rs`,
  `src/uar/api/knowledge.rs` — confirmed `HybridRetriever::build_context`
  and `RagRetrievalPipeline` are only reachable from the standalone
  `POST /api/knowledge/{id}/search` REST endpoint, not the chat stream.
- [x] 1.3 Found the real integration point: `RunManager::execute_run`
  (`src/uar/runtime/manager.rs`, ~line 724) — the only code path that
  does RAG retrieval as part of a chat turn, injecting unnumbered
  bullets into the system prompt with zero citation output.
  **Plan-assumption correction**: plan.md's Change 13 done-condition
  assumed a citation-carrying RAG-augmented streaming path already
  existed to "wire into"; it did not. Documented in `proposal.md`'s
  `## Why`.

## 2. `CitationStream` type

- [x] 2.1 `src/uar/rag/citation_stream.rs`: `CitationStream` wrapping
  `Vec<crate::uar::domain::events::RagCitation>`, with
  `from_matches(&[KnowledgeMatch], &HashMap<String,String>)`,
  `from_context(&RAGContext)`, `from_citations(&[Citation])` builders.
- [x] 2.2 `prompt_block()` — renders the numbered `[1] (doc) snippet`
  block for system-prompt injection.
- [x] 2.3 `to_normalized_event(run_id)` — `Option<NormalizedEvent>`,
  `None` for an empty stream (nothing to inject or emit).
- [x] 2.4 `pub mod citation_stream;` added to `src/uar/rag/mod.rs`.
- [x] 2.5 Unit tests: empty stream, 1-based marker ordering, document-name
  fallback (unresolved id → id; missing id → chunk id), prompt-block
  contents, snippet truncation, event conversion, `from_context` ordering.

## 3. Domain event wiring

- [x] 3.1 `RagCitation` struct + `NormalizedEvent::RagCitations` variant
  added to `src/uar/domain/events.rs`, documented as distinct from
  `Citation` (web citations).
- [x] 3.2 `src/uar/api/sse.rs`: `to_agui_event` maps `RagCitations` →
  `agui.rag_citations` (`None` when citations is empty, matching the
  existing `Citation` arm's empty-guard convention).
- [x] 3.3 `src/uar/api/sse.rs`: `to_agui_spec_event` maps `RagCitations` →
  `CUSTOM` `uar.rag_citations`, same pattern as `uar.citation.added`.
- [x] 3.4 Confirmed `to_runtime_entity_event`, `adapters.rs::to_ag_ui`,
  `openai/routes.rs`'s stream loop all have wildcard/catch-all arms —
  no changes needed there for the new variant to compile.

## 4. Wire into `RunManager::execute_run`

- [x] 4.1 Resolve document names best-effort via
  `PersistenceLayer::get_document` (already on the trait), deduped by
  `document_id` per turn.
- [x] 4.2 Build `CitationStream::from_matches(&matches, &document_names)`,
  replace the old unnumbered-bullet injection with
  `citation_stream.prompt_block()`.
- [x] 4.3 Emit `citation_stream.to_normalized_event(run_id)` via the
  existing `RunEventEmitter` (`emitter.emit(...)`) before the model
  streams its answer, so the client can resolve `[n]` markers as soon
  as they appear.
- [x] 4.4 Removed the now-unused `std::fmt::Write` import (the old
  `writeln!`-based bullet injection was the only caller in this file).

## 5. Frontend wiring

- [x] 5.1 `frontend/src/types/chat-content.ts`: `RagCitationMarker`,
  `RagCitationsContentBlock`, added to the `ContentBlock` union.
- [x] 5.2 `frontend/src/protocols/agui-adapter.ts`: `uar.rag_citations`
  CUSTOM-event case added to `customToLegacy`.
- [x] 5.3 `frontend/src/stores/chat-stream-store.ts`: `AguiRagCitations`
  wire type (snake_case, matching backend JSON), `agui.rag_citations`
  switch case converts to camelCase and calls
  `chatMessageStore.addRagCitations`.
- [x] 5.4 `frontend/src/stores/chat-message-store.ts`: `addRagCitations`
  action, following the existing `addContextUpdate`/`addCitation`
  streaming-state pattern.
- [x] 5.5 `frontend/src/hooks/use-message-citations.ts`: hook selecting
  a message's `rag-citations` block via `selectMessageById` (hooks
  layer — components never touch the store directly).
- [x] 5.6 `frontend/src/components/citations/citation-hover-panel.tsx`:
  `CitationBadge` (one `[n]` marker + `HoverCard`) and `MessageCitations`
  (ordered strip, renders nothing when empty).
- [x] 5.7 Wired `MessageCitations` into
  `frontend/src/components/assistant-ui/enhanced-thread.tsx`'s
  `AssistantMessage`, directly under the message bubble.
- [ ] 5.8 **Deferred**: inline-linking the literal `[n]` substrings the
  model emits mid-message to their `CitationBadge` (see proposal.md's
  "Out of scope" — the Sources-strip approach ships instead).

## 6. BDD coverage

- [x] 6.1 `tests/bdd/features/rag-citation.feature`: two scenarios
  (citations appear for a RAG-augmented turn; no citations when no KB
  is attached), reusing the existing `kb-retrieval`/`no-kb`
  Given-step vocabulary from `chat-kb-retrieval.feature` /
  `chat-no-kb.feature`.
- [x] 6.2 `tests/bdd/steps/rag-citation.steps.ts`: three new `Then`
  steps (`a RAG citation source badge is shown`, `no RAG citation
  source badge is shown`, `hovering the first citation badge reveals
  its source document`), driving the real `[aria-label="Sources"]`
  DOM the frontend renders.
- [x] 6.3 `bddgen test -c tests/bdd/playwright.config.ts` + `playwright
  test -c tests/bdd/playwright.config.ts` — 8/8 scenarios passed,
  including both new `rag-citation.feature` scenarios.

## 7. Verification

- [x] 7.1 `cargo check --locked --no-default-features --features server-full`
  — clean, zero warnings (full workspace, ~15.5 min cold build).
- [x] 7.2 `cargo test --locked --no-default-features --features server-full citation_stream`
  — see result recorded in the final report (long-running due to cold
  test-profile rebuild in this fresh worktree).
- [x] 7.3 `pnpm -C frontend install` (used `--no-frozen-lockfile`: the
  committed lockfile already predates this worktree's HEAD — an
  unrelated `@vitest/coverage-v8` drift, not introduced by this
  change) + `git submodule update --init frontend/packages/prometheus-entity-management`
  (was an uninitialized submodule in this worktree, unrelated to this
  change) + `pnpm -C frontend/packages/prometheus-entity-management build`
  (workspace package ships no prebuilt `dist/`).
- [x] 7.4 `pnpm typecheck` — clean, zero errors, after the above setup.
- [x] 7.5 Final focused validation: `cargo fmt --all -- --check`,
  `cargo test --locked --no-default-features --features server-full --lib
  citation_stream` (8/8), `pnpm typecheck`, `pnpm lint`, and `pnpm build`
  all passed. The full BDD build chain also built the production bundle and
  both Rust binaries before the 8/8 Playwright run.
