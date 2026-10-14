"""Commit-only source lineage. Reads metadata and retained evidence, never tests.

A changed source byte, mode, symlink, submodule pin, plan or matrix blocks reuse.
Published artifact conformance remains a separate, required execution boundary.
"""
import hashlib
from pathlib import Path
import subprocess

from acceptance import ROOT, PHASE, digest, read, require, save


def file_record(path):
    path = Path(path).resolve()
    return {'path': str(path), 'sha256': digest(path)}


def retained(record):
    require(digest(record['path']) == record['sha256'], 'retained evidence file changed: ' + record['path'])
    return read(record['path'])


def clean(source):
    require(source['dirty_patch_sha256'] == hashlib.sha256(b'').hexdigest() and not source['untracked_files'],
            'final source must be committed and clean within the inventoried scope')
    for module in source['submodules']:
        clean(module['identity'])


def pins(source):
    return [{'path': row['path'], 'git_head': row['identity']['git_head'], 'children': pins(row['identity'])}
            for row in sorted(source['submodules'], key=lambda value: value['path'])]


def validate_files(acceptance):
    require(acceptance.get('evidence_files'), 'local acceptance does not retain its complete evidence closure')
    for record in acceptance['evidence_files']:
        require(digest(record['path']) == record['sha256'],
                'retained evidence file changed: ' + record['path'])


def load_local_acceptance(path, identity):
    value = read(path)
    require(value['status'] == 'passed' and value['boundary'] == 'local_product', 'actual complete local acceptance required')
    require(value['inventory_sha256'] == identity['inventory_sha256'] and value['source_revision'] == identity['source_revision'],
            'local acceptance is not bound to current inventory/source')
    if value.get('lineage'):
        lineage = value['lineage']
        clean(identity['source'])
        require(lineage['mode'] == 'commit_only_exact_content' and lineage['finalized_source'] == identity['source'],
                'finalized source identity differs')
        old = retained(lineage['tested_acceptance'])
        require(old == value['original_tested_acceptance'], 'original acceptance was rewritten')
        old_inventory = retained(lineage['tested_inventory'])
        require(old['status'] == 'passed' and old['boundary'] == 'local_product' and
                old['inventory_sha256'] == lineage['tested_inventory']['sha256'] and
                old['source_revision'] == old_inventory['source']['git_head'] and
                old_inventory['verification_authorized'] is True and
                old_inventory['production'] == identity['production'], 'original tested authority differs')
        require(old_inventory['source'] == lineage['tested_source'] and
                old_inventory['source']['content_sha256'] == identity['source']['content_sha256'],
                'tested content differs from finalized source')
        require(pins(old_inventory['source']) == pins(identity['source']), 'finalization changed a submodule pin')
        require(old['matrix_sha256'] == digest(PHASE / 'implementation/dot17-scenario-matrix.json') and
                old_inventory['sequencing_sha256'] == identity['sequencing_sha256'], 'plan or scenario matrix changed')
        retained(lineage['tested_reconciliation_receipt'])
        retained(lineage['tested_customer_build_receipt'])
        validate_files(old)
        receipt = read(Path(path).parent / 'receipt.json')
        require(receipt['status'] == 'passed' and receipt['inventory_sha256'] == identity['inventory_sha256'] and
                receipt['source_revision'] == identity['source_revision'], 'finalization did not finish under current inventory')
        observations = [row for row in receipt['observations'] if row['id'] == 'source-content-finalized']
        require(len(observations) == 1 and observations[0]['data'] == lineage and
                read(Path(path).parent / 'source-content-finalized.json') == observations[0],
                'finalization observation or lineage differs')
    else:
        validate_files(value)
    return value


def load_customer_receipt(path, identity, local):
    customer = read(path)
    require(customer['status'] == 'passed', 'actual passing customer build required')
    if customer['inventory_sha256'] != identity['inventory_sha256']:
        lineage = local.get('lineage')
        require(lineage and file_record(path) == lineage['tested_customer_build_receipt'] and
                customer['inventory_sha256'] == lineage['tested_inventory']['sha256'],
                'customer build is not retained by exact-content finalization')
        require(customer['source_revision'] == lineage['tested_source']['git_head'], 'customer tested source differs')
    else:
        require(customer['source_revision'] == identity['source_revision'], 'customer source differs')
    for observation in customer['observations']:
        require(read(Path(path).parent / (observation['id'] + '.json')) == observation,
                'customer build observation changed')
    outputs = [row['data'] for row in customer['observations'] if row['id'] == 'customer-build-artifacts']
    require(len(outputs) == 1 and outputs[0]['artifacts'], 'customer build artifacts absent')
    for artifact in outputs[0]['artifacts']:
        require(digest(artifact['path']) == artifact['sha256'], 'actual customer artifact changed')
    return customer


def finalize_source(config, evidence):
    old_inventory_record = file_record(config['tested_inventory'])
    old_inventory = retained(old_inventory_record)
    old_acceptance_record = file_record(config['tested_acceptance'])
    old = retained(old_acceptance_record)
    prior_receipt_record = file_record(config['tested_reconciliation_receipt'])
    prior_receipt = retained(prior_receipt_record)
    customer_record = file_record(config['tested_customer_build_receipt'])
    customer = retained(customer_record)
    current = evidence.identity
    require(old_inventory['schema_version'] == 1 and old_inventory['verification_authorized'] is True,
            'tested inventory did not authorize its original execution')
    require(old_inventory['phase'] == current['phase'] and old_inventory['production'] == current['production'] and
            old_inventory['sequencing_sha256'] == current['sequencing_sha256'], 'finalization changed production plan or inventory')
    require(not old.get('lineage'), 'finalization consumes original actual local acceptance, not another lineage')
    require(old['status'] == 'passed' and old['boundary'] == 'local_product' and
            old['inventory_sha256'] == old_inventory_record['sha256'] and
            old['source_revision'] == old_inventory['source']['git_head'], 'original local acceptance binding differs')
    clean(current['source'])
    require(old_inventory['source']['content_sha256'] == current['source']['content_sha256'] and
            pins(old_inventory['source']) == pins(current['source']), 'commit changed tested content or dependency pins')
    ancestry = subprocess.run(['git', '-C', str(ROOT), 'merge-base', '--is-ancestor',
        old_inventory['source']['git_head'], current['source']['git_head']], capture_output=True)
    require(ancestry.returncode == 0, 'final revision does not descend from the tested checkout')
    matrix_sha = digest(PHASE / 'implementation/dot17-scenario-matrix.json')
    require(old['matrix_sha256'] == matrix_sha and old['sequencing_sha256'] == current['sequencing_sha256'],
            'accepted scenario matrix or sequencing changed')
    require(prior_receipt['status'] == 'passed' and prior_receipt['inventory_sha256'] == old_inventory_record['sha256'] and
            prior_receipt['source_revision'] == old_inventory['source']['git_head'], 'original reconciliation receipt differs')
    cases = [row['data'] for row in prior_receipt['observations'] if row['id'] == 'scenario-reconciliation']
    require(len(cases) == 1 and cases[0]['boundary'] == 'local_product' and cases[0]['matrix_sha256'] == matrix_sha and
            not cases[0]['missing'] and cases[0]['resolved'], 'original reconciliation did not cover the phase')
    for observed in prior_receipt['observations']:
        require(read(Path(prior_receipt_record['path']).parent / (observed['id'] + '.json')) == observed,
                'original reconciliation observation changed')
    validate_files(old)
    require(customer['status'] == 'passed' and customer['inventory_sha256'] == old_inventory_record['sha256'],
            'original customer build must precede commit-only finalization')
    for observed in customer['observations']:
        require(read(Path(customer_record['path']).parent / (observed['id'] + '.json')) == observed,
                'original customer build observation changed')
    lineage = {'mode': 'commit_only_exact_content', 'tested_inventory': old_inventory_record,
        'tested_acceptance': old_acceptance_record, 'tested_reconciliation_receipt': prior_receipt_record,
        'tested_customer_build_receipt': customer_record, 'tested_source': old_inventory['source'],
        'finalized_source': current['source'], 'matrix_sha256': matrix_sha,
        'sequencing_sha256': current['sequencing_sha256'], 'new_product_execution': False}
    value = {'schema_version': 1, 'status': 'passed', 'boundary': 'local_product',
        'inventory_sha256': current['inventory_sha256'], 'source_revision': current['source_revision'],
        'matrix_sha256': matrix_sha, 'sequencing_sha256': current['sequencing_sha256'],
        'original_tested_acceptance': old, 'lineage': lineage,
        'product_result': 'retained unchanged; metadata equivalence only',
        'publication': 'not_performed', 'installed_published_conformance': 'still_required'}
    load_customer_receipt(customer_record['path'], current, value)
    evidence.observe('source-content-finalized', lineage)
    save(evidence.directory / 'finalized-acceptance.json', value)
