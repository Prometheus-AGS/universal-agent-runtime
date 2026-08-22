# Artifact-refiner deterministic replay

Date: 2026-08-18

Command, run from the worktree root before finalization:

```bash
python3 - <<'PY'
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.')
base = root / 'crates/prometheus-skill-system/skills/imported/artifact-refiner/references/schemas'
artifact = root / '.refiner/artifacts/harden-jwt-defaults'
for label, schema_name, instance_name in [
    ('manifest', 'artifact-manifest.schema.json', 'artifact_manifest.json'),
    ('constraints', 'constraints.schema.json', 'constraints.json'),
    ('state', 'refinement-state.schema.json', 'state.json'),
]:
    schema = json.loads((base / schema_name).read_text())
    instance = json.loads((artifact / instance_name).read_text())
    errors = list(Draft7Validator(schema, format_checker=FormatChecker()).iter_errors(instance))
    if errors:
        raise SystemExit(f'{label} FAIL: ' + '; '.join(error.message for error in errors))
    print(f'{label} schema PASS')
manifest = json.loads((artifact / 'artifact_manifest.json').read_text())
for variant in manifest['variants']:
    referenced = root / variant['file']
    if not referenced.is_file() or referenced.stat().st_size == 0:
        raise SystemExit(f'referenced file FAIL: {referenced}')
    print(f'referenced file PASS: {referenced}')
constraints = json.loads((artifact / 'constraints.json').read_text())['constraints']
state = json.loads((artifact / 'state.json').read_text())
if len(constraints) != 4 or state['iteration_history'][-1]['constraints_satisfied'] != len(constraints):
    raise SystemExit('constraint/state consistency FAIL')
print('blocking constraints PASS: 4/4')
print('state consistency PASS')
PY
```

Observed output, exit 0:

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/harden-jwt-defaults/dist/verification-summary.md
blocking constraints PASS: 4/4
state consistency PASS
```

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh harden-jwt-defaults
```

Observed output, exit 0:

```text
.refiner/history/harden-jwt-defaults/2026-08-18_16-57-19Z
```

The same schema and consistency replay then observed exit 0 with:

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/harden-jwt-defaults/dist/verification-summary.md
blocking constraints PASS: 4/4
finalized state PASS
```
