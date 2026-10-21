"""Installed-runtime A2A v1 approval fixture; do not run before phase acceptance.

The caller targets an installed UAR artifact with a deterministic provider that
requests one non-preapproved MCP tool for UAR_A2A_APPROVAL_INPUT. The MCP fixture
records actual calls in the local Dot19 peer trace. Run this fixture in fresh
processes for each tuple:

  transport={jsonrpc,grpc} × governance={enforced,disabled} × decision={approve,deny}

Environment:
  UAR_A2A_TRANSPORT             jsonrpc or grpc
  UAR_A2A_GOVERNANCE            enforced or disabled (recorded in the receipt)
  UAR_A2A_BASE_URL              HTTP base URL for JSONRPC and agent card
  UAR_A2A_GRPC_TARGET           host:port for gRPC
  UAR_A2A_TOKEN                 verified credential holding required roles
  UAR_A2A_ROLE_LIMITED_TOKEN    verified credential missing the protected role
  UAR_A2A_TENANT_ID             verified tenant claim shared by both credentials
  UAR_A2A_AGENT_ID              configured agent with the protected explicit skill
  UAR_A2A_MCP_TRACE_PATH        local `dot19_mcp_peer.py` JSONL trace path
  UAR_A2A_APPROVAL_INPUT        deterministic prompt that requests the fixture tool
  UAR_A2A_DYNAMIC_AGENT_ID      agent eligible to activate a protected non-preferred skill
  UAR_A2A_DYNAMIC_SKILL_ID      protected non-preferred skill exposed to `activate_skill`
  UAR_A2A_PROVIDER_TRACE_PATH   local `scripts/dot/provider.py` JSONL trace path
  UAR_A2A_PROTECTED_PROMPT_MARKER unique text present only in the protected skill prompt
  UAR_A2A_GRPCURL               grpcurl executable (grpc mode only; default grpcurl)
  UAR_A2A_GRPCURL_ARGS          optional whitespace-separated transport arguments

The server/tool fixture is responsible for selecting governance posture and
ensuring its MCP tool has no `preapproved_tools` entry. This script intentionally
does not start a development binary or synthetic adapter.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import urllib.error
import urllib.request
import uuid


parser = argparse.ArgumentParser()
parser.add_argument("--transport", choices=("jsonrpc", "grpc"), required=True)
parser.add_argument("--governance", choices=("enforced", "disabled"), required=True)
parser.add_argument("--decision", choices=("approve", "deny"), required=True)
parser.add_argument("--receipt-path", required=True)
args = parser.parse_args()

transport = args.transport
decision = args.decision
receipt_path = args.receipt_path

base_url = os.environ["UAR_A2A_BASE_URL"].rstrip("/")
token = os.environ["UAR_A2A_TOKEN"]
role_limited_token = os.environ["UAR_A2A_ROLE_LIMITED_TOKEN"]
trace_path = Path(os.environ["UAR_A2A_MCP_TRACE_PATH"])
input_text = os.environ["UAR_A2A_APPROVAL_INPUT"]
dynamic_skill_id = os.environ["UAR_A2A_DYNAMIC_SKILL_ID"]
dynamic_agent_id = os.environ["UAR_A2A_DYNAMIC_AGENT_ID"]
provider_trace_path = Path(os.environ["UAR_A2A_PROVIDER_TRACE_PATH"])
protected_prompt_marker = os.environ["UAR_A2A_PROTECTED_PROMPT_MARKER"]
governance = os.environ["UAR_A2A_GOVERNANCE"]
assert governance == args.governance, "fixture governance argument must match target configuration"
assert os.environ["UAR_A2A_TRANSPORT"] == transport, "fixture transport argument must match target configuration"
agent_id = os.environ["UAR_A2A_AGENT_ID"]
tenant_id = os.environ["UAR_A2A_TENANT_ID"]
assert tenant_id, "fixture requires a nonempty verified tenant claim"

TASK_STATES = {
    "TASK_STATE_UNSPECIFIED",
    "TASK_STATE_SUBMITTED",
    "TASK_STATE_WORKING",
    "TASK_STATE_COMPLETED",
    "TASK_STATE_FAILED",
    "TASK_STATE_CANCELED",
    "TASK_STATE_INPUT_REQUIRED",
    "TASK_STATE_REJECTED",
    "TASK_STATE_AUTH_REQUIRED",
}
MESSAGE_ROLES = {"ROLE_USER", "ROLE_AGENT"}
PART_CONTENT_FIELDS = {"text", "raw", "url", "data"}


def peer_tool_calls():
    if not trace_path.exists():
        return 0
    rows = [json.loads(line) for line in trace_path.read_text().splitlines() if line]
    return sum(row.get("event") == "tool_call" for row in rows)


def provider_rows():
    assert provider_trace_path.exists(), "controlled provider trace was not created"
    return [json.loads(line) for line in provider_trace_path.read_text().splitlines() if line]


def http_request(payload, credential=token, target_agent=agent_id):
    # Keep the conformance client's probes below the shipped HTTP rate limit.
    time.sleep(0.2)
    endpoint = base_url + "/a2a/v1/agents/" + target_agent
    headers = {
        "content-type": "application/json",
        "a2a-version": "1.0",
    }
    if credential is not None:
        headers["authorization"] = "Bearer " + credential
    request = urllib.request.Request(
        endpoint,
        data=json.dumps(payload).encode(),
        headers=headers,
    )
    try:
        with urllib.request.urlopen(request, timeout=120) as response:
            body = response.read()
            return response.status, json.loads(body) if body else None, response.headers.get("a2a-version") == "1.0"
    except urllib.error.HTTPError as error:
        body = error.read()
        return error.code, json.loads(body) if body else None, error.headers.get("a2a-version") == "1.0"


def jsonrpc(method, params, credential=token, allow_error=False, target_agent=agent_id):
    params = dict(params)
    params.setdefault("tenant", tenant_id)
    request_id = str(uuid.uuid4())
    status, payload, is_a2a_response = http_request(
        {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
        , credential, target_agent
    )
    if is_a2a_response:
        assert isinstance(payload, dict), payload
        assert payload.get("jsonrpc") == "2.0" and payload.get("id") == request_id, payload
    if allow_error:
        return status, payload, is_a2a_response
    assert is_a2a_response and status == 200 and "error" not in payload, {
        "status": status, "a2a_response": is_a2a_response, "body": payload,
    }
    return payload["result"]


def grpc(method, payload, credential=token, allow_error=False, target_agent=agent_id):
    payload = dict(payload)
    payload.setdefault("tenant", tenant_id)
    command = [os.environ.get("UAR_A2A_GRPCURL", "grpcurl")]
    command.extend(os.environ.get("UAR_A2A_GRPCURL_ARGS", "").split())
    if credential is not None:
        command.extend(["-H", "authorization: Bearer " + credential])
    command.extend(
        [
            "-H",
            "x-uar-agent-id: " + target_agent,
            "-import-path",
            "proto",
            "-proto",
            "proto/a2a_v1.proto",
            "-d",
            json.dumps(payload),
            os.environ["UAR_A2A_GRPC_TARGET"],
            "lf.a2a.v1.A2AService/" + method,
        ]
    )
    completed = subprocess.run(command, check=False, capture_output=True, text=True)
    if allow_error:
        return completed.returncode, completed.stdout, completed.stderr
    assert completed.returncode == 0, {"command": command, "stdout": completed.stdout, "stderr": completed.stderr}
    return json.loads(completed.stdout)


def send_params(message_id, text):
    return {
        "message": {
            "messageId": message_id,
            "role": "ROLE_USER",
            "parts": [{"text": text}],
        },
        "configuration": {"returnImmediately": True},
    }


def task_state(task, wire=transport):
    status = task.get("status")
    assert isinstance(status, dict), task
    if wire == "jsonrpc":
        assert "state" in status, task
        state = status["state"]
    else:
        # TASK_STATE_UNSPECIFIED is the implicit enum zero value and may be
        # omitted by ProtoJSON.
        state = status.get("state", "TASK_STATE_UNSPECIFIED")
    assert state in TASK_STATES, task
    return state


def part_projection(part):
    assert isinstance(part, dict), part
    content = PART_CONTENT_FIELDS & part.keys()
    assert len(content) == 1, part
    normalized = dict(part)
    if "text" in normalized and normalized.get("mediaType") == "text/plain":
        normalized.pop("mediaType")
    return normalized


def message_projection(message, wire):
    assert isinstance(message, dict), message
    assert isinstance(message.get("messageId"), str) and message["messageId"], message
    assert message.get("role") in MESSAGE_ROLES, message
    parts = message.get("parts", [])
    assert isinstance(parts, list) and parts, message
    metadata = message.get("metadata", {})
    assert isinstance(metadata, dict), message
    for field in ("taskId", "contextId"):
        assert message.get(field) is None or isinstance(message[field], str), message
    return {
        "messageId": message["messageId"],
        "taskId": message.get("taskId", ""),
        "contextId": message.get("contextId") or "",
        "role": message["role"],
        "parts": [part_projection(part) for part in parts],
        "metadata": metadata,
    }


def artifact_projection(artifact, wire):
    assert isinstance(artifact, dict), artifact
    assert isinstance(artifact.get("artifactId"), str) and artifact["artifactId"], artifact
    parts = artifact.get("parts", [])
    assert isinstance(parts, list) and parts, artifact
    metadata = artifact.get("metadata", {})
    assert isinstance(metadata, dict), artifact
    return {
        "artifactId": artifact["artifactId"],
        "name": artifact.get("name", ""),
        "description": artifact.get("description", ""),
        "parts": [part_projection(part) for part in parts],
        "metadata": metadata,
    }


def task_projection(task, wire=transport):
    assert isinstance(task, dict), task
    assert isinstance(task.get("id"), str) and task["id"], task
    assert task.get("contextId") is None or isinstance(task["contextId"], str), task
    metadata = task.get("metadata", {})
    assert isinstance(metadata, dict), task
    history = task.get("history", [])
    assert isinstance(history, list), task
    artifacts = task.get("artifacts", [])
    assert isinstance(artifacts, list), task
    status = task["status"]
    assert isinstance(status, dict), task
    status_message = status.get("message")
    if status_message is not None:
        status_message = message_projection(status_message, wire)
    return {
        "id": task["id"],
        "contextId": task.get("contextId") or "",
        "state": task_state(task, wire),
        "statusMessage": status_message,
        "history": [message_projection(message, wire) for message in history],
        "artifacts": [artifact_projection(artifact, wire) for artifact in artifacts],
        "metadata": metadata,
    }


def send_response_task(response, wire=transport):
    assert isinstance(response, dict) and set(response) == {"task"}, response
    task_projection(response["task"], wire)
    return response["task"]


def submit(text, credential=token, target_agent=agent_id):
    message_id = "approval-" + str(uuid.uuid4())
    if transport == "jsonrpc":
        result = jsonrpc("SendMessage", send_params(message_id, text), credential, target_agent=target_agent)
        return send_response_task(result)
    params = send_params(message_id, text)
    params["message"]["parts"][0]["mediaType"] = "text/plain"
    response = grpc(
        "SendMessage",
        params,
        credential,
        target_agent=target_agent,
    )
    return send_response_task(response)


def get_task(task_id, target_agent=agent_id, credential=token):
    if transport == "jsonrpc":
        task = jsonrpc("GetTask", {"id": task_id}, credential, target_agent=target_agent)
    else:
        task = grpc("GetTask", {"id": task_id}, credential, target_agent=target_agent)
    task_projection(task)
    return task


def task_metadata(task):
    # Metadata is optional on the pinned Task/Message/Artifact contracts in
    # both bindings. Approval assertions below still require their records.
    metadata = task.get("metadata", {})
    assert isinstance(metadata, dict), task
    return metadata


def pending_approval(task, expected_tool=None):
    approvals = task_metadata(task).get("uar.approvals", [])
    assert isinstance(approvals, list), task
    pending = [approval for approval in approvals if approval.get("state") == "pending"]
    assert len(pending) == 1, task
    approval = pending[0]
    challenge_id = approval.get("challengeId")
    assert isinstance(challenge_id, str) and challenge_id, approval
    if expected_tool is not None:
        assert approval.get("toolName") == expected_tool, approval
    return approval


def assert_approval_decision(task, challenge_id, approved):
    approvals = task_metadata(task).get("uar.approvals", [])
    assert isinstance(approvals, list), task
    matched = [approval for approval in approvals if approval.get("challengeId") == challenge_id]
    assert len(matched) == 1, task
    approval = matched[0]
    assert approval.get("state") == ("approved" if approved else "denied"), approval
    decision = approval.get("decision")
    assert isinstance(decision, dict) and decision.get("approved") is approved, approval


def decide(task, approved, credential=token, target_agent=agent_id, expected_tool=None):
    pending = pending_approval(task, expected_tool)
    message = {
        "messageId": "decision-" + str(uuid.uuid4()),
        "taskId": task["id"],
        "contextId": task.get("contextId", ""),
        "role": "ROLE_USER",
        "parts": [{"text": "approve" if approved else "deny"}],
    }
    metadata = {"uar.approval": {"id": pending["challengeId"], "approved": approved}}
    if transport == "jsonrpc":
        response = jsonrpc(
            "SendMessage",
            {"message": message, "metadata": metadata, "configuration": {"returnImmediately": True}},
            credential,
            target_agent=target_agent,
        )
    else:
        response = grpc(
            "SendMessage",
            {"message": message, "metadata": metadata, "configuration": {"returnImmediately": True}},
            credential,
            target_agent=target_agent,
        )
    return send_response_task(response), pending["challengeId"]


def list_tasks(credential):
    page_token = ""
    expected_total = None
    tasks = []
    while True:
        params = {"pageSize": 100}
        if page_token:
            params["pageToken"] = page_token
        if transport == "jsonrpc":
            page = jsonrpc("ListTasks", params, credential)
            assert {"tasks", "nextPageToken", "pageSize", "totalSize"} <= page.keys(), page
            total = page["totalSize"]
            page_tasks = page["tasks"]
            next_token = page["nextPageToken"]
        else:
            page = grpc("ListTasks", params, credential)
            # ProtoJSON omits default-valued implicit scalars, empty repeated
            # fields, and empty strings. The server still reports pageSize=100.
            assert page.get("pageSize") == 100, page
            total = page.get("totalSize", 0)
            page_tasks = page.get("tasks", [])
            next_token = page.get("nextPageToken", "")
        assert isinstance(total, int) and isinstance(page_tasks, list), page
        assert isinstance(next_token, str), page
        for item in page_tasks:
            task_projection(item, transport)
        if expected_total is None:
            expected_total = total
        assert total == expected_total, page
        tasks.extend(task["id"] for task in page_tasks)
        if not next_token:
            break
        assert next_token != page_token, page
        page_token = next_token
    assert len(tasks) == expected_total, {"tasks": tasks, "total": expected_total}
    return tasks


def assert_role_denied_without_reservation():
    trusted_before = list_tasks(token)
    limited_before = list_tasks(role_limited_token)
    assert limited_before == trusted_before, "fixture tokens must address the same A2A task principal"
    before_calls = peer_tool_calls()
    if transport == "jsonrpc":
        status, body, is_a2a_response = jsonrpc(
            "SendMessage", send_params("role-denied-" + str(uuid.uuid4()), input_text), role_limited_token, True
        )
        assert is_a2a_response and status == 403 and body["error"]["code"] == -32011, body
    else:
        params = send_params("role-denied-" + str(uuid.uuid4()), input_text)
        params["message"]["parts"][0]["mediaType"] = "text/plain"
        returncode, stdout, stderr = grpc("SendMessage", params, role_limited_token, True)
        assert returncode != 0 and "PermissionDenied" in (stdout + stderr), (stdout, stderr)
    assert list_tasks(role_limited_token) == limited_before, "role denial reserved a durable task"
    assert peer_tool_calls() == before_calls, "role denial reached the controlled MCP peer"


def assert_authentication_required_without_reservation():
    before = list_tasks(token)
    before_calls = peer_tool_calls()
    if transport == "jsonrpc":
        status, body, is_a2a_response = jsonrpc(
            "SendMessage", send_params("auth-required-" + str(uuid.uuid4()), input_text), None, True
        )
        if governance == "enforced":
            assert status == 401 and body is None and not is_a2a_response, body
        else:
            assert is_a2a_response and status == 401 and body["error"]["code"] == -32010, body
    else:
        params = send_params("auth-required-" + str(uuid.uuid4()), input_text)
        params["message"]["parts"][0]["mediaType"] = "text/plain"
        returncode, stdout, stderr = grpc("SendMessage", params, None, True)
        assert returncode != 0 and "Unauthenticated" in (stdout + stderr), (stdout, stderr)
    assert list_tasks(token) == before, "authentication failure reserved a durable task"
    assert peer_tool_calls() == before_calls, "authentication failure reached the controlled MCP peer"


def assert_tenant_mismatch_without_reservation():
    before = list_tasks(token)
    before_calls = peer_tool_calls()
    mismatched_tenant = tenant_id + "-mismatch"
    params = send_params("tenant-denied-" + str(uuid.uuid4()), input_text)
    params["tenant"] = mismatched_tenant
    if transport == "jsonrpc":
        status, body, is_a2a_response = jsonrpc("SendMessage", params, token, True)
        assert is_a2a_response and status == 403 and body["error"]["code"] == -32011, body
    else:
        params["message"]["parts"][0]["mediaType"] = "text/plain"
        returncode, stdout, stderr = grpc("SendMessage", params, token, True)
        assert returncode != 0 and "PermissionDenied" in (stdout + stderr), (stdout, stderr)
    assert list_tasks(token) == before, "tenant mismatch reserved a durable task"
    assert peer_tool_calls() == before_calls, "tenant mismatch reached the controlled MCP peer"


def assert_grpc_streaming_role_denied_without_reservation():
    before = list_tasks(token)
    before_calls = peer_tool_calls()
    params = send_params("streaming-role-denied-" + str(uuid.uuid4()), input_text)
    params["message"]["parts"][0]["mediaType"] = "text/plain"
    returncode, stdout, stderr = grpc("SendStreamingMessage", params, role_limited_token, True)
    assert returncode != 0 and "PermissionDenied" in (stdout + stderr), (stdout, stderr)
    assert list_tasks(token) == before, "role-denied gRPC streaming reserved a durable task"
    assert peer_tool_calls() == before_calls, "role-denied gRPC streaming reached the controlled MCP peer"


def assert_cross_transport_projection(task_id):
    json_task = task_projection(jsonrpc("GetTask", {"id": task_id}), "jsonrpc")
    grpc_task = task_projection(grpc("GetTask", {"id": task_id}), "grpc")
    assert json_task == grpc_task, {"jsonrpc": json_task, "grpc": grpc_task}
    json_recent = task_projection(jsonrpc("GetTask", {"id": task_id, "historyLength": 1}), "jsonrpc")
    grpc_recent = task_projection(grpc("GetTask", {"id": task_id, "historyLength": 1}), "grpc")
    assert json_recent == grpc_recent, {"jsonrpc": json_recent, "grpc": grpc_recent}
    assert json_recent["history"] == json_task["history"][-1:], {
        "complete_history": json_task["history"],
        "recent_history": json_recent["history"],
    }


def wait_for_task(task_id, expected_states, target_agent=agent_id, credential=token):
    deadline = time.monotonic() + 120
    while time.monotonic() < deadline:
        task = get_task(task_id, target_agent, credential)
        if task_state(task) in expected_states:
            return task
        time.sleep(0.1)
    raise AssertionError({"task_id": task_id, "expected": expected_states,
                          "last": get_task(task_id, target_agent, credential)})


assert peer_tool_calls() == 0, "fixture process must begin with no MCP tool invocation"
assert_role_denied_without_reservation()
assert_authentication_required_without_reservation()
assert_tenant_mismatch_without_reservation()
assert_grpc_streaming_role_denied_without_reservation()
task = submit(input_text)
task = get_task(task["id"])
immediate_get_state = task_state(task)
task = wait_for_task(task["id"], {"TASK_STATE_INPUT_REQUIRED"})
assert task_state(task) == "TASK_STATE_INPUT_REQUIRED", task
assert peer_tool_calls() == 0, "required MCP tool executed before host approval"

result, approval_id = decide(task, decision == "approve")
if decision == "approve":
    terminal = wait_for_task(result["id"], {"TASK_STATE_COMPLETED"})
    assert peer_tool_calls() == 1
else:
    terminal = wait_for_task(result["id"], {"TASK_STATE_REJECTED"})
    assert peer_tool_calls() == 0, "denied required MCP tool executed"
assert_approval_decision(terminal, approval_id, decision == "approve")
assert_cross_transport_projection(result["id"])

dynamic_input = json.dumps(
    {
        "dot_action": "tool_sequence",
        "marker": "a2a-dynamic-protected-skill",
        "steps": [
            {"tool": "activate_skill", "arguments": {"skill_id": dynamic_skill_id}},
        ],
    }
)
before_dynamic_calls = peer_tool_calls()
dynamic_task = submit(dynamic_input, role_limited_token, dynamic_agent_id)
dynamic_result = wait_for_task(
    dynamic_task["id"],
    {"TASK_STATE_INPUT_REQUIRED"},
    dynamic_agent_id,
    role_limited_token,
)
dynamic_result, dynamic_approval_id = decide(
    dynamic_result,
    True,
    role_limited_token,
    dynamic_agent_id,
    expected_tool="activate_skill",
)
dynamic_terminal = wait_for_task(
    dynamic_result["id"],
    {"TASK_STATE_COMPLETED", "TASK_STATE_FAILED", "TASK_STATE_REJECTED", "TASK_STATE_CANCELED"},
    dynamic_agent_id,
    role_limited_token,
)
assert_approval_decision(dynamic_terminal, dynamic_approval_id, True)
assert peer_tool_calls() == before_dynamic_calls, "protected dynamic skill exposed its MCP effect to a role-limited caller"
dynamic_rows = [
    row for row in provider_rows()
    if row.get("instruction", {}).get("marker") == "a2a-dynamic-protected-skill"
]
assert dynamic_rows, "controlled provider did not receive the dynamic activation prompt"
activation_calls = []
activation_results = []
for row in dynamic_rows[1:]:
    for message in row.get("messages", []):
        if message.get("role") == "assistant":
            for tool_call in message.get("tool_calls", []):
                function = tool_call.get("function", {})
                if function.get("name") != "activate_skill":
                    continue
                arguments = function.get("arguments")
                if isinstance(arguments, str):
                    try:
                        arguments = json.loads(arguments)
                    except ValueError:
                        continue
                activation_calls.append(arguments)
        if message.get("role") != "tool":
            continue
        content = message.get("content")
        if isinstance(content, str):
            try:
                content = json.loads(content)
            except ValueError:
                continue
        if isinstance(content, dict) and content.get("status") == "activation_failed":
            activation_results.append(content)
assert {"skill_id": dynamic_skill_id} in activation_calls, (
    "controlled provider did not call native activate_skill with the protected skill"
)
assert any(
    result.get("failure", {}).get("code") == "ineligible"
    and result["failure"].get("skill_id") == dynamic_skill_id
    for result in activation_results
), "native activate_skill did not report the protected skill as ineligible"
assert all(
    protected_prompt_marker not in json.dumps(row.get("messages", []))
    for row in dynamic_rows[1:]
), "protected skill prompt reached a later upstream provider request"

Path(receipt_path).write_text(
    json.dumps(
        {
            "transport": transport,
            "governance": governance,
            "decision": decision,
            "tenant": tenant_id,
            "task_id": task["id"],
            "immediate_get_state": immediate_get_state,
            "tool_count": peer_tool_calls(),
            "dynamic_task_id": dynamic_task["id"],
            "result": "passed",
        },
        indent=2,
    )
)
