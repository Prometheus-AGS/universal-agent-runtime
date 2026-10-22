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
