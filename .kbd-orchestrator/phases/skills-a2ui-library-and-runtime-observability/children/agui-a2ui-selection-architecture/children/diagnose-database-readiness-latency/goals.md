# Goals — Diagnose persistent database-backed readiness latency

Stage: ready for Assess. Scope: diagnosis only, requested with `/kbd-new-child`.

1. Reproduce and characterize the installed UAR readiness latency with a small,
   bounded set of read-only observations. Distinguish startup from steady state,
   liveness from readiness, and database health from authenticated query service.
2. Trace the actual readiness request through the installed source, middleware,
   persistence adapter and database operations. Identify where time is spent
   using correlated, credential-redacted evidence rather than attribution from
   a timeout or process status alone.
3. Confirm or rule out concrete hypotheses using a written assessment and plan.
   Separate connection/authentication delay, query execution/data volume, host
   resource contention and application-side waiting where the evidence allows.
4. Deliver an evidence-backed diagnosis and the smallest justified remediation
   recommendation, including uncertainty, impact and a proposed verification
   gate. Do not implement fixes or perform additional restarts in this child.

The uncomfortable constraint: restarting both services did not recover stable
readiness. A passing scalar query or HTTP liveness probe is not proof that the
database-backed readiness operation is healthy. The cause remains unknown.

No inference calls, credential output, database writes, destructive actions,
configuration changes, dependency updates or product test suites are authorized
by this diagnostic scope. Preserve the paused native goal and existing phase
acceptance/cancellation decisions. Run kbd-status at task/change/phase boundaries.
