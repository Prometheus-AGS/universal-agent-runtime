"""Fail closed on missing scenario observations; never promote source descriptions."""
from pathlib import Path

from acceptance import (PHASE, Blocked, SOURCE_INTEGRATION_TARGETS, SOURCE_MEMORY_CONTINUATION_PREFIX,
                        digest, read, require, save, source_continuation_targets,
                        validate_source_continuation, validate_source_observation)


def source_continuation(config, rows, identity):
    continuation = config.get('source_continuation')
    if continuation is None:
        return None
    proof = validate_source_continuation(continuation, identity)
    targets = source_continuation_targets(proof)
    expected_ids = ['source-continuation-scope', *targets]
    candidates = [(path, row) for path, row in rows if row['profile'] == 'source_integration' and
                  [observed['id'] for observed in row['observations']] == expected_ids]
    require(len(candidates) == 1, 'continuation requires exactly one current suffix receipt')
    current_path, current = candidates[0]
    require(current['observations'][0]['data'] == proof, 'continuation scope proof differs from current receipt')
    for observed, target in zip(current['observations'][1:], targets):
        validate_source_observation(Path(current_path).parent, observed, target, True)
    prior_path = Path(proof['prior_receipt'])
    prior = read(prior_path)
    stages = [(prior_path, prior, proof)]
    if proof.get('continuation_boundary') == 'fixed_memory_suffix4':
        original_proof = proof['retained_prefix_scope']
        original_path = Path(original_proof['prior_receipt'])
        original = read(original_path)
        stages.append((original_path, original, original_proof))
        retained = [(original_path, original, observed) for observed in original['observations'][:2]]
        retained += [(prior_path, prior, observed) for observed in prior['observations'][1:-1]]
        require([observed['id'] for _, _, observed in retained] ==
                list(SOURCE_INTEGRATION_TARGETS[:2] + SOURCE_MEMORY_CONTINUATION_PREFIX),
                'retained memory prefix is not the eight actual passing whole targets')
    else:
        retained = [(prior_path, prior, observed) for observed in prior['observations'][:2]]
    return {'proof': proof, 'current_path': current_path, 'retained': retained, 'prior_stages': stages,
            'targets': targets}


def reconcile(config, evidence):
    matrix = read(PHASE / 'implementation/dot17-scenario-matrix.json')
    kind = config['boundary']
    require(kind in ('local_product', 'published_product'), 'unknown evidence boundary')
    receipts = []
    for path in config['receipts']:
        row = read(path)
        require(row['inventory_sha256'] == evidence.identity['inventory_sha256'], 'stale inventory receipt')
        require(row['source_revision'] == evidence.identity['source_revision'], 'stale source receipt')
        if row['status'] != 'passed':
            raise Blocked('required stage is not passed: ' + str(path))
        if kind == 'published_product' and row['profile'] != 'source_integration':
            require(row['artifact']['kind'] == 'independently_installed_publication',
                    'source/candidate evidence cannot certify an installed published artifact')
        # Each recorded observation must still exist, byte-for-byte as decoded.
        for observed in row['observations']:
            actual = read(Path(path).parent / (observed['id'] + '.json'))
            require(actual == observed, 'observation file differs from receipt')
        receipts.append((path, row))
    continuation = source_continuation(config, receipts, evidence.identity)
    def require_profiles(rows):
        present = {row['profile'] for _, row in rows}
        if not {'local_surreal_kv', 'shared_two_runtime_surreal_db'} <= present:
            raise Blocked('both independently observed durable runtime profiles are required')
    require_profiles(receipts)
    try:
        require_profiles([(path, row) for path, row in receipts if row['profile'] == 'local_surreal_kv'])
    except Blocked:
        evidence.observe('single-profile-rejected', {'input': 'actual local receipts with shared receipts omitted',
                         'outcome': 'blocked', 'no_synthetic_passing_receipt': True})
    else:
        raise AssertionError('reconciliation accepted only one profile')
    if kind == 'published_product':
        from finalize_source import load_local_acceptance
        local = load_local_acceptance(config['local_acceptance'], evidence.identity)
        release_path = Path(config['clean_release_inputs_receipt'])
        release = read(release_path)
        require(release['status'] == 'passed' and release['profile'] == 'release_inputs' and
                release['inventory_sha256'] == evidence.identity['inventory_sha256'] and
                release['source_revision'] == evidence.identity['source_revision'], 'clean final release input closure missing')
        closures = [row for row in release['observations'] if row['id'] == 'locked-release-inputs']
        require(len(closures) == 1 and read(release_path.parent / 'locked-release-inputs.json') == closures[0],
                'clean input closure observation differs')
        provenance_path = release_path.parent / 'release-input-provenance.json'
        provenance = closures[0]['data']['provenance']
        require(digest(provenance_path) == closures[0]['data']['sha256'] and read(provenance_path) == provenance and
                provenance['input_boundary'] == 'release' and provenance['source_sha'] == evidence.identity['source_revision'],
                'candidate or changed provenance cannot satisfy final clean closure')
        evidence.observe('clean-release-input-closure', {'receipt': str(release_path.resolve()),
            'sha256': digest(release_path), 'provenance_sha256': digest(provenance_path)})
    resolved, missing = [], []
    for scenario in matrix['scenarios']:
        if kind == 'local_product' and scenario.get('evidence_boundary') == 'publication':
            continue
        if kind == 'published_product' and scenario.get('evidence_boundary') != 'publication' and not scenario['change'].startswith(('dot-18-', 'dot-19-', 'dot-20-')):
            # Changes 01–17 remain prerequisites in the same-source local acceptance.
            continue
        driver = scenario.get('driver')
        if driver and driver.get('kind') == 'profile_completeness_invariant':
            require(any(row['id'] == 'single-profile-rejected' for row in evidence.observations), 'negative profile control missing')
            resolved.append({'scenario': scenario['id'], 'profile': 'both', 'observation': 'single-profile-rejected'})
            continue
        if not driver or driver.get('implementation') != 'complete':
            missing.append({'scenario': scenario['id'], 'reason': scenario.get('source_gap', 'driver incomplete')})
            continue
        for profile in scenario['profiles']:
            for requirement in driver['required_observations']:
                if profile not in requirement.get('applicable_profiles', scenario['profiles']):
                    continue
                matches = []
                for path, row in receipts:
                    accepted_profile = requirement.get('profile', profile)
                    if row['profile'] not in requirement.get('profiles_any', [accepted_profile]):
                        continue
                    if kind == 'published_product' and requirement.get('source_only'):
                        continue
                    observed_rows = [(path, row, observation) for observation in row['observations']]
                    if continuation and Path(path).resolve() == Path(continuation['current_path']).resolve():
                        observed_rows += continuation['retained']
                    for observation_path, observation_row, observation in observed_rows:
                        if observation['id'] == requirement['id']:
                            matches.append({'receipt': str(observation_path), 'sha256': digest(observation_path),
                                            'observation': observation['id']})
                if not matches:
                    missing.append({'scenario': scenario['id'], 'profile': profile,
                                    'observation': requirement['id']})
                else:
                    resolved.append({'scenario': scenario['id'], 'profile': profile,
                                     'evidence': matches})
    evidence.observe('scenario-reconciliation', {'matrix_sha256': digest(PHASE / 'implementation/dot17-scenario-matrix.json'),
        'boundary': kind, 'resolved': resolved, 'missing': missing})
    if missing:
        raise Blocked('scenario coverage incomplete; see scenario-reconciliation.json')
    if kind == 'published_product':
        from finalize_source import load_customer_receipt
        customer = load_customer_receipt(config['customer_build_receipt'], evidence.identity, local)
        require(any(row['id'] == 'customer-build-artifacts' for row in customer['observations']), 'customer artifact missing')
    evidence_files = []
    for path, row in receipts:
        evidence_files.append({'path': str(Path(path).resolve()), 'sha256': digest(path)})
        for observation in row['observations']:
            observed_path = Path(path).parent / (observation['id'] + '.json')
            evidence_files.append({'path': str(observed_path.resolve()), 'sha256': digest(observed_path)})
            if continuation and Path(path).resolve() == Path(continuation['current_path']).resolve() and \
                    observation['id'] in continuation['targets']:
                log_path = Path(path).parent / (observation['id'] + '.log')
                evidence_files.append({'path': str(log_path.resolve()), 'sha256': digest(log_path)})
    if continuation:
        for prior_path, prior, proof in continuation['prior_stages']:
            evidence_files.append({'path': str(prior_path.resolve()), 'sha256': digest(prior_path)})
            for observation in prior['observations']:
                observed_path = prior_path.parent / (observation['id'] + '.json')
                evidence_files.append({'path': str(observed_path.resolve()), 'sha256': digest(observed_path)})
                if observation['id'] in SOURCE_INTEGRATION_TARGETS:
                    log_path = prior_path.parent / (observation['id'] + '.log')
                    evidence_files.append({'path': str(log_path.resolve()), 'sha256': digest(log_path)})
            for key in ('prior_inventory', 'frozen_manifest', 'frozen_archive'):
                path = Path(proof[key])
                evidence_files.append({'path': str(path.resolve()), 'sha256': digest(path)})
    save(evidence.directory / 'acceptance.json', {'schema_version': 1, 'status': 'passed',
         'boundary': kind, 'inventory_sha256': evidence.identity['inventory_sha256'],
         'source_revision': evidence.identity['source_revision'], 'matrix_scenarios': len(matrix['scenarios']),
         'matrix_sha256': digest(PHASE / 'implementation/dot17-scenario-matrix.json'),
         'sequencing_sha256': evidence.identity['sequencing_sha256'], 'evidence_files': evidence_files,
         'deployment_health': 'separate_deployment_evidence_required', 'publication': 'not_performed'})
