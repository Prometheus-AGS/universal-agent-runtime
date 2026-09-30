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

### Requirement: Exact endpoint settings remain distinct from route and price

UAR SHALL capture configured provider/model route, served wire alias, canonical price identity and trusted endpoint/profile/settings revisions separately. Omitted reasoning SHALL resolve off without family/history-derived request fields. Explicit unsupported or unknown reasoning SHALL refuse before dispatch. Settings-only validation SHALL explicitly make no guaranteed-fit claim; justified counting/limits evidence SHALL be required for guaranteed fit. Historical attempts SHALL retain captured settings across explicit revisioned rebind.

#### Scenario: A gateway alias is executed under its actual schema

Given a configured qualified route whose served alias differs, when a member is admitted with its validated settings, then the leaf retains the alias and captured profile, and unsupported explicit reasoning is refused without a provider request or silent downgrade.

### Requirement: One catalog executor owns current execution authority

UAR SHALL acquire a non-expiring catalog CAS execution claim with service identity, fresh incarnation and monotonic epoch before executable capability/admission. Admission, dispatch, recovery, effects and execution mutations SHALL check that fence. Clean release SHALL require confirmed owned-root/child cleanup. Crash SHALL NOT permit timeout takeover. Replacement SHALL require authenticated privileged operator authorization, current expected epoch, reason and verified fencing evidence; model/tool authority SHALL NOT confer this permission.

#### Scenario: Authorized recovery replaces an excluded executor

Given confirmed old-executor/child exclusion, when a privileged operator submits current expected epoch and evidence, then replacement records old/new fences and an audit receipt, transfers queued authority once and retains dispatched work as uncertain without replay.

#### Scenario: A competing or unauthorized executor is refused

Given a held claim, when a second executor tries execution/recovery or an unauthenticated, unauthorized, stale-epoch or insufficient-evidence replacement is requested, then the operation is refused without ownership mutation, provider dispatch or protected effect.

### Requirement: Resource migration and known output are truthful

UAR SHALL diagnose exact unsupported nonempty definition context artifact/history/memory/fork selections and private binding context-grant pointers as required unsupported until implemented. Empty selections, supported contextStrategy and exact skills SHALL retain their supported meaning. Required RAG SHALL retain its existing refusal. Unmapped permittedChildren/nested-subteam execution SHALL remain unsupported. An explicit next-schema optional exclusion SHALL be reported; legacy declarations SHALL NOT silently downgrade. Recovery SHALL preserve known successful output/dependency readiness while unknown accounting remains reserved, and SHALL block unsafe replay for unknown effects.

#### Scenario: A required history declaration cannot be fulfilled

Given AgentDefinition context.history is selected or authorized-summary without an enforcing mapping, when executable preflight or launch occurs, then it refuses at /context/history with a stable diagnostic and remediation rather than silently joining ambient history.

#### Scenario: Known execution survives accounting recovery

Given a successful task/output with unresolved usage but confirmed effects, when recovery occurs, then success/output/readiness remains recorded and its unknown reservation stays charged.

### Requirement: Durable peer tools authorize directed scope before disclosure

For the approved cooperating-pair profile, UAR SHALL derive sender/scope from the current bound attempt and enforce installed directed communication edge/mode plus current task/delegation authority before roster/inbox disclosure, send, admission and result selection. Queue-only messages SHALL NOT activate work. Explicit delegation SHALL commit task/envelope/queued attempt/reservation/command receipt in one catalog CAS; duplicate command IDs SHALL retain original identity and changed payloads SHALL conflict. Ordinary child-thread tool meanings SHALL remain unchanged.

#### Scenario: A forbidden same-team edge cannot act

Given valid same-team membership but a forbidden or revoked directed edge, when a member requests inbox content, send, delegation or selected result disclosure, then content/action is denied without an accepted envelope or new attempt. A reverse reply edge SHALL NOT be inferred.

### Requirement: A safe yield resumes through one fresh continuation

UAR SHALL persist typed yield intent, stop further model/tool dispatch and join exact roots/children with confirmed effects before releasing active capacity. Durable waits SHALL evaluate all named same-team targets, refuse cycles and invalidate old authority on reassignment/revocation. A qualifying wait SHALL admit at most one uniquely linked continuation with fresh attempt/run/root and current authorization/budget. Queued reservations SHALL count against budget/pending bounds rather than active slots. Unknown effects SHALL block unsafe wake/replay; unknown usage SHALL remain reserved.

#### Scenario: One-slot cooperation completes once

Given global/team capacity one, when the coordinator delegates and safely yields, then the worker executes and the coordinator resumes once with selected result artifacts under new authority and completes its task.

#### Scenario: Wait readiness is complete and recoverable

Given multiple targets and a durable wait, when only one completes, then no continuation is admitted; when all have known terminal outcomes, including failure/cancellation, then actual outcomes are carried into one continuation or a visible blocked reason. A cycle refuses; reassignment invalidates the old wait; replay/restart does not duplicate admission or replay dispatched work.
