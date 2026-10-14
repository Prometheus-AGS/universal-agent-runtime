# Installed runtime Pod qualification

This is source authoring, pending testing. No resources were provisioned or
operated. The runner refuses execution while the lead's exact source inventory
has `verification_authorized: false`. Neither completing the inventory nor
filling in a configuration authorizes a deployment or disruptive qualification.

The normal shared profile targets the actual `know-me` LKE installation. Its
Host lifecycle is `kubernetes`, and all product calls go through the existing
Envoy HTTPS listener at `https://runtime.know-me.tools/dot17/a` or `/dot17/b`.
The root-owned Helm override defines the StatefulSet and exact per-Pod routes.
Every start reads the Pod UID, StatefulSet owner UID, immutable architecture
image digest, actual packaged executable checksum, mounted configuration
checksum, per-Pod Service and ready EndpointSlices. The HTTPRoute must bind the
existing `argocd/argocd-gateway` `knowme-runtime-https` listener and rewrite the
selected prefix. An unrelated ready Pod cannot substitute for a selected host.

The uncomfortable limit: a container exit is not proof that all descendants on
a predecessor node are excluded. The adapter signals only the verified runtime
PID1, then reads kubelet's termination record for that exact container and its
replacement in the same Pod UID. It never deletes a Pod, evicts a workload,
changes replicas, inhibits a node or grants recovery authority. Unknown effects
stay held. Physical exclusion remains the separately qualified controller stage.
`stop()` causes the kubelet to replace a container; it cannot stop a StatefulSet
permanently. Final cleanup therefore leaves the dedicated installed runtimes
running. Their later decommissioning is an operator deployment action.

A Kubernetes Host contains the ordinary `base_url`, `workspace`, `credentials`
(environment variable names), `config` (private local exact mounted bytes),
`environment` (declared effective infrastructure prerequisites), and these fields:

```json
{
  "lifecycle": "kubernetes",
  "architecture": "amd64",
  "image_digest": "sha256:ACTUAL_ARCHITECTURE_MANIFEST_DIGEST",
  "binary_sha256": "ACTUAL_PACKAGED_RUNTIME_CHECKSUM",
  "kubernetes": {
    "context": "know-me",
    "namespace": "dot17-uar",
    "pod": "dot17-uar-0",
    "container": "uar",
    "statefulset": "dot17-uar",
    "statefulset_uid": "ACTUAL_INSTALLED_UID",
    "runtime_config_path": "/etc/uar/runtime.yaml",
    "service": "dot17-uar-a",
    "http_route": "ACTUAL_INSTANCE_HTTPROUTE_NAME",
    "route_prefix": "/dot17/a"
  }
}
```

Use the names actually installed by the chart; the uppercase values above are
unusable placeholders. The local candidate image must match a same-inventory
`oci-build` receipt, including both locally materialized Linux architectures.
The installed published mode reads the actual release manifest, archive checksum,
image index and both architecture manifests. The Pod image and executable must
then match those published digests. A source build, tag string, or supplied
checksum alone cannot establish publication or installed conformance.

## Controlled peers

`k8s/dot-qualification/render.py` is a fixed source renderer, separately guarded
by the same inventory. It writes manifests and source hashes; it never applies
them. Its private configuration requires `namespace: dot17-uar`,
`name: dot17-qualification-source`, an immutable `python_image`, distinct
`ports: {provider: 18080, mcp: 18081}`, the chart's actual `runtime_labels`,
`existing_claim: surrealdb-data-surrealdb-0`, and `trace_subpath: peer`.
It mounts inventoried peer Python/JSON sources read-only and emits no PVC.
The peer Pod mounts only the `peer` subdirectory of the database's single 10Gi
retained claim in `dot17-uar`. Required Pod affinity places it on the database
Pod's node because this claim uses ReadWriteOnce. The database must be installed
and its directory initializer completed before starting the peer.

The Dot Helm profile enables `surrealdb.qualificationSharedVolume`. That option
sets the database Pod's UID, GID and fsGroup to 65532 and uses the same immutable
UAR image as a fixed Python initializer to create `surreal` and `peer` directories
on the claim. The database mounts only `surreal` at `/data`; its `/surreal`
entrypoint and `rocksdb:/data/surreal.db` argument remain unchanged. The option
is absent from ordinary chart values, which retain their existing mount and
security context behavior. Enable it only for a fresh isolated qualification
claim: it does not move an existing database from the claim root.

Use the candidate UAR image containing Python 3.13 for `python_image`, with its
actual immutable digest. The peer runs as UID/GID 65532 with a read-only root,
bytecode writes disabled, and writable trace/control/token/port/log files only
under `/var/lib/uar/dot17`. Current peer source requires no writable `/tmp`.
The cached Surreal image declares the named user `nonroot`; its metadata does
not establish numeric UID 65532. Explicit UID override compatibility, new-volume
filesystem/fsGroup permissions, initializer writes and shared same-node mounts
remain deployment qualification requirements, not established results.

The claim survives Pod replacement but supplies no database high availability.
Runtime stdio traces still use each runtime Pod's `emptyDir` and survive only
same-Pod container restarts; retained HTTP peer traces cannot certify a replaced
process without fresh source, Pod and process identity observations. The single
retained volume requires explicit paid-resource approval before provisioning;
deleting its claim does not by itself delete the retained storage or end billing.
The peer Pod has no service-account token and no network egress.
Only matching runtime Pods can reach its two incoming ports. Image availability,
Python compatibility, network policy behavior and installation remain unverified.

Set profile `qualification_peer` to the actual peer Pod coordinates:
`{context, namespace, pod, container, image_digest}`. `provider_port` selects its
Service port. Configure the real model provider endpoint using that Service DNS
name. The runner launches only the fixed checksum-bound peer entrypoint; it
cannot accept a shell command or script callback. Runtime stdio MCP launches
`python3 /opt/dot17/dot17_effect_mcp.py /var/lib/uar/dot17/effects.jsonl` and the
MCP elicitation peer through normal private runtime configuration. The renderer
includes the wire JSON companion and the supervisor identity module required by
the effect descendant mode. The image must contain `/etc/machine-id` for that
mode; no privileged host mount is introduced.

A remote trace reference is `{kind: "kubernetes_file", context, namespace, pod,
container, path}`; `path` must be below `/var/lib/uar/dot17/`. Reads accept only
complete newline-terminated records. Remote HTTP MCP entries additionally carry
`peer` with its actual image digest; their lifecycle uses the same fixed peer
launcher. The runtime, not any qualification peer, owns tool admission,
approvals, persistence, budgets and effects.

## Stage-specific hosts

`profiles.<profile>.stage_overrides.<stage>` replaces only specified top-level
profile fields before a stage runs. This allows the calendar, memory expiry,
follow-up clock and schedule-release cases to name a separately authorized
isolated Linux VM and same-guest shared SurrealDB. It does not authorize changing
an LKE node clock or the developer machine clock. Both durable backend profiles
remain required for these cases. Fresh settings and prior-version upgrade stages
also require their own isolated actual installations and stores.

The `schedule-release` stage moves the real guest clock to a new daily due
instant immediately before releasing actual terminal provider bytes. It observes
both scanners, the current-revision occurrence, coalesced dates, Required
approval, exact external effect counts and persisted budget reservation and
settlement. The separate budget case proves exhausted budget refusal. No runtime
clock hook, precomputed passing receipt or synthetic internal event is used.

The source template `scripts/dot/qualification-profile.example.json` contains the
exact local/LKE Host coordinates and receipt fields. Copy it to an operator
private location; replace every placeholder with an observed identity, add both
MCP transport entries, and supply actual compiler-produced artifact and issued
run credentials. Empty transport lists, null artifacts, placeholder checksums,
and absent stage-specific records intentionally cannot run. The peer template is
`k8s/dot-qualification/peer-config.example.json`. No secret is in either template.
The shared `effect_trace` is a list of both runtime Pod file references, so a
winning execution on either Pod contributes to independent effect counts.

On the first attach, after starting the controlled peers, the adapter performs
one verified orderly container restart so the real runtime's SDK discovery
connects to those live endpoints. Subsequent starts bind the already observed
replacement. Runtime stdio traces may therefore contain an earlier installation
handshake; they are preserved, never truncated or accepted as a new interaction.
Every interaction and effect assertion is tied to newly admitted run/request IDs
or a measured before/after count. The peer lifecycle cannot repair an unrelated
failed installation or change the configured endpoint.

Clock qualification credentials must remain valid across the documented future
wall-clock range. Provision isolated credentials and expiry intentionally; the
runner never changes verification time or manufactures accepted credentials to
survive a clock step.

The controlled provider exposes only fixed private `state` and `release`
operations for response barriers. Its remote process creates a random token in
an exclusive mode-0600 file inside the peer's dedicated data volume; the token
is never an execution grant and is not retained in evidence. The runner reads
and uses it only from an authenticated localhost request inside that peer Pod.
Root/child qualification holds actual final upstream text/usage only after all
real tool results, then releases that same response. It cannot complete a tool,
approve an effect or close a runtime channel directly.
