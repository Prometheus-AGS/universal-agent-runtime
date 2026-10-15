"""Prepare immutable schema inputs for the local full-phase acceptance gate.

This is not called during implementation. Product dependencies are not changed.
"""
import base64
import hashlib
import io
import json
from pathlib import Path
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
PROVENANCE = ROOT / '.kbd-orchestrator/phases/archive/2026-10-14-dot-feature-support/implementation/protocol-sources/agui-published-provenance.json'


def prepare(directory):
    directory = Path(directory).resolve()
    directory.mkdir(parents=True, exist_ok=True)
    provenance = json.loads(PROVENANCE.read_text())
    assert provenance['name'] == '@ag-ui/core' and provenance['version'] == '1.0.2'
    with urllib.request.urlopen(provenance['dist']['tarball'], timeout=60) as response:
        raw = response.read()
    integrity = 'sha512-' + base64.b64encode(hashlib.sha512(raw).digest()).decode()
    assert integrity == provenance['dist']['integrity'], 'Published schema integrity mismatch'
    with tarfile.open(fileobj=io.BytesIO(raw), mode='r:gz') as archive:
        for member in archive.getmembers():
            target = (directory / member.name).resolve()
            assert target.is_relative_to(directory), 'Invalid schema archive path'
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            elif member.isfile():
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(archive.extractfile(member).read())
            else:
                raise AssertionError('Unexpected schema archive member')
    # Reuse the exact already-locked compatible peer, without installing/upgrading product packages.
    zod = ROOT / 'node_modules/.pnpm/zod@3.25.76/node_modules/zod'
    assert json.loads((zod / 'package.json').read_text())['version'] == '3.25.76'
    dependencies = directory / 'package/node_modules'
    dependencies.mkdir(exist_ok=True)
    link = dependencies / 'zod'
    if not link.exists():
        link.symlink_to(zod, target_is_directory=True)
    module = directory / 'package/dist/schemas.mjs'
    assert module.is_file()
    (directory / 'provenance.json').write_text(json.dumps({
        'name': provenance['name'], 'version': provenance['version'],
        'integrity': integrity, 'zod': '3.25.76', 'module': str(module),
    }, indent=2) + '\n')
    return module


if __name__ == '__main__':
    import sys
    print(prepare(sys.argv[1]))
