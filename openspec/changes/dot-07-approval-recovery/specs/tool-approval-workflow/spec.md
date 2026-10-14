# Spec Delta

## ADDED Requirements

### Requirement: Recovered approvals require current executable authority

Recovered approvals SHALL expose current state and authorize at most one continuation under current identity, policy and bindings. Restart SHALL NOT turn a stale decision into execution authority.

#### Scenario: Pending approval after restart

- **WHEN** an authorized owner reads a pending approval after its original process exits
- **THEN** the current approval and continuation disposition remain visible and execution still requires valid current authority

#### Scenario: Competing decisions

- **WHEN** two processes attempt to consume the same approval or its binding has changed
- **THEN** at most one valid continuation is admitted and stale or expired decisions cannot authorize effects

#### Scenario: Pending checkpoint publication

- **WHEN** the trusted host publishes a recoverable pending challenge
- **THEN** its protected pre-effect checkpoint is already committed and the approval row contains only its stable checkpoint identity and binding digest
- **AND** a checkpoint with a pending marker cannot execute through ordinary resume without its exact current decision

#### Scenario: Orderly shutdown while pending

- **WHEN** the original host cancels and joins execution during orderly shutdown
- **THEN** it preserves only checkpoint-linked pending challenges after acknowledging cancellation of their prepared admission
- **AND** failed cancellation or uncertain effects block predecessor exclusion and the original expiry remains unchanged

#### Scenario: Current authority changed

- **WHEN** current owner, policy, descriptor, payload or binding cannot renew a recovered pending call
- **THEN** no effect executes and stale executable mapping becomes visibly invalidated
- **AND** unsupported storage or a persistence outage is an explicit unavailable outcome, never an invented approval

#### Scenario: Recovered block contains sibling calls

- **WHEN** a protected unresolved assistant call block contains the selected call and remaining siblings
- **THEN** the host executes calls in original order through current admission and each sibling's own approval
- **AND** model continuation receives only complete real results without fabricated history repair

#### Scenario: Decision admission response

- **WHEN** a fresh approval request admits a continuation before its async gate commits a decision
- **THEN** the response distinguishes continuation admission from durable decision state and execution success remains observable through the new run

#### Scenario: Unknown recovered effect

- **WHEN** the recovered executor commits its decision and loses effect certainty
- **THEN** its immutable continuation claim remains held and another decision cannot replay the original call

#### Scenario: Expired denied or unbound decision

- **WHEN** an original challenge expires, is denied or has no protected executable mapping
- **THEN** the current API exposes expired, denied or invalidated state and no effect executes
- **AND** restart does not renew the timeout or assign legacy issuer authority

#### Scenario: Denial without executable continuation

- **WHEN** an authenticated owner denies a current pending challenge without a protected continuation or on an unsupported resume backend
- **THEN** the pending decision can atomically become denied without a continuation claim or effect
- **AND** an approval still requires its protected executable mapping

#### Scenario: Deadline passes during decision persistence

- **WHEN** an approve or deny intent reaches storage after its immutable expiry, including after awaiting a writer lock
- **THEN** the atomic transition commits expired with no human decision and cannot commit approved or denied
- **AND** the host exposes the actual committed state and executes no effect from the lost decision

#### Scenario: Required invalidation persistence fails

- **WHEN** current authority rejects recovery but the required invalidation cannot be durably committed
- **THEN** the API or run events expose approval persistence unavailable without claiming a committed invalidation
- **AND** the acquired continuation claim remains held and retry cannot replay the call


