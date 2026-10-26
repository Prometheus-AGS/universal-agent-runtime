## ADDED Requirements

### Requirement: Orchestrator intent routing uses the current registered roster
The orchestrator SHALL use intent classification and the current eligible registered-agent roster to determine the execution target, SHALL execute the selected agent with its persisted artifact and intersected policy through the existing thread service, and SHALL retain attribution and permitted context across multiple calls. Skill matching alone SHALL NOT establish agent routing. An unavailable or unregistered selection SHALL NOT execute as an invented target.

#### Scenario: Registered specialist handles matching intent
- **WHEN** a user prompts the orchestrator with an intent matching an eligible registered specialist
- **THEN** the routing decision identifies that specialist, its actual child execution supplies the useful response, and follow-up context remains correctly attributed

#### Scenario: Roster changes before the next request
- **WHEN** an authorized custom specialist is added, changed or removed
- **THEN** subsequent routing uses the current eligible roster and does not silently execute a stale or unregistered artifact

### Requirement: Agent selection streams to the requesting user
The orchestrator SHALL emit redacted structured selection information correlated to the run, selected agent and thread before the selected-agent response completes. AG-UI SHALL deliver this as a supported client-consumable event and the shipped client SHALL display the selection. Routing evidence SHALL NOT expose hidden reasoning, credentials or private agent prompts.

#### Scenario: AG-UI user observes specialist selection
- **WHEN** the orchestrator routes a streamed AG-UI request
- **THEN** the client receives and renders selected-agent information during execution and the subsequent response is attributable to that selected agent

#### Scenario: Client reconnects to a routed run
- **WHEN** the supported reconnect flow resumes event consumption
- **THEN** selection and response correlation remain coherent and reconnect does not cause duplicate selected-agent execution
