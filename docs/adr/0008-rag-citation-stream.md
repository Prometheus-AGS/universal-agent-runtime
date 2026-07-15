# 8. Emit a RAG citation stream

Date: 2026-07-13

## Status

Accepted

## Context

RAG responses need to be auditable. Users and downstream systems must know which sources support which claims. A citation stream is the standard way to surface this information.

## Decision

- Add a `CitationStream` type in `src/uar/rag/`.
- Emit citations as `[1]`, `[2]` markers on the SSE event channel alongside the generated text.
- Build a React hover-to-source panel that consumes the same stream.
- Ensure A2UI surfaces can consume the citation stream without transformation.
- Cover the feature with a BDD scenario in `tests/bdd/features/rag-citation.feature`.

## Consequences

- Users can verify the origin of every RAG claim.
- The same citation stream feeds both the React operator UI and A2UI renderers.
- The feature is testable end-to-end via BDD.

## Alternatives considered

- Inline full citations in every chunk: rejected because it bloats the stream and complicates rendering.
- Post-hoc citation matching: rejected because it is less accurate than emitting citations at generation time.
