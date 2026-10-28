# TEAM-01 result-handling repair — execution packet 003

Status: PREPARED, NOT EXECUTED. No signing amendment: the existing sole exit criterion and limits remain unchanged. Repair23/28 is consumed; all prior consumption is retained. Frozen source manifest SHA256 8b30ef7996d8a16ae56196991b5613e903f891a738534cdd71ec5d0d757a7a24. Sole acceptance: the existing math/writing concurrency and merged-article BDD on Mac. Contract fd4bfb892b805f980295aafa26259b427ee5cee30af2b6891e4ec7ea37263d76 remains unchanged. This is task 1.4, after implementation tasks 1.1–1.3 and authorized follow-ups 1.5 and 1.6.

Approve one serial pass through the following items. Stop at the first blocking review finding, failed command, unknown effect, expired authority or exhausted allowance. No automatic retry or repair. Preserve all previous consumption and failures; 4,800 calls / 3,000 active minutes / 28 repairs and separate $10 embedding caps remain. This packet requests no hosted embedding work; existing startup can attempt requests to the fixture’s disabled loopback embedding endpoint. No external embedding provider is selected by this packet. Liter/MiniMax-M3 uses the operator's subscription.

## 1. Review and source binding

Fresh source-only code reviewer with Rust expertise, then JavaScript reviewer, serially, inspect handoffs/result-bdd-repair.patch and handoffs/result-fixture-repair.patch with current BDD/configuration and directly called source. Inspect the configured-cap path in manager.rs, orchestrator.rs and liter_driver.rs as current artifacts; no producer diagnostic handoff. Review final-answer extraction, unchanged TEAM-01 outcomes and literal activation commands. No Rust production source changed. Use a source-only critic role without mandatory Cargo prerequisites for Rust. No producer history, gotchas/team memory or other reviewer reports; no product commands or source edits. Read-only source access is permitted. Save reports under review/execute-003/. Stop on blocking findings. These support the same scenario, not a new acceptance suite.

Compare the frozen source manifest and confirm the two-file delta supporting reuse of the already-built binary, using this exact repository-root command:

```sh
python3 - <<'PY_SOURCE'
import hashlib, json, os
from pathlib import Path
p = Path('.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agent-team-concurrency-repair')
manifest = json.loads((p / 'source-manifest-003.json').read_text())
current = set()
for entry in manifest['roots']:
    path = Path(entry)
    if path.is_file():
        current.add(str(path))
    elif path.is_dir():
        for folder, directories, names in os.walk(path):
            directories[:] = sorted(d for d in directories if d not in manifest['excluded_directory_names'])
            current.update(str(Path(folder) / name) for name in names if (Path(folder) / name).is_file() and name != '.DS_Store')
assert current == {row['path'] for row in manifest['files']}, 'Source input membership changed'
for row in manifest['files']:
    assert hashlib.sha256(Path(row['path']).read_bytes()).hexdigest() == row['sha256'], row['path']
previous_path = p / 'source-manifest-002.json'
assert hashlib.sha256(previous_path.read_bytes()).hexdigest() == '45371cdc1495244d02eff575a37825b351188d1b70c9e368ab555a2156ae9b1a'
previous = json.loads(previous_path.read_text())
assert manifest['roots'] == previous['roots'] and manifest['excluded_directory_names'] == previous['excluded_directory_names']
old = {row['path']: row['sha256'] for row in previous['files']}
new = {row['path']: row['sha256'] for row in manifest['files']}
assert old.keys() == new.keys()
assert {path for path in old if old[path] != new[path]} == {
    'scripts/behaviors/steps/math-writing-team.steps.mjs',
    'target/prompt-macos-hbitxj2c/config.json',
}, 'Binary reuse requires unchanged build inputs'
print('Frozen source inputs match; only the approved BDD and fixture configuration changed')
PY_SOURCE
```

## 2. Bind the existing executable before restart

Packet002 successfully built the Rust source, which is unchanged. Reuse that binary; do not run Cargo. The source comparison above permits only the BDD parser and retained configuration difference. This command verifies the recorded build identity and binary hash before restart, without a process probe.

```sh
python3 - <<'PY_IDENTITY'
import hashlib, json
from pathlib import Path
p = Path('.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agent-team-concurrency-repair')
launch = json.loads(Path('target/prompt-macos-launch.json').read_text())
assert launch['source_manifest_sha256'] == '45371cdc1495244d02eff575a37825b351188d1b70c9e368ab555a2156ae9b1a'
assert launch['built_source_identity'] == 'sha256:' + launch['source_manifest_sha256']
assert hashlib.sha256(Path(launch['binary']).read_bytes()).hexdigest() == launch['binary_sha256']
with (p / 'evidence/prerestart-server-identity-003.json').open('x') as receipt:
    json.dump(launch, receipt, indent=2)
print('Existing packet002 executable identity recorded; no rebuild')
PY_IDENTITY
```

## 3. Replace only the owned UAR, preserving the existing fixture

Repository cwd. The command below checks the retained PID's exact executable/config, verifies owner JWT lifetime before stopping it, requests graceful SIGINT, waits up to 60 seconds for exit, checks open handles on the exact store, then copies the closed store to a fresh backup. It launches the candidate with the same private config/cwd and provider environment. This expressly authorizes the named process/store/readiness checks; no other preflight is implied. No automatic force-kill, rollback or database replacement. Keep the candidate running after the scenario. If activation fails, retain its logs/state and discuss.

```sh
python3 - <<'PY_ACTIVATE'
import base64, hashlib, json, os, shutil, signal, subprocess, time, urllib.request, urllib.error
from pathlib import Path
root = Path.cwd()
child = root / '.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agent-team-concurrency-repair'
launch = json.loads((root / 'target/prompt-macos-launch.json').read_text())
config = Path(launch['config'])
fixture = config.parent
store = fixture / 'store.db'
backup = fixture / 'store.before-team-repair-003'
binary = root / 'target/uar-team-concurrency-repair/debug/universal-agent-runtime'
assert binary.is_file() and not backup.exists()
previous = json.loads((child / 'evidence/prerestart-server-identity-003.json').read_text())
assert launch == previous, 'Owned launch metadata changed after pre-restart capture'
support = json.loads((root / 'target/uar-user-outcomes-007/supporting/supporting.env.json').read_text())
token = support['UAR_BOUNDARY_OWNER_AUTHORIZATION'].removeprefix('Bearer ')
payload = token.split('.')[1]
expiry = json.loads(base64.urlsafe_b64decode(payload + '=' * (-len(payload) % 4)))['exp']
assert expiry > time.time() + 900, 'Owner authority needs renewal before stopping UAR'
command = subprocess.check_output(['ps', '-p', str(launch['pid']), '-o', 'command='], text=True).strip()
assert command == launch['binary'] + ' --config ' + str(config), 'Retained process identity changed'
os.kill(launch['pid'], signal.SIGINT)
for _ in range(120):
    if subprocess.run(['ps', '-p', str(launch['pid']), '-o', 'pid='], stdout=subprocess.DEVNULL).returncode != 0:
        break
    time.sleep(0.5)
else:
    raise SystemExit('Old UAR did not exit; stop without force-kill')
handles = subprocess.run(['/usr/sbin/lsof', '+D', str(store)], capture_output=True, text=True)
assert handles.returncode == 1 and not handles.stdout.strip() and not handles.stderr.strip(), 'Store release unconfirmed'
shutil.copytree(store, backup)
private = json.loads((root / 'target/uar-user-outcomes-007/private/uar_fixture_inputs_5d3ed9f4d4218e05b533/env.json').read_text())
env = {k: v for k, v in os.environ.items() if not k.startswith('UAR_') and k not in ['LLM_MODEL', 'LLM_BASE_URL', 'LLM_API_KEY']}
env['UAR_PROMPT_PROVIDER_KEY'] = private['LITER_LLM_MASTER_KEY']
logpath = fixture / 'server-team-repair-003.log'
with logpath.open('xb') as log:
    process = subprocess.Popen([str(binary), '--config', str(config)], cwd=fixture, env=env,
        stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
receipt = {**launch, 'pid': process.pid, 'binary': str(binary),
    'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'log': str(logpath),
    'source_manifest_sha256': previous['source_manifest_sha256'],
    'built_source_identity': previous['built_source_identity'],
    'verification_source_manifest_sha256': hashlib.sha256((child / 'source-manifest-003.json').read_bytes()).hexdigest(),
    'config_sha256': hashlib.sha256(config.read_bytes()).hexdigest(),
    'backup': str(backup), 'status': 'starting'}
(child / 'evidence').mkdir(exist_ok=True)
(child / 'evidence/activation-003.json').write_text(json.dumps(receipt, indent=2) + '\n')
for _ in range(60):
    assert process.poll() is None, 'Candidate exited; inspect retained log'
    try:
        with urllib.request.urlopen(launch['base_url'] + '/readyz', timeout=2) as response:
            ready = response.status == 200
    except (OSError, urllib.error.URLError):
        ready = False
    if ready:
        break
    time.sleep(1)
else:
    raise SystemExit('Candidate did not become ready; no BDD run')
receipt['status'] = 'ready'
(child / 'evidence/activation-003.json').write_text(json.dumps(receipt, indent=2) + '\n')
(root / 'target/prompt-macos-launch.json').write_text(json.dumps(receipt, indent=2) + '\n')
print('Repaired UAR ready; PID', process.pid)
PY_ACTIVATE
```

The saved config selects liter/MiniMax-M3 and the existing SurrealKV store, with its model output maximum changed from512 to64000. The running process retains its prior in-memory configuration until this approved restart. Source-level admission narrows the request to this model maximum; the configured context window remains unchanged. Built-source provenance remains packet002; the new verification manifest/config hash is recorded separately. No key value is printed or changed. The previous store backup and old failed team remain retained. Startup uses the existing maintenance/reconciliation path. The PID/config mismatch check fails closed rather than killing a different process. The 15-minute JWT margin covers activation plus the single bounded scenario; expired authority is a stop, not permission to mint a token.

## 4. Run only the existing BDD

Repository cwd. This wrapper supplies the retained private owner/workspace authority without exposing it, then runs the exact Cucumber command. Node remains the cross-platform scenario implementation. No other scenario is selected; retry=0.

```sh
python3 - <<'PY_BDD'
import json, os, subprocess
from pathlib import Path
root = Path.cwd()
support = json.loads((root / 'target/uar-user-outcomes-007/supporting/supporting.env.json').read_text())
launch = json.loads((root / 'target/prompt-macos-launch.json').read_text())
env = dict(os.environ)
env.update(UAR_AUTH_TOKEN=support['UAR_BOUNDARY_OWNER_AUTHORIZATION'].removeprefix('Bearer '),
    UAR_WORKSPACE_ID='uar-user-outcomes-007-supporting-workspace', UAR_MODEL='liter/MiniMax-M3',
    UAR_BASE_URL=launch['base_url'], UAR_CUCUMBER_REPORT='.prometheus/verification/team-repair-003-macos.cucumber.json')
report = root / env['UAR_CUCUMBER_REPORT']
assert not report.exists(), 'Preserve prior result; no automatic retry'
result = subprocess.run(['node', 'node_modules/@cucumber/cucumber/bin/cucumber.js',
    '--config', 'scripts/behaviors/team.cucumber.mjs'], env=env)
raise SystemExit(result.returncode)
PY_BDD
```

Expected effects: one retained package/binding/team with two specialists, calculation and outline tasks running concurrently, then one dependent article task. Three agent task attempts with genuine inference; any existing tool-loop model calls remain subject to the same cumulative per-attempt budget. The authorized response ceiling is 64,000 tokens plus a separate 64,000 input allowance: 128,000 tokens per attempt and 384,000 tokens for three attempts. Existing120second/1USD per-attempt and360second/3USD team limits stay unchanged. The input allowance is an explicit host allocation with estimated context sizing, not proof of exact provider tokenization or guaranteed model fit. These reservation ceilings are not a claim of actual spend. Existing HTTP polls, run-registration reads, result artifacts, context and settled-call assertions are part of this one BDD. No separate fault scenario or earlier certified behavior is rerun.

Both previously failed BDD receipts and their logs remain unchanged. The last run recorded succeeded specialist attempts, but the math parser could not read JSON with surrounding text and the writer returned reasoning without a finished outline. This packet keeps arithmetic/identity/overlap/merge/accounting assertions and explicitly rejects reasoning-only editorial output. It does not infer success from task status. This is one proposed rerun of the failed TEAM-01 gate after the result-extraction and fixture correction. There is no automatic retry. New candidate startup preserves the same agents/KBs/store. Source-only review and the owned restart are proposed; the previously built binary is reused because Rust/build inputs are unchanged. No compiler or other test command is included.

## Result and clean handoff

On PASS only: record actual source/binary/config identity, overlapping run IDs, two successful artifacts and final article/context receipt in parent-results.md; reconcile canonical task1.4/change/Execute; sync/archive only repair-agent-team-concurrent-operation using installed OpenSpec/KBD commands; Reflect then canonical child exit to uar-verification-simplification. Do not start another parent scenario. On failure leave the child incomplete, preserve receipts and discuss fix versus backlog. Earlier evidence is never overwritten or promoted to a repaired-source PASS.

The uncomfortable limit: the saved512-token clamp is proven by source/configuration, but retained evidence has no final wire request or finish_reason; successful final output remains unverified. This gateway remains settings-only. The host can enforce the outbound output ceiling and reserve its explicit allowance, but its local input sizing is estimated and must not be presented as an exact model-token count or fit guarantee. Provider-reported actual usage and unresolved liability remain authoritative. The earlier lost CAS cause also remains historical; only the real BDD establishes whether this workflow succeeds. No claim of runtime success is made by this packet.
