# ASSESSMENT: diagnose-database-readiness-latency

Project: Universal Agent Runtime
Date: 2026-09-05
Stage: Assess — fact-finding only, not an execution plan.
Codebase baseline: HEAD 226d4a0af89811975662cf3f203c699f70e8cebc; no working-tree changes in src/server.rs or src/uar/persistence/providers/surreal.rs when inspected.
Cross-tool progress: child has zero defined changes/tasks; parent Presentation has 3/3 accepted changes and 12/12 accepted tasks. Overall implementation remains 112/120, not a coverage percentage.

## Evidence boundaries

The latency predates this release. The September 5 deployment receipt records
pre-install UAR readiness and database-health timeouts, intermittent readiness
after installation, and continued failure after the approved shared-database
restart. These are historical observations, not fresh probes this stage.

After the last recorded restart, UAR liveness returned HTTP 200 in 0.021005s;
readiness exceeded 15s and a separate probe returned HTTP 408 in 30.022509s.
Authenticated WebSocket scalar queries completed in 5705ms and 9109ms; another
startup query exceeded 15s. Those CLI wall times combine process startup,
connection, authentication and query work. They do not isolate query execution.
The scalar-query receipt does not establish that the same namespace/database
was selected as the application, or that a reused application connection behaves
the same way.

The installed binary digest recorded in docs/releases/local-install-2026-09-05.md
is 2e3d3ab62920d43e1c26b78fb77e1cd46c80c9c4e1cfc97d369c73be9e46cd4c.
Installed identity, service PIDs, endpoint/namespace/database, effective timeout
and current operational state must be reconfirmed before experiments. No new
health/database requests, service restarts, builds or product tests were run
for this assessment. Memory recall returned an unreachable-endpoint stub; local
gotchas and deployment/session receipts supply prior context.

## Implementation status

- DONE — source-level readiness path exists. /healthz and /readyz are registered
  at src/server.rs:1408. The readiness handler is at src/server.rs:2484.
- PARTIAL — dependency-aware readiness. The handler awaits persistence.list_skills(),
  then checks memory-service presence and reads the MCP tool count. Only a
  returned persistence error sets all_ok false. Missing optional services are
  reported not_configured without failing readiness.
- PARTIAL — lightweight persistence probe. In the Surreal adapter, list_skills
  (src/uar/persistence/providers/surreal.rs:845) calls fetch_all("skills").
  fetch_all at line 341 issues SELECT * FROM skills with no LIMIT, takes rows,
  converts values to JSON, and list_skills deserializes every SkillRecord,
  including embeddings. The handler's "limit 0" comment is contradicted by
  these statements. Full-row materialization is confirmed; its latency
  contribution on this installation is not measured.
- PARTIAL — timeout diagnostics. src/server.rs:1774 uses the configured request
  timeout (source default 30000ms in src/config.rs:1269). /readyz is not exempt
  in should_apply_request_timeout at line 2539. The wrapper returns HTTP 408
  on deadline expiration, without per-dependency results. This is a source
  explanation consistent with the recorded 30s response, not proof of where
  that request spent its time.
- PARTIAL — dependency identity/freshness. Persistence success is always labeled
  postgres even when the configured adapter is Surreal. The surrealdb check
  tests only memory_service presence; it performs no fresh connectivity query.
  MCP tool-count collection is not proof that remote MCP servers are healthy.
- DONE — probe authentication exclusion in the inspected source:
  src/uar/security/middleware.rs:95 bypasses auth for both probe paths.
  src/uar/security/rate_limit.rs limits API/protocol prefixes, not /readyz.
  These source observations lower auth/rate-limiting priority; they do not
  certify the installed request path or rule out all middleware waits.
- PARTIAL — startup attribution. TCP bind precedes persistence initialization
  (src/server.rs:492 versus 612); serving follows later at line 1920.
  SurrealDbProvider::new connects, signs in, selects namespace/database and
  runs three migration queries before reporting success (surreal.rs:32).
  A listener or an initialization log alone does not prove completed startup.
- MISSING — correlated timing evidence separating connection/authentication,
  server execution, response transfer/decoding, application scheduling and
  synchronous MCP registry access. Generic request timing is aggregate only.
- MISSING — evidence-backed root cause and justified remediation proposal.

## Spec gap summary

openspec/specs/deep-health-probes/spec.md requires active PostgreSQL, Redis and
SurrealDB connectivity checks, all failures reported, and unauthenticated probes.
The actual handler is configured-adapter-based, has no Redis probe, uses a
misleading postgres label, and substitutes memory-service existence for a
SurrealDB connectivity check. A request deadline can replace the specified
structured 503 with 408. These are documented contract differences, not
authorization to add dependencies, change the endpoint contract or implement fixes.

The spec provides no numeric readiness-latency objective. The observed 30s
deadline is not an acceptable-latency target. A later remediation proposal needs
an explicit operational budget; this diagnostic assessment does not invent one.

openspec/specs/native-service-deployment/spec.md requires observed readiness
after restart and preserved operator configuration/data. The deployment receipt
does not establish stable readiness, so release availability remains unverified.
Its broad inference/restart certification requirements are not authorized work
for this diagnosis-only child.

## Diagnostic gaps for the next stage

These are discriminating questions, not an executable probe sequence.

| Candidate explanation | Evidence for consideration | What remains unknown |
|---|---|---|
| Full skill-row transfer/conversion dominates readiness | SELECT * and full decoding are in the source | Actual row count, payload size and time contribution; small-row reads might also be slow |
| Connection/authentication setup dominates standalone probes | Recorded CLI elapsed times include setup | Separate timing versus a reused authenticated connection in the application namespace |
| Shared database/host resource contention | Database health also timed out before installation; scalar probes were slow | Correlated CPU, memory, I/O and competing-client activity; no cause established |
| Application-side waiting or scheduling | Readiness awaits persistence, then synchronous registry work | Same-window direct database comparison and safe attribution on the existing process |
| Startup-only latency | Initialization took minutes after restart | Repeated post-start failures weaken a startup-only account; present steady-state behavior remains unmeasured |

The next stage must define bounded, sequential, read-only observations, sanitized
outputs, timestamps and stop conditions. Read-only queries still consume shared
resources; do not launch an unbounded SELECT * merely to reproduce this handler.
Do not print skill contents, embeddings, credentials, full environment, plist
arguments or unfiltered logs. Confirm effective adapter/namespace/database and
installed identity without dumping secrets. Prefer existing evidence; stop and
report uncertainty if discriminating evidence requires new instrumentation,
restart, configuration changes or database writes. No upstream engine defect
is asserted: version-specific behavior claims require authoritative retrieval.

## Build health and test coverage

- Build check: UNKNOWN for this stage — not rerun. Historical release build
  PASS is recorded in docs/releases/local-install-2026-09-05.md (47m41s);
  historical frontend build PASS includes four retained PGlite eval warnings.
- Test coverage: PARTIAL for probe behavior; numeric coverage UNKNOWN.
  tests/production_readiness_tests.rs:14 checks a constructed JSON value, not
  the handler. tests/integration/live/capability_cases.rs:245 exercises probes,
  but accepts either 200 or 503 and does not establish a readiness-latency budget
  or installed shared-database behavior.
- Existing accepted Presentation suites remain accepted, not rerun or rebranded
  as evidence for this fault. Preserve deferred 429/coverage, live-peer/billing,
  zoom/contrast, credential-rotation and dependency-alert warnings.
- No Rust files were edited; no Cargo tier was triggered. Assessment artifact
  verification is Tier 0 whitespace/JSON validation plus isolated review.

## Constraint check

No source/configuration/dependency/database/service mutations or product test
suites are part of Assess. Scope permits child artifacts and append-only learning;
canonical lifecycle commands own generated KBD projections. constraints.md is
absent; AGENTS.md, versions.toml and child scope.json govern. No new product
guards were added. Parent archive approval, cancelled release rows and the paused
native goal are unchanged. Workspace-info tool was not available in discovery.

Workflow deviation: the first tool read the waypoint and skill, not the position
reminder; the reminder was read immediately afterward. No position was inferred.
The inherited global evidence/certification/publication COMPLETE fields refer to
older work and do not certify this new child.

## Goal progress

1. PARTIAL — historical reproduction is documented; bounded current-state
   characterization remains outstanding.
2. PARTIAL — source path is traced; measured attribution and installed-source
   identity confirmation remain outstanding.
3. NOT MET — candidates are separated, but none has been experimentally
   confirmed or ruled out; a bounded diagnostic plan is still required.
4. NOT MET — no root cause or remediation recommendation is justified yet.

The uncomfortable finding: even replacing the full-table probe might only hide
a broader database/host problem, because independent scalar probes were also
slow and readiness failed before this release. Neither "the database is slow"
nor "SELECT * is the cause" is a supported diagnosis.

ASSESSMENT COMPLETE
