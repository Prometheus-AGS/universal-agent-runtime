"""Controlled upstream wire peer, never a substitute UAR executor.

The real installed host owns policy, approval, tools, checkpoints and storage.
Each received provider call is independently fsync'd before a response is sent.
"""
import http.server
import hmac
import urllib.parse
import urllib.error
import urllib.request
import json
import os
from pathlib import Path
import threading
import time
import uuid


class Provider:
    def __init__(self, port, trace, control_token=None, bind_host='127.0.0.1'):
        self.trace = Path(trace)
        self.lock = threading.Lock()
        self.control_token = control_token
        self.barriers = {}
        peer = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *args):
                pass

            def authorized_control(self):
                expected = 'Bearer ' + (peer.control_token or '')
                return bool(peer.control_token) and hmac.compare_digest(self.headers.get('Authorization', ''), expected)

            def control_response(self, status, value):
                payload = json.dumps(value).encode()
                self.send_response(status)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            def do_GET(self):
                if self.path.startswith('/__dot/state?'):
                    if not self.authorized_control():
                        self.control_response(401, {'error': 'control_authority_required'})
                        return
                    marker = urllib.parse.parse_qs(urllib.parse.urlsplit(self.path).query).get('marker', [''])[0]
                    with peer.lock:
                        state = peer.barriers.get(marker)
                        value = None if state is None else {key: state[key] for key in ('waiting', 'released', 'terminal_usage_sent')}
                    self.control_response(200 if value is not None else 404, {'marker': marker, 'state': value})
                    return
                payload = json.dumps({'object': 'list', 'data': [
                    {'id': 'gpt-4o-mini', 'object': 'model', 'owned_by': 'dot-fixture'}]}).encode()
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            def do_POST(self):
                if self.path == '/__dot/release':
                    if not self.authorized_control():
                        self.control_response(401, {'error': 'control_authority_required'})
                        return
                    body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                    with peer.lock:
                        state = peer.barriers.get(body['marker'])
                        if state is not None:
                            state['released'] = True
                            state['event'].set()
                    self.control_response(200 if state is not None else 404, {'released': state is not None})
                    return
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                messages = body['messages']
                last = max(index for index, message in enumerate(messages) if message['role'] == 'user')
                text = messages[last]['content']
                if isinstance(text, list):
                    text = ''.join(part.get('text', '') for part in text)
                try:
                    instruction = json.loads(text)
                except (ValueError, TypeError):
                    instruction = {'dot_action': 'text', 'marker': text}
                peer.record({'request_id': uuid.uuid4().hex, 'instruction': instruction,
                             'model': body.get('model'), 'messages': messages,
                             'tool_names': [tool['function']['name'] for tool in body.get('tools', [])]})
                delay = instruction.get('delay_seconds', 0)
                if not isinstance(delay, (int, float)) or not 0 <= delay <= 10:
                    self.send_error(400, 'invalid upstream delay')
                    return
                time.sleep(delay)
                mode = instruction.get('dot_action', 'text')
                after = messages[last + 1:]
                content = instruction.get('marker', 'dot-complete')
                completed = sum(message['role'] == 'tool' for message in after)
                steps = instruction.get('steps', []) if mode == 'tool_sequence' else [instruction]
                issue_tool = mode in ('tool', 'tool_sequence') and completed < len(steps)
                if issue_tool:
                    step = steps[completed]
                    desired = step['tool']
                    name = next((tool['function']['name'] for tool in body.get('tools', [])
                                 if tool['function']['name'] == desired or
                                 tool['function']['name'].endswith('__' + desired)), None)
                    if name is None:
                        self.send_error(422, 'required real host tool not advertised')
                        return
                    delta = {'role': 'assistant', 'tool_calls': [{'index': 0,
                        'id': 'dot-' + uuid.uuid4().hex, 'type': 'function',
                        'function': {'name': name, 'arguments': json.dumps(step['arguments'])}}]}
                    finish = 'tool_calls'
                else:
                    delta = {'role': 'assistant', 'content': content}
                    finish = 'stop'
                def frame(value, reason=None):
                    return 'data: ' + json.dumps({'id': 'dot-' + uuid.uuid4().hex,
                        'object': 'chat.completion.chunk', 'created': 1, 'model': 'gpt-4o-mini',
                        'choices': [{'index': 0, 'delta': value, 'finish_reason': reason}]}, ensure_ascii=False) + '\n\n'
                barrier_mode = instruction.get('wire_mode')
                root_barrier = (barrier_mode == 'root_completion_barrier' and not issue_tool)
                schedule_barrier = (barrier_mode == 'schedule_release_barrier' and not issue_tool
                                    and instruction['marker'] not in peer.barriers)
                if root_barrier and (mode != 'tool_sequence' or not steps or completed != len(steps)):
                    self.send_error(400, 'completed actual tool sequence required for root completion barrier')
                    return
                if barrier_mode == 'settlement_barrier' or schedule_barrier or root_barrier:
                    if not peer.control_token or (barrier_mode == 'settlement_barrier' and mode != 'text'):
                        self.send_error(400, 'standalone controlled text peer required')
                        return
                    marker = instruction['marker']
                    with peer.lock:
                        if marker in peer.barriers:
                            self.send_error(409, 'duplicate barrier marker')
                            return
                        state = {'event': threading.Event(), 'waiting': True, 'released': False, 'terminal_usage_sent': False}
                        peer.barriers[marker] = state
                    self.send_response(200)
                    self.send_header('Content-Type', 'text/event-stream')
                    self.send_header('Connection', 'close')
                    self.end_headers()
                    self.wfile.write(frame(delta).encode())
                    self.wfile.flush()
                    peer.record({'event': 'barrier_waiting', 'marker': marker,
                                 'wire_mode': barrier_mode, 'completed_tool_results': completed})
                    if not state['event'].wait(1800):
                        peer.record({'event': 'barrier_timeout', 'marker': marker})
                        self.close_connection = True
                        return
                    terminal = frame({}, finish) + 'data: ' + json.dumps({
                        'id': 'dot-usage', 'object': 'chat.completion.chunk', 'created': 1, 'model': 'gpt-4o-mini',
                        'choices': [], 'usage': {'prompt_tokens': 2, 'completion_tokens': 3, 'total_tokens': 5}}) + '\n\n'
                    terminal += 'data: [DONE]\n\n'
                    self.wfile.write(terminal.encode())
                    self.wfile.flush()
                    with peer.lock:
                        state['terminal_usage_sent'] = True
                    peer.record({'event': 'terminal_usage_sent', 'marker': marker, 'tokens': 5,
                                 'claim_scope': 'upstream wire emitted; runtime settlement is independent'})
                    self.close_connection = True
                    return
                frames = frame(delta) + frame({}, finish)
                frames += 'data: ' + json.dumps({'id': 'dot-usage', 'object': 'chat.completion.chunk',
                    'created': 1, 'model': 'gpt-4o-mini', 'choices': [],
                    'usage': {'prompt_tokens': 2, 'completion_tokens': 3, 'total_tokens': 5}}) + '\n\n'
                if instruction.get('wire_mode') != 'missing_terminal':
                    frames += 'data: [DONE]\n\n'
                else:
                    frames = frame(delta)
                payload = frames.encode()
                self.send_response(200)
                self.send_header('Content-Type', 'text/event-stream')
                self.send_header('Content-Length', str(len(payload)))
                self.end_headers()
                if instruction.get('wire_mode') == 'fragmented':
                    for byte in payload:
                        self.wfile.write(bytes([byte]))
                        self.wfile.flush()
                else:
                    self.wfile.write(payload)

        self.server = http.server.ThreadingHTTPServer((bind_host, port), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)

    def record(self, value):
        with self.lock, open(self.trace, 'a') as stream:
            stream.write(json.dumps(value) + '\n')
            stream.flush()
            os.fsync(stream.fileno())

    def rows(self):
        if not self.trace.exists():
            return []
        with self.lock:
            return [json.loads(line) for line in self.trace.read_text().splitlines()]

    def barrier_state(self, marker):
        request = urllib.request.Request('http://127.0.0.1:' + str(self.server.server_port) +
            '/__dot/state?marker=' + urllib.parse.quote(marker, safe=''),
            headers={'Authorization': 'Bearer ' + self.control_token})
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return json.load(response)['state']
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return None
            raise

    def release(self, marker):
        request = urllib.request.Request('http://127.0.0.1:' + str(self.server.server_port) + '/__dot/release',
            data=json.dumps({'marker': marker}).encode(), method='POST',
            headers={'Authorization': 'Bearer ' + self.control_token, 'Content-Type': 'application/json'})
        with urllib.request.urlopen(request, timeout=5) as response:
            return json.load(response)

    def start(self):
        self.thread.start()

    def stop(self):
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()


def main():
    import argparse
    from acceptance import boundary
    parser = argparse.ArgumentParser(description='Independent controlled upstream for staged real persistence faults')
    parser.add_argument('--inventory', required=True)
    parser.add_argument('--port', required=True, type=int)
    parser.add_argument('--trace', required=True)
    parser.add_argument('--control-token-env', required=True)
    args = parser.parse_args()
    boundary(args.inventory)
    os.umask(0o077)
    token = os.environ[args.control_token_env]
    if not token:
        raise RuntimeError('empty control token')
    trace = Path(args.trace)
    with open(trace, 'x'):
        pass
    peer = Provider(args.port, trace, token)
    try:
        peer.server.serve_forever()
    finally:
        peer.server.server_close()


if __name__ == '__main__':
    main()


def provider_for(spec, evidence, label='provider-requests'):
    if any(host.get('lifecycle') == 'kubernetes' for host in spec.get('hosts', {}).values()):
        from kubernetes import KubernetesProvider
        return KubernetesProvider(spec, evidence, label)
    return Provider(spec['provider_port'], evidence.directory / (label + '.jsonl'), uuid.uuid4().hex)
