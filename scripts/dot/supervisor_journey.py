"""Staged real native/CRI qualification. Never implicitly reboot a machine.

Prepared cases identify real runs/claims and an actual detached fixture child.
Only live observations and real host API transitions count as evidence. The
operator owns isolated service installation and explicit reboot between stages.
"""
import base64
import importlib.util
import json
import os
from pathlib import Path
import re
import ssl
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

from acceptance import ROOT, Blocked, Host, digest, poll, read, require, save


def helper():
    path = ROOT / 'scripts/supervisor/recovery_scenarios.py'
    spec = importlib.util.spec_from_file_location('dot17_recovery_http', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def observe(case):
    remote = case['observer']
    require(remote.get('dedicated_qualification_host') is True, 'dedicated host declaration required')
    require(re.fullmatch(r'[A-Za-z0-9_.@-]+', remote['ssh_target']) and
            not remote['ssh_target'].startswith('-'), 'invalid explicit SSH destination')
    require(remote['python'] in ('python', 'python3', 'python.exe', 'python3.exe'),
            'observer interpreter must be a fixed Python executable name')
    known_hosts = Path(remote['known_hosts']).resolve()
    require(known_hosts.is_file(), 'pinned SSH host keys required')
    payload = {key: remote[key] for key in ('controller_source', 'controller_config', 'trace_file')}
    payload.update(runtime_epoch=case['runtime_epoch'], marker=case['marker'],
                   effect_markers=[item['marker'] for item in case.get('continuation_effects', [])],
                   controller_sha256=digest(ROOT / 'scripts/supervisor/controller.py'))
    encoded = base64.urlsafe_b64encode(json.dumps(payload).encode()).decode()
    command = ['ssh', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes',
               '-o', 'UserKnownHostsFile=' + str(known_hosts), '-o', 'ConnectTimeout=15',
               remote['ssh_target'], remote['python'], '-', encoded]
    completed = subprocess.run(command, input=(ROOT / 'scripts/dot/supervisor_probe.py').read_bytes(),
                               capture_output=True, timeout=120)
    require(completed.returncode == 0, 'independent dedicated-host observation unavailable')
    require(len(completed.stdout) <= 4 * 1024 * 1024, 'observer response limit')
    value = json.loads(completed.stdout)
    require(value['machine_id'] == remote['expected_machine_id'], 'wrong dedicated host')
    require(value['registration']['runtime_epoch'] == case['runtime_epoch'], 'wrong predecessor epoch')
    return value


def live_child(case, evidence, label):
    before = observe(case)
    time.sleep(1.5)
    after = observe(case)
    require(after['registration'] == before['registration'], 'predecessor identity changed')
    require(after['descendant'] == before['descendant'], 'fixture child replaced')
    require(after['descendant_process']['same_process'] is True, 'real detached child not alive')
    require(after['heartbeat_count'] > before['heartbeat_count'], 'detached child heartbeat not advancing')
    require(after['descendant']['boot_identity'] == after['boot_id'], 'child belongs to another boot')
    evidence.observe(label + '-live-domain', {'before': before, 'after': after})
    return after


def stopped_child(case, before, evidence, label):
    after = observe(case)
    time.sleep(1.5)
    later = observe(case)
    require(after['registration'] == before['registration'] == later['registration'], 'domain identity changed')
    require(after['descendant'] == before['descendant'] == later['descendant'], 'fixture descendant replaced')
    require(after['heartbeat_count'] == later['heartbeat_count'], 'excluded descendant still executing')
    require(after['effect_count'] == later['effect_count'] == before['effect_count'], 'exclusion replayed an effect')
    evidence.observe(label + '-stopped-domain', {'after': after, 'later': later})
    return later


def native_prepare(config, case, evidence):
    continuation_contract(case)
    before = live_child(case, evidence, case['name'])
    domain = before['registration']['domain']
    require(domain['kind'] == 'native_boot' and domain['boot_id'] == before['boot_id'],
            'native prepare needs the original boot')
    require(before['root_process']['same_process'] is not True, 'crash the owned fixture root before this stage')
    request_case = dict(case, expect_excluded=False)
    request_case.pop('resume', None)
    request_case.pop('team_reclaim', None)
    result = helper().run_case(config, request_case)
    require(observe(case)['exclusion'] is None, 'same-boot refusal committed exclusion')
    save(case['continuation_file'], {'schema_version': 1, 'source': evidence.identity['source'],
         'case_sha256': digest(case['private_case_file']), 'before': before, 'same_boot_refusal': result})
    evidence.observe(case['name'] + '-same-boot-refusal', result)
    raise Blocked('native same-boot refusal observed; explicitly reboot this dedicated host, then run native-complete')


def native_complete(config, case, evidence):
    continuation_contract(case)
    saved = read(case['continuation_file'])
    require(saved['schema_version'] == 1 and saved['source'] == evidence.identity['source'],
            'source changed across native qualification stages')
    require(saved['case_sha256'] == digest(case['private_case_file']), 'native case changed after preparation')
    before = saved['before']
    after = stopped_child(case, before, evidence, case['name'])
    domain = before['registration']['domain']
    require(after['machine_id'] == before['machine_id'] == domain['machine_id'], 'reboot used a different host')
    require(after['boot_id'] != before['boot_id'], 'same-boot native takeover remains forbidden')
    result = helper().run_case(config, dict(case, expect_excluded=True))
    final = observe(case)
    require(final['exclusion'] is not None, 'controller failed to retain actual exclusion')
    evidence.observe(case['name'] + '-reboot-exclusion', {'http': result, 'observation': final})
    complete_continuation(config, case, before, result, evidence, 'native')


def kubernetes(config, case, evidence):
    continuation_contract(case)
    require(case.get('authorize_node_inhibition') is True,
            'this dedicated-node case must explicitly authorize retained kubelet inhibition')
    before = live_child(case, evidence, case['name'])
    domain = before['registration']['domain']
    require(domain['kind'] == 'kubernetes', 'CRI case requires a registered Pod domain')
    for key in ('pod_cgroup', 'kubelet_cgroup'):
        require(before[key].get('matches_retained') and before[key]['events']['populated'] == '1',
                'original live cgroup unavailable')
    require(before['kubelet']['ActiveState'] == 'active', 'kubelet was not live before exclusion')
    result = helper().run_case(config, dict(case, expect_excluded=True))
    after = stopped_child(case, before, evidence, case['name'])
    require(after['boot_id'] == before['boot_id'] == domain['node_boot_id'], 'node reboot invalidates CRI scenario')
    require(after['exclusion'] is not None, 'controller did not retain physical observation')
    proof = after['exclusion']
    require(proof['descendant_cgroup_empty'] is True and proof['kubelet_cgroup_empty'] is True,
            'controller did not observe original subtrees empty')
    for key in ('pod_cgroup', 'kubelet_cgroup'):
        # CRI may remove an already observed empty group. Absence alone never
        # qualifies; exact pre-observation plus retained controller proof does.
        if after[key]['present']:
            require(after[key]['matches_retained'] and after[key]['events']['populated'] == '0',
                    'retained cgroup remains populated or was replaced')
    require(after['kubelet']['ActiveState'] in ('inactive', 'failed') and
            after['kubelet']['MainPID'] == '0' and
            after['kubelet']['UnitFileState'] in ('masked', 'masked-runtime'), 'node restart authority not inhibited')
    require(after['sandbox']['id'] == domain['sandbox_id'] and
            after['sandbox']['state'] == 'SANDBOX_NOTREADY', 'original sandbox not stopped')
    require(sorted(row['id'] for row in after['containers']) == sorted(domain['container_ids']) and
            all(row['state'] == 'CONTAINER_EXITED' for row in after['containers']), 'original containers not stopped')
    evidence.observe(case['name'] + '-cri-exclusion', {'http': result, 'observation': after,
                     'node_inhibition_retained': True})
    complete_continuation(config, case, before, result, evidence, 'kubernetes')


def continuation_contract(case):
    role = case.get('case_role')
    require(role in ('safe-checkpoint', 'uncertain-checkpoint', 'safe-team', 'uncertain-team'),
            'physical recovery requires an explicit real continuation case role')
    effects = case.get('continuation_effects', [])
    require(effects and len({item['marker'] for item in effects}) == len(effects),
            'distinct actual continuation effect markers required')
    require(all(isinstance(item['marker'], str) and item['marker'] != case['marker'] and
                item['expected_delta'] in (0, 1) for item in effects),
            'continuation markers must distinguish safe execution from uncertain holds')
    require(any(item['expected_delta'] == 1 for item in effects) if role.startswith('safe-') else
            all(item['expected_delta'] == 0 for item in effects), 'continuation effect disposition differs from role')
    if role.endswith('checkpoint'):
        require('resume' in case and 'team_reclaim' not in case and
                case['resume']['expected_successes'] == (1 if role.startswith('safe-') else 0),
                'checkpoint role needs its real competing resume outcome')
    else:
        require('team_reclaim' in case and 'resume' not in case,
                'team role needs its real queued/uncertain ownership reclaim')
        team = case['team_reclaim']
        require(team['queued_attempt_ids'] if role == 'safe-team' else team['uncertain_attempt_ids'],
                'empty team catalog cannot qualify recovery')
        if role == 'uncertain-team':
            require(not team['queued_attempt_ids'], 'uncertain-only case cannot dispatch queued effects')
        dispatches = case.get('queued_dispatches', [])
        require({item['attempt_id'] for item in dispatches} == set(team['queued_attempt_ids']),
                'every transferred queued attempt requires its real dispatch')


def complete_continuation(config, case, before, result, evidence, mechanism):
    for run_id in result.get('resumed_run_ids', []):
        resume = case['resume']
        host = Host({'base_url': config['runtime_urls'][0], 'workspace': resume['workspace_id'],
                     'credentials': {'owner': resume['owner_token_env']}}, evidence, case['name'])
        poll(lambda: host.api('GET', '/api/uar/runs/' + urllib.parse.quote(run_id, safe='')),
             lambda row: row['status'] == 'done', 'resumed real run did not finish')
    for item in case.get('queued_dispatches', []):
        host = Host({'base_url': config['runtime_urls'][0], 'workspace': item['workspace_id'],
                     'credentials': {'owner': item['owner_token_env']}}, evidence, case['name'])
        path = '/api/v1/collaboration/team-instances/' + urllib.parse.quote(item['team_id'], safe='')
        path += '/attempts/' + urllib.parse.quote(item['attempt_id'], safe='') + '/dispatch'
        team_path = '/api/v1/collaboration/team-instances/' + urllib.parse.quote(item['team_id'], safe='')
        current = host.api('GET', team_path)
        body = dict(item['body'], expectedTeamRevision=current.get('data', current)['revision'])
        host.request('POST', path, body, expect=200,
                     headers={'x-uar-admin-key': os.environ[item['admin_key_env']]})
    expected = {item['marker']: before['continuation_effect_counts'][item['marker']] + item['expected_delta']
                for item in case['continuation_effects']}
    final = poll(lambda: observe(case), lambda row: row['continuation_effect_counts'] == expected,
                 'continuation did not produce the exact actual effect count')
    time.sleep(1.5)
    later = observe(case)
    require(later['continuation_effect_counts'] == expected and later['exclusion'] == final['exclusion'],
            'recovered work duplicated an effect or changed exclusion authority')
    evidence.observe('supervisor-' + mechanism + '-' + case['case_role'], {
        'runtime_epoch': case['runtime_epoch'], 'http': result,
        'actual_effect_counts': expected, 'independent_observations': [final, later],
        'unknown_effects_settled': False})


def controller_security(config, case, evidence):
    before = observe(case)
    settings = case['controller_security']
    endpoint = settings['endpoint'].rstrip('/')
    require(urllib.parse.urlsplit(endpoint).scheme == 'https', 'controller security case requires TLS')
    def request(body, certificate, key):
        context = ssl.create_default_context(cafile=settings['server_ca'])
        context.load_cert_chain(certificate, key)
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, *_):
                raise AssertionError('controller redirected credential-bearing request')
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
                                              urllib.request.HTTPSHandler(context=context))
        query = urllib.request.Request(endpoint + '/v1/containment/register',
            data=json.dumps(body).encode(), headers={'Content-Type': 'application/json'}, method='POST')
        try:
            response = opener.open(query, timeout=30)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            return response.status, json.loads(response.read(65536))
    registration = before['registration']
    domain = registration['domain']
    body = {key: registration[key] for key in ('runtime_epoch', 'team_incarnation',
                                              'qualification_id', 'deployment_id')}
    body['process_id'] = int(domain['root_process_identity'].split(':', 1)[0]) if domain['kind'] == 'native_boot' else 2
    certificate, key = settings['authorized_certificate'], settings['authorized_key']
    status, value = request(body, certificate, key)
    require(status == 200 and value == registration, 'authorized original certificate baseline failed')
    # Use a different certificate signed by the test client CA but absent from
    # the controller fingerprint allowlist. A network/TLS error is not a pass.
    status, _ = request(body, settings['wrong_certificate'], settings['wrong_key'])
    require(status == 409, 'unallowlisted client certificate obtained registration')
    foreign = dict(body, deployment_id=settings['foreign_deployment_id'], runtime_epoch=str(uuid.uuid4()))
    require(foreign['deployment_id'] != body['deployment_id'], 'foreign deployment must differ')
    status, _ = request(foreign, certificate, key)
    require(status == 409, 'certificate registered another deployment')
    after = observe(case)
    require(after['domain_count'] == before['domain_count'] and after['registration'] == registration,
            'rejected controller authority mutated registration state')
    evidence.observe(case['name'] + '-controller-security', {'wrong_certificate_status': 409,
                     'cross_deployment_status': 409, 'registration_unchanged': True})
    evidence.observe('supervisor-' + config['mechanism'] + '-controller-security', {
        'wrong_certificate_status': 409, 'cross_deployment_status': 409,
        'before': before, 'after': after})


def run_supervisor(config, evidence):
    settings = config.get('supervisor')
    if not settings:
        raise Blocked('dedicated native/CRI qualification configuration unavailable')
    require(settings['cases'], 'real supervisor cases required')
    mechanism = settings.get('mechanism')
    require(mechanism in ('native', 'kubernetes'), 'explicit supervisor mechanism required')
    roles = []
    blocked = []
    for reference in settings['cases']:
        # Credentials and resume bodies stay in private operator inputs. The
        # case file is immutable across a reboot; stage is an outer selector.
        case = read(reference['private_case_file'])
        case['private_case_file'] = reference['private_case_file']
        require(re.fullmatch(r'[A-Za-z0-9_-]+', case['name']), 'invalid evidence name')
        stage = reference['stage']
        if stage in ('native-prepare', 'native-complete', 'kubernetes'):
            require((stage == 'kubernetes') == (mechanism == 'kubernetes'),
                    'case stage differs from selected containment mechanism')
            roles.append(case.get('case_role'))
        operation = {'native-prepare': native_prepare, 'native-complete': native_complete,
                     'kubernetes': kubernetes, 'controller-security': controller_security}.get(stage)
        require(operation is not None, 'unknown concrete supervisor stage')
        try:
            operation(settings, case, evidence)
        except Blocked as error:
            blocked.append(case['name'] + ': ' + str(error))
    if blocked:
        raise Blocked('; '.join(blocked))
    if roles:
        require(sorted(roles) == sorted(('safe-checkpoint', 'uncertain-checkpoint',
                                       'safe-team', 'uncertain-team')),
                'all four continuation roles must qualify exactly once')
