"""Fixed know-me qualification transport and container lifecycle, never fencing.

No apply, delete, rollout, arbitrary shell command, or external callback is
accepted. A kubelet container termination is not supervisor exclusion authority.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import urllib.parse
import uuid

from acceptance import ROOT, Blocked, digest, poll, require

DNS = re.compile(r'[a-z0-9]([-a-z0-9.]*[a-z0-9])?')
BASE = '/var/lib/uar/dot17/'


def prefix(target):
    require(target['context'] == 'know-me', 'qualification context must be explicitly know-me')
    require(DNS.fullmatch(target['namespace']), 'invalid qualification namespace')
    return ['kubectl', '--context', 'know-me', '--namespace', target['namespace']]


def kube_json(target, resource, name=None, selector=None):
    require(resource in ('pod', 'statefulset', 'service', 'httproute', 'endpointslice'), 'unsupported read resource')
    command = prefix(target) + ['get', resource]
    if name is not None:
        require(DNS.fullmatch(name), 'invalid Kubernetes resource name')
        command.append(name)
    if selector is not None:
        require(resource == 'endpointslice' and selector.startswith('kubernetes.io/service-name='), 'unsupported selector')
        command += ['--selector', selector]
    command += ['--output=json']
    try:
        result = subprocess.run(command, capture_output=True, timeout=30)
    except FileNotFoundError as error:
        raise Blocked('required kubectl executable is not installed') from error
    if result.returncode:
        raise Blocked('required qualification Kubernetes resource unavailable: ' + resource)
    require(len(result.stdout) <= 16 * 1024 * 1024, 'Kubernetes response limit exceeded')
    return json.loads(result.stdout)


def exec_python(target, fixed_source, args=()):
    """Only module-owned literal programs call this; configuration has no code field."""
    require(DNS.fullmatch(target['pod']) and DNS.fullmatch(target['container']), 'invalid explicit Pod/container')
    result = subprocess.run(prefix(target) + ['exec', target['pod'], '--container', target['container'],
        '--', 'python3', '-c', fixed_source, *[str(value) for value in args]], capture_output=True, timeout=45)
    if result.returncode:
        raise Blocked('fixed qualification Pod operation unavailable; no alternate executor used')
    require(len(result.stdout) <= 16 * 1024 * 1024, 'Pod observer response limit exceeded')
    return result.stdout


def file_path(reference):
    value = reference['path']
    require(value.startswith(BASE) and '..' not in Path(value).parts, 'fixture path outside dedicated data directory')
    return value


def read_trace(reference):
    raw = exec_python(reference, "import pathlib,sys; p=pathlib.Path(sys.argv[1]); sys.stdout.buffer.write(p.read_bytes() if p.exists() else b'')", [file_path(reference)])
    # Only completed JSONL records count; concurrent writer may be mid-record.
    return [json.loads(line) for line in raw.split(b'\n')[:-1] if line]


def write_control(reference, present):
    exec_python(reference, "import pathlib,sys; p=pathlib.Path(sys.argv[1]); p.parent.mkdir(parents=True,exist_ok=True); p.touch(exist_ok=True) if sys.argv[2]=='1' else p.unlink(missing_ok=True)",
                [file_path(reference), '1' if present else '0'])


IDENTITY = """import hashlib,json,os,pathlib,sys
p=pathlib.Path('/proc/1')
stat=(p/'stat').read_text().rsplit(')',1)[1].split()
exe=os.readlink(p/'exe')
config=pathlib.Path(sys.argv[1])
print(json.dumps({'pid':1,'start_ticks':stat[19],'exe':exe,'binary_sha256':hashlib.sha256((p/'exe').read_bytes()).hexdigest(),'config_sha256':hashlib.sha256(config.read_bytes()).hexdigest()}))
"""
SIGNAL = """import json,os,pathlib,signal,sys
expected=json.loads(sys.argv[1]);p=pathlib.Path('/proc/1')
stat=(p/'stat').read_text().rsplit(')',1)[1].split()
assert os.readlink(p/'exe')==expected['exe'] and stat[19]==expected['start_ticks']
assert pathlib.Path(expected['exe']).name=='universal-agent-runtime'
os.kill(1,signal.SIGKILL if sys.argv[2]=='crash' else signal.SIGTERM)
print(json.dumps({'signal_sent':True,'pid':1}))
"""


class KubernetesRuntime:
    def __init__(self, host):
        self.host = host
        self.target = host.spec['kubernetes']
        self.current = None
        self.stopped = False
        self.exits = 0

    def snapshot(self):
        target = self.target
        pod = kube_json(target, 'pod', target['pod'])
        require(pod['metadata']['namespace'] == target['namespace'] and not pod['metadata'].get('deletionTimestamp'),
                'qualification Pod is deleting or belongs to another namespace')
        require(any(row['kind'] == 'StatefulSet' and row['name'] == target['statefulset']
                    and row['uid'] == target['statefulset_uid'] for row in pod['metadata']['ownerReferences']),
                'Pod owner differs from selected qualification StatefulSet')
        status = next(row for row in pod['status']['containerStatuses'] if row['name'] == target['container'])
        require(status.get('ready') and 'running' in status['state'], 'runtime container is not ready')
        wanted = self.host.spec['image_digest']
        require(re.fullmatch(r'sha256:[0-9a-f]{64}', wanted), 'runtime architecture digest is not immutable')
        require(status['imageID'].split('@')[-1] == wanted, 'actual container image digest differs from source-bound artifact')
        identity = json.loads(exec_python(target, IDENTITY, [target['runtime_config_path']]))
        require(identity['exe'] == '/usr/local/bin/universal-agent-runtime', 'runtime container PID1 is not the packaged executable')
        require(identity['config_sha256'] == digest(self.host.spec['config']), 'actual mounted runtime config differs from private profile')
        if self.host.spec.get('binary_sha256'):
            require(identity['binary_sha256'] == self.host.spec['binary_sha256'], 'actual Pod executable checksum differs')
        return {'pod_uid': pod['metadata']['uid'], 'node': pod['spec']['nodeName'],
                'container_id': status['containerID'], 'image_id': status['imageID'],
                'restart_count': status['restartCount'], 'last_state': status.get('lastState', {}), 'identity': identity}

    def route(self, observed):
        target = self.target
        service = kube_json(target, 'service', target['service'])
        require(service['spec']['selector'].get('statefulset.kubernetes.io/pod-name') == target['pod'],
                'per-runtime service does not select the exact Pod')
        route = kube_json(target, 'httproute', target['http_route'])
        require(any(parent.get('namespace', target['namespace']) == 'argocd' and parent['name'] == 'argocd-gateway'
                    and parent.get('sectionName') == 'knowme-runtime-https' for parent in route['spec']['parentRefs']),
                'qualification route is not bound to the existing HTTPS listener')
        require('runtime.know-me.tools' in route['spec']['hostnames'], 'qualification route hostname differs')
        parents = [row for row in route.get('status', {}).get('parents', [])
                   if row.get('parentRef', {}).get('name') == 'argocd-gateway' and
                   row.get('parentRef', {}).get('sectionName') == 'knowme-runtime-https']
        require(any(all(any(condition['type'] == kind and condition['status'] == 'True'
                    and condition.get('observedGeneration', 0) >= route['metadata']['generation']
                    for condition in row.get('conditions', [])) for kind in ('Accepted', 'ResolvedRefs'))
                    for row in parents), 'Envoy has not accepted the current exact route and backend references')
        requested = urllib.parse.urlsplit(self.host.base)
        require(requested.scheme == 'https' and requested.hostname == 'runtime.know-me.tools'
                and requested.path == target['route_prefix'], 'runtime API bypasses selected Envoy route')
        rules = [rule for rule in route['spec']['rules'] if any(match.get('path', {}).get('value') == target['route_prefix']
                 for match in rule.get('matches', []))]
        require(len(rules) == 1 and len(rules[0]['backendRefs']) == 1 and rules[0]['backendRefs'][0]['name'] == target['service'],
                'Envoy prefix does not have one exact runtime backend')
        require(any(row.get('type') == 'URLRewrite' and row['urlRewrite']['path'].get('replacePrefixMatch') == '/'
                    for row in rules[0].get('filters', [])), 'qualification prefix is not rewritten to native API root')
        slices = kube_json(target, 'endpointslice', selector='kubernetes.io/service-name=' + target['service'])
        ready = [endpoint for item in slices['items'] for endpoint in item['endpoints'] if endpoint.get('conditions', {}).get('ready')]
        require(ready and all(endpoint.get('targetRef', {}).get('uid') == observed['pod_uid'] for endpoint in ready),
                'runtime service routes to another Pod')

    def start(self):
        self.host.boots += 1
        def ready():
            try:
                return self.snapshot()
            except (Blocked, AssertionError, KeyError, StopIteration):
                return None
        current = poll(ready, bool, 'actual installed runtime Pod not ready')
        first_attach = self.current is None
        if self.current:
            require(current['pod_uid'] == self.current['pod_uid'], 'Pod replacement requires independent supervisor qualification')
        self.route(current)
        self.current = current
        self.stopped = False
        if first_attach:
            # Peers are started before Host.start. A verified container restart
            # makes SDK discovery use those actual live peers instead of an
            # installation-time failed connection. No Pod is deleted.
            self.stop()
            current = self.current
            self.route(current)
            self.stopped = False
        self.host.request('GET', '/health', principal=None)
        self.host.evidence.observe(self.host.label + '-pod-start-' + str(self.host.boots), current)

    def stop(self, crash=False):
        if self.current is None or self.stopped:
            return
        before = self.snapshot()
        require(before['pod_uid'] == self.current['pod_uid'] and before['container_id'] == self.current['container_id'],
                'runtime changed outside owned lifecycle')
        # Killing PID1 can also close kubectl exec before its acknowledgement.
        # The independent kubelet record below is the required exit observation.
        sent = subprocess.run(prefix(self.target) + ['exec', self.target['pod'], '--container', self.target['container'],
            '--', 'python3', '-c', SIGNAL, json.dumps(before['identity']), 'crash' if crash else 'orderly'],
            capture_output=True, timeout=45)
        def replaced():
            try:
                after = self.snapshot()
            except (Blocked, AssertionError, StopIteration, KeyError):
                return None
            return after if after['container_id'] != before['container_id'] else None
        after = poll(replaced, bool, 'kubelet did not report a joined container exit and replacement', 240)
        require(after['pod_uid'] == before['pod_uid'] and after['restart_count'] > before['restart_count'],
                'a replaced Pod is not a joined container restart')
        terminated = after['last_state'].get('terminated', {})
        require(terminated.get('containerID') == before['container_id'] and terminated.get('finishedAt'),
                'kubelet did not retain the exact predecessor container termination')
        require(terminated.get('exitCode') == (137 if crash else 0), 'runtime predecessor did not exit with the required disposition')
        self.exits += 1
        self.host.evidence.observe(self.host.label + '-pod-exit-' + str(self.exits),
            {'before': before, 'after': after, 'crash': crash, 'signal_transport_returncode': sent.returncode,
             'supervisor_exclusion': 'not_claimed',
             'unknown_effects': 'remain held; this observation grants no recovery authority'})
        self.current = after
        self.stopped = True

FIXTURES = {'dot19_mcp_peer.py': ROOT / 'tests/fixtures/dot19_mcp_peer.py',
            'provider_peer.py': ROOT / 'scripts/dot/provider_peer.py'}
LAUNCH = """import hashlib,json,os,pathlib,subprocess,sys
source,expected,arguments,log_path=sys.argv[1:]
assert hashlib.sha256(pathlib.Path(source).read_bytes()).hexdigest()==expected
log=pathlib.Path(log_path);log.parent.mkdir(parents=True,exist_ok=True)
with log.open('xb') as output:
 p=subprocess.Popen(['python3',source,*json.loads(arguments)],stdin=subprocess.DEVNULL,stdout=output,stderr=subprocess.STDOUT,start_new_session=True)
stat=pathlib.Path('/proc',str(p.pid),'stat').read_text().rsplit(')',1)[1].split()
print(json.dumps({'pid':p.pid,'start_ticks':stat[19],'source_sha256':expected,'log':str(log)}))
"""
CLOSE = """import json,os,pathlib,signal,sys,time
expected=json.loads(sys.argv[1]);p=pathlib.Path('/proc',str(expected['pid']))
def alive():
 try:
  stat=(p/'stat').read_text().rsplit(')',1)[1].split()
  assert stat[19]==expected['start_ticks'],'PID reused; refusing signal'
  return stat[0]!='Z'
 except FileNotFoundError:return False
if alive():os.kill(expected['pid'],signal.SIGTERM)
end=time.monotonic()+15
while alive() and time.monotonic()<end:time.sleep(.1)
if alive():os.kill(expected['pid'],signal.SIGKILL)
end=time.monotonic()+10
while alive() and time.monotonic()<end:time.sleep(.1)
assert not alive(),'fixture exit not observed'
print(json.dumps({'fixture_exit_observed':True,'identity':expected}))
"""


class RemoteProcess:
    def __init__(self, target, identity):
        self.target, self.identity = target, identity
        self.closed = False

    def close(self):
        if not self.closed:
            exec_python(self.target, CLOSE, [json.dumps(self.identity)])
            self.closed = True


def remote_process(target, source_path, args, log_label):
    """Launch only inventoried fixture source in the preinstalled dedicated peer Pod."""
    name = Path(source_path).name
    require(name in FIXTURES and re.fullmatch(r'[a-zA-Z0-9_-]+', log_label), 'unrecognized fixture launch')
    pod = kube_json(target, 'pod', target['pod'])
    require(pod['metadata'].get('labels', {}).get('app.kubernetes.io/name') == 'dot17-qualification-peer',
            'fixture execution requires the isolated qualification peer Pod')
    status = next(row for row in pod['status']['containerStatuses'] if row['name'] == target['container'])
    require(status['ready'] and status['imageID'].split('@')[-1] == target['image_digest'], 'peer image identity differs')
    if name == 'dot19_mcp_peer.py':
        companion = ROOT / 'tests/fixtures/dot19_mcp_wire.json'
        actual = exec_python(target, "import hashlib,pathlib; print(hashlib.sha256(pathlib.Path('/opt/dot17/dot19_mcp_wire.json').read_bytes()).hexdigest())").decode().strip()
        require(actual == digest(companion), 'mounted MCP wire companion differs from inventoried source')
    identity = json.loads(exec_python(target, LAUNCH, ['/opt/dot17/' + name, digest(FIXTURES[name]),
        json.dumps([str(value) for value in args]), BASE + log_label + '-' + uuid.uuid4().hex + '.log']))
    return RemoteProcess(target, identity)


class KubernetesProvider:
    def __init__(self, spec, evidence, label):
        self.spec, self.evidence, self.label = spec, evidence, label
        self.target = spec['qualification_peer']
        self.trace = dict(self.target, kind='kubernetes_file', path=BASE + label + '-' + uuid.uuid4().hex + '.jsonl')
        self.control_path = BASE + 'provider-control-' + uuid.uuid4().hex + '.key'
        self.process = None

    def start(self):
        self.process = remote_process(self.target, 'provider_peer.py',
            ['--port', self.spec['provider_port'], '--trace', file_path(self.trace),
             '--provider-sha256', digest(ROOT / 'scripts/dot/provider.py'),
             '--control-token-file', self.control_path], self.label)
        def listening():
            try:
                return json.loads(exec_python(self.target,
                    "import json,sys,urllib.request; print(json.dumps(json.load(urllib.request.urlopen('http://127.0.0.1:'+sys.argv[1]+'/v1/models',timeout=2))))",
                    [self.spec['provider_port']]))
            except (Blocked, OSError):
                return None
        poll(listening, bool, 'real qualification provider did not bind')
        self.evidence.observe(self.label + '-remote-peer', {'target': self.target, 'trace': self.trace,
            'process': self.process.identity, 'scope': 'controlled upstream only; installed UAR owns all execution'})

    def rows(self):
        return read_trace(self.trace)

    def control(self, operation, marker):
        require(operation in ('state', 'release'), 'unknown controlled provider operation')
        value = exec_python(self.target, """import json,pathlib,sys,urllib.error,urllib.parse,urllib.request
port,key,operation,marker=sys.argv[1:]
token=pathlib.Path(key).read_text()
url='http://127.0.0.1:'+port+'/__dot/'
if operation=='state':
 request=urllib.request.Request(url+'state?marker='+urllib.parse.quote(marker,safe=''),headers={'Authorization':'Bearer '+token})
else:
 request=urllib.request.Request(url+'release',data=json.dumps({'marker':marker}).encode(),method='POST',headers={'Authorization':'Bearer '+token,'Content-Type':'application/json'})
try:
 with urllib.request.urlopen(request,timeout=5) as response:print(json.dumps(json.load(response)))
except urllib.error.HTTPError as error:
 if operation=='state' and error.code==404:print(json.dumps({'state':None}))
 else:raise
""", [self.spec['provider_port'], self.control_path, operation, marker])
        return json.loads(value)

    def barrier_state(self, marker):
        return self.control('state', marker)['state']

    def release(self, marker):
        return self.control('release', marker)

    def stop(self):
        if self.process:
            self.process.close()
        exec_python(self.target, "import pathlib,sys; pathlib.Path(sys.argv[1]).unlink(missing_ok=True)", [self.control_path])
