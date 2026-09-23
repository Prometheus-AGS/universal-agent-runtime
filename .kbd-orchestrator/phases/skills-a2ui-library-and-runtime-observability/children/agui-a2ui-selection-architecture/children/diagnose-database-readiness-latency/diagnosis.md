# DIAGNOSIS: persistent database-backed readiness latency

Project: Universal Agent Runtime
Date: 2026-09-05
Task: Execute 2.2 — attribute and recommend without implementing
Disposition: INCONCLUSIVE — no root cause established and no recovery observed

## Decision

The bounded observation is valid but incomplete. It confirms that the selected
process PIDs matched their recorded baseline throughout one attempt, that database
HTTP health returned200 in2458.477ms, that UAR liveness returned200 in6.861ms,
and that a JSON WebSocket connection plus upgrade/header validation completed in
353.701ms. Contemporaneous configuration and executable digests were not retained,
so their continuity during collection is unverified. The collector then stopped
on an unsupported WebSocket frame during sign-in response handling. It did not
decode authentication, select the namespace/database, issue a query, or call UAR
readiness.

No measured comparison isolates database execution, full-row transfer/decoding,
connection/authentication setup, UAR persistence waiting, MCP registry work, or
host contention. No product remediation is justified from this sample. The
unsupported frame is a collector compatibility boundary, not evidence that
authentication or the database failed. Its opcode, flags, length and payload
were not retained, so this report does not characterize the frame further.

## Evidence boundary

Current evidence is `evidence/identity.json`, `evidence/observations.json` and
`evidence/observation-validation.json`. The receipt covers one attempt from
2026-09-05T15:00:18.191Z to2026-09-05T15:00:25.258Z. Four operations were
attempted and20 were explicitly skipped. Both passive snapshots recorded memory
pressure1; sampled CPU was0% for both processes before the requests and0% UAR /
1.4% database at the final snapshot. These point samples and cumulative I/O
counters cannot establish absence or presence of contention.

The earlier assessment attributes historical timings to
`docs/releases/local-install-2026-09-05.md`: database-health and readiness
timeouts before installation, intermittent readiness after installation, and
continued slow behavior after the authorized shared-database restart. It reports
database health200 in0.564665s, scalar CLI requests in5705ms and9109ms, one
startup request over15seconds, and post-start readiness408 in30.022509s. That
receipt was outside the independent task3.1 critic's allowed artifact set, so
these values are externally asserted and unverified by final review. They provide
context only and cannot decide a disposition. The reported CLI durations also
combine client process, connection, authentication, selection, execution and
decoding rather than a current same-window comparison.

## Hypothesis dispositions

The dispositions apply to causal explanations for persistent readiness latency,
not to whether the named mechanism exists in source.

| Hypothesis family | Disposition | Supporting evidence | Disconfirming or limiting evidence |
|---|---|---|---|
| Full skill-row transfer/conversion dominates readiness | UNRESOLVED | The earlier assessment asserts that `src/server.rs:2491-2492` awaits `list_skills`, `src/uar/persistence/providers/surreal.rs:341-352` executes unbounded `SELECT *`, and lines845-856 deserialize returned records. Those source claims were not independently reverified in task3.1. | No readiness or skill query ran in the current sample. The earlier assessment's slow scalar-query context is also unverified by final review. No row count, response size, server statement time, conversion time or application timing was measured. |
| Connection/authentication setup dominates standalone probes | UNRESOLVED | The earlier assessment's unverified historical CLI timings include setup and report5.705s and9.109s. The current WebSocket path reached a successful JSON upgrade, showing the setup path is measurable. | Current socket plus upgrade/header validation took353.701ms, which does not support socket-open alone as the dominant multi-second cost in this attempt. Authentication completion was not decoded;0.748ms is only time to local frame rejection. Whether UAR reused a client was outside final review. |
| Shared database or host contention dominates | UNRESOLVED | Database HTTP health took2458.477ms while UAR liveness took6.861ms in the same attempt. The earlier assessment's database timeout/scalar history is unverified by final review. Bounded database log tails contain earlier timeout/error categories. | Both current pressure snapshots were1 and sampled process CPU was low. Log categories predate the attempt and contain no retained message context. No instantaneous disk throughput, competing-client activity, server execution timing or repeated current sample exists. Correlation is not causation. |
| Application-side waiting or scheduling dominates | UNRESOLVED | The earlier assessment asserts that readiness serially awaits `persistence.list_skills()`, then reads the MCP registry, under a30-second request deadline; those source claims were not independently reverified. Current UAR liveness was faster than database health. | Current readiness was skipped, so there is no same-window readiness/direct-query comparison. No internal timing separates persistence await, response conversion, executor scheduling or registry access. The direct database session never completed authentication. |
| Startup-only latency explains the failures | UNRESOLVED | The earlier assessment asserts that startup initialization took several minutes and one startup query exceeded15seconds, but its historical receipt was outside final review and is unverified here. | Current process PIDs were long-lived, but current readiness was never called. Database-health latency is not readiness latency. The current evidence therefore neither confirms nor contradicts a startup-only readiness explanation. |

All five causal families remain UNRESOLVED. No hypothesis receives SUPPORTED or
CONTRADICTED root-cause status. Source-path assertions describe possible mechanisms
and contract differences, not their measured contribution on this installation.

## Prior source-trace assertions, separate from causation

The earlier assessment reports the following source findings. The independent
task3.1 critic was not given product source, so it cannot reverify them. Treat them
as externally asserted context, not final-review-confirmed evidence:

- Readiness calls `persistence.list_skills()` despite describing it as a
  lightweight operation with “limit0”; the Surreal adapter actually performs
  `SELECT * FROM skills`, converts every row to JSON and deserializes the records.
- Persistence success is labeled `postgres` even when the configured provider is
  SurrealDB. The separate `surrealdb` check tests memory-service presence rather
  than issuing a fresh connectivity operation.
- The configured source-derived30-second request deadline applies to `/readyz`.
  Its timeout can replace a structured dependency result with HTTP408.
- The MCP registry count is read after the persistence await. Its contribution
  was not measured.

If reverified against the named source lines, these would be implementation and
specification differences. This task does not convert them into a diagnosis or
authority to change the endpoint.

## Smallest justified next evidence and required authority

Do not implement a product fix from this report. The smallest next action is a
new, explicitly authorized diagnostic-only task with this sequence:

1. Review only the child collector and amend it to classify an unexpected frame
   using sanitized FIN/opcode/mask/length metadata, then stop without retaining
   payload bytes. Preserve the existing256KiB cap and all fail-closed behavior.
2. If the classification establishes a specific standards-valid frame form,
   separately review the minimal handling required for only that form. Do not add
   general control-frame or fragmentation support without observed need.
3. Re-run syntax, privacy, deadline and source-digest checks without connecting.
4. Ask the operator for explicit authorization for exactly one new bounded
   collection attempt against the same confirmed target. Preserve one active
   request, first-error stop, no automatic retry, no service intervention and
   all existing query/request caps.
5. If that attempt reaches the planned operations, compare same-session scalar
   and bounded ID-query client/server timings with `/readyz`. If it stops again,
   report the new compatibility boundary instead of expanding the experiment.

This follow-up requires a plan/task amendment and fresh operational authorization
because the existing plan prohibits automatic retry. It does not require product
source, configuration, dependency, database-record or service-lifecycle changes.
If direct queries become fast while readiness remains slow, a later proposal can
seek internal timings around `list_skills` conversion and MCP registry access.
If scalar queries remain slow, investigate the shared database/host path before
changing readiness. Neither branch is selected now.

## Risks and unavailable comparisons

- A second attempt consumes shared database resources even under the caps.
- Supporting more frame forms could introduce collector bugs; isolated source
  review and fail-closed limits are mandatory.
- Calling existing `/readyz` still invokes the unbounded full-row operation. Its
  first timeout or error must stop collection; closing the client does not prove
  server cancellation.
- A successful bounded sample would not establish sustained availability,
  release certification or recovery.
- Database query execution, response transfer/decoding, application scheduling,
  readiness latency and stable post-start behavior remain unmeasured currently.

## Goal state

1. PARTIAL — installed identity and one bounded current attempt are recorded,
   but current readiness was not characterized.
2. PARTIAL — source path and effective installed target are traced, but measured
   latency attribution is absent.
3. NOT MET — all five causal families remain unresolved.
4. NOT MET — no product remediation or recovered readiness is justified. The
   report supplies only the smallest next evidence and required authority.

The task execution record states that no product test, inference request, build,
install, restart, source/configuration/dependency/database/service mutation,
commit, push, archive or spec sync occurred. The reviewed evidence directly shows
zero database queries/readiness calls and no retained product-test result; it does
not provide a complete external command audit, so broader negative claims remain
unverified unless independently checked. Independent artifact-only acceptance
remains Execute task3.1. Reflect must wait for that task and change acceptance.
