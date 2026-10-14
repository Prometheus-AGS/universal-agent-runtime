# Dot17 MCP LKE and embedded storage source handoff

Status: source authored only. Whole expanded Dot verification boundary closed.
No probes, tests, builds, formatters, checks, validators, source-review gates,
mounts, disk fills, clock changes, deployments or cluster/service operations ran.

## Installed stdio baseline source composition addendum

The final bounded follow-up changes `scripts/dot/mcp_journey.py` and
`docs/operations/dot-mcp-lke-qualification.md` only. Remote stdio now retains an
existing trace baseline instead of requiring a missing file. The fixed observer
binds it to actual before/after Pod UID, StatefulSet owner, container ID/image,
PID1/config identity, mounted peer and wire hashes, file device/inode/bytes hash,
and a live initialized peer PID/start identity whose actual command names the
exact stdio source and trace/control paths. Existing nonempty records without a
matching live initialized writer block; procfs unavailability blocks. The
disconnect file must still be absent and runner-owned HTTP paths remain fresh.

Existing rows are retained without truncation. Every later read requires that
baseline prefix to remain unchanged; answer and capability acceptance only use
new rows. Effects remain measured as per-run before/after deltas. No old response
or initialize row can satisfy a new case. No runtime authority or replacement
stdio peer was introduced. The observed installed trace is initial evidence,
not a claim that prior interactions qualify the new run.

No execution or source-review gate was performed. This follow-up is source
complete pending the full-phase gate; runtime/process assumptions and Python
behavior remain unverified. Existing recovery fault, embedded volume, controller
and fixture implementation were untouched. Files are released to the lead.

## Owned source changes

### Matrix 107–109 source continuation

`scripts/dot/mcp_journey.py` adds real 600-second lifetime, root cancellation,
HTTP admitted-grant revocation/90-second expiry, request-stream abandonment,
and cold persisted-expiry scenarios. The no-SDK-replay case now retains the
actual canonical non-success outcome. Cold expiry compares canonical receipts
and durable approval history across the expiry read and another joined restart, rejects responses and
asserts no pending approval, peer answer or external effect. Both cold records
must actually remain pending after abrupt death; another disposition does not
stand in for the required expiry transition. No new run starts after that death.

`tests/fixtures/dot19_mcp_peer.py` adds a private HTTP request-abandon file
observation, real stdio EOF and HTTP socket/handler lifecycle trace events.
Abandonment closes one real stream, preserves the process, and the next real
call must use the same PID. Handler wait stays bounded at 720 seconds, above
the 600-second production input lifetime. A peer wait deadline is explicitly
not proof of root-owned cleanup. No runtime authority endpoint was added.

`docs/operations/dot-mcp-lke-qualification.md` specifies the admitted
`credential_grant` template, real endpoint/environment requirements and exact
observation IDs. Matrix 107 requires per-transport absolute-lifetime/root-cancel/
request-abandon plus HTTP request-stream-abandon/grant-revoke/grant-expiry and
existing delayed form acceptance. Matrix 108 maps no-sdk-replay; matrix 109 maps
cold-history plus cold-expiry for each transport. The runner must allow at least
the aggregate real 30-minute lifetime waits plus admission/restart/90-second
grant time. A2A owns the separate frozen-child matrix 106 helper; the runner
calls it while the host is live before `run_mcp`. No production changes, tests,
checks, probes or review gates were performed in this continuation. All source
claims remain pending execution and compilation where applicable.

Grant cases use the actual profile `credentials.host` principal for their entire
run, including approval and interaction reads/responses. The registered remote
HTTP destination must explicitly admit that verified trusted-host ID and the
configured grant scope/header names. Normal user authority is never silently
promoted to a trusted host. This uses existing host admission and resource
configuration; no production change or manufactured authority proof was added.

- `scripts/dot/recovery_fault_probe.py`: independent read-only bounded Linux guest
  embedded volume/process/database-descriptor/statvfs/log observation; exact
  call-ID terminal ENOSPC warning extraction; GET-only original Pod API-object
  observation from retained controller registration/private path and its actual
  config TLS API/token/CA. Network and controller journal paths preserved.
- `scripts/dot/recovery_fault_journey.py`: `storage_faults.mechanism` network or
  embedded_enospc using existing real Hyper100 and admitted-provider barriers;
  actual upload ENOSPC response and exact terminal accounting ENOSPC warning,
  unresolved liability/no accepted/indexed ghost after restoration and cold
  process; no automatic volume mutation. Matrix35 now requires original Pod UID
  present at prepare, absent/replaced at offline with old CRI/cgroup/child still
  live, then genuine controller exclusion/inhibition after restoration.
- `scripts/dot/mcp_journey.py`: Kubernetes file references, source-bound runtime
  stdio fixture, runner-owned exact remote HTTP peer process lifecycle, fixed
  control/readiness operations, remote identity evidence, trace retention across
  ordinary and abrupt runtime-container restart, cleanup after partial setup.
- `tests/fixtures/dot19_mcp_peer.py`: explicit fixed HTTP bind choice, loopback
  default and 0.0.0.0 for the isolated internal Kubernetes peer Service.
- `docs/operations/dot-recovery-fault-qualification.md`: added actual embedded
  ENOSPC and deleted-original-Pod contracts, requirements and limitations.
- `docs/operations/dot-mcp-lke-qualification.md`: exact remote file/peer/config,
  installed Envoy API route and persistent trace contracts.

## Runner integration

Existing `run_recovery_faults(profile,evidence)` and storage-exercise/storage-cold
stage names remain. Choose `storage_faults.mechanism=embedded_enospc` and supply
its documented `embedded_volume` observation inputs. Network remains default.
Continuations remain case/source-hash bound and Blocked until required stages.
MCP uses runner-owned `kubernetes.read_trace`, `write_control`, `exec_python` and
`remote_process`; HTTP entries add `peer` target including immutable image digest.
Both fixture source and wire JSON must be mounted under `/opt/dot17`. Remote
stdio trace targets must match the exact selected Host Kubernetes container.

Retained embedded observations add storage-exhausted-volume,
storage-enospc-transactions and storage-restored-volume alongside existing
storage-armed/failures/exercise/cold. Actual error propagation is mandatory;
filesystem exhaustion alone cannot qualify either failed transaction. API-native
error/unknown cleanup outcomes are retained rather than promoted to success.
Matrix35 retains `original_pod` inside controller observations; preparation and
offline bind the original UID and API/CA identity. Operator deletion authorization
is a private case input, never permission to operate anything during authoring.

## Requirements and uncomfortable limits

Embedded case requires an installed actual production binary in a dedicated Linux
VM on macOS/Linux, bounded preallocated guest-only ext4/xfs volume separate from
root and evidence, matching runtime mount/network namespaces and writable DB
file descriptors, JSON log on another filesystem, actual controlled peer and
real embedding search. It does not certify native macOS disk faults. Missing
provisioned resources block execution only; no provisioning ran or was inferred.
Only ENOSPC is covered; quota/permission/power/device-loss are separate mechanisms.

MCP needs actual operator-provisioned know-me peer Pod, source ConfigMap, internal
HTTP Service, retained trace mount and selected installed runtime Envoy route.
Container replacement and peer exit are not supervisor physical-exclusion proof.

Matrix35 requires independent node management and Kubernetes API GET access while
the runtime/controller and old kubelet control path are isolated. A whole node
inaccessible over every management channel is explicitly unsupported. The
stale-internal-executor API submission gap remains unchanged; no authority hook
or forged claim/checkpoint row was added.

Everything remains unverified: syntax, runtime identity observation, VM platform
coverage, actual SDK error chains, live transaction behavior, process cleanup,
cluster image/config/route wiring and all scenario outcomes. No acceptance result
or production qualification receipt has been generated.
