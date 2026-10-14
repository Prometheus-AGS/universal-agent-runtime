"""Read-only Linux observation for dedicated recovery fault qualification.

Streamed through pinned SSH. Never creates a controller, writes protected state,
changes a service, or changes the network. Unsupported inspection blocks proof.
"""
import base64
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import socket
import sqlite3
import ssl
import sys
import time
import urllib.parse
import urllib.error
import urllib.request


def require(value, message):
    if not value:
        raise RuntimeError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def process(options):
    require(sys.platform == 'linux', 'fault process observation currently requires Linux procfs')
    pid = Path(options['pid_file']).read_text().strip()
    require(pid.isdecimal() and int(pid) > 1, 'invalid dedicated service PID')
    root = Path('/proc') / pid
    stat = (root / 'stat').read_text().rsplit(')', 1)[1].split()
    require(stat[0] not in ('Z', 'X'), 'dedicated service is not alive')
    require(root.stat().st_uid == options['uid'], 'dedicated service UID changed')
    args = [part.decode() for part in (root / 'cmdline').read_bytes().split(b'\0') if part]
    config = str(Path(options['config']).resolve())
    require(args.count('--config') == 1 and args[args.index('--config') + 1] == config,
            'actual service command does not use the retained config')
    require(sha(config) == options['config_sha256'], 'dedicated service config changed')
    executable = str((root / 'exe').resolve())
    require(executable == str(Path(options['executable']).resolve()) and
            sha(executable) == options['executable_sha256'], 'actual service executable changed')
    if options.get('source'):
        source = str(Path(options['source']).resolve())
        require(source in args and sha(source) == options['source_sha256'], 'controller source changed')
    if options.get('network') or options.get('embedded_volume'):
        require((root / 'ns/net').stat().st_ino == Path('/proc/self/ns/net').stat().st_ino,
                'observer is outside the runtime network namespace')
        environment = dict(item.split(b'=', 1) for item in (root / 'environ').read_bytes().split(b'\0') if b'=' in item)
        env = list(environment)
        require(not any(name.upper().startswith((b'UAR_PERSISTENCE', b'UAR_SUPERVISOR')) for name in env),
                'runtime persistence/supervisor environment override is not represented by config')
        if options.get('application_listener'):
            allowed_log_format = {b'UAR_SERVER__LOG_FORMAT'} if options.get('embedded_volume') else set()
            require('--port' not in args and not any(name.upper().startswith((b'UAR_SERVER', b'UAR_RESILIENCE')) for name in env if name not in allowed_log_format),
                    'application listener or timeout override is not represented by config')
        if options.get('embedded_volume'):
            require(environment.get(b'UAR_SERVER__LOG_FORMAT') == b'json' and
                    environment.get(b'UAR_LOG_FILE', b'').decode() == options['embedded_volume']['runtime_log'],
                    'embedded fault requires the actual runtime JSON log on a separate volume')
    return {'identity': pid + ':' + stat[19], 'uid': options['uid'],
            'executable_sha256': options['executable_sha256'], 'config_sha256': options['config_sha256'],
            'source_sha256': options.get('source_sha256')}


def connectivity(options, config):
    purpose = options['network']
    if purpose == 'persistence':
        require(config['persistence']['provider'] == 'surreal', 'requires actual SurrealDB persistence')
        endpoint = config['persistence']['database_url']
    elif purpose == 'predecessor_controller':
        primary = config['supervisor_recovery']
        entries = [primary, *primary.get('recovery_peers', [])]
        matches = [entry for entry in entries if entry['controller_id'] == options['controller_id']]
        require(len(matches) == 1, 'predecessor controller is not uniquely configured')
        endpoint = matches[0]['endpoint']
    else:
        raise RuntimeError('unsupported network observation')
    url = urllib.parse.urlsplit(endpoint)
    require(url.scheme in ('http', 'https', 'ws', 'wss') and url.hostname and not url.username,
            'fault stage requires a network endpoint without embedded credentials')
    # A literal address prevents a changed DNS answer from masquerading as a
    # partition of the actual configured predecessor or persistence service.
    address = ipaddress.ip_address(url.hostname)
    port = url.port or (443 if url.scheme in ('https', 'wss') else 80)
    observations = []
    for _ in range(3):
        started = time.monotonic()
        try:
            with socket.create_connection((str(address), port), timeout=3):
                connected, error = True, None
        except OSError as failure:
            connected, error = False, type(failure).__name__
        observations.append({'connected': connected, 'error_type': error,
                             'elapsed_seconds': time.monotonic() - started})
        time.sleep(.25)
    return {'purpose': purpose, 'address': str(address), 'port': port, 'attempts': observations}


def embedded_volume(options, config, identity):
    """Observe a bounded guest-owned filesystem; never fill, mount or repair it."""
    wanted = options['embedded_volume']
    pid = identity['identity'].split(':', 1)[0]
    proc = Path('/proc') / pid
    require(options['uid'] > 0, 'fault runtime must not bypass filesystem reservations as root')
    require((proc / 'ns/mnt').stat().st_ino == Path('/proc/self/ns/mnt').stat().st_ino,
            'observer must share runtime mount namespace')
    def platform_text(path):
        value = Path(path)
        return value.read_bytes().replace(b'\0', b' ').decode().strip() if value.is_file() else ''
    vendor = platform_text('/sys/class/dmi/id/sys_vendor')
    product = platform_text('/sys/class/dmi/id/product_name')
    device_tree = platform_text('/proc/device-tree/model') + ' ' + platform_text('/proc/device-tree/compatible')
    known_guest = any(value in (vendor + ' ' + product).lower() for value in
                      ('qemu', 'kvm', 'vmware', 'virtualbox', 'parallels', 'virtual machine', 'bhyve'))
    known_guest = known_guest or any(value in device_tree.lower() for value in ('linux,dummy-virt', 'qemu,virt', 'apple virtual machine'))
    require(known_guest and wanted['dedicated_disposable_guest_volume'] is True,
            'embedded disk fault requires a dedicated virtual guest, never the workstation filesystem')
    require(config['persistence']['provider'] == 'surreal', 'embedded fault requires SurrealKV')
    endpoint = config['persistence']['database_url']
    require(endpoint.startswith('surrealkv:///'), 'explicit absolute SurrealKV directory required')
    database = Path(endpoint.removeprefix('surrealkv://')).resolve(strict=True)
    mount = Path(wanted['mountpoint']).resolve(strict=True)
    require(str(mount).startswith('/var/lib/uar/dot17-volume/') and database.is_relative_to(mount)
            and database != mount, 'database outside dedicated fault mount')
    rows = [line.split() for line in Path('/proc/self/mountinfo').read_text().splitlines()]
    mounted = [row for row in rows if row[4] == str(mount)]
    require(len(mounted) == 1, 'fault directory is not an exact distinct filesystem mount')
    row = mounted[0]
    separator = row.index('-')
    require(row[3] == '/' and row[separator + 1] in ('ext4', 'xfs') and 'rw' in row[5].split(','),
            'fault mount must be a writable whole ext4/xfs filesystem')
    require(sum(other[2] == row[2] for other in rows) == 1 and
            not any(other[4].startswith(str(mount) + '/') for other in rows),
            'fault device aliases or nested mounts are not isolated')
    device = Path(wanted['device']).resolve(strict=True)
    uuid_device = Path('/dev/disk/by-uuid') / wanted['filesystem_uuid']
    require(uuid_device.resolve(strict=True) == device and
            f'{os.major(device.stat().st_rdev)}:{os.minor(device.stat().st_rdev)}' == row[2],
            'actual filesystem device/UUID differs from dedicated volume')
    require(database.stat().st_dev == mount.stat().st_dev != Path('/').stat().st_dev,
            'fault volume shares guest root or database lives on a different device')
    capacity = int((Path('/sys/dev/block') / row[2] / 'size').read_text()) * 512
    require(32 * 1024 * 1024 <= capacity <= wanted['maximum_bytes'] <= 2 * 1024**3,
            'fault device exceeds the bounded disposable volume capacity')
    files = []
    for descriptor in (proc / 'fd').iterdir():
        try:
            path = descriptor.resolve(strict=True)
            if path.is_relative_to(database) and descriptor.stat().st_dev == database.stat().st_dev:
                info = (proc / 'fdinfo' / descriptor.name).read_text().splitlines()
                flags = int(next(line.split()[1] for line in info if line.startswith('flags:')), 8)
                if flags & os.O_ACCMODE:
                    files.append(str(path))
        except FileNotFoundError:
            continue
    require(files, 'actual runtime has no writable SurrealKV file on the fault device')
    log = Path(wanted['runtime_log']).resolve(strict=True)
    require(log.stat().st_dev != mount.stat().st_dev, 'runtime diagnostic log would be lost on the fault volume')
    log_stat = log.stat()
    require(any(descriptor.stat().st_ino == log_stat.st_ino and descriptor.stat().st_dev == log_stat.st_dev
                for descriptor in (proc / 'fd').iterdir() if descriptor.exists()),
            'runtime does not own the declared JSON log file')
    result = {'database': str(database), 'mountpoint': str(mount), 'device': str(device),
              'filesystem_uuid': wanted['filesystem_uuid'], 'major_minor': row[2], 'capacity_bytes': capacity,
              'guest_vendor': vendor, 'guest_product': product, 'guest_device_tree': device_tree,
              'mount_namespace': Path('/proc/self/ns/mnt').stat().st_ino,
              'runtime_open_database_files': sorted(files),
              'log': {'device': log_stat.st_dev, 'inode': log_stat.st_ino, 'size': log_stat.st_size}}
    measurements = []
    for _ in range(3):
        stats = os.statvfs(mount)
        measurements.append({'available_bytes': stats.f_bavail * stats.f_frsize,
                             'free_bytes': stats.f_bfree * stats.f_frsize})
        time.sleep(.25)
    result['measurements'] = measurements
    if options.get('log_since'):
        baseline = options['log_since']
        require((log_stat.st_dev, log_stat.st_ino) == (baseline['device'], baseline['inode']) and
                log_stat.st_size >= baseline['size'] and log_stat.st_size - baseline['size'] <= 16 * 1024 * 1024,
                'runtime log rotated, truncated or exceeded bounded diagnostic read')
        with log.open('rb') as stream:
            stream.seek(baseline['size'])
            raw = stream.read(16 * 1024 * 1024 + 1)
        result['settlement_errors'] = []
        for line in raw.split(b'\n')[:-1]:
            try:
                value = json.loads(line)
            except ValueError:
                continue
            fields = value.get('fields', {})
            if fields.get('call_id') in options['call_ids'] and fields.get('message') in (
                    'Durable usage remains pending', 'Execution completed with unresolved durable accounting'):
                result['settlement_errors'].append({'call_id': fields['call_id'], 'message': fields['message'],
                    'error': fields.get('error', ''), 'line_sha256': hashlib.sha256(line).hexdigest()})
    return result


def original_pod(config, registration, private):
    """GET only: API object absence never substitutes for physical exclusion."""
    domain = registration['domain']
    require(domain['kind'] == 'kubernetes' and domain['cluster_id'] == config['cluster_id'],
            'Pod API observation requires the exact registered cluster')
    settings = config['deployments'][registration['deployment_id']]
    require(settings['namespace'] == domain['namespace'] and settings['pod_uid'] == domain['pod_uid'],
            'controller deployment differs from retained original Pod')
    path = '/api/v1/namespaces/' + urllib.parse.quote(domain['namespace'], safe='') + '/pods/' + urllib.parse.quote(settings['pod_name'], safe='')
    require(path == private['pod_path'], 'recorded Pod path differs from controller registration')
    base = urllib.parse.urlsplit(config['kubernetes_api'])
    require(base.scheme == 'https' and base.hostname and not base.username and not base.query and not base.fragment,
            'authenticated TLS Kubernetes API required')
    token = Path(config['kubernetes_token']).read_text().strip()
    require(token and '\r' not in token and '\n' not in token, 'Kubernetes observation token unavailable')
    ca = config.get('kubernetes_ca')
    context = ssl.create_default_context(cafile=ca) if ca else ssl.create_default_context()
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *_):
            raise RuntimeError('Kubernetes observation redirect refused')
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
                                         urllib.request.HTTPSHandler(context=context))
    request = urllib.request.Request(config['kubernetes_api'].rstrip('/') + path,
        headers={'Authorization': 'Bearer ' + token, 'Accept': 'application/json'}, method='GET')
    result = {'cluster_id': domain['cluster_id'], 'namespace': domain['namespace'], 'pod_name': settings['pod_name'],
              'original_uid': domain['pod_uid'], 'api': config['kubernetes_api'], 'ca_sha256': sha(ca) if ca else None}
    try:
        with opener.open(request, timeout=15) as response:
            require(response.status == 200, 'unexpected original Pod GET status')
            raw = response.read(4 * 1024 * 1024 + 1)
            require(len(raw) <= 4 * 1024 * 1024, 'Pod observation response limit')
            pod = json.loads(raw)
            metadata = pod['metadata']
            require(metadata['namespace'] == domain['namespace'] and metadata['name'] == settings['pod_name'],
                    'API returned a different Pod identity')
            result.update(status=200, observed_uid=metadata['uid'], resource_version=metadata['resourceVersion'],
                          deletion_timestamp=metadata.get('deletionTimestamp'))
    except urllib.error.HTTPError as error:
        require(error.code == 404, 'Kubernetes API unavailable or unauthorized; disappearance is unproven')
        error.close()
        result.update(status=404, observed_uid=None)
    return result


def observe(options):
    before = process(options)
    config = json.loads(Path(options['config']).read_text())
    result = {'machine_id': Path('/etc/machine-id').read_text().strip(),
              'boot_id': Path('/proc/sys/kernel/random/boot_id').read_text().strip(),
              'process': before}
    if options.get('network') or options.get('embedded_volume'):
        if options.get('network'):
            result['network'] = connectivity(options, config)
        else:
            result['embedded_volume'] = embedded_volume(options, config, before)
        if options.get('application_listener'):
            require(config['server']['host'] == '127.0.0.1' and config['resilience']['timeout_disabled'] is True,
                    'storage fault requires a direct loopback listener and explicit operator-wait timeout configuration')
            pid = before['identity'].split(':', 1)[0]
            sockets = set()
            for path in (Path('/proc') / pid / 'fd').iterdir():
                try:
                    sockets.add(str(path.readlink()))
                except FileNotFoundError:
                    # A concurrently closed unrelated connection is not a
                    # replacement for the required live listener below.
                    continue
            port = config['server']['port']
            matches = [line.split() for line in (Path('/proc') / pid / 'net/tcp').read_text().splitlines()[1:]
                       if line.split()[1] == '0100007F:' + format(port, '04X') and line.split()[3] == '0A']
            require(any('socket:[' + row[9] + ']' in sockets for row in matches),
                    'configured HTTP listener is not owned by the observed runtime process')
            result['application_listener'] = {'host': '127.0.0.1', 'port': port,
                'network_namespace': Path('/proc/self/ns/net').stat().st_ino}
    else:
        uri = Path(config['journal']).resolve().as_uri() + '?mode=ro'
        with sqlite3.connect(uri, uri=True, timeout=5) as db:
            db.execute('BEGIN')
            row = db.execute('SELECT registration,private_data FROM domains WHERE epoch=?',
                             (options['runtime_epoch'],)).fetchone()
            require(row is not None, 'actual predecessor registration unavailable')
            excluded = db.execute('SELECT observation FROM excluded_domains WHERE registration=?',
                                  (row[0],)).fetchone()
            # Decode exact retained requests instead of trusting a supplied
            # challenge identifier or a count of unrelated controller records.
            retained = []
            for challenge, request, receipt in db.execute('SELECT challenge,request,receipt FROM exclusions'):
                request = json.loads(request)
                if request['registration']['runtime_epoch'] == options['runtime_epoch']:
                    retained.append({'id': challenge, 'request': request, 'receipt': json.loads(receipt)})
            result['journal'] = {'registration': json.loads(row[0]),
                                 'observation': json.loads(excluded[0]) if excluded else None,
                                 'exclusions': sorted(retained, key=lambda item: item['id'])}
            if options.get('original_pod'):
                result['original_pod'] = original_pod(config, json.loads(row[0]), json.loads(row[1]))
    require(process(options) == before, 'service changed during observation')
    return result


if __name__ == '__main__':
    try:
        print(json.dumps(observe(json.loads(base64.urlsafe_b64decode(sys.argv[1])))))
    except Exception:
        # Config, process environment, SQLite contents and SSH diagnostics stay
        # private on the dedicated host rather than leaking in a traceback.
        raise SystemExit('dedicated recovery fault observation unavailable')
