# Artifact-refiner validation

Date: 2026-08-20

The installed adapter lacked its referenced canonical prompts and schemas. This
run uses the canonical schema copies vendored by
`prometheus-skill-pack/skills/imported/artifact-refiner` and the accepted
direct-content contract from the immediately preceding lock child.

## Pre-review progressive-state validation

Command:

```bash
python - <<'PY'
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.refiner/artifacts/fix-frontend-pnpm-lock-consistency')
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
    'c4f0f8bb.json': ['specify'],
    '2d55c8cc.json': ['specify', 'plan'],
    'dfa99139.json': ['specify', 'plan', 'execute'],
}
for name, phases in expected.items():
    item = json.loads((root / 'checkpoints' / name).read_text())
    assert item['current_iteration'] == 1 and item['phases_completed'] == phases
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')
state = json.loads((root / 'state.json').read_text())
assert state['convergence_status'] == 'running'
print('PRE_REVIEW_STATE_PASS=running')
PY
shasum -a 256 frontend/pnpm-lock.yaml \
  openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md \
  openspec/changes/fix-frontend-pnpm-lock-consistency/specs/frontend-build-tooling/spec.md \
  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/head-candidate-delta-audit.md \
  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/frozen-install-verification.md
openspec validate fix-frontend-pnpm-lock-consistency --strict --no-interactive
git diff --check -- frontend/pnpm-lock.yaml \
  openspec/changes/fix-frontend-pnpm-lock-consistency \
  .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency \
  .refiner/artifacts/fix-frontend-pnpm-lock-consistency
echo PRE_REVIEW_REFINER_PASS
```

Observed exit: `0`

Observed output:

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=2d55c8cc.json
CHECKPOINT_SCHEMA_PASS=c4f0f8bb.json
CHECKPOINT_SCHEMA_PASS=dfa99139.json
REFERENCE_PASS=.refiner/artifacts/fix-frontend-pnpm-lock-consistency/dist/verification-summary.md
CONSTRAINT_IDS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=3
PRE_REVIEW_STATE_PASS=running
43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593  frontend/pnpm-lock.yaml
18dfc79e04e3bb916e5af127b0612426e64038c9d9531117a98dd6a0c6e1cb70  openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md
f074b0356429f93cc9e4ab835005c7dd6ae5712f2bf0d4b968c53a600466b308  openspec/changes/fix-frontend-pnpm-lock-consistency/specs/frontend-build-tooling/spec.md
e465ac0617188dd800a7756d3ba04cf9604312a5d717f6a275d3eff55e3d5d98  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/head-candidate-delta-audit.md
72b900fe26ee476d0f4e113a691710f1096d8bb681e09bb3408d108921028c9d  openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/frozen-install-verification.md
Change 'fix-frontend-pnpm-lock-consistency' is valid
PRE_REVIEW_REFINER_PASS
```

The state is deliberately `running`. Convergence and active/history persistence
will be recorded only after history-free critic and judge review.

## Iteration-2 corrected-state validation

Command:

```bash
set -euo pipefail
python - <<'PY'
import hashlib
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.refiner/artifacts/fix-frontend-pnpm-lock-consistency')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
schema_map = {
    'artifact_manifest.json': 'artifact-manifest.schema.json',
    'constraints.json': 'constraints.schema.json',
    'state.json': 'refinement-state.schema.json',
}
for name, schema_name in schema_map.items():
    Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    print(f'SCHEMA_PASS={name}')
checkpoints = sorted((root / 'checkpoints').glob('*.json'))
for checkpoint in checkpoints:
    Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    print(f'CHECKPOINT_SCHEMA_PASS={checkpoint.name}')
manifest = json.loads((root / 'artifact_manifest.json').read_text())
for variant in manifest['variants']:
    referenced = Path(variant['file'])
    assert referenced.is_file()
    print(f'REFERENCE_PASS={referenced}')
constraint_ids = [item['id'] for item in json.loads((root / 'constraints.json').read_text())['constraints']]
state = json.loads((root / 'state.json').read_text())
state_ids = [item['id'] for item in state['constraints']]
assert constraint_ids == state_ids
assert state['current_iteration'] == 2
assert state['phases_completed'] == ['execute']
assert state['convergence_status'] == 'running'
assert state['iteration_history'] == [{
    'iteration': 1,
    'phases_completed': ['specify', 'plan', 'execute', 'reflect'],
    'constraints_satisfied': 2,
    'constraints_total': 5,
    'decision': 'continue',
    'timestamp': '2026-08-20T20:41:14Z',
}]
expected = [
    ('c4f0f8bb', 'specify', 1),
    ('2d55c8cc', 'plan', 1),
    ('dfa99139', 'execute', 1),
    ('20c5bf6b', 'reflect', 1),
    ('360e8d2b', 'execute', 2),
]
assert [(row['checkpoint_id'], row['phase'], row['iteration']) for row in state['checkpoints']] == expected
for checkpoint_id, phase, iteration in expected:
    item = json.loads((root / 'checkpoints' / f'{checkpoint_id}.json').read_text())
    assert item['current_iteration'] == iteration
    assert item['phases_completed'][-1] == phase
assert hashlib.sha256((root / 'dist/verification-summary.md').read_bytes()).hexdigest() == '03682cc896121cbba953fd78b16ea88d11d8c22725d0dc2408078e2be609b5ed'
assert hashlib.sha256((root / 'state.json').read_bytes()).hexdigest() == '7f9c0159167796dab0a11e528342a1f8a982afd41c4efc54b9506ad6fa9c3930'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md').read_bytes()).hexdigest() == '5f7743e8adcbaf28355657ed3d05828d71e1a415c84f5b869d5cf3ebfc6d9e1e'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json').read_bytes()).hexdigest() == '01382bd8b0fc008f67141f218ed9e5d6fa2d99daf1c5456ae53492ca8b24ecee'
print(f'CONSTRAINT_IDS_PASS={len(constraint_ids)}')
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')
print('CURRENT_HASHES_PASS=4')
print('ITERATION_2_RUNNING_STATE_PASS')
PY
openspec validate fix-frontend-pnpm-lock-consistency --strict --no-interactive
git diff --check -- frontend/pnpm-lock.yaml openspec/specs/frontend-build-tooling/spec.md
node <<'NODE'
const { execFileSync } = require('node:child_process');
const { readFileSync } = require('node:fs');
const roots = [
  'openspec/changes/fix-frontend-pnpm-lock-consistency',
  '.refiner/artifacts/fix-frontend-pnpm-lock-consistency',
  '.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency',
];
const files = execFileSync('git', ['ls-files', '--others', '--exclude-standard', '-z', '--', ...roots]).toString('utf8').split('\0').filter(Boolean);
for (const file of files) {
  const buffer = readFileSync(file);
  if (buffer.includes(0)) continue;
  const text = buffer.toString('utf8');
  if (!file.endsWith('.patch') && /(^|\n)[^\n]*[ \t]+(?=\n|$)/.test(text)) throw new Error(`trailing whitespace: ${file}`);
  if (/(^|\n)(<<<<<<<|=======|>>>>>>>)( |$)/.test(text)) throw new Error(`conflict marker: ${file}`);
}
console.log(`UNTRACKED_TEXT_FILES_INSPECTED=${files.length}`);
console.log('UNTRACKED_DIFF_CHECK_PASS');
NODE
echo ITERATION_2_REFINER_VALIDATION_PASS
```

Observed exit: `0`

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=20c5bf6b.json
CHECKPOINT_SCHEMA_PASS=2d55c8cc.json
CHECKPOINT_SCHEMA_PASS=360e8d2b.json
CHECKPOINT_SCHEMA_PASS=c4f0f8bb.json
CHECKPOINT_SCHEMA_PASS=dfa99139.json
REFERENCE_PASS=.refiner/artifacts/fix-frontend-pnpm-lock-consistency/dist/verification-summary.md
CONSTRAINT_IDS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=5
CURRENT_HASHES_PASS=4
ITERATION_2_RUNNING_STATE_PASS
Change 'fix-frontend-pnpm-lock-consistency' is valid
UNTRACKED_TEXT_FILES_INSPECTED=39
UNTRACKED_DIFF_CHECK_PASS
ITERATION_2_REFINER_VALIDATION_PASS
```

Iteration 1 remains in the receipt as historical evidence of the state the
critic reviewed. This section validates the corrected current variant and its
new chronological checkpoints; it does not rewrite or relabel the older hash.

## Iteration-3 causal-classification validation

Command:

```bash
set -euo pipefail
python - <<'PY'
import hashlib
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.refiner/artifacts/fix-frontend-pnpm-lock-consistency')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
for name, schema_name in {
    'artifact_manifest.json': 'artifact-manifest.schema.json',
    'constraints.json': 'constraints.schema.json',
    'state.json': 'refinement-state.schema.json',
}.items():
    Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    print(f'SCHEMA_PASS={name}')
for checkpoint in sorted((root / 'checkpoints').glob('*.json')):
    Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    print(f'CHECKPOINT_SCHEMA_PASS={checkpoint.name}')
state = json.loads((root / 'state.json').read_text())
constraint_ids = [item['id'] for item in json.loads((root / 'constraints.json').read_text())['constraints']]
assert constraint_ids == [item['id'] for item in state['constraints']]
assert state['current_iteration'] == 3
assert state['phases_completed'] == ['execute']
assert state['convergence_status'] == 'running'
expected = [
    ('c4f0f8bb', 'specify', 1),
    ('2d55c8cc', 'plan', 1),
    ('dfa99139', 'execute', 1),
    ('20c5bf6b', 'reflect', 1),
    ('360e8d2b', 'execute', 2),
    ('0ce3bc7a', 'reflect', 2),
    ('1d80847b', 'execute', 3),
]
assert [(row['checkpoint_id'], row['phase'], row['iteration']) for row in state['checkpoints']] == expected
for checkpoint_id, phase, iteration in expected:
    item = json.loads((root / 'checkpoints' / f'{checkpoint_id}.json').read_text())
    assert item['current_iteration'] == iteration
    assert item['phases_completed'][-1] == phase
manifest = json.loads((root / 'artifact_manifest.json').read_text())
for variant in manifest['variants']:
    referenced = Path(variant['file'])
    assert referenced.is_file()
    print(f'REFERENCE_PASS={referenced}')
assert hashlib.sha256((root / 'dist/verification-summary.md').read_bytes()).hexdigest() == 'f31dd5e20a9f0f4810fcbd45758a0f50cf0ff5508fcb32541ff58fdef5c6b820'
assert hashlib.sha256((root / 'state.json').read_bytes()).hexdigest() == 'f2fc89435fc8e4dc8c88f57a44f49dd595344d7afa1d4f7fdb026b2871ddd84f'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md').read_bytes()).hexdigest() == 'c189498613812679355838e5aafb1956040dec0db245094e27d9d43d798f2320'
classification = json.loads(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json').read_text())
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json').read_bytes()).hexdigest() == '68d18449d5cd1454995645ce1bdc43b4ae3d7cac7935ccba4c980364d951fe38'
assert classification['unclassifiedMutationCount'] == 0
peer = [row for row in classification['mutations'] if row['classification'] == 'peer-context-propagation']
assert all(row['causalAnchors'] for row in peer)
assert all(len(row['causalAnchors']) != len(classification['directManifestEdges']) for row in peer)
assert any(
    change['token'] == 'yup@1.7.1' and
    any(anchor.endswith('#devDependencies.@cucumber/cucumber') for anchor in change['causalAnchors'])
    for row in classification['mutations']
    for change in row.get('contextChanges') or []
)
print(f'CONSTRAINT_IDS_PASS={len(constraint_ids)}')
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')
print(f'PEER_CONTEXT_RECORDS_PASS={len(peer)}')
print('CURRENT_HASHES_PASS=4')
print('ITERATION_3_RUNNING_STATE_PASS')
PY
openspec validate fix-frontend-pnpm-lock-consistency --strict --no-interactive
git diff --check -- frontend/pnpm-lock.yaml openspec/specs/frontend-build-tooling/spec.md
node <<'NODE'
const { execFileSync } = require('node:child_process');
const { readFileSync } = require('node:fs');
const roots = [
  'openspec/changes/fix-frontend-pnpm-lock-consistency',
  '.refiner/artifacts/fix-frontend-pnpm-lock-consistency',
  '.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency',
];
const files = execFileSync('git', ['ls-files', '--others', '--exclude-standard', '-z', '--', ...roots]).toString('utf8').split('\0').filter(Boolean);
for (const file of files) {
  const buffer = readFileSync(file);
  if (buffer.includes(0)) continue;
  const text = buffer.toString('utf8');
  if (!file.endsWith('.patch') && /(^|\n)[^\n]*[ \t]+(?=\n|$)/.test(text)) throw new Error(`trailing whitespace: ${file}`);
  if (/(^|\n)(<<<<<<<|=======|>>>>>>>)( |$)/.test(text)) throw new Error(`conflict marker: ${file}`);
}
console.log(`UNTRACKED_TEXT_FILES_INSPECTED=${files.length}`);
console.log('UNTRACKED_DIFF_CHECK_PASS');
NODE
echo ITERATION_3_REFINER_VALIDATION_PASS
```

Observed exit: `0`

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=0ce3bc7a.json
CHECKPOINT_SCHEMA_PASS=1d80847b.json
CHECKPOINT_SCHEMA_PASS=20c5bf6b.json
CHECKPOINT_SCHEMA_PASS=2d55c8cc.json
CHECKPOINT_SCHEMA_PASS=360e8d2b.json
CHECKPOINT_SCHEMA_PASS=c4f0f8bb.json
CHECKPOINT_SCHEMA_PASS=dfa99139.json
REFERENCE_PASS=.refiner/artifacts/fix-frontend-pnpm-lock-consistency/dist/verification-summary.md
CONSTRAINT_IDS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=7
PEER_CONTEXT_RECORDS_PASS=131
CURRENT_HASHES_PASS=4
ITERATION_3_RUNNING_STATE_PASS
Change 'fix-frontend-pnpm-lock-consistency' is valid
UNTRACKED_TEXT_FILES_INSPECTED=41
UNTRACKED_DIFF_CHECK_PASS
ITERATION_3_REFINER_VALIDATION_PASS
```

Iteration 3 is still `running`: convergence and persistence remain gated on a
fresh independent critic and judge PASS for this exact variant.

## Iteration-4 manifest-anchor validation

Command:

```bash
set -euo pipefail
python - <<'PY'
import hashlib
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
root = Path('.refiner/artifacts/fix-frontend-pnpm-lock-consistency')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
for name, schema_name in {'artifact_manifest.json':'artifact-manifest.schema.json','constraints.json':'constraints.schema.json','state.json':'refinement-state.schema.json'}.items():
    Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    print(f'SCHEMA_PASS={name}')
for checkpoint in sorted((root / 'checkpoints').glob('*.json')):
    Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    print(f'CHECKPOINT_SCHEMA_PASS={checkpoint.name}')
state = json.loads((root / 'state.json').read_text())
expected = [('c4f0f8bb','specify',1),('2d55c8cc','plan',1),('dfa99139','execute',1),('20c5bf6b','reflect',1),('360e8d2b','execute',2),('0ce3bc7a','reflect',2),('1d80847b','execute',3),('631ba6b5','reflect',3),('e13e8027','execute',4)]
assert state['current_iteration'] == 4 and state['phases_completed'] == ['execute'] and state['convergence_status'] == 'running'
assert [(row['checkpoint_id'],row['phase'],row['iteration']) for row in state['checkpoints']] == expected
for checkpoint_id, phase, iteration in expected:
    item = json.loads((root / 'checkpoints' / f'{checkpoint_id}.json').read_text())
    assert item['current_iteration'] == iteration and item['phases_completed'][-1] == phase
constraint_ids = [item['id'] for item in json.loads((root / 'constraints.json').read_text())['constraints']]
assert constraint_ids == [item['id'] for item in state['constraints']]
manifest = json.loads((root / 'artifact_manifest.json').read_text())
for variant in manifest['variants']:
    referenced = Path(variant['file']); assert referenced.is_file(); print(f'REFERENCE_PASS={referenced}')
assert hashlib.sha256((root / 'dist/verification-summary.md').read_bytes()).hexdigest() == '6d5f0cf2b1c8b5b2775c3ce39dffa88454e00c85395dc7f90c4593efae541acb'
assert hashlib.sha256((root / 'state.json').read_bytes()).hexdigest() == 'de5b6a82f7329ac2a6143de8244b0eda84b79c059ddcb24283c24c293ad0c0c0'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md').read_bytes()).hexdigest() == 'a45d589f20c03bc828e886548904d99571357431b98ad75ee34aa81ea7fd91b8'
classification_path = Path('openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json')
assert hashlib.sha256(classification_path.read_bytes()).hexdigest() == 'e986720672df17d0c2c826e6b42fa630554d0405cff68b1866a6703818d2ce87'
classification = json.loads(classification_path.read_text())
for edge in classification['directManifestEdgeDetails']:
    file, selector = edge['anchor'].split('#')
    section, dependency = selector.split('.', 1)
    package = json.loads(Path(file).read_text())
    assert dependency in package.get(section, {})
    if edge['candidateImporterSpecifier'] is not None:
        assert package[section][dependency] == edge['candidateImporterSpecifier']
print(f'CONSTRAINT_IDS_PASS={len(constraint_ids)}')
print(f'CHECKPOINT_CHRONOLOGY_PASS={len(expected)}')
print(f'DIRECT_MANIFEST_EDGE_VALIDATION_PASS={len(classification["directManifestEdgeDetails"])}')
print('CURRENT_HASHES_PASS=4')
print('ITERATION_4_RUNNING_STATE_PASS')
PY
openspec validate fix-frontend-pnpm-lock-consistency --strict --no-interactive
git diff --check -- frontend/pnpm-lock.yaml openspec/specs/frontend-build-tooling/spec.md
node <<'NODE'
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs');
const roots=['openspec/changes/fix-frontend-pnpm-lock-consistency','.refiner/artifacts/fix-frontend-pnpm-lock-consistency','.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency'];
const files=execFileSync('git',['ls-files','--others','--exclude-standard','-z','--',...roots]).toString('utf8').split('\0').filter(Boolean);
for(const file of files){const buffer=readFileSync(file);if(buffer.includes(0))continue;const text=buffer.toString('utf8');if(!file.endsWith('.patch')&&/(^|\n)[^\n]*[ \t]+(?=\n|$)/.test(text))throw new Error(`trailing whitespace: ${file}`);if(/(^|\n)(<<<<<<<|=======|>>>>>>>)( |$)/.test(text))throw new Error(`conflict marker: ${file}`)}
console.log(`UNTRACKED_TEXT_FILES_INSPECTED=${files.length}`);console.log('UNTRACKED_DIFF_CHECK_PASS');
NODE
echo ITERATION_4_REFINER_VALIDATION_PASS
```

Observed exit: `0`

```text
SCHEMA_PASS=artifact_manifest.json
SCHEMA_PASS=constraints.json
SCHEMA_PASS=state.json
CHECKPOINT_SCHEMA_PASS=0ce3bc7a.json
CHECKPOINT_SCHEMA_PASS=1d80847b.json
CHECKPOINT_SCHEMA_PASS=20c5bf6b.json
CHECKPOINT_SCHEMA_PASS=2d55c8cc.json
CHECKPOINT_SCHEMA_PASS=360e8d2b.json
CHECKPOINT_SCHEMA_PASS=631ba6b5.json
CHECKPOINT_SCHEMA_PASS=c4f0f8bb.json
CHECKPOINT_SCHEMA_PASS=dfa99139.json
CHECKPOINT_SCHEMA_PASS=e13e8027.json
REFERENCE_PASS=.refiner/artifacts/fix-frontend-pnpm-lock-consistency/dist/verification-summary.md
CONSTRAINT_IDS_PASS=5
CHECKPOINT_CHRONOLOGY_PASS=9
DIRECT_MANIFEST_EDGE_VALIDATION_PASS=44
CURRENT_HASHES_PASS=4
ITERATION_4_RUNNING_STATE_PASS
Change 'fix-frontend-pnpm-lock-consistency' is valid
UNTRACKED_TEXT_FILES_INSPECTED=43
UNTRACKED_DIFF_CHECK_PASS
ITERATION_4_REFINER_VALIDATION_PASS
```

Iteration 4 remains `running` until both reviewers pass the actual-manifest
anchor correction.

## Final convergence and persistence validation

Command:

```bash
set -euo pipefail
python - <<'PY'
import hashlib
import json
from pathlib import Path
from jsonschema import Draft7Validator, FormatChecker
active = Path('.refiner/artifacts/fix-frontend-pnpm-lock-consistency')
history = Path('.refiner/history/fix-frontend-pnpm-lock-consistency/2026-08-20_21-07-18Z')
schemas = Path('/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/imported/artifact-refiner/references/schemas')
for root in (active, history):
    for name, schema_name in {'artifact_manifest.json':'artifact-manifest.schema.json','constraints.json':'constraints.schema.json','state.json':'refinement-state.schema.json'}.items():
        Draft7Validator(json.loads((schemas / schema_name).read_text()), format_checker=FormatChecker()).validate(json.loads((root / name).read_text()))
    checkpoints = sorted((root / 'checkpoints').glob('*.json'))
    assert len(checkpoints) == 11
    for checkpoint in checkpoints:
        Draft7Validator(json.loads((schemas / 'refinement-state.schema.json').read_text()), format_checker=FormatChecker()).validate(json.loads(checkpoint.read_text()))
    state = json.loads((root / 'state.json').read_text())
    assert state['current_iteration'] == 4
    assert state['phases_completed'] == ['execute', 'reflect', 'persist']
    assert state['convergence_status'] == 'converged'
    assert state['iteration_history'][-1]['constraints_satisfied'] == 5
    assert state['iteration_history'][-1]['constraints_total'] == 5
    assert state['iteration_history'][-1]['decision'] == 'terminate'
    for variant in json.loads((root / 'artifact_manifest.json').read_text())['variants']:
        assert Path(variant['file']).is_file()
assert hashlib.sha256((active / 'dist/verification-summary.md').read_bytes()).hexdigest() == 'db8d6d80cd4367c13f08540db15dbed37e93d6d655542589f21d4c06a70f1a90'
assert hashlib.sha256((active / 'state.json').read_bytes()).hexdigest() == '40317ff219dd533a1359ade9103ce0fedd5605f1009eb831183413796109e952'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/verification.md').read_bytes()).hexdigest() == 'aba41bf91c1f964bd3663ca5dfd8da6d39cd98b86284fcd9c0b54cac0dc5b44b'
assert hashlib.sha256(Path('openspec/changes/fix-frontend-pnpm-lock-consistency/evidence/lock-delta-classification.json').read_bytes()).hexdigest() == 'e986720672df17d0c2c826e6b42fa630554d0405cff68b1866a6703818d2ce87'
print('ACTIVE_SCHEMAS_PASS=14')
print('HISTORY_SCHEMAS_PASS=14')
print('CONVERGED_CONSTRAINTS_PASS=5')
print('FINAL_HASHES_PASS=4')
PY
diff -qr .refiner/artifacts/fix-frontend-pnpm-lock-consistency .refiner/history/fix-frontend-pnpm-lock-consistency/2026-08-20_21-07-18Z
test "$(find .refiner/artifacts/fix-frontend-pnpm-lock-consistency -type f | wc -l | tr -d ' ')" = 19
test "$(find .refiner/history/fix-frontend-pnpm-lock-consistency/2026-08-20_21-07-18Z -type f | wc -l | tr -d ' ')" = 19
openspec validate fix-frontend-pnpm-lock-consistency --strict --no-interactive
git diff --check -- frontend/pnpm-lock.yaml openspec/specs/frontend-build-tooling/spec.md
node <<'NODE'
const {execFileSync}=require('node:child_process'),{readFileSync}=require('node:fs');
const roots=['openspec/changes/fix-frontend-pnpm-lock-consistency','.refiner/artifacts/fix-frontend-pnpm-lock-consistency','.refiner/history/fix-frontend-pnpm-lock-consistency','.kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/children/fix-frontend-pnpm-lock-consistency'];
const files=execFileSync('git',['ls-files','--others','--exclude-standard','-z','--',...roots]).toString('utf8').split('\0').filter(Boolean);
for(const file of files){const buffer=readFileSync(file);if(buffer.includes(0))continue;const text=buffer.toString('utf8');if(!file.endsWith('.patch')&&/(^|\n)[^\n]*[ \t]+(?=\n|$)/.test(text))throw new Error(`trailing whitespace: ${file}`);if(/(^|\n)(<<<<<<<|=======|>>>>>>>)( |$)/.test(text))throw new Error(`conflict marker: ${file}`)}
console.log(`UNTRACKED_TEXT_FILES_INSPECTED=${files.length}`);console.log('UNTRACKED_DIFF_CHECK_PASS');
NODE
echo ACTIVE_HISTORY_IDENTITY_PASS
echo FINAL_REFINER_VALIDATION_PASS
```

Observed exit: `0`

```text
ACTIVE_SCHEMAS_PASS=14
HISTORY_SCHEMAS_PASS=14
CONVERGED_CONSTRAINTS_PASS=5
FINAL_HASHES_PASS=4
Change 'fix-frontend-pnpm-lock-consistency' is valid
UNTRACKED_TEXT_FILES_INSPECTED=64
UNTRACKED_DIFF_CHECK_PASS
ACTIVE_HISTORY_IDENTITY_PASS
FINAL_REFINER_VALIDATION_PASS
```

The registry file is intentionally excluded because it contains unrelated
operator/parent dirt. This child proves active/history identity directly and
does not rewrite the shared registry to manufacture agreement.
