# MCP child and root isolation qualification

This source packet covers scenario106 through the installed host, its durable
thread/interaction stores, a real registered child artifact, and the independent
stdio and Streamable HTTP peers. It is authored only. No tests, checks, builds,
processes, probes, reviews or deployments have run for this packet.

The uncomfortable failure is a plausible child interaction that actually belongs
to a new connection, or successful root cleanup that also closes another root's
pending input. Neither a durable cancelled flag nor a provider-generated message
alone proves handler teardown or preserved authority.

## Fixed runner interface

`scripts/dot/mcp_child_journey.py` exports
`run_mcp_child(host, spec, evidence, peers, provider)`. The normal profile runner
calls it immediately before `run_mcp`, while the prepared peers and selected host
are live and before any abrupt host crash. It adds no CLI bypass and accepts no
callback, supplied pass result, fixture database row or supervisor attestation.

Each existing `mcp_transports` entry supplies:

```json
{
  "mcp_child": {
    "root_artifact": "the actual complete AgentArtifact object",
    "child_artifact_id": "the ID of the already persisted child AgentArtifact"
  }
}
```

The example's root_artifact string is explanatory: the actual value must be an
object accepted by the existing native run API. The root artifact explicitly
lists `spawn_agent` in `policy.tools.allow` and admits the entry's real
`elicit_tool` and `effect_tool`, model, server, execution mode and sufficient root
budgets. The child artifact is installed beforehand through the normal catalog
administration path and explicitly sets `policy.tools.allow` to the one-element
list containing the exact `elicit_tool` runtime ID. Its remaining policy, provider
and sandbox requirements must be a valid subset of that root. The driver reads
`GET /api/agents/{child_artifact_id}` before and after; it does not install inline
child definitions or mutate catalog ownership. Existing entry `artifact` and
`run_credentials` overrides retain their normal profile meaning; the dedicated
root_artifact replaces only the artifact for these two roots. Both roots use the
same authenticated host, owner, artifact, credentials and server configuration.

The provider adapter exposes `rows()`, `barrier_state(marker)` and
`release(marker)`. The fixed `root_completion_barrier` instruction holds only the
final model text/usage after all actual tool results in a nonempty tool sequence.
Its authenticated loopback control belongs to the isolated upstream peer, not UAR.
The runtime receives no clock, pause, authority or state-write testing endpoint.

## Actual observations

For each transport, root A first invokes the real elicitation tool and consumes
one actual response. It then invokes native `spawn_agent` with exactly
`artifact_id`, `delegated_prompt`, `task_name` and `history_fork:{mode:"none"}`.
The trusted host performs its registered-artifact lookup. Root A's final upstream
response waits while that child's elicitation becomes pending.

The driver reads committed child lifecycle events from the root's bounded event
API, rejecting retention gaps. It compares the child's durable interaction to A's
original `owner_key`, `run_id`, `session_id`, `workspace_id`, `channel_id`,
`connection_id` and `server_name`; all must match. Child-run interaction lookup
must not contain an independent channel. Independently captured child provider
requests must advertise the allowed elicitation tool while omitting the parent's
effect and spawn tools. Root provider requests supply the positive broader-tool
control. Exactly two real MCP initialize observations must occur for A and B;
the child cannot reconnect under a freshly resolved server environment.

Root B independently opens input on that same owner/server while A's child waits.
B must have different channel and connection IDs (and a different peer process
for stdio). Releasing only A's upstream final wire must produce root completion,
a committed cancelled child lifecycle and a nonresolvable child interaction with
no fabricated response. Late acceptance must receive409. The independent peer
must observe actual stdio input EOF or HTTP handler completion from socket EOF,
socket error or a written response; the peer's long wait deadline never counts.

B must remain resolvable with no peer close, receive its exact response once,
then request a separate Required approval before its effect can run. The real peer
must record exactly one effect after that approval. Input acceptance confers no
tool authority. No fixture writes durable interaction or thread rows.

Observation IDs are `mcp-<entry.name>-child-root-isolation`, one per declared
transport entry. Matrix106 requires both stdio and HTTP observations from the
same source/profile/artifact-bound whole runner receipt. Retained data includes
actual lifecycle/interaction identities, initialize traces, handler closure,
provider tool advertisements, approval IDs and independent effect counts.

A missing persisted child, unsupported artifact, unexpected reconnect, missing
peer EOF or unavailable real collaborator fails qualification; none is converted
into a synthetic passing result. The qualification configuration must be isolated
and authorized separately before running. Source authoring does not provision,
modify, deploy, stop or restart this machine or the know-me LKE environment.
