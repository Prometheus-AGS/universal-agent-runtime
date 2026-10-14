"""Read actual published release identities; never publish or deploy anything."""
import hashlib
import json
from pathlib import Path
import urllib.parse
import urllib.request

from acceptance import ROOT, Blocked, digest, require


def fetch(url, token=None):
    require(urllib.parse.urlsplit(url).scheme == 'https', 'published evidence requires HTTPS')
    headers = {'Accept': 'application/json, application/vnd.oci.image.index.v1+json, '
                         'application/vnd.oci.image.manifest.v1+json'}
    if token:
        headers['Authorization'] = 'Bearer ' + token
    with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=60) as response:
        raw = response.read(32 * 1024 * 1024 + 1)
    require(len(raw) <= 32 * 1024 * 1024, 'published metadata unexpectedly large')
    return raw


def installed_identity(config):
    """Bind the independently installed binary to actual published manifest bytes.

    URLs point to actual published metadata. Supplied digest strings alone never
    pass; download and hash index and both architecture manifests independently.
    Registry auth, if needed, is supplied in environment, never a receipt.
    """
    import os
    manifest_raw = fetch(config['release_manifest_url'])
    manifest = json.loads(manifest_raw)
    require(manifest['release'] == config['tag'], 'published tag mismatch')
    require(manifest['source']['sha'] == config['source_revision'], 'published source mismatch')
    image_installation = config.get('installation_mode') == 'kubernetes_image'
    installed = None if image_installation else Path(config['installed_binary']).resolve()
    if installed is not None:
        require(not installed.is_relative_to(ROOT), 'source-checkout build is not an independent installation')
        if not installed.is_file():
            raise Blocked('published binary has not been independently installed')
    matches = [item for item in manifest['artifacts'] if item['name'] == config['archive_name']]
    require(len(matches) == 1 and matches[0]['kind'] == 'platform-archive', 'published package missing')
    package = matches[0]
    if installed is not None:
        require(digest(installed) == package['binary']['sha256'], 'installed executable differs from published package')
    archive = Path(config['downloaded_archive'])
    require(archive.is_file() and digest(archive) == package['sha256'], 'downloaded package checksum mismatch')
    token = os.environ.get(config.get('registry_token_env', ''))
    index_raw = fetch(config['image_index_url'], token)
    index_digest = 'sha256:' + hashlib.sha256(index_raw).hexdigest()
    require(index_digest == manifest['image']['digest'], 'published image index differs from release manifest')
    index = json.loads(index_raw)
    arch = {}
    for architecture in ('amd64', 'arm64'):
        entries = [item for item in index['manifests'] if item.get('platform', {}).get('os') == 'linux'
                   and item.get('platform', {}).get('architecture') == architecture]
        require(len(entries) == 1, 'missing/ambiguous Linux image architecture: ' + architecture)
        child = fetch(config['image_manifest_urls'][architecture], token)
        observed = 'sha256:' + hashlib.sha256(child).hexdigest()
        require(observed == entries[0]['digest'], 'architecture digest mismatch: ' + architecture)
        arch[architecture] = observed
    return {'kind': 'independently_installed_publication', 'tag': manifest['release'],
            'source_revision': manifest['source']['sha'], 'manifest_url': config['release_manifest_url'],
            'manifest_sha256': hashlib.sha256(manifest_raw).hexdigest(), 'image_index_digest': index_digest,
            'architecture_digests': arch, 'archive_name': package['name'], 'archive_sha256': package['sha256'],
            'installed_binary': str(installed) if installed is not None else None,
            'installation_mode': 'kubernetes_image' if image_installation else 'native_binary',
            'installed_binary_sha256': package['binary']['sha256'],
            'running_image_identity': 'must_be_observed_by_Host_start' if image_installation else 'not_applicable',
            'product_conformance': 'must_be_observed_by_installed_journeys',
            'deployment_health': 'not_claimed', 'signature_verification': 'separate_release_gate'}
