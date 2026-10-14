"""Real eligible-local governance transitions and retained explicit approval.

Only the full-phase runner calls this module. It owns isolated child processes,
an actual provider peer, one temporary Cedar file and real host API mutations.
"""
import copy
import json
from pathlib import Path
import uuid

from acceptance import ROOT, Blocked, Host, poll, require
from journeys import effects, finish, pending, start_run, tool_input
from provider import Provider
from settings_journey import admin_headers, mutate


def snapshot(host, run, anonymous=False):
    return host.api('GET', '/api/uar/runs/' + run + '/events?after=0',
                    principal=None if anonymous else 'owner')


def completed(host, run, anonymous=False):
    return poll(lambda: host.api('GET', '/api/uar/runs/' + run,
                                 principal=None if anonymous else 'owner'),
                lambda row: row['status'] in ('done', 'error', 'cancelled'), 'governance run not terminal')


def successful_tool(events, name):
    return any(row['eventName'] == 'agui.tool_result' and row['data']['name'] == name
               and row['data']['success'] for row in events['events'])


def denied_tool(events, name):
    return any(row['eventName'] == 'agui.tool_call.denied' and row['data']['name'] == name
               for row in events['events'])


def local_cases(settings, evidence):
    spec = settings['local']
    host = Host(spec['host'], evidence, 'eligible-local-governance')
    directory = Path(host.spec['working_directory']).resolve()
    require(spec.get('isolated_working_directory') is True and not directory.is_relative_to(ROOT),
            'governance fixture needs an isolated working directory outside source')
    policy_path = directory / 'policies/dot17-governance.cedar'
    policy_path.parent.mkdir(parents=True, exist_ok=True)
    with open(policy_path, 'x') as stream:
        stream.write('permit (principal, action, resource);\n'
                     'forbid (principal, action == Action::"execute_tool", '
                     'resource == Tool::"native_system_info");\n')
    provider = Provider(spec['provider_port'], evidence.directory / 'governance-provider.jsonl')
    provider.start()
    saved = {}
    results = []
    try:
        host.start()
        status_path = '/api/uar/settings/governance/status'
        status = host.api('GET', status_path, principal='admin', headers=admin_headers(host))
        require(status['may_disable'] is True and status['jwt_required'] is False,
                'local fixture is not eligible for governance Off')
        saved['governance.enabled'] = mutate(host, 'governance.enabled', False)['saved']
        off = host.api('GET', status_path, principal='admin', headers=admin_headers(host))
        require(off['effective_state'] == 'off', 'committed local Off not effective')
        def launch(name, args, anonymous=False):
            body = {'artifact': spec['artifact'], 'input': tool_input(tool=name, arguments=args)}
            if not anonymous:
                body['run_credentials'] = spec['run_credentials']
            return host.api('POST', '/api/uar/runs', body,
                            principal=None if anonymous else 'owner')['run_id']
        # This installed native descriptor is ReadOnly/NotRequired. External
        # MCP annotations cannot turn web_search into a non-Required descriptor.
        run = launch('native_system_info', {}, anonymous=True)
        completed(host, run, anonymous=True)
        events = snapshot(host, run, anonymous=True)
        require(successful_tool(events, 'native_system_info') and not denied_tool(events, 'native_system_info'),
                'eligible anonymous non-Required tool did not bypass automatic Cedar')
        results.append({'case': 'automatic-cedar-off', 'run_id': run, 'anonymous': True})
        catalog = host.api('GET', '/api/uar/discovery/tools', principal=None)
        native_search = [row['name'] for row in catalog['tools']
                         if row['source'] == 'native_tool' and row['name'] in ('web_search', 'native_web_search')]
        native_search += [row['name'] for row in catalog['built_in_tools']
                          if row['name'] in ('web_search', 'native_web_search')]
        if native_search:
            raise Blocked('installed native web_search now exists; qualify its actual descriptor instead of claiming absence')
        advertised = sorted({name for row in provider.rows() for name in row['tool_names']})
        require('native_system_info' in advertised, 'actual non-Required alternative was not advertised')
        evidence.observe('governance-web-search-applicability', {
            'status': 'unavailable', 'native_web_search': native_search,
            'actual_advertised_tool_names': advertised, 'actual_discovery': catalog,
            'alternative_tool': 'native_system_info', 'alternative_run_id': run,
            'alternative_success_observed': True, 'mcp_approval_contract': 'Required'})
        mutate(host, 'governance.enabled', True)
        run = launch('native_system_info', {})
        completed(host, run)
        events = snapshot(host, run)
        require(denied_tool(events, 'native_system_info') and not successful_tool(events, 'native_system_info'),
                'Cedar On did not deny the installed native tool')
        require(not pending(host, run), 'Cedar denial incorrectly offered approval override')
        results.append({'case': 'cedar-on-denial', 'run_id': run})
        mutate(host, 'governance.enabled', False)
        # Explicit policy remains authoritative in the approved parent plan.
        original = host.api('GET', '/api/uar/settings/run_policy.global', principal='admin', headers=admin_headers(host))['saved']
        saved['run_policy.global'] = original
        for disposition in ('deny', 'ask'):
            policy = copy.deepcopy(original)
            policy['tool_approval'] = disposition
            mutate(host, 'run_policy.global', policy)
            marker = uuid.uuid4().hex
            run = launch('native_echo', {'message': marker})
            if disposition == 'deny':
                completed(host, run)
                require(denied_tool(snapshot(host, run), 'native_echo'), 'explicit Deny was bypassed')
            else:
                approval = poll(lambda: pending(host, run), bool, 'explicit Ask was bypassed')
                require(not successful_tool(snapshot(host, run), 'native_echo'), 'Ask executed before decision')
                approval_id = approval.get('approval_id') or approval['approvalId']
                host.api('POST', '/api/uar/runs/' + run + '/tool-approval',
                         {'approval_id': approval_id, 'approved': True})
                finish(host, run)
                require(successful_tool(snapshot(host, run), 'native_echo'), 'approved Ask did not execute')
            results.append({'case': 'explicit-policy-' + disposition, 'run_id': run})
        mutate(host, 'run_policy.global', original)
        unselected = copy.deepcopy(spec['artifact'])
        unselected['policy']['tools']['deny'] = list(set(
            unselected['policy']['tools'].get('deny', []) + ['native_echo']))
        marker = uuid.uuid4().hex
        run = host.api('POST', '/api/uar/runs', {'artifact': unselected,
            'input': tool_input(tool='native_echo', arguments={'message': marker}),
            'run_credentials': spec['run_credentials']})['run_id']
        completed(host, run)
        rows = [row for row in provider.rows() if row['instruction'].get('arguments', {}).get('message') == marker]
        require(rows and all('native_echo' not in row['tool_names'] for row in rows),
                'Off exposed a tool excluded from the selected artifact')
        require(not successful_tool(snapshot(host, run), 'native_echo'), 'unselected tool executed')
        results.append({'case': 'unselected-tool-unavailable', 'run_id': run})
        for approved in (False, True):
            marker = uuid.uuid4().hex
            run = start_run(host, spec, tool_input('success', marker))
            approval = poll(lambda: pending(host, run), bool, 'Required descriptor did not pause under Off')
            require(not effects(spec['effect_trace'], marker), 'Required tool executed before decision')
            approval_id = approval.get('approval_id') or approval['approvalId']
            host.api('POST', '/api/uar/runs/' + run + '/tool-approval',
                     {'approval_id': 'stale-' + approval_id, 'approved': True}, expect=(404, 409))
            require(not effects(spec['effect_trace'], marker), 'stale approval executed effect')
            body = {'approval_id': approval_id, 'approved': approved}
            host.api('POST', '/api/uar/runs/' + run + '/tool-approval', body)
            completed(host, run)
            require(len(effects(spec['effect_trace'], marker)) == int(approved), 'Required disposition effect count differs')
            host.api('POST', '/api/uar/runs/' + run + '/tool-approval', body, expect=(200, 409))
            require(len(effects(spec['effect_trace'], marker)) == int(approved), 'duplicate decision replayed effect')
            results.append({'case': 'required-' + ('approve' if approved else 'deny'), 'run_id': run,
                            'approval_id': approval_id})
        for enabled in (True, False, True, False):
            mutate(host, 'governance.enabled', enabled)
            host.api('GET', status_path, principal='admin', headers=admin_headers(host))
        for log in host.logs:
            log.flush()
        warnings = [line for line in Path(host.logs[-1].name).read_text().splitlines()
                    if 'governance.inactive_local_mode' in line]
        require(len(warnings) == 1 and 'required_approval_enforced' in warnings[0],
                'inactive governance warning count or retained-approval meaning differs')
        evidence.observe('eligible-local-governance', {'cases': results, 'warning_count': len(warnings),
                         'scope': 'installed non-Required native tool; explicit Ask/Deny and MCP Required preserved',
                         'web_search_non_required': 'not supplied by this installed native tool set'})
        host.restart()
        status = host.api('GET', status_path, principal='admin', headers=admin_headers(host))
        require(status['effective_state'] == 'off', 'saved Off did not survive eligible restart')
        warnings = [line for line in Path(host.logs[-1].name).read_text().splitlines()
                    if 'governance.inactive_local_mode' in line]
        require(len(warnings) == 1, 'new process did not emit exactly one inactive warning')
        evidence.observe('governance-warning-new-process', {'warning_count': 1, 'boot_instance_id': status['boot_instance_id']})
    finally:
        try:
            if host.process is not None and host.process.poll() is None:
                for key, value in saved.items():
                    mutate(host, key, value)
        finally:
            try:
                host.stop()
            finally:
                provider.stop()
                policy_path.unlink()


def run_governance(spec, evidence):
    settings = spec.get('governance')
    if not settings:
        raise Blocked('isolated eligible-local and mandatory-auth governance configuration unavailable')
    if evidence.profile == 'local_surreal_kv':
        local_cases(settings, evidence)
    host = Host(settings['mandatory_host'], evidence, 'mandatory-governance')
    host.start()
    try:
        headers = admin_headers(host)
        status = host.api('GET', '/api/uar/settings/governance/status', principal='admin', headers=headers)
        require(status['effective_state'] == 'required' and not status['may_disable'], 'authenticated profile allows Off')
        path = '/api/uar/settings/governance.enabled'
        current = host.api('GET', path, principal='admin', headers=headers)
        host.api('PUT', path, {'value': False, 'expected_revision': current['revision']},
                 principal='admin', headers=headers, expect=(400, 403, 409))
        after = host.api('GET', '/api/uar/settings/governance/status', principal='admin', headers=headers)
        require(after['effective_enabled'] is True, 'refused disable weakened actual enforcement')
        evidence.observe('mandatory-governance-refusal', {'before': status, 'after': after})
    finally:
        host.stop()
