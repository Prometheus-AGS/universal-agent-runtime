# TEAM-01 token-reservation repair — execution packet 002

Status: PREPARED, NOT EXECUTED. Frozen source manifest SHA256 45371cdc1495244d02eff575a37825b351188d1b70c9e368ab555a2156ae9b1a. Sole acceptance: the existing math/writing concurrency and merged-article BDD on Mac. Contract fd4bfb892b805f980295aafa26259b427ee5cee30af2b6891e4ec7ea37263d76 remains unchanged. This is task 1.4, after implementation tasks 1.1–1.3 and authorized follow-up 1.5.

Approve one serial pass through the following items. Stop at the first blocking review finding, failed command, unknown effect, expired authority or exhausted allowance. No automatic retry or repair. Preserve all previous consumption and failures; 4,800 calls / 3,000 active minutes / 28 repairs and separate $10 embedding caps remain. This packet requests no hosted embedding work; existing startup can attempt requests to the fixture’s disabled loopback embedding endpoint. No external embedding provider is selected by this packet. Liter/MiniMax-M3 uses the operator's subscription.

## 1. Review and source binding

Fresh source-only code reviewer with Rust expertise, then JavaScript reviewer, serially, inspect handoffs/token-budget-repair.patch and handoffs/bdd-token-repair.patch with current affected files and directly called source. Review this repair, the unchanged TEAM-01 oracle and literal activation commands. Use a source-only critic role without mandatory Cargo prerequisites for Rust. No producer history, gotchas/team memory or other reviewer reports; no product commands or source edits. Read-only source access is permitted. Save reports under review/execute-002/. Stop on blocking findings. These support the same scenario, not a new acceptance suite.

Compare the frozen source manifest before building, using this exact repository-root command:

```sh
python3 - <<'PY_SOURCE'
import hashlib, json, os
from pathlib import Path
p = Path('.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agent-team-concurrency-repair')
manifest = json.loads((p / 'source-manifest-002.json').read_text())
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
print('Frozen source inputs match')
PY_SOURCE
```

## 2. Build the repaired native server

Before the build, record the currently running server artifact’s disk identity. The same Cargo target directory is reused, so building will replace that path; its pre-build hash must not be mistaken for the new file’s hash afterward. This does not stop or probe the process. Startup PID/config identity is checked separately in step3.

```sh
python3 - <<'PY_PREBUILD'
import hashlib, json
from pathlib import Path
p = Path('.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/children/uar-verification-simplification/children/agent-team-concurrency-repair')
launch = json.loads(Path('target/prompt-macos-launch.json').read_text())
assert hashlib.sha256(Path(launch['binary']).read_bytes()).hexdigest() == launch['binary_sha256']
with (p / 'evidence/prebuild-server-identity-002.json').open('x') as receipt:
    json.dump(launch, receipt, indent=2)
print('Previous server artifact identity recorded before rebuilding')
PY_PREBUILD

Repository cwd; one build writer. Retain stdout/stderr and exit code in evidence/build-002.log and evidence/build-002-result.json under this child. No lint, formatting, unit suites, platform builds or additional acceptance scripts.

```sh
CARGO_TARGET_DIR=target/uar-team-concurrency-repair cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime
```

Read the Rust workspace skill's phase-gated verification reference before this first Cargo command. Do not update a lockfile or install anything if the command fails. Keep the current UAR running while the build runs.

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
backup = fixture / 'store.before-team-repair-002'
binary = root / 'target/uar-team-concurrency-repair/debug/universal-agent-runtime'
assert binary.is_file() and not backup.exists()
previous = json.loads((child / 'evidence/prebuild-server-identity-002.json').read_text())
assert launch == previous, 'Owned launch metadata changed after pre-build capture'
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
logpath = fixture / 'server-team-repair-002.log'
with logpath.open('xb') as log:
    process = subprocess.Popen([str(binary), '--config', str(config)], cwd=fixture, env=env,
        stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
receipt = {**launch, 'pid': process.pid, 'binary': str(binary),
    'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(), 'log': str(logpath),
    'source_manifest_sha256': hashlib.sha256((child / 'source-manifest-002.json').read_bytes()).hexdigest(),
    'built_source_identity': 'sha256:' + hashlib.sha256((child / 'source-manifest-002.json').read_bytes()).hexdigest(),
    'backup': str(backup), 'status': 'starting'}
(child / 'evidence').mkdir(exist_ok=True)
(child / 'evidence/activation-002.json').write_text(json.dumps(receipt, indent=2) + '\n')
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
(child / 'evidence/activation-002.json').write_text(json.dumps(receipt, indent=2) + '\n')
(root / 'target/prompt-macos-launch.json').write_text(json.dumps(receipt, indent=2) + '\n')
print('Repaired UAR ready; PID', process.pid)
PY_ACTIVATE
```

The active config already selects liter/MiniMax-M3 and the existing SurrealKV store. No key value is printed or changed. The previous store backup and old failed team remain retained. Startup uses the existing maintenance/reconciliation path. The PID/config mismatch check fails closed rather than killing a different process. The 15-minute JWT margin covers activation plus the single bounded scenario; expired authority is a stop, not permission to mint a token.

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
    UAR_BASE_URL=launch['base_url'], UAR_CUCUMBER_REPORT='.prometheus/verification/team-repair-002-macos.cucumber.json')
report = root / env['UAR_CUCUMBER_REPORT']
assert not report.exists(), 'Preserve prior result; no automatic retry'
result = subprocess.run(['node', 'node_modules/@cucumber/cucumber/bin/cucumber.js',
    '--config', 'scripts/behaviors/team.cucumber.mjs'], env=env)
raise SystemExit(result.returncode)
PY_BDD
```

Expected effects: one retained package/binding/team with two specialists, calculation and outline tasks running concurrently, then one dependent article task. Three genuine model turns; operator-authorized response ceiling64,000tokens plus a separate64,000input allowance:128,000tokens per attempt and384,000tokens for three attempts. Existing120second/1USD per-attempt and360second/3USD team limits stay unchanged. The input allowance is an explicit host allocation with estimated context sizing, not proof of exact provider tokenization or guaranteed model fit. These reservation ceilings are not a claim of actual spend. Existing HTTP polls, run-registration reads, result artifacts, context and settled-call assertions are part of this one BDD. No separate fault scenario or earlier certified behavior is rerun.

The previously failed BDD and its admission logs remain unchanged. This is one approved rerun of the failed TEAM-01 gate after source correction. There is no automatic retry. New candidate startup preserves the same agents/KBs/store. Source-only review and the necessary rebuild/owned restart are repeated because the running binary has the previous source.

## Result and clean handoff

On PASS only: record actual source/binary/config identity, overlapping run IDs, two successful artifacts and final article/context receipt in parent-results.md; reconcile canonical task1.4/change/Execute; sync/archive only repair-agent-team-concurrent-operation using installed OpenSpec/KBD commands; Reflect then canonical child exit to uar-verification-simplification. Do not start another parent scenario. On failure leave the child incomplete, preserve receipts and discuss fix versus backlog. Earlier evidence is never overwritten or promoted to a repaired-source PASS.

The uncomfortable limit: this gateway remains settings-only. The host can enforce the outbound output ceiling and reserve its explicit allowance, but its local input sizing is estimated and must not be presented as an exact model-token count or fit guarantee. Provider-reported actual usage and unresolved liability remain authoritative. The earlier lost CAS cause also remains historical; only the real BDD establishes whether this workflow succeeds. No claim of runtime success is made by this packet.
