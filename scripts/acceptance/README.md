# User-outcome acceptance

> Historical implementation retained for reference. The operator replaced this packet-based process with one-at-a-time manual BDD scripts in `scripts/behaviors/`. Its former gate and credential setup requirements are not instructions for the active manual sequence. Consult `.prometheus/verification/macos-behavior-ledger.md` for completed behaviors.

Source authored for `reengineer-user-outcome-verification` task2.1. Verification
is CLOSED. No invocation, import, provider call, observer, build, or runtime
acceptance has been executed. The example is deliberately nonexecutable. Its
limits are illustrative and grant no resource or gate authority.

`user_outcomes.py` is a standard-library serial runner. Each positive case runs
three times with fresh random facts on **both** Postgres+pgvector and disk-backed
Surreal. A successful protocol terminal and a useful-result predicate are separate
requirements. FAIL, BLOCKED and NOT_RUN remain distinct. The first failure stops;
there is no retry. Explicit case subsets are labeled as subsets and never claim
phase acceptance. The full manifest must contain all25 outcome families; missing
or unimplemented cases block before starting a service.

## Source modules

- `core.py`: frozen Case/Predicate/Outcome and Response types; bounded HTTP,
  owned processes, observation commands, redaction, immutable receipt files.
- `cases_prompt.py`: U01 configured genuine prompt and U02 warm/cold continuation.
- `user_outcomes.py`: fixture admission, case instantiation, serial execution,
  first-failure retention, awaited cleanup, and final manifest.
- `fixtures/fixture.example.json`: shape only. Exact scripts, input bytes,
  credentials, stores, ports, source/artifact identity and allowance must be
  instantiated and frozen by the integration owner before a gate is proposed.
- `fixtures/scenarios.json`: literal ordered source manifest of97 cases and582
  planned rows (three repetitions on two adapters). It imports or executes
  nothing. The executable fixture must preserve this denominator; a new case
  requires its source definition and manifest entry. These counts are planned,
  not observed PASS results.

No process starts on import. `--execute` is an execution-intent switch, not a
substitute for operator approval or signed packet adoption. A future exact
approved command names `--fixture`, a new exclusive `--evidence` directory and
`--execute`; optional repeated `--case` selects explicit frozen IDs. Do not run
that command while the gate is CLOSED.

## Module API

A local `cases_*.py` module exports `cases() -> list[Case]` and imports shared
primitives from `core`. `Case(case_id, family, run, repetitions=3, durable=False)`
contains an implemented `run(ctx) -> Outcome`. Every case has three repetitions;
`durable` records case semantics and is not automatic restart evidence. Outcomes
contain `protocol: Predicate(passed, detail)`, `useful: Predicate(passed, detail)`
and `facts: dict`. Both predicates must be exactly true. Return neither a stub
PASS nor a green skip. `require(condition, message)` raises Failed;
`Blocked(message)` denotes a missing prerequisite. An uncaught exception fails
the packet and still attempts owned cleanup.

`Context` exposes `fixture`, `backend`, `run_id`, `case_id`, `attempt`, `evidence`;
`request(method, path, payload=None, headers=None) -> Response`;
`record(name, value)`; `observe(name, values=None) -> dict`; `restart()`;
`owned_process(argv, env=None)`, `wait_process(process, seconds)` and
`stop_process(process)`. Response has `status`, lowercase `headers`, `body` and
`json()`. New modules and every external executable/script/config input must
appear in the frozen file inventory. Use smallest owned actions and source-backed
mounted schemas. A case must derive its expected value independently of output.

`cases_prompt.configure_provider(ctx, binding=None)` reads the mounted provider
registration, creates it only if absent, verifies exact configured identity,
and calls the actual provider `/test` endpoint to obtain the required live
model-admission proof. This is a genuine inference call and must be accounted
in the future packet. It does not overwrite a differing existing registration.
`prompt(ctx, text, session_id=None, agent_id='default-agent', model=None,
send_model=True, expected_model=None)` sends the mounted nonstreaming completion
schema. It checks completion terminal, native run inspection, actual effective
agent/model/revision and settled provider accounting. It additionally requires a
run-bound genuine endpoint observation; a fixture's `kind: genuine` label alone
cannot certify inference. The returned object contains `content`, `run_id`,
`session_id`, `inspection`, `accounting`, `provenance`. Callers must grade content.
`answer_json(result)` parses a strict JSON object. `send_model=False` permits
saved-default/session precedence cases; set `expected_model` to the declared
binding expected to have actually executed.

## Fixture admission and observer contract

Executable fixtures require schema1, frozen source/artifact/config/schema
identities, both durable backend profiles, finite positive wall/request/command/
cleanup/HTTP/output allowances, three repetitions, explicit modules/case IDs and
absolute regular frozen input files with SHA256. Command argv never uses an
implicit shell. Every executable and existing absolute file argument is frozen.
Credentials live in named environment references: `credential_env`, `headers_env`
and `environment_env`. No inline api_key or environment values are admitted.
The runner refuses redirects and off-origin HTTP paths. The server is owned by
this invocation and must use a fresh isolated fixture, never an operator DB.

Backend `observers[name]` is an exact argv list for an **implemented**, frozen
observation/fixture program. Each invocation receives `UAR_ACCEPTANCE_CONTEXT`
JSON with `run_id` (packet ownership UUID), `backend`, `case_id`, `attempt`, and
`values`. It inherits the backend's referenced environment. It must finish within
the command allowance and emit a bounded JSON object to stdout; nonzero exits
fail. Missing hooks block. Hooks must collect actual production/store receipts,
not echo requested values as observed facts. Observer source is part of the
phase implementation inventory and independent review. The required baseline is:

| Hook | Input `values` | Required observed output |
| --- | --- | --- |
| prepare | profile_id | run_id, owned=true after exclusive run-owned store/namespace/disk/port setup |
| ready | empty | ready=true only after bounded actual owned-server readiness |
| admission | profile_id | run_id, backend, profile_id, durable=true, empty_owned_store=true, source_identity, artifact_identity, configuration_revision, schema_revision, runtime_epoch, store_identity; Postgres pgvector=true; Surreal ephemeral=false and disk_path |
| inference | run_id (native inference ID), session_id, qualified_model | same exact inference/session/model IDs, kind=genuine, synthetic=false, actual base_url, configuration_digest, nonempty provider_receipt from real dispatch evidence |
| session | session_id, phase=before_restart/after_restart | session_id, backend, durable=true, canonical_messages in exact persisted turn order, actual runtime_epoch |
| cleanup | processes_joined=true | packet run_id, residuals=[] only after deleting exact-owned fixture resources and confirming absence |

The packet run_id in the envelope differs from a native run_id in inference
values. U02 requires the postrestart canonical message array to retain the entire
prerestart prefix and append turns, and requires different actual runtime epochs.
`restart()` first stops and waits the exact server process group, starts the same
server argv with the same durable profile, and awaits readiness. Reconstructing
a handle in one process cannot satisfy this mechanism. Foreign-owner continuation
and durable history consumers are covered by their separately required families;
U02 does not infer those from its private-fact result.

## Evidence and cleanup

Each receipt is created exclusively, fsynced, chmod400, hashed and chained in the
final chmod400 manifest; the directory becomes chmod500 after sealing. This is
local tamper evidence, not a cryptographic operator signature or externally
immutable store. Fixture admission failures retain BLOCKED evidence without
starting any service. Request/response JSON is redacted recursively; known
referenced credentials, authorization/cookie/key fields and private reasoning
fields are removed. Opaque non-JSON bodies retain byte identity only: their
underlying cause remains unrecovered. No hidden chain-of-thought is retained.
Raw child stdout/stderr stays in private temporary files, is bounded during finite
command waits, and only redacted receipts persist. All created processes are
joined before fixture cleanup; process cleanup failures block completion.

The uncomfortable constraint is that a green HTTP response and self-reported
model name can come from a fake endpoint. Accounting is necessary but also does
not prove the upstream is genuine. The actual inference observer, fixture
ownership and immutable source binding are therefore mandatory. A malicious or
incorrect observer can still lie; its source must be reviewed and its actual
receipts accepted under the later exact gate. This source authoring makes no
runtime, durability, backend parity, customer build, or statistical SLO claim.

The integration owner may declare `startup_initial_capture` to take the actual
preseed snapshot after prepare and before the first server starts. Its output
remains a normal immutable receipt; startup cases grade the persisted snapshot.
Server observers receive `UAR_ACCEPTANCE_SERVER_STDOUT_PATH` and
`UAR_ACCEPTANCE_SERVER_STDERR_PATH`, naming private mode600 temporary files for
this exact owned process. They change on cold restart; raw files are removed
when closed. Use bounded reads and source-defined markers, not invented flags.
Structured SSE response bodies retain bounded event/data objects and DONE,
including typed errors and selected-agent/results metadata. Explicit reasoning
events/fields are withheld; malformed blocks retain source identity and an
`evidence_complete=false` marker. Cleanup has its explicitly bounded allowance
after the useful-work wall limit, so exhaustion still permits owned teardown.

## Shared packet processes

`packet_processes.py` owns the one genuine budget forwarder above backend
contexts. Its exact frozen argv, private configuration, environment references,
epoch and listener ports are mandatory. Readiness uses its actual child stdout
and matching private ready file, without an additional network probe. The daemon
survives both backend cleanups and every cold restart. Shutdown requires the
joined successful terminal ledger receipt; a forced stop or blocked/unknown
ledger prevents acceptance. The private reservation ledger survives cleanup.

Both server settlement and the iOS runner use the pure
`budget_proxy.terminal_receipt_error` admission validator. Known completed/failed
HTTP receipts must match epoch/input/binding/kind, contain finite positive ordered
reservation/start/finish/settlement timestamps and a SHA-256 response digest, and
stay within actual reserved response bytes. Completed model usage must fit the
actual output reservation; completed embeddings require actual input/total usage.
Known failed HTTP retains its full reservation without requiring success usage.
The server additionally retains its read-only transaction, owner/source/config
binding and cumulative arithmetic checks. Settlement waits at most90seconds and
does not dispatch, refund, reset accounting or establish useful-success evidence.
This source-authored behavior remains UNVERIFIED under the CLOSED operator gate.

Finite observer processes release their private temporary output files after
their actual process-group join. Hundreds of case observations must not retain
two open file descriptors each until phase cleanup. A failed backend cleanup is
recorded once and stops the packet; it is not automatically retried before the
next backend. These behaviors are source-authored and remain unexecuted.


Owned datastore startup and cleanup use concrete lifecycle receipts. Before any
Postgres schema creation, the runtime observer requires the private
`postgres-fixture-prepared.json` handshake with the exact frozen native
`runtime.postgres_store.config_path` byte hash, run, scope, ownership nonce,
immutable container ID, four ownership labels, retained mount, port and actual
SQL/pgvector readiness. A configured lifecycle exit status alone is insufficient.
Cleanup always attempts the owned lifecycle after process joining, even when
prepare, readiness, Store construction or forensic capture failed. The PG helper
returns a matching `postgres-fixture-cleaned.json` with actual exact-name absence
evidence; a claim without a created container may safely produce absence, while
an unclaimed existing container is never removed.

Surreal preparation receives a private `UAR_ACCEPTANCE_SURREAL_CID_FILE` plus
`UAR_ACCEPTANCE_OWNED_LABEL`, `UAR_ACCEPTANCE_OWNED_RUN_LABEL`,
`UAR_ACCEPTANCE_OWNED_NONCE_LABEL` and `UAR_ACCEPTANCE_OWNED_BACKEND_LABEL`.
Cleanup exports `UAR_ACCEPTANCE_SURREAL_CONTAINER_ID` only after inspecting the
actual immutable ID, exact run/scope/nonce/backend labels, pinned image, daemon
arguments, writable owned mount and loopback port. Removal targets that ID.
Actual bounded name/label/CID listings establish absence, including creation
failure before a container exists; listing errors or diagnostics never mean
absence. No lifecycle operation described here has been executed during source
authoring.

If forensic capture or its durable retention fails, metadata says unavailable
with null counts/jobs and a bounded content-free failure digest. Cleanup retains
private state and disk for investigation while still attempting exact-owned
container teardown. `active_residuals=[]` means that container absence was
actually established; `residuals` remains nonempty when private state is retained,
so this partial cleanup cannot pass the acceptance packet. Full private-directory
removal requires saved genuine capture, confirmed schema/database removal and
confirmed container absence. The uncomfortable constraint is that unavailable
forensics prevents complete cleanup acceptance even when owned services stop.
