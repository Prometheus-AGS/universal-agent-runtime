## ADDED Requirements

### Requirement: Non-widening Presentation eligibility
UAR SHALL resolve Presentation selections against enabled owner-accessible templates through global, agent, conversation and turn scopes. Narrower scopes and client support SHALL NOT widen parent eligibility.

#### Scenario: A conversation requests a denied template
- **WHEN** a parent policy excludes a template and a conversation explicitly selects it
- **THEN** the effective policy excludes it and records the exclusion

#### Scenario: Delegated execution
- **WHEN** the host creates a child thread
- **THEN** its Presentation eligibility is no broader than its parent's effective ceiling

#### Scenario: Editing inherited policy
- **WHEN** a user opens and saves other conversation settings without changing Presentation assignment
- **THEN** the inherited selection remains inherited rather than becoming a copied explicit list
