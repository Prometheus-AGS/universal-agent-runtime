## ADDED Requirements

### Requirement: A delegated child turn without an explicit output cap receives the delegated default allowance
A captured child turn that runs on the local thread kernel, is not a team attempt, and has neither an explicit output cap nor a destination budget contract SHALL receive the delegated default output allowance of 8,192 tokens. For a known context window, the default SHALL be at most half that window, but never less than the ordinary response reserve. The model's maximum output and the context-capacity check SHALL still apply.

#### Scenario: Routed child with no explicit cap
- **WHEN** the orchestrator routes a request to a registered child agent whose model has a known 1,048,576-token window and a 64,000-token maximum output, and no caller cap or destination contract applies
- **THEN** the child turn's output allowance is 8,192 tokens, and the owned log records `delegated output allowance bound` with `source` `delegated_default`

#### Scenario: Explicit caller cap takes precedence
- **WHEN** a root request carries an explicit output cap that is passed to its child turn
- **THEN** the child turn uses that cap, clamped by the model maximum, and not the delegated default

#### Scenario: Team attempts are excluded
- **WHEN** a child turn is a team attempt
- **THEN** it keeps the team output allocation and the team output check, and does not receive the delegated default

#### Scenario: Small known context window
- **WHEN** a child turn with no explicit cap runs on a model whose known context window is 4,000 tokens and whose maximum output is unknown
- **THEN** its default output allowance is 2,000 tokens, half the window, and not 8,192
