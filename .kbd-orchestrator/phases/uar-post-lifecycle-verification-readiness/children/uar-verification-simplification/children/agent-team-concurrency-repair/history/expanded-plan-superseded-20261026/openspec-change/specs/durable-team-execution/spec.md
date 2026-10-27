# Spec Delta

## Purpose

Make admitted agent-team work observable through genuine member execution, safely reconciled terminal outcomes and dependent tasks that consume attributed member outputs.

## ADDED Requirements

### Requirement: Terminal settlement reconciles the original attempt without replay

UAR SHALL reconcile failed terminal persistence using the original owner, workspace, team, attempt, run and execution fence. An exact already-committed outcome SHALL be reused without duplicate artifacts, model/tool dispatch, reservation release or usage revision increments. Conflicting outcomes and stale authority SHALL be refused.

#### Scenario: SETTLE-01 repeat a known terminal outcome
- **WHEN** original durable execution evidence confirms a terminal outcome and an authorized reconciliation command is repeated
- **THEN** the same outcome and receipt are returned with unchanged committed output and usage revision
- **AND** no new execution attempt, provider call or duplicate artifact is created
- **AND** the same stability is observed for a genuine confirmed terminal outcome whose usage remains unknown; absent qualifying evidence blocks that acceptance claim

#### Scenario: Same command carries conflicting intent
- **WHEN** a reconciliation command ID is reused with changed payload or stale authority
- **THEN** UAR refuses the conflict without changing the original outcome or releasing its reservation

### Requirement: Unconfirmed settlement remains explicit and charged

UAR SHALL distinguish known execution outcome from accounting certainty. An unconfirmed catalog response SHALL NOT imply rollback or commit. If durable evidence cannot establish the original outcome, UAR SHALL expose uncertainty instead of leaving finished local work silently represented as active. Unknown reservations SHALL remain charged; recovery SHALL NOT replay agent work.

#### Scenario: Retained member failure has unknown accounting
- **WHEN** the retained failed mathematics attempt is reconciled and its exact terminal execution is established but usage remains unknown
- **THEN** the known failure is retained while the unknown reservation remains charged
- **AND** accounting certainty is not inferred from failure or elapsed time

#### Scenario: Authoritative outcome cannot be established
- **WHEN** the exact original terminal evidence is absent, conflicting or unreadable
- **THEN** the operation reports an explicit uncertain disposition with a safe diagnostic
- **AND** it does not claim completion, release liability or automatically redispatch the task

### Requirement: Independent specialist work overlaps before dependent integration

UAR SHALL permit independent eligible tasks assigned to distinct team members to execute concurrently within configured capacity. A dependent task SHALL not dispatch before its prerequisites succeed. Its selected context SHALL contain the attributed completed outputs it was authorized to consume.

#### Scenario: TEAM-01 mathematics and editorial work merge
- **WHEN** a team containing the saved math and writing specialists receives independent calculation and outline tasks followed by a dependent article
- **THEN** distinct real member runs have a proven overlapping active interval and successful terminal outcomes
- **AND** the math answer matches an independent calculation
- **AND** the final writer consumes both completed artifacts and returns an article containing that result, the outline headline and its saved byline

#### Scenario: Article is requested before prerequisites complete
- **WHEN** the article is admitted while either required task has not succeeded
- **THEN** admission is refused or remains non-executing under the documented dependency contract
- **AND** no article provider call or successful output is recorded
