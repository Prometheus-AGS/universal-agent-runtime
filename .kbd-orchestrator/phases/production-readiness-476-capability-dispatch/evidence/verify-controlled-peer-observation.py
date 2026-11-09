"""Supplementary fixture integrity probe; execute only at the full phase gate."""
import base64
import hashlib
import json
from pathlib import Path
import subprocess
import time
import urllib.error
import urllib.request
import uuid

root = Path(__file__).resolve().parents[4]
run = root / 'target/production-readiness-476' / ('peer-observation-' + uuid.uuid4().hex)
run.mkdir(parents=True, exist_ok=False)
peer_source = root / 'scripts/post_lifecycle/controlled_peer.py'
port_file = run / 'port'
peer_log = run / 'peer.jsonl'
result = {'kind': 'supplementary-owned-peer-integrity', 'status': 'FAIL',
          'product_acceptance': False, 'source_sha256': hashlib.sha256(peer_source.read_bytes()).hexdigest()}
process = None
try:
    with (run / 'process.log').open('wb') as output:
        process = subprocess.Popen(['python3', str(peer_source), '--port', '0', '--port-file', str(port_file),
                                    '--receipt-file', str(peer_log), '--dimensions', '384'],
                                   cwd=root, stdout=output, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 10
        while not port_file.exists():
            if process.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError('Owned peer failed to publish its port')
            time.sleep(0.05)
        base = 'http://127.0.0.1:' + port_file.read_text().strip()

        def request(path, payload=None, raw=None):
            data = raw if raw is not None else (json.dumps(payload).encode() if payload is not None else None)
            req = urllib.request.Request(base + path, data=data,
                                         headers={'Content-Type': 'application/json'} if data is not None else {})
            try:
                with urllib.request.urlopen(req, timeout=10) as response:
                    return response.status, json.loads(response.read())
            except urllib.error.HTTPError as error:
                return error.code, json.loads(error.read())

        for mode, status in [('success', 200), ('provider_failure', 503)]:
            code, _ = request('/__uar_fixture/control', {'action': 'configure_chat', 'case': 'observation_probe', 'mode': mode})
            assert code == 200, 'Owned control must succeed'
            code, _ = request('/v1/chat/completions', {'model': 'fixture', 'tools': None, 'stream': False,
                'messages': [{'role': 'user', 'content': 'UAR_CASE:observation_probe text'}]})
            assert code == status, 'Explicit null tools must preserve configured success/failure behavior'
        request('/__uar_fixture/control', {'action': 'configure_chat', 'case': 'observation_probe', 'mode': 'success'})
        asset_root = root / 'scripts/behaviors/fixtures/capability-dispatch'
        png = (asset_root / 'synthetic.png').read_bytes()
        jpeg = (asset_root / 'synthetic.jpeg').read_bytes()
        parts = [{'type': 'text', 'text': 'UAR_CASE:observation_probe first'},
                 {'type': 'image_url', 'image_url': {'url': 'data:image/png;base64,' + base64.b64encode(png).decode(), 'detail': 'low'}},
                 {'type': 'text', 'text': 'middle'},
                 {'type': 'image_url', 'image_url': {'url': 'data:image/jpeg;base64,' + base64.b64encode(jpeg).decode(), 'detail': 'high'}}]
        code, _ = request('/v1/chat/completions', {'model': 'fixture', 'tools': None, 'stream': False,
                                                'max_completion_tokens': 32, 'messages': [{'role': 'user', 'content': parts}]})
        assert code == 200, 'Owned mixed control must succeed'
        code, status = request('/__uar_fixture/status')
        assert code == 200
        assert status['counts']['chat_arrivals'] == status['observation_count'] == 3
        assert all(row['tool_count'] == 0 for row in status['observations'])
        observed = status['observations'][-1]
        actual_parts = observed['messages'][-1]['parts']
        assert [part['type'] for part in actual_parts] == ['text', 'image_url', 'text', 'image_url']
        assert [part['sha256'] for part in actual_parts if part['type'] == 'image_url'] == [hashlib.sha256(png).hexdigest(), hashlib.sha256(jpeg).hexdigest()]
        assert observed['max_completion_tokens'] == 32
        assert 'data:image/' not in json.dumps(status), 'Observation must remain body-free'
        code, _ = request('/v1/chat/completions', raw=b'x' * (1024 * 1024 + 1))
        assert code == 400, 'Existing peer envelope must refuse oversized body'
        _, after = request('/__uar_fixture/status')
        assert after['counts']['chat_arrivals'] == 4 and after['observation_count'] == 3, 'Oversized send cannot masquerade as zero arrival'
        result.update(status='PASS', assertions=['null-tools-success-200', 'null-tools-provider-failure-503',
                      'ordered-image-byte-hashes', 'actual-output-scalar', 'body-free-observation', 'oversized-arrival-before-parse'])
finally:
    if process is not None:
        if process.poll() is None:
            process.terminate()
        try:
            exit_code = process.wait(timeout=10)
            result['owned_peer_exit'] = exit_code
            if exit_code != 0:
                result['status'] = 'FAIL'
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            result.update(status='FAIL', cleanup='owned-peer-required-kill')
    result['receipt'] = str(run / 'result.json')
    (run / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
if result['status'] != 'PASS':
    raise SystemExit(1)
