"""Installed credential and API-key durability through the production API."""
import uuid

from acceptance import require
from journeys import start_run, finish


def run_credentials(host, peer, spec, evidence):
    base = '/api/uar/credentials'
    provider = spec['credential_provider_id']
    path = base + '/' + provider
    original = 'dot17-private-' + uuid.uuid4().hex
    rotated = 'dot17-rotated-' + uuid.uuid4().hex
    owner = host.api('GET', base + '/principal')
    host.api('PUT', path, {'api_key': original})
    first = host.api('GET', path)
    require(first['api_key_hint'] == original[-4:], 'credential hint disagrees with saved secret')
    for principal in ('other_tenant', 'other_issuer'):
        host.api('GET', path, principal=principal, expect=(401, 404))
    host.restart()
    reader = peer or host
    saved = reader.api('GET', path)
    require(saved['api_key_hint'] == original[-4:], 'cold/shared credential missing')
    reader.api('PUT', path, {'api_key': rotated})
    changed = host.api('GET', path)
    require(changed['created_at'] == first['created_at'], 'rotation recreated ownership record')
    require(changed['api_key_hint'] == rotated[-4:], 'cross-process rotation stale')
    reader.api('DELETE', path, expect=204)
    host.api('GET', path, expect=404)
    evidence.observe('credential-api-restart', {'principal': owner, 'created_at': first['created_at'],
                     'joined_restart': True, 'shared_reader': reader.label,
                     'credential_values': 'intentionally_omitted'})
    keys = '/api/uar/auth/keys'
    created = host.api('POST', keys, {'name': 'dot17-' + uuid.uuid4().hex,
        'roles': ['user'], 'expires_in_secs': 3600}, expect=201)
    raw_key = created['raw_key']
    key_id = created['metadata']['id']
    def with_key(node, expect=200):
        return node.api('GET', keys, principal=None, headers={'x-api-key': raw_key}, expect=expect)
    with_key(host)
    host.restart()
    with_key(host)
    with_key(reader)
    reader.api('DELETE', keys + '/' + key_id)
    with_key(host, 401)
    if peer:
        with_key(peer, 401)
        peer.restart()
        with_key(peer, 401)
    host.restart()
    with_key(host, 401)
    evidence.observe('api-key-cross-runtime-revocation', {'key_id': key_id, 'joined_restart': True,
                     'revocation_writer': reader.label, 'revoked_admissions': '401',
                     'key_material': 'intentionally_omitted'})
