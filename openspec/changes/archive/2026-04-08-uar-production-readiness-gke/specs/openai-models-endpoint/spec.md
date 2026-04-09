## ADDED Requirements

### Requirement: List available models via OpenAI-compatible endpoint
The server SHALL expose `GET /v1/models` returning a list of available models in OpenAI API format.

#### Scenario: List all configured models
- **WHEN** a GET request is made to `/v1/models`
- **THEN** the server returns HTTP 200 with `{"object": "list", "data": [...]}` where each entry has `id`, `object: "model"`, `created`, and `owned_by` fields

#### Scenario: Only configured providers shown
- **WHEN** a provider has no API key configured
- **THEN** models from that provider are excluded from the response

### Requirement: Retrieve single model details
The server SHALL expose `GET /v1/models/{model_id}` returning details for a specific model.

#### Scenario: Model exists
- **WHEN** a GET request is made to `/v1/models/openai/gpt-4o`
- **THEN** the server returns HTTP 200 with the model object including capabilities and pricing

#### Scenario: Model not found
- **WHEN** a GET request is made to `/v1/models/nonexistent/model`
- **THEN** the server returns HTTP 404 with an OpenAI-compatible error object
