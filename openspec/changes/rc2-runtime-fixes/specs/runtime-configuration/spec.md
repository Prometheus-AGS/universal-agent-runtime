## ADDED Requirements

### Requirement: The LLM model follows the documented source precedence
UAR SHALL take its default LLM model from `UAR_LLM__MODEL` when set, otherwise from `LLM_MODEL`, otherwise from the configuration file's `llm.model`.

#### Scenario: Removing each source in turn
- **WHEN** all three sources name different models and UAR restarts first without `UAR_LLM__MODEL` and then without `LLM_MODEL`
- **THEN** the loaded model is, in order, the `UAR_LLM__MODEL` value, the `LLM_MODEL` value and the configuration file value.

### Requirement: Memory uses and reports the configured embedding API
The memory service SHALL use the adapter named by `memory.embedding_api`, falling back to `memory.embedding_provider`, SHALL refuse to start on an unsupported value, and SHALL log the adapter it actually uses.

#### Scenario: Mistral against an OpenAI-compatible endpoint
- **WHEN** memory is configured with `embedding_api: mistral` and `embedding_model: mistral-embed`
- **THEN** the startup log reports `embedding_api=mistral` and 1024 dimensions
- **AND** saving a memory record sends a `mistral-embed` request to the configured endpoint.

#### Scenario: Unsupported embedding API
- **WHEN** UAR starts with `memory.embedding_api: bogus`
- **THEN** startup fails and the error names the unsupported value.
