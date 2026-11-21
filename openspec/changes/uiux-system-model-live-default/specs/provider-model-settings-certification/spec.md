## ADDED Requirements

### Requirement: The system default model is the registry default provider plus its default model
The system-level model SHALL be the registry default provider plus that provider's
`default_model`. The console SHALL change it by writing the provider's `default_model`
(when it differs) and then calling the live default route (`POST /api/uar/providers/{id}/default`),
in that order, and MUST NOT write `llm.default_provider`, `llm.model` or
`run_policy.global.model`. The next turn that resolves no higher-level model SHALL use the
new pair without a restart.

#### Scenario: Operator changes provider and model
- **WHEN** an operator selects an enabled provider and one of its models as the system default in the console and confirms
- **THEN** the console persists that provider's `default_model` and then calls the live default route
- **AND** the next turn of an open conversation whose agent names no model is routed to the selected provider and model without restarting the service
- **AND** the console shows the selected pair as the live system default

#### Scenario: Operator changes only the model of the current default provider
- **WHEN** an operator selects another model of the provider that is already the registry default
- **THEN** the provider's `default_model` is persisted and the live default route is called for the same provider
- **AND** the next turn of an agent with no model uses the newly selected model

#### Scenario: The system default survives restart
- **WHEN** the service is restarted after a successful system default change
- **THEN** the registry default provider and its `default_model` read back as the selected pair
- **AND** the console shows the same pair

#### Scenario: The selected provider no longer exists
- **WHEN** an operator confirms a selection whose provider was removed after the picker loaded
- **THEN** the default route returns not found, the live and durable default are unchanged and the console reports the failure on the picker
- **AND** the console does not show the failed pair as the system default

#### Scenario: The default route cannot persist
- **WHEN** the provider's `default_model` already equals the selection, the store is unreachable and the operator confirms
- **THEN** the console issues only the default route call, which returns a persist error
- **AND** the live and stored default are unchanged on every replica and no providers or settings invalidation is published
- **AND** the console shows the error and does not show the selected pair as live

#### Scenario: The model save fails
- **WHEN** the selected model differs from the provider's `default_model`, the store is unreachable and the operator confirms
- **THEN** the provider save returns a persist error and the console issues no default route call
- **AND** the console shows the save error and states that the system default was not changed

#### Scenario: The route fails after the model was persisted
- **WHEN** the provider's `default_model` write succeeds and the default route then fails
- **THEN** the live registry default provider is unchanged
- **AND** the console states that the model was saved but the system default was not changed

#### Scenario: The picker allows no unusable pair
- **WHEN** a provider is disabled, not configured, or lists no models, or a model is absent from its provider's models
- **THEN** the picker does not allow selecting that pair as the system default

#### Scenario: No competing write path
- **WHEN** an operator completes any system default change from the console
- **THEN** the only settings-related writes are the provider update route and the live default route
- **AND** no write to `llm.default_provider`, `llm.model` or `run_policy.global.model` is issued by the console

### Requirement: The system default picker handles the full model catalog
The system default picker SHALL list the whole bundled model catalog and the configured
models as provider-qualified entries, filter them as the operator types, and show the number
of matches. It SHALL render a bounded number of option elements regardless of catalog size,
and SHALL stay within the picker responsiveness budget (a long task of at most 200 ms, a
proposed budget). Entries that cannot be a system default SHALL be shown as not selectable.

#### Scenario: The full catalog is filtered
- **WHEN** an operator opens the picker with the full bundled catalog and types a query
- **THEN** the picker shows the number of matching entries
- **AND** no more than 200 option elements are in the page and no task blocks the page for more than 200 ms

#### Scenario: An entry that cannot be the default is chosen
- **WHEN** an operator activates an entry whose provider is not configured and enabled
- **THEN** nothing is selected and the stored default is unchanged

### Requirement: A default change is confirmed only after the runtime acknowledges it
The console SHALL show confirmation of a system default change only after the live route
has acknowledged it, and SHALL then show the requested value and the activation `live`. A
change the runtime did not acknowledge SHALL show no confirmation.

#### Scenario: The route acknowledges the change
- **WHEN** the live default route answers successfully
- **THEN** the confirmation appears after that answer with the requested pair and the label `live`

#### Scenario: The route has not answered yet
- **WHEN** the operator confirms and the live default route has not yet answered
- **THEN** no confirmation is shown and the control does not show the new pair as live

#### Scenario: The route does not acknowledge
- **WHEN** the live default route returns an error
- **THEN** no confirmation is shown and the previous default stays displayed

### Requirement: The default route rejects a disabled provider
The live default route SHALL reject a provider that is disabled, because a disabled
default provider leaves no usable system default. The rejection SHALL change no durable
or live state.

#### Scenario: A disabled provider is requested as default
- **WHEN** a client calls the default route for a registered provider that is disabled
- **THEN** the route returns 409 with the error `Provider '{id}' is disabled`
- **AND** the registry default and the persisted default provider are unchanged

### Requirement: The system default picker is reachable from primary navigation
The console SHALL expose the system default picker on a destination in primary
navigation and through the command palette, without requiring a direct URL. The picker
SHALL show the live system default pair and an explicit state when no usable default
exists (no default provider, a disabled default provider, or a default provider without a
`default_model`). Every other control that sets a provider's default model or the default
provider SHALL perform the same single action.

#### Scenario: Reaching the picker from the console landing page
- **WHEN** an operator opens the console and uses primary navigation or the command palette to open the system default model control
- **THEN** the picker opens within two interactions, operable by keyboard alone
- **AND** it works at the compact and desktop widths used for certification

#### Scenario: No usable default
- **WHEN** the registry has no default provider, or the default provider is disabled or has no `default_model`
- **THEN** the picker states that no usable system default exists and which part is missing
- **AND** it does not display a stale pair as the default

#### Scenario: Another default control is used
- **WHEN** an operator uses the Models-page star or the Providers-page make-default action
- **THEN** the same two-step write and the same live-default display apply as for the picker

### Requirement: run_policy.global.model is retired from model selection
The resolver SHALL NOT use a `model` carried by the Global scope of `run_policy.global`,
on the service path or the embedded path. A stored value SHALL be inert, SHALL NOT be the
provenance of the effective model, and SHALL have no console control. Other Global fields
keep their effect. The effective model resolves Turn, Conversation, Agent (explicit),
then the registry default provider and its `default_model`.

#### Scenario: A stored global model does not select a model
- **WHEN** `run_policy.global.model` holds a route, the agent names no model, and the registry default is another provider and model
- **THEN** the stored Global `model` reads back unchanged before and after the turn
- **AND** the effective model of the next turn is the registry default pair
- **AND** the effective run policy's model provenance is not the Global scope

#### Scenario: Higher levels still win
- **WHEN** `run_policy.global.model` holds a route and a conversation policy names a different model
- **THEN** the effective model is the conversation model

#### Scenario: Other global policy fields are unaffected
- **WHEN** `run_policy.global` carries a presentation or tool-approval setting
- **THEN** that setting continues to apply to the effective run policy as before
