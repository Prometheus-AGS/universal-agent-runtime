# Dot17 single retained claim correction

Phase: Dot feature support Execute. Source correction applied after the lead's
`BASELINE CAPTURED exactsource02 before+after` release on 2026-10-13. Source is
frozen for the lead's next serialized gate. No resource has been provisioned.

## Observed problem and resulting contract

The qualification chart requested a 10Gi SurrealDB claim while the controlled
peer renderer emitted a second 1Gi claim. The latter would also incur Linode's
minimum-volume charge. The approved planning scope permits proposing one retained
10Gi claim; paid-resource approval and deployment remain separate pending actions.

The peer renderer now requires `existing_claim: surrealdb-data-surrealdb-0` and
`trace_subpath: peer`, emits no PVC, and mounts only that subdirectory. Required
same-namespace Pod affinity to the chart's actual SurrealDB labels co-locates the
peer with the database on `kubernetes.io/hostname` for ReadWriteOnce access.

The Dot chart profile enables `surrealdb.qualificationSharedVolume`. Only that
option adds a Pod security context with nonroot UID/GID/fsGroup 65532 and a fixed
Python initializer using the exact UAR repository and required immutable digest.
The repository Dockerfile supplies Python 3.13. No shell or tools from the
SurrealDB image are assumed. The initializer creates `surreal` and `peer` at the
claim root; the database mounts `surreal` at `/data`. Its image, entrypoint and
database arguments are preserved. The ordinary chart, without this option,
retains its previous root mount and security context behavior.

This is for a fresh isolated claim in `dot17-uar`. Enabling it on an existing
root-layout database would select a different directory; no migration of an
existing application database is implemented or authorized.

## Files changed

- `k8s/dot-qualification/render.py`: fixed existing-claim/subPath requirements,
  required database co-location, removal of the second PVC object.
- `k8s/dot-qualification/peer-config.example.json`: explicit existing claim and
  peer subdirectory inputs; image placeholder remains deliberately unusable.
- `k8s/helm/uar/templates/surrealdb-statefulset.yaml`: opt-in numeric identity,
  fixed candidate-image Python initializer and database subPath.
- `k8s/helm/uar/values-dot-know-me.yaml`: enables the qualification-only option
  while retaining the 10Gi retained storage selection.
- `docs/operations/dot-installed-pod-qualification.md`: installation order,
  exact configuration contract, permission evidence and durability limitations.
- This handoff records the correction and pending qualification evidence.

No runner, runtime, API, policy, source-inventory projection or task checkbox was
edited. No arbitrary command, callback or runtime authority hook was added.

## Evidence and limits

Before editing, read-only Docker metadata inspection found the cached exact
SurrealDB v3.3.0 index declares `USER nonroot` and entrypoint `/surreal`; it did not
establish the user's numeric UID. The cached image was arm64, not evidence of an
executed amd64 deployment. The current chart's service, labels and derived claim
name were read from source. Kubernetes GET observations found CSI
`fsGroupPolicy: ReadWriteOnceWithFSType` and the existing application SurrealDB PV
uses ext4, Filesystem and ReadWriteOnce. The new claim does not exist, so its
filesystem and effective ownership are unobserved.

The explicit 65532 override matches the UAR/peer source contract. SurrealDB
execution with that override, group-writable initialization on the new CSI
volume, both same-node mounts, candidate Python availability and read-only-root
peer execution remain unverified. Official documentation permits a selected
nonroot identity with accessible mounted files; it does not qualify this install:
[SurrealDB Docker guidance](https://github.com/surrealdb/surrealdb/blob/main/docker/DOCKER.md),
[Kubernetes CSI ownership contract](https://kubernetes.io/docs/reference/kubernetes-api/config-and-storage-resources/csi-driver-v1),
[Pod security context](https://kubernetes.io/docs/tasks/configure-pod-container/security-context).

Database and HTTP peer files use separate directories on one retained volume.
This is not high availability. Runtime stdio trace files remain on Pod-local
emptyDir and do not survive Pod replacement. Old retained peer records never
substitute for fresh source/Pod/process identity evidence. Install the database
and complete its initializer before starting the peer. Retain requires deliberate
later storage cleanup; deleting a PVC alone does not end volume billing.

## Verification and next owner

No render, test, check, lint, build, probe, container execution, image pull/push,
mount or cluster write ran in this correction. No new verification result is
claimed. The lead owns source inventory rebinding and the next serialized gate,
then any explicitly approved deployment. The uncomfortable limit is that the
single-volume source contract can still fail actual CSI permissions or database
UID compatibility; only that real deployment evidence can close those claims.

The fixed claim and subdirectory requirements protect the dedicated storage
boundary against accidentally selecting an application claim or database tree.
Required co-location addresses the observed ReadWriteOnce constraint. No guards
for unrelated hypothetical failures were introduced.
