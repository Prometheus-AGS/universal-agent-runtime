## MODIFIED Requirements

### Requirement: Knowledge journey works end to end
An administrator SHALL be able to create a knowledge base, upload and index a document with a supported embedding provider, retrieve ranked matches, bind eligible knowledge bases at global, agent, or conversation scope, ground chat with citations, and remove the data through the owning API and client interface.

#### Scenario: Grounded answer
- **WHEN** an indexed document contains an exact test fact and an effective run policy selects that knowledge base
- **THEN** search returns a non-zero ranked match and chat includes the fact with a resolvable citation

#### Scenario: Indexing failure
- **WHEN** document indexing fails
- **THEN** the UI displays the failure and offers a deterministic retry without falsely marking the document indexed

#### Scenario: No selected knowledge base
- **WHEN** the effective run policy selects no knowledge bases
- **THEN** UAR performs no knowledge retrieval and does not fall back to searching every knowledge base

#### Scenario: Configured embedding provider is enforced
- **WHEN** a knowledge base specifies an embedding provider and model
- **THEN** ingestion and query embedding use that configuration or fail visibly as unsupported
