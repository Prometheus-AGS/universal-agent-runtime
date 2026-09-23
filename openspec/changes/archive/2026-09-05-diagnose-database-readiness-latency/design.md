## Context

See proposal.md for motivation. The reviewed diagnostic plan and assessment live
under .kbd-orchestrator/phases/skills-a2ui-library-and-runtime-observability/children/
agui-a2ui-selection-architecture/children/diagnose-database-readiness-latency/.
The plan's Observation contract defines the exact bounded sequence; this design
does not authorize another sequence. Source confirms full-table readiness work,
but recorded scalar CLI times combine setup and query work. Causation is unknown.

## Goals / Non-Goals

**Goals:** produce comparable, sanitized measurements tied to the actual installed
target; establish what evidence supports and what remains unresolved. Preserve
enough timing provenance to distinguish setup from same-session requests.

**Non-Goals:** product fixes, availability certification, workload isolation by
stopping other clients, profiler attachment, new dependencies, record mutation,
service restarts or tests. No UI, provider or realtime-state changes.

## Decisions

1. Use one diagnostic change and five sequential tasks: identity, collector,
   observations, diagnosis, independent acceptance. Identity and collector
   failures lead to explicit skipped observations and an inconclusive report;
   they never justify guessed credentials/configuration or a passing runtime claim.
2. Prepare a dependency-free collector only within the KBD child during Execute.
   Inspect existing runtime/CLI capabilities and protocol syntax first. Do not
   import application startup or dotenv. Existing tools are preferred to new
   SDKs because installation would expand scope and change the environment.
3. Use the confirmed existing WebSocket identity and namespace/database. Time
   socket-open, signin and selection independently. Query sequentially on that
   same connection: two scalar returns, ID-only reads limited to1and32rows with
   TIMEOUT5s, then a scalar return. This separates external client setup from
   later requests; it does not reproduce UAR's reused client or full-table workload.
4. Use at most two health/session/readiness rounds, only if the first has no
   stop condition. Follow all per-request, response-size and total caps in the
   plan and spec. No retries or expanding deadlines. Existing readiness itself
   still reads all skills; stop active probes after its first timeout rather
   than potentially queue more work behind an uncancelled server operation.
5. Keep raw response/log data in bounded memory only. Retain normalized timings,
   counters and category names. Never print request authentication, response
   tokens, record IDs or skill/embedding content. Avoid full environment and
   process-argument inspection output; extract only the necessary approved fields.
6. Use three hypothesis dispositions: supported, contradicted in this sample,
   unresolved. Require counterevidence and limitations. Inconclusive is a valid
   reviewed deliverable, not a claim that causal phase goals or readiness passed.

## Risks / Trade-offs

- Shared-resource impact → one active request, tight caps, passive snapshots and
  stop-on-first-error; read-only does not mean zero load.
- Correlated data mistaken for causal proof → distinguish timing scopes and retain
  competing explanations; fast limited reads do not prove full-table decoding cost.
- Effective config or protocol cannot be safely resolved → report unsupported
  observations; no guessed defaults, dependency installation or fallback retries.
- Small sample misses intermittency → report sample bounds, not availability SLOs.
- Cross-model review unavailable within inference-free scope → use fresh native
  artifact-only context and label the same-model limitation honestly.

## Migration Plan

No deployment, migration or rollback is required: no running-service or product
state changes are authorized. Keep diagnostic artifacts and append-only history.
Register the new change/tasks through canonical KBD commands, leaving all tasks
pending. During Execute, run kbd-status at every completed task/change/phase.
After task5 and Execute acceptance, enter Reflect separately before phase closure.
OpenSpec archive/sync and existing Presentation archive approval remain separate.

## Open Questions

Which latency component dominates, whether this sample reproduces the symptom,
and whether safe existing observations can separate application internals remain
diagnostic questions, not preselected conclusions. A future remediation budget
requires operator approval and is separate from the observation caps.
