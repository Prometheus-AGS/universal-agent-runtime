# Persist — entity-graph-crud

Finalize and store session outcomes.

## 1. Merge artifacts

- Ensure all new files are saved on disk with correct paths.
- Remove dead experimental files if superseded.

## 2. Optional state file

From workspace root:

```bash
bash prometheus-entity-skills/entity-graph-crud/scripts/state-init.sh .
```

Update `.entity-graph-skills/entity-graph-crud/state.json` fields:

- `current_phase`: `persist`
- `phases_completed`: include `specify`, `plan`, `execute`, `reflect`, `persist`
- `artifacts`: list of file paths touched
- `entity_spec` / `plan`: summaries or pointers to docs in repo

## 3. Handoff notes

Write 3–6 bullets for the human maintainer:

- Entity type + list key shape
- API base paths
- Relation schemas registered (or explicit “none”)
- Known limitations (e.g. partial list rows if list fetch omits fields — use `detailFetch` or `useEntity` as needed; engine GC is configurable — see library `configureEngine`.)

## 4. Orchestrator follow-up (optional)

If `detect-orchestrators.sh` reported available frameworks, suggest next automation (e.g. OpenSpec change, KBD phase) **without** blocking merge.

## Done when

Code is on disk, typecheck green, state file updated or intentionally skipped, handoff notes present in PR description or internal doc.
