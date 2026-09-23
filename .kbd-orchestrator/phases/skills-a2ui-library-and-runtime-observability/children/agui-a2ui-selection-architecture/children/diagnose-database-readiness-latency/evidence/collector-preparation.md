# Collector preparation — task1.2

Recorded: 2026-09-05T14:25:36Z. Stage: Execute, preparation only.
Collector: ../collector.mjs.
SHA-256: 5dbbe09b51e705681d4d9c4be0da8315500df2172095ed094c14888a9dd0f467.

## Result and limits

The dependency-free Node collector is source-reviewed and passes syntax checking.
Only its passive --prepare mode ran. No HTTP/database probe, authentication,
query, product test, application import, dotenv load or new dependency occurred.
No observations.json exists. This is not proof that the live transport works.

The uncomfortable limit: this deliberately restricted WebSocket client accepts
only uncompressed, unfragmented text frames. An unexpected control, fragmented
or binary frame ends the experiment as unsupported; no fallback or retry is
allowed. Thus an inconclusive collection can reflect collector incompatibility,
not a database fault. Syntax and source review cannot remove that uncertainty.

## Verified capabilities and protocol sources

- Node v26.5.0 on darwin imported only node:fs, node:http, node:net, node:crypto,
  node:child_process, node:util, node:url and node:path. node --help documents
  --check as syntax checking without execution. No package install or SDK import.
- Context7 resolved /surrealdb/docs.surrealdb.com. Its
  [RPC contract](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/reference/rest-api/rpc-protocol.mdx)
  supplies root signin user/pass, use(namespace,database), query(statement,vars),
  and result/status/time response fields. Signin bodies/tokens are discarded.
- Pinned [3.2.4 formats](https://github.com/surrealdb/surrealdb/blob/v3.2.4/surrealdb/core/src/rpc/format/mod.rs)
  and [RPC route](https://github.com/surrealdb/surrealdb/blob/v3.2.4/surrealdb/server/src/ntw/rpc.rs)
  confirm json negotiation on /rpc. The handshake must actually agree before
  signin; the collector does not infer success from this source inspection.
- [RFC6455](https://www.rfc-editor.org/rfc/rfc6455.html) supplies handshake
  verification, client masking and frame-length fields. Oversized announced
  payloads are rejected before reading their bodies; response cap256KiB.
- The pinned [database health handler](https://github.com/surrealdb/surrealdb/blob/v3.2.4/surrealdb/server/src/ntw/health.rs)
  returns an empty success body. Local src/server.rs:2420 and2484 supply the
  expected UAR status fields ok and ready. Non-200 responses retain HTTP status
  and observed byte count but no body text. Failures stop the whole collection.
- Local iostat manual confirms -Id -c1 produces one cumulative-since-boot
  sample. These are not instantaneous I/O rates. vm_stat and numeric ps fields
  provide passive snapshots; no process arguments are retained.
- [XNU pressure conversion](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_memorystatus_notify.c)
  and [dispatch constants](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/event_private.h)
  map sysctl kern.memorystatus_vm_pressure_level values1/2/4 to normal/warning/
  critical. An earlier preparation read returned2; the final --prepare read
  returned1. Neither observation establishes a causal relationship with UAR.
  Unknown or non-normal pressure prevents collection. No pressure-generating
  memory_pressure command was run; its manual was inspected only.

The instructed Firecrawl developer lookup was attempted but the installed CLI
returned exit1, unknown command developer. Official documentation and pinned
upstream source supplied the protocol facts instead; no upstream defect is claimed.

## Guard-to-contract mapping

| Observed failure or boundary | Collector mechanism |
|---|---|
| Prior30s readiness timeouts/shared-resource risk | One sequential operation;5s health,35s readiness,15s setup,10s query,120s whole-session timer,360s collection timer; no retry |
| Inherited credential exposure incident | Never import UAR/dotenv, parse only reviewed literal application credentials, no raw errors/arguments/environment/logs, discard tokens and IDs |
| Stale installed/configuration identity | Refresh matching hashes/PIDs/start times/listeners and relevant override checks before operations; executable digests streamed at start; compare again after each round |
| Host pressure observed during preparation | Passive pressure checks before operations and at round boundaries; non-normal/unknown states stop |
| Untrusted response size/shape |256KiB response bound,16KiB handshake/header bound; validate frame/RPC/statement shapes before accepting results |
| Accidental repeated collection | Exclusive-create observations.json is a permanent attempt marker; an existing receipt is never overwritten by a new run |

All guards are confined to the collector. No production guard or fix was added.
The sequence is exactly database health, UAR liveness, one direct session with
signin/use and five fixed read queries, readiness, liveness; at most two rounds.
The second round requires successful first-round completion without a stop.
No readiness cancellation guarantee is implied by client close. Identity and
pressure are sampled at boundaries, not continuously monitored by a polling loop.

The five fixed queries are RETURN true twice, SELECT id FROM skills LIMIT1
TIMEOUT5s, SELECT id FROM skills LIMIT32 TIMEOUT5s, then RETURN true. Retained
results are only booleans/row counts. LIMIT bounds results, not all engine work.
Each existing log is read once at finalization, at most256KiB, and reduced to
category counts and matching timestamps. Historical log entries stay historical.
Evidence is capped at1MiB. Local command execution is asynchronous so it does
not block request/session deadline timers. Timer scheduling is not hard realtime.

## Verification observed

Commands below were run from the repository root; CHILD denotes this diagnostic
child directory, not an arbitrary replacement or an executable shell variable.

- node --check CHILD/collector.mjs: exit0, no output after each edit.
- git diff --check: exit0, no output after each edit.
- node CHILD/collector.mjs --prepare: exit0; mode passive_preparation, Nodev26.5.0,
  platformdarwin, builtinModulesOnlytrue, pressure1, activeRequests0, exact five
  planned queries, collectorExecutionNotValidatedtrue.
- test ! -e CHILD/evidence/observations.json: exit0.
- shasum -a256 CHILD/collector.mjs: digest above.
- Isolated artifact-critic source review: final no remaining findings. Four
  earlier warnings were corrected; receipt in ../review/collector-source.json.

These are Tier0 syntax/record checks and passive capability reads, not product
tests or task2.1 evidence. Live compatibility, deadlines under real transport,
response outcomes, current readiness and root cause remain unverified.

## Next task

Task2.1 may invoke node CHILD/collector.mjs --collect once, with its working
directory at the repository root. It must first inspect this receipt, the
collector, plan.md and identity.json, and confirm no observations.json exists.
The script refreshes identity and pressure itself. Do not delete a failed or
partial receipt, enlarge limits, restart services or rerun to obtain a pass.
Record attempted/skipped comparisons and proceed to an honest diagnosis even
if a prerequisite prevents all active requests. No additional authority is
requested for fixes, inference, tests, deployment, commit/push or archive.
