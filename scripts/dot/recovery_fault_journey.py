"""Staged real-service faults; invoked only beyond the full-phase boundary.

The operator changes dedicated services/networks between immutable stages.
Neither supplied outcomes nor callbacks can make a stage pass. Stale executor
writes require a mechanism the public API does not expose and remain blocked.
"""
import base64
import hashlib
import http.client
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import time
import urllib.parse
import urllib.error
import urllib.request
import uuid

from acceptance import ROOT, Blocked, Host, digest, poll, read, require, save
from supervisor_journey import helper, live_child, observe, stopped_child

BASE = '/api/uar/recovery/supervisor/challenges'


def probe(case, role, network=None, log_since=None, call_ids=(), original_pod=False):
    remote = case[role]
    require(remote.get('dedicated_qualification_host') is True, 'dedicated fault observer required')
    require(re.fullmatch(r'[A-Za-z0-9_.@-]+', remote['ssh_target']) and
            not remote['ssh_target'].startswith('-'), 'invalid explicit SSH destination')
    require(remote['python'] == 'python3', 'Linux fault observer requires python3')
    known = Path(remote['known_hosts']).resolve()
    require(known.is_file(), 'pinned SSH host keys required')
    options = dict(remote['process'])
    options['runtime_epoch'] = case['runtime_epoch']
    if role == 'controller_observer':
        options['source_sha256'] = digest(ROOT / 'scripts/supervisor/controller.py')
        options['original_pod'] = original_pod
    if network:
        if network == 'embedded_enospc':
            options['embedded_volume'] = case['storage_faults']['embedded_volume']
            if log_since:
                options['log_since'], options['call_ids'] = log_since, list(call_ids)
        else:
            options['network'] = network
            options['controller_id'] = case['controller_id']
        options['application_listener'] = bool(case.get('storage_faults'))
    encoded = base64.urlsafe_b64encode(json.dumps(options).encode()).decode()
    command = ['ssh', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes',
               '-o', 'UserKnownHostsFile=' + str(known), '-o', 'ConnectTimeout=15',
               remote['ssh_target'], remote['python'], '-', encoded]
    done = subprocess.run(command, input=(ROOT / 'scripts/dot/recovery_fault_probe.py').read_bytes(),
                          capture_output=True, timeout=60)
    if done.returncode != 0:
        raise Blocked('independent recovery fault observation unavailable; no outcome inferred')
    require(len(done.stdout) <= 4 * 1024 * 1024, 'fault observer response limit')
    result = json.loads(done.stdout)
    require(result['machine_id'] == remote['expected_machine_id'], 'wrong fault observation host')
    return result


def network(case, purpose, connected):
    result = probe(case, 'runtime_observer', purpose)
    require(all(item['connected'] is connected for item in result['network']['attempts']),
            'actual runtime endpoint did not remain in the required network state')
    return result


def post(config, path, body):
    return helper().request(config['runtime_urls'][0], path, config['host_token_env'], body)


def challenge(config, case):
    status, result = post(config, BASE, {'runtime_epoch': case['runtime_epoch'], 'target': case['target']})
    require(status == 200, 'actual durable predecessor challenge unavailable')
    value = result['challenge']
    require(value['target'] == case['target'] and value['generation'] == 1 and
            value['registration']['runtime_epoch'] == case['runtime_epoch'] and
            value['registration']['controller_id'] == case['controller_id'], 'challenge identity differs')
    return value


def exclude(config, value, success):
    path = BASE + '/' + urllib.parse.quote(value['id'], safe='') + '/exclude'
    status, result = post(config, path, {})
    require(status == (200 if success else 409), 'unexpected actual exclusion response')
    if success:
        require(result['challenge_id'] == value['id'] and result['exclusion_recorded'] is True and
                result['effects_reconciled'] is False and result['execution_resumed'] is False,
                'exclusion response overclaimed authority')
        replay, _ = post(config, path, {})
        require(replay == 409, 'durable challenge consumption was not enforced')
    return {'challenge_id': value['id'], 'status': status, 'response': result}


def controller(case, value=None, original_pod=False):
    result = probe(case, 'controller_observer', original_pod=original_pod)
    journal = result['journal']
    require(journal['registration']['runtime_epoch'] == case['runtime_epoch'] and
            journal['registration']['controller_id'] == case['controller_id'], 'wrong journal registration')
    if value:
        require(journal['registration'] == value['registration'], 'controller/runtime registration differs')
    return result


def retained(result, value):
    matches = [item for item in result['journal']['exclusions'] if item['id'] == value['id']]
    require(len(matches) == 1 and matches[0]['request'] == value and
            matches[0]['receipt']['challenge'] == value and
            matches[0]['receipt']['observation'] == result['journal']['observation'],
            'exact durable controller challenge/observation is missing')


def pod_disappeared(before, after):
    initial, current = before['original_pod'], after['original_pod']
    for field in ('cluster_id', 'namespace', 'pod_name', 'original_uid', 'api', 'ca_sha256'):
        require(initial[field] == current[field], 'original Pod API observation binding changed')
    require(initial['status'] == 200 and initial['observed_uid'] == initial['original_uid'] and
            (current['status'] == 404 or current['status'] == 200 and current['observed_uid'] != initial['original_uid']),
            'original Pod API object still exists; a terminating object is not disappearance')


def live_original_cri(before, after):
    domain = before['registration']['domain']
    require(after['registration'] == before['registration'] and after['boot_id'] == domain['node_boot_id'],
            'original CRI registration or node boot changed')
    for observed in (before, after):
        group = observed['pod_cgroup']
        require(group['present'] and group['matches_retained'] and group['events']['populated'] == '1',
                'original Pod cgroup is not independently live')
        require(observed['sandbox']['id'] == domain['sandbox_id'] and observed['sandbox']['state'] == 'SANDBOX_READY',
                'original CRI sandbox is not live')
        require(sorted(row['id'] for row in observed['containers']) == sorted(domain['container_ids']) and
                all(row['state'] == 'CONTAINER_RUNNING' for row in observed['containers']),
                'original CRI containers are no longer executing')
    require(after['descendant'] == before['descendant'] and after['descendant_process']['same_process'] is True and
            after['heartbeat_count'] > before['heartbeat_count'], 'original detached child no longer advances')


def persist(case, evidence, stage, data):
    record = {'schema_version': 1, 'source': evidence.identity['source'],
              'case_sha256': digest(case['private_case_file']), 'stage': stage, 'data': data}
    save(Path(case['continuation_directory']) / (stage + '.json'), record)
    evidence.observe(case['name'] + '-' + stage, data)


def prior(case, evidence, stage):
    value = read(Path(case['continuation_directory']) / (stage + '.json'))
    require(value['schema_version'] == 1 and value['source'] == evidence.identity['source'] and
            value['case_sha256'] == digest(case['private_case_file']) and value['stage'] == stage,
            'fault source or private case changed between stages')
    return value['data']


def unchanged_runtime(before, after):
    require(before['machine_id'] == after['machine_id'] and before['boot_id'] == after['boot_id'] and
            before['process'] == after['process'], 'surviving runtime changed')
    if 'network' in before:
        require(before['network']['address'] == after['network']['address'] and
                before['network']['port'] == after['network']['port'], 'runtime endpoint changed')
    else:
        same_volume(before, after)


def same_volume(before, after):
    for field in ('database', 'mountpoint', 'device', 'filesystem_uuid', 'major_minor',
                  'capacity_bytes', 'guest_vendor', 'guest_product', 'guest_device_tree', 'mount_namespace'):
        require(before['embedded_volume'][field] == after['embedded_volume'][field],
                'embedded database volume identity changed: ' + field)


def no_effect(case, before):
    after = observe(case)
    require(after['registration'] == before['registration'] and after['descendant'] == before['descendant'] and
            after['effect_count'] == before['effect_count'], 'fault stage changed domain or produced an effect')
    return after


def restart_prepare(config, case, evidence):
    # Reuse a genuinely excluded domain. Establish that this is a successful
    # recovery baseline, not same-boot refusal hidden behind a restarted service.
    before = observe(case)
    require(before['exclusion'] is not None, 'first complete a real native/CRI exclusion case')
    value = challenge(config, case)
    result = exclude(config, value, True)
    saved = controller(case, value)
    retained(saved, value)
    persist(case, evidence, 'restart-prepare', {'challenge': value, 'http': result,
            'controller': saved, 'domain': no_effect(case, before)})
    raise Blocked('baseline retained; explicitly restart only the dedicated controller service, then restart-complete')


def restart_complete(config, case, evidence):
    saved = prior(case, evidence, 'restart-prepare')
    before = saved['controller']
    after = controller(case, saved['challenge'])
    require(after['machine_id'] == before['machine_id'] and after['boot_id'] == before['boot_id'] and
            after['process']['identity'] != before['process']['identity'], 'real same-boot controller restart not observed')
    require(after['journal'] == before['journal'], 'controller restart lost or changed durable evidence')
    require(after['process']['config_sha256'] == before['process']['config_sha256'] and
            after['process']['executable_sha256'] == before['process']['executable_sha256'], 'controller installation changed')
    value = challenge(config, case)
    require(value['registration'] == saved['challenge']['registration'], 'restart rebound predecessor domain')
    result = exclude(config, value, True)
    final = controller(case, value)
    retained(final, value)
    require(final['journal']['observation'] == before['journal']['observation'], 'restart invented a different exclusion')
    persist(case, evidence, 'restart-complete', {'http': result, 'controller': final,
            'domain': no_effect(case, saved['domain'])})


def fault_prepare(config, case, evidence, purpose):
    require(case.get('authorize_physical_exclusion') is True, 'dedicated physical exclusion authorization required')
    runtime = network(case, purpose, True)
    if purpose == 'predecessor_controller':
        domain = live_child(case, evidence, case['name'] + '-partition-prepare')
        require(domain['registration']['domain']['kind'] == 'kubernetes', 'partition recovery requires the real CRI profile')
        require(case.get('authorize_node_inhibition') is True, 'retained dedicated node inhibition authorization required')
    else:
        domain = observe(case)
        if domain['registration']['domain']['kind'] == 'native_boot':
            require(domain['boot_id'] != domain['registration']['domain']['boot_id'], 'outage baseline must be physically excludable')
        else:
            require(case.get('authorize_node_inhibition') is True, 'retained dedicated node inhibition authorization required')
    require(domain['exclusion'] is None, 'fault needs a fresh unexcluded predecessor')
    value = challenge(config, case)
    journal = controller(case, value, original_pod=purpose == 'predecessor_controller')
    if purpose == 'predecessor_controller':
        pod = journal['original_pod']
        require(pod['status'] == 200 and pod['observed_uid'] == pod['original_uid'] and not pod.get('deletion_timestamp'),
                'prepare requires the actual original Pod API object before operator deletion')
        require(case.get('authorize_original_pod_deletion') is True,
                'operator deletion of the exact isolated Pod must be explicitly authorized')
    require(journal['journal']['observation'] is None, 'predecessor already has exclusion proof')
    persist(case, evidence, purpose + '-prepare', {'challenge': value, 'runtime': runtime,
            'controller': journal, 'domain': domain})
    if purpose == 'predecessor_controller':
        raise Blocked('baseline retained; operator must isolate the dedicated predecessor control paths and remove only the authorized original Pod API object, preserving independent management observation, then run predecessor_controller-offline')
    raise Blocked('baseline retained; operator must isolate the configured ' + purpose + ' endpoint, then run its offline stage')


def fault_offline(config, case, evidence, purpose):
    saved = prior(case, evidence, purpose + '-prepare')
    runtime = network(case, purpose, False)
    unchanged_runtime(saved['runtime'], runtime)
    result = exclude(config, saved['challenge'], False)
    after = controller(case, saved['challenge'], original_pod=purpose == 'predecessor_controller')
    require(after['journal'] == saved['controller']['journal'], 'unavailable dependency nevertheless committed controller proof')
    if purpose == 'predecessor_controller':
        pod_disappeared(saved['controller'], after)
        domain = live_child(case, evidence, case['name'] + '-partition-offline')
        live_original_cri(saved['domain'], domain)
        require(domain['effect_count'] == saved['domain']['effect_count'], 'partition produced a new effect')
    else:
        status, _ = post(config, BASE, {'runtime_epoch': case['runtime_epoch'], 'target': case['target']})
        require(status == 409, 'persistence outage admitted another durable challenge')
        domain = no_effect(case, saved['domain'])
    after_runtime = network(case, purpose, False)
    unchanged_runtime(runtime, after_runtime)
    persist(case, evidence, purpose + '-offline', {'runtime': after_runtime, 'controller': after,
            'domain': domain, 'http': result})
    raise Blocked('actual dependency outage refusal observed; restore the isolated endpoint, then run its restored stage')


def fault_restored(config, case, evidence, purpose):
    saved = prior(case, evidence, purpose + '-prepare')
    offline = prior(case, evidence, purpose + '-offline')
    runtime = network(case, purpose, True)
    unchanged_runtime(saved['runtime'], runtime)
    unchanged_runtime(offline['runtime'], runtime)
    before = controller(case, saved['challenge'], original_pod=purpose == 'predecessor_controller')
    if purpose == 'predecessor_controller':
        pod_disappeared(saved['controller'], before)
    require(before['journal'] == saved['controller']['journal'], 'state changed before dependency restoration proof')
    result = exclude(config, saved['challenge'], True)
    final = controller(case, saved['challenge'])
    retained(final, saved['challenge'])
    domain = stopped_child(case, saved['domain'], evidence, case['name'] + '-restored')
    if purpose == 'predecessor_controller':
        proof = final['journal']['observation']
        require(proof['kind'] == 'kubernetes_runtime' and proof['descendant_cgroup_empty'] is True and
                proof['kubelet_cgroup_empty'] is True and proof['restart_inhibited'] is True and
                proof['kubelet_runtime_masked'] is True, 'missing actual CRI exclusion and node inhibition proof')
        require(domain['kubelet']['ActiveState'] in ('inactive', 'failed') and domain['kubelet']['MainPID'] == '0' and
                domain['kubelet']['UnitFileState'] in ('masked', 'masked-runtime'), 'kubelet inhibition is not independently observed')
        original = saved['domain']['registration']['domain']
        require(domain['sandbox']['id'] == original['sandbox_id'] and domain['sandbox']['state'] == 'SANDBOX_NOTREADY' and
                sorted(row['id'] for row in domain['containers']) == sorted(original['container_ids']) and
                all(row['state'] == 'CONTAINER_EXITED' for row in domain['containers']), 'original CRI identities were not independently stopped')
    persist(case, evidence, purpose + '-restored', {'runtime': runtime, 'controller': final,
            'domain': domain, 'http': result})


def controlled_peer(case, marker, release=False):
    settings = case['storage_faults']['provider_control']
    base = urllib.parse.urlsplit(settings['base_url'])
    require(base.scheme == 'http' and base.hostname == '127.0.0.1' and not base.username and
            not base.query and not base.fragment and base.path in ('', '/'), 'control peer must be actual loopback fixture')
    token = os.environ.get(settings['token_env'])
    if not token:
        raise Blocked('controlled upstream private token unavailable')
    path = '/__dot/release' if release else '/__dot/state?marker=' + urllib.parse.quote(marker, safe='')
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *_):
            raise Blocked('controlled peer redirect refused')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    request = urllib.request.Request(settings['base_url'].rstrip('/') + path,
        data=json.dumps({'marker': marker}).encode() if release else None,
        headers={'Authorization': 'Bearer ' + token, 'Content-Type': 'application/json'},
        method='POST' if release else 'GET')
    try:
        with opener.open(request, timeout=10) as response:
            return json.loads(response.read(65536))
    except urllib.error.HTTPError as error:
        if error.code == 404 and not release:
            return None
        raise


def provider_rows(case, marker):
    path = Path(case['storage_faults']['provider_control']['trace'])
    if not path.is_file():
        raise Blocked('independent controlled provider trace unavailable')
    return [row for row in (json.loads(line) for line in path.read_bytes().split(b'\n')[:-1] if line)
            if row.get('marker') == marker or row.get('instruction', {}).get('marker') == marker]


def storage_host(case, evidence, observed):
    host = Host(case['storage_faults']['host'], evidence, 'storage-fault-runtime')
    url = urllib.parse.urlsplit(host.base)
    listener = observed['application_listener']
    require(url.scheme == 'http' and url.hostname == listener['host'] == '127.0.0.1' and
            (url.port or 80) == listener['port'] and url.path in ('', '/'), 'upload must use the observed direct runtime listener')
    require(Path('/etc/machine-id').read_text().strip() == observed['machine_id'] and
            Path('/proc/self/ns/net').stat().st_ino == listener['network_namespace'],
            'storage runner must share the actual runtime host/network namespace')
    return host


def await_network(case, initial, connected, evidence, label):
    deadline = time.monotonic() + 600
    print('Dot storage fault: operator must ' + ('restore' if connected else 'stop or partition') +
          ' the dedicated database now; waiting for measured state', flush=True)
    while time.monotonic() < deadline:
        observed = probe(case, 'runtime_observer', 'persistence')
        unchanged_runtime(initial, observed)
        if all(row['connected'] is connected for row in observed['network']['attempts']):
            evidence.observe(case['name'] + '-' + label, observed)
            return observed
        time.sleep(2)
    raise Blocked('actual operator database transition was not observed within the live barrier lifetime')


def is_disk(case):
    mechanism = case['storage_faults'].get('mechanism', 'network')
    require(mechanism in ('network', 'embedded_enospc'), 'unsupported actual storage failure mechanism')
    return mechanism == 'embedded_enospc'


def storage_observation(case, healthy, log_since=None, call_ids=()):
    if not is_disk(case):
        return network(case, 'persistence', healthy)
    result = probe(case, 'runtime_observer', 'embedded_enospc', log_since, call_ids)
    require(all((row['available_bytes'] > 0) is healthy for row in result['embedded_volume']['measurements']),
            'dedicated guest filesystem did not stay in its required available/exhausted state')
    return result


def await_storage(case, initial, healthy, evidence):
    if not is_disk(case):
        return await_network(case, initial, healthy, evidence,
                             'storage-restored-network' if healthy else 'storage-offline-network')
    print('Dot embedded storage fault: operator must ' + ('restore free space on' if healthy else 'exhaust only') +
          ' the retained bounded disposable guest volume; never fill the workstation disk', flush=True)
    deadline = time.monotonic() + 600
    while time.monotonic() < deadline:
        result = probe(case, 'runtime_observer', 'embedded_enospc')
        unchanged_runtime(initial, result)
        if all((row['available_bytes'] > 0) is healthy for row in result['embedded_volume']['measurements']):
            evidence.observe(case['name'] + ('-storage-restored-volume' if healthy else '-storage-exhausted-volume'), result)
            return result
        time.sleep(2)
    raise Blocked('actual guest volume transition was not observed within the live barrier lifetime')


def enospc(error):
    return bool(re.search(r'no space left on device|\bENOSPC\b|os error 28\b', error, re.IGNORECASE))


class PendingUpload:
    """An actual direct HTTP/1 body withheld after the application's first poll.

    Axum 0.8.9 constructs Multipart without reading; this handler first checks
    its KB, then polls Multipart. Hyper 1.11.0 emits 100 on that body demand.
    A proxy-generated provisional response cannot qualify this fixture.
    """
    def __init__(self, host, kb, marker):
        boundary = 'dot17-' + uuid.uuid4().hex
        self.payload = (f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{marker}.txt"\r\n'
                        'Content-Type: text/plain\r\n\r\n' + marker + f'\r\n--{boundary}--\r\n').encode()
        self.path = '/api/uar/knowledge-bases/' + urllib.parse.quote(kb, safe='') + '/documents'
        url = urllib.parse.urlsplit(host.base)
        self.host = host
        self.socket = socket.create_connection((url.hostname, url.port or 80), timeout=15)
        self.socket.settimeout(180)
        self.headers = self.socket.makefile('rb', buffering=0)
        try:
            values = {'Host': url.netloc, 'Authorization': 'Bearer ' + host.credential(),
                'x-uar-workspace-id': host.workspace, 'Content-Type': 'multipart/form-data; boundary=' + boundary,
                'Content-Length': str(len(self.payload)), 'Expect': '100-continue', 'Connection': 'close'}
            require(all('\r' not in value and '\n' not in value for value in values.values()), 'invalid upload header')
            request = 'POST ' + self.path + ' HTTP/1.1\r\n' + ''.join(key + ': ' + value + '\r\n' for key, value in values.items()) + '\r\n'
            self.socket.sendall(request.encode())
            status = self.headers.readline(8193)
            require(status.startswith(b'HTTP/1.1 100 '), 'direct runtime did not demand the multipart body; commit fault is not armed')
            size = len(status)
            while True:
                line = self.headers.readline(8193)
                size += len(line)
                require(line and size <= 65536, 'invalid provisional response headers')
                if line == b'\r\n':
                    break
            self.provisional = {'status': 100, 'request_body_bytes_sent': 0,
                                'payload_sha256': hashlib.sha256(self.payload).hexdigest()}
        except Exception:
            self.close()
            raise

    def commit(self, require_enospc=False):
        self.socket.sendall(self.payload)
        response = http.client.HTTPResponse(self.socket)
        response.begin()
        raw = response.read(2 * 1024 * 1024 + 1)
        require(len(raw) <= 2 * 1024 * 1024, 'upload failure response too large')
        status = response.status
        result = {'status': status, 'response_sha256': hashlib.sha256(raw).hexdigest()}
        self.host.evidence.http.append({'host': self.host.label, 'method': 'POST', 'path': self.path, **result})
        # With valid multipart and the KB gate already passed, the existing
        # ingestion commit error maps to 500. Timeout/disconnect is not proof.
        require(status == 500, 'expected actual ingestion transaction failure, not timeout or early rejection')
        if require_enospc:
            require(enospc(raw.decode('utf-8', errors='replace')),
                    'ingestion transaction did not report actual ENOSPC; a full filesystem alone is not commit evidence')
            result['observed_commit_error'] = 'ENOSPC'
        return result

    def close(self):
        self.headers.close()
        self.socket.close()


def unresolved_events(host, run, call_ids, marker):
    value = host.api('GET', '/api/uar/runs/' + run + '/events?after=0')
    require(value.get('gapReason') is None, 'run event gap prevents execution/accounting observation')
    text = ''.join(row['data']['delta']['text'] for row in value['events'] if row['eventName'] == 'agui.message.delta')
    require(marker in text, 'actual completed provider text was lost during accounting outage')
    found = set()
    for row in value['events']:
        data = row['data']
        if row['eventName'] == 'agui.artifact' and data.get('artifact_id') == 'uar.accounting:unresolved':
            content = json.loads(data['content'])
            require(content['source'] == 'uar.accounting' and content['payload']['state'] == 'pending',
                    'accounting outage was falsely projected as settled')
            found.add(content['payload']['callId'])
    require(found == set(call_ids), 'actual failed settlement did not expose exact unresolved call identity')
    return value


def no_uploaded_content(host, kb, marker):
    base = '/api/uar/knowledge-bases/' + urllib.parse.quote(kb, safe='')
    require(host.api('GET', base + '/documents') == [], 'failed commit left a document/job metadata promise')
    result = host.api('POST', base + '/search', {'query': marker, 'limit': 10, 'min_score': 0})
    require(result['results'] == [], 'failed commit produced indexed content in the dedicated KB')
    return {'document_count': 0, 'indexed_result_count': 0}


def storage_exercise(config, case, evidence):
    initial = storage_observation(case, True)
    host = storage_host(case, evidence, initial)
    require(host.base == config['runtime_urls'][0].rstrip('/'), 'storage fault must use the surviving runtime')
    details = case['storage_faults']
    marker = 'dot17-settlement-' + uuid.uuid4().hex
    upload_marker = 'dot17-upload-' + uuid.uuid4().hex
    kb = host.api('POST', '/api/uar/knowledge-bases/', {'name': upload_marker}, expect=201)['id']
    docs_path = '/api/uar/knowledge-bases/' + urllib.parse.quote(kb, safe='') + '/documents'
    require(host.api('GET', docs_path) == [], 'new dedicated KB unexpectedly contains documents')
    body = {'artifact': details['artifact'], 'run_credentials': details['run_credentials'],
            'input': json.dumps({'dot_action': 'text', 'marker': marker, 'wire_mode': 'settlement_barrier'})}
    run = host.api('POST', '/api/uar/runs', body)['run_id']
    poll(lambda: controlled_peer(case, marker),
         lambda row: bool(row and row['state']['waiting'] and not row['state']['released']),
         'actual admitted upstream response barrier unavailable')
    calls = host.api('GET', '/api/uar/accounting/calls/' + run)
    require(len(calls) == 1 and calls[0]['state'] == 'pending', 'real already-dispatched call has no durable pending admission')
    call_ids = [row['admission']['identity']['call_id'] for row in calls]
    global_path = '/api/uar/accounting/scopes/global/global'
    scope = host.api('GET', global_path, principal='host')
    require(scope['pending_calls'] > 0, 'provider call did not reserve durable liability')
    upload = PendingUpload(host, kb, upload_marker)
    try:
        evidence.observe(case['name'] + '-storage-armed', {'runtime': initial, 'run_id': run,
            'pending_calls': calls, 'scope': scope, 'kb_id': kb, 'upload_marker': upload_marker,
            'provider_marker': marker, 'upload_provisional': upload.provisional,
            'provider_rows': provider_rows(case, marker)})
        offline = await_storage(case, initial, False, evidence)
        upload_result = upload.commit(require_enospc=is_disk(case))
        require(controlled_peer(case, marker, True)['released'] is True, 'actual provider barrier release failed')
        poll(lambda: controlled_peer(case, marker),
             lambda row: bool(row and row['state']['terminal_usage_sent']), 'real terminal provider usage not emitted')
        terminal = poll(lambda: host.api('GET', '/api/uar/runs/' + run),
                        lambda row: row['status'] in ('done', 'error', 'cancelled'), 'run never completed after peer terminal usage')
        require(terminal['status'] != 'cancelled', 'cancelled execution cannot qualify a completed usage settlement outage')
        events = unresolved_events(host, run, call_ids, marker)
        if is_disk(case):
            # Reads can remain available while the actual KV transaction cannot
            # append. Do not demand a network outage response from disk failure.
            unavailable = host.api('GET', '/api/uar/accounting/calls/' + run, expect=(200, 503))
            require(unavailable == calls or isinstance(unavailable, dict) and
                    unavailable.get('error') == 'accounting_storage_unavailable',
                    'disk failure changed durable pending usage or returned an unrelated error')
            errors = storage_observation(case, False, initial['embedded_volume']['log'], call_ids)
            unchanged_runtime(offline, errors)
            terminal_errors = [row for row in errors['embedded_volume']['settlement_errors']
                               if row['message'] == 'Execution completed with unresolved durable accounting' and enospc(row['error'])]
            require({row['call_id'] for row in terminal_errors} == set(call_ids),
                    'exact terminal accounting transaction lacks runtime-emitted ENOSPC evidence')
            evidence.observe(case['name'] + '-storage-enospc-transactions',
                {'upload': upload_result, 'settlement_errors': terminal_errors,
                 'runtime': errors, 'claim': 'transaction errors observed; execution/cleanup success not inferred'})
        else:
            unavailable = host.api('GET', '/api/uar/accounting/calls/' + run, expect=503)
            require(unavailable.get('error') == 'accounting_storage_unavailable', 'wrong accounting inspection failure')
        denied_marker = 'dot17-admission-' + uuid.uuid4().hex
        denied_body = dict(body, input=json.dumps({'dot_action': 'text', 'marker': denied_marker}))
        denied = host.api('POST', '/api/uar/runs', denied_body, expect=(200, 503))
        if isinstance(denied, dict) and denied.get('run_id'):
            refused = poll(lambda: host.api('GET', '/api/uar/runs/' + denied['run_id']),
                           lambda row: row['status'] in ('done', 'error', 'cancelled'), 'outage admission did not terminate')
            require(refused['status'] == 'error', 'outage admitted another executable run')
        require(not provider_rows(case, denied_marker), 'new outage admission dispatched provider work')
        unchanged_runtime(offline, storage_observation(case, False))
        evidence.observe(case['name'] + '-storage-failures', {'upload': upload_result, 'terminal': terminal,
            'unresolved_events': events, 'provider_rows': provider_rows(case, marker),
            'accounting_unavailable': unavailable, 'new_admission': denied, 'new_provider_dispatches': 0})
        restored = await_storage(case, initial, True, evidence)
        rows = host.api('GET', '/api/uar/accounting/calls/' + run)
        require(rows == calls, 'uncommitted terminal usage changed or settled the durable pending call')
        final_scope = host.api('GET', global_path, principal='host')
        require(final_scope == scope, 'storage outage released reservation or fabricated a settled charge')
        upload_absence = no_uploaded_content(host, kb, upload_marker)
        require(not provider_rows(case, denied_marker), 'restoration replayed a refused admission')
        upstream = provider_rows(case, marker)
        require(sum('instruction' in row for row in upstream) == 1 and
                any(row.get('event') == 'terminal_usage_sent' and row.get('tokens') == 5 for row in upstream),
                'real provider request/terminal usage evidence missing or replayed')
        persist(case, evidence, 'storage-exercise', {'runtime': restored, 'run_id': run, 'calls': rows,
            'scope': final_scope, 'kb_id': kb, 'docs_path': docs_path, 'upload_marker': upload_marker,
            'provider_marker': marker, 'denied_marker': denied_marker, 'upload': upload_result,
            'upload_absence': upload_absence, 'provider_rows': upstream})
    finally:
        upload.close()
    raise Blocked('real commit/settlement failures retained; operator must restart the dedicated runtime, then run storage-cold')


def storage_cold(config, case, evidence):
    saved = prior(case, evidence, 'storage-exercise')
    observed = storage_observation(case, True)
    host = storage_host(case, evidence, observed)
    before = saved['runtime']
    require(observed['machine_id'] == before['machine_id'] and observed['boot_id'] == before['boot_id'] and
            observed['process']['identity'] != before['process']['identity'], 'actual cold runtime process not observed')
    for field in ('config_sha256', 'executable_sha256'):
        require(observed['process'][field] == before['process'][field], 'cold accounting reader installation changed')
    if is_disk(case):
        same_volume(before, observed)
    calls = host.api('GET', '/api/uar/accounting/calls/' + saved['run_id'])
    require(calls == saved['calls'] and all(row['state'] == 'pending' for row in calls),
            'cold storage read forgot unresolved usage')
    scope = host.api('GET', '/api/uar/accounting/scopes/global/global', principal='host')
    require(scope == saved['scope'], 'cold restart released or settled unresolved spend')
    upload_absence = no_uploaded_content(host, saved['kb_id'], saved['upload_marker'])
    require(not provider_rows(case, saved['denied_marker']), 'cold restart replayed refused provider dispatch')
    # The completed cold stage retains the measured, source-bound prior phase;
    # its deliberately blocked standalone receipt is never credited as a pass.
    evidence.observe(case['name'] + '-storage-exercise', saved)
    persist(case, evidence, 'storage-cold', {'runtime': observed, 'calls': calls, 'scope': scope,
            'kb_id': saved['kb_id'], 'upload_absence': upload_absence, 'new_provider_dispatches': 0})


def run_recovery_faults(config, evidence):
    settings = config['supervisor']
    references = settings.get('fault_cases')
    if not references:
        raise Blocked('dedicated staged recovery fault cases unavailable')
    for reference in references:
        case = read(reference['private_case_file'])
        case['private_case_file'] = reference['private_case_file']
        require(re.fullmatch(r'[A-Za-z0-9_-]+', case['name']), 'invalid fault evidence name')
        require(case['target'] == {'kind': 'checkpoint_epoch', 'runtime_epoch': case['runtime_epoch']},
                'fault driver accepts exact checkpoint epoch targets only')
        stage = reference['stage']
        if stage == 'stale-executor':
            raise Blocked('no public production mechanism submits ClaimIntent/checkpoint writes with an old runtime epoch; '
                          'physical exclusion terminates that executor. No stale-executor evidence was fabricated')
        if stage == 'restart-prepare':
            restart_prepare(settings, case, evidence)
        elif stage == 'restart-complete':
            restart_complete(settings, case, evidence)
        elif stage == 'storage-exercise':
            storage_exercise(settings, case, evidence)
        elif stage == 'storage-cold':
            storage_cold(settings, case, evidence)
        else:
            operation = None
            for purpose in ('persistence', 'predecessor_controller'):
                for phase, handler in (('prepare', fault_prepare), ('offline', fault_offline), ('restored', fault_restored)):
                    if stage == purpose + '-' + phase:
                        operation = (handler, purpose)
            require(operation is not None, 'unknown fixed recovery fault stage')
            operation[0](settings, case, evidence, operation[1])
