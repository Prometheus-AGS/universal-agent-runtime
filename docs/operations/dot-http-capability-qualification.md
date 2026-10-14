# Advertised HTTP operation qualification

`scripts/dot/http_capability_journey.py` is a fixed production-path driver for
scenario61. It is authored source, not executed evidence. Run it only through the
phase runner after the lead opens the full expanded Dot implementation boundary.
No direct CLI bypass, callback map or configurable arbitrary HTTP operation exists.

The source contract is `src/uar/api/assistant_capabilities.rs`: 31 HTTP operations,
comprising eight memory controls, seven schedule operations, nine follow-up
operations, and seven outcomes/approval/checkpoint/recovery operations. The driver
checks the exact identifier, method and route template for every entry. Adding an
advertised operation requires an explicit driver update; an unknown entry fails.
Discovery describes installed support and never grants authority. Direct embedded
availability remains false. Native and MCP coverage stays in `capabilities.py`.

## Live profile composition

Call `run_http_capabilities(host, spec, evidence)` from the fixed ordinary profile
after existing memory, logical schedule/follow-up and tool-operation stages, while
the provider and runtime remain live. It uses the existing `Host` and profile
`artifact`, `run_credentials` and `deployment_binding_id`. Credentials include
`owner` and `host`; the latter is needed for explicit refusal when recovery is
unconfigured. There are no new resource IDs supplied by the operator.

Already recorded successful requests are reused only when method/path match the
exact operation, the HTTP status is successful, the host label matches, and the
principal label is `owner` (or `host` for recovery). Foreign-principal empty reads,
failed requests, native calls and MCP calls cannot stand in for HTTP operations.
The input is the current fixed runner's `Evidence.http`, not a caller-supplied
list of operation results. Responses remain represented by the existing body
hashes; secret response bodies are not added to coverage receipts.

Missing ordinary operations are invoked with concrete domain behavior:

- Memory policy is read and, when missing, roundtripped without changing consent.
  A real explicit memory is saved for owned export/read/control/history/update as
  needed. Its content must be found by actual semantic search and is then deleted,
  with a subsequent read requiring404. This also covers positive legacy compatible
  search, which currently has no advertised operation ID. Existing memory mutation
  stages are reused rather than repeated wholesale.
- A missing schedule control uses a new actual logical instance and a once-only
  schedule one day in the future. Revisioned update, actual paused state and resume
  state are observed as required. The exact created schedule is cancelled in
  cleanup; no clock is changed and no due execution is fabricated.
- A missing follow-up control uses actual source/destination instances and a
  consent-off rule. Revisions and enabled/disabled/revoked states are checked.
  Inbox/acknowledgement gaps require an actual completed source turn and exactly
  one matching delivered item; acknowledgment is read back. The created rule is
  revoked in cleanup. Already-covered delivery/restart scenarios are not rerun.
- Outcomes/approval/checkpoint-list gaps use a real Required tool run, its pending
  approval and actual owner decision, followed by terminal host state and the
  missing durable read. No approval record or tool receipt is seeded.

Unavailable HTTP entries are exercised with explicit bounded requests. Ordinary
storage-disabled operations require503. The existing ephemeral approval read may
return200 only with `durable:false` and no records; deciding a nonexistent approval
must return404 with `resolved:false`. These responses are refusals, never successful
coverage. A404/401 cannot masquerade as a successful available operation. Recovery
refusal uses authenticated host authority, so an authorization failure is not a
substitute for unsupported behavior. Random IDs are only nonexistent resources in
unsupported negative controls, never forged protected records or exclusion proof.

The live stage emits `http-memory-search` when memory is available and
`discovered-http-operations`, retaining the actual inventory, HTTP indices,
unsupported observations and any missing protected operations. Only
`checkpoint.resume`, `recovery.challenge` and `recovery.exclude` may remain pending
there. A passed live stage with pending protected operations is not complete
scenario61 acceptance.

## Final per-profile reconciliation

The separate fixed `http-capabilities` runner stage calls
`reconcile_http_capabilities(spec, evidence)`. Set `spec.capability_receipts` to
actual receipt paths from the same profile, source revision, full-phase inventory
and executable provenance. Include the ordinary profile receipt and the actual
qualified supervisor safe-checkpoint receipt. Each stage must have passed; each
backing observation file must equal the retained observation. A changed profile,
source, binary provenance, advertised availability or observation blocks the result.
These are prior fixed-runner artifacts, not arbitrary operator passing callbacks.

The qualified supervisor producer retains successful `challenge_status`,
`challenge_path`, `exclusion_path`, `resume_path`, challenge ID, exclusion result,
replay rejection, competing resume statuses and resumed run IDs. Final coverage
accepts only the fixed `supervisor-native-safe-checkpoint` or
`supervisor-kubernetes-safe-checkpoint` observation, with a single successful
resume, a conflicting contender, and repeated independent actual continuation
effect observations. A bare HTTP200, epoch timeout, empty PID lookup or supplied
boolean cannot qualify physical exclusion or protected continuation. Existing
supervisor drivers own the actual native reboot or CRI exclusion; this driver does
not initiate them or invent authority.

Final reconciliation emits `reconciled-http-operations` only after the exact set of
advertised available HTTP entries has real successful coverage and every unavailable
entry has an explicit observed refusal. Missing coverage raises a blocked result.
Scenario61 should require this final observation alongside existing native/MCP
operation observations; `discovered-http-operations` alone is insufficient.

The uncomfortable limit: a discovery response and a successful request are not
standalone proof of complete assistant behavior. This packet composes actual fixed
production scenarios and qualified recovery effects; it does not replace their
ownership, consent, restart or accounting checks. It does not claim installed
publication, direct embedded parity or external follow-up connectors. The intended
final environments are this machine and the existing know-me LKE Envoy TLS path;
no test, build, deployment, provisioning, reboot or clock change was performed while
authoring this source.
