## MODIFIED Requirements

### Requirement: Provider default models use bounded selection
The Provider Overrides surface SHALL present each configured provider's default model as an accessible bounded selection control whose options are exactly that provider's enabled configured models. Inventories containing one through seven enabled models SHALL use the simple selection path, while inventories containing eight or more enabled models SHALL provide search over both display names and raw model identifiers without accepting free-form values.

#### Scenario: Provider model options are opened
- **WHEN** a provider has between one and seven enabled configured models and an operator opens its default-model control
- **THEN** every enabled model in that provider's configured model list is available through the simple selection path
- **AND** disabled models and models owned only by other providers are not available

#### Scenario: Large provider model inventory is opened
- **WHEN** a provider has eight or more enabled configured models and an operator opens its default-model control
- **THEN** the control provides a search input and every valid enabled model remains available
- **AND** the unfiltered option order matches the provider configuration order

#### Scenario: Provider models are searched
- **WHEN** an operator enters a search term in a large provider model inventory
- **THEN** matching is case-insensitive after trimming surrounding query whitespace
- **AND** a model remains visible when the literal term occurs in either its display name or raw model identifier
- **AND** a distinct no-match state appears when no valid model matches

#### Scenario: Provider default model is selected
- **WHEN** an operator selects one of the provider's available models with a pointer or keyboard
- **THEN** the provider settings draft records that model id as `default_model` exactly once
- **AND** the existing settings save and realtime reconciliation path remains in use
- **AND** arbitrary text cannot become the selected model

#### Scenario: Provider model labels are ambiguous
- **WHEN** two valid enabled models have the same display name
- **THEN** the selection results expose their raw model identifiers so the operator can distinguish them

#### Scenario: Stored provider model is unavailable
- **WHEN** the stored default model is not present in the provider's current enabled model list
- **THEN** the control reports the stale value as unavailable and offers valid replacements
- **AND** it does not automatically select or save a replacement
