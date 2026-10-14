# Dedicated-host supervisor qualification

The Dot17 `supervisor` stage is authored for the full-phase boundary. It has not
been executed. `scripts/dot/supervisor_journey.py` composes real host recovery
APIs with an independent, read-only SSH observer. The observer reads the actual
controller journal, boot/process identities, detached-child trace and, on a
qualified Kubernetes node, retained CRI/cgroup/kubelet state. It never writes
database records, fabricates an exclusion, reboots a machine or restores kubelet.

The uncomfortable limit: this stage needs deliberately prepared dedicated
machines and real service/Pod deployments. A developer workstation and invented
case IDs cannot stand in for them. A missing host, journal, original cgroup or
CRI record blocks qualification. A passing HTTP response alone is insufficient.

## Candidate installation

Install the exact source tree and candidate UAR binary on an isolated host with
its real native service manager or Kubernetes/CRI profile. Use the installation
instructions in `supervisor-recovery.md`. For a previously unqualified mechanism,
start the independent controller with `--qualification-candidate` and a private
exact-source-hash qualification record that says `production_qualified: false`,
`isolated_candidate: true`. Its ID begins `dot17-candidate-`; deployment names
begin `dot17-`. Default production startup cannot accept this record. No gate
automatically promotes it. Production qualification requires reconciled evidence
for every required positive and negative case and an explicit operator promotion.

Provision separate candidate TLS credentials and private persistent stores. Run
the real detached-child fixture `tests/fixtures/dot17_effect_mcp.py` through an
authorized UAR tool call using mode `hold_descendant` and a fresh marker. It
records the child's PID, start identity, machine/boot identity and fsynced
heartbeats. Configure a second actual run/claim for each safe-resume,
unknown-effect and team-reclaim case; use the production API to create work.
Do not edit protected execution or accounting rows to manufacture a scenario.

Prepare a private JSON case file with the fields accepted by
`scripts/supervisor/recovery_scenarios.py`: `name`, `runtime_epoch`, `target`, and
the optional real `resume` or `team_reclaim` inputs. Add `marker`, an absolute
`continuation_file`, and this observer configuration:

```json
{
  "dedicated_qualification_host": true,
  "ssh_target": "qualification-user@dedicated-host",
  "known_hosts": "/operator-private/qualification-known-hosts",
  "python": "python3",
  "expected_machine_id": "actual-dedicated-machine-identity",
  "controller_source": "/installed/uar/scripts/supervisor/controller.py",
  "controller_config": "/operator-private/controller.json",
  "trace_file": "/operator-private/detached-child.jsonl"
}
```

These are placeholders, not executable evidence. SSH uses strict pinned host
keys, batch authentication and a fixed streamed Python observer. The observer
refuses controller source whose hash differs from the current phase source.
Its account needs read access to the protected journal and actual platform
observations; it never prints controller credentials or raw private settings.
On Windows use an installed `python.exe` on the dedicated SSH host.

Each storage profile's runner configuration has a `supervisor` object containing
`runtime_urls`, `host_token_env`, `ordinary_user_token_env`, and `cases`. Every
The object also specifies `mechanism: native` or `mechanism: kubernetes`. Every
case reference contains `private_case_file` and one of the following `stage`
values. Tokens are environment references, never case-file credential bytes.

Physical recovery requires four real cases, each exactly once per mechanism:
`case_role: safe-checkpoint`, `uncertain-checkpoint`, `safe-team`, and
`uncertain-team`. A checkpoint case supplies `resume`; a team case supplies
`team_reclaim`. Safe checkpoint resume requires exactly one CAS winner and an
actual completed resumed run; uncertain checkpoint resume requires zero winners.
`resume.workspace_id` identifies the real owner's workspace. Team cases supply
nonempty real queued or uncertain attempt IDs. An uncertain-only team case has
no queued attempts. Safe transferred attempts are dispatched through the real
collaboration API, with a fresh read of the team's revision.

Every case adds `continuation_effects`, a list of `{marker, expected_delta}`
objects. Markers differ from the detached-child marker and must appear in the
same independent fsynced effect trace. Safe cases require at least one delta1;
held uncertain effects require delta0. `queued_dispatches` lists exactly the
transferred queued attempts with `team_id`, `attempt_id`, `workspace_id`,
`owner_token_env`, `admin_key_env`, and a real `body.commandId`. These are actual
admitted work identities, not substitute expected responses. Trace observations
must show the exact effect deltas and remain stable on a second observation.

The fixed final evidence IDs are `supervisor-native-<case_role>` and
`supervisor-kubernetes-<case_role>`. They are emitted only after real physical
exclusion, continuation/reclaim and independent effect observation. Exclusion
alone cannot emit them. The authority stage separately emits
`supervisor-<mechanism>-controller-security`. A failed, missing or duplicate
continuation role prevents successful qualification of the composition.

## Native stages

1. `native-prepare`: after deliberately crashing the isolated UAR root, retain
   its detached child. The stage observes advancing heartbeats and the original
   boot, confirms the old root identity is no longer observed, checks ordinary
   caller/wrong-identity refusal and actual same-boot exclusion refusal, and
   saves an immutable private continuation. Its outcome is **blocked awaiting
   explicit reboot**, not passed. The runner does not kill or reboot the host.
2. Authorize and perform a reboot of that dedicated test machine through the
   operator's normal procedure. Restore its independent controller and a fresh
   UAR service over the same durable state. Do not change the case file, source
   tree, original identities or continuation file.
3. `native-complete`: reads the continuation, compares exact phase source and
   case hashes, observes the same machine and a different boot with stopped old
   heartbeats, then calls real exclusion, replay and configured resume/reclaim
   races. The response must explicitly leave effects unresolved and execution
   unstarted. The actual controller journal must retain its exclusion record.

Run each supported native platform separately. Two contenders may share a local
runtime URL; the shared profile must use its distinct real runtime URLs. Retain
safe-resume and unknown-effect cases separately: exclusion itself cannot clear
pending money or unknown effects. The detached-child marker is an observation
marker, distinct from any resumed tool effect marker.

## Qualified Kubernetes stage

`kubernetes` requires `authorize_node_inhibition: true` in the private case. It
observes a living detached child, the exact populated Pod and kubelet cgroups,
then obtains real exclusion through a surviving runtime. It observes stopped
heartbeats, unchanged node boot, exact stopped sandbox/container IDs, and
inactive runtime-masked kubelet. Existing retained cgroups must be empty and
match their original device/inode. If CRI cleanup removes one, the original
live observation plus the controller's durable empty-subtree observation are
required; disappearance alone never passes.

The controller can leave the **entire dedicated node inhibited** on either
success or a partially completed failure. No runner cleanup unmasks kubelet.
Only the operator may restore it after retaining the evidence and resolving
uncertain work. Unreachable-node, controller-restart, wrong-certificate,
cross-deployment, stale-executor and persistence-outage cases also remain
required for full mechanism qualification; this lifecycle stage does not by
itself certify those separate scenarios or a published release.

## Controller authority stage

`controller-security` uses the same real registration and independent observer.
The private case adds `controller_security` with `endpoint`, `server_ca`,
`authorized_certificate`, `authorized_key`, `wrong_certificate`, `wrong_key`,
and `foreign_deployment_id`. Use the original registration certificate for the
baseline. The wrong certificate is signed by the isolated client CA but is
absent from the controller fingerprint allowlist; an HTTP 409 must be observed.
A TLS/network failure is not counted as authorization enforcement. The foreign
deployment differs from the certificate's registration deployment. The stage
requires its registration request to fail and independently checks that the
controller's durable registration count and original row are unchanged.


## Superseding native qualification scope — 2026-10-14

The operator selected “Just use the Linux guest.” This release uses the approved isolated Linux guest for native systemd recovery qualification and the existing two-runtime LKE profile for shared qualification. This supersedes earlier requirements for separate macOS and Windows native recovery or timezone evidence for this release. Those platforms receive no qualification credit from Linux evidence. Existing platform implementations and historical records remain; the development Mac is not rebooted or subjected to clock changes.

Both durable profiles, all 24 production items, all nine reliability requirements and all nine issue #379 requirements remain. Linux native qualification must still prove actual containment and reboot-bound exclusion, surviving-descendant handling, stale/replayed authority rejection, safe checkpoint/team continuation, and preservation of uncertain external effects and unsettled accounting. Calendar, timezone, quiet-hour, memory-retention and bounded disk-failure scenarios remain. Linux amd64/arm64 publication, the development-Mac customer build, installed-artifact conformance and final M3/K3 review remain required. This scope decision records no passing result and authorizes no additional paid resources.


### Linux candidate executable from the OCI build

The local OCI build exports the actual regular ELF runtime executable from each verified image filesystem. Its `local-two-architecture-image` observation records `source_content_sha256` and `runtime_binaries` for amd64 and arm64, including each executable path, SHA-256, size, architecture and image digest. These are candidate artifacts; no publication is claimed.

For Linux guest qualification, install the selected arm64 executable and run the source-bound acceptance driver inside that guest. Copy the genuine same-inventory OCI receipt, observation, archive and exported executable without relabeling their paths or results. The host configuration selects `candidate_runtime_origin: "oci"`, `os: "linux"`, `architecture: "arm64"` and the genuine `oci_build_receipt`. The runner checks the receipt/source binding, archive, exported executable, actual installed executable bytes and current host architecture. The guest's installed path may differ from the export path. Native macOS candidate validation retains its existing local build receipt. Source/plan inventory and every operational stage still require their actual inputs and observations; exporting an executable proves no recovery behavior.
