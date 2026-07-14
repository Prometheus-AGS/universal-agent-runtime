# RAG citation UX

## Purpose

Give RAG-augmented chat responses numbered `[1]`, `[2]` citation
markers that trace back to the actual retrieved knowledge chunk, on
both the backend SSE event stream and the chat transcript UI, via a
transport-agnostic `CitationStream` type reusable by future consumers
(A2UI surfaces, eval harnesses) beyond the chat SSE path.

## ADDED Requirements

### Requirement: CitationStream numbers retrieval results

`src/uar/rag/citation_stream.rs` MUST expose a `CitationStream` type
that assigns stable 1-based marker numbers to a list of retrieved
knowledge chunks, in the order they are supplied. Markers MUST be
assignable from either raw `KnowledgeMatch` results or an assembled
`RAGContext`, and MUST NOT depend on any transport, SSE, or UI type —
so a future consumer (e.g. an A2UI renderer or eval harness) can use
`CitationStream::markers()` directly without going through the SSE
event stream at all.

#### Scenario: Two retrieved chunks are numbered in order

- **WHEN** `CitationStream::from_matches` is called with two
  `KnowledgeMatch` results, in relevance order
- **THEN** the first result's marker is `1` and the second's is `2`

#### Scenario: No chunks were retrieved

- **WHEN** `CitationStream::from_matches` is called with an empty
  slice
- **THEN** `CitationStream::is_empty()` is `true`
- **AND** `prompt_block()` returns an empty string
- **AND** `to_normalized_event(run_id)` returns `None`

### Requirement: Numbered markers are injected into the prompt

The RAG-augmented chat path (`RunManager::execute_run` in
`src/uar/runtime/manager.rs`) MUST inject `CitationStream::prompt_block()`
into the system prompt in place of any unnumbered retrieval-content
injection, so the model has a `[1]`, `[2]`, ... numbering scheme to
cite back against that matches the numbering emitted on the SSE
stream.

#### Scenario: An agent with a knowledge base attached answers a question

- **WHEN** `artifact.memory.kb.enabled` is `true` and retrieval returns
  one or more chunks for the user's turn
- **THEN** the outgoing LLM request's system prompt contains a
  `[RELEVANT KNOWLEDGE]` block with each chunk prefixed by its
  1-based `[n]` marker

### Requirement: RagCitations SSE event carries the numbered set

`src/uar/domain/events.rs` MUST expose a `NormalizedEvent::RagCitations
{ run_id, citations: Vec<RagCitation> }` variant, distinct from the
existing `NormalizedEvent::Citation` variant (which carries LLM-native
*web* citations — URL/title/snippet — sourced from the LLM protocol
stream, not RAG retrieval). Each `RagCitation` MUST carry `marker`,
`chunk_id`, `document_id`, `document_name`, `relevance_score`, and
`snippet`. The event MUST be emitted on the run's broadcast channel
before the model's answer streams, whenever retrieval returned a
non-empty `CitationStream`; it MUST NOT be emitted for an empty stream.

#### Scenario: A RAG-augmented turn emits its citation set

- **WHEN** retrieval for a chat turn returns 2 chunks
- **THEN** a `NormalizedEvent::RagCitations` event is emitted on that
  run's stream, containing exactly 2 `RagCitation` entries with
  `marker` values `1` and `2`

#### Scenario: A turn with no knowledge base attached

- **WHEN** `artifact.memory.kb.enabled` is `false`, or retrieval
  returns no chunks
- **THEN** no `NormalizedEvent::RagCitations` event is emitted for
  that run

### Requirement: RagCitations is carried on both SSE event vocabularies

Both `to_agui_event` and `to_agui_spec_event` in `src/uar/api/sse.rs`
MUST map `NormalizedEvent::RagCitations` to a wire event: the legacy
vocabulary as `agui.rag_citations`, and the AG-UI-spec vocabulary as a
`CUSTOM` event named `uar.rag_citations`. Both mappings MUST omit the
event entirely (return `None`) when the citations list is empty.

#### Scenario: A client using the legacy AG-UI vocabulary

- **WHEN** the server streams with `stream_mode` set to the legacy
  `agui` mode
- **THEN** a non-empty `RagCitations` event arrives as
  `event: agui.rag_citations` with a `citations` array in its payload

#### Scenario: A client using the official AG-UI-spec vocabulary

- **WHEN** the server streams with `stream_mode` set to `agui_spec`
- **THEN** a non-empty `RagCitations` event arrives as a `CUSTOM` event
  with `name: "uar.rag_citations"`

### Requirement: Hover-to-source panel in the chat transcript

The chat transcript (`frontend/src/components/assistant-ui/enhanced-thread.tsx`)
MUST render an ordered strip of numbered citation badges under any
assistant message that carries a `rag-citations` content block.
Hovering a badge MUST reveal a panel showing the cited chunk's
document name, relevance score, and snippet. The component MUST
render nothing when the message has no citations, and MUST follow the
codebase's Components → Hooks → Stores layering (the rendering
component reads citation data through a hook, never a store or
`fetch` call directly).

#### Scenario: An assistant message carries citations

- **WHEN** an assistant message's content includes a `rag-citations`
  block with 2 markers
- **THEN** the transcript shows 2 numbered badges under that message
- **AND** hovering the first badge shows its document name and
  snippet in a hover panel

#### Scenario: An assistant message carries no citations

- **WHEN** an assistant message's content has no `rag-citations` block
- **THEN** no citation badge row is rendered for that message
