"""Author concrete qualification objects after the runner boundary; never apply."""
import argparse
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/dot'))
from acceptance import boundary, digest, read, require, save


def render(config):
    namespace = config['namespace']
    require(namespace == 'dot17-uar', 'qualification namespace must be explicit dedicated dot17-uar')
    image = config['python_image']
    require(re.fullmatch(r'[^\s]+@sha256:[0-9a-f]{64}', image), 'peer image must be immutable')
    claim = config['existing_claim']
    require(claim == 'surrealdb-data-surrealdb-0', 'qualification peer must share the dedicated database claim')
    trace_subpath = config['trace_subpath']
    require(trace_subpath == 'peer', 'qualification peer must use its separate peer directory')
    files = {'provider.py': 'scripts/dot/provider.py', 'provider_peer.py': 'scripts/dot/provider_peer.py',
        'dot19_mcp_peer.py': 'tests/fixtures/dot19_mcp_peer.py',
        'dot19_mcp_wire.json': 'tests/fixtures/dot19_mcp_wire.json',
        'dot17_effect_mcp.py': 'tests/fixtures/dot17_effect_mcp.py',
        'controller.py': 'scripts/supervisor/controller.py'}
    data = {name: (ROOT / path).read_text() for name, path in files.items()}
    name = config['name']
    require(re.fullmatch(r'dot17-[a-z0-9-]+', name), 'dedicated qualification resource name required')
    labels = {'app.kubernetes.io/name': 'dot17-qualification-peer', 'app.kubernetes.io/instance': name}
    metadata = {'namespace': namespace, 'name': name, 'labels': labels}
    ports = config['ports']
    require(set(ports) == {'provider', 'mcp'} and len(set(ports.values())) == 2
            and all(isinstance(port, int) and 1024 <= port <= 65535 for port in ports.values()), 'two distinct peer ports required')
    items = [
        {'apiVersion': 'v1', 'kind': 'ConfigMap', 'metadata': metadata, 'immutable': True, 'data': data},
        {'apiVersion': 'v1', 'kind': 'Pod', 'metadata': metadata, 'spec': {
            'automountServiceAccountToken': False, 'restartPolicy': 'Never',
            'securityContext': {'runAsNonRoot': True, 'runAsUser': 65532, 'runAsGroup': 65532, 'fsGroup': 65532},
            'affinity': {'podAffinity': {'requiredDuringSchedulingIgnoredDuringExecution': [{
                'labelSelector': {'matchLabels': {'app.kubernetes.io/name': 'surrealdb',
                    'app.kubernetes.io/component': 'database'}},
                'namespaces': [namespace], 'topologyKey': 'kubernetes.io/hostname'}]}},
            'containers': [{'name': 'peer', 'image': image, 'imagePullPolicy': 'IfNotPresent',
                'command': ['python3', '-c', 'import time; time.sleep(31536000)'],
                'securityContext': {'allowPrivilegeEscalation': False, 'readOnlyRootFilesystem': True,
                    'capabilities': {'drop': ['ALL']}},
                'env': [{'name': 'PYTHONDONTWRITEBYTECODE', 'value': '1'}],
                'resources': {'requests': {'cpu': '100m', 'memory': '64Mi'}, 'limits': {'cpu': '1', 'memory': '256Mi'}},
                'ports': [{'name': key, 'containerPort': value} for key, value in ports.items()],
                'volumeMounts': [{'name': 'fixtures', 'mountPath': '/opt/dot17', 'readOnly': True},
                    {'name': 'trace', 'mountPath': '/var/lib/uar/dot17', 'subPath': trace_subpath}]}],
            'volumes': [{'name': 'fixtures', 'configMap': {'name': name}},
                        {'name': 'trace', 'persistentVolumeClaim': {'claimName': claim}}]}},
        {'apiVersion': 'v1', 'kind': 'Service', 'metadata': metadata, 'spec': {'selector': labels,
            'ports': [{'name': key, 'port': value, 'targetPort': key} for key, value in ports.items()]}},
        {'apiVersion': 'networking.k8s.io/v1', 'kind': 'NetworkPolicy', 'metadata': metadata, 'spec': {
            'podSelector': {'matchLabels': labels}, 'policyTypes': ['Ingress', 'Egress'],
            'ingress': [{'from': [{'podSelector': {'matchLabels': config['runtime_labels']}}],
                         'ports': [{'protocol': 'TCP', 'port': port} for port in ports.values()]}],
            'egress': []}}]
    return {'apiVersion': 'v1', 'kind': 'List', 'items': items}, {name: digest(ROOT / path) for name, path in files.items()}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--inventory', required=True)
    parser.add_argument('--config', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    boundary(args.inventory)
    value, sources = render(read(args.config))
    save(args.output, value)
    save(args.output + '.sources.json', sources)


if __name__ == '__main__':
    main()
