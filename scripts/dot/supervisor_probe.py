"""Read-only dedicated-host observer, streamed over authenticated SSH by Dot17.

No controller construction, service mutation, process termination or database
write is performed here. Missing observations fail the caller's qualification.
"""
import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys


def observe(options):
    path = Path(options['controller_source'])
    if hashlib.sha256(path.read_bytes()).hexdigest() != options['controller_sha256']:
        raise RuntimeError('controller source identity changed')
    spec = importlib.util.spec_from_file_location('dot17_controller_identity', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    config = json.loads(Path(options['controller_config']).read_text())
    with sqlite3.connect(Path(config['journal']).resolve().as_uri() + '?mode=ro', uri=True) as db:
        row = db.execute('SELECT registration,private_data FROM domains WHERE epoch=?',
                         (options['runtime_epoch'],)).fetchone()
        if row is None:
            raise RuntimeError('actual predecessor registration absent')
        registration, private = map(json.loads, row)
        excluded = db.execute('SELECT observation FROM excluded_domains WHERE registration=?',
                              (row[0],)).fetchone()
        domain_count = db.execute('SELECT COUNT(*) FROM domains').fetchone()[0]
    kind, machine, boot = module.native_identity()
    marker = options['marker']
    trace = Path(options['trace_file'])
    # A concurrent append may leave only the last line incomplete. Earlier
    # malformed records are failures, never silently discarded evidence.
    raw = trace.read_bytes()
    lines = raw.split(b'\n')[:-1]
    all_records = [json.loads(line) for line in lines if line]
    markers = options.get('effect_markers', [])
    effect_counts = {value: sum(record.get('event') == 'effect' and
                              record.get('marker') == value for record in all_records)
                     for value in markers}
    records = all_records
    records = [record for record in records if record.get('marker') == marker]
    started = [record for record in records if record.get('event') == 'descendant_started']
    if len(started) != 1:
        raise RuntimeError('exactly one real detached descendant required')
    child = started[0]
    beats = [record for record in records if record.get('event') == 'descendant_heartbeat']
    settings = config['deployments'][registration['deployment_id']]
    domain = registration['domain']
    def process(identity):
        pid = int(identity.split(':', 1)[0])
        try:
            if domain['kind'] == 'kubernetes':
                # The fixture writes its container PID. Resolve that namespace
                # PID only inside the retained Pod cgroup, then compare kernel
                # start time; a matching number elsewhere has no relevance.
                matches = []
                for entry in Path('/proc').iterdir():
                    if not entry.name.isdecimal():
                        continue
                    try:
                        status = dict(line.split(':', 1) for line in (entry / 'status').read_text().splitlines())
                        nested = status.get('NSpid', '').split()
                        if nested and int(nested[-1]) == pid and Path(module.Controller.process_cgroup(int(entry.name))).is_relative_to(Path(private['cgroup'])):
                            actual = module.root_identity(int(entry.name), entry.stat().st_uid)
                            matches.append(str(pid) + ':' + actual.split(':', 1)[1])
                    except (OSError, module.Refused):
                        continue
                if len(matches) != 1:
                    return {'identity': None, 'same_process': None}
                current = matches[0]
            else:
                current = module.root_identity(pid, settings.get('runtime_uid'))
            return {'identity': current, 'same_process': current == identity}
        except (OSError, subprocess.CalledProcessError, module.Refused):
            # This is not exclusion proof. A boot transition or retained cgroup
            # evidence is still mandatory in the coordinator.
            return {'identity': None, 'same_process': None}
    result = {'registration': registration, 'machine_id': machine, 'boot_id': boot,
              'platform': kind, 'controller_sha256': options['controller_sha256'],
              'descendant': child, 'descendant_process': process(child['process_identity']),
              'heartbeat_count': len(beats), 'last_heartbeat': beats[-1] if beats else None,
              'effect_count': sum(record.get('event') == 'effect' for record in records),
              'continuation_effect_counts': effect_counts,
              'trace_sha256': hashlib.sha256(raw).hexdigest(),
              'domain_count': domain_count,
              'exclusion': json.loads(excluded[0]) if excluded else None}
    if domain['kind'] == 'native_boot':
        result['root_process'] = process(domain['root_process_identity'])
    else:
        def group(value):
            path = Path(value['cgroup'])
            if not path.exists():
                return {'present': False}
            stat = path.stat()
            return {'present': True, 'device': stat.st_dev, 'inode': stat.st_ino,
                    'matches_retained': stat.st_dev == value['device'] and stat.st_ino == value['inode'],
                    'events': dict(line.split() for line in (path / 'cgroup.events').read_text().splitlines())}
        def cri(*args):
            return json.loads(module.command([config['crictl'], '--runtime-endpoint',
                                             config['cri_endpoint'], *args]))
        result['pod_cgroup'] = group(private)
        result['kubelet_cgroup'] = group(private['kubelet'])
        result['sandbox'] = cri('inspectp', domain['sandbox_id'])['status']
        result['containers'] = [cri('inspect', value)['status'] for value in domain['container_ids']]
        state = module.command([config['systemctl'], 'show',
            '--property=ActiveState,UnitFileState,MainPID,ControlGroup,KillMode', config['kubelet_unit']])
        result['kubelet'] = dict(line.split('=', 1) for line in state.splitlines())
    return result


if __name__ == '__main__':
    print(json.dumps(observe(json.loads(base64.urlsafe_b64decode(sys.argv[1])))))
