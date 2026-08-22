# B4 artifact-refiner deterministic replay

Date: 2026-08-15

Command, run from the `uar-1-0-readiness` worktree root:

```bash
python3 - <<'PY'
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.')
base = root / 'crates/prometheus-skill-system/skills/imported/artifact-refiner/references/schemas'
artifact = root / '.refiner/artifacts/skill-scoped-governance'
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

Observed output (exit 0):

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/skill-scoped-governance/dist/verification-summary.md
blocking constraints PASS: 4/4
state consistency PASS
```

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh skill-scoped-governance
```

Observed output (exit 0):

```text
.refiner/history/skill-scoped-governance/2026-08-15_07-34-40Z
```

## Iteration 2 replay after adversarial corrections

The same deterministic replay command was run after the cold-restart,
compatibility, deletion, and literal-evidence corrections.

Observed output (exit 0):

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/skill-scoped-governance/dist/verification-summary.md
blocking constraints PASS: 4/4
state consistency PASS
```

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh skill-scoped-governance
```

Observed output (exit 0):

```text
.refiner/history/skill-scoped-governance/2026-08-15_07-51-01Z
```

## Iteration 3 replay after matching-compatibility correction

The same deterministic replay command was run after the pending-binding
matching fallback and its focused assertion were added.

Observed output (exit 0):

```text
manifest schema PASS
constraints schema PASS
state schema PASS
referenced file PASS: .refiner/artifacts/skill-scoped-governance/dist/verification-summary.md
blocking constraints PASS: 4/4
state consistency PASS
```

Finalization command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh skill-scoped-governance
```

Observed output (exit 0):

```text
.refiner/history/skill-scoped-governance/2026-08-15_08-03-54Z
```
