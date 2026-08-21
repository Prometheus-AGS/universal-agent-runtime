# Artifact-refiner validation — `fix-mcp-reconnect-shared-service-state`

Date: 2026-08-21
Artifact: `.refiner/artifacts/fix-mcp-reconnect-shared-service-state`
History: `.refiner/history/fix-mcp-reconnect-shared-service-state/2026-08-21_13-53-23Z`

The command below validates the final active artifact and persisted history
without invoking Cargo, a browser, or GitHub Actions.

## Command

```bash
python3 - <<'PY'
import hashlib
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker

root = Path('.')
active = root / '.refiner/artifacts/fix-mcp-reconnect-shared-service-state'
history = root / '.refiner/history/fix-mcp-reconnect-shared-service-state/2026-08-21_13-53-23Z'
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
schema_map = {
    'artifact_manifest.json': 'artifact-manifest.schema.json',
    'constraints.json': 'constraints.schema.json',
    'state.json': 'refinement-state.schema.json',
}
for name, schema_name in schema_map.items():
    schema = json.loads((schemas / schema_name).read_text())
    instance = json.loads((active / name).read_text())
    Draft7Validator(schema, format_checker=FormatChecker()).validate(instance)
    print(f'SCHEMA_PASS={name}')
state_schema = json.loads((schemas / 'refinement-state.schema.json').read_text())
checkpoints = sorted((active / 'checkpoints').glob('*.json'))
for checkpoint in checkpoints:
    Draft7Validator(state_schema, format_checker=FormatChecker()).validate(
        json.loads(checkpoint.read_text())
    )
print(f'CHECKPOINT_SCHEMAS_PASS={len(checkpoints)}')

manifest = json.loads((active / 'artifact_manifest.json').read_text())
references = []
for variant in manifest['variants']:
    references.append(Path(variant['file']))
    references.extend(Path(item) for item in variant.get('files', []))
assert all(path.is_file() and path.stat().st_size > 0 for path in references)
print(f'MANIFEST_REFERENCES_PASS={len(references)}')

constraints = json.loads((active / 'constraints.json').read_text())['constraints']
state = json.loads((active / 'state.json').read_text())
assert state['constraints'] == constraints
for checkpoint in checkpoints:
    assert json.loads(checkpoint.read_text())['constraints'] == constraints
print(f'CONSTRAINT_OBJECTS_PASS={len(constraints)}')

expected = {
    '416b85b4.json': (1, ['specify'], 0),
    '33669e2a.json': (1, ['specify', 'plan'], 1),
    'aac0bd4e.json': (1, ['specify', 'plan', 'execute'], 2),
    '79f6d143.json': (2, ['specify', 'plan', 'execute'], 3),
    'b3a33295.json': (2, ['specify', 'plan', 'execute', 'reflect'], 4),
    'ef1a7add.json': (2, ['specify', 'plan', 'execute', 'reflect', 'persist'], 5),
}
for checkpoint in checkpoints:
    item = json.loads(checkpoint.read_text())
    assert (item['current_iteration'], item['phases_completed'], len(item['checkpoints'])) == expected[checkpoint.name]
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')

assert state['convergence_status'] == 'converged'
assert state['iteration_history'][-1]['decision'] == 'terminate'
assert state['iteration_history'][-1]['constraints_satisfied'] == len(constraints)
print(f'CONVERGENCE_PASS={len(constraints)}/{len(constraints)}')

registry = json.loads((root / '.refiner/registry.json').read_text())['artifacts']
entry = registry['fix-mcp-reconnect-shared-service-state']
assert entry == {
    'path': '.refiner/artifacts/fix-mcp-reconnect-shared-service-state',
    'artifact_type': 'content',
    'content_type': 'direct:content',
    'updated_at': '2026-08-21T13:53:23Z',
    'finalized_at': '2026-08-21T13:53:23Z',
}
print('REGISTRY_IDENTITY_PASS=fix-mcp-reconnect-shared-service-state')

expected_hashes = {
    'src/mcp/registry.rs': '7222a8826be0a99640dcc8570bd34b8c0fab0e8b16d5245999f0fa2c8bcf78d8',
    'scripts/certify-release-candidate.sh': '99cc348efd56f3062da878699bbff0f3fe58d66ce9e25efcf338661446035fb9',
    'scripts/validate-mcp-process-boundary-evidence.mjs': '4390b36500e4a671f538a27196f219e46c666975fb98898d8fb77d4e8467d6f0',
    'scripts/validate-candidate-certification.mjs': 'd3fafec3dc8d4da8fc4f83ae875fb29b192dd232cb920d31d29151bcd54ed214',
    'scripts/validate-candidate-certification-workflow.mjs': 'e974be9e8d010e0e102a8ed4c330f38cdd0e5bd0cc2d4b30df1034f51613de82',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/verification.md': '7b10da070e0558c29d12f8d3789a54a65b02c5a4b032f0df19deab0ff3e594fd',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/installed-preflight-results.json': 'ddea79152623addeb0df85a6b0366c4d272af812f243cbb3a42588d604546de3',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/lifecycle.json': '491ecb309ed031e1f981d80f86893754e43387e9227f94c3646498c14bd1b547',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/failure-recovery.jsonl': '75719769b897fa50b0bb26dd530d6dce374f05e05f686898c281ef33d628b0ff',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/mcp-process-boundary.json': '3867c71a77be386df3a6d803a22153c41a33b17b1e9666df613f0f0a871549ec',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/mcp-process-boundary-validation.txt': '6f4abcbfe9709048425165465b55213f546c9f3c0eeeab5021d8c5a238a1f2fe',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/mcp-process-trace.jsonl': 'cb425ffda2be2b59b50b129d853df14c85ec273da5af8584143537a651fdb0f9',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/mcp-crash.sse': '372c9dfe0bb59173a79aaf82162b552ebdb758a0b3ddeddd4a455c209c1eca4d',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/mcp-timeout.sse': 'f155719e0a4593efe34af0901bd76631034aa25404baeb32f7bfcaf6b6a75ae8',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/parallel-load.json': '1bfd70b7941f75d9dbb6933f0f96a247e53b6dd6b2fec024a32d88ea2ab16ae5',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/soak.json': '54893a9f70d83c7d14a4eef67242471c885b62ae5f0318ba26dc05539087b96d',
    'openspec/changes/archive/2026-08-21-fix-mcp-reconnect-shared-service-state/evidence/upgrade.json': 'a7cfca5cedb7dd3e11a905fd0e7f10eebe0e9929edb019ae24f60f7dde840f8d',
}
for relative, expected_hash in expected_hashes.items():
    actual = hashlib.sha256((root / relative).read_bytes()).hexdigest()
    assert actual == expected_hash, (relative, actual, expected_hash)
print(f'HASH_RECEIPTS_PASS={len(expected_hashes)}')

def tree_digest(path):
    digest = hashlib.sha256()
    files = sorted(item for item in path.rglob('*') if item.is_file())
    for item in files:
        digest.update(item.relative_to(path).as_posix().encode())
        digest.update(b'\0')
        digest.update(item.read_bytes())
        digest.update(b'\0')
    return digest.hexdigest(), files

active_digest, active_files = tree_digest(active)
history_digest, history_files = tree_digest(history)
assert active_digest == history_digest
assert [item.relative_to(active) for item in active_files] == [item.relative_to(history) for item in history_files]
print(f'ACTIVE_HISTORY_RELATIVE_TREE_MATCH={active_digest}')
print(f'ACTIVE_HISTORY_FILE_COUNT={len(active_files)}')
PY
```

## Observed output

Exit status: 0.

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMAS_PASS=6
MANIFEST_REFERENCES_PASS=16
CONSTRAINT_OBJECTS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=6
CONVERGENCE_PASS=5/5
REGISTRY_IDENTITY_PASS=fix-mcp-reconnect-shared-service-state
HASH_RECEIPTS_PASS=17
ACTIVE_HISTORY_RELATIVE_TREE_MATCH=f7d8c02ad0666883b1a9ef66433ede05937d59dfc528d9f4104488e998216ec8
ACTIVE_HISTORY_FILE_COUNT=14
```

## Limits

This receipt validates the artifact-refiner structure, references, recorded
hashes, chronology, convergence, registry identity, and active/history byte
identity. It does not rerun Cargo, the installed preflight, the parent three-hour
soak, deployment validation, or GA promotion.
