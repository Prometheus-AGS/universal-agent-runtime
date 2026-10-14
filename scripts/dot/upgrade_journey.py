"""Real prior-to-current installed upgrade; never a same-binary substitute.

Deferred until the lead opens the full production boundary. Only production
provider/settings/inference APIs write state. Controlled peers observe routing.
"""
import json
import os
from pathlib import Path
import urllib.parse
import uuid

from acceptance import Blocked, Host, digest, require
from provider import Provider
from publication import installed_identity

PROVIDERS = '/api/uar/providers/'
SETTINGS = '/api/uar/settings/'


def admin(host):
    variable = host.spec.get('settings_admin_key_env')
    if variable and variable not in os.environ:
        raise Blocked('upgrade administration credential unavailable')
    return {'x-uar-admin-key': os.environ[variable]} if variable else {}


def api(host, method, path, body=None, expected=200, administrator=True, previous=False):
    # Retain actual HTTP status/hash in the standard evidence journal. Missing
    # prior routes and incompatible response contracts are explicit blockers,
    # never replaced by writes into the current binary or a protected store.
    acceptable = (expected, 400, 404, 405, 422, 501) if previous else expected
    try:
        result = host.api(method, path, body, expect=acceptable,
                          principal='admin' if administrator else 'owner',
                          headers=admin(host) if administrator else None)
    except AssertionError:
        if previous:
            raise Blocked('prior installed provider/settings/inference API is incompatible; actual HTTP evidence retained') from None
        raise
    status = host.evidence.http[-1]['status']
    if status != expected:
        raise Blocked('prior installed API contract unavailable: ' + method + ' ' + path)
    return result


def saved_setting(host, key, previous=False):
    row = api(host, 'GET', SETTINGS + urllib.parse.quote(key, safe=''), previous=previous)
    if not isinstance(row, dict) or ('saved' not in row and 'data' not in row):
        raise Blocked('installed settings API does not expose a saved value')
    # The historical metadata contract exposed `data`; the current contract
    # retains it and additionally separates saved/effective/revision.
    return row, row['saved'] if 'saved' in row else row['data']


def save_model(host, selector, previous=False):
    before, _ = saved_setting(host, 'llm.model', previous)
    body = {'value': selector}
    if 'revision' in before:
        body['expected_revision'] = before['revision']
    elif not previous:
        raise Blocked('current settings API omitted the required write revision')
    api(host, 'PUT', SETTINGS + 'llm.model', body, previous=previous)
    _, value = saved_setting(host, 'llm.model', previous)
    require(value == selector, 'deliberate model selection did not persist through the settings API')


def selection(host, entry, previous=False, aggregate=True):
    path = PROVIDERS + urllib.parse.quote(entry['id'], safe='')
    view = api(host, 'GET', path, previous=previous)
    providers = api(host, 'GET', PROVIDERS, previous=previous)
    _, default = saved_setting(host, 'llm.default_provider', previous)
    _, saved_provider = saved_setting(host, 'provider.' + entry['id'], previous)
    for value in (view, saved_provider):
        require(isinstance(value, dict) and value.get('id') == entry['id'] and
                value.get('default_model') == entry['default_model'] and
                value.get('base_url') == entry['base_url'] and value.get('enabled') is True,
                'saved or running provider/model choice differs')
    require(default == entry['id'] and providers['default_id'] == entry['id'],
            'saved default and live registry default differ')
    model = api(host, 'GET', '/api/uar/resolve-model', previous=previous, administrator=False)
    require(model.get('ok') is True and model['provider_id'] == entry['id'] and
            model['model_id'] == entry['default_model'], 'live default model resolver differs')
    value = {'provider_id': entry['id'], 'model_id': entry['default_model'],
             'saved_default': default, 'provider_setting': saved_provider, 'resolved': model}
    if aggregate:
        effective = api(host, 'GET', '/api/uar/settings/effective')
        require(effective['llm'] == {'provider_id': entry['id'], 'model': entry['default_model'],
                                    'provider_host': 'local'}, 'effective aggregate does not reflect live registry')
        require(isinstance(effective['revision'], str) and effective['revision'].startswith('sha256:'),
                'effective aggregate revision missing')
        value['effective'] = effective
    return value


def choose(host, entry, previous=False):
    api(host, 'POST', PROVIDERS + urllib.parse.quote(entry['id'], safe='') + '/default',
        previous=previous)


def verify_provider(host, entry, previous=False):
    value = api(host, 'POST', PROVIDERS + urllib.parse.quote(entry['id'], safe='') + '/test',
                {'model': entry['default_model']}, previous=previous)
    require(isinstance(value, dict) and value.get('ok') is True and value.get('received_text') is True,
            'real configured provider inference did not verify')


def route(host, entries, peers, selected, evidence, label, previous=False):
    marker = 'dot17-upgrade-' + label + '-' + uuid.uuid4().hex
    # No explicit provider/model, run credential, artifact policy or agent
    # override can accidentally make this a test of per-request routing.
    result = api(host, 'POST', '/v1/chat/completions', {'stream': False,
        'messages': [{'role': 'user', 'content': json.dumps({'dot_action': 'text', 'marker': marker})}]},
        administrator=False, previous=previous)
    try:
        text = result['choices'][0]['message']['content']
    except (KeyError, IndexError, TypeError):
        raise Blocked('installed default inference response contract is incompatible') from None
    require(isinstance(text, str) and marker in text, 'real default inference did not return the unique peer output')
    traces = [[row for row in peer.rows() if row.get('instruction', {}).get('marker') == marker] for peer in peers]
    require(traces[selected] and not traces[1 - selected], 'default inference reached the wrong controlled provider')
    require(all(row['model'] == entries[selected]['default_model'] for row in traces[selected]),
            'actual provider wire used a different model')
    evidence.observe(label + '-routing', {'marker': marker, 'provider_id': entries[selected]['id'],
        'model_id': entries[selected]['default_model'], 'requests': traces[selected],
        'other_provider_request_count': len(traces[1 - selected]), 'returned_marker': True})


def joined_stop(host):
    require(host.process is not None and host.process.poll() is None,
            'runtime exited before the required joined orderly stop')
    process = host.process
    host.stop()
    require(process.returncode == 0 and host.process is None, 'prior runtime shutdown did not join successfully')
    return {'pid': process.pid, 'exit_code': process.returncode, 'joined': True}


def installation(settings, spec, evidence):
    prior, current = settings['prior_host'], settings['current_host']
    require(prior['lifecycle'] == current['lifecycle'] == 'child_process', 'upgrade must own both actual processes')
    require(Path(prior['binary']).resolve() != Path(current['binary']).resolve(), 'upgrade binary paths must differ')
    require(prior['binary_sha256'] != current['binary_sha256'], 'same executable digest cannot prove an upgrade')
    provenance = installed_identity(settings['prior_publication'])
    require(Path(provenance['installed_binary']).resolve() == Path(prior['binary']).resolve() and
            provenance['installed_binary_sha256'] == prior['binary_sha256'], 'prior launch differs from actual prior publication')
    require(provenance['source_revision'] != evidence.identity['source_revision'], 'prior publication is the current source')
    current_path = Path(current['binary']).resolve()
    require(current_path.is_file() and digest(current_path) == current['binary_sha256'], 'current candidate binary unavailable or changed')
    if evidence.artifact['kind'] == 'independently_installed_publication':
        require(str(current_path) == evidence.artifact['installed_binary'] and
                current['binary_sha256'] == evidence.artifact['installed_binary_sha256'], 'upgrade target is not the inspected current publication')
    else:
        require(any(Path(host['binary']).resolve() == current_path and host['binary_sha256'] == current['binary_sha256']
                    for host in spec['hosts'].values()), 'upgrade target is not a phase candidate host binary')
    for key in ('config', 'working_directory', 'base_url', 'workspace', 'environment',
                'inherit_environment', 'credentials', 'settings_admin_key_env'):
        require(prior.get(key) == current.get(key), 'upgrade must retain the exact host setting: ' + key)
    config = json.loads(Path(prior['config']).read_text())
    require(not config.get('providers') and not config.get('llm', {}).get('model'),
            'upgrade selection must be created by the prior runtime API, not pre-seeded config')
    names = set(prior['environment']) | set(prior.get('inherit_environment', []))
    require(not any(name.upper().startswith(('UAR_LLM', 'UAR_PROVIDERS', 'UAR_PERSISTENCE')) for name in names),
            'upgrade config cannot be replaced by model/provider/persistence environment overrides')
    store = config['persistence']
    require(store['provider'] == 'surreal', 'upgrade profile requires actual durable Surreal persistence')
    root = Path(settings['fresh_data_root']).resolve()
    if root.exists():
        raise Blocked('upgrade data root already exists; allocate a new isolated prior installation')
    endpoint = store['database_url']
    if evidence.profile == 'local_surreal_kv':
        require(endpoint.startswith('surrealkv://'), 'local upgrade requires durable SurrealKV')
        path = Path(endpoint.removeprefix('surrealkv://'))
        require(path.is_absolute() and path.resolve().is_relative_to(root), 'local database must live in the fresh isolated data root')
    else:
        require(urllib.parse.urlsplit(endpoint).scheme in ('ws', 'wss', 'http', 'https'),
                'shared upgrade requires a real remote SurrealDB')
        require(store.get('surreal_ns', '').startswith('dot17-upgrade-') and store.get('surreal_db'),
                'shared upgrade requires a dedicated named namespace/database')
    root.mkdir(parents=True, exist_ok=False, mode=0o700)
    evidence.observe('upgrade-installations', {'prior': provenance,
        'current_binary': str(current_path), 'current_binary_sha256': current['binary_sha256'],
        'current_source': evidence.identity['source_revision'], 'private_config_sha256': digest(prior['config']),
        'fresh_data_root': str(root), 'same_configuration_and_stores': True})
    return config


def run_upgrade(spec, evidence):
    settings = spec.get('upgrade')
    if not settings:
        raise Blocked('actual prior/current upgrade installations are not configured')
    installation(settings, spec, evidence)
    configuration_digest = digest(settings['prior_host']['config'])
    previous = Host(settings['prior_host'], evidence, 'upgrade-prior')
    current = Host(settings['current_host'], evidence, 'upgrade-current')
    ports = settings.get('provider_ports', [0, 0])
    models = settings.get('fixture_model_ids', ['gpt-4o-mini', 'gpt-4o'])
    require(len(ports) == len(models) == 2 and models[0] != models[1], 'two distinct controlled model choices required')
    peers, entries = [], []
    try:
        for index in range(2):
            peer = Provider(ports[index], evidence.directory / ('upgrade-provider-' + str(index) + '.jsonl'))
            peer.start()
            peers.append(peer)
            entries.append({'id': 'dot17-upgrade-' + ('a' if index == 0 else 'b'),
                'display_name': 'Dot17 isolated upgrade provider ' + str(index),
                'base_url': 'http://127.0.0.1:' + str(peer.server.server_address[1]) + '/v1',
                'api_key': 'dot17-controlled-peer-only', 'protocol': 'chat',
                'default_model': models[index], 'models': [{'id': models[index], 'enabled': True,
                    'supports_tools': True, 'supports_streaming': True}], 'enabled': True})
        try:
            previous.start()
        except AssertionError:
            raise Blocked('actual prior binary cannot launch with the retained upgrade configuration; inspect its private boot log') from None
        initial = api(previous, 'GET', PROVIDERS, previous=True)
        require(initial.get('providers') == [] and initial.get('default_id') is None,
                'prior installation is not empty; existing operator state must not be overwritten')
        for entry in entries:
            api(previous, 'POST', PROVIDERS, entry, expected=201, previous=True)
            verify_provider(previous, entry, previous=True)
        choose(previous, entries[0], previous=True)
        prior_a = selection(previous, entries[0], previous=True, aggregate=False)
        route(previous, entries, peers, 0, evidence, 'prior-a', previous=True)
        choose(previous, entries[1], previous=True)
        save_model(previous, entries[1]['id'] + '/' + entries[1]['default_model'], previous=True)
        prior_b = selection(previous, entries[1], previous=True, aggregate=False)
        route(previous, entries, peers, 1, evidence, 'prior-b', previous=True)
        evidence.observe('upgrade-prior-selection', {'a': prior_a, 'b': prior_b,
            'saved_llm_model': saved_setting(previous, 'llm.model', True)[1],
            'shutdown': joined_stop(previous)})
        require(digest(current.spec['config']) == configuration_digest, 'upgrade configuration changed after prior selection')
        current.start()
        selected = selection(current, entries[1])
        require(saved_setting(current, 'llm.model')[1] == entries[1]['id'] + '/' + entries[1]['default_model'],
                'upgrade changed saved llm.model')
        # Model-verification records may legitimately be invalidated by the
        # candidate. Re-establish through actual inference; never replace the
        # migrated provider/default/model settings to repair an upgrade failure.
        for entry in entries:
            verify_provider(current, entry)
        route(current, entries, peers, 1, evidence, 'upgraded-b')
        choose(current, entries[0])
        switched_a = selection(current, entries[0])
        require(switched_a['effective']['revision'] != selected['effective']['revision'],
                'live default switch did not change the aggregate revision')
        route(current, entries, peers, 0, evidence, 'current-live-a')
        choose(current, entries[1])
        switched_b = selection(current, entries[1])
        require(switched_b['effective']['revision'] != switched_a['effective']['revision'] and
                switched_b['effective']['llm'] == selected['effective']['llm'], 'second live default switch not reflected')
        route(current, entries, peers, 1, evidence, 'current-live-b')
        shutdown = joined_stop(current)
        require(digest(current.spec['config']) == configuration_digest, 'configuration changed before persistence restart')
        current.start()
        restarted = selection(current, entries[1])
        require(saved_setting(current, 'llm.model')[1] == entries[1]['id'] + '/' + entries[1]['default_model'],
                'joined candidate restart changed saved model')
        route(current, entries, peers, 1, evidence, 'current-restarted-b')
        evidence.observe('upgrade-retained-live-default', {'after_upgrade': selected,
            'live_a': switched_a, 'live_b': switched_b, 'joined_restart': shutdown, 'after_restart': restarted,
            'claim': 'distinct prior binary persisted selection; current binary retained and actually routed it'})
    finally:
        # A failed joined stop is never followed by another boot. No force-kill
        # fallback can silently turn a migration case into crash recovery.
        try:
            current.stop()
        finally:
            try:
                previous.stop()
            finally:
                for peer in peers:
                    peer.stop()
