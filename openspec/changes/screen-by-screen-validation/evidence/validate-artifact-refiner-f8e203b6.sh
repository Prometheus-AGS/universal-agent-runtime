#!/usr/bin/env bash
set -euo pipefail

active=".refiner/artifacts/screen-by-screen-validation"
history=".refiner/history/screen-by-screen-validation/2026-08-21_05-53-37Z"
schema_root="crates/prometheus-skill-system/skills/imported/artifact-refiner/references/schemas"
registry=".refiner/registry.json"

python3 - "$active" "$history" "$schema_root" "$registry" <<'PY'
import hashlib
import json
import re
import sys
from pathlib import Path

from jsonschema import Draft7Validator, FormatChecker

active = Path(sys.argv[1])
history = Path(sys.argv[2])
schema_root = Path(sys.argv[3])
registry_path = Path(sys.argv[4])

def validate(instance_path: Path, schema_name: str) -> None:
    instance = json.loads(instance_path.read_text())
    schema = json.loads((schema_root / schema_name).read_text())
    Draft7Validator(schema, format_checker=FormatChecker()).validate(instance)

validate(active / 'artifact_manifest.json', 'artifact-manifest.schema.json')
validate(active / 'constraints.json', 'constraints.schema.json')
validate(active / 'state.json', 'refinement-state.schema.json')
print('ACTIVE_SCHEMAS_PASS=3')

state = json.loads((active / 'state.json').read_text())
expected_phases = ['specify', 'plan', 'execute', 'reflect', 'persist']
assert state['convergence_status'] == 'converged'
assert state['phases_completed'] == expected_phases
assert state['current_iteration'] == 1
assert state['iteration_history'] == [{
    'iteration': 1,
    'phases_completed': expected_phases,
    'constraints_satisfied': 5,
    'constraints_total': 5,
    'decision': 'terminate',
    'timestamp': state['iteration_history'][0]['timestamp'],
}]
print('STATE_CONVERGED_PASS')

checkpoint_ids = []
checkpoint_times = []
for index, item in enumerate(state['checkpoints']):
    assert item['phase'] == expected_phases[index]
    checkpoint = active / 'checkpoints' / f"{item['checkpoint_id']}.json"
    assert checkpoint.is_file()
    validate(checkpoint, 'refinement-state.schema.json')
    snapshot = json.loads(checkpoint.read_text())
    assert snapshot['phases_completed'] == expected_phases[:index + 1]
    assert snapshot['current_iteration'] == 1
    assert snapshot['constraints'] == json.loads(
        (active / 'constraints.json').read_text()
    )['constraints']
    checkpoint_ids.append(item['checkpoint_id'])
    checkpoint_times.append(item['timestamp'])
assert len(checkpoint_ids) == 5
assert checkpoint_times == sorted(checkpoint_times)
assert len(list((active / 'checkpoints').glob('*.json'))) == 5
print('CHECKPOINT_SCHEMAS_REFERENCES_CHRONOLOGY_PASS=5')

constraints = json.loads((active / 'constraints.json').read_text())['constraints']
constraint_ids = [item['id'] for item in constraints]
assert constraint_ids == [
    'screen-evidence-complete',
    'interaction-strength',
    'memory-and-fail-closed-controls',
    'bundle-process-source-integrity',
    'scope-process-and-truth',
]
assert state['constraints'] == constraints
print('CONSTRAINT_OBJECTS_MATCH=5')

registry = json.loads(registry_path.read_text())
assert registry['artifacts']['screen-by-screen-validation'] == {
    'path': '.refiner/artifacts/screen-by-screen-validation',
    'artifact_type': 'content',
    'content_type': 'direct:content',
    'updated_at': '2026-08-21T05:53:37Z',
    'finalized_at': '2026-08-21T05:53:37Z',
}
print('REGISTRY_ARTIFACT_IDENTITY_MATCH=screen-by-screen-validation')

manifest = json.loads((active / 'artifact_manifest.json').read_text())
references = []
for variant in manifest['variants']:
    if isinstance(variant.get('file'), str):
        references.append(variant['file'])
    references.extend(variant.get('files', []))
assert len(references) == 5
assert all(Path(reference).is_file() for reference in references)
print('MANIFEST_REFERENCES_PASS=5')

summary = (active / 'dist/verification-summary.md').read_text()
receipts = {
    'openspec/changes/screen-by-screen-validation/evidence/validation-matrix.md': 'Matrix',
    'docs/certifications/product-screens/f8e203b6/manifest.json': 'Bundle manifest',
    'docs/certifications/product-screens/f8e203b6/cucumber-report.json': 'Cucumber report',
    'docs/certifications/product-screens/f8e203b6/certification-run.log': 'Certification transcript',
    'frontend/src/features/knowledge/ui/knowledge-page.tsx': 'Knowledge screen',
    'frontend/src/platform/agui/runtime-chunk-projection.ts': 'Approval projection',
    'tests/bdd/steps/product-screen-validation.steps.ts': 'Product-screen steps',
    'tests/bdd/steps/cross-screen-security.steps.ts': 'Security steps',
    'tests/bdd/steps/local-first-resilience.steps.ts': 'Local-first steps',
}
for path, label in receipts.items():
    digest = hashlib.sha256(Path(path).read_bytes()).hexdigest()
    assert f'- {label} SHA-256: `{digest}`' in summary
print(f'SUMMARY_HASH_RECEIPTS_MATCH={len(receipts)}')

def tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob('*') if item.is_file()):
        relative = path.relative_to(root).as_posix()
        digest.update(relative.encode())
        digest.update(b'\0')
        digest.update(hashlib.sha256(path.read_bytes()).digest())
        digest.update(b'\0')
    return digest.hexdigest()

active_digest = tree_digest(active)
history_digest = tree_digest(history)
assert active_digest == history_digest
active_files = sorted(path.relative_to(active) for path in active.rglob('*') if path.is_file())
history_files = sorted(path.relative_to(history) for path in history.rglob('*') if path.is_file())
assert active_files == history_files
for relative in active_files:
    assert (active / relative).read_bytes() == (history / relative).read_bytes()
print(f'ACTIVE_HISTORY_RELATIVE_TREE_MATCH={active_digest}')
print(f'ACTIVE_HISTORY_FILE_COUNT={len(active_files)}')
PY
