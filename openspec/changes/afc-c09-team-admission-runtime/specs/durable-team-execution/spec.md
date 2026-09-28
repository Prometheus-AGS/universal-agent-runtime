# Durable team execution

## ADDED Requirements

### Requirement: Admission is durable and bounded

UAR SHALL atomically reserve an aggregate team budget and a single fenced task attempt before dispatching a member turn. The reservation SHALL bind the current team, task, member, workspace, installed binding and ownership epoch. A duplicate request SHALL return the existing attempt, and a stale or over-budget request SHALL not dispatch.

#### Scenario: A worker is admitted once

Given an eligible ready task with a current owner and available team budget, when the operator starts it twice with the same command ID, then UAR records one reservation, one attempt and one dispatch intent.

#### Scenario: A stale owner tries to start work

Given task ownership moved to another member, when the previous member's epoch is used for admission, then UAR rejects the request without reserving budget or starting a turn.

### Requirement: Effective authority is current at effects

UAR SHALL recompute the current host, Cedar, binding and parent constraints at admission and revalidate team/member/task/attempt epochs before each protected effect. A C09.2 assignment or accepted message SHALL NOT by itself authorize execution or tools.

#### Scenario: Membership is revoked during a turn

Given an admitted member turn, when that membership is revoked before a protected tool effect, then the effect is denied and the attempt records the interruption or uncertain outcome truthfully.

### Requirement: Context and usage remain scoped and recoverable

UAR SHALL deliver only selected, authorized task context and artifact references to a member. It SHALL persist attempt, dispatch, reservation and usage state across a runtime restart, deduplicate cumulative usage by attempt/run identity, and retain unknown usage as reserved until reconciled.

#### Scenario: A restart follows reservation

Given a reservation committed before dispatch acknowledgement, when UAR restarts, then it recovers the same attempt and dispatch intent without creating a second reservation or silently reporting success.

#### Scenario: Two workspaces remain isolated

Given teams in separate workspaces, when one member requests context, artifacts or usage receipts, then UAR discloses only records granted to that member's workspace and team.
