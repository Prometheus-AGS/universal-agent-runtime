"""Real stdio MCP upstream effects/elicitation peer for installed UAR.

Configure with arguments TRACE_FILE. No host runtime authority is implemented.
Elicitation acceptance and successful transport are separate from actual effects.
"""
import importlib.util
import json
import os
import signal
import subprocess
import time
from pathlib import Path
import sys
import uuid

descendant = sys.argv[1] == '--descendant'
trace = sys.argv[2] if descendant else sys.argv[1]


def record(event, **values):
    with open(trace, 'a') as stream:
        stream.write(json.dumps(dict(event=event, pid=os.getpid(), **values)) + '\n')
        stream.flush()
        os.fsync(stream.fileno())


def send(value):
    print(json.dumps(dict(jsonrpc='2.0', **value)), flush=True)


if descendant:
    controller_path = Path(__file__).resolve().parents[2] / 'scripts/supervisor/controller.py'
    if Path(__file__).resolve().parent == Path('/opt/dot17'):
        controller_path = Path('/opt/dot17/controller.py')
    spec = importlib.util.spec_from_file_location('dot17_supervisor_identity', controller_path)
    controller = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(controller)
    kind, machine, boot = controller.native_identity()
    identity = controller.root_identity(os.getpid(), os.getuid() if hasattr(os, 'getuid') else None)
    record('descendant_started', marker=sys.argv[3], process_identity=identity,
           platform=kind, machine_identity=machine, boot_identity=boot)
    while True:
        record('descendant_heartbeat', marker=sys.argv[3], process_identity=identity,
               boot_identity=boot, monotonic_ns=time.monotonic_ns())
        time.sleep(.5)

record('started')
for line in sys.stdin:
    request = json.loads(line)
    if 'id' not in request:
        continue
    request_id = request['id']
    method = request.get('method')
    if method == 'initialize':
        record('initialized', capabilities=request['params'].get('capabilities'))
        result = {'protocolVersion': request['params']['protocolVersion'], 'capabilities': {'tools': {}},
                  'serverInfo': {'name': 'dot17-effects', 'version': '1.0'}}
    elif method == 'tools/list':
        result = {'tools': [{'name': 'effect', 'description': 'Controlled external write',
            'inputSchema': {'type': 'object', 'properties': {'mode': {'type': 'string'},
                'marker': {'type': 'string'}}, 'required': ['mode', 'marker'], 'additionalProperties': False},
            'annotations': {'readOnlyHint': False, 'destructiveHint': True,
                'idempotentHint': False, 'openWorldHint': True}}]}
    elif method == 'tools/call':
        args = request['params']['arguments']
        mode, marker = args['mode'], args['marker']
        record('dispatch', mode=mode, marker=marker)
        if mode.startswith('elicit'):
            interaction = 'dot17-' + uuid.uuid4().hex
            params = {'mode': 'form', 'message': 'Confirm this fixture marker', 'requestedSchema': {
                'type': 'object', 'properties': {'confirmed': {'type': 'boolean'}}, 'required': ['confirmed']}}
            send({'id': interaction, 'method': 'elicitation/create', 'params': params})
            for response_line in sys.stdin:
                response = json.loads(response_line)
                if response.get('id') == interaction:
                    record('elicitation_response', marker=marker, response=response)
                    break
            else:
                record('disconnected', marker=marker)
                sys.exit(2)
            outcome = response.get('result', {})
            if outcome.get('action') != 'accept':
                result = {'content': [{'type': 'text', 'text': outcome.get('action', 'error')}], 'isError': True}
                send({'id': request_id, 'result': result})
                continue
        if mode == 'hold_descendant':
            options = {'start_new_session': True} if os.name != 'nt' else {
                'creationflags': subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP}
            child = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--descendant', trace, marker],
                stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, **options)
            record('descendant_dispatched', marker=marker, child_pid=child.pid)
            while True:
                time.sleep(1)
        if mode == 'application_error':
            result = {'content': [{'type': 'text', 'text': 'dot17-original-error-' + marker}], 'isError': True}
        else:
            record('effect', marker=marker, mode=mode)
            if mode == 'unknown':
                os._exit(23)
            if mode == 'hold_after_effect':
                while True:
                    time.sleep(1)
            result = {'content': [{'type': 'text', 'text': 'dot17-effect-' + marker}], 'isError': False}
    else:
        send({'id': request_id, 'error': {'code': -32601, 'message': 'method not found'}})
        continue
    send({'id': request_id, 'result': result})
