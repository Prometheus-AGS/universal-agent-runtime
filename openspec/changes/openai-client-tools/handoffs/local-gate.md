# Local gate supervisor source handoff — task 12.1

Phase Execute. Verification CLOSED. This handoff records source authoring only.
No supervisor, fixture, acceptance program, import, compiler, check, build,
readiness probe, network request or external gate was executed.

## Owned artifact

`scripts/release/client_tools_local_gate.py` is one stdlib supervisor. It uses the
already built binary at
`target/uar-post-lifecycle-main/debug/universal-agent-runtime`; the lead separately
reported build003 compile/link PASS. This script does not build or certify that
binary. Existing `.gitignore` line 2 excludes `target/`; no ignore rule changed.

Exact proposed gate command, **not run**, from the repository root:

```bash
python3 scripts/release/client_tools_local_gate.py --evidence target/client-tools-local-gate-001
```

That evidence directory must not exist. The default binary is used. The operator
must explicitly authorize this exact gate after inventory/review; naming it here
does not open the gate.

## All authored effects inside that one command

1. Create a fresh evidence directory under repository `target/`, mode 0700,
   with private temporary/model/store directories. Never reuse existing state.
2. Create fresh test-only HS256 issuer secret and two JWTs (admin provisioning
   subject and distinct second subject, same isolated tenant). Config,
   credentials, child logs and JSON reports are mode 0600. Credential bytes are
   never command arguments or printed. A fresh settings admin key satisfies the
   existing default protected-settings requirement; authentication is not waived.
3. Start the real provider/MCP HTTP peer on loopback port 0 and retain its
   synthetic request/effect trace. Its actual listener port comes from its owned
   port file and its HTTP control endpoint is probed inside this gate.
4. Write explicit source-supported config: authenticated loopback HTTP and gRPC,
   fresh embedded SurrealKV file, fixture provider/model, ACP enabled with auth,
   disabled memory/native file/web/terminal/session-search tools. OpenAI embedding
   client points at the local fixture with a test-only key, avoiding local-model
   asset downloads; no embedding calls are part of the matrix. Config is JSON
   valid as YAML in a `.yaml` file, using the existing YAML loader.
5. Use an allowlisted child environment. No inherited UAR/LLM/provider credential,
   CONFIG_FILE, MCP_CONFIG_PATH, proxy, encryption key or launch authority enters
   the children. Preserve only PATH/HOME/language/system runtime fields; set an
   isolated TMPDIR and source-supported UAR_LOCAL_ONLY=true. Candidate gets its
   exact generated --config and MCP_CONFIG_PATH. No user/global config is edited.
6. Configure one real HTTP MCP peer with only its safe raw `effect` tool
   administrator-preapproved. This applies solely to the fresh isolated fixture
   configuration. No governance setting, auth policy or unrelated tool is changed.
7. Start the prebuilt candidate with isolated working directory and retain logs.
   Poll authenticated `/readyz`, which includes the real persistence operation.
   Read actual `/api/tools`; require exactly the configured MCP server's `effect`
   record and use its returned `namespaced_name` without guessing a namespace.
8. POST a complete persisted AgentArtifact to `/api/agents` using the generated
   admin-subject JWT. Its provider/model is the fixture and its host tool allowance
   contains only that actual advertised name. Save exact agent/catalog receipts.
   Confirm unauthenticated `/api/tools` returns 401.
9. Run the existing acceptance matrix subprocess over **all five interfaces**:
   OpenAI, Anthropic, AG-UI, official A2A 1.0, ACP. Pass distinct credential env
   variable names, separate OpenAI agent selector / Anthropic provider-qualified
   model, actual MCP tool arguments and report path. This exercises real provider
   HTTP and durable storage, concurrent duplicate submits, held-response socket
   disconnect/retry, incomplete upstream streams, input/result forgery cases,
   multi-round history, choices/parallel controls, and real mixed MCP effects.
10. Bound the matrix process (default 2700 seconds). On normal return, failure,
    timeout, Ctrl-C or SIGTERM, stop owned children in reverse order and join each.
    Candidate receives SIGTERM and has 45 seconds; a forced group kill is recorded
    as cleanup failure, followed by a bounded join. The peer is stopped only after
    candidate stop/join so its transport obligations can drain. Logs are closed
    after children join. Retain lifecycle PIDs, commands without secrets, exit
    results, forced-kill/clean-shutdown flags and local-gate.json.

No arbitrary shell command, Docker invocation, workflow, dependency install,
binary build, global configuration mutation or production credential is used.
All health/setup/authorization probes are **inside** the proposed gate, not
standalone implied preflights.

## Exit status and remaining gates

- **1**: startup/provisioning/matrix failure, missing expected receipt, unclean
  shutdown, or forced cleanup. Retained logs/reports name the observed boundary.
- **2**: every observed matrix case passed and owned cleanup was clean, but full
  product acceptance still needs separate explicit cold-restart, graph-cursor and
  cumulative-budget gates. It is deliberately not full acceptance or phase DONE.
- This bounded supervisor does not return 0 or silently promote matrix success
  into acceptance of those additional gates.

The existing acceptance program already authors restart-seed/resume, graph and
cold-graph, budget and cold-budget modes. This supervisor does **not** provision
those graph/budget agents or execute lifecycle restart modes in this pass. They
remain honest external gate inputs needing a concrete separately authorized
supervisor extension or operator lifecycle. Database backend-matrix coverage and
binary/container provenance are also outside this one local-file matrix gate.
The source-grounded recipe has no known unbound config/API identifier; actual
startup, compile/runtime compatibility, JWT admission, MCP connection, agent
execution and all five protocol results remain unverified until execution.

## Source bindings used

Read `.agent-team/uar-core/client-tools-acceptance-bindings.md`, existing release
JWT/config/AgentArtifact helpers, `src/config.rs`, provider registry, MCP config
and descriptor/approval rules, discovery tools/agent handlers, A2A admission,
SurrealKV durability and embedding backend construction. These were source
inspections, not verification gates. Only the new supervisor and this handoff
were authored in task 12.1; no production source changed.

## Source repair 12.2 — cleanup interruption and private HOME

The isolated source reviewer identified two concrete cleanup failures: one child
signal/join error aborted remaining cleanup/log closure, and an interrupt raised
inside finally could abandon both later children and the terminal report.

stop_join now attempts termination and bounded graceful/final joins per child,
records errors and actual joined status independently, and continues through every
owned child and log. A kill attempt is distinct from a delivered kill and from a
confirmed join; an unjoined child records null exit code and cleanup failure.
Log-close failures are retained without skipping other log handles. No partial or
forced cleanup is promoted to clean success.

SIGINT/SIGTERM handlers now record requests without raising from a cleanup frame.
Startup HTTP operations, child launches, readiness waits and the bounded matrix
poll loop honor that request and enter finally. Cleanup/report writing therefore
cannot be abandoned by a repeated handled signal; received signals force a
non-success outcome. Unexpected OS failure still appears in the retained cleanup
receipt rather than being treated as joined.

The same review identified real operator-home skill discovery. Children now
receive a fresh private evidence/home as their per-process HOME instead of the
operator HOME; the supervisor's own/global HOME is unchanged. The isolated home
directory is mode 0700 and is retained with the other gate evidence.

Only the supervisor and this append-only handoff changed. Exact proposed command
is unchanged. No script, import, test, compiler, format check, probe, service,
network call or review gate was run during this repair. Verification stays CLOSED.

## Gate001 observed failure and bounded source repair

Lead reports gate001 exited before readiness because native UAR attempted the
hardcoded `/data/ingest` directory. No interface case ran. Both owned children
joined (candidate exit 1, peer exit -15); no forced kill occurred. That is a
startup failure with confirmed physical release, not an unjoined cleanup.

Runtime writer confirmed `server.ingest_directory` is a PathBuf serialized as a
YAML string, defaulting to `/data/ingest` for existing container behavior. The
supervisor now creates a private empty evidence/ingest directory and configures
its absolute path under that exact key. It creates no global `/data`, disables
no ingestion behavior, and preserves the real watcher lifecycle.

Cleanup receipts now separately record cleanup_confirmed, exit_status_expected,
clean_shutdown and actual exit_code. owned_shutdown_clean reports physical
bounded cleanup only; owned_exit_statuses_expected records process outcomes.
An already-exited candidate with code 1 can therefore report confirmed cleanup
while retaining the original startup failure. A matrix success followed by a
nonzero candidate exit remains a failure (failed_child_exit), and forced,
unjoined, or uncertain cleanup remains failing. Existing startup/matrix failure
status and its original diagnostic are preserved rather than relabeled solely
because the process exited nonzero.

Only scripts/release/client_tools_local_gate.py and this handoff changed. No
runtime, build, import, formatting, check, startup or network probe was run by
this writer. The lead owns the explicitly authorized test/fix/retest loop and
selection of a fresh next evidence directory; gate001 evidence is retained.

## Gate002 shipped-policy asset setup repair

Lead reports gate002 progressed beyond ingestion but refused governed startup:
its isolated working directory lacked the mandatory relative `policies` tree.
No interface matrix ran. Source confirms server.rs loads `policies` relative to
CWD; GovernanceEngine reads sorted top-level `.cedar` files; Dockerfile explicitly
ships the repository policies tree into its runtime working directory.

The supervisor now copies the existing ROOT/policies tree into evidence/policies
before starting either child. It requires at least one copied `.cedar` file and
records each actual copied policy path and SHA-256 in local-gate.json. It authors
no policy, adds no permit, and changes no governance/auth setting. The isolated
CWD/HOME/credentials remain unchanged. Copy or mandatory-asset failure is a gate
failure with retained diagnostics.

Only the supervisor and this handoff changed. No production code, build, script,
import, check, readiness probe, service or network command was executed by this
writer. Lead owns the authorized gate003 invocation and its fresh evidence path.

## Gate003 actual model verification and matrix limiter setup

Lead reports gate003 reached 95 cases, all failed, and the provider trace stayed
empty: first OpenAI requests returned model_not_verified, then the rapid failure
matrix exhausted the global request limiter and returned 429. An enabled catalog
model is not its required persisted verification proof.

Source binds the real protected administration route to
POST /api/providers/{id}/test (also mounted under /api/uar/providers), JSON
`{"model":"gpt-4o-mini"}`. It requires the existing X-UAR-Admin-Key boundary,
performs actual provider inference, requires nonempty returned text, fingerprints
current configuration and persists ModelVerification. The supervisor now calls
that endpoint after readiness with its generated admin key, accepts only matching
ok/received_text/provider_id/model_id, and retains the endpoint receipt plus the
actual peer trace. It authors no verification status or bypass. The source
endpoint has two 30-second inference/collection deadlines, so this one setup
request uses a 75-second observer timeout instead of the ordinary 5-second probe.

The existing real peer already handles the verification request: no declared
tools produces nonempty normal JSON/SSE text. No fixture change is needed. After
retaining the verification trace, the supervisor resets the peer control/trace
and requires an empty request trace before matrix setup. Existing per-case driver
control resets remain intact.

Source confirms the global limiter reads resilience.rate_limit_enabled,
requests_per_second and burst_size. Isolated gate config now explicitly keeps
limiting enabled with 200 requests/second and burst 400, recorded in the receipt.
This capacity prevents the acceptance matrix's own fast failure/provisioning burst
from obscuring protocol findings. Production defaults remain unchanged; auth and
governance remain enabled.

Only the supervisor and this handoff changed. No program, import, test, check,
build, startup or network probe ran by this writer. Lead owns the next authorized
run; gate003 receipts remain failure evidence, not protocol acceptance.

## Gate004 exact model selector repair and separate production identity finding

Source inspection of the retained non-secret model-verification receipt shows
actual verification succeeded for fixture/gpt-4o-mini. The saved artifact also
selects that exact provider/model. Main server resolve_requested_model interprets
OpenAI `model` as a provider/model selector, while `agent_id` selects the separate
registered artifact. The supervisor incorrectly supplied the generated agent ID
as --openai-model, causing a proof lookup for fixture/<agent-id>. It now supplies
--openai-model fixture/gpt-4o-mini and retains --agent-id independently. Anthropic
already uses the same provider-qualified selector. No proof status is authored,
invalidated, bypassed or weakened by this change.

A separate production finding was sent to the lead and the assigned Liter owner:
ProviderRegistry::resolve_to_llm_config strips the qualified model to bare
`gpt-4o-mini` when base_url is explicit, while retaining resolved_provider_id.
The observed explicit-control rejection names gpt-4o-mini as the Liter adapter;
its capability check therefore receives a wire-model spelling as provider
identity. This requires the production owner to retain/resolve the trusted
provider/protocol identity; changing the configured provider name or dropping
controls is not a valid supervisor repair. AG-UI's later unavailable-provider
result is consistent with prior provider failures entering registry cooldown,
but this writer does not claim independently verified cooldown causality.

Only the supervisor selector and this handoff changed. Existing fixture already
supports actual model verification, so it was not edited. Gate004 clean candidate
exit remains cleanup evidence, not functional acceptance. No program, import,
check, build, startup, probe or network request ran by this writer.

## Optional extended local gates — source authoring

The supervisor now accepts `--extended`. Default matrix invocation/provisioning
remains unchanged. Extended setup adds a second actual catalog provider `openai`
with the same explicit loopback endpoint/test key, protocol=chat and an empty
model list for source-supported catalog enrichment. It performs that provider's
real protected model verification; no verification flag or host_provider_kind
config key is invented. The original `fixture` provider still exercises the
production custom OpenAI-compatible adapter path repaired by its source owner.

Proposed **unexecuted by this writer** command for the lead's scoped loop:

```bash
python3 scripts/release/client_tools_local_gate.py --extended --evidence target/client-tools-local-gate-005
```

All additional commands below are authored subprocess argv inside this one gate,
using the same already-selected absolute driver path, URLs, private credentials,
all-five protocol list and real advertised host tool as the matrix:

1. Require the complete 95-case matrix report with every case passed. Any matrix
   failure stops the gate before extended setup/actions.
2. Driver `--mode restart-seed --restart-state <evidence>/ordinary-restart-state.json
   --instance-identity candidate:<pid>:generation-1`; save all native pending batches.
3. Candidate-only SIGTERM and actual bounded join; require confirmed cleanup and
   expected zero exit. Keep peer, credential values, config bytes, private HOME,
   MCP registry file, ports and SurrealKV store unchanged. Start candidate-2 with
   the identical binary/config/environment, observe authenticated readiness, then
   driver `--mode restart-resume` with the same state and actual new PID/generation.
4. GET the real `general-purpose` artifact and retain its before receipt. Preserve
   fields while binding provider.default to verified fixture/gpt-4o-mini and
   appending a unique system-prompt worker marker; PUT the whole artifact and
   retain the after receipt. Select actual installed `orchestrator-agent` graph.
5. Driver `--mode graph`, then `--mode graph-restart-seed` with its own state file.
   Pass exact source RouterNode marker
   `Route Rust implementation, correctness, or safety questions to rust-reviewer.`
   and the unique persisted worker marker through the existing before/after flag
   names. The updated driver's expected real sequence is router/worker/worker.
6. Cleanly join candidate-2 and restart candidate-3 identically while retaining
   peer/store. Require the persisted specialist worker marker survived; driver
   `--mode graph-restart-resume` uses its saved state and actual new identity.
7. GET actual protected `/api/providers/openai/models`; require enabled
   gpt-4o-mini with live catalog context_window=128000 and max_output_tokens=16384.
   Fail if those source-bound limits do not resolve/match. Record R1=R2=144384,
   S=5 and L=144386 for the current production catalog-reservation path. Both
   R1<=L and R2<=L hold; S+R2>L establishes cumulative usage rather than a fresh
   request that could never fit. Write explicit provider-wire usage JSON
   `{input_tokens:2,output_tokens:3}` for the driver's existing usage-file flag.
8. POST a dedicated ordinary budget artifact using verified openai/gpt-4o-mini
   and `extensions.budgets.max_tokens_per_session=144386`. Retain its receipt.
   Driver `--mode budget`, then `--mode budget-restart-seed` with the budget agent
   and provider-usage-file flags. OpenAI/Anthropic selectors are explicitly
   openai/gpt-4o-mini; registered agent ID is a separate field.
9. Cleanly join candidate-3, restart candidate-4 with identical configuration and
   store, then driver `--mode budget-restart-resume` with saved state/new identity.
10. Stop/join all remaining owned children and close every log using the existing
    bounded fail-closed cleanup. Previously joined candidate generations are
    retained as exact receipts rather than falsely re-stopped or duplicated.

All eight extended driver mode reports are retained alongside matrix evidence.
The peer stays running across all three actual candidate restarts; only the
existing driver's explicit scenario boundaries reset traces. No post-seed trace
reset or provider mutation is inserted between a seed and its cold resume.
Configuration SHA-256 must stay identical across each candidate restart.

Any extended driver failure is `failed_extended`, exit 1, with completed reports
and failed stage retained; matrix success is never relabeled whole-gate success.
Successful extended local observations still return exit 2 as
extended_passed_remaining_release_gates: broader release/backend-matrix evidence
remains outside this local functional gate. No phase acceptance is claimed.

Only supervisor and handoff changed in this authoring task. No execution, import,
check, formatter, compiler, build, probe or network call ran by this writer.
Budget numeric recipe was coordinated with the assigned source planner and is
also guarded against actual live catalog metadata before any budget case.


Extended graph marker correction: the router marker now matches the actual
RouterNode **system** message, `You are a routing assistant. Respond with exactly
one option from the provided list.` The earlier “Route Rust…” string is user
routing content, not the active system identity used by peer/driver assertions.
Graph transport selectors still pass orchestrator-agent separately from explicit
verified fixture/gpt-4o-mini model selectors; the persisted specialist receives
that exact provider route. No graph ID or synthetic private executor is created.
No execution or check ran for this correction.

## Gate005 actual catalog pricing binding

Assigned budget writer traced accounting_usage_unresolved to the custom fixture
model lacking a catalog pricing identity. Terminal usage without a resolvable
catalog cost intentionally leaves accounting Pending; this is not a persistence
query failure. The supervisor's existing fixture model now declares the supported
`pricing_identity: {provider_id: "openai", model_id: "gpt-4o-mini"}`. The registry
validates that actual catalog identity and propagates pricing separately from
routing. Custom provider ID fixture, loopback wire endpoint, explicit controls,
real provider verification and accounting enforcement remain intact. No rate is
invented and accounting is not disabled. Extended openai model catalog enrichment
remains unchanged.

Only this configuration span and handoff changed. No program, import, check,
build, startup, network probe or gate was executed by this writer. Lead owns the
next gate006 --extended against the unchanged build006 binary.

## Cohort 13.17 optional safety integration

Only scripts/release/client_tools_local_gate.py and this handoff changed.
`--extended` now provisions a separate mutable registered safety agent, initially
allowing the actual advertised local MCP tool, and runs the native safety mode
after the complete 95-case matrix and before any cold restart. It retains the
verified fixture model, exact host-tool JSON, authenticated loopback endpoints,
same store and owned child lifecycle. Default matrix-only behavior is unchanged.

JWT creation accepts an optional tenant with its existing default unchanged.
Extended execution creates a third private JWT with the owner subject and issuer
but a different tenant, writes it only in the existing mode0600 credentials
receipt, and passes it through UAR_CLIENT_TOOLS_GATE_OTHER_TENANT using the
coordinated --other-tenant-bearer-token-env driver flag. No credential is printed.
The dedicated safety artifact receipt is retained as safety-agent.json.

Every extended driver result now requires exit0, execution_complete true,
nonempty all-passed results and zero coverage gaps. Safety is recorded among
completed gates only after that path succeeds. A driver's exit2 or documented
missing coverage stops execution before ordinary cold restart; partial coverage
cannot become acceptance. Driver-owned final safety count and additional binding
cases are being coordinated; this supervisor does not hardcode a guessed count.
The existing proposed invocation adds no new operator command:
`python3 scripts/release/client_tools_local_gate.py --evidence target/client-tools-local-gate-001 --extended`
(use a fresh evidence path for each authorized attempt).

No script/import/check/build/startup/probe/network request ran. Source inspection
and authorized writing only. Driver flag implementation and actual safety results
remain unverified pending the driver's handoff and lead-owned execution.

## Cohort 13.18 cancellation and mixed crash recovery supervisor

Only scripts/release/client_tools_local_gate.py and this handoff changed. Default
matrix behavior remains unchanged. Extended mode order is now: 95-case matrix;
60-case safety; cancellation; ordinary restart seed/clean join/resume; warm graph;
graph restart seed/clean join/resume; warm budget; budget restart seed/clean
join/resume; mixed-crash-seed; intentional candidate-only SIGKILL and OS join;
mixed-crash-resume; final ordinary child shutdown and log closure.

The cancellation and mixed-crash modes use the driver owner's native contracts,
existing verified provider/selected agent/auth/host-tool flags. Mixed seed must
return one successful aggregate observation with seed_ready=true,
pending_batches=5 and mixed_host_effects=5 before any crash. The protected state
must exist; its SHA256 is retained and checked unchanged before recovery. Every
mode still requires exit0, execution_complete, all-passed observations and zero
coverage gaps; missing mode/receipt support fails closed.

Supervisor.crash_candidate accepts only its own candidate child, requires that
it is still running, sends SIGKILL only to that PID, and waits for that exact
process. Its expected_crash_for_recovery receipt records PID, generation, signal,
timestamps, delivered signal, actual return code and join confirmation. Recovery
requires actual -SIGKILL with a successful OS join. The controlled peer stays
alive. Configuration, store, JWTs, peer and protected state are reused without
resetting provider control, trace or MCP effects. The successor receives a new
recorded generation and must pass authenticated readiness before native resume.

The expected crash is explicitly clean_shutdown=false and cleanup_confirmed=false;
it is never called graceful cleanup. Final cleanup accounting accepts this one
separately confirmed expected action, records that owned_shutdown_clean excludes
the expected crash, and still requires every ordinary child to stop/join without
forced kill or cleanup error. Unexpected crash exit, unjoined work, forced final
cleanup or child exit failure keeps the gate failed. Final logs are closed.
This proves only crash recovery after confirmed settled public handoffs, not
arbitrary crash-mid-effect exactly-once behavior.

The existing proposed --extended command contains these actions; no separate
shell kill/start command is introduced. Use a fresh repo-scoped evidence path.
No execution, import, test, check, build, startup or network probe ran. Driver
mode implementation/final report shape remains a coordinated source dependency;
lead owns source review and authorized actual execution.

## Published 1.2.2 image gate feasibility (source-only)

The proposed isolated container layout is source-compatible with the supervisor:
`/workspace/scripts/release/client_tools_local_gate.py`, its sibling acceptance
driver, `/workspace/tests/fixtures/client_tools_provider.py`, and
`/workspace/policies/` form the complete read-only bundle. ROOT is derived from
the supervisor's path, not Git metadata; no repository checkout, manifests,
operator config or private HOME is required. Bind a separate writable target
at `/workspace/target`. Set working directory `/workspace` or pass an absolute
fresh evidence path under `/workspace/target`; otherwise relative --evidence
would resolve beneath the image's default /opt/uar and fail the ROOT constraint.
Pass --binary /usr/local/bin/universal-agent-runtime and --extended to the
Python entrypoint. Native host target/ binaries must not be mounted.

Dockerfile runtime inherits the Ubuntu toolchain stage, including Python3.13
at /usr/local/bin/python3 and the native runtime libraries used by the Linux
server build. It copies the release binary to the proposed path and builds with
server-full,postgres-backend; this gate uses embedded SurrealKV, not an external
PostgreSQL service. Both Python programs and the supervisor import stdlib only.
The required live collaborators are the owned Python peer and candidate on
127.0.0.1, including real HTTP MCP: no host ports or external network are needed.
The image itself remains the canonical published digest, not a rebuilt test
image; digest and platform must be supplied by the lead's publication receipt.
Linux amd64 matches the reported x86_64 daemon.

The writable target must be pre-owned/writable by the exact host numeric UID:GID
used for the container. Bundle parent directories and files must be readable by
that identity. The binary must remain executable. The supervisor creates all
private configs, JWTs, HOME, TMPDIR, ingest files, store, logs and reports there,
with secret files mode0600. No /data or /var/lib/uar permission is required by its
configured candidate. Dockerfile declares a /var/lib/uar volume; the clean child
environment removes its persistence/cache selectors and uses the explicit local
config instead, so it is not the gate database. Disable the image's default
healthcheck for this gate: it targets fixed port1906 while the supervisor uses
an ephemeral candidate listener and owns readiness internally.

Important asset limitation: clean_environment intentionally removes inherited
UAR_STATIC_DIR, UAR_MODELS_DIR and UAR_BUILTIN_SKILLS_DIR. The candidate therefore
does not automatically discover /opt/uar static, model or builtin-skill assets.
The configured embeddings use the local OpenAI peer and memory is disabled;
this caller-tool gate does not require UI static files or downloaded local
embeddings. Skill-pack discovery's source contract logs/skips a missing pack.
The required governance policies are explicitly copied from the minimal bundle
into evidence cwd. Thus this proposed gate tests the immutable published server
binary and its container userspace through the real five-interface path, but
cannot certify shipped UI/model/builtin-skill discovery. Do not claim those
assets were exercised, and do not restore operator/provider environment to
silently broaden the gate. Retain bundle identities/policy hashes with evidence.

No demonstrated extra gate asset or source change is required for this layout.
However actual digest contents, dynamic-link closure, numeric-UID permissions,
container loopback operation and Linux runtime behavior remain unverified:
source inspection cannot establish them. No Docker command, import, test, build,
network request, image inspection or executable probe ran for this feasibility
assessment. Only this additive handoff changed. Lead must obtain authorization
for the concrete digest-bound container gate after native acceptance/publication.

Lead authority reconciliation: the operator subsequently directed local functional tests for all five interfaces, fixes, retesting and republication, and explicitly required the result to work after publication. That instruction covers the necessary digest-bound local container invocation described above. It does not authorize product tests in GitHub Actions or unrelated verification. The exact digest, command, bundle hashes and outcome will be recorded when executed; no additional permission is inferred from this feasibility assessment.
