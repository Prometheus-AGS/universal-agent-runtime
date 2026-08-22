# Artifact-refiner deterministic replay

Date: 2026-08-18

Iteration 1 retained the initial independent critic/judge BLOCK. Iteration 2
retired the legacy routine CI workflow and verified the stale standalone Rust
lockfile reconciliation, but the critic proved the runtime-first publication
claim incomplete. Iteration 3 records all four path-only runtime prerequisites
and makes no publishable-now claim.

Final pre-finalization schema/reference replay observed exit 0:

```text
artifact_manifest.json schema PASS
constraints.json schema PASS
state.json schema PASS
referenced file PASS: .refiner/artifacts/resolve-sdk-distribution/dist/verification-summary.md
blocking constraint definitions PASS: 3/3
state consistency PASS
checkpoint references PASS: 15
```

Independent critic and judge both passed iteration 3.

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh resolve-sdk-distribution
```

Observed output, exit 0:

```text
.refiner/history/resolve-sdk-distribution/2026-08-18_20-42-45Z
```

Post-finalization replay validated both active and history snapshots. Observed
output, exit 0:

```text
artifact: .refiner/artifacts/resolve-sdk-distribution
artifact_manifest.json schema PASS
constraints.json schema PASS
state.json schema PASS
finalized state PASS
checkpoint references PASS: 15
artifact: .refiner/history/resolve-sdk-distribution/2026-08-18_20-42-45Z
artifact_manifest.json schema PASS
constraints.json schema PASS
state.json schema PASS
finalized state PASS
checkpoint references PASS: 15
```
