"""Real disabled-host capability discovery and operation refusal."""
import uuid

from acceptance import Blocked, Host, poll, require
from journeys import helper, start_run, tool_input
from provider import Provider


def run_unsupported(spec, evidence):
    settings = spec.get('unsupported')
    if not settings:
        raise Blocked('isolated memory-disabled, supervisor-unconfigured profile unavailable')
    host = Host(settings['host'], evidence, 'unsupported-capabilities')
    provider = Provider(settings['provider_port'], evidence.directory / 'unsupported-provider.jsonl')
    provider.start()
    try:
        host.start()
        inventory = helper('dot16_capability_inventory.py').inventory(host.base, host.credential())
        operations = {row['id']: row for row in inventory['operations']}
        for identifier in ('memory.policy.read', 'memory.policy.update', 'recovery.challenge'):
            require(all(not entry['available'] for entry in operations[identifier]['entry_points']),
                    'disabled capability advertised as usable: ' + identifier)
        host.api('GET', '/api/memory/policy', expect=503)
        host.api('PUT', '/api/memory/policy', {'enabled': False}, expect=503)
        epoch = str(uuid.uuid4())
        host.api('POST', '/api/uar/recovery/supervisor/challenges',
                 {'runtime_epoch': epoch, 'target': {'kind': 'checkpoint_epoch', 'runtime_epoch': epoch}},
                 principal='host', expect=503)
        run = start_run(host, settings, tool_input(tool='memory_capture_policy', arguments={}))
        terminal = poll(lambda: host.api('GET', '/api/uar/runs/' + run),
                        lambda row: row['status'] in ('done', 'error', 'cancelled'),
                        'disabled memory control did not terminate')
        events = host.api('GET', '/api/uar/runs/' + run + '/events?after=0')
        results = [row['data'] for row in events['events'] if row['eventName'] == 'agui.tool_result'
                   and row['data']['name'] == 'memory_capture_policy']
        require(results and all(row['success'] is False for row in results),
                'disabled native memory control was missing or reported successful work')
        evidence.observe('unsupported-assistant-controls', {'inventory': inventory, 'run_id': run,
            'terminal': terminal['status'], 'native_tool_results': results,
            'memory_http_status': 503, 'supervisor_http_status': 503,
            'scope': 'actual disabled memory and unconfigured supervisor; no unsupported backend parity claim'})
    finally:
        try:
            host.stop()
        finally:
            provider.stop()
