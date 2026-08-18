## ADDED Requirements

### Requirement: Retrieval hits are visible provenance events
When knowledge-base retrieval contributes context to a run, the stream SHALL
emit events identifying the knowledge base and document for each used chunk,
and the chat UI SHALL render a source badge for those hits. Chat retrieval SHALL
pass through the same decomposition, deduplication, verification, and audit
pipeline used by the knowledge search API.

#### Scenario: KB hit shown in UI
- **WHEN** a user message triggers retrieval with at least one chunk above threshold
- **THEN** the AG-UI stream carries citation/KB-hit events with knowledge-base ID, document ID, and document name, and the chat UI displays the source document

#### Scenario: Chat uses the hardened retrieval pipeline
- **WHEN** an agent-scoped chat message searches one or more resolved knowledge bases
- **THEN** retrieval runs through query decomposition, cross-query deduplication, verification annotation, result limiting, and the `rag.retrieval.decision` audit event before context injection

#### Scenario: Ingestion status reaches indexed
- **WHEN** a document finishes chunking and embedding successfully
- **THEN** its status transitions from pending to indexed on SurrealDB and Postgres providers
