"""Deferred installed-runtime A2A v1 streaming acceptance. Authoring is not a run.

Usage: --transport jsonrpc|grpc --mode send|subscribe --receipt-path PATH
Environment: UAR_A2A_BASE_URL, UAR_A2A_TOKEN, UAR_A2A_INPUT,
UAR_A2A_GRPC_TARGET, UAR_A2A_AGENT_ID (default compiler-agent),
UAR_A2A_TENANT_ID (optional), UAR_A2A_GRPCURL (default grpcurl),
UAR_A2A_GRPCURL_ARGS (optional). Subscribe mode additionally requires
UAR_A2A_STREAM_TASK_RECEIPT pointing to an exact {task_id, context_id}
receipt for an already active task. The external fixture must hold that task's
controlled provider until this subscription is established, then release it.
The external owner must start/stop the installed artifact and retain provenance.
No client status-poll or autonomous reconnect loop is used here.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess
import urllib.parse
import urllib.request
import uuid

parser = argparse.ArgumentParser()
parser.add_argument("--transport", choices=("jsonrpc", "grpc"), required=True)
parser.add_argument("--mode", choices=("send", "subscribe"), required=True)
parser.add_argument("--receipt-path", required=True)
args = parser.parse_args()
base = os.environ["UAR_A2A_BASE_URL"].rstrip("/")
token = os.environ["UAR_A2A_TOKEN"]
agent = os.environ.get("UAR_A2A_AGENT_ID", "compiler-agent")
endpoint = base + ("/a2a/v1/compiler" if agent == "compiler-agent" else "/a2a/v1/agents/" + urllib.parse.quote(agent, safe=""))
headers = {"Authorization": "Bearer " + token, "A2A-Version": "1.0", "Content-Type": "application/json"}
request_id = str(uuid.uuid4())
method = "SendStreamingMessage" if args.mode == "send" else "SubscribeToTask"
expected = None
if args.mode == "send":
    params = {"message": {"messageId": str(uuid.uuid4()), "role": "ROLE_USER", "parts": [{"text": os.environ["UAR_A2A_INPUT"]}]}}
else:
    expected = json.loads(Path(os.environ["UAR_A2A_STREAM_TASK_RECEIPT"]).read_text())
    assert expected["task_id"] and expected["context_id"]
    params = {"id": expected["task_id"]}
if os.environ.get("UAR_A2A_TENANT_ID"):
    params["tenant"] = os.environ["UAR_A2A_TENANT_ID"]

with urllib.request.urlopen(urllib.request.Request(base + "/.well-known/agent-card.json", headers=headers)) as response:
    card = json.load(response)
assert card["capabilities"]["streaming"] is True, card
assert any(interface["protocolBinding"] == ("JSONRPC" if args.transport == "jsonrpc" else "GRPC")
           and interface["protocolVersion"] == "1.0" for interface in card["supportedInterfaces"])

payloads = []
if args.transport == "jsonrpc":
    headers["Accept"] = "text/event-stream"
    request = urllib.request.Request(endpoint, data=json.dumps({"jsonrpc": "2.0", "id": request_id,
        "method": method, "params": params}).encode(), headers=headers)
    with urllib.request.urlopen(request) as response:
        assert response.headers.get_content_type() == "text/event-stream"
        assert response.headers.get("a2a-version") == "1.0"
        data = []
        for line in response:
            line = line.decode("utf-8").rstrip("\r\n")
            if not line:
                if data:
                    envelope = json.loads("\n".join(data))
                    assert envelope["jsonrpc"] == "2.0" and envelope["id"] == request_id, envelope
                    assert "error" not in envelope, envelope
                    payloads.append(envelope["result"])
                    data = []
            elif line.startswith("data:"):
                data.append(line[5:].removeprefix(" "))
        assert not data, "truncated SSE event at EOF"
else:
    command = [os.environ.get("UAR_A2A_GRPCURL", "grpcurl")]
    command.extend(os.environ.get("UAR_A2A_GRPCURL_ARGS", "").split())
    command.extend(["-H", "authorization: Bearer " + token, "-H", "x-uar-agent-id: " + agent,
                    "-import-path", "proto", "-proto", "proto/a2a_v1.proto", "-d", json.dumps(params),
                    os.environ["UAR_A2A_GRPC_TARGET"], "lf.a2a.v1.A2AService/" + method])
    completed = subprocess.run(command, check=False, capture_output=True, text=True)
    assert completed.returncode == 0, {"stdout": completed.stdout, "stderr": completed.stderr}
    decoder = json.JSONDecoder()
    pending = completed.stdout.lstrip()
    while pending:
        payload, used = decoder.raw_decode(pending)
        payloads.append(payload)
        pending = pending[used:].lstrip()

assert payloads and set(payloads[0]) == {"task"}, payloads
initial = payloads[0]["task"]
task_id = initial["id"]
context_id = initial["contextId"]
assert task_id and context_id, initial
if expected:
    assert task_id == expected["task_id"] and context_id == expected["context_id"], initial
state = initial["status"]["state"]
for payload in payloads[1:]:
    assert len(payload) == 1 and set(payload) <= {"statusUpdate", "artifactUpdate"}, payload
    update = next(iter(payload.values()))
    assert update["taskId"] == task_id and update["contextId"] == context_id, update
    if "statusUpdate" in payload:
        state = update["status"]["state"]
# Interrupted or unknown state and bare EOF do not earn terminal acceptance.
assert state in {"TASK_STATE_COMPLETED", "TASK_STATE_FAILED", "TASK_STATE_CANCELED", "TASK_STATE_REJECTED"}, state
Path(args.receipt_path).write_text(json.dumps({"transport": args.transport, "mode": args.mode,
    "task_id": task_id, "context_id": context_id, "terminal_state": state,
    "events": payloads, "status": "passed"}, indent=2) + "\n")
print(json.dumps({"transport": args.transport, "mode": args.mode, "task_id": task_id,
                  "terminal_state": state, "status": "passed"}))
