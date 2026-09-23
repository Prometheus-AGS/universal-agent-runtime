## Purpose

Defines which chunking strategies a knowledge base can select, how strategy names are parsed, that the selected strategy is the one applied when documents are ingested, and how agentic chunking behaves and fails.

## ADDED Requirements

### Requirement: Strategy names parse to the strategy they name
Every chunking strategy name the runtime offers in its settings or accepts in its knowledge-base API SHALL parse to that strategy. `agentic` SHALL select agentic chunking. `fixed_size` and `fixed` SHALL both select fixed-size chunking with the requested size.

#### Scenario: Agentic selected through the API
- **WHEN** a knowledge base is created with `chunk_strategy` `agentic`
- **THEN** the stored strategy is agentic chunking, not recursive chunking

#### Scenario: Settings spelling of fixed size
- **WHEN** a knowledge base configuration names `fixed_size` with a chunk size of 100
- **THEN** the stored strategy is fixed-size chunking of 100 characters

### Requirement: Ingestion uses the knowledge base's explicitly chosen strategy
When a document is ingested into a knowledge base whose chunking strategy was chosen explicitly, the runtime SHALL chunk it with that stored strategy. A strategy SHALL count as explicitly chosen only when a create or update request for that knowledge base carried a chunking strategy or a chunk size. The runtime SHALL record the strategy actually used, including a fallback, in each stored chunk's metadata, and SHALL report in the knowledge-base response whether the strategy was chosen explicitly.

#### Scenario: Two knowledge bases, two strategies
- **WHEN** the same document is uploaded to a knowledge base created with fixed-size chunking of 100 characters and to one created with agentic chunking
- **THEN** the first stores 100-character chunks and the second stores the chunks agentic chunking produced, and each chunk's metadata names its strategy

#### Scenario: Explicit recursive choice is honoured
- **WHEN** a knowledge base is created with `chunk_strategy` `recursive` and `chunk_size` 512 and a document is uploaded to it
- **THEN** the document is chunked recursively at 512 characters and the knowledge base reports its strategy as explicitly chosen

### Requirement: Knowledge bases without an explicit strategy keep semantic chunking
When a document is ingested into a knowledge base whose chunking strategy was never chosen explicitly, the runtime SHALL chunk it with semantic chunking at similarity threshold 0.5, as it did before this change, whatever default strategy value is stored for that knowledge base. Knowledge bases stored before this change SHALL be treated as having no explicit strategy.

#### Scenario: Existing knowledge base with a stored default
- **WHEN** a knowledge base stored before this change, whose stored strategy is recursive 512 and which carries no explicit-choice marker, receives a new upload
- **THEN** the document is chunked semantically at threshold 0.5, each chunk's metadata names semantic chunking, and the knowledge base reports its strategy as not explicitly chosen

#### Scenario: Knowledge base created without a strategy
- **WHEN** a knowledge base is created with a configuration that names neither a chunking strategy nor a chunk size, and a document is uploaded to it
- **THEN** the document is chunked semantically at threshold 0.5

### Requirement: Agentic chunking splits at model-proposed boundaries
Agentic chunking SHALL split a document into contiguous units, ask the configured language model to propose where new chunks begin as offsets into those units, and cut the document at the proposed unit starts. Each chunk SHALL be an exact slice of the input, and the chunks in order SHALL concatenate to the exact input. Documents longer than one model window SHALL be processed window by window without losing or repeating any text.

#### Scenario: Three-topic fixture
- **WHEN** a document with three topics is chunked and the model proposes boundaries at the starts of the second and third topics
- **THEN** more than one chunk is returned, the chunk start offsets equal the proposed unit starts, and the chunks concatenate to the exact input

#### Scenario: Document longer than one window
- **WHEN** a document spans more than three model windows
- **THEN** the model is asked once per window, no chunk is longer than one window, and the chunks concatenate to the exact input

### Requirement: Agentic chunking falls back to recursive chunking
When the model call fails, no model is configured, no usable credential is available, the reply is not parseable, the proposed offsets are out of range or not strictly increasing, or the reconstruction check fails, agentic chunking SHALL chunk the whole document with recursive chunking at 512 characters and SHALL report the fallback and its reason in chunk metadata and on the document record. It SHALL NOT fail the ingestion because of the fallback, and SHALL NOT store or log the model provider's error text.

#### Scenario: Model error
- **WHEN** agentic chunking is selected and the model call returns an error
- **THEN** the chunks equal the recursive chunking of the same document and their metadata reports the recursive fallback

#### Scenario: Invalid offsets
- **WHEN** the model proposes offsets that decrease or exceed the window
- **THEN** the chunks equal the recursive chunking of the same document and their metadata reports the recursive fallback

### Requirement: Document ingestion accepts a request-scoped model credential
A document upload SHALL accept an optional model credential with the same fields and validation rules as a run-scoped credential: provider id, provider kind, base URL, API key and optional default model. The runtime SHALL use it only for that document's agentic chunking, with the credential's default model. It SHALL NOT persist, serialize, log or trace the credential's key or base URL, and SHALL NOT keep the credential after that document's ingestion finishes. An invalid credential, or a credential without a default model sent to a knowledge base that chunks agentically, SHALL be rejected with HTTP 422 before the document is saved, and the error SHALL NOT contain the key, the base URL or the submitted value.

#### Scenario: Agentic ingestion with a request credential
- **WHEN** a document is uploaded to an agentic knowledge base with a credential whose base URL points at a provider that proposes chunk boundaries
- **THEN** that provider receives the chunking request with the credential's key, the document is stored as more than one chunk at the proposed boundaries, and the document reports agentic chunking with no fallback

#### Scenario: Invalid credential
- **WHEN** an upload carries a credential with an unsupported provider kind, a base URL with user information, or a plain-HTTP base URL to a non-loopback host
- **THEN** the response is 422, no document is saved, and the response body contains neither the key nor the base URL

#### Scenario: The credential leaves no residue
- **WHEN** an ingestion that used a request credential has finished
- **THEN** neither the key nor the base URL appears in captured logs, the settings file, the persistence directory, the document record or any chunk's metadata

### Requirement: Agentic ingestion without a usable credential reports its fallback
When a knowledge base's effective strategy is agentic and a launch-token-authenticated upload carries no model credential, the runtime SHALL NOT call any model, including the host's default model, SHALL chunk the document recursively at 512 characters, and SHALL report the strategy used and the reason `no_credential` in the upload response and on the document record. When agentic chunking falls back for any other reason, the document record SHALL report the strategy used and the reason once the document is indexed.

#### Scenario: Sidecar upload without a credential
- **WHEN** the host uploads a document to an agentic knowledge base without a credential
- **THEN** the upload response reports the fallback reason `no_credential`, no model request is made, and once indexed the document reports `recursive_fallback` with reason `no_credential` and its chunks equal the recursive chunking of the same text

#### Scenario: Provider error with a request credential
- **WHEN** the provider at the credential's base URL answers the chunking request with an error
- **THEN** the document is indexed with recursive chunks, reports `recursive_fallback` with reason `call_failed`, and its record contains no text from the provider's error

### Requirement: Knowledge bases belong to the request's principal
In sidecar mode, creating, listing, reading, updating, deleting, uploading to and searching knowledge bases SHALL act as the principal the host asserted for the request. A knowledge base created under one principal SHALL NOT be visible to, or changed by, requests under another principal.

#### Scenario: Knowledge base of another principal
- **WHEN** principal A creates a knowledge base and uploads a document, and principal B lists knowledge bases, reads, updates, deletes, uploads to or searches that knowledge base by id
- **THEN** B's list omits it, every by-id request returns 404, and A's knowledge base and document are unchanged

### Requirement: Hosts can detect request-scoped ingestion credentials
The runtime SHALL list the capability `ingest_scoped_credentials` in `GET /api/uar/capabilities` exactly when document uploads accept a request-scoped model credential and report chunking fallbacks on the document.

#### Scenario: Capability listed
- **WHEN** the host reads the capabilities of a runtime that implements request-scoped ingestion credentials
- **THEN** the list contains `ingest_scoped_credentials`
