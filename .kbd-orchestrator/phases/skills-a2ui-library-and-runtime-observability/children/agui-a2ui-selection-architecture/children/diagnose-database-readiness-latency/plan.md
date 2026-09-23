# PLAN: diagnose-database-readiness-latency

Project: Universal Agent Runtime
Date: 2026-09-05
OpenSpec available: YES (installed CLI 1.10.0; AGENTS.md's 1.5.0 statement is stale).
Changes to implement: 1 diagnostic deliverable, 5 sequential tasks; no product fix.
Baseline: assessment.md at HEAD226d4a0a; overall112/120 before registering this new
change. Registration adds one denominator entry, not a reopened completed change.

## Scope and decisions

The user approved openspec/changes/diagnose-database-readiness-latency/** for
diagnostic records. Child scope.json now includes that exact path. No permission
to change product code, dependencies, configuration, database records or running
services was added. Plan does not run live probes. Execute requires a subsequent
user request. No product test suites or inference requests in this child.

Use one OpenSpec change, diagnose-database-readiness-latency, with a new narrow
capability readiness-latency-diagnostics. Its delta specifies diagnostic evidence
and safety, not a new readiness response contract. Do not silently fix the
postgres label, presence-only SurrealDB check, full-table read or missing Redis
check. Do not sync/archive specs or change the paused native goal.

Optional Analyze was explicitly skipped. No library-candidates.json or linked
evolution plan exists for this child; older evolver registry entries are unrelated.
No library adoption, new dependencies or evolver bridge is needed.

## Ordered change list

1. diagnose-database-readiness-latency — identify the supported explanation for
   persistent installed readiness latency, or deliver a precisely bounded
   inconclusive diagnosis and the evidence/authority needed to resolve it.
   - Scope: diagnostic artifacts; source inspection and bounded operational reads.
   - Depends on: accepted Assess handoff; no other implementation changes.
   - Recommended agent: Codex, Astra as requested.
   - Est. complexity: M.
   - Complexity score: High — application/persistence/shared-host attribution
     crosses boundaries and has unresolved causal hypotheses.
   - Model class: frontier. project.json has no model_policy; fallback retained.
   - Customer value: HIGH — avoid another unsupported recovery attempt.
   - Details: establish trustworthy identity and collect a small correlated
     sample. Distinguish connection/authentication cost, database query reports,
     client round trips and application-side waiting without claiming causation
     from a timeout alone.

## Task order and acceptance

| Task | Depends on | Deliverable and exit criteria |
|---|---|---|
| 1. Establish diagnostic identity | None | evidence/identity.json and execution.md record UTC, source HEAD, installed executable digest, current UAR/DB PIDs and start times, listener identities, nonsecret effective adapter/endpoint/namespace/database and timeout provenance. Compare deployment receipt. If identity/config cannot be resolved safely, record the exact gap and stop active probing; do not assume file defaults are effective settings. |
| 2. Prepare bounded collector | 1 | A dependency-free diagnostic helper under this child only, with source-reviewed query allowlist, deadline/size limits and sanitized outputs; syntax check passes. Inspect local CLI/runtime capabilities and documented protocol before use. Never import/run UAR startup or load repo dotenv. If existing tools cannot supply safe stage timings, record unsupported observations rather than add dependencies or alter UAR. |
| 3. Collect observations | 1–2 | evidence/observations.json plus execution.md contain attempted/skipped operations and reasons, exact sanitized commands, timestamps, elapsed times, outcomes and limits below. Unavailable identity/collector yields explicitly skipped probes, not fabricated timings. No automatic retry or service intervention. |
| 4. Attribute and recommend | 3 | diagnosis.md maps every hypothesis to supporting/disconfirming evidence and one of supported, contradicted in this sample, or unresolved. Separate confirmed source defects from causes. Give the smallest evidence-justified remediation proposal, or the next required evidence/authority; do not implement it. Explain counterevidence and operational risks. |
| 5. Review and phase acceptance | 4 | Fresh-context artifact-only critic checks diagnosis and evidence. Resolve critical findings or explicitly retain an unresolved/blocked disposition; no false root-cause acceptance. Produce verification.md and handoff-out.md with evidence bounds, remaining warnings and operator decision. Accept the diagnostic change only when its evidence contract passes; readiness recovery is not an exit criterion or an implied result. |

After Execute and its task/change acceptance complete, enter Reflect separately
and write reflection.md with the plan/delivery delta before phase closure.
Run kbd-status after each task/change/phase completion. No parallel operational
tasks; only the independent artifact review can overlap non-operational bookkeeping.
If task1/2 cannot safely enable probes, later tasks can document the limitation;
their exit criteria require explicit unavailable/skipped evidence, not a claimed
reproduction. Do not mark the phase's original causal goals MET if unresolved.

## Observation contract (Execute only)

Read-only operations still consume shared resources. Use one collector, at most
one active request, no polling loop, maximum two rounds and 360 seconds total
active observation wall time. Stop earlier whenever a gate below triggers.
This is an experiment budget, not an availability SLO.

1. Capture passive process CPU/RSS/elapsed-time and host memory/I/O snapshots
   before/after the round. Use numeric allowlisted fields, never process argument
   lists or environment dumps. Read at most 256KiB from each relevant existing
   log tail into memory; retain only timestamps, named startup/error categories
   and counts. No raw log copies, profiler attachment or log-level changes.
2. Each round: database HTTP health (5s cap), UAR /healthz (5s cap), one direct
   authenticated database session (120s whole-session cap), UAR /readyz (35s cap),
   UAR /healthz (5s cap). Maximum two database-health, four liveness and two
   readiness requests total. Run round2 only if round1 completed without any
   timeout, transport/auth error, identity drift or observed degradation.
3. Direct session: use the confirmed existing WebSocket endpoint/identity and
   application namespace/database. Measure socket open, signin and use separately
   (15s each); suppress signin/token payloads. Then issue, sequentially on that
   same session, RETURN true twice; SELECT id FROM skills LIMIT 1 TIMEOUT 5s;
   SELECT id FROM skills LIMIT 32 TIMEOUT 5s; RETURN true once more.
   Each RPC has a10s client deadline. Thus at most five read queries per session,
   ten total, two signins and two namespace/database selections. Session selection
   and authentication are protocol operations, not application-record mutations.
   If identity is not root, use its documented existing authentication form;
   do not elevate or guess. If correct auth/protocol cannot be confirmed, stop.
4. Record monotonic client round-trip time, response-byte count, RPC status and
   server-reported statement time when supplied. These are different clocks and
   scopes; do not subtract them and label the difference network/queue/decoding
   time without corroboration. Parse only small allowed payloads. Retain scalar
   booleans and row counts, never record IDs, skill text, embeddings or tokens.
   No COUNT scan, full-row query, EXPLAIN scan or full-table diagnostic query.
   LIMIT bounds returned rows, not a guarantee of bounded engine work.
5. Each response buffer is capped at256KiB; each sanitized evidence file at1MiB.
   Unexpected format, size, RPC/statement failure or deadline ends active probes.
   Closing a client is not proof that its server operation was cancelled.
   In particular /readyz still invokes the existing full-table read: the first
   timeout ends the whole active collection; do not stack another query behind it.
   Continue only passive snapshots and reporting. Do not kill/restart services.
6. An unresponsive dependency, process disappearance, changed PID/config identity,
   newly observed pressure affecting the host, or operator interruption ends
   active collection immediately. Record which comparisons were prevented.
   Do not re-run a failed round automatically, enlarge caps, create another
   database/namespace or attempt credential repair.

Before Execute, validate syntax against installed capabilities without connecting.
The verified documentation establishes JSON RPC query/use shapes, response
status/time/result fields, SELECT LIMIT and TIMEOUT clauses; it does not establish
the cause or guarantee cancellation/latency of pinned3.2.4 on this installation.
Verify the precise codec/auth form before the first request. If it differs, keep
the same operation/size/deadline bounds or report unsupported; no fallback retries.

## Interpretation and feasibility

- Slow signin/socket-open with small subsequent same-session scalar round trips
  supports setup overhead for the external client, not necessarily UAR's reused
  client. A fast external session does not certify UAR connection health.
- Slow scalar round trips before skill reads weaken a full-table-only account.
  Server-reported time may narrow the explanation but is not a whole-host trace.
- Fast bounded ID queries with slow readiness do not prove decoding cost:
  full-table transfer, application client contention, scheduling and registry
  access remain confounded. No full-table size or total-row claim from32rows.
- Resource counters are correlations, not proof that another client caused it.
  Do not stop other clients to manufacture an isolated result.
- Short successful rounds show only sampled behavior, never sustained readiness.
  Old post-start failures weaken a startup-only theory; current startup/steady
  state must be labeled from identity and existing startup evidence.
- Without safe internal timings, full causal attribution may be impossible.
  The uncomfortable outcome is a reviewed inconclusive report with a narrowly
  scoped instrumentation proposal. No fabricated root cause to make a task green.

No numeric service-latency target is established by the canonical spec. A future
fix proposal must name a proposed operator-approved budget separately from these
probe caps and include phase-end tests for that behavior; no such tests run here.

## Verification, records and next action

Tier0: git diff --check per artifact edit; JSON parse/packet equality; syntax-only
check of any collector when later authored. OpenSpec validation checks the new
delta and tasks, not operational behavior. Independent plan review occurs before
change structures are emitted. Independent diagnostic review occurs at task5,
after observations and analysis, with no product tests at intermediate tasks.
No build, install, deployment, product suite or release certification.

OpenSpec artifacts: proposal.md, design.md, specs/readiness-latency-diagnostics/spec.md,
tasks.md. All five tasks start unchecked. Canonical KBD register commands own
change/task/progress records. New change registration should yield112/121 overall
and0/1 for this child; confirm from runtime, never hand-edit counts.

Next after Plan is explicitly requested Execute for this one diagnostic change.
Do not archive the three Presentation changes or reinstate cancelled release
gates. Retain429/coverage, credential-rotation, dependency-alert, PGlite, peer/
billing and zoom/contrast warnings. Keep append-only .prometheus history.

## Documentation consulted

- [RPC query/use and statement response contract](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/rest-api/rpc-protocol.mdx)
- [SELECT LIMIT/TIMEOUT syntax](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/query-language/statements/select.mdx)

Retrieved through Context7 on2026-09-05. No upstream defect attribution.
