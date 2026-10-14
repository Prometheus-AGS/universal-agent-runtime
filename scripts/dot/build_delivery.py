"""Local serial customer build, separate from product acceptance and publication."""
import os
import hashlib
from pathlib import Path
import shutil
import subprocess
import time

from acceptance import ROOT, Blocked, digest, read, require, save


def candidate_prefix_inputs():
    """Complete fixed frontend input closure, including actual installed bytes.

    This is intentionally more conservative than a lockfile-only comparison:
    changed installed modules, ignored Vite inputs or linked packages invalidate
    retention of the original passed prefix. Values of environment secrets are
    hashed, never included in observations.
    """
    package = read(ROOT / 'package.json')['packageManager']
    require(package.startswith('pnpm@'), 'unknown candidate package manager')
    pnpm = Path(shutil.which('pnpm')).resolve()
    node = Path(shutil.which('node')).resolve()
    corepack = pnpm.parent.parent
    pnpm_cache = Path(os.environ.get('COREPACK_HOME', str(Path.home() / '.cache/node/corepack'))) / 'v1/pnpm' / package.split('@', 1)[1]
    require((pnpm_cache / 'package.json').is_file(), 'actual pinned pnpm cache unavailable')
    roots = [ROOT / 'package.json', ROOT / 'pnpm-lock.yaml', ROOT / 'pnpm-workspace.yaml',
             ROOT / '.npmrc', ROOT / '.pnpmfile.cjs', ROOT / 'pnpmfile.cjs', ROOT / 'frontend',
             ROOT / 'node_modules', ROOT / 'scripts/validate-static-bundle.mjs', Path.home() / '.npmrc',
             node, corepack, pnpm_cache]
    resolved_roots = [root.resolve() for root in roots if root.exists()]
    rows, seen = [], set()
    def walk(path):
        name = str(path.absolute())
        if name in seen:
            return
        seen.add(name)
        if path.is_symlink():
            rows.append({'path': name, 'kind': 'symlink', 'target': os.readlink(path)})
            target = path.resolve()
            require(any(target == root or target.is_relative_to(root) for root in resolved_roots),
                    'dependency link escapes complete captured roots: ' + name)
        elif path.is_file():
            stat = path.stat()
            rows.append({'path': name, 'kind': 'file', 'sha256': digest(path), 'bytes': stat.st_size,
                         'executable': bool(stat.st_mode & 0o111)})
        elif path.is_dir():
            rows.append({'path': name, 'kind': 'directory'})
            for child in sorted(path.iterdir()):
                if child.name != '.git':
                    walk(child)
        elif not path.exists():
            rows.append({'path': name, 'kind': 'absent'})
        else:
            raise Blocked('unsupported prefix input: ' + name)
    for root in roots:
        walk(root)
    relevant = {key: hashlib.sha256(value.encode()).hexdigest() for key, value in os.environ.items()
                if key in ('NODE_ENV', 'NODE_OPTIONS', 'PNPM_HOME', 'COREPACK_HOME', 'COREPACK_NPM_REGISTRY',
                           'NPM_CONFIG_USERCONFIG', 'npm_config_userconfig')
                or key.startswith(('VITE_', 'npm_config_', 'NPM_CONFIG_'))}
    return {'schema_version': 1, 'roots': [str(root.absolute()) for root in roots],
            'entries': sorted(rows, key=lambda row: row['path']), 'environment_sha256': relevant,
            'node': str(node), 'pnpm': str(pnpm), 'package_manager': package}


def candidate_static_assets():
    rows = []
    for path in sorted((ROOT / 'static').rglob('*')):
        require(not path.is_symlink(), 'generated static output must be actual files')
        if path.is_file():
            rows.append({'path': str(path.relative_to(ROOT / 'static')), 'sha256': digest(path),
                         'bytes': path.stat().st_size})
    require(rows, 'retained static output is absent')
    return rows


def retain_failed_cargo_prefix(config, target, commands, evidence):
    reference = config['resume_failed_cargo']
    baseline_path = Path(reference['baseline']).resolve()
    require(digest(baseline_path) == reference['sha256'], 'candidate prefix baseline changed')
    baseline = read(baseline_path)
    require(baseline['schema_version'] == 1 and baseline['kind'] == 'candidate_prefix_baseline',
            'unknown candidate prefix baseline')
    require(baseline['target_directory'] == str(target), 'failed Cargo target directory changed')
    original_inventory = baseline['inventory']
    require(digest(original_inventory['path']) == original_inventory['sha256'], 'original inventory changed')
    old = read(original_inventory['path'])
    require(old['verification_authorized'] is True and old['source'] == baseline['tested_source'] and
            old['phase'] == evidence.identity['phase'] and old['production'] == evidence.identity['production'] and
            old['sequencing_sha256'] == evidence.identity['sequencing_sha256'], 'retained prefix plan or tested authority differs')
    previous = baseline['prior_receipt']
    require(digest(previous['path']) == previous['sha256'], 'original failed receipt changed')
    receipt = read(previous['path'])
    require(receipt['status'] == 'failed' and receipt['profile'] == 'candidate_build' and
            receipt['inventory_sha256'] == original_inventory['sha256'] and
            receipt['source_revision'] == old['source']['git_head'], 'baseline is not the original failed candidate gate')
    records = baseline['prior_observations']
    require(len(records) == 5 and len(receipt['observations']) == 5, 'only the fixed failed-Cargo pipeline can resume')
    for index, record in enumerate(records):
        observation = record['observation']
        require(observation == receipt['observations'][index] and observation['id'] == 'candidate-build-' + str(index),
                'original pipeline order or observation changed')
        require(digest(record['observation_file']) == record['observation_sha256'] and
                read(record['observation_file']) == observation and digest(record['log_file']) == record['log_sha256'] and
                observation['data']['log_sha256'] == record['log_sha256'], 'retained original observation or log differs')
        command, cwd = commands[index]
        require(observation['data']['command'] == command and observation['data']['cwd'] == str(cwd),
                'retained prefix is from another command pipeline')
        require(observation['data']['returncode'] == 0 if index < 4 else observation['data']['returncode'] != 0,
                'retention requires four actual passes followed by failed Cargo')
    require(candidate_prefix_inputs() == baseline['prefix_inputs'], 'consumed frontend/install input closure changed; cannot retain prefix')
    require(candidate_static_assets() == baseline['static_assets'], 'actual static bundle changed; cannot retain prefix')
    evidence.observe('candidate-prefix-retained', {'baseline': str(baseline_path), 'sha256': reference['sha256'],
        'original_inventory': original_inventory, 'original_source': baseline['tested_source'],
        'original_failed_receipt': previous, 'original_passed_observations': [row['observation'] for row in records[:4]],
        'input_entries': len(baseline['prefix_inputs']['entries']), 'static_assets': len(baseline['static_assets']),
        'execution': 'prefix not repeated; original observations retained under original tested identity',
        'current_action': 'run only fixed failed Cargo step against current authorized source'})


def run_customer_build(config, evidence):
    from finalize_source import load_local_acceptance
    acceptance = load_local_acceptance(config['local_acceptance'], evidence.identity)
    boss = Path(config['boss_checkout']).resolve()
    package = read(boss / 'package.json')
    require('build:mac:arm64' in package['scripts'], 'required Boss customer target unavailable')
    # The caller supplies actual expected artifact paths from the project build
    # configuration; no directory guessing or stale glob selection is permitted.
    paths = [Path(path).resolve() for path in config['artifacts']]
    require(paths and len(paths) == len(set(paths)), 'explicit distinct customer artifact paths required')
    for path in paths:
        require(path.is_relative_to(boss), 'customer output must belong to the selected checkout')
        if path.exists():
            raise Blocked('customer output already exists; allocate a clean output destination before building')
    boss_head = subprocess.check_output(['git', '-C', str(boss), 'rev-parse', 'HEAD']).decode().strip()
    import hashlib
    boss_patch = subprocess.check_output(['git', '-C', str(boss), 'diff', 'HEAD', '--binary'])
    command = ['pnpm', 'build:mac:arm64']
    started = time.time()
    with open(evidence.directory / 'boss-build.log', 'xb') as output:
        result = subprocess.run(command, cwd=boss, env=os.environ.copy(), stdout=output, stderr=subprocess.STDOUT)
    evidence.observe('customer-build-command', {'command': command, 'cwd': str(boss),
        'returncode': result.returncode, 'started': started, 'log_sha256': digest(output.name)})
    require(result.returncode == 0, 'customer build failed; see retained log')
    artifacts = []
    for path in paths:
        require(path.is_file() and path.stat().st_mtime >= started, 'customer build did not produce named artifact')
        artifacts.append({'path': str(path), 'sha256': digest(path), 'bytes': path.stat().st_size})
    evidence.observe('customer-build-artifacts', {'artifacts': artifacts, 'publication': 'not_claimed',
        'boss_source': {'git_head': boss_head, 'dirty_patch_sha256': hashlib.sha256(boss_patch).hexdigest()}})


def run_candidate_build(config, evidence):
    """Build the inventoried source locally before candidate runtime evidence."""
    target = Path(config['target_directory']).resolve()
    require(not target.is_relative_to(ROOT / 'src'), 'build output must not overwrite source')
    binary = target / 'release' / ('universal-agent-runtime.exe' if os.name == 'nt' else 'universal-agent-runtime')
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    commands = [(['pnpm', 'install', '--frozen-lockfile'], ROOT),
                (['pnpm', '-C', 'frontend', 'install', '--frozen-lockfile'], ROOT),
                (['pnpm', 'build'], ROOT),
                (['node', 'scripts/validate-static-bundle.mjs', 'static'], ROOT),
                (['cargo', 'build', '--locked', '--release', '--no-default-features',
                  '--bin', 'universal-agent-runtime', '--features', 'server-full'], ROOT)]
    start_index = 0
    if 'resume_failed_cargo' in config:
        retain_failed_cargo_prefix(config, target, commands, evidence)
        start_index = 4
    for index in range(start_index, len(commands)):
        command, cwd = commands[index]
        with open(evidence.directory / f'candidate-build-{index}.log', 'xb') as output:
            result = subprocess.run(command, cwd=cwd, env=environment, stdout=output, stderr=subprocess.STDOUT)
        evidence.observe('candidate-build-' + str(index), {'command': command, 'returncode': result.returncode,
            'cwd': str(cwd), 'log_sha256': digest(output.name)})
        require(result.returncode == 0, 'candidate build failed; later gates not run')
    require(binary.is_file(), 'candidate build emitted no runtime binary')
    evidence.observe('candidate-runtime-artifact', {'path': str(binary), 'sha256': digest(binary),
        'source': evidence.identity['source'], 'publication': 'not_claimed'})


def run_release_inputs(config, evidence):
    output = evidence.directory / 'release-input-provenance.json'
    commands = [(['bash', 'scripts/verify-runtime-image-toolchain-pin.sh'], 'image-toolchain-policy'),
                (['node', 'scripts/generate-release-input-provenance.mjs', str(output), str(ROOT)], 'release-input-policy')]
    for command, label in commands:
        with open(evidence.directory / (label + '.log'), 'xb') as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        evidence.observe(label, {'command': command, 'returncode': result.returncode, 'log_sha256': digest(log.name)})
        if result.returncode:
            raise Blocked('effective locked inputs disagree with approved policy or checkout is not immutable; no policy was edited')
    provenance = read(output)
    require(provenance['source_sha'] == evidence.identity['source_revision'] and
            provenance['input_boundary'] == 'release', 'release inputs refer to another source or candidate boundary')
    evidence.observe('locked-release-inputs', {'provenance': provenance, 'sha256': digest(output)})


def run_candidate_inputs(evidence):
    """Validate actual candidate pins without calling a clean-release closure a candidate."""
    inventory_path = evidence.directory / 'candidate-input-inventory.json'
    save(inventory_path, evidence.identity)
    output = evidence.directory / 'candidate-input-provenance.json'
    commands = [(['bash', 'scripts/verify-runtime-image-toolchain-pin.sh'], 'candidate-image-toolchain-policy'),
        (['node', 'scripts/generate-release-input-provenance.mjs', str(output), str(ROOT),
          '--candidate-inventory', str(inventory_path)], 'candidate-input-policy')]
    for command, label in commands:
        with open(evidence.directory / (label + '.log'), 'xb') as log:
            result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        evidence.observe(label, {'command': command, 'returncode': result.returncode, 'log_sha256': digest(log.name)})
        if result.returncode:
            raise Blocked('candidate input policy or exact source inventory differs; no release closure claimed')
    provenance = read(output)
    require(provenance['scope'] == 'local_candidate' and provenance['input_boundary'] == 'local_candidate'
            and provenance['release_certification'] is False and provenance['source_tree'] is None
            and provenance['source_sha'] == evidence.identity['source_revision']
            and provenance['candidate_source'] == evidence.identity['source']
            and provenance['candidate_inventory_sha256'] == digest(inventory_path), 'candidate provenance identity differs')
    evidence.observe('candidate-locked-inputs', {'provenance': provenance, 'sha256': digest(output),
        'publication': 'not_claimed', 'clean_release_closure': 'not_claimed'})
