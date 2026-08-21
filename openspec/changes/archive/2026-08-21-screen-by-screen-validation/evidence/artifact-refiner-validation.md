# Artifact Refiner validation — `screen-by-screen-validation`

Profile: `direct:content`; filesystem state provider. This receipt validates
only the final `f8e203b6` screen-certification artifact.

## Command

```bash
bash openspec/changes/screen-by-screen-validation/evidence/validate-artifact-refiner-f8e203b6.sh
```

## Observed output

```text
ACTIVE_SCHEMAS_PASS=3
STATE_CONVERGED_PASS
CHECKPOINT_SCHEMAS_REFERENCES_CHRONOLOGY_PASS=5
CONSTRAINT_OBJECTS_MATCH=5
REGISTRY_ARTIFACT_IDENTITY_MATCH=screen-by-screen-validation
MANIFEST_REFERENCES_PASS=5
SUMMARY_HASH_RECEIPTS_MATCH=9
ACTIVE_HISTORY_RELATIVE_TREE_MATCH=f5c709da5cecfff10aed33f39a925b3bfc7e57d19e4af02de21ceab6778e1757
ACTIVE_HISTORY_FILE_COUNT=11
```

Exit status: 0.

The validator uses the checked-in Artifact Refiner manifest, constraints, and
state schemas. It validates all five progressive checkpoint snapshots, exact
constraint objects, registry artifact identity, manifest references, nine
source/evidence hashes, and byte
identity between the active artifact and history snapshot
`.refiner/history/screen-by-screen-validation/2026-08-21_05-53-37Z`.

## Uncomfortable fact

A path-sensitive directory digest cannot prove active/history equality because
the two roots have different path prefixes. This receipt instead hashes relative
paths plus file content and then compares every relative file byte-for-byte.
The validator also compares complete constraint objects; matching IDs alone is
insufficient because descriptions and validation methods are load-bearing. A
committed active/history pair without its registry identity is likewise not a
persisted artifact.
