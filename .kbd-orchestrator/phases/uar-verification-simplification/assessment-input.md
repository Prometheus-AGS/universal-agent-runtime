# Verification simplification assessment input

Planning research only. Active child: `uar-verification-simplification`; parent: `uar-post-lifecycle-verification-readiness`. Operator verification lock: **CLOSED**. This artifact proposes design; it establishes no executed behavior or acceptance verdict.

## Smallest reusable real path

Reuse `configured_prompt` as the first workflow: configure the genuine provider, generate a fresh fact and arithmetic task, invoke the protected default agent, and compare the answer with an independently computed result (`scripts/acceptance/cases_prompt.py:152–164`). Its `prompt` helper submits `/v1/chat/completions`, correlates native run/session/agent/model identity, and requires settled provider accounting plus run-bound genuine provenance (`scripts/acceptance/cases_prompt.py:50–139`). These checks distinguish useful inference from a successful HTTP response. Configuration currently adds a separate genuine `/test` request (`scripts/acceptance/cases_prompt.py:17–46`); expose that cost separately rather than describing this as a single inference call.

Proposed minimum entry: one explicitly selected workflow, platform artifact, durable backend profile, admitted provider binding, existing cumulative budget authority, isolated resources and receipt directory. Preserve the useful predicate and genuine provenance; do not require another platform or unrelated SDK/embedding assets to exercise this entry. This requires runner changes: the existing subset path still enumerates every fixture backend and three repetitions (`scripts/acceptance/user_outcomes.py:229–268`). A first diagnostic repetition must remain visibly incomplete against the retained three-repetition acceptance requirement.

## Observed architectural coupling

The fixture loader mandates both durable adapters, all six lifecycle/provenance hooks and frozen executables before dispatch (`scripts/acceptance/user_outcomes.py:49–68,114–129`). Collection disallows selection, requires exactly 97 cases, requires 25 families, creates backend-by-case-by-three rows and blocks missing prerequisites (`scripts/acceptance/failure_sweep.py:28–42,280–314`). Therefore small workflows cannot independently enter collection through this contract.

The platform controller loads macOS and Linux fixtures together, requires identical packet/source/case identities, shares one process owner, and stops later-platform dispatch when corrected source requires rebinding (`scripts/acceptance/platform_suite.py:56–66,185–212`). Its loader fixes Windows to blocked and Android to deferred (`scripts/acceptance/platform_suite.py:47–49`). These are controller constraints, not proof that platform workflows ran.

The parent plan records repeated preparation and authority blockers and still-incomplete derived tasks (`.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/plan.md:178–200`). The uncomfortable limit: simplifying orchestration alone cannot supply missing artifacts, executors, credentials or genuine functional evidence.

## Proposed independent workflow groups

Retain a complete, versioned coverage registry; dispatch selected cells independently with only actual prerequisites:

1. Configured inference and interfaces: native/chat, OpenAI, Anthropic, ACP, current/legacy A2A HTTP/gRPC, AG-UI, A2UI continuation, shipped SDK/UI paths. Separate streaming/terminal/usefulness predicates; polling cannot stand in for streaming (`docs/user-outcome-verification.md:17–21,47–61`).
2. Durable agents and routing: custom CRUD, preserved protected default/orchestrator definitions, saved configuration, intent/current-roster selection, selection stream and follow-up (`docs/user-outcome-verification.md:23–45`).
3. Skills and context: effective strategies, bounded reattachment, protected records, structured output and actual final-wire budgets (`docs/user-outcome-verification.md:63–92`).
4. Knowledge lifecycle: genuine grounding, same-/changed-width regeneration, interruption and restart (`docs/user-outcome-verification.md:111–117`).
5. Teams and operational continuity: ordering, actual concurrency, consumed context, cancellation/recovery; REST configuration/startup/restart (`.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/plan.md:17,22`).

Record every applicable server cell separately on Postgres+pgvector and disk-backed Surreal, across macOS/Linux/Windows. Record iOS embedded host persistence independently; existing documentation expressly limits its scope and excludes server-interface/both-adapter certification (`docs/user-outcome-verification.md:157–159`). Keep Windows missing-host cells BLOCKED; Android DEFERRED with historical failure retained. Full coverage remains required; selected workflow success never promotes absent cells to PASS.

## KEEP / ADAPT / RETIRE

| Disposition | Harness responsibility |
| --- | --- |
| KEEP | Separate protocol/useful predicates; redacted exclusive evidence and hash-chain receipts (`scripts/acceptance/core.py:42–61,74–129`); real restart and exact-owned cleanup (`scripts/acceptance/core.py:493–521`). |
| ADAPT | Fixture admission into workflow-scoped resource contracts; retain actual source/artifact/config/schema/store identity checks (`scripts/acceptance/user_outcomes.py:192–214`). Shared cumulative accounting remains authoritative across workflow invocations. |
| ADAPT | Dependency-aware collection and source-aware aggregation into a general workflow scheduler; preserve FAIL/BLOCKED/NOT_RUN and mixed-source refusal (`scripts/acceptance/failure_sweep.py:46–55,300–343`). |
| RETIRE as mandatory entry | Hardcoded 97/582 execution shape and all-platform prebinding. Retain historical manifests/results as history and map all obligations into the registry. |
| RETIRE as default workflow | Inline correction waiting/artifact refresh. Prefer terminal failure receipt, separate authorized implementation, then an explicit same-item rerun; retain the original failure and correction consumption (`scripts/acceptance/failure_sweep.py:322–343`). |

A thin wrapper around the old suite is cheaper but preserves these blockers. A wholesale harness replacement risks losing discriminating predicates. Prefer adapting the existing case/evidence primitives behind a smaller runner. Any replacement authority is a proposal only: preserve all historical failures, UNKNOWN reservations, calls, active time, repair consumption and separate provider ceilings; no budget reset or implied execution permission.

Handoff: inspected the three assigned harness files, `user_outcomes.py`, `cases_prompt.py`, the product verification document and parent plan. Only this assessment file was authored. No source execution, imports, validation, services, network, provider calls or verification occurred. Current source identity and proposed runtime behavior remain unverified.
