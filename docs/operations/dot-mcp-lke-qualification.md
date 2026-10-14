# MCP peer qualification on the installed know-me runtime

The MCP source fixture supports both local processes and the fixed Kubernetes
peer transport owned by `scripts/dot/kubernetes.py`. Nothing was deployed or
executed while authoring it. Runtime API calls remain the runner's selected
`https://runtime.know-me.tools` Envoy route with its retained prefix and existing
TLS redirect contract. Internal controlled MCP peers use their isolated Service;
they do not replace the runtime API or receive admission authority.

Each `mcp_transports` entry still declares `name`, `transport` (`stdio` or `http`),
`server_name`, exact `elicit_tool` and `effect_tool`, `trace`, and
`disconnect_file`. For Kubernetes, both file fields are objects:

```json
{
  "kind": "kubernetes_file",
  "context": "know-me",
  "namespace": "dedicated-qualification-namespace",
  "pod": "actual-selected-pod",
  "container": "actual-selected-container",
  "path": "/var/lib/uar/dot17/unique-case-mcp.jsonl"
}
```

The trace and disconnect reference must identify the same container and different
paths. Runner-owned HTTP peers require fresh paths; the fixed helper creates
parent directories and refuses existing files. An installed runtime may already
have started its stdio peer, so stdio preserves an observed trace baseline instead
of truncating it or requiring its absence. The helper only reads complete newline-terminated peer records and toggles the
controlled peer's disconnect file. The actual trace and control volumes must
survive runtime-container restart; each restart asserts the previous trace is
an unchanged prefix of the new trace.

For HTTP, add `http_port` and `peer` with the same context/namespace/Pod/container
coordinates and immutable peer `image_digest`. The `remote_process` helper
requires the dedicated peer Pod label, actual matching container image and
source-bound mounted `/opt/dot17/dot19_mcp_peer.py` plus
`/opt/dot17/dot19_mcp_wire.json`. It launches only this known peer and retains
its actual PID/start identity. The MCP fixture uses a unique port publication
file under the dedicated trace directory, waits for its real selected port, and
retains process identity. The peer's new fixed `--host 0.0.0.0` option permits
its internal Service to reach it; local fixtures keep `127.0.0.1`. Do not expose
this controlled peer externally. Its HTTP Service targetPort must match
`http_port`, and the actual installed runtime config must point to that Service.

For stdio, UAR itself starts the source-bound peer from the mounted ConfigMap.
The trace and disconnect references select the runtime container with its
persistent dedicated fixture-data mount. The fixture never starts a substitute
stdio client or server outside UAR. Both source files must be present beside
each other because the peer loads the pinned wire description relative to its
own filename.

Before attaching to an installed stdio peer, the runner binds the observation to
the declared StatefulSet owner, Pod UID, running container ID and immutable image,
plus the actual runtime PID1 executable/start identity and mounted configuration
digest. It observes the same container identity before and after reading the
baseline. Existing initialized records must include a PID that still identifies
a live Python peer with the exact mounted source, stdio transport, trace and
disconnect paths. The observer retains that process's start ticks and command,
the peer/wire source digests, and the trace's device, inode and observed bytes
digest. A historical trace without a matching live initialized writer blocks;
missing procfs access also blocks. This is a qualification resource requirement,
not permission to clear or replace existing records.

Every later trace read requires the retained baseline to remain an unchanged
prefix. Existing response and initialize rows are excluded from scenario
acceptance; effect checks compare counts immediately before and after each new
run, and responses use the newly persisted request ID. Thus a retained trace can
establish its starting position without making an old accept or capability
record pass a new scenario. The normal Host first-attach container restart
remains separately observed by the runner's lifecycle adapter.

Controlled HTTP shutdown and restart use the runner's exact retained peer
process identity. Disconnect closes only that controlled peer. Runtime
start/stop/restart uses the installed Host Kubernetes lifecycle and its actual
kubelet termination observations. Neither peer termination nor runtime container
replacement is physical supervisor exclusion; no held effects are released.
The abrupt final case performs only cold interaction history reads, refuses
answers to the lost channel and observes no replay.

The uncomfortable dependency is the operator-provisioned isolated peer Pod,
matching source ConfigMap, durable trace volumes, actual Services and dedicated
runtime route. Missing or mismatched resources block execution rather than
switching to local fake peers. No arbitrary commands, callback receipts, copied
traces or client-side answers can qualify the MCP scenarios. Acceptance remains
separate from each real Required tool approval. All accept/decline/cancel,
transport-disconnect, SDK no-replay, sibling-isolation and cold-history scenarios
remain unexecuted under the full-phase verification boundary.

## Absolute lifetimes, abandonment and cold expiry (matrix 107–109)

Each transport now has a real ten-minute input case: the stored expiry must be
exactly 600 seconds after creation, the ordinary tool timer must not terminate
it early, expiry must persist, and the peer may receive only cancellation. A
joined restart must retain the expired record. The existing 31-second pending
form acceptance still proves the ordinary 30-second execution timer pauses.
The driver polls for up to 660 seconds; the peer's real request wait is bounded
at 720 seconds. No clock or stored timestamp is changed.

Root cancellation uses the actual `POST /api/uar/runs/{id}/cancel` endpoint.
For the HTTP peer, configure `credential_grant` on its `mcp_transports` entry
with the registered destination's permitted `credential_revision`, `scopes`
and `headers`. These are private fixture credentials admitted by the real host,
never a fabricated lease or persistence write. Grant cases use the profile's
real `credentials.host` credential for admission, approval, interaction and
terminal reads, preserving that exact authenticated ownership throughout. Its
verified trusted-host ID must be allowed by the registered destination's grant
policy; a normal user credential cannot stand in for this authority. The fixture generates a fresh
revision and sends the normal run request's `mcp_servers` field, then either
revokes it through `POST /api/uar/runs/{id}/mcp-grants/{server}/revoke` or waits
for its actual 90-second expiry. Revocation uses an admitted 900-second grant.
Both must close input before the independent ten-minute lifetime, reject the
answer, terminate the sequence without success, and leave the effect absent.
An expiry case cannot pass on an earlier disconnect or ordinary tool timeout.

For HTTP request abandonment, the controlled peer observes the dedicated
`disconnect_file + ".abandon-request"` file while an actual request is pending,
closes only that request stream, and keeps its process running. The input must
become nonresolvable, a response must be refused, and a subsequent real declined
call must return from the same peer PID. The control file must start absent and
is removed after the case. Existing per-transport process-disconnect cases
remain separately recorded. EOF and HTTP handler completion records identify
the actual request and disposition; a peer's own 720-second timeout cannot be
used as evidence of timely root cleanup.

After the final abrupt host death, both transport records must still be pending
without live waiters before the cold-expiry case can begin. The driver waits for
their actual persisted deadline, reads them to cause the production API's
expiry CAS, refuses further responses, and compares canonical tool outcomes and
durable approval history before and after the read and another joined restart. No approval may be
pending and no additional peer response or external effect may occur. Only
cold reads occur after the abrupt death; no new logical run is admitted.

The exact observation IDs use each configured entry's `name`: matrix 107 uses
`mcp-{name}-absolute-lifetime`, `mcp-{name}-root-cancel`,
`mcp-{name}-request-abandon`, plus the HTTP entry's
`mcp-{name}-request-stream-abandon`, `mcp-{name}-grant-revoke` and
`mcp-{name}-grant-expiry`. Matrix 108 uses `mcp-{name}-no-sdk-replay`, including
one actual invocation and current canonical non-success receipts. Matrix 109
uses `mcp-{name}-cold-history` and `mcp-{name}-cold-expiry` for both transports.
The runner's stage allowance must accommodate two live ten-minute cases, one
cold ten-minute wait, restarts and grant expiry; a four-minute wrapper cannot
qualify these scenarios. All observations remain unexecuted source contracts.

## Issuer scope of foreign-input controls

The installed host trusts its one configured issuer. The same-issuer other-tenant
credential must authenticate and receive empty interaction history and a denied
response. The other-issuer credential must receive401 on both operations; this is
authentication rejection, not authenticated issuer isolation. Accepted principals
from separate issuers are exercised by the existing real separate-host credential
integration target and principal storage targets. No additional issuer trust is
introduced for qualification. These outcomes remain unexecuted until their gate.
