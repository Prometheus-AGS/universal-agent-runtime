## ADDED Requirements

### Requirement: Provider default models use bounded selection
The Provider Overrides surface SHALL present each configured provider's default model as an accessible selection control whose options are exactly that provider's enabled configured models.

#### Scenario: Provider model options are opened
- **WHEN** an operator opens the default-model control for a configured provider
- **THEN** every enabled model in that provider's configured model list is available for selection
- **AND** disabled models and models owned only by other providers are not available

#### Scenario: Provider default model is selected
- **WHEN** an operator selects one of the provider's available models
- **THEN** the provider settings draft records that model id as `default_model`
- **AND** the existing settings save and realtime reconciliation path remains in use

### Requirement: Sensitive setting masks preserve secret length
Settings API responses SHALL obscure every character of a stored API key with one mask character and SHALL NOT return any plaintext character from the stored key.

#### Scenario: Stored provider API key is read
- **WHEN** a provider settings record contains an API key with N characters
- **THEN** the response contains an API-key mask with exactly N characters
- **AND** every returned character is a mask character

#### Scenario: Provider API key is absent
- **WHEN** a provider settings record has no API key or has an empty API key
- **THEN** the response does not fabricate a non-empty credential mask

### Requirement: Unchanged nested credential masks are non-destructive
The settings API SHALL preserve an existing nested API key when an update submits the unchanged response mask while modifying other fields in the same settings object.

#### Scenario: Unrelated provider field is saved
- **WHEN** an operator changes a non-sensitive provider field and the request includes the unchanged API-key mask returned by the settings API
- **THEN** the existing stored API key remains unchanged
- **AND** the response continues to return only its length-preserving mask

#### Scenario: Replacement provider API key is saved
- **WHEN** an operator submits a new API-key value that does not equal the current response mask
- **THEN** the new value replaces the stored API key
- **AND** subsequent reads return a mask matching the new value's character count
