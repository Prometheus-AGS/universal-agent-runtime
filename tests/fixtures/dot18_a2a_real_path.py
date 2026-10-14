"""Deferred installed-runtime fixture. Never invoked before full Dot implementation.

Modes: submit (save receipt), read (after externally joined cold restart), cancel,
pending, approve, deny. Decision modes require an exact pending host challenge.
Environment: UAR_A2A_BASE_URL, UAR_A2A_TEST_TOKEN, UAR_A2A_OTHER_TOKEN,
UAR_A2A_AGENT_ID, UAR_A2A_INPUT. Arguments: MODE RECEIPT_PATH.
The caller owns runtime lifecycle and installed artifact provenance.
"""
import json
import os
from pathlib import Path
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid

base = os.environ["UAR_A2A_BASE_URL"].rstrip("/")
discovery_base = os.environ.get("UAR_A2A_DISCOVERY_BASE_URL", base).rstrip("/")
token = os.environ["UAR_A2A_TEST_TOKEN"]
other_token = os.environ["UAR_A2A_OTHER_TOKEN"]
agent = os.environ.get("UAR_A2A_AGENT_ID", "compiler-agent")
endpoint = base + ("/a2a/v1/compiler" if agent == "compiler-agent" else "/a2a/v1/agents/" + urllib.parse.quote(agent, safe=""))


def request(url, payload=None, credential=None, version="1.0"):
    headers = {"content-type": "application/json", "A2A-Version": version}
    if credential:
        headers["Authorization"] = "Bearer " + credential
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(url, data=data, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=120) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        body = error.read()
        try:
            return error.code, json.loads(body)
        except ValueError:
            return error.code, None


def rpc(method, params, credential=token, version="1.0"):
    return request(endpoint, {"jsonrpc": "2.0", "id": str(uuid.uuid4()), "method": method, "params": params}, credential, version)


mode, receipt_path = sys.argv[1:]
status, card = request(base + "/.well-known/agent-card.json")
assert status == 200
interface = card["supportedInterfaces"][0]
assert interface["protocolVersion"] == "1.0" and interface["protocolBinding"] == "JSONRPC"
assert interface["url"] == discovery_base + "/a2a/v1/compiler"
assert isinstance(card["securitySchemes"], dict)
assert card["securityRequirements"]
assert "supported_interfaces" not in card and "security_schemes" not in card
assert request(endpoint, {"jsonrpc": "2.0", "id": 1, "method": "GetTask", "params": {"id": "missing"}})[0] == 401
assert rpc("GetTask", {"id": "missing"}, version="0.3")[1]["error"]["code"] == -32009
assert rpc("GetTask", {"id": "missing"})[1]["error"]["code"] == -32001
assert rpc("SendMessage", {"message": {"messageId": "invalid-role", "role": "user", "parts": [{"text": "hello"}]}})[1]["error"]["code"] == -32602
assert rpc("SendMessage", {"message": {"messageId": "invalid-oneof", "role": "ROLE_USER", "parts": [{"text": "hello", "data": {}}]}})[1]["error"]["code"] == -32602
assert rpc("SendMessage", {"message": {"messageId": "unsupported-file", "role": "ROLE_USER", "parts": [{"url": "https://example.invalid/file"}]}})[1]["error"]["code"] == -32005
assert rpc("SubscribeToTask", {"id": "missing"})[1]["error"]["code"] == -32004

if mode == "submit":
    message_id = str(uuid.uuid4())
    params = {"message": {"messageId": message_id, "role": "ROLE_USER", "parts": [{"text": os.environ["UAR_A2A_INPUT"]}]}, "configuration": {"returnImmediately": True}}
    status, result = rpc("SendMessage", params)
    assert status == 200 and "task" in result["result"], result
    task = result["result"]["task"]
    task_id = task["id"]
    assert task["contextId"] and task["status"]["state"].startswith("TASK_STATE_")
    assert task["history"][0]["messageId"] == message_id
    assert task["history"][0]["role"] == "ROLE_USER"
    assert "type" not in task["history"][0]["parts"][0]
    _, repeated = rpc("SendMessage", params)
    assert repeated["result"]["task"]["id"] == task_id
    Path(receipt_path).write_text(json.dumps({"task_id": task_id, "version": card["version"], "context_id": task["contextId"]}, indent=2))
else:
    receipt = json.loads(Path(receipt_path).read_text())
    task_id = receipt["task_id"]
    status, result = rpc("CancelTask" if mode == "cancel" else "GetTask", {"id": task_id, "historyLength": 0})
    assert status == 200 and "result" in result, result
    task = result["result"]
    assert task["id"] == task_id and task["contextId"] == receipt["context_id"]
    assert task.get("history", []) == []
    if mode == "cancel":
        assert task["status"]["state"] == "TASK_STATE_CANCELED"
        assert not task.get("metadata", {}).get("cleanup_unconfirmed", False)
    if mode in {"pending", "approve", "deny"}:
        assert task["status"]["state"] == "TASK_STATE_INPUT_REQUIRED", task
        pending = [record for record in task["metadata"]["uar.approvals"] if record["state"] == "pending"]
        assert len(pending) == 1, pending
        approval_id = pending[0]["challengeId"]
        if mode != "pending":
            params = {"message": {"messageId": str(uuid.uuid4()), "taskId": task_id,
                "contextId": receipt["context_id"], "role": "ROLE_USER", "parts": [{"text": mode}]},
                "metadata": {"uar.approval": {"id": approval_id, "approved": mode == "approve"}},
                "configuration": {"returnImmediately": True}}
            _, foreign = rpc("SendMessage", params, other_token)
            assert foreign["error"]["code"] == -32001
            _, decided = rpc("SendMessage", params)
            assert decided["result"]["task"]["id"] == task_id, decided
            _, repeated = rpc("SendMessage", params)
            assert repeated["result"]["task"]["id"] == task_id, repeated
            params["metadata"]["uar.approval"]["approved"] = mode != "approve"
            _, opposite = rpc("SendMessage", params)
            assert opposite["error"]["code"] == -32004, opposite

_, denied = rpc("GetTask", {"id": task_id}, other_token)
assert denied["error"]["code"] == -32001
print(json.dumps({"mode": mode, "task_id": task_id, "card_version": card["version"], "result": "passed"}))
