## ADDED Requirements

### Requirement: Execution paths expose correlated ordered lifecycle events
Linear, graph and subagent paths SHALL emit redacted RuntimeStep-equivalent lifecycle records with run, parent/child, node, iteration and tool identities where applicable. Records SHALL reflect committed state, preserve per-run causal order and produce one terminal result per execution identity. Replay SHALL retain identity and SHALL NOT create new execution.

#### Scenario: Graph and child trace
- **WHEN** a graph node spawns a child that executes tools and returns
- **THEN** its ordered events correlate the parent run, node, iteration, child and tool outcomes without prompt bodies, hidden reasoning or credentials

#### Scenario: Failure and replay
- **WHEN** a node fails and a subscriber reconnects from a cursor
- **THEN** replay exposes the same correlated terminal outcome once logically, without rerunning the node or tools

### Requirement: Host execution enforces shared limits and cleanup
The trusted host SHALL enforce root/child policy intersection, deny-final approvals and existing bounded iteration/depth/concurrency/total-child limits. Model and tool usage across retries, summarization and child work SHALL consume shared budget exactly once with explicit unknown usage. Exhaustion SHALL stop new dispatch. Cancellation SHALL propagate, drain or reconcile outstanding work, and retain inspectable cleanup status; terminal cancellation SHALL NOT conceal live orphan work.

#### Scenario: Approval denied
- **WHEN** a suspended linear or graph tool request is denied by governing policy or the operator
- **THEN** no child, resume path or adapter can execute the denied action and one truthful terminal result is recorded

#### Scenario: Budget or iteration exhaustion
- **WHEN** child requests and parent work reach the configured shared budget or iteration limit
- **THEN** further dispatch is refused, already incurred usage remains attributed, and unused reservations are released only on proof they were not dispatched

#### Scenario: Cancel with outstanding children
- **WHEN** cancellation arrives during graph or remote child execution
- **THEN** new dispatch stops, cancellation propagates, cleanup reaches a recorded outcome and any unresolved remote work remains explicit rather than a false clean completion

#### Scenario: Resume after a completed effect
- **WHEN** a checkpoint resumes after a tool effect completed but before a subsequent node began
- **THEN** the host restores the receipt, resumes pending work and does not replay that completed effect

