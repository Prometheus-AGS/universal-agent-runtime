# Artifact-refiner validation

Date: 2026-08-20

Command:

```bash
python - <<'PY'
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker

root = Path('.refiner/artifacts/fix-pnpm-lock-submodule-consistency')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
for name, schema_name in (
    ('artifact_manifest.json', 'artifact-manifest.schema.json'),
    ('constraints.json', 'constraints.schema.json'),
    ('state.json', 'refinement-state.schema.json'),
):
    Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    print(f'SCHEMA_PASS={name}')
for checkpoint in sorted((root / 'checkpoints').glob('*.json')):
    Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    print(f'CHECKPOINT_SCHEMA_PASS={checkpoint.name}')
manifest = json.loads((root / 'artifact_manifest.json').read_text())
for variant in manifest['variants']:
    path = Path(variant['file'])
    assert path.is_file()
    print(f'REFERENCE_PASS={path}')
constraint_ids = [item['id'] for item in json.loads((root / 'constraints.json').read_text())['constraints']]
state_ids = [item['id'] for item in json.loads((root / 'state.json').read_text())['constraints']]
assert constraint_ids == state_ids
print(f'CONSTRAINT_IDS_PASS={len(constraint_ids)}')
expected = {
    '46384bce.json': (1, ['specify']),
    '86b09ed2.json': (1, ['specify', 'plan']),
    '685de206.json': (1, ['specify', 'plan', 'execute']),
    '7b66e6eb.json': (1, ['specify', 'plan', 'execute', 'reflect']),
    '9ce57cf7.json': (2, ['execute']),
}
for path in (root / 'checkpoints').glob('*.json'):
    item = json.loads(path.read_text())
    assert (item['current_iteration'], item['phases_completed']) == expected[path.name]
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')
PY
shasum -a 256 pnpm-lock.yaml \
  openspec/changes/fix-pnpm-lock-submodule-consistency/verification.md \
  openspec/changes/fix-pnpm-lock-submodule-consistency/specs/frontend-build-tooling/spec.md
openspec validate fix-pnpm-lock-submodule-consistency --strict --no-interactive
git diff --check -- pnpm-lock.yaml \
  openspec/changes/fix-pnpm-lock-submodule-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency \
  .refiner/artifacts/fix-pnpm-lock-submodule-consistency
echo DIFF_CHECK_PASS
```

Observed exit: `0`

Observed output:

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=46384bce.json
CHECKPOINT_SCHEMA_PASS=685de206.json
CHECKPOINT_SCHEMA_PASS=7b66e6eb.json
CHECKPOINT_SCHEMA_PASS=86b09ed2.json
CHECKPOINT_SCHEMA_PASS=9ce57cf7.json
REFERENCE_PASS=.refiner/artifacts/fix-pnpm-lock-submodule-consistency/dist/verification-summary.md
CONSTRAINT_IDS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=5
645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350  pnpm-lock.yaml
58c2bf5d692a6351b562df68cca7523e74a50964f5985401ecbbf0ad1a3e889e  openspec/changes/fix-pnpm-lock-submodule-consistency/verification.md
43d94f90b42cd36ec0d203f14b14e5c359293fc191e0d0cb5e1d06d52535a5d5  openspec/changes/fix-pnpm-lock-submodule-consistency/specs/frontend-build-tooling/spec.md
Change 'fix-pnpm-lock-submodule-consistency' is valid
DIFF_CHECK_PASS
```

The checkpoint set preserves the first BLOCK review as iteration 1 Reflect and
the corrected candidate as iteration 2 Execute. It does not claim convergence
before fresh independent review.

## Final convergence and persistence replay

Command:

```bash
python - <<'PY'
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker

root = Path('.refiner/artifacts/fix-pnpm-lock-submodule-consistency')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
for name, schema_name in (
    ('artifact_manifest.json', 'artifact-manifest.schema.json'),
    ('constraints.json', 'constraints.schema.json'),
    ('state.json', 'refinement-state.schema.json'),
):
    Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    print(f'SCHEMA_PASS={name}')
for checkpoint in sorted((root / 'checkpoints').glob('*.json')):
    Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    print(f'CHECKPOINT_SCHEMA_PASS={checkpoint.name}')
state = json.loads((root / 'state.json').read_text())
constraints = json.loads((root / 'constraints.json').read_text())
assert [item['id'] for item in state['constraints']] == [item['id'] for item in constraints['constraints']]
print('CONSTRAINT_IDS_PASS=5')
assert state['convergence_status'] == 'converged'
assert state['iteration_history'][-1]['constraints_satisfied'] == 5
print('CONVERGENCE_PASS=5/5')
print(f'CHECKPOINT_COUNT_PASS={len(state["checkpoints"])}')
PY
diff -qr \
  .refiner/artifacts/fix-pnpm-lock-submodule-consistency \
  .refiner/history/fix-pnpm-lock-submodule-consistency/2026-08-20_17-25-32Z
find .refiner/artifacts/fix-pnpm-lock-submodule-consistency -type f | wc -l
find .refiner/history/fix-pnpm-lock-submodule-consistency/2026-08-20_17-25-32Z -type f | wc -l
openspec validate fix-pnpm-lock-submodule-consistency --strict --no-interactive
git diff --check -- pnpm-lock.yaml \
  openspec/changes/fix-pnpm-lock-submodule-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-pnpm-lock-submodule-consistency \
  .refiner/artifacts/fix-pnpm-lock-submodule-consistency \
  .refiner/history/fix-pnpm-lock-submodule-consistency
echo FINAL_REFINER_PASS
```

Observed exit: `0`

Observed output:

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=46384bce.json
CHECKPOINT_SCHEMA_PASS=5fbf1c39.json
CHECKPOINT_SCHEMA_PASS=685de206.json
CHECKPOINT_SCHEMA_PASS=7b66e6eb.json
CHECKPOINT_SCHEMA_PASS=86b09ed2.json
CHECKPOINT_SCHEMA_PASS=9363445e.json
CHECKPOINT_SCHEMA_PASS=9ce57cf7.json
CONSTRAINT_IDS_PASS=5
CONVERGENCE_PASS=5/5
CHECKPOINT_COUNT_PASS=7
15
15
Change 'fix-pnpm-lock-submodule-consistency' is valid
FINAL_REFINER_PASS
```

Fresh corrected-candidate review result: artifact critic `PASS`; independent
artifact judge `PASS`. The 15 active files and 15 history files are byte-identical.
