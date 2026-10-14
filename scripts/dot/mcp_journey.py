"""Real installed-host elicitation journeys. Run only through the phase gate.

mcp_transports has one stdio and one http entry. Each supplies name, transport,
server_name, exact elicit_tool/effect_tool, trace and disconnect_file; http also
supplies http_port. Host configuration points to the checked-in dot19 peer with
these exact trace/control paths. HTTP lifecycle belongs to PeerSet; the actual
host starts/stops stdio peers. Optional artifact/run_credentials override the
profile's run admission inputs without writing protected host state.
"""
import copy
from datetime import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
from urllib.parse import quote

from acceptance import ROOT, Blocked, digest, poll, require
from journeys import decision, finish, pending, start_run

PEER = ROOT / 'tests/fixtures/dot19_mcp_peer.py'
WIRE = ROOT / 'tests/fixtures/dot19_mcp_wire.json'


def rows(entry):
    if isinstance(entry['trace'], dict):
        from kubernetes import read_trace
        observed = read_trace(entry['trace'])
    else:
        path = Path(entry['trace'])
        if not path.exists():
            observed = []
        else:
            # Peer traces are one complete fsynced JSON line per observed boundary.
            text = path.read_text()
            observed = [json.loads(line) for line in text.splitlines(keepends=True) if line.endswith('\n')]
    baseline = entry.get('_trace_baseline', [])
    require(observed[:len(baseline)] == baseline, 'installed peer trace baseline was truncated or replaced')
    return observed


def current_rows(entry):
    return rows(entry)[len(entry.get('_trace_baseline', [])):]


def effects(entry):
    return [row for row in rows(entry) if row['event'] == 'effect']


def control(entry, present, suffix=''):
    require(suffix in ('', '.abandon-request'), 'unsupported peer control')
    original = entry['disconnect_file']
    reference = dict(original, path=original['path'] + suffix) if isinstance(original, dict) else original + suffix
    if isinstance(reference, dict):
        from kubernetes import write_control
        write_control(reference, present)
    elif present:
        Path(reference).write_text('disconnect observed pending request\n')
    else:
        Path(reference).unlink(missing_ok=True)


STDIO_BASELINE = """import hashlib,json,os,pathlib,stat,sys
trace,control=map(pathlib.Path,sys.argv[1:])
assert not control.exists() and not control.is_symlink()
source=pathlib.Path('/opt/dot17/dot19_mcp_peer.py')
wire=source.with_name('dot19_mcp_wire.json')
result={'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'wire_sha256':hashlib.sha256(wire.read_bytes()).hexdigest(),'rows':[],'processes':[],'file':None}
if trace.exists():
    assert not trace.is_symlink()
    with trace.open('rb') as stream:
        before=os.fstat(stream.fileno()); assert stat.S_ISREG(before.st_mode)
        raw=stream.read(16*1024*1024+1); assert len(raw)<=16*1024*1024
        after=os.fstat(stream.fileno()); assert (before.st_dev,before.st_ino)==(after.st_dev,after.st_ino)
    records=[json.loads(line) for line in raw.split(b'\\n')[:-1] if line]
    result['rows']=records
    result['file']={'device':before.st_dev,'inode':before.st_ino,'observed_bytes':len(raw),
                    'sha256':hashlib.sha256(raw).hexdigest()}
    for pid in sorted({row['pid'] for row in records if row.get('event')=='initialize'}):
        assert isinstance(pid,int) and pid>1
        proc=pathlib.Path('/proc')/str(pid)
        try:
            start=(proc/'stat').read_text().rsplit(')',1)[1].split()[19]
            command=(proc/'cmdline').read_bytes().decode().rstrip('\\0').split('\\0')
            if len(command)<2 or command[1]!=str(source): continue
            def option(name):
                assert command.count(name)==1
                return command[command.index(name)+1]
            if option('--transport')!='stdio' or option('--trace')!=str(trace) or option('--disconnect-file')!=str(control): continue
            exe=os.readlink(proc/'exe')
            assert 'python' in pathlib.Path(exe).name.lower()
            assert (proc/'stat').read_text().rsplit(')',1)[1].split()[19]==start
            result['processes'].append({'pid':pid,'start_ticks':start,'exe':exe,'command':command})
        except FileNotFoundError:
            continue
    # Existing records need a real initialized writer, not a copied old trace.
    assert not records or result['processes']
else:
    assert not trace.is_symlink()
    trace.parent.mkdir(parents=True,exist_ok=True)
control.parent.mkdir(parents=True,exist_ok=True)
print(json.dumps(result))
"""


def fresh_remote(entry, spec, evidence):
    from kubernetes import IDENTITY, exec_python, file_path, kube_json
    trace, disconnect = entry['trace'], entry['disconnect_file']
    require(trace.get('kind') == disconnect.get('kind') == 'kubernetes_file', 'unsupported MCP remote file reference')
    for field in ('context', 'namespace', 'pod', 'container'):
        require(trace[field] == disconnect[field], 'MCP trace and disconnect belong to different peers')
    paths = [file_path(trace), file_path(disconnect)]
    require(paths[0] != paths[1], 'MCP control path aliases trace')
    if entry['transport'] == 'http':
        # Runner-owned HTTP peers still require genuinely fresh paths.
        exec_python(trace, "import pathlib,sys; paths=[pathlib.Path(x) for x in sys.argv[1:]]; assert all(not p.exists() and not p.is_symlink() for p in paths); [p.parent.mkdir(parents=True,exist_ok=True) for p in paths]", paths + [paths[1] + '.abandon-request'])
        return
    hosts = [host for host in spec['hosts'].values() if host.get('lifecycle') == 'kubernetes'
             and all(host['kubernetes'][field] == trace[field] for field in ('context', 'namespace', 'pod', 'container'))]
    require(len(hosts) == 1, 'stdio baseline must belong to one declared installed runtime')
    host = hosts[0]
    target = host['kubernetes']
    def identity():
        pod = kube_json(target, 'pod', target['pod'])
        require(pod['metadata']['namespace'] == target['namespace'] and not pod['metadata'].get('deletionTimestamp'),
                'stdio baseline runtime Pod is deleting or mismatched')
        require(any(owner['kind'] == 'StatefulSet' and owner['name'] == target['statefulset']
                    and owner['uid'] == target['statefulset_uid'] for owner in pod['metadata']['ownerReferences']),
                'stdio baseline runtime belongs to another StatefulSet')
        status = next(row for row in pod['status']['containerStatuses'] if row['name'] == target['container'])
        require(status.get('ready') and 'running' in status['state'] and status['imageID'].split('@')[-1] == host['image_digest'],
                'stdio baseline runtime container/image is not the selected live artifact')
        return {'pod_uid': pod['metadata']['uid'], 'container_id': status['containerID'],
                'image_id': status['imageID'], 'restart_count': status['restartCount']}
    before = identity()
    runtime = json.loads(exec_python(target, IDENTITY, [target['runtime_config_path']]))
    require(runtime['exe'] == '/usr/local/bin/universal-agent-runtime'
            and runtime['config_sha256'] == digest(host['config']), 'stdio baseline runtime/config identity differs')
    baseline = json.loads(exec_python(trace, STDIO_BASELINE, paths))
    require(baseline['source_sha256'] == digest(PEER) and baseline['wire_sha256'] == digest(WIRE),
            'actual runtime-mounted stdio peer differs from inventoried source')
    require(identity() == before, 'runtime changed during stdio baseline observation')
    entry['_trace_baseline'] = baseline['rows']
    evidence.observe('mcp-' + entry['name'] + '-installed-stdio-baseline',
        {'container': before, 'runtime': runtime, 'trace': trace, 'baseline': baseline,
         'claim_scope': 'retained pre-run observations; excluded from new scenario acceptance'})


class PeerSet:
    def __init__(self, spec, evidence):
        self.entries = spec.get('mcp_transports', [])
        require({entry['transport'] for entry in self.entries} == {'stdio', 'http'},
                'MCP journeys require both stdio and Streamable HTTP')
        require(len({entry['name'] for entry in self.entries}) == len(self.entries),
                'duplicate MCP fixture name')
        self.evidence = evidence
        self.processes = {}
        self.logs = {}
        self.boots = {}
        try:
            for entry in self.entries:
                require('_trace_baseline' not in entry, 'trace baseline is observation-owned, never profile input')
                if isinstance(entry['trace'], dict):
                    require(isinstance(entry['disconnect_file'], dict), 'mixed local and remote MCP files')
                    fresh_remote(entry, spec, evidence)
                    if entry['transport'] == 'http':
                        for field in ('context', 'namespace', 'pod', 'container'):
                            require(entry['peer'][field] == entry['trace'][field], 'HTTP MCP process does not own declared trace')
                        self.start(entry)
                    continue
                for field in ['trace', 'disconnect_file']:
                    require(Path(entry[field]).is_absolute(), 'MCP fixture paths must be absolute')
                    if Path(entry[field]).exists():
                        raise Blocked('MCP fixture path already exists: ' + entry[field])
                    Path(entry[field]).parent.mkdir(parents=True, exist_ok=True)
                require(not Path(entry['disconnect_file'] + '.abandon-request').exists(), 'old request-abandon control is present')
                if entry['transport'] == 'http':
                    self.start(entry)
        except Exception:
            self.close()
            raise

    def start(self, entry):
        name = entry['name']
        self.boots[name] = self.boots.get(name, 0) + 1
        if isinstance(entry['trace'], dict):
            from kubernetes import exec_python, file_path, remote_process
            port = dict(entry['trace'], path=entry['trace']['path'] + '.port-' + uuid.uuid4().hex)
            port_path = file_path(port)
            process = remote_process(entry['peer'], PEER,
                ['--transport', 'http', '--host', '0.0.0.0', '--trace', file_path(entry['trace']),
                 '--disconnect-file', file_path(entry['disconnect_file']), '--port', str(entry['http_port']),
                 '--port-file', port_path], 'mcp-' + name + '-' + str(self.boots[name]))
            self.processes[name] = process
            ready = poll(lambda: exec_python(entry['peer'],
                "import pathlib,sys; p=pathlib.Path(sys.argv[1]); print(p.read_text() if p.exists() else '')",
                [port_path]).decode().strip(), bool, 'remote MCP HTTP peer not ready')
            require(int(ready) == entry['http_port'], 'remote MCP endpoint differs from declared Service target')
            self.evidence.observe('mcp-' + name + '-remote-start-' + str(self.boots[name]),
                {'peer': entry['peer'], 'process': process.identity, 'trace': entry['trace'],
                 'transport': 'streamable_http', 'wire_port': int(ready)})
            return
        port = self.evidence.directory / f'mcp-{name}-port-{self.boots[name]}'
        log = open(self.evidence.directory / f'mcp-{name}-{self.boots[name]}.log', 'xb')
        env = {'PATH': os.environ.get('PATH', ''), 'PYTHONUNBUFFERED': '1'}
        process = subprocess.Popen([sys.executable, str(PEER), '--transport', 'http',
            '--trace', entry['trace'], '--disconnect-file', entry['disconnect_file'],
            '--port', str(entry['http_port']), '--port-file', str(port)],
            cwd=self.evidence.directory, env=env, stdin=subprocess.DEVNULL,
            stdout=log, stderr=subprocess.STDOUT)
        self.processes[name], self.logs[name] = process, log
        poll(lambda: port.read_text() if port.exists() else None, bool, 'MCP HTTP peer not ready')
        require(int(port.read_text()) == entry['http_port'], 'fixture HTTP endpoint differs from declaration')

    def stop(self, entry):
        process = self.processes.pop(entry['name'], None)
        if process is not None:
            if isinstance(entry['trace'], dict):
                process.close()
                return
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=30)
            self.logs.pop(entry['name']).close()

    def restart(self, entry):
        self.stop(entry)
        control(entry, False)
        self.start(entry)

    def restart_host(self, host):
        retained = {entry['name']: rows(entry) for entry in self.entries}
        host.restart()
        for entry in self.entries:
            previous = retained[entry['name']]
            require(rows(entry)[:len(previous)] == previous, 'host restart lost or replaced independent peer trace')

    def close(self):
        for entry in reversed(self.entries):
            if entry['transport'] == 'http':
                self.stop(entry)


def prepare_peers(spec, evidence):
    return PeerSet(spec, evidence)


def run_spec(spec, entry):
    selected = copy.deepcopy(spec)
    for field in ['artifact', 'run_credentials']:
        if field in entry:
            selected[field] = entry[field]
    return selected


def begin(host, spec, entry, mode, sequence=False, grant_seconds=None, grant_observation=None):
    steps = [{'tool': entry['elicit_tool'], 'arguments': {'mode': mode}}]
    if sequence:
        steps.append({'tool': entry['effect_tool'], 'arguments': {}})
    selected = run_spec(spec, entry)
    prompt = json.dumps({'dot_action': 'tool_sequence', 'steps': steps})
    if grant_seconds is None:
        run = start_run(host, selected, prompt)
    else:
        require(entry['transport'] == 'http' and isinstance(entry.get('credential_grant'), dict),
                'credential lifetime case requires a registered HTTP destination and real admitted grant template')
        grant = copy.deepcopy(entry['credential_grant'])
        require(grant.get('credential_revision') and grant.get('scopes'), 'credential grant template lacks revision/scopes')
        grant['expires_at_unix'] = int(time.time()) + grant_seconds
        grant['credential_revision'] += '-' + uuid.uuid4().hex
        if grant_observation is not None:
            grant_observation['expires_at_unix'] = grant['expires_at_unix']
        run = host.api('POST', '/api/uar/runs', {'artifact': selected['artifact'], 'input': prompt,
            'run_credentials': selected['run_credentials'],
            'mcp_servers': [{'name': entry['server_name'], 'grant': grant}]})['run_id']
    initial = decision(host, run, True)
    path = '/api/uar/runs/' + run + '/mcp-interactions'
    views = poll(lambda: host.api('GET', path)['interactions'],
                 lambda values: any(view['resolvable'] for view in values),
                 'MCP request never reached durable host interaction')
    view = next(view for view in views if view['resolvable'])
    require(view['record']['server_name'] == entry['server_name'], 'interaction bound to wrong server')
    require(view['record']['run_id'] == run, 'interaction bound to another run')
    return run, path, view['record'], initial


def persisted(host, path, id):
    return next(row for row in host.api('GET', path)['interactions'] if row['record']['id'] == id)


def peer_answer(entry, record):
    return [row for row in current_rows(entry) if row['event'] == 'elicitation_response'
            and row['request_id'] == record['request_id']]


def timestamp(value):
    return datetime.fromisoformat(value.replace('Z', '+00:00')).timestamp()


def wait_deadline(deadline):
    # Real elapsed time only. Never alter a host clock or persisted expiry.
    while time.time() <= deadline:
        time.sleep(min(5, max(0.05, deadline - time.time() + 0.1)))


class GrantHostView:
    """Use the profile's real trusted-host credential for its entire run scope."""
    def __init__(self, host):
        self.host = host

    def api(self, method, path, body=None, expect=200, principal='host', headers=None):
        return self.host.api(method, path, body, expect=expect, principal=principal, headers=headers)


def pending_lifetimes(host, spec, entry, evidence, peers, wire):
    prefix = 'mcp-' + entry['name']
    before = len(effects(entry))
    run, path, record, _ = begin(host, spec, entry, 'form')
    require(timestamp(record['expires_at']) - timestamp(record['created_at']) == 600,
            'interaction does not have its explicit ten-minute lifetime')
    expired = poll(lambda: persisted(host, path, record['id']),
        lambda value: not value['resolvable'], 'absolute interaction lifetime did not end', seconds=660)
    require(expired['record']['state'] == 'expired' and expired['record']['response'] is None,
            'input ended for another cause instead of its absolute lifetime')
    require(time.time() >= timestamp(record['expires_at']), 'interaction expired before its persisted deadline')
    host.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
    answers = poll(lambda: peer_answer(entry, record), bool, 'peer did not receive lifetime cancellation')
    require(all(answer['message'].get('result', {}).get('action') == 'cancel' for answer in answers),
            'expired input delivered an acceptance')
    finish(host, run)
    require(len(effects(entry)) == before, 'expired input executed an effect')
    peers.restart_host(host)
    require(persisted(host, path, record['id']) == expired, 'expiry was not durable across joined restart')
    evidence.observe(prefix + '-absolute-lifetime', {'run': run, 'interaction': expired,
        'peer_response': answers, 'duration_seconds': 600, 'effect_delta': 0, 'joined_restart': True})

    cases = ['root-cancel'] + (['grant-revoke', 'grant-expiry'] if entry['transport'] == 'http' else [])
    for case in cases:
        before = len(effects(entry))
        admitted_grant = {}
        node = GrantHostView(host) if case.startswith('grant-') else host
        run, path, record, _ = begin(node, spec, entry, 'form', sequence=True,
            grant_seconds=90 if case == 'grant-expiry' else 900 if case == 'grant-revoke' else None,
            grant_observation=admitted_grant)
        observation = None
        if case == 'root-cancel':
            observation = node.api('POST', '/api/uar/runs/' + run + '/cancel')
            require(observation['cancelled'], 'pending root was not cancelled')
        elif case == 'grant-revoke':
            observation = node.api('POST', '/api/uar/runs/' + run + '/mcp-grants/' + quote(entry['server_name'], safe='') + '/revoke')
            require(observation['revoked'] and observation['renewal'] == 'resume_required', 'real admitted grant was not revoked')
        closed = poll(lambda: persisted(node, path, record['id']),
            lambda value: not value['resolvable'], 'authority loss did not close pending input', seconds=150)
        require(closed['record']['response'] is None and closed['record']['state'] in ('cancelled', 'disconnected'),
                'authority loss manufactured a response or extended input lifetime')
        require(time.time() < timestamp(record['expires_at']), 'grant/cancellation case merely waited for interaction expiry')
        if case == 'grant-expiry':
            require(time.time() >= admitted_grant['expires_at_unix'] and closed['record']['state'] == 'cancelled',
                    'grant expiry case ended early or for an unrelated disconnect')
        node.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
        terminal = poll(lambda: node.api('GET', '/api/uar/runs/' + run),
            lambda value: value['status'] in ('done', 'error', 'cancelled'), 'authority loss did not terminate run')
        require(terminal['status'] in ('error', 'cancelled'), 'authority loss reported successful sequence completion')
        require(not pending(node, run) and len(effects(entry)) == before, 'authority loss left approval or executed effect')
        answers = peer_answer(entry, record)
        require(all(answer['message'].get('result', {}).get('action') != 'accept' for answer in answers),
                'authority loss delivered accepted input')
        evidence.observe(prefix + '-' + case, {'run': run, 'interaction': closed, 'authority_observation': observation,
            'admitted_grant': admitted_grant, 'terminal': terminal, 'peer_responses': answers, 'effect_delta': 0})
    if entry['transport'] == 'http':
        before = len(effects(entry))
        run, path, record, _ = begin(host, spec, entry, 'form', sequence=True)
        control(entry, True, '.abandon-request')
        try:
            abandoned = poll(lambda: [row for row in current_rows(entry)
                if row['event'] == 'request_abandoned' and row['request_id'] == record['request_id']],
                bool, 'peer did not close its actual request stream')
            closed = poll(lambda: persisted(host, path, record['id']),
                lambda value: not value['resolvable'], 'abandoned request retained a resolvable callback')
            require(closed['record']['response'] is None and closed['record']['state'] in ('cancelled', 'disconnected'),
                    'request abandonment produced a response or unrelated expiry')
            host.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
            finish(host, run, 'error')
        finally:
            control(entry, False, '.abandon-request')
        require(not peer_answer(entry, record) and len(effects(entry)) == before,
                'abandoned request delivered an answer or authorized an effect')
        # A new real call proves the same HTTP peer process remained usable.
        next_run, next_path, next_record, _ = begin(host, spec, entry, 'form')
        host.api('POST', next_path + '/' + next_record['id'], wire['decline'])
        answer = poll(lambda: peer_answer(entry, next_record), bool, 'retained HTTP peer is unavailable after request abandonment')
        require(answer[0]['pid'] == abandoned[0]['pid'], 'request-abandon case replaced the entire peer process')
        finish(host, next_run)
        evidence.observe(prefix + '-request-stream-abandon', {'run': run, 'interaction': closed,
            'peer_observation': abandoned, 'same_peer_followup_response': answer, 'effect_delta': 0})


def run_mcp(host, spec, evidence, peers=None):
    if peers is None:
        raise Blocked('prepare_peers must run before the installed host boots')
    wire = json.loads(WIRE.read_text())
    for entry in peers.entries:
        if isinstance(entry['trace'], dict) and entry['transport'] == 'stdio':
            require(host.spec['lifecycle'] == 'kubernetes', 'remote stdio trace belongs to a non-Kubernetes runtime')
            require(all(entry['trace'][field] == host.spec['kubernetes'][field]
                        for field in ('context', 'namespace', 'pod', 'container')),
                    'stdio observations do not belong to the actual selected runtime container')
        prefix = 'mcp-' + entry['name']
        for mode in ['form', 'url']:
            for action in ['accept', 'decline', 'cancel']:
                before = len(effects(entry))
                run, path, record, initial = begin(host, spec, entry, mode, sequence=action == 'accept')
                require(len(effects(entry)) == before, 'effect occurred while elicitation was pending')
                response = dict(wire['formAccept' if mode == 'form' else 'urlAccept']) if action == 'accept' else {'action': action}
                if mode == 'form' and action == 'accept':
                    host.api('POST', path + '/' + record['id'], wire['invalidFormAccept'], expect=400)
                    # The ordinary 30-second tool timer must pause for actual input.
                    time.sleep(31)
                    require(persisted(host, path, record['id'])['resolvable'], 'ordinary tool timer consumed human wait')
                for principal in ['other_tenant']:
                    foreign = host.api('GET', path, principal=principal)['interactions']
                    require(not foreign, 'foreign principal saw elicitation content')
                    host.api('POST', path + '/' + record['id'], response, principal=principal, expect=(403, 409))
                # This host trusts one configured issuer. A different issuer
                # is an authentication rejection, not an accepted owner scope.
                host.api('GET', path, principal='other_issuer', expect=401)
                host.api('POST', path + '/' + record['id'], response, principal='other_issuer', expect=401)
                commit = host.api('POST', path + '/' + record['id'], response)
                require(commit['response_committed'] and commit['transport_delivery'] == 'unconfirmed',
                        'API fabricated a transport receipt')
                host.api('POST', path + '/' + record['id'], response, expect=409)
                answer = poll(lambda: peer_answer(entry, record), bool, 'real peer did not observe elicitation response')
                require(len(answer) == 1 and answer[0]['message']['result'] == response, 'peer decision differs from durable response')
                next_approval = None
                if action == 'accept':
                    second = poll(lambda: pending(host, run), bool, 'accepted input bypassed next Required approval')
                    next_approval = second.get('approval_id') or second['approvalId']
                    require(next_approval != initial and len(effects(entry)) == before, 'input acceptance conferred tool authority')
                    decision(host, run, True)
                finish(host, run)
                require(len(effects(entry)) == before + (1 if action == 'accept' else 0), 'independent effect count differs')
                view = persisted(host, path, record['id'])
                require(view['record']['state'] == 'answered' and view['record']['response'] == response and not view['resolvable'],
                        'terminal interaction did not preserve exact decision')
                peers.restart_host(host)
                require(persisted(host, path, record['id']) == view, 'restart changed durable interaction')
                evidence.observe(prefix + '-' + mode + '-' + action, {'run_id': run,
                    'initial_approval': initial, 'next_approval': next_approval,
                    'authenticated_foreign_principals': ['other_tenant'],
                    'untrusted_issuer_rejection_status': 401,
                    'interaction': view, 'peer_response': answer, 'effect_delta': len(effects(entry)) - before,
                    'joined_restart': True})

        # The pinned SDK's optional automatic-round helper must not resend calls.
        previous_calls = len([row for row in rows(entry) if row['event'] == 'tool_call'])
        retry_run = start_run(host, run_spec(spec, entry), json.dumps({'dot_action': 'tool_sequence',
            'steps': [{'tool': entry['elicit_tool'], 'arguments': {'mode': 'mrtr'}}]}))
        decision(host, retry_run, True)
        finish(host, retry_run, 'error')
        retry_calls = [row for row in rows(entry) if row['event'] == 'tool_call'][previous_calls:]
        require(len(retry_calls) == 1, 'SDK automatically retried an unadmitted tool round')
        require(not host.api('GET', '/api/uar/runs/' + retry_run + '/mcp-interactions')['interactions'],
                'unsupported protocol round invented an interaction')
        receipts = host.api('GET', '/api/uar/runs/' + retry_run + '/tool-outcomes')['receipts']
        require(receipts and all(row['outcome'] in ('application_failed', 'transport_failed', 'outcome_unknown')
                                 and not row['legacy_uncertain'] for row in receipts),
                'unsupported multi-round invocation did not retain a current non-success outcome')
        evidence.observe(prefix + '-no-sdk-replay', {'run': retry_run, 'calls': retry_calls, 'canonical_receipts': receipts})

        # Acceptance is not enough: a separately denied Required effect remains absent.
        before = len(effects(entry))
        run, path, record, initial = begin(host, spec, entry, 'form', sequence=True)
        host.api('POST', path + '/' + record['id'], wire['formAccept'])
        denied = decision(host, run, False)
        require(denied != initial, 'second operation reused first approval')
        finish(host, run, 'error')
        require(len(effects(entry)) == before, 'denied effect followed accepted input')
        evidence.observe(prefix + '-accepted-input-denied-effect', {'run': run, 'interaction': record['id'],
            'separate_denial': denied, 'effect_delta': 0})

        # Kill only our controlled peer after a durable request is observed.
        run, path, record, _ = begin(host, spec, entry, 'form')
        control(entry, True)
        disconnected = poll(lambda: persisted(host, path, record['id']),
            lambda view: not view['resolvable'], 'disconnected peer still accepted decisions')
        host.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
        finish(host, run, 'error')
        require(not peer_answer(entry, record), 'disconnected peer received fabricated answer')
        if entry['transport'] == 'http':
            peers.restart(entry)
        else:
            control(entry, False)
        evidence.observe(prefix + '-disconnect', {'run': run, 'interaction': disconnected,
            'effect_delta': len(effects(entry)) - before})
        evidence.observe(prefix + '-request-abandon', {'run': run, 'request_id': record['request_id'],
            'interaction': disconnected, 'mechanism': 'controlled peer process exits with its request pending',
            'peer_responses': peer_answer(entry, record), 'effect_delta': len(effects(entry)) - before})

        initialized = [row for row in current_rows(entry) if row['event'] == 'initialize']
        require(any(row['params'].get('capabilities', {}).get('elicitation') == wire['clientCapabilities']['elicitation']
                    for row in initialized), 'real client omitted selected elicitation capability')
        evidence.observe(prefix + '-capabilities', {'initialize_messages': initialized,
            'transport': entry['transport'], 'server': entry['server_name']})
        pending_lifetimes(host, spec, entry, evidence, peers, wire)

    # Finish all live journeys before one abrupt host death. The native
    # supervisor intentionally does not grant same-boot takeover without
    # descendant exclusion, so this stage performs only history reads afterward.
    cold_runs = []
    for entry in peers.entries:
        before = len(effects(entry))
        run, path, record, _ = begin(host, spec, entry, 'form')
        new_run, new_path, new_record, _ = begin(host, spec, entry, 'form')
        require(new_record['channel_id'] != record['channel_id'] and new_record['connection_id'] != record['connection_id'],
                'another root inherited old elicitation connection authority')
        host.api('POST', new_path + '/' + new_record['id'], wire['decline'])
        finish(host, new_run)
        require(persisted(host, path, record['id'])['resolvable'], 'closing sibling root closed original channel')
        cold_runs.append((entry, run, path, record, before, new_run, new_record))
    retained = {entry['name']: rows(entry) for entry in peers.entries}
    host.stop(crash=True)
    for entry in peers.entries:
        if entry['transport'] == 'http':
            peers.restart(entry)
    host.start()
    for entry in peers.entries:
        previous = retained[entry['name']]
        require(rows(entry)[:len(previous)] == previous, 'abrupt host replacement lost independent peer evidence')
    pending_cold = []
    for entry, run, path, record, before, new_run, new_record in cold_runs:
        cold = persisted(host, path, record['id'])
        require(not cold['resolvable'] and cold['record']['response'] is None, 'cold callback became answer authority')
        require(cold['record']['state'] in ('pending', 'cancelled', 'disconnected', 'expired'), 'cold state was fabricated')
        host.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
        require(len(effects(entry)) == before and not peer_answer(entry, record), 'restart replayed pending operation')
        evidence.observe('mcp-' + entry['name'] + '-cold-history', {'run': run, 'cold': cold,
            'independent_run_before_crash': new_run, 'independent_interaction': new_record['id'],
            'effect_delta': 0, 'new_logical_run_after_crash': False})
        if cold['record']['state'] == 'pending':
            outcomes = host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes')
            approvals = host.api('GET', '/api/uar/runs/' + run + '/tool-approval')
            pending_cold.append((entry, run, path, record, before, outcomes, approvals))
    require(len(pending_cold) == len(peers.entries), 'cold expiry requires actual pending records surviving abrupt death')
    wait_deadline(max(timestamp(record['expires_at']) for _, _, _, record, _, _, _ in pending_cold))
    expired_cold = []
    for entry, run, path, record, before, outcomes, approvals in pending_cold:
        cold = persisted(host, path, record['id'])
        require(cold['record']['state'] == 'expired' and cold['record']['response'] is None and not cold['resolvable'],
                'cold history read did not persist expiry without a waiter')
        require(not pending(host, run) and len(effects(entry)) == before and not peer_answer(entry, record),
                'cold expiry created approval, delivered response or executed an effect')
        require(host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes') == outcomes,
                'cold history expiry created or changed a canonical execution outcome')
        require(host.api('GET', '/api/uar/runs/' + run + '/tool-approval') == approvals,
                'cold history expiry created or changed approval history')
        host.api('POST', path + '/' + record['id'], wire['formAccept'], expect=409)
        expired_cold.append((entry, run, path, record, cold, outcomes, approvals))
    peers.restart_host(host)
    for entry, run, path, record, cold, outcomes, approvals in expired_cold:
        require(persisted(host, path, record['id']) == cold, 'cold history expiry was not persisted across restart')
        require(host.api('GET', '/api/uar/runs/' + run + '/tool-outcomes') == outcomes,
                'cold expiry restart changed canonical execution outcomes')
        require(host.api('GET', '/api/uar/runs/' + run + '/tool-approval') == approvals,
                'cold expiry restart changed approval history')
        evidence.observe('mcp-' + entry['name'] + '-cold-expiry', {'run': run, 'interaction': cold,
            'canonical_outcomes_unchanged': outcomes, 'approval_history_unchanged': approvals,
            'joined_restart': True, 'new_logical_run_after_crash': False})
