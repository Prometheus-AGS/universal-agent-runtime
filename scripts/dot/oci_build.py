"""Local multi-architecture OCI export. No push, tag publication or product test."""
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile

from acceptance import ROOT, Blocked, digest, read, require
from build_delivery import run_candidate_inputs


def run_oci_build(config, evidence):
    run_candidate_inputs(evidence)
    builder = config['builder']
    require(re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*', builder), 'explicit installed builder name required')
    cargo_cache_key = config.get('cargo_cache_key')
    require(isinstance(cargo_cache_key, str) and re.fullmatch(r'[0-9a-f]{64}', cargo_cache_key),
            'oci_build.cargo_cache_key must be a 64-character lowercase hexadecimal SHA-256')
    archive = evidence.directory / 'runtime-linux-amd64-arm64.oci.tar'
    metadata = evidence.directory / 'buildx-metadata.json'
    command = ['docker', 'buildx', 'build', '--builder', builder,
               '--platform', 'linux/amd64,linux/arm64', '--file', str(ROOT / 'Dockerfile'),
               '--build-arg', 'UAR_CARGO_BUILD_JOBS=1',
               '--build-arg', 'UAR_CARGO_CACHE_KEY=' + cargo_cache_key,
               '--output', 'type=oci,dest=' + str(archive), '--metadata-file', str(metadata),
               '--provenance=mode=max', '--progress=plain', str(ROOT)]
    environment = dict(os.environ, BUILDX_METADATA_PROVENANCE='max')
    secret = config.get('github_token_file')
    if secret:
        require(Path(secret).is_file(), 'private build credential file missing')
        command[-1:-1] = ['--secret', 'id=github_token,src=' + str(Path(secret).resolve())]
    with open(evidence.directory / 'oci-build.log', 'xb') as output:
        completed = subprocess.run(command, cwd=ROOT, env=environment, stdout=output, stderr=subprocess.STDOUT)
    # Record only the secret's fixed identifier, not a private filesystem path.
    public_command = list(command)
    if secret:
        public_command[public_command.index('--secret') + 1] = 'id=github_token,src=<private-file>'
    evidence.observe('oci-build-command', {'command': public_command, 'returncode': completed.returncode,
        'log_sha256': digest(output.name), 'execution': 'local', 'publication': 'not_performed'})
    require(completed.returncode == 0, 'local OCI build failed; no architecture credit')
    require(archive.is_file() and metadata.is_file(), 'OCI exporter did not produce required artifacts')
    info = read(metadata)
    require(info.get('buildx.build.provenance') and info.get('containerimage.digest'), 'actual BuildKit provenance or digest missing')
    architectures = {}
    runtime_binaries = {}
    observed_digests = set()
    with tarfile.open(archive) as image:
        members = {member.name.removeprefix('./'): member for member in image.getmembers()}
        require(len(members) == len(image.getmembers()), 'duplicate OCI archive path')
        def raw(name):
            member = members.get(name)
            require(member is not None and member.isfile() and member.size <= 64 * 1024 * 1024,
                    'missing or invalid OCI metadata member: ' + name)
            stream = image.extractfile(member)
            require(stream is not None, 'OCI member is unreadable')
            return stream.read()
        def blob(descriptor):
            algorithm, value = descriptor['digest'].split(':', 1)
            require(algorithm == 'sha256' and re.fullmatch(r'[0-9a-f]{64}', value), 'unsupported OCI content digest')
            body = raw('blobs/sha256/' + value)
            require(len(body) == descriptor['size'] and hashlib.sha256(body).hexdigest() == value, 'OCI metadata digest differs')
            observed_digests.add(descriptor['digest'])
            return json.loads(body)
        def visit(descriptor):
            value = blob(descriptor)
            if 'manifests' in value:
                for child in value['manifests']:
                    visit(child)
                return
            if 'config' not in value:
                return
            config_value = blob(value['config'])
            architecture = config_value.get('architecture')
            if config_value.get('os') != 'linux' or architecture not in ('amd64', 'arm64'):
                return  # Attestation manifests are not runtime architectures.
            require(architecture not in architectures, 'duplicate OCI runtime architecture')
            executable = evidence.directory / architecture / 'universal-agent-runtime'
            executable.parent.mkdir(mode=0o700)
            runtime_path = 'usr/local/bin/universal-agent-runtime'
            # Read only the packaged executable. Never extract archive-controlled
            # paths, symlinks or unrelated files onto the qualification host.
            whiteouts = {'.wh..wh..opq', 'usr/.wh..wh..opq', 'usr/local/.wh..wh..opq',
                         'usr/local/bin/.wh..wh..opq', '.wh.usr',
                         'usr/.wh.local', 'usr/local/.wh.bin',
                         'usr/local/bin/.wh.universal-agent-runtime'}
            # Validate each compressed layer before exporting its executable.
            for layer in value['layers']:
                algorithm, value_hash = layer['digest'].split(':', 1)
                require(algorithm == 'sha256' and re.fullmatch(r'[0-9a-f]{64}', value_hash), 'unsupported layer digest')
                member = members.get('blobs/sha256/' + value_hash)
                require(member is not None and member.isfile() and member.size == layer['size'], 'OCI layer missing')
                result = hashlib.sha256()
                stream = image.extractfile(member)
                for chunk in iter(lambda: stream.read(1024 * 1024), b''):
                    result.update(chunk)
                require(result.hexdigest() == value_hash, 'OCI layer content differs')
                found, removes_previous = False, False
                with image.extractfile(member) as compressed:
                    with tarfile.open(fileobj=compressed, mode='r|*') as filesystem:
                        for entry in filesystem:
                            name = entry.name.removeprefix('./').lstrip('/')
                            if name in whiteouts:
                                removes_previous = True
                            if name != runtime_path:
                                continue
                            require(not found and entry.isfile() and 64 <= entry.size <= 1024 * 1024 * 1024,
                                    'packaged runtime must be one bounded regular executable per layer')
                            found = True
                            with filesystem.extractfile(entry) as binary, executable.open('wb') as output:
                                header = binary.read(64)
                                require(len(header) == 64 and header[:6] == b'\x7fELF\x02\x01' and
                                        int.from_bytes(header[18:20], 'little') == {'amd64': 62, 'arm64': 183}[architecture],
                                        'packaged executable architecture differs from OCI manifest')
                                output.write(header)
                                for chunk in iter(lambda: binary.read(1024 * 1024), b''):
                                    output.write(chunk)
                                require(output.tell() == entry.size, 'packaged executable is truncated')
                            os.chmod(executable, 0o700)
                if removes_previous and not found and executable.exists():
                    executable.unlink()
            require(executable.is_file(), 'runtime executable absent from final OCI filesystem')
            architectures[architecture] = descriptor['digest']
            runtime_binaries[architecture] = {'path': str(executable.resolve()), 'sha256': digest(executable),
                'bytes': executable.stat().st_size, 'architecture': architecture, 'os': 'linux',
                'image_digest': descriptor['digest']}
        index_raw = raw('index.json')
        observed_digests.add('sha256:' + hashlib.sha256(index_raw).hexdigest())
        for descriptor in json.loads(index_raw)['manifests']:
            visit(descriptor)
    require(set(architectures) == {'amd64', 'arm64'}, 'both independently materialized image architectures required')
    require(info['containerimage.digest'] in observed_digests, 'build metadata digest does not identify exported OCI content')
    evidence.observe('local-two-architecture-image', {'archive': str(archive), 'sha256': digest(archive),
        'architectures': architectures, 'metadata_sha256': digest(metadata),
        'image_digest': info['containerimage.digest'], 'source_revision': evidence.identity['source_revision'],
        'source_content_sha256': evidence.identity['source']['content_sha256'], 'runtime_binaries': runtime_binaries,
        'dockerfile_sha256': digest(ROOT / 'Dockerfile'), 'inputs': 'candidate-input-provenance.json',
        'scope': 'actual local OCI build; frozen JavaScript/locked Rust commands from inventoried Dockerfile; no bit-for-bit reproducibility or publication claim'})
