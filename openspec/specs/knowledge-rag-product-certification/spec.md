# knowledge-rag-product-certification Specification

## Purpose
TBD - created by archiving change certify-knowledge-rag-flow. Update Purpose after archive.
## Requirements
### Requirement: Knowledge journey works end to end
An administrator SHALL be able to create a knowledge base, upload and index a document, retrieve ranked matches, ground chat, and remove the data through the React interface.

#### Scenario: Grounded answer
- **WHEN** an indexed document contains an exact test fact and an agent uses that knowledge base
- **THEN** search returns a non-zero ranked match and chat includes the fact

#### Scenario: Indexing failure
- **WHEN** document indexing fails
- **THEN** the UI displays the failure and offers a deterministic retry without falsely marking the document indexed
