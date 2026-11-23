## ADDED Requirements

### Requirement: An explicit model beats the system default
The system default (registry default provider plus its `default_model`) SHALL apply only
when no agent, conversation or turn names a model; a change to it SHALL NOT alter a turn
whose agent, conversation or turn names one, and there SHALL be no force or lock option.
An agent's explicit model is its artifact `policy.provider.default` when provider and
model are both non-empty, or the model in its `uar.run_policy` extension.

#### Scenario: The system default changes while an agent pins a model
- **WHEN** an agent has an explicit model A and the operator changes the system default to model B
- **THEN** the next turn of that agent is routed to A

#### Scenario: The system default changes for an agent with no model
- **WHEN** an agent has no explicit model and the operator changes the system default to model B
- **THEN** the next turn of that agent in an already open conversation is routed to B without a restart

#### Scenario: A conversation model overrides the agent
- **WHEN** a conversation has a saved model C and its agent has explicit model A
- **THEN** the next turn is routed to C
- **AND** a later system default change does not alter that routing

#### Scenario: The extension carries the agent's model
- **WHEN** an agent's artifact default is empty and its `uar.run_policy` extension names model A
- **THEN** the agent is treated as having an explicit model and its turns are routed to A after a system default change

### Requirement: An agent model change applies to the next turn
Editing an agent's model SHALL take effect on the next turn of an open conversation for
that agent, without a restart, on whichever replica serves that turn. An agent whose
explicit model cannot be routed SHALL fail the turn visibly and SHALL NOT fall back to the
system default.

#### Scenario: The agent model is edited
- **WHEN** an operator changes the model of an agent that has an open conversation
- **THEN** the next turn of that conversation is routed to the new model
- **AND** the same holds when a different replica serves that turn

#### Scenario: The new agent model is unavailable
- **WHEN** an operator sets an agent model that no registered provider offers
- **THEN** the next turn fails with a visible error and no request is made to any model
- **AND** the system default model is not used instead

### Requirement: The console shows which agents keep an explicit model when the default changes
When the operator changes the system default, the console SHALL list the agents that keep
an explicit model, with each agent's model, and SHALL state how many agents follow the
new default. The explicit-model rule SHALL be the same one the runtime uses. The list
SHALL be visible with the change result and from the picker, so a change that does not
reach an agent is never silent. Team-member and conversation overrides are not part of
this list.

#### Scenario: Agents that pin a model are listed
- **WHEN** an operator changes the system default and two of five agents have explicit models
- **THEN** the console names those two agents with their models
- **AND** it states that three agents follow the new default

#### Scenario: Every agent follows the default
- **WHEN** no agent has an explicit model
- **THEN** the console states that no agent is pinned

#### Scenario: Pin through the extension is shown
- **WHEN** an agent's model comes only from its `uar.run_policy` extension
- **THEN** that agent is listed as pinned

#### Scenario: Agents cannot be read
- **WHEN** the agent list fails to load when the default changes
- **THEN** the console reports that pinned agents could not be determined
- **AND** it does not state that every agent follows the new default

### Requirement: A sub-agent's explicit model wins over conversation and system defaults (pending operator confirmation)
Status: proposed, pending operator confirmation; the operator has decided only that an
agent's explicit model beats the system default. An agent or sub-agent's explicit model
SHALL win over conversation and system defaults. A sub-agent without one SHALL inherit its
parent's effective model. A sub-agent whose explicit model cannot be routed SHALL fail
delegation visibly and SHALL NOT be replaced by another model.

#### Scenario: A conversation model does not reach a sub-agent with its own model
- **WHEN** a conversation selects model C, the root agent delegates to a sub-agent whose artifact names model D, and model D's provider has a granted credential
- **THEN** the sub-agent's model is D and the root agent's model is C

#### Scenario: A sub-agent without a model inherits
- **WHEN** a conversation selects model C and the root agent delegates to a sub-agent with no explicit model
- **THEN** the sub-agent's model is C

#### Scenario: A sub-agent model cannot be routed
- **WHEN** the sub-agent's explicit model names a provider with no granted credential
- **THEN** delegation fails with a visible error
- **AND** no other model is substituted

### Requirement: Each answer shows the model that served it, why and since when
The transcript SHALL show, for every assistant turn, the provider and the model id the
provider reported as having produced the response, the origin of that choice (`system
default`, `this conversation`, `agent` or `configured fallback`) and, for `system default`,
the time the default became that model, to the second. It SHALL NOT show a requested or
candidate model as the answering model.

#### Scenario: A conversation follows a changed default
- **WHEN** the system default changes from model A to model B and a conversation with no override sends a message
- **THEN** its answer's receipt names model B, the origin `system default` and the time the default changed
- **AND** answers given before the change keep their original receipts

#### Scenario: A conversation override answers
- **WHEN** a conversation with an override sends a message after the default changes
- **THEN** its answer's receipt names the override model with the origin `this conversation`

#### Scenario: The provider reports a different model id
- **WHEN** the model id reported by the provider differs from the requested one
- **THEN** the receipt shows the reported id on the serving provider
- **AND** the requested id is not shown as the answering model

#### Scenario: A turn keeps the time of the model it started with
- **WHEN** the system default changes while a turn is generating
- **THEN** that turn's receipt shows the time its own model became the default, not the time of the later change

#### Scenario: A refused choice is never reported as served
- **WHEN** a requested model was refused or is unavailable
- **THEN** no receipt names that model as the answering model

### Requirement: Clearing a conversation override previews the inherited result
Before a conversation's model override is cleared, the sheet SHALL show the model the
conversation will inherit and its origin, and SHALL change nothing until the person
confirms. If the inherited model is unavailable the sheet SHALL say so and SHALL NOT clear
the override.

#### Scenario: The preview is shown and confirmed
- **WHEN** a person clears the override of a conversation whose inherited model is available
- **THEN** the sheet previews that model with the origin `system default`
- **AND** after confirmation the conversation's next answer is produced by that model

#### Scenario: The clear is cancelled
- **WHEN** the person cancels after seeing the preview
- **THEN** the override and the next answer's model are unchanged

#### Scenario: The inherited model is unavailable
- **WHEN** the inherited model's provider is disabled
- **THEN** the sheet states that the inherited model is unavailable and does not clear the override
- **AND** the conversation continues to answer on its override

### Requirement: One conversation's model choice does not change another
Setting, changing or clearing a conversation's model override SHALL change only that
conversation. It SHALL NOT change another conversation, the agent definition, the system
default or any other scope.

#### Scenario: Two conversations share an agent
- **WHEN** one conversation sets an override and another conversation using the same agent has none
- **THEN** the second conversation's next answer still follows the system default
- **AND** the agent's stored model and the global policy are unchanged

### Requirement: A turn already generating finishes on the model it started with
A turn whose generation has started SHALL finish on the model it was admitted with. A
change of the system default SHALL apply from the next message and SHALL NOT restart or
re-send the generating turn.

#### Scenario: The default changes mid-generation
- **WHEN** the system default changes while a turn is generating on model A
- **THEN** that turn completes on model A with one request and its receipt names model A
- **AND** the conversation's next message is answered by the new default

### Requirement: A model choice that cannot be served is refused with a reason
A system default or conversation override naming a disabled provider or a model its
provider does not offer SHALL be refused with a reason the person can act on. The previous
effective value SHALL remain in force and be shown.

#### Scenario: A disabled provider is chosen as the default
- **WHEN** an operator chooses a model of a disabled provider as the system default
- **THEN** the change is refused with a reason that names the provider as disabled
- **AND** the system default and the next answer's model are unchanged

#### Scenario: An unavailable model is chosen as an override
- **WHEN** a person chooses a model that the provider does not offer as a conversation override
- **THEN** the choice is refused with a reason that says the model is not offered
- **AND** the conversation's next answer uses the previous effective model
