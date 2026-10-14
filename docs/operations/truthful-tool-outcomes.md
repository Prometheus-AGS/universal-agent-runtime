# Truthful MCP tool outcomes

MCP response delivery and tool success are separate facts. `isError: true`
inside a successful `tools/call` response is an application failure. UAR keeps
the complete SDK-represented response, including text, images, embedded
resources, `structuredContent`, and `_meta`, in a host-held typed failure.
Diagnostic `Display` and `Debug` exclude that response content.

| Host outcome | Meaning | Admission evidence | Success dependency |
| --- | --- | --- | --- |
| `succeeded` | A definitive successful tool response arrived | `succeeded` | May advance |
| `application_failed` | A definitive response reported `isError: true` | `application_failed` | Blocked |
| `transport_failed` | Admission/readiness failed, or the MCP request queue was already closed before enqueue | `transport_failed` | Blocked |
| `outcome_unknown` | Dispatch began, but no definitive response arrived | `outcome_unknown` | Blocked; reconciliation required |

The uncomfortable thing: a lost response cannot prove that an external effect
did not occur. Even an application error does not prove that a write had no
effect. UAR does not automatically resend the failed invocation. Existing
reconnection can repair a transport for a later authorized call; it cannot
replay the previous call. Unknown work retains its held claim and ends the
current model turn with `TOOL_OUTCOME_UNKNOWN`, before another provider request.
Operator or connector reconciliation must supply independently checked evidence.

The public registry retains `Result<Value>` and the existing successful response
shape. An application error now returns `Err(McpToolFailure)` rather than
successful data. Scoped runtime wrapping retains its source chain. The existing
`McpRuntimeError::ToolFailed { server, tool }` variant remains available with its
original fields; `InvocationFailed { server, tool, source }` is additive.
Downstream exhaustive enum matches must include the new variant.

The orchestrator retains an error envelope such as:

```json
{
  "status": "error",
  "outcome": "application_failed",
  "tool_call_id": "host-correlated-call",
  "tool": "server__tool",
  "provenance": {
    "source": "mcp",
    "terminal_state": "failed",
    "observed_by": "trusted_host"
  },
  "result": {
    "isError": true,
    "content": [{"type": "text", "text": "original server error"}],
    "structuredContent": {"reason": "operation rejected"}
  },
  "message": "MCP application failure"
}
```

`ToolResult.success` and projected `ToolEnd.ok` are false for all three failure
outcomes. Error content passes through existing run secret projection before
canonical persistence and event publication. Display/model output retains its
existing truncation policy; canonical receipt retention limits remain explicit.
“Complete” means the represented, scrubbed receipt, not an unrestricted copy of
the original wire bytes or credentials.

Durable MCP execution uses `CanonicalToolReceipt` with source `Mcp` or `Graph`.
Its optional `outcome` field is the version-1 certainty extension to the existing
receipt schema. It participates in immutable duplicate/conflict comparison.
Writing the same call with different certainty is rejected. Historical receipts
remain byte-for-byte history: absent `outcome` stays absent and cannot be migrated
into invented success. Uncorrelated historical paired-host boolean `Failed`
evidence lacks dispatch certainty and cannot grant new replay authority.

Team execution reads canonical receipts for the run and its admitted descendants
before settlement. A legacy MCP/graph receipt with absent certainty, a legacy
boolean failure without trusted native-only `UarRuntime` admission ownership,
missing confirmation, or unknown effect holds the attempt as
uncertain and blocks dependent tasks. Definitive application/transport failures
cannot be promoted to task success by the model's final prose. Native successes
and ordinary no-tool turns preserve their existing behavior.

The same trusted ownership and receipt certainty rule applies to joined-yield recovery and
agent-instance effect settlement. Explicit application and transport failures
are known terminal outcomes; unknown work remains held. A historical native
failure retains its existing certainty when that exact invocation's admission
ownership identifies the trusted native-only runtime path. Historical paired-host
failures lack a persisted invocation-to-call mapping and remain held when their
certainty cannot be proved. A receipt with the same tool name and run cannot
establish that mapping. This does not change current explicit native outcomes.
The native ownership exception cannot certify an MCP response or grant replay of
an ambiguous external effect.

The separate `connector_effect` catalog plans closed GitHub, Notion, Slack, and
Jira HTTP operations. It currently has no MCP execution caller or MCP effect-ID
bridge. This change qualifies the actual canonical MCP connector receipt path;
it does not reinterpret arbitrary HTTP response JSON, add a new bridge, or expand
those connectors' reconciliation/replay authority.

The host finish API adds `application_failed`, `transport_failed`, and
`outcome_unknown`; the existing boolean `finish` adapter and historical `failed`
value remain supported. A paired host must recognize the new outcomes. Rejection
of an unsupported terminal value remains visible; UAR never retries it as
`succeeded` or downgrades unknown execution into known failure. Rollback code that
cannot read the new fields must not interpret the receipts as success.

## Local production-boundary gate

Run only after the whole slice is frozen and KBD task 2 begins:

```sh
UAR_DOT04_EVIDENCE_ROOT="$PRIVATE_EVIDENCE_DIRECTORY" cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend --test truthful_tool_outcomes -- --test-threads=1 --nocapture
```

`UAR_DOT04_INFRA_FILE` is mandatory for shared qualification. It points to a
private JSON manifest containing `surreal.url`, `surreal.username`, and
`surreal.password` for an operator-owned disposable durable SurrealDB server.
The gate creates a unique namespace and isolated databases. Do not print the
manifest or credentials. No Postgres credentials are needed by this gate.
Set optional `UAR_DOT04_EVIDENCE_ROOT` to an operator-owned private directory to
retain profile receipts, process receipts, helper and binary logs, concurrent
PID/overlap records, and restart artifacts. Each run creates unique private
profile subdirectories; the gate never deletes that supplied root or its retained
contents. Without it, scratch roots are temporary. Raw evidence can contain
synthetic credentials and stays private; sanitize any public acceptance copies.
The profile matches the established serialized server build. No Postgres-specific
storage implementation was changed. The common optional outcome field applies
to canonical receipt serializers; this gate does not exercise a Postgres receipt
round trip. Feature compilation is not Postgres runtime evidence. It does not claim Postgres
observer/checkpoint parity.

The controlled peer uses `/usr/bin/python3` and the Python standard library over
real MCP stdio. Liter uses a loopback HTTP protocol peer. Children clear ambient
environment and run from isolated scratch directories. The gate covers direct
and scoped application failures, success, known predispatch rejection, and a
disconnect after an independently fsynced external effect. It checks actual
orchestrator/admission consumers, `ToolEnd`, immutable canonical receipts,
catalog dependency settlement, and local fresh-process reopen. Direct manager
cases use the in-process authenticated run router. The four scoped orchestrator
cases per profile enter an ordinary UAR binary through authenticated
`POST /api/uar/runs`, with its startup-installed scoped MCP runtime and verified
turn owner. Each uses an isolated durable backend, which the library observer
helper reopens after the owning server and its MCP children exit. These four
changed entrypoints replace the manual scoped fixtures; they do not add scenario
rows. Library writer/observer helpers remain distinct from actual server binaries.
The additional server-process fixture enters public collaboration admission to
exercise actual team actor execution, including the legacy certainty hold,
instance passivation, and joined-yield recovery. It authors 11 local and 16 shared process scenarios,
including two same-name invocations whose unrelated native receipt must not
confirm the paired-host invocation's ambiguous historical failure.
The full target retains the original 59 scenario rows and adds three activation
controls per profile, for 65 total. These controls enter the actual production
activation host with a real MCP transport: first eligible activation discovers
and calls the required tool; a disallowed server cannot start; a missing required
tool after discovery cannot commit the active skill or its prospective preflight.
Failure does not promise cache rollback: a completed eligible discovery may
remain cached until normal host shutdown, but it does not execute a tool or
publish the failed activation. Filesystem skill configs qualify the server name
with the skill ID before discovery, so this fixture's required tool is
`dot04__outcomes__effect`. The required-tool guard checks the prospective prepared
descriptors before activation commit, allowing the first activation to prepare
its own policy-eligible dependency without widening authority.
For shared team scenarios, two real server binaries remain alive concurrently
against the same durable database: A holds execution authority, while B observes
through public runtime routes and must not acquire A's held owner or replay its
work. The fixture records both PIDs, starts, observed overlap, ownership, and
public persistence/catalog scope. It then restarts a real binary and reads the
persisted team outcome and dependencies. Historical recovery rows are deliberately
seeded fault inputs to the actual public recovery consumer, not claims that the
fixture reproduced an earlier deployment.

This is controlled fault injection, not genuine provider inference certification.
Source authoring and whitespace/syntax checks are not evidence that the gate
passed. The execution report must include its exact command, output, source and
executable identities, and any failed attempts retained before repair. No product
suite belongs in GitHub Actions.
