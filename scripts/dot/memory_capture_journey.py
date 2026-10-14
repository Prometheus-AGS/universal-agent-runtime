"""Real post-stream memory capture and retention; full-phase gate entry only."""
import hashlib
import http.server
import json
import os
from pathlib import Path
import socket
import ssl
import threading
import time
import urllib.parse
import uuid

from acceptance import Blocked, Host, digest, poll, require
from calendar_journey import GuestClock, timestamp
from journeys import segment
from provider import Provider


class EmbeddingPeer:
    """Controlled external wire peer, with an observable pre-response barrier.

    The pinned client has a fixed HTTPS URL. Guest provisioning supplies DNS
    and a trusted certificate; this peer never disables client TLS validation.
    """
    def __init__(self, spec, evidence):
        addresses = {item[4][0] for item in socket.getaddrinfo('api.openai.com', 443)}
        if addresses != {'127.0.0.1'}:
            raise Blocked('isolated guest api.openai.com override must resolve only to 127.0.0.1')
        for field in ['certificate', 'private_key', 'provisioning_evidence']:
            value = spec[field]
            require(Path(value['path']).is_file() and digest(value['path']) == value['sha256'],
                    'embedding peer provisioning file changed: ' + field)
        self.trace = evidence.directory / 'memory-embedding-requests.jsonl'
        self.lock = threading.Lock()
        self.records = []
        self.holds = {}
        peer = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def do_POST(self):
                if self.path != '/v1/embeddings':
                    self.send_error(404)
                    return
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                require(body['model'] == 'text-embedding-3-small', 'unexpected configured embedding model')
                inputs = body['input']
                require(isinstance(inputs, list) and all(isinstance(value, str) for value in inputs), 'unexpected embedding input')
                request = uuid.uuid4().hex
                peer.record({'event': 'received', 'id': request, 'inputs': inputs})
                with peer.lock:
                    gates = [peer.holds[value] for value in inputs if value in peer.holds]
                for gate in gates:
                    if not gate.wait(180):
                        self.send_error(504, 'qualification controller did not release embedding response')
                        return
                vectors = []
                for index, value in enumerate(inputs):
                    vector = [0.0] * 1536
                    vector[int.from_bytes(hashlib.sha256(value.encode()).digest()[:2], 'big') % 1536] = 1.0
                    vectors.append({'object': 'embedding', 'index': index, 'embedding': vector})
                payload = json.dumps({'object': 'list', 'data': vectors, 'model': body['model'],
                                      'usage': {'prompt_tokens': len(inputs), 'total_tokens': len(inputs)}}).encode()
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
                self.wfile.flush()
                peer.record({'event': 'responded', 'id': request, 'inputs': inputs})

        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(spec['certificate']['path'], spec['private_key']['path'])
        self.server = http.server.ThreadingHTTPServer(('127.0.0.1', 443), Handler)
        self.server.socket = context.wrap_socket(self.server.socket, server_side=True)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def record(self, row):
        with self.lock:
            with open(self.trace, 'a', encoding='utf-8') as stream:
                os.chmod(self.trace, 0o600)
                stream.write(json.dumps(row) + '\n')
                stream.flush()
                os.fsync(stream.fileno())
            self.records.append(row)

    def seen(self, content, event):
        with self.lock:
            return [row for row in self.records if row['event'] == event and content in row['inputs']]

    def hold(self, content):
        with self.lock:
            require(content not in self.holds, 'duplicate embedding hold')
            self.holds[content] = threading.Event()

    def release(self, content):
        with self.lock:
            self.holds[content].set()

    def start(self):
        self.thread.start()

    def stop(self):
        with self.lock:
            for gate in self.holds.values():
                gate.set()
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()


def records(host, content):
    return [row for row in host.api('GET', '/api/memory/records')['memories'] if row['content'] == content]


def record_id(row):
    # Pinned Surreal SDK exposes {table,key:{String|Number|Uuid:value}}.
    value = row['id']
    require(value['table'] == 'memory', 'capture returned another record table')
    key = value['key']
    require(len(key) == 1 and next(iter(key)) in ('String', 'Number', 'Uuid'), 'unsupported memory key')
    return 'memory:' + str(next(iter(key.values())))


def path(row):
    return '/api/memory/records/' + segment(record_id(row))


def policy(host, days):
    retention = {'kind': 'until_deleted'} if days is None else {'kind': 'days', 'days': days}
    return host.api('PUT', '/api/memory/policy', {'enabled': True, 'retention': retention})


def chat(host, spec, marker, session=None):
    session = session or str(uuid.uuid4())
    content = json.dumps({'dot_action': 'text', 'marker': marker})
    body = {'model': spec['chat_model'], 'agent_id': spec['chat_agent_id'],
            'messages': [{'role': 'user', 'content': content}], 'stream': True,
            'stream_mode': 'openai', 'memory_enabled': True}
    stream, headers = host.request('POST', '/api/chat/completion', body,
                                  headers={'x-uar-session-id': session})
    require(isinstance(stream, str) and 'data: [DONE]' in stream, 'compatibility chat stream did not finish')
    lower = {name.lower(): value for name, value in headers.items()}
    require(lower.get('x-uar-session-id') == session and lower.get('x-uar-run-id'), 'chat omitted native correlation')
    fragments = []
    for line in stream.splitlines():
        if line.startswith('data: ') and line != 'data: [DONE]':
            value = json.loads(line[6:])
            for choice in value.get('choices', []):
                fragments.append(choice.get('delta', {}).get('content') or '')
    require(''.join(fragments) == marker, 'actual streamed assistant content differs')
    return {'session': session, 'run': lower['x-uar-run-id'], 'user_content': content, 'assistant_content': marker}


def captured(host, stream, days, revision):
    rows = []
    for content in [stream['user_content'], stream['assistant_content']]:
        matches = poll(lambda: records(host, content), bool, 'post-stream automatic capture absent')
        require(len(matches) == 1, 'capture duplicated content identity')
        row = matches[0]
        meta = row['metadata']['uar_capture']
        require(meta['automatic'] and meta['policy_revision'] == revision, 'capture used wrong committed policy')
        require(meta['retention_days'] == days, 'capture used wrong retention')
        expires = meta['effective_expires_at']
        require((expires is None) if days is None else
                timestamp(expires) - timestamp(meta['first_capture']) == days * 86400, 'expiry not anchored at first capture')
        rows.append(row)
    return rows


def absent(hosts, rows):
    for host in hosts:
        for row in rows:
            host.api('GET', path(row), expect=404)
            require(not records(host, row['content']), 'expired capture remains in export')
            host.api('GET', path(row) + '/history', expect=404)
            control = host.api('GET', path(row) + '/control')
            require(control['expired'] or control['deleted'], 'hidden record has no expiry/deletion evidence')


def assert_visible(hosts, rows):
    for host in hosts:
        for row in rows:
            require(host.api('GET', path(row))['content'] == row['content'], 'unexpired capture disappeared')


def queued(host, editor, spec, evidence, embeddings, clock):
    for mode in ['new-policy', 'disable-reenable']:
        prior = policy(editor, 365)
        marker = 'queued-' + mode + '-' + uuid.uuid4().hex
        content = json.dumps({'dot_action': 'text', 'marker': marker})
        embeddings.hold(content)
        stream = chat(host, spec, marker)
        received = poll(lambda: embeddings.seen(content, 'received'), bool, 'queued capture did not reach embedding barrier')
        require(not embeddings.seen(content, 'responded') and not records(host, content), 'capture committed before controlled barrier')
        if mode == 'new-policy':
            current = policy(editor, 30)
            embeddings.release(content)
            rows = captured(host, stream, 30, current['revision'])
            evidence.observe('memory-queue-current-policy', {'prior': prior, 'current': current,
                'stream': stream, 'embedding_received': received, 'records': rows})
        else:
            disabled = editor.api('PUT', '/api/memory/policy', {'enabled': False})
            current = policy(editor, 365)
            require(current['consent_epoch'] > prior['consent_epoch'], 'disable failed to revoke queued epoch')
            embeddings.release(content)
            # Capture processes user then assistant sequentially. Observing the
            # assistant request proves the first save returned after revocation.
            poll(lambda: embeddings.seen(marker, 'responded'), bool, 'revoked queue did not finish embedding sequence')
            deadline = time.monotonic() + 8
            while time.monotonic() < deadline:
                clock.ensure()
                require(not records(host, content) and not records(host, marker), 'old queued consent revived after reenable')
                time.sleep(.25)
            fresh = chat(host, spec, 'fresh-consent-' + uuid.uuid4().hex)
            fresh_rows = captured(host, fresh, 365, current['revision'])
            require(not records(host, content) and not records(host, marker), 'fresh queue revived revoked work')
            evidence.observe('memory-queue-revocation', {'prior': prior, 'disabled': disabled,
                'reenabled': current, 'stream': stream, 'fresh_records': fresh_rows,
                'negative_observation_monotonic_seconds': 8})


def run_memory_capture(config, evidence):
    """Dedicated VM stage; do not overlap calendar or ordinary profile peers."""
    hosts = [Host(value, evidence, 'memory-' + label) for label, value in config['hosts'].items()]
    require(len(hosts) == (1 if evidence.profile == 'local_surreal_kv' else 2), 'memory profile host count mismatch')
    configurations = [json.loads(Path(host.spec['config']).read_text()) for host in hosts]
    memory = [value['memory'] for value in configurations]
    for value in memory:
        require(value['enabled'] and value['auto_capture'] and not value['inject_context'], 'capture fixture configuration differs')
        require(value['embedding_provider'] == 'openai' and value['embedding_model'] == 'text-embedding-3-small',
                'memory queue fixture requires pinned external embedding path')
        require(value.get('openai_api_key') == 'dot-memory-fixture-only', 'never route a real provider key to fixture TLS peer')
        endpoint = value.get('surreal_endpoint')
        if endpoint:
            addresses = {item[4][0] for item in socket.getaddrinfo(urllib.parse.urlparse(endpoint).hostname, None)}
            require(addresses and addresses <= {'127.0.0.1', '::1'}, 'retention database must share actual guest clock')
    if len(hosts) == 2:
        require(memory[0]['surreal_endpoint'] and memory[0]['surreal_endpoint'] == memory[1]['surreal_endpoint']
                and memory[0]['namespace'] == memory[1]['namespace'] and memory[0]['database'] == memory[1]['database'],
                'shared capture hosts must use same guest-local durable memory database')
    database = config['database_provisioning_evidence']
    require(Path(database['path']).is_file() and digest(database['path']) == database['sha256'],
            'actual guest-local durable database provisioning evidence unavailable')
    clock = GuestClock(config['guest_clock'], evidence)
    embeddings = EmbeddingPeer(config['embedding_peer'], evidence)
    provider = Provider(config['provider_port'], evidence.directory / 'memory-provider.jsonl')
    embeddings.start()
    provider.start()
    try:
        clock.move('2027-01-01T12:00:00Z')
        for host in hosts:
            host.start()
        host, editor = hosts[0], hosts[-1]
        initial = host.api('GET', '/api/memory/policy')
        require(not initial['enabled'] and initial['host_capture_enabled'], 'use fresh owner with capture default off')
        disabled_stream = chat(host, config, 'default-off-' + uuid.uuid4().hex)
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            for content in [disabled_stream['user_content'], disabled_stream['assistant_content']]:
                require(not records(host, content) and not embeddings.seen(content, 'received'),
                        'default-off conversation queued an automatic capture')
            time.sleep(.25)
        evidence.observe('memory-capture-default-off', {'stream': disabled_stream,
            'policy': initial, 'negative_observation_monotonic_seconds': 8})
        samples = {}
        for days in [7, 30, 90, 365, None]:
            chosen = policy(editor, days)
            stream = chat(host, config, 'retention-' + str(days) + '-' + uuid.uuid4().hex)
            samples[days] = captured(host, stream, days, chosen['revision'])
            evidence.observe('memory-retention-' + str(days), {'policy': chosen, 'stream': stream, 'records': samples[days]})
        for instant, expired in [('2027-01-09T12:00:00Z', [7]), ('2027-02-01T12:00:00Z', [7, 30]),
                                 ('2027-04-02T12:00:00Z', [7, 30, 90]), ('2028-01-02T12:00:00Z', [7, 30, 90, 365])]:
            clock.move(instant)
            absent(hosts, [row for days in expired for row in samples[days]])
            assert_visible(hosts, [row for days, rows in samples.items() if days not in expired for row in rows])
            evidence.observe('memory-expiry-' + str(expired[-1]), {'instant': instant, 'expired_days': expired,
                'surviving_record_ids': [record_id(row) for days, rows in samples.items() if days not in expired for row in rows]})
        chosen = policy(editor, 365)
        short_stream = chat(host, config, 'shortening-' + uuid.uuid4().hex)
        shortened = captured(host, short_stream, 365, chosen['revision'])
        policy(editor, 7)
        bounds = []
        for row in shortened:
            control = host.api('GET', path(row) + '/control')
            require(timestamp(control['effective_expires_at']) - timestamp(control['first_capture']) == 7 * 86400,
                    'shortening failed to change existing capture bound')
            bounds.append(control)
        chosen = policy(editor, 90)
        for row, control in zip(shortened, bounds):
            require(host.api('GET', path(row) + '/control')['effective_expires_at'] == control['effective_expires_at'],
                    'extension lengthened existing shortened capture')
        extended_stream = chat(host, config, 'new-extension-' + uuid.uuid4().hex)
        extended = captured(host, extended_stream, 90, chosen['revision'])
        clock.move('2028-01-11T12:00:00Z')
        absent(hosts, shortened)
        policy(editor, 365)
        chat(host, config, short_stream['assistant_content'], short_stream['session'])
        poll(lambda: len(embeddings.seen(short_stream['assistant_content'], 'responded')), lambda count: count >= 2,
             'exact expired capture retry did not traverse actual embedding path')
        absent(hosts, shortened)
        assert_visible(hosts, extended)
        evidence.observe('memory-shorten-extend-no-revival', {'shortened': bounds, 'new_records': extended,
            'retried_session': short_stream['session'], 'expired_ids': [record_id(row) for row in shortened]})
        queued(host, editor, config, evidence, embeddings, clock)
        chosen = policy(editor, None)
        unlimited_stream = chat(host, config, 'until-deleted-' + uuid.uuid4().hex)
        unlimited = captured(host, unlimited_stream, None, chosen['revision'])
        clock.move('2029-02-02T12:00:00Z')
        assert_visible(hosts, unlimited)
        for node in reversed(hosts):
            node.stop()
        for node in hosts:
            node.start()
        assert_visible(hosts, unlimited)
        for row in unlimited:
            editor.api('DELETE', path(row))
        absent(hosts, unlimited)
        evidence.observe('memory-until-deleted-restart', {'records': unlimited, 'joined_restart': True,
            'final_clock': '2029-02-02T12:00:00Z', 'deleted_ids': [record_id(row) for row in unlimited]})
    finally:
        try:
            errors = []
            for host in reversed(hosts):
                try:
                    host.stop()
                except Exception as error:
                    errors.append(str(error))
            if errors:
                raise Blocked('memory stage child cleanup unconfirmed: ' + '; '.join(errors))
        finally:
            try:
                embeddings.stop()
            finally:
                try:
                    provider.stop()
                finally:
                    clock.restore()
