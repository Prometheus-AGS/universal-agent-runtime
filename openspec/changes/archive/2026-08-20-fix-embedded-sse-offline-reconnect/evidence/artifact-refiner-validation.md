# Artifact-refiner validation — `fix-embedded-sse-offline-reconnect`

Date: 2026-08-20
Artifact: `.refiner/artifacts/fix-embedded-sse-offline-reconnect`
History: `.refiner/history/fix-embedded-sse-offline-reconnect/2026-08-20_09-15-12Z`

The final gate validated the manifest, constraints, final state, every
checkpoint, manifest references, exact constraint identity, chronological
phase progression, convergence, registry identity, and active/history byte
identity. The schema CLI printed only its upstream deprecation warning and
exited 0 for each invocation.

```bash
ACTIVE=.refiner/artifacts/fix-embedded-sse-offline-reconnect
HISTORY=.refiner/history/fix-embedded-sse-offline-reconnect/2026-08-20_09-15-12Z
SCHEMAS=/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas
jsonschema -i "$ACTIVE/artifact_manifest.json" "$SCHEMAS/artifact-manifest.schema.json"
jsonschema -i "$ACTIVE/constraints.json" "$SCHEMAS/constraints.schema.json"
jsonschema -i "$ACTIVE/state.json" "$SCHEMAS/refinement-state.schema.json"
for checkpoint in "$ACTIVE"/checkpoints/*.json; do
  jsonschema -i "$checkpoint" "$SCHEMAS/refinement-state.schema.json"
done
# Read-only Python assertions replayed manifest references, five exact
# constraint IDs, five progressive checkpoints and their prior-reference
# counts, converged/terminate state, registry identity, and every active/history
# path and byte.
```

Observed output:

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMAS_PASS=5
MANIFEST_REFERENCES_PASS=1
CONSTRAINT_IDS_MATCH=5
PROGRESSIVE_CHECKPOINTS_MATCH=5
CONVERGENCE_STATUS=converged
REGISTRY_IDENTITY_PASS=1
ACTIVE_HISTORY_FILES_MATCH=13
```

The first iteration's five checkpoint files were deleted after independent
review proved they all contained the same phase-complete state. Iteration 3 is
the retained correction cycle: Specify contains only Specify, then Plan,
Execute, Reflect, and Persist each add exactly one phase and one prior
checkpoint reference. Schema validity is therefore paired with chronological
semantic validation.
