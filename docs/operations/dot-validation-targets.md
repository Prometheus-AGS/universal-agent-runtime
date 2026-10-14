# Dot final validation targets

The operator selected this development machine and the existing LKE cluster in
kubectl context `know-me`. The current objective is **code complete, testing
pending**. No test, build, validator, product probe, deployment or disruptive
qualification starts until every expanded-phase production task is implemented
and the lead records the complete source inventory. The inventory remains closed
to execution until testing is explicitly resumed.

## Configuration observed on 2026-10-12

Read-only Kubernetes resource queries found Gateway `argocd/argocd-gateway`
(GatewayClass `eg`). Its listeners `knowme-runtime-https` and
`knowme-runtime-http` serve `runtime.know-me.tools`; both allow routes from all
namespaces. HTTPS terminates with the existing `know-me-tools-site-tls`
certificate reference. Envoy proxy deployment configuration places its data
plane in `envoy-gateway-system`, matching the chart's network policy. No
runtime-host HTTPRoute was present in that configuration snapshot.

The cluster has `linode-block-storage-retain`, with provisioner
`linodebs.csi.linode.com` and Immediate binding. These are configuration
observations, not successful rollout, certificate validity, DNS, capacity,
health or product acceptance. Re-read configuration at the final boundary;
existing workloads and gateway listeners are shared resources.

## Minimum isolated shared profile

Use namespace `dot17-uar`, two named runtime Pods, one dedicated networked
SurrealDB StatefulSet with a retained 10Gi volume, and the controlled peer Pod
described by the acceptance runner. Postgres and Redis are unnecessary for the
selected SurrealDB profile; their separate adapter acceptance remains in the
source integration plan. Use `k8s/helm/uar/values-dot-know-me.yaml` together with
private operator values. The override reuses the existing gateway and storage
class instead of creating either resource.

The runtime Pods are `dot17-uar-0` and `dot17-uar-1`. Per-Pod Services expose
them through `https://runtime.know-me.tools/dot17/a` and `/dot17/b`. The
HTTPRoute binds explicitly to the HTTPS listener and rewrites each prefix to
the runtime root. The normal root route targets the shared Service. A separate
HTTPRoute on `knowme-runtime-http` redirects the hostname to HTTPS with301.
The backend rules disable request/backend-request timeouts for the real SSE and
approval-wait journeys. Idle-stream policy remains an independent gateway setting.
No second gateway, TLS secret or load balancer is proposed.

The private values must supply the source-bound candidate image repository and
immutable digest. The existing-secret setting names `dot17-uar-private`; create
that private secret only at the authorized installation boundary. It must
contain deliberate JWT and settings-admin secrets, SurrealDB server credentials
(`SURREAL_USER`, `SURREAL_PASS`) and the matching runtime configuration keys
(`UAR_PERSISTENCE__SURREAL_USER`, `UAR_PERSISTENCE__SURREAL_PASS`). Additional
scenario credentials are environment references in private runner inputs,
never checked-in token values. The shared namespace/database are both `dot17`.
Credential-management qualification also requires `CREDENTIAL_ENCRYPTION_KEY`
from a private secret, with its exact format from the existing credential service
documentation. Do not substitute dummy startup defaults for this service.
Private Secret `dot17-uar-config` supplies `runtime.yaml`, mounted read-only at
`/etc/uar/runtime.yaml` and passed explicitly to the packaged binary. The runner
binds its digest. ConfigMap `dot17-qualification-source` supplies the exact
controlled fixture files at `/opt/dot17`; the fixed peer source renderer belongs
to the runner. These resources are isolated qualification inputs, not part of a
production authorization bypass.

The checked-in override boots with memory disabled, matching the safe default.
Memory acceptance uses its own explicit private memory-enabled configuration,
local embedding assets or the isolated controlled embedding peer, and actual
owner opt-in through the policy API. Disabled-memory negative acceptance and
enabled-memory acceptance are distinct stages. No default hosted inference
provider or paid model call is needed for the controlled protocol scenarios.
Normal memory-enabled qualification must override
`uar.env.UAR_MEMORY__ENABLED: "true"` in the private Helm values as well as the
mounted configuration. Otherwise the safe-default environment value takes
precedence and correctly leaves memory unavailable. Automatic capture still
starts off until owner consent.
The memory-enabled private secret also supplies `UAR_MEMORY__SURREAL_USER` and
`UAR_MEMORY__SURREAL_PASS`; its namespace and database remain `dot17`.

Both runtimes share durable business state in SurrealDB. Their empty writable
runtime directories are scratch space, not a source-content durability claim.
Uploaded source, jobs, session history, key revocations, budgets and checkpoints
must be recovered from the actual durable provider through their production
APIs after a process restart. Image contents provide built-in skills.

## Local profile and disruptive qualification

Use separate private working directories and SurrealKV stores on this machine,
with the exact candidate binary bound to the final source inventory. The local
runner owns and joins only the child processes it launches. It must not restart
the desktop application, alter this machine's clock or reboot this machine.

The selected validation locations do not themselves prove native Windows/Linux
qualification, macOS reboot recovery, real clock jumps, or whole-node Kubernetes
fencing. Those source-authored cases remain pending execution. Reboot-bound
native cases need an explicitly authorized isolated machine. Clock-change cases
need an isolated guest, with the shared database inside the same guest for
database-time semantics. Kubernetes exclusion needs an explicitly isolated node
and independent containment controller; Pod deletion or a missed heartbeat is
not exclusion proof. Do not inhibit an LKE node serving unrelated applications.
Unavailable infrastructure leaves the specific qualification unverified; it
does not waive a requirement or justify a fabricated receipt.

## Incremental cost proposal

Reuse the already-paid LKE workers and Envoy gateway where capacity permits.
The proposed 10Gi database volume is approximately US$1/month at the published
US$0.10/GB-month Block Storage rate. No new control plane, load balancer,
managed database, GPU or hosted inference subscription is required for this
minimum profile. Existing worker capacity has not been measured; an added
dedicated recovery node or isolated clock guest would add its quoted compute
cost and needs a separate concrete provisioning decision. Taxes and any
transfer overage are additional. Retained volumes remain billable until removed.

Price source checked2026-10-12: [Akamai Cloud pricing](https://www.akamai.com/cloud/pricing).
Routing syntax: [Envoy Gateway HTTPS redirect](https://gateway.envoyproxy.io/docs/tasks/traffic/http-redirect).
Nothing in this proposal is a provisioning, deployment or destructive-action
receipt.

## Concrete resource proposal for the remaining qualification

Start with the normal profile above: this machine, the existing cluster and
gateway, two runtime Pods, the controlled peer, and one 10Gi durable database
volume. Incremental standing cost is approximately $1/month if existing workers
have enough capacity. Capacity has not been qualified. This is the minimum
normal-profile setup, not a price for completing every platform recovery case.

For disruptive Kubernetes qualification, add one temporary, isolated 8Gi shared
CPU worker with no unrelated applications. A published Linode 8GB example is
$0.072/hour, capped at $48/month; a 48-hour allocation is approximately $3.46
before tax and overage. This size is an initial allocation proposal, not measured
capacity. The trusted controller and durable database stay outside that worker
so they survive its inhibition. Confirm the exact plan offered in the cluster's
region before provisioning. Do not place this workload on an existing shared
application node merely to avoid the temporary charge.

Use an isolated Linux guest for real clock changes and bounded disk exhaustion.
It can run on existing spare hardware at no additional cloud charge. Otherwise,
allocate a second temporary VM; using the same published 8GB example for another
48 hours adds approximately $3.46. The database for clock cases belongs inside
that guest. The intentionally full filesystem is a small separate disposable
volume, never the development checkout or the main database volume.

Native qualification additionally needs a spare Mac that can reboot and isolated
Windows and Linux machines or guests with their real service managers. Reuse
existing licensed hardware first. No spare machines or licenses have been
identified, so their purchase or rental cost is unresolved. Do not treat a Linux
Pod as evidence of Windows or macOS native recovery. The approved macOS fallback
requires a confirmed reboot when same-boot descendant exclusion is unavailable.

With reused native hardware and a local Linux guest, the proposed incremental
cloud allowance is about $1/month plus $3.46 for one 48-hour dedicated recovery
worker. With a second cloud Linux guest for the same period it is about $1/month
plus $6.92. These are resource allowances, not a guarantee that qualification
finishes within 48 hours. Stop temporary compute after evidence collection;
retained storage continues billing. No resources have been purchased or created.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


### Linux candidate executable from the OCI build

The local OCI build exports the actual regular ELF runtime executable from each verified image filesystem. Its `local-two-architecture-image` observation records `source_content_sha256` and `runtime_binaries` for amd64 and arm64, including each executable path, SHA-256, size, architecture and image digest. These are candidate artifacts; no publication is claimed.

For Linux guest qualification, install the selected arm64 executable and run the source-bound acceptance driver inside that guest. Copy the genuine same-inventory OCI receipt, observation, archive and exported executable without relabeling their paths or results. The host configuration selects `candidate_runtime_origin: "oci"`, `os: "linux"`, `architecture: "arm64"` and the genuine `oci_build_receipt`. The runner checks the receipt/source binding, archive, exported executable, actual installed executable bytes and current host architecture. The guest's installed path may differ from the export path. Native macOS candidate validation retains its existing local build receipt. Source/plan inventory and every operational stage still require their actual inputs and observations; exporting an executable proves no recovery behavior.
