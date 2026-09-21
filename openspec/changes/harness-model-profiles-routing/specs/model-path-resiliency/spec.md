## MODIFIED Requirements

### Requirement: Model selection and failover respect provider health
The runtime SHALL consult provider health when selecting the run model and each configured fallback, skip cooldown providers, and consider every configured fallback in order until an admissible destination succeeds. Admission SHALL require current tenant policy, capabilities, context/output fit, compatible templates/continuity and remaining shared cost/latency budgets. Ineligible candidates SHALL be skipped with a redacted reason. If no configured destination is admissible, the runtime SHALL return an explicit unavailable, unsupported or budget outcome without dispatch.

#### Scenario: Primary in cooldown
- **WHEN** the policy-resolved provider is in cooldown and a healthy admissible fallback is configured
- **THEN** the run uses the fallback without attempting the primary

#### Scenario: Healthy but too small fallback
- **WHEN** the next healthy fallback cannot fit protected content
- **THEN** its reason is recorded, subsequent configured candidates are considered in order, and none receives a silently truncated request

## ADDED Requirements

### Requirement: Exhausted candidate selection is explicit
The runtime SHALL terminate selection without dispatch when every configured candidate is rejected, retaining redacted reasons and canonical protected data.

#### Scenario: Every candidate is rejected
- **WHEN** all configured candidates are unavailable, incompatible, forbidden or unable to satisfy the remaining budget
- **THEN** the runtime returns a typed unavailable, unsupported or budget outcome with redacted selection reasons and sends no provider request

### Requirement: Routing is constrained and explainable
Routing SHALL evaluate task classification, required capabilities/context, explicit model choice, configured availability, tenant/governance constraints and cost/latency budgets before ranking admissible destinations. An explicit model choice SHALL be preserved unless existing policy explicitly authorizes fallback. Ranking signals SHALL carry benchmark provenance and version; learned routing SHALL remain experimental until a frozen local quality/cost/latency comparison passes predefined thresholds.

#### Scenario: Explicit disallowed or unavailable model
- **WHEN** the selected model is forbidden or unavailable and substitution is not authorized
- **THEN** the runtime returns an explicit outcome instead of silently selecting another model

#### Scenario: Routing baseline
- **WHEN** a proposed selector is evaluated on a frozen representative task set
- **THEN** results report quality, cost, latency, classification errors and predicted-versus-actual usage against the current selector with dataset, model and benchmark provenance; unmeasured improvements are not claimed

### Requirement: Destination changes preserve retry safety
Retries and failovers SHALL retain existing typed-error retryability, Retry-After/backoff/attempt/total-time limits, idle timeouts and the semantic commit boundary. Metadata-only frames SHALL NOT become semantic output. Completed tool effects SHALL NOT be replayed by request preparation, transport retry or stream resume.

#### Scenario: Rate limit before semantic output
- **WHEN** a retryable rate limit occurs before semantic output
- **THEN** retry policy and current budget govern the next freshly prepared attempt without rerunning tools

#### Scenario: Failure after semantic output
- **WHEN** an established stream fails after semantic content was committed
- **THEN** the run persists and reports interruption under the existing contract rather than replaying or failing over transparently
