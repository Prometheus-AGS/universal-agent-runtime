# A2 artifact-refiner deterministic replay

Date: 2026-08-15

The direct-content artifact at
`.refiner/artifacts/gap-03-a2a-tenant-partitioning` was replayed against the
vendored manifest, constraints, and state schemas. The replay also verified the
referenced summary and four-constraint state consistency.

Observed output (exit 0):

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/gap-03-a2a-tenant-partitioning/dist/verification-summary.md
blocking constraints PASS: 4/4
state consistency PASS
```

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh gap-03-a2a-tenant-partitioning
```

Observed output (exit 0):

```text
.refiner/history/gap-03-a2a-tenant-partitioning/2026-08-15_09-50-40Z
```
