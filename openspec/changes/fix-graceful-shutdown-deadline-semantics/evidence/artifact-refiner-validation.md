# Artifact-refiner validation

Date: 2026-08-22

The installed adapter pointed to absent canonical files, so the repository's
vendored canonical artifact-refiner at
`crates/prometheus-skill-system/skills/imported/artifact-refiner` was used.
State was intentionally contained under this OpenSpec evidence tree to avoid
modifying the unrelated dirty root `.refiner` registry and history.

Provider resolution observed:

```json
{"provider_type":"filesystem","config":{"state_directory":".refiner","scope":"project"}}
```

The refinement ran one progressive Specify, Plan, Execute, Reflect, Persist
iteration for a direct-code artifact. The constraints schema does not enumerate
`code`, so `constraints.json` uses its schema-valid `content` discriminator;
the state and manifest retain `code`. This schema mismatch is disclosed rather
than patched in the vendored skill.

Final deterministic replay validated the canonical JSON schemas, phase
chronology, blocking-constraint identity, manifest reference, finalized
registry entry, byte-identical active/history copies, and strict OpenSpec.

Observed output, exit `0`:

```text
state.json: PASS
constraints.json: PASS
artifact_manifest.json: PASS
checkpoint-specify: PASS phases=1
checkpoint-plan: PASS phases=2
checkpoint-execute: PASS phases=3
checkpoint-reflect: PASS phases=4
checkpoint-persist: PASS phases=5
constraints-and-references: PASS 5 constraints, 1 reference
registry: PASS finalized
active-history-identity: PASS files=13
Change 'fix-graceful-shutdown-deadline-semantics' is valid
```

The converged artifact explicitly leaves the parent 10,800-second
certification pending and makes no cross-profile, release, or runtime-level
claim.
