"""Installed safe-default settings and actual persisted apply-boundary changes."""
import ipaddress
import json
import os
from pathlib import Path
import uuid

from acceptance import Blocked, Host, require, poll
from journeys import helper


def admin_headers(host):
    variable = host.spec.get('settings_admin_key_env')
    return {'x-uar-admin-key': os.environ[variable]} if variable else {}


def mutate(host, key, value):
    path = '/api/uar/settings/' + key
    headers = admin_headers(host)
    before = host.api('GET', path, principal='admin', headers=headers)
    host.api('PUT', path, {'value': value, 'expected_revision': before['revision']},
             principal='admin', headers=headers)
    return before


def run_settings(spec, evidence):
    default = Host(spec['safe_defaults']['fresh_host'], evidence, 'fresh-default')
    configured = Host(spec['safe_defaults']['configured_host'], evidence, 'configured-settings')
    fresh_config = json.loads(Path(default.spec['config']).read_text())
    require(not fresh_config.get('providers') and not fresh_config.get('llm', {}).get('model'),
            'fresh default test has an explicit provider/model')
    require('cors_allowed_origins' not in fresh_config.get('server', {}), 'default-origin test supplied an override')
    # A reused private DB cannot demonstrate fresh shipped defaults.
    fresh_root = Path(spec['safe_defaults']['fresh_data_root']).resolve()
    if fresh_root.exists():
        raise Blocked('safe-default data directory already exists; allocate a fresh isolated directory')
    from egress_capture import Capture
    capture = Capture(spec['safe_defaults']['egress_capture'], evidence)
    capture.start()
    try:
        default.start()
        effective = helper('dot20_effective_settings.py').inspect_installed(
            default.base, default.credential('admin'), default.credential(), {}, [])
        require(effective['llm']['provider_host'] == 'disabled', 'fresh runtime silently selected a model')
        require(effective['memory_embedder']['provider_host'] in ('local', 'disabled'), 'remote default embedder')
        default.api('GET', '/api/uar/runs', principal=None, expect=401)
        _, headers = default.request('GET', '/.well-known/agent-card.json', principal=None,
                                      headers={'Origin': 'https://unlisted.invalid'})
        require(not any(key.lower() == 'access-control-allow-origin' for key in headers), 'empty CORS default permits origin')
        body = default.api('POST', '/api/uar/runs', {'input': 'Explain this fresh installation.'},
                           expect=(200, 400, 422, 503))
        if 'run_id' in body:
            run = body['run_id']
            final = poll(lambda: default.api('GET', '/api/uar/runs/' + run),
                         lambda value: value['status'] in ('error', 'done', 'cancelled'), 'unconfigured inference hung')
            require(final['status'] == 'error', 'unconfigured model reported successful inference')
            events = default.api('GET', '/api/uar/runs/' + run + '/events?after=0')
            require('model' in json.dumps(events).lower(), 'missing-model error is not actionable')
        else:
            require('model' in json.dumps(body).lower(), 'inference failed for a different prerequisite')
        status = default.api('GET', '/api/uar/settings/governance/status', principal='admin', headers=admin_headers(default))
        require(status['jwt_required'] is True and status['bound_addresses'], 'actual ingress inventory/auth missing')
        if evidence.profile == 'local_surreal_kv':
            for address in status['bound_addresses']:
                ip = address.rsplit(':', 1)[0].strip('[]')
                require(ipaddress.ip_address(ip).is_loopback, 'actual listener bound beyond loopback')
        if evidence.profile == 'shared_two_runtime_surreal_db':
            explicit = fresh_config['server']['host']
            require(not ipaddress.ip_address(explicit).is_loopback, 'shared qualification did not explicitly configure network bind')
            require(any(address.rsplit(':', 1)[0].strip('[]') == explicit for address in status['bound_addresses']),
                    'actual shared listener differs from explicit deployed bind')
            evidence.observe('shared-explicit-bind', {'configured_host': explicit, 'actual_addresses': status['bound_addresses'],
                                                    'private_route_anonymous_status': 401})
        evidence.observe('shipped-default-settings', {'effective': effective, 'bound_addresses': status['bound_addresses'],
             'missing_model': body, 'egress_claim': 'separate independent packet capture observation required'})
    finally:
        default.stop()
        capture.finish()
    try:
        configured.start()
        before = configured.api('GET', '/api/uar/settings/effective', principal='admin')
        key = 'native_tools.terminal_exec_enabled'
        old = mutate(configured, key, not before['native_tools']['terminal_exec_enabled'])
        pre_restart = configured.api('GET', '/api/uar/settings/effective', principal='admin')
        require(pre_restart['native_tools'] == before['native_tools'], 'restart-only setting reported prematurely active')
        configured.restart()
        after = configured.api('GET', '/api/uar/settings/effective', principal='admin')
        helper('dot20_effective_settings.py').require_effective_change(before, after, 'native_tools')
        selected = before['llm']
        require(after['llm'] == selected, 'persisted operator model choice changed on restart')
        allowed = spec['safe_defaults']['allowed_origin']
        for origin, permitted in [(allowed, True), ('https://not-allowed.invalid', False)]:
            _, headers = configured.request('GET', '/.well-known/agent-card.json', principal=None, headers={'Origin': origin})
            observed = next((value for key, value in headers.items() if key.lower() == 'access-control-allow-origin'), None)
            require(observed == origin if permitted else observed is None, 'explicit CORS allowlist differs')
        mutate(configured, key, old['saved'])
        configured.restart()
        evidence.observe('effective-settings-live-revision', {'before': before, 'after_restart': after,
                         'persisted_model_unchanged': True, 'configured_origin': allowed})
    finally:
        configured.stop()
