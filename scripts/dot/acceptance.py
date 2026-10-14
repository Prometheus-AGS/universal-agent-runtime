#!/usr/bin/env python3
"""Full-phase Dot acceptance. Source authoring is not permission to execute.

Only fixed production journey implementations are callable. This program never
accepts arbitrary commands, precomputed passing results, or scenario callbacks.
"""
import argparse
import hashlib
import json
import os
import re
from pathlib import Path
import platform
import signal
import subprocess
import sys
import tarfile
import time
import urllib.error
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
PHASE = ROOT / '.kbd-orchestrator/phases/dot-feature-support'
PROFILES = ('local_surreal_kv', 'shared_two_runtime_surreal_db')
SOURCE_INTEGRATION_TARGETS = ('credentials_api_integration_test', 'durable_api_keys_integration',
                              'truthful_tool_outcomes', 'stream_terminal_contract',
                              'fresh_process_resume', 'approval_recovery', 'durable_ingestion',
                              'run_cost_settlement', 'dot_memory_owner_contract', 'dot_memory_controls',
                              'dot_owner_adjacent_contract', 'principal_knowledge_namespace')
SOURCE_CONTINUATION_PREFIX = SOURCE_INTEGRATION_TARGETS[:2]
SOURCE_CONTINUATION_SUFFIX = SOURCE_INTEGRATION_TARGETS[2:]
SOURCE_CONTINUATION_ALLOWED_PATHS = (
    'tests/truthful_tool_outcomes.rs',
    'scripts/dot/acceptance.py',
    'scripts/dot/reconcile.py',
    'scripts/dot/finalize_source.py',
)
SOURCE_MEMORY_CONTINUATION_PREFIX = SOURCE_INTEGRATION_TARGETS[2:8]
SOURCE_MEMORY_CONTINUATION_SUFFIX = SOURCE_INTEGRATION_TARGETS[8:]
SOURCE_MEMORY_SCOPE_DOCS = (
    'docs/operations/supervisor-recovery.md',
    'docs/operations/dot-supervisor-qualification.md',
    'docs/operations/dot-validation-targets.md',
    'docs/operations/dot-release-acceptance.md',
    'docs/durable-schedules.md',
)
SOURCE_MEMORY_SCOPE_MATRIX = '.kbd-orchestrator/phases/dot-feature-support/implementation/dot17-scenario-matrix.json'
SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS = (
    'src/uar/memory/controls_store.rs',
    'scripts/dot/acceptance.py',
    'scripts/dot/reconcile.py',
    'scripts/dot/oci_build.py',
    *SOURCE_MEMORY_SCOPE_DOCS,
    SOURCE_MEMORY_SCOPE_MATRIX,
)
# These identify the actual failed inventory17 stage and its retained snapshot.
SOURCE_MEMORY_PRIOR_RECEIPT_SHA256 = '293ec8d8c18c8f3f723ba10da0fbd4f6db68ee48ca9bd7a9dd29209224b2e85a'
SOURCE_MEMORY_PRIOR_ARCHIVE_SHA256 = 'b60b9bdff6471e518b70af6b1ec1aeb4d190ec6d76f8cf7e921365e9cc55c041'
SOURCE_MEMORY_PRIOR_MANIFEST_SHA256 = '9f45e01fb5867ee1506afdf538982a4bb9333210126f1fa0ab77aadc1ffbde58'
SOURCE_MEMORY_SCOPE_APPEND_SHA256 = '32ff726a9bd3c44d93f07f095b59a8feb920e233e8bbbd76605b85ff5af46259'
SOURCE_MEMORY_SCOPE_RELEASE = {
    'decision_date': '2026-10-14',
    'operator_choice': 'Just use the Linux guest.',
    'native_environment': 'approved isolated Linux guest',
    'native_service_manager': 'systemd',
    'shared_environment': 'existing know-me LKE two-runtime profile',
    'unqualified_native_platforms': ['macOS', 'Windows'],
    'scenarios_removed': 0,
    'observations_waived': 0,
    'development_mac_reboot_authorized': False,
    'additional_paid_resources_authorized': False,
}


class Blocked(RuntimeError):
    pass


def require(value, message):
    if not value:
        raise AssertionError(message)


def digest(path):
    with open(path, 'rb') as stream:
        result = hashlib.sha256()
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def save(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with open(path, 'x', encoding='utf-8') as stream:
        os.chmod(path, 0o600)
        json.dump(value, stream, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())


def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args]).decode().strip()


SOURCE_PATHS = ('src', 'crates', 'frontend', 'migrations', 'scripts', 'tests', 'vendor', 'packaging', 'sdks',
                'deploy', 'k8s', 'docs', 'policies', 'config', 'configs', 'catalog', 'schemas', 'proto', 'resources',
                '.github', '.cargo', 'Cargo.toml', 'Cargo.lock', 'versions.toml',
                'package.json', 'pnpm-lock.yaml', 'pnpm-workspace.yaml',
                'build.rs', 'Dockerfile', '.dockerignore', 'rust-toolchain.toml', '.gitmodules',
                '.env.example', 'config.embedded.yaml', 'config.remote.postgres.yaml',
                'config.remote.surreal.yaml', 'example.config.yaml', 'config.test.yaml', 'test-config.yaml',
                'docker-compose.dev.yaml', 'docker-compose.prod.yaml', 'docker-compose.prod.postgres.yaml',
                'docker-compose.test.yaml', 'docker-compose.override.yaml',
                '.kbd-orchestrator/phases/dot-feature-support/implementation/dot17-scenario-matrix.json')


def source_identity():
    """Read-only source fingerprint; no compiler, formatter or product probe.

    Deliberately excludes the inventory itself, KBD projections/evidence and
    append-only memory. A receipt cannot invalidate the source it describes.
    Submodule dirty state and untracked source are included recursively.
    """
    def repository(path):
        diff = subprocess.check_output(['git', '-C', str(path), 'diff', 'HEAD', '--binary',
                                       '--', *SOURCE_PATHS] if path == ROOT else
                                      ['git', '-C', str(path), 'diff', 'HEAD', '--binary'])
        args = ['git', '-C', str(path), 'ls-files', '--others', '--exclude-standard']
        if path == ROOT:
            args += ['--', *SOURCE_PATHS]
        untracked = subprocess.check_output(args).decode().splitlines()
        entries = [{'path': name, 'sha256': digest(path / name)} for name in sorted(untracked)]
        modules = []
        indexed = subprocess.check_output(['git', '-C', str(path), 'ls-files', '--stage']).decode().splitlines()
        for entry in indexed:
            if entry.startswith('160000 '):
                name = entry.split('\t', 1)[1]
                child = path / name
                if not (child / '.git').exists():
                    raise Blocked('source submodule not initialized: ' + str(child))
                modules.append({'path': name, 'identity': repository(child)})
        tracked_args = ['git', '-C', str(path), 'ls-files', '-z']
        if path == ROOT:
            tracked_args += ['--', *SOURCE_PATHS]
        tracked = subprocess.check_output(tracked_args).decode().split('\0')
        files = []
        module_paths = {row['path'] for row in modules}
        for name in sorted(set(tracked + untracked) - {''} - module_paths):
            file = path / name
            if file.is_symlink():
                files.append({'path': name, 'type': 'symlink', 'target': os.readlink(file)})
            elif file.is_file():
                files.append({'path': name, 'type': 'file', 'sha256': digest(file),
                              'executable': bool(file.stat().st_mode & 0o111)})
            elif file.exists():
                raise Blocked('non-file source entry cannot be content-bound: ' + name)
            # A tracked deletion is represented by absence from actual content.
        content = {'files': files, 'submodules': [{'path': row['path'],
                   'git_head': row['identity']['git_head'], 'content_sha256': row['identity']['content_sha256']}
                   for row in sorted(modules, key=lambda row: row['path'])]}
        return {'git_head': subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD']).decode().strip(),
                'dirty_patch_sha256': hashlib.sha256(diff).hexdigest(),
                'untracked_files': entries, 'submodules': modules,
                'content_sha256': hashlib.sha256(json.dumps(content, sort_keys=True, separators=(',', ':')).encode()).hexdigest()}
    return repository(ROOT)


def boundary(inventory_path):
    """Require exact production denominator and byte-bound lead declaration."""
    inventory = read(inventory_path)
    ledger = read(PHASE / 'implementation-sequencing.json')
    waypoint = read(ROOT / '.kbd-orchestrator/current-waypoint.json')
    expected = {(task['change'], str(task['backendTaskId'])) for task in ledger['tasks']
                if task['kind'] == 'production'}
    require(len(expected) >= 24, 'expanded phase production denominator was reduced')
    require(waypoint['activePhase'] == 'dot-feature-support', 'wrong active phase')
    require(inventory['schema_version'] == 1 and inventory['phase'] == 'dot-feature-support',
            'wrong inventory schema/phase')
    require(inventory['sequencing_sha256'] == digest(PHASE / 'implementation-sequencing.json'),
            'plan changed after inventory')
    actual = {(item['change'], str(item['backendTaskId'])) for item in inventory['production']}
    require(actual == expected and len(inventory['production']) == len(expected),
            'every production item, including runner, must be inventoried exactly once')
    for item in inventory['production']:
        require(item['status'] == 'source_complete' and (ROOT / item['handoff']).is_file(),
                'unfinished production item: ' + item['change'])
    if inventory.get('verification_authorized') is not True:
        raise Blocked('source inventory is pending testing; explicit testing continuation has not authorized execution')
    require(inventory['source'] == source_identity(), 'source differs from full-phase inventory')
    return dict(inventory, source_revision=inventory['source']['git_head'])


class Evidence:
    def __init__(self, directory, identity, profile, artifact):
        self.directory = Path(directory)
        self.directory.mkdir(parents=True, exist_ok=False, mode=0o700)
        self.identity = identity
        self.profile = profile
        self.artifact = artifact
        self.observations = []
        self.http = []

    def observe(self, name, data):
        require(name not in [x['id'] for x in self.observations], 'duplicate observation')
        row = {'id': name, 'observed_at': time.time(), 'data': data}
        save(self.directory / (name + '.json'), row)
        self.observations.append(row)

    def finish(self, status, reason=None):
        value = {'schema_version': 1, 'source_revision': self.identity['source_revision'],
                 'inventory_sha256': self.identity['inventory_sha256'], 'profile': self.profile,
                 'artifact': self.artifact, 'status': status, 'reason': reason,
                 'observations': self.observations, 'http': self.http,
                 'certification': 'incomplete', 'publication': 'not_claimed'}
        save(self.directory / 'receipt.json', value)
        return value


class Host:
    def __init__(self, spec, evidence, label):
        self.spec = spec
        self.evidence = evidence
        self.label = label
        self.base = spec['base_url'].rstrip('/')
        self.workspace = spec['workspace']
        self.process = None
        self.boots = 0
        self.logs = []
        self.runtime = None
        if spec['lifecycle'] == 'kubernetes':
            from kubernetes import KubernetesRuntime
            self.runtime = KubernetesRuntime(self)

    def credential(self, name='owner'):
        variable = self.spec['credentials'][name]
        if variable not in os.environ:
            raise Blocked('credential environment variable unavailable: ' + variable)
        return os.environ[variable]

    def request(self, method, path, body=None, expect=200, principal='owner', headers=None):
        request_headers = {'x-uar-workspace-id': self.workspace, 'Accept': 'application/json'}
        if principal is not None:
            request_headers['Authorization'] = 'Bearer ' + self.credential(principal)
        if isinstance(body, bytes):
            payload = body
        elif body is not None:
            payload = json.dumps(body).encode()
            request_headers['Content-Type'] = 'application/json'
        else:
            payload = None
        request_headers.update(headers or {})
        request = urllib.request.Request(self.base + path, data=payload,
                                         headers=request_headers, method=method)
        try:
            response = urllib.request.urlopen(request, timeout=120)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            status = response.status
            try:
                value = json.loads(raw) if raw else None
            except ValueError:
                value = raw.decode(errors='replace')
            response_headers = dict(response.headers)
        self.evidence.http.append({'host': self.label, 'method': method, 'path': path,
                                   'status': status, 'principal': principal, 'response_sha256': hashlib.sha256(raw).hexdigest()})
        require(status in (expect if isinstance(expect, tuple) else (expect,)),
                f'{self.label} {method} {path}: expected {expect}, received {status}: {value}')
        return value, response_headers

    def api(self, *args, **kwargs):
        return self.request(*args, **kwargs)[0]

    def start(self):
        if self.runtime is not None:
            return self.runtime.start()
        if self.spec['lifecycle'] != 'child_process':
            raise Blocked('this journey requires joined child-process lifecycle; use supervisor stages for services/Pods')
        require(self.process is None, 'runtime already owned')
        binary = Path(self.spec['binary']).resolve()
        config = Path(self.spec['config']).resolve()
        if not binary.is_file() or not config.is_file():
            raise Blocked('installed runtime binary/config unavailable')
        require(digest(binary) == self.spec['binary_sha256'], 'runtime binary digest differs')
        self.boots += 1
        log = open(self.evidence.directory / f'{self.label}-boot-{self.boots}.log', 'xb')
        os.chmod(log.name, 0o600)
        self.logs.append(log)
        environment = {'PATH': os.environ.get('PATH', ''), **self.spec['environment']}
        for key in self.spec.get('inherit_environment', []):
            if key not in os.environ:
                raise Blocked('runtime prerequisite missing: ' + key)
            environment[key] = os.environ[key]
        self.process = subprocess.Popen([str(binary), '--config', str(config)],
            cwd=self.spec['working_directory'], env=environment,
            stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
            start_new_session=True)
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            require(self.process.poll() is None, 'runtime exited before readiness; retained boot log')
            try:
                with urllib.request.urlopen(self.base + '/health', timeout=2) as response:
                    if response.status == 200:
                        return
            except (OSError, urllib.error.URLError):
                pass
            time.sleep(.2)
        raise Blocked('runtime did not become ready; retained boot log')

    def stop(self, crash=False):
        if self.runtime is not None:
            return self.runtime.stop(crash)
        if self.process is None:
            return
        process = self.process
        if process.poll() is None:
            process.send_signal(signal.SIGKILL if crash else signal.SIGTERM)
            try:
                result = process.wait(timeout=45)
            except subprocess.TimeoutExpired:
                raise Blocked('runtime exit not joined; no restart permitted')
            if not crash:
                require(result == 0, 'orderly shutdown did not finish successfully')
        # Do not claim process-group termination as supervisor exclusion. External
        # effects and detached descendants require the independent controller.
        self.process = None
        for log in self.logs:
            log.flush()

    def restart(self):
        self.stop()
        self.start()


def poll(read_value, predicate, message, seconds=180):
    deadline = time.monotonic() + seconds
    value = None
    while time.monotonic() < deadline:
        value = read_value()
        if predicate(value):
            return value
        time.sleep(.25)
    raise AssertionError(message + ': ' + json.dumps(value))


def fixture(host, name, args, environment):
    target = ROOT / 'tests/fixtures' / name
    require(target.is_file(), 'fixture source missing: ' + name)
    env = dict(os.environ, **environment)
    completed = subprocess.run([sys.executable, str(target), *args], env=env,
                               capture_output=True, timeout=240)
    label = name.removesuffix('.py') + '-' + uuid.uuid4().hex
    for suffix, content in [('stdout', completed.stdout), ('stderr', completed.stderr)]:
        path = host.evidence.directory / (label + '.' + suffix)
        path.write_bytes(content)
        os.chmod(path, 0o600)
    require(completed.returncode == 0, 'real-path fixture failed: ' + label)
    return completed.stdout.decode()


def source_command(target):
    return ['cargo', 'test', '--locked', '--no-default-features', '--features',
            'server-full,postgres-backend,in-memory-backend', '--test', target, '--',
            '--include-ignored', '--nocapture', '--test-threads=1']


def validate_source_observation(directory, observed, target, passed):
    require(observed['id'] == target, 'unexpected source observation target')
    actual = read(Path(directory) / (target + '.json'))
    require(actual == observed, 'source observation file differs from receipt')
    data = observed['data']
    require(data['command'] == source_command(target) and
            data['boundary'] == 'source_integration', 'source target command differs')
    log = Path(directory) / (target + '.log')
    require(digest(log) == data['log_sha256'], 'source target log differs')
    if passed:
        require(data['returncode'] == 0, 'retained source target did not pass')
        summaries = re.findall(
            r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',
            log.read_text(errors='replace'))
        require(summaries and int(summaries[-1][0]) > 0 and
                all(int(value) == 0 for value in summaries[-1][1:]),
                'source target omitted required cases')
    else:
        require(data['returncode'] != 0, 'failed predecessor target was not failed')


def current_source_input_paths():
    tracked = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '-z', '--', *SOURCE_PATHS]).decode().split('\0')
    untracked = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '--others', '--exclude-standard',
                                          '--', *SOURCE_PATHS]).decode().splitlines()
    indexed = subprocess.check_output(['git', '-C', str(ROOT), 'ls-files', '--stage']).decode().splitlines()
    modules = {row.split('\t', 1)[1] for row in indexed if row.startswith('160000 ')}
    return set(tracked + untracked) - {'', *modules}


def validate_source_continuation(config, identity, archived_current=None):
    if config.get('allowed_changed_paths') == list(SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS):
        require(archived_current is None, 'memory continuation cannot be nested')
        return validate_memory_source_continuation(config, identity)
    expected = {'prior_receipt', 'prior_receipt_sha256', 'prior_inventory', 'prior_inventory_sha256',
                'frozen_manifest', 'frozen_manifest_sha256', 'frozen_archive', 'frozen_archive_sha256',
                'allowed_changed_paths'}
    require(set(config) == expected, 'source continuation configuration is not fixed')
    require(config['allowed_changed_paths'] == list(SOURCE_CONTINUATION_ALLOWED_PATHS),
            'source continuation allowlist differs')
    prior_receipt = Path(config['prior_receipt'])
    prior_inventory = Path(config['prior_inventory'])
    manifest_path = Path(config['frozen_manifest'])
    archive_path = Path(config['frozen_archive'])
    for path, expected_digest in ((prior_receipt, config['prior_receipt_sha256']),
                                  (prior_inventory, config['prior_inventory_sha256']),
                                  (manifest_path, config['frozen_manifest_sha256']),
                                  (archive_path, config['frozen_archive_sha256'])):
        require(path.is_file() and digest(path) == expected_digest, 'source continuation input differs: ' + str(path))
    prior = read(prior_receipt)
    inventory = read(prior_inventory)
    manifest = read(manifest_path)
    require(prior['status'] == 'failed' and prior['profile'] == 'source_integration',
            'source continuation predecessor must be the failed source stage')
    require(prior['inventory_sha256'] == config['prior_inventory_sha256'] == manifest['inventory_sha256'],
            'source continuation predecessor inventory differs')
    require(prior['source_revision'] == manifest['source_revision'] == inventory['source']['git_head'],
            'source continuation predecessor revision differs')
    require(inventory['source']['content_sha256'] == manifest['source_content_sha256'],
            'frozen manifest does not bind predecessor source')
    require(Path(manifest['archive']).resolve() == archive_path.resolve() and
            manifest['archive_sha256'] == config['frozen_archive_sha256'],
            'frozen archive binding differs')
    observed = prior['observations']
    require([row['id'] for row in observed] == list(SOURCE_CONTINUATION_PREFIX + SOURCE_CONTINUATION_SUFFIX[:1]),
            'predecessor did not stop at the first failed whole target')
    for row, target in zip(observed[:len(SOURCE_CONTINUATION_PREFIX)], SOURCE_CONTINUATION_PREFIX):
        validate_source_observation(prior_receipt.parent, row, target, True)
    validate_source_observation(prior_receipt.parent, observed[-1], SOURCE_CONTINUATION_SUFFIX[0], False)
    archived_files = None
    if archived_current is not None:
        require(archived_current['inventory_sha256'] == identity['inventory_sha256'] and
                archived_current['source_content_sha256'] == identity['source']['content_sha256'] and
                archived_current['source_revision'] == identity['source_revision'],
                'archived continuation successor differs')
        archived_files = frozen_source_files(archived_current)
    require(len(manifest['files']) == manifest['entries'], 'frozen manifest entry count differs')
    manifest_paths, changed = set(), []
    for item in manifest['files']:
        relative = Path(item['path'])
        require(not relative.is_absolute() and '..' not in relative.parts and str(relative) not in ('', '.'),
                'unsafe frozen manifest path')
        require(item['path'] not in manifest_paths, 'duplicate frozen manifest path')
        manifest_paths.add(item['path'])
        path = ROOT / relative
        if archived_files is not None:
            same = archived_files.get(item['path']) == item
        elif item['type'] == 'file':
            same = path.is_file() and not path.is_symlink() and digest(path) == item['sha256'] and \
                   bool(path.stat().st_mode & 0o111) == item['executable']
        elif item['type'] == 'symlink':
            same = path.is_symlink() and os.readlink(path) == item['target']
        else:
            raise Blocked('unsupported frozen manifest entry type')
        if not same:
            changed.append(item['path'])
    require(set(changed) == set(SOURCE_CONTINUATION_ALLOWED_PATHS),
            'current source differs outside the fixed continuation allowlist')
    current_paths = set(archived_files) if archived_files is not None else current_source_input_paths()
    unexpected = current_paths - manifest_paths - set(SOURCE_CONTINUATION_ALLOWED_PATHS)
    require(not unexpected, 'current source adds unbound input: ' + ', '.join(sorted(unexpected)))
    current = identity['source'] if archived_files is not None else source_identity()
    require(current == identity['source'], 'current continuation source differs from current inventory')
    require(inventory['verification_authorized'] is True and inventory['production'] == identity['production'] and
            inventory['sequencing_sha256'] == identity['sequencing_sha256'] and inventory['phase'] == identity['phase'],
            'predecessor continuation authority or production plan differs')
    return {'schema_version': 1, 'prior_receipt': str(prior_receipt.resolve()),
            'prior_receipt_sha256': config['prior_receipt_sha256'],
            'prior_inventory': str(prior_inventory.resolve()), 'prior_inventory_sha256': config['prior_inventory_sha256'],
            'frozen_manifest': str(manifest_path.resolve()), 'frozen_manifest_sha256': config['frozen_manifest_sha256'],
            'frozen_archive': str(archive_path.resolve()), 'frozen_archive_sha256': config['frozen_archive_sha256'],
            'allowed_changed_paths': list(SOURCE_CONTINUATION_ALLOWED_PATHS),
            'changed_paths': sorted(changed), 'verified_entries': len(manifest['files']),
            'current_inventory_sha256': identity['inventory_sha256'],
            'current_source_revision': identity['source_revision']}


def frozen_source_files(manifest):
    require(len(manifest['files']) == manifest['entries'], 'frozen manifest entry count differs')
    files = {}
    for item in manifest['files']:
        relative = Path(item['path'])
        require(not relative.is_absolute() and '..' not in relative.parts and str(relative) not in ('', '.'),
                'unsafe frozen manifest path')
        require(item['path'] not in files and item['type'] in ('file', 'symlink'),
                'duplicate or unsupported frozen manifest entry')
        files[item['path']] = item
    return files


def validate_memory_source_continuation(config, identity):
    expected = {'prior_receipt', 'prior_receipt_sha256', 'prior_inventory', 'prior_inventory_sha256',
                'frozen_manifest', 'frozen_manifest_sha256', 'frozen_archive', 'frozen_archive_sha256',
                'allowed_changed_paths'}
    require(set(config) == expected and
            config['allowed_changed_paths'] == list(SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS),
            'memory continuation configuration is not fixed')
    require(config['prior_receipt_sha256'] == SOURCE_MEMORY_PRIOR_RECEIPT_SHA256 and
            config['frozen_archive_sha256'] == SOURCE_MEMORY_PRIOR_ARCHIVE_SHA256 and
            config['frozen_manifest_sha256'] == SOURCE_MEMORY_PRIOR_MANIFEST_SHA256,
            'memory continuation is not the actual failed inventory17 predecessor')
    prior_path = Path(config['prior_receipt'])
    inventory_path = Path(config['prior_inventory'])
    manifest_path = Path(config['frozen_manifest'])
    archive_path = Path(config['frozen_archive'])
    for path, expected_digest in ((prior_path, config['prior_receipt_sha256']),
                                  (inventory_path, config['prior_inventory_sha256']),
                                  (manifest_path, config['frozen_manifest_sha256']),
                                  (archive_path, config['frozen_archive_sha256'])):
        require(path.is_file() and digest(path) == expected_digest, 'memory continuation input differs: ' + str(path))
    prior, inventory, manifest = read(prior_path), read(inventory_path), read(manifest_path)
    require(prior['status'] == 'failed' and prior['profile'] == 'source_integration' and
            prior['inventory_sha256'] == config['prior_inventory_sha256'] == manifest['inventory_sha256'] and
            prior['source_revision'] == manifest['source_revision'] == inventory['source']['git_head'] and
            inventory['source']['content_sha256'] == manifest['source_content_sha256'] and
            Path(manifest['archive']).resolve() == archive_path.resolve() and
            manifest['archive_sha256'] == config['frozen_archive_sha256'],
            'memory predecessor source binding differs')
    observed = prior['observations']
    require([row['id'] for row in observed] ==
            ['source-continuation-scope', *SOURCE_MEMORY_CONTINUATION_PREFIX, SOURCE_MEMORY_CONTINUATION_SUFFIX[0]],
            'memory predecessor did not stop at the fixed failed whole target')
    require(read(prior_path.parent / 'source-continuation-scope.json') == observed[0],
            'archived original continuation observation differs')
    historical = observed[0]['data']
    historical_config = {key: historical[key] for key in expected}
    historical_identity = dict(inventory, inventory_sha256=config['prior_inventory_sha256'],
                               source_revision=inventory['source']['git_head'])
    # Reproduce the original15->17 proof from frozen bytes, never from the new checkout.
    require(validate_source_continuation(historical_config, historical_identity, manifest) == historical,
            'original inventory15 to inventory17 scope proof differs')
    for row, target in zip(observed[1:-1], SOURCE_MEMORY_CONTINUATION_PREFIX):
        validate_source_observation(prior_path.parent, row, target, True)
    validate_source_observation(prior_path.parent, observed[-1], SOURCE_MEMORY_CONTINUATION_SUFFIX[0], False)
    files = frozen_source_files(manifest)
    changed = []
    for name, item in files.items():
        path = ROOT / name
        if item['type'] == 'file':
            same = path.is_file() and not path.is_symlink() and digest(path) == item['sha256'] and \
                   bool(path.stat().st_mode & 0o111) == item['executable']
        else:
            same = path.is_symlink() and os.readlink(path) == item['target']
        if not same:
            changed.append(name)
    require(set(changed) == set(SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS),
            'current source differs outside fixed memory/scope correction paths')
    require(not current_source_input_paths() - set(files) - set(SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS),
            'memory continuation adds unbound source inputs')
    # Read selected files without extracting the archive or trusting archive paths.
    with tarfile.open(archive_path) as archive:
        for name in (*SOURCE_MEMORY_SCOPE_DOCS, SOURCE_MEMORY_SCOPE_MATRIX):
            member = archive.getmember(name)
            require(member.isfile() and files[name]['type'] == 'file', 'scope snapshot entry is not a regular file')
            old = archive.extractfile(member).read()
            require(hashlib.sha256(old).hexdigest() == files[name]['sha256'], 'scope snapshot file differs')
            path = ROOT / name
            require(path.is_file() and not path.is_symlink() and
                    bool(path.stat().st_mode & 0o111) == files[name]['executable'],
                    'qualification scope changed a file type or mode: ' + name)
            current = path.read_bytes()
            if name in SOURCE_MEMORY_SCOPE_DOCS:
                require(current.startswith(old) and
                        hashlib.sha256(current[len(old):]).hexdigest() == SOURCE_MEMORY_SCOPE_APPEND_SHA256,
                        'qualification scope document is not the exact approved append: ' + name)
            else:
                old_matrix, matrix = json.loads(old), json.loads(current)
                require('release_native_qualification' not in old_matrix and
                        matrix.pop('release_native_qualification', None) == SOURCE_MEMORY_SCOPE_RELEASE and
                        matrix == old_matrix and len(matrix['scenarios']) == 109,
                        'qualification scope changed scenarios or observations')
    require(source_identity() == identity['source'], 'current memory continuation source differs from inventory')
    require(inventory['verification_authorized'] is True and inventory['production'] == identity['production'] and
            inventory['sequencing_sha256'] == identity['sequencing_sha256'] and inventory['phase'] == identity['phase'],
            'memory continuation changed production authority or plan')
    return {'schema_version': 1, 'continuation_boundary': 'fixed_memory_suffix4',
            'prior_receipt': str(prior_path.resolve()), 'prior_receipt_sha256': config['prior_receipt_sha256'],
            'prior_inventory': str(inventory_path.resolve()), 'prior_inventory_sha256': config['prior_inventory_sha256'],
            'frozen_manifest': str(manifest_path.resolve()), 'frozen_manifest_sha256': config['frozen_manifest_sha256'],
            'frozen_archive': str(archive_path.resolve()), 'frozen_archive_sha256': config['frozen_archive_sha256'],
            'allowed_changed_paths': list(SOURCE_MEMORY_CONTINUATION_ALLOWED_PATHS),
            'changed_paths': sorted(changed), 'verified_entries': len(files),
            'current_inventory_sha256': identity['inventory_sha256'],
            'current_source_revision': identity['source_revision'], 'retained_prefix_scope': historical,
            'scope_append_sha256': SOURCE_MEMORY_SCOPE_APPEND_SHA256,
            'release_native_qualification': SOURCE_MEMORY_SCOPE_RELEASE}


def source_continuation_targets(proof):
    if proof.get('continuation_boundary') == 'fixed_memory_suffix4':
        return SOURCE_MEMORY_CONTINUATION_SUFFIX
    require('continuation_boundary' not in proof, 'unknown source continuation boundary')
    return SOURCE_CONTINUATION_SUFFIX


def source_gates(evidence, environment, continuation=None):
    """Whole real-production integration targets, serial in one build directory.

    These complement installed-host journeys; they cannot certify the installed
    published executable because Cargo supplies its own runtime binary.
    """
    targets = SOURCE_INTEGRATION_TARGETS
    required = ('UAR_CREDENTIAL_TEST_SURREAL_URL', 'UAR_CREDENTIAL_TEST_POSTGRES_URL',
                'UAR_API_KEY_TEST_SURREAL_URL', 'UAR_API_KEY_TEST_POSTGRES_URL',
                'UAR_DOT04_INFRA_FILE', 'UAR_DOT04_NAMESPACE', 'UAR_DOT04_EVIDENCE_ROOT',
                'UAR_DOT05_INFRA_FILE', 'UAR_DOT05_NAMESPACE', 'UAR_DOT06_INFRA_FILE',
                'UAR_DOT06_NAMESPACE', 'UAR_DOT07_EVIDENCE_ROOT',
                'UAR_ACCOUNTING_SURREAL_URL', 'UAR_ACCOUNTING_POSTGRES_URL',
                'UAR_DOT_MEMORY_SURREAL_URL', 'UAR_REMOTE_SURREAL_DURABILITY_ATTESTED',
                'UAR_DOT12_POSTGRES_URL', 'UAR_DOT12_SURREAL_URL')
    merged = dict(os.environ, **environment)
    missing = [key for key in required if not merged.get(key)]
    if missing:
        raise Blocked('real integration infrastructure missing: ' + ', '.join(missing))
    require(merged['UAR_REMOTE_SURREAL_DURABILITY_ATTESTED'] == '1', 'durability attestation missing')
    require(not any(key.endswith('_CHILD') or key.endswith('_JOB') or key in ('UAR_ACCOUNTING_FIXTURE_MODE', 'UAR_DOT12_KNOWLEDGE_STAGE')
                    for key in merged if key.startswith('UAR_')), 'child-test mode would bypass parent journey')
    if continuation is not None:
        proof = validate_source_continuation(continuation, evidence.identity)
        evidence.observe('source-continuation-scope', proof)
        targets = source_continuation_targets(proof)
    for target in targets:
        command = source_command(target)
        with open(evidence.directory / (target + '.log'), 'xb') as output:
            result = subprocess.run(command, cwd=ROOT, env=merged,
                                    stdout=output, stderr=subprocess.STDOUT)
        evidence.observe(target, {'command': command, 'returncode': result.returncode,
                                 'log_sha256': digest(output.name), 'boundary': 'source_integration'})
        require(result.returncode == 0, 'failed integration target: ' + target)
        summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',
                               Path(output.name).read_text(errors='replace'))
        require(summaries and int(summaries[-1][0]) > 0 and all(int(value) == 0 for value in summaries[-1][1:]),
                'integration target omitted required cases: ' + target)



def run_stale_epoch(environment, evidence):
    merged = dict(os.environ, **environment)
    if not merged.get('UAR_DOT_STALE_EPOCH_CASE'):
        raise Blocked('actual qualified closed predecessor case is required before stale-executor target')
    case = read(merged['UAR_DOT_STALE_EPOCH_CASE'])
    require(digest(case['qualified_observation']) == case['qualified_observation_sha256'],
            'qualified predecessor observation changed')
    command = ['cargo', 'test', '--locked', '--no-default-features', '--features',
        'server-full,postgres-backend,in-memory-backend', '--test', 'dot_stale_epoch_contract',
        '--', '--include-ignored', '--nocapture', '--test-threads=1']
    with open(evidence.directory / 'dot_stale_epoch_contract.log', 'xb') as output:
        result = subprocess.run(command, cwd=ROOT, env=merged, stdout=output, stderr=subprocess.STDOUT)
    require(result.returncode == 0, 'actual stale executor fence target failed')
    summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',
        Path(output.name).read_text(errors='replace'))
    require(summaries and int(summaries[-1][0]) > 0 and all(int(value) == 0 for value in summaries[-1][1:]),
            'stale executor target omitted required case')
    evidence.observe('dot_stale_epoch_contract', {'command': command, 'returncode': result.returncode,
        'log_sha256': digest(output.name), 'qualified_observation_sha256': case['qualified_observation_sha256']})


def stage_hosts(stage, profile, spec):
    if stage == 'stale-epoch':
        return []
    if stage == 'settings':
        return [spec['safe_defaults']['fresh_host'], spec['safe_defaults']['configured_host']]
    if stage == 'unsupported':
        return [spec['unsupported']['host']]
    if stage == 'upgrade':
        return [spec['upgrade']['current_host']]
    if stage == 'governance':
        rows = [spec['governance']['mandatory_host']]
        if profile == 'local_surreal_kv':
            rows.append(spec['governance']['local']['host'])
        return rows
    return list(spec['hosts'].values())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', choices=['journeys', 'source-integration', 'installed', 'supervisor', 'calendar', 'settings', 'streams', 'memory-capture', 'recovery-faults', 'governance', 'upgrade', 'customer-build', 'candidate-build', 'release-inputs', 'unsupported', 'deployment-health', 'followup-clock', 'schedule-release', 'http-capabilities', 'stale-epoch', 'finalize-source', 'oci-build', 'reconcile'])
    parser.add_argument('--inventory', required=True)
    parser.add_argument('--config', required=True)
    parser.add_argument('--evidence', required=True)
    args = parser.parse_args()
    os.umask(0o077)
    inventory = boundary(args.inventory)
    inventory['inventory_sha256'] = digest(args.inventory)
    config = read(args.config)
    require(set(config['profiles']) == set(PROFILES), 'both real profiles required')
    from journeys import run_profile
    from publication import installed_identity
    results = []
    standalone = {'source-integration': 'source_integration', 'customer-build': 'customer_build', 'candidate-build': 'candidate_build', 'release-inputs': 'release_inputs', 'oci-build': 'oci_build', 'reconcile': 'reconciliation', 'finalize-source': 'source_finalization'}
    for profile in (standalone[args.stage],) if args.stage in standalone else PROFILES:
        base_spec = config['profiles'].get(profile, {})
        spec = dict(base_spec, **base_spec.get('stage_overrides', {}).get(args.stage, {}))
        artifact = {'kind': 'local_candidate', 'source_revision': inventory['source_revision']}
        evidence = Evidence(Path(args.evidence) / profile, inventory, profile, artifact)
        try:
            if args.stage == 'installed' or (config.get('artifact_boundary') == 'published' and args.stage not in standalone):
                def committed(identity):
                    require(identity['dirty_patch_sha256'] == hashlib.sha256(b'').hexdigest() and not identity['untracked_files'],
                            'published source must be a committed tree, including the runner')
                    for child in identity['submodules']:
                        committed(child['identity'])
                committed(inventory['source'])
                artifact = installed_identity(spec['publication'])
                require(artifact['source_revision'] == inventory['source_revision'], 'installed publication is not final phase source')
                for node in stage_hosts(args.stage, profile, spec):
                    if node['lifecycle'] == 'kubernetes':
                        require(node['image_digest'] == artifact['architecture_digests'][node['architecture']],
                                'Pod architecture digest differs from published image')
                        require(node['binary_sha256'] == artifact['installed_binary_sha256'],
                                'Pod executable must match published platform archive checksum')
                    else:
                        require(Path(node['binary']).resolve() == Path(artifact['installed_binary']).resolve(),
                                'installed boot path differs from verified publication')
                        require(node['binary_sha256'] == artifact['installed_binary_sha256'], 'installed boot digest mismatch')
                evidence.artifact = artifact
                evidence.observe('published-provenance', artifact)
            if args.stage not in standalone and evidence.artifact['kind'] == 'local_candidate':
                nodes = stage_hosts(args.stage, profile, spec)
                receipts = []
                native_nodes = [node for node in nodes if node['lifecycle'] != 'kubernetes']
                require(all(node.get('candidate_runtime_origin') in (None, 'oci') for node in native_nodes),
                        'unknown native candidate runtime origin')
                if any(node.get('candidate_runtime_origin') != 'oci' for node in native_nodes):
                    build_path = spec.get('candidate_build_receipt', config['candidate_build_receipt'])
                    build = read(build_path)
                    require(build['status'] == 'passed' and build['inventory_sha256'] == inventory['inventory_sha256'],
                            'candidate executable requires same-inventory local build receipt')
                    artifacts = [row['data'] for row in build['observations'] if row['id'] == 'candidate-runtime-artifact']
                    require(len(artifacts) == 1, 'candidate runtime artifact missing')
                    for node in nodes:
                        if node['lifecycle'] != 'kubernetes' and node.get('candidate_runtime_origin') != 'oci':
                            require(Path(node['binary']).resolve() == Path(artifacts[0]['path']).resolve() and
                                    node['binary_sha256'] == artifacts[0]['sha256'], 'candidate boot differs from built artifact')
                    receipts.append({'kind': 'binary', 'sha256': digest(build_path), 'artifact': artifacts[0]})
                if any(node['lifecycle'] == 'kubernetes' or node.get('candidate_runtime_origin') == 'oci'
                       for node in nodes):
                    build_path = spec.get('oci_build_receipt') or config.get('oci_build_receipt')
                    require(build_path and Path(build_path).is_file(), 'actual candidate OCI build receipt missing')
                    build = read(build_path)
                    require(build['status'] == 'passed' and build['profile'] == 'oci_build' and
                            build['inventory_sha256'] == inventory['inventory_sha256'] and
                            build['source_revision'] == inventory['source_revision'],
                            'installed candidate requires same-inventory/source local OCI build receipt')
                    observations = [row for row in build['observations'] if row['id'] == 'local-two-architecture-image']
                    require(len(observations) == 1 and
                            read(Path(build_path).parent / 'local-two-architecture-image.json') == observations[0],
                            'candidate OCI artifact observation missing or changed')
                    built = observations[0]['data']
                    require(built['source_revision'] == inventory['source_revision'] and
                            built['source_content_sha256'] == inventory['source']['content_sha256'] and
                            Path(built['archive']).is_file() and digest(built['archive']) == built['sha256'],
                            'candidate OCI source or actual archive digest differs')
                    require(set(built['architectures']) == set(built['runtime_binaries']) == {'amd64', 'arm64'},
                            'candidate OCI requires both architecture manifests and exported executables')
                    for node in nodes:
                        if node['lifecycle'] != 'kubernetes' and node.get('candidate_runtime_origin') != 'oci':
                            continue
                        architecture = node['architecture']
                        require(architecture in ('amd64', 'arm64'), 'candidate OCI architecture is not supported')
                        binary = built['runtime_binaries'][architecture]
                        require(binary['architecture'] == architecture and binary['os'] == 'linux' and
                                binary['image_digest'] == built['architectures'][architecture] and
                                Path(binary['path']).is_file() and digest(binary['path']) == binary['sha256'] and
                                Path(binary['path']).stat().st_size == binary['bytes'],
                                'candidate OCI exported executable differs from selected architecture')
                        require(node['binary_sha256'] == binary['sha256'],
                                'installed executable digest differs from selected OCI export')
                        if node['lifecycle'] == 'kubernetes':
                            require(node['image_digest'] == built['architectures'][architecture],
                                    'installed Pod architecture digest differs from locally exported OCI content')
                        else:
                            machine = {'aarch64': 'arm64', 'arm64': 'arm64',
                                       'x86_64': 'amd64', 'amd64': 'amd64'}.get(platform.machine().lower())
                            require(node['os'] == 'linux' and sys.platform == 'linux' and machine == architecture,
                                    'native OCI qualification must run on the actual selected Linux architecture')
                            require(Path(node['binary']).is_file() and digest(node['binary']) == binary['sha256'],
                                    'actual installed native executable differs from OCI export')
                    receipts.append({'kind': 'oci_image', 'sha256': digest(build_path), 'artifact': built})
                evidence.artifact = dict(artifact, build_receipts=receipts)
            if args.stage == 'candidate-build':
                from build_delivery import run_candidate_build
                run_candidate_build(config['candidate_build'], evidence)
            elif args.stage == 'oci-build':
                from oci_build import run_oci_build
                run_oci_build(config['oci_build'], evidence)
            elif args.stage == 'release-inputs':
                from build_delivery import run_release_inputs
                run_release_inputs(config, evidence)
            elif args.stage == 'stale-epoch':
                run_stale_epoch(dict(config['integration_environment'], UAR_DOT_STALE_EPOCH_CASE=config['stale_epoch_cases'][profile]), evidence)
            elif args.stage == 'http-capabilities':
                from http_capability_journey import reconcile_http_capabilities
                reconcile_http_capabilities(spec, evidence)
            elif args.stage == 'source-integration':
                source_gates(evidence, config['integration_environment'], config.get('source_continuation'))
            elif args.stage == 'customer-build':
                from build_delivery import run_customer_build
                run_customer_build(config['customer_build'], evidence)
            elif args.stage == 'finalize-source':
                from finalize_source import finalize_source
                finalize_source(config['source_finalization'], evidence)
            elif args.stage == 'reconcile':
                from reconcile import reconcile
                reconcile(config['reconciliation'], evidence)
            elif args.stage == 'supervisor':
                from supervisor_journey import run_supervisor
                run_supervisor(spec, evidence)
            elif args.stage == 'recovery-faults':
                from recovery_fault_journey import run_recovery_faults
                run_recovery_faults(spec, evidence)
            elif args.stage == 'memory-capture':
                from memory_capture_journey import run_memory_capture
                run_memory_capture(spec, evidence)
            elif args.stage == 'deployment-health':
                from deployment_health import run_deployment_health
                run_deployment_health(spec, evidence)
            elif args.stage == 'unsupported':
                from unsupported_journey import run_unsupported
                run_unsupported(spec, evidence)
            elif args.stage == 'governance':
                from governance_journey import run_governance
                run_governance(spec, evidence)
            elif args.stage == 'upgrade':
                from upgrade_journey import run_upgrade
                run_upgrade(spec, evidence)
            elif args.stage == 'streams':
                from budget_journey import run_stream_profile
                run_stream_profile(spec, evidence)
            elif args.stage == 'schedule-release':
                from schedule_release_journey import run_schedule_release
                run_schedule_release(spec, evidence)
            elif args.stage == 'followup-clock':
                from followup_clock_journey import run_followup_clock
                run_followup_clock(spec, evidence)
            elif args.stage == 'calendar':
                from calendar_journey import run_calendar
                run_calendar(spec, evidence)
            elif args.stage == 'settings':
                from settings_journey import run_settings
                run_settings(spec, evidence)
            else:
                run_profile(profile, spec, evidence)
            # Journey success is narrower than the complete matrix. Certification
            # can be promoted only by the separate reconciliation command.
            results.append(evidence.finish('passed'))
        except Blocked as error:
            results.append(evidence.finish('blocked', str(error)))
        except Exception as error:
            results.append(evidence.finish('failed', str(error)))
    save(Path(args.evidence) / 'stage.json', {'stage': args.stage, 'profiles': results,
         'certification': 'incomplete', 'publication': 'not_claimed'})
    return 0 if all(row['status'] == 'passed' for row in results) else 1


if __name__ == '__main__':
    sys.exit(main())
