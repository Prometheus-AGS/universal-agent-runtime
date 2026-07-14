## ADDED Requirements

### Requirement: Retrieval hits are visible provenance events
When knowledge-base retrieval contributes context to a run, the stream SHALL
emit events identifying the knowledge base and document for each used chunk,
and the chat UI SHALL render them.

#### Scenario: KB hit shown in UI
- **WHEN** a user message triggers retrieval with at least one chunk above threshold
- **THEN** the AG-UI stream carries citation/KB-hit events with KB and document identity and the chat UI displays them

#### Scenario: Ingestion status reaches indexed
- **WHEN** a document finishes chunking and embedding successfully
- **THEN** its status transitions from pending to indexed on SurrealDB and Postgres providers
