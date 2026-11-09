# Runtime measurement and comparison scope

The selected promotion lane is Linux amd64 container APIs behind the trusted host/gateway, SurrealDB default, PostgreSQL 18 only within its documented capability restrictions. Current-source capacity, reliability distributions and comparative model quality are **unverified**. The existing operator contract excludes arbitrary throughput/SLO/comparative claims and Rust in-process embedding certification from this first promotion. This disposition does not remove any concrete source defect in #462–475.

## Selected promises and retained evidence

Comparison scope is **NONE**. The operator selected the synthetic #472 request-path baseline at concurrency 1 and 4, eight requests per round and three repetitions per concurrency, for 48 requests in total. The sole acceptance threshold is a 0% observed error rate. Latency, throughput and resource observations carry no selected threshold or capacity promise. The selected experiment remains unverified until the frozen manifest, installed artifact and actual receipt are available.

`benches/hot_path.rs` measures deterministic routing/context/prompt helpers. Its July 4 table is historical helper-overhead evidence on its recorded source; it is not a packaged-runtime capacity result. `evals/routing-accuracy.yaml` and the context suite exercise deterministic helpers. Existing eval scorers and baselines remain useful: deterministic checks are primary, and any LLM judge is advisory with named provider/model/prompt/temperature. A controlled provider can establish framework overhead and protocol wiring; only genuine output through the supported packaged runtime can establish model task success.

Use the existing Cucumber measurement feature for the selected baseline. Existing recovery, interruption, history and resource-owner scenarios supply separate reliability evidence within their recorded inputs. Any receipt carried forward to a different source or artifact needs an explicit applicability rationale.

## Rig and equivalence disclosure

Rig is explicitly **not compared** in this lane. The reason is that comparative promotion is not selected, no equivalent owner-selected task/model/tool/budget/profile exists, and no real-model spend is authorized. This is an exclusion of the comparison as a whole, not a finding that Rig has inferior performance or weaker correctness. No Rig version is claimed because no version-pinned comparator was selected. The resulting evidence cannot support UAR superiority over Rig or any other implementation.

If a comparison is later selected, it must pin Rig or explicitly give a technically reasoned Rig-specific exclusion, and pin every other comparator, adapter/glue source and configuration. Include glue overhead and identify what the runtime supplies natively versus what the host or measurement tool supplies. Equivalent request-path overhead and durable recovery workloads are separate comparisons. For each profile retain admission/persistence acknowledgement, checkpoint/resume, event replay, effect retry/idempotency, unknown/in-doubt handling and operational responsibilities. Weaker durability or host-supplied mechanisms narrow the claim; they cannot silently become a speed/reliability advantage.

## Required future receipt contract

Before any measurement execution, record the selected representative workload, hardware/OS/resource profile, target concurrency, acceptance thresholds and spend boundary in the canonical phase plan. Freeze source, artifact digest, store/configuration, provider/model versions, prompts, tools, input data, sampling and input/output/reasoning token budgets. Runtime/provider dependency changes must not quietly change the experiment.

Retain cold/warm startup, first-response and complete-run latency, p50/p95/p99 with sample counts, throughput, errors/cancels, CPU/RSS and queue/backpressure observations at declared concurrency. Expose provider time separately from runtime overhead. A bounded reliability run includes long conversation/tool loop, interrupted stream, backend outage/recovery, controlled restart and resource return in the supported lane. Report loss, duplicate-effect and unknown outcomes rather than only successful averages. Reuse stable run/attempt/call/receipt identities without adding a second accounting ledger.

Raw outcomes, repeated-run variation, failed, blocked, negative and inconclusive results remain part of retained evidence. A provider stub cannot satisfy a real-model prerequisite. Equivalent model/task/tool/budget conditions and deterministic task checks are required before comparative-quality claims. External publication, paid model calls and dedicated infrastructure changes retain their own authorization boundaries.

## Regression policy

A result is bound to its exact source/artifact/configuration/workload. Reassess the affected selected measurements after materially changing request preparation/routing, context strategy/tokenization, retry/stream handling, tool/effect dispatch, durable admission/accounting/recovery, persistence engine/version, provider/model/adapter, scheduler/backpressure, telemetry capture, compiler/dependency/runtime or artifact resource settings. Preserve a passing unaffected gate; do not expand every change into an unbounded benchmark suite. If no measurement was selected, update the disclosure rather than inventing a regression threshold. Historical helper numbers must retain their date/source scope until deliberately replaced with approved measurements.

The parent KBD Execute ledger records the phase-scoped authorization, implementation boundary and serial gate commands. Source authorship and fixture preparation supply no benchmark or acceptance result.

## Source-authored packaged-runtime baseline

A narrow new feature now exists in the current Cucumber facility:
`scripts/behaviors/runtime-measurement.cucumber.mjs` selects
`features/runtime-measurement.feature` and its dedicated steps. It measures the
actual packaged `/v1/chat/completions` streaming route through the external
identity gateway. It does not start a runtime, inspect a process, configure a
provider, authenticate, start a collector, run existing reliability cases or call
an external model itself. No new product endpoint or dependency is introduced.
All source remains unexecuted. The historical live load test remains separate:
its hardcoded concurrency and send-to-headers timing are not adopted here.

The feature requires `UAR_RUNTIME_MEASUREMENT_MANIFEST`, `UAR_BASE_URL` and
`UAR_WORKSPACE_ID`. The checked-in
`fixtures/runtime-measurement/unselected.json` deliberately has null choices and
status unselected. It refuses before any HTTP. `manifest.schema.json` documents
the format; the source validator enforces required selections and freeze
consistency without a new schema-validation dependency. No supplied file is a
runnable benchmark configuration today.

The external owner must freeze these inputs before measurement:

- Selection decision/reference and experiment ID; synthetic-only/zero authorized
  external spend, comparison NONE and technically reasoned Rig exclusion.
- Exact source commit **and whole-source inventory digest**, Linux amd64 image,
  config/settings digests, hardware CPU/kernel/OS/container profile and allocated
  CPU/memory limits, persistence/durability/gateway profile. The external host's
  egress policy must restrict model traffic to the owned synthetic peer. A
  declaration alone does not prove an incorrectly configured host obeyed it.
- Agent artifact/provider-settings hashes, qualified model plus actual leaf model
  ID, provider/model revisions, temperature, prompt-cache setting, empty caller
  tools and explicit retry disclosure. Automatic memory is disabled for this
  deterministic request-path baseline; that does not qualify memory behavior.
- Synthetic task IDs, one user-input text per task, deterministic expected text,
  explicit provider delay and response-chunk text/delays, synthetic reported
  token counts and the canonical workload SHA256. Tasks cycle in declared order;
  there is no model judge, random task selection or hidden prompt default.
- Every concurrency level, requests per round, repetition count, input/output/
  reasoning budgets, maximum experiment/request/observation/framework time, payload/journal
  bounds and maximum request count. This text baseline requires explicitly chosen
  zero reasoning budget. Synthetic reported counts do not prove a real tokenizer
  input bound or real-image capacity. The existing HTTP output cap is sent;
  input budget limitations remain explicit in the receipt.
- Each named threshold is either selected with `lte`/`gte` and an owner-supplied
  numeric value, or expressly unselected with a reason. Unknown/omitted dimensions
  refuse; at least one numeric threshold must be selected. There is no default
  SLO or minimum benchmark volume masquerading as owner selection.
- Exact external observer revision, clock definition/skew, sample cadence,
  startup cold/warm definitions and preexisting observer/peer journal paths.
  Reliability selection for each existing case also must be explicit.

The externally launched `support/runtime-measurement-peer.mjs` consumes the same
frozen manifest via `UAR_RUNTIME_MEASUREMENT_MANIFEST`. Its explicit `PORT` and
optional `UAR_RUNTIME_MEASUREMENT_BIND` belong to external provisioning. Loopback
is default; all-interface binding is solely for an externally isolated container.
The peer sends the selected deterministic chunks and journals actual receive,
first semantic send, response completion/connection close, request/input/output
hashes, offered tool count and synthetic reported usage. It never logs headers or
prompt bodies, runs a real model or changes UAR settings. It accepts only the
selected marker-bearing measurement requests, not a general verification prompt.
Registry verification/setup is an external prerequisite; this feature does not
fabricate a verified provider.

## Timing, distributions and raw observations

Every request retains a fresh stable native intent/session, request/output hashes,
HTTP status, raw frame arrival times/types/hashes, first nonempty semantic text,
terminal `[DONE]` time, finish metadata, complete read time and actual native
terminal/cancellation availability. Headers, role-only metadata and empty text
are not semantic first response. A truncated stream, wrong deterministic answer
or streamed error is an error sample, never a successful low-latency completion.

Each concurrency/repetition round retains all attempted rows, client peak in
flight and initial sender backlog, wall interval, success throughput, observed
error/cancel rates and nearest-rank p50/p95/p99 **with sample counts**. The three
latency quantiles use successful semantic completions; failed rows and their
elapsed times remain raw evidence. A small sample's p99 may equal its maximum;
it is not a tail guarantee. Per-repetition distributions retain variability and
negative outcomes rather than pooling them into one attractive number. Read-only
native reconciliation runs after the timed round so it does not insert hidden
per-request lookups into the throughput interval. Client abort does not become
native cancellation without actual terminal evidence. Unknown live work stops
later rounds; requesting cancellation is not release proof.

A unique peer request/response interval supplies measured synthetic-provider
service time, including the selected peer pacing. End-to-end minus that interval
is labeled **gateway/network/runtime/serialization residual**, never pure
framework overhead. Missing/multiple/incomplete peer intervals are unavailable.
The manifest and peer request/output hashes bind the exact deterministic work;
they do not establish genuine model task success or tokenization correctness.

Observer input is append-only bounded JSONL. Every record binds `experimentId`,
exact `identity` (including sourceInventoryDigest) and `observerRevision`. Startup
records pair `startup_start` / `startup_ready` for the same phase/runtimeEpoch and
SHA256 of the owner's definition. Actual monotonic nanoseconds plus observed
`/readyz` HTTP200 determine elapsed time. Cold/warm meaning is the owner's stated
process/cache/store condition, not an inferred OS-cache flush. All pairs and their
distributions are retained; the selected `coldStartupMs`/`warmStartupMs` value is
the maximum observed matching pair, not an invented default. The runner never
launches, restarts or polls readiness itself.

`resource_sample` rows carry `timeUnixMs`, `runtimeEpoch` and independent
`{availability,value,source}` objects:

| Field | Required measured basis | Interpretation |
|---|---|---|
| `cpuUsageNs` | `process_cpu_cumulative_nanoseconds` | Delta CPU time / sampled wall interval in cores used; not a single-core percentage |
| `rssBytes` | `process_rss_bytes` | Sampled process RSS peak; container memory.current is not relabeled RSS |
| `runtimeQueueDepth` | `runtime_owned_queue_observation` | Actual runtime queue; client sender backlog is kept separately |
| `runtimeBackpressureTotal` | `runtime_owned_backpressure_counter` | Counter delta; arbitrary HTTP503 is not assumed queue pressure |

Per-round resource windows require bracketing samples from one runtime epoch,
within the selected clock skew/cadence, with no counter reset. Edge intervals are
sampled bounds, not exact instantaneous CPU/RSS. Missing observer data, unsupported
runtime queue instrumentation, counter resets or incompatible source semantics
are explicit unavailable values. No product CPU/queue API was invented to fill a
gap. Actual HTTP429/503 counts are additionally retained with their narrower
meaning. A selected threshold requiring unavailable evidence is BLOCKED, not PASS.

## Reusing bounded reliability evidence

The only eligible producer is the existing local Linux artifact workflow,
`scripts/behaviors/features/production-artifact-qualification.feature`, with a
fresh `uar.production-artifact-qualification/v1` receipt. The consumer requires
`status: PASSED`, `authentication_supplied_by_test: false`, no cleanup errors,
full exact identity, matching source-feature hash and real before/after candidate
snapshots. It consumes the producer's original normalized facts:

- Cancellation: actual closed/non-ended provider observations, original cancelled
  run/call/terminal records and successful fresh follow-up.
- Tool loop: completed independent MCP request/invocation/effect records and
  identical provider/effect/accounting arrays across retained-event readback.
- Bounded conversation: sequential original input/run IDs, canonical persisted
  message IDs and hashes present in the actual later provider request. This is
  deterministic retention wiring; genuine summary quality remains unavailable.
- Interrupted stream: actual partial provider close, error terminal/calls, no
  successful terminal completion and successful same-profile follow-up.
- Controlled restart: inspected prior/stopped/new-start state, successful exit,
  retained volume/image and identical original persisted canary.
- Optional externally staged remote-Surreal outage: actual readiness503→200,
  ordered transport loss/recovery, same UAR process/container/start/configuration,
  retained canary and subsequent actual write. Embedded storage does not qualify
  this configuration-specific promise.

The shared `candidate-artifact-binding.mjs` compares actual Linux container/image
projections, trusted-host sealed launch-config/source-inventory digest labels and
nine live effective nonsecret settings. It reads bounded referenced artifacts;
it never calls Docker/Git/APIs or reads Env/full configuration. External gateway
ownership supplies settings authority. Tests carry no credentials. Configuration
identity is the external host's sealed digest assertion, not an independent
secret configuration dump. Legacy Mac/native/C50/C51/summary/telemetry receipts
without these real producer bindings remain historical and unavailable; no PASS
wrapper upgrades them. No selected case establishes all-timings reliability,
global exactly-once effects, process release, RSS recovery or genuine model quality.

Custom results are append-only under
`.prometheus/verification/runtime-measurement/`, with full frozen selection,
per-request and per-round observations, journal/receipt hashes, availability,
threshold findings and negative/inconclusive outcomes. No current result exists
from this source authoring. The feature's configuration/peer/manifest/observer
contracts still require the final approved combined source/artifact gate; no
import, build, test or measurement has run while the gate is closed.

Client framing/hash work and synthetic-peer journal/serialization work are part
of the measured setup. No subtraction claims to remove all measurement overhead.
The required owner `frameworkTimeoutMs` registers Cucumber's step budget from the
same manifest; its exact bytes must remain unchanged before HTTP. This framework
budget is distinct from the chosen experiment duration and product thresholds.

## Clean release prerequisites and actual observer source

Current dirty implementation snapshots are **not eligible**. Final qualification
requires a clean committed root and recursively checked-out clean submodules,
normal existing release provenance (`schema_version: 1`, `input_boundary:
release`, real `source_sha`/`source_tree`), and the actual successful
`linux-image-build.json` (`exit_code: 0`, approved command equality,
`source_identity`, matching `image_id`/`image_source_id`). Dot-only
`local_candidate` provenance is refused. The existing release provenance process
and checklist retain release authority; no workflow or generator was changed.

After final commit, a fresh inventory must write to ignored `target/` or outside
the checkout. Do not regenerate a tracked .kbd snapshot after commit and then
silently qualify a dirty tree. New bounded source producer:
`scripts/behaviors/support/candidate-artifact-inventory.mjs`. A later command,
**not authorized or run by this document**, is:

```text
node /Users/gqadonis/.claude/worktrees/production-readiness-476/scripts/behaviors/support/candidate-artifact-inventory.mjs /Users/gqadonis/.claude/worktrees/production-readiness-476 /Users/gqadonis/.claude/worktrees/production-readiness-476/target/release-source-inventory.json /ABSOLUTE/OWNER-SELECTED-inventory-limits.json
```

The limits JSON requires positive `maxFileBytes`, `maxTotalBytes`, `maxEntries`,
`gitOutputBytes`, `gitTimeoutMs`. Budgets include both source captures. The output
parent must already exist. The file is create-only. Two complete byte captures
plus actual committed blob OIDs and clean Git identity must agree; regular blobs,
symlink link bytes and recursively checked-out gitlinks have distinct semantics.
Non-ASCII source paths, external/dangling symlinks, missing recursive checkouts or
special inputs are rejected instead of being assigned misleading file hashes.
The output retains the existing source-inventory identity schema and the existing
Linux builder's sorted regular-file preimage fields. This is source identity
metadata, never build/runtime acceptance. The fresh `identity.head` must equal
normal release provenance `source_sha`; current implementation inventories cannot
be reused as final release evidence.

Binding selection kind `uar.candidate-artifact-binding-inputs/v1` requires
`inventory`, `inputProvenance`, `imageBuild` references each `{path,sha256}`,
`baseUrl`, `workspace`, `runtimeConfigDigest`, complete `expectedSettings` and
`maxInputBytes`. Whitelist keys: `server.log_format`, `memory.inject_context`,
`memory.injection_min_similarity`, `llm.model`, `llm.timeout_secs`,
`llm.max_retries`, `llm.parallel_tool_calls`, `llm.cost_tracking`, `llm.tracing`.
Snapshot identity includes canonical selected-settings digest. External launch
labels must include `dev.prometheus.qualification.config-sha256` and
`dev.prometheus.qualification.source-inventory-sha256`, alongside existing owned
case/base-url/workspace and source revision labels.

`runtime-measurement-observer.mjs` is the bounded external producer source. Its
later CLI takes absolute selected manifest and absolute observer selection paths.
The observer selection requires kind
`uar.runtime-measurement-observer-selection/v1`, `mode` (`resources`, `cold` or
`warm`), owned `container`, `volume`, `caseId`, `bindingInputsPath` and positive
`durationMs`, `sampleIntervalMs`, `commandTimeoutMs`, `httpTimeoutMs`,
`maxCommandBytes`, `maxSamples`, `maxJournalBytes`. Startup additionally requires
`priorLiveReceipt: {path,sha256}` pointing to a fresh successful artifact receipt.
All are owner selections, with no supplied workload/duration/SLO defaults.
`manifest.observer.sourceRevision` must be the actual observer file SHA256.

It uses the existing projected Docker inspect/image/volume and Docker-exec Python
pattern. Procfs locates exactly one `/usr/local/bin/universal-agent-runtime`
executable, reads CPU ticks and VmRSS with process-start fencing, converts actual
clock ticks to cumulative nanoseconds and KiB to bytes. This is **process RSS**;
container total RSS remains unavailable. Queue/backpressure always remain
unavailable because this source has no actual internal observer for them.
Observer sampling/Docker/procfs work perturbs the experiment; no subtraction
claims otherwise. Missing executable/fields are unavailable, not zero.

Startup requires an actual witnessed stopped instance with prior live settings,
then an externally operated new start and a positive `/readyz`200. It retains
monotonic stopped-boundary and ready observations and actual old/new snapshots.
Elapsed time includes external scheduling delay after the witnessed boundary;
it never derives startup duration from a single current Created/StartedAt value.
Cold/warm cache/store conditions must be externally established and explicitly
stated; a label alone does not prove cache eviction. Source/build/config/settings
references are read before/after; append-only journal data is eligible only with
a matching successful `observation_boundary` record. No observer is running and
no measurements, imports, process/API/Docker probes or checks were executed.
