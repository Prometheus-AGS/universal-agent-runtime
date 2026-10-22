#!/usr/bin/env python3
"""HTTP-only production caller-tool acceptance against an already installed UAR.

Candidate lifecycle, real durable database provisioning, agent/provider binding,
credentials, and a safe executable host tool are operator-owned prerequisites.
The separately running client_tools_provider.py is a real provider HTTP peer.
No import performs I/O. This program never builds or starts a candidate.
"""
import argparse
import copy
import concurrent.futures
import http.client as http_client
import threading
import json
import os
from pathlib import Path
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

EXTENSION = "https://prometheus.dev/a2a/extensions/client-tools/v1"
TRANSPORT = threading.local()
EVIDENCE = []


class CapturedRequest(Exception):
    def __init__(self, url, body, token, headers):
        self.request = (url, body, token, headers)


PROTOCOLS = ("openai", "anthropic", "agui", "a2a", "acp")
SCHEMA = {"type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"], "additionalProperties": False}
TOOLS = [{"type": "function", "function": {"name": name, "description": "Acceptance caller-owned lookup", "parameters": SCHEMA, "strict": True}} for name in ("test_lookup", "test_second")]


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def read_http_body(response, deadline):
    if deadline is None:
        return response.read().decode()
    chunks = []
    while True:
        require(time.monotonic() < deadline, "graph HTTP response did not reach EOF before deadline")
        part = response.read1(65536)
        if not part:
            break
        chunks.append(part)
    return b"".join(chunks).decode()


def http(url, body=None, token=None, headers=None, method=None, timeout=180, deadline=None):
    if getattr(TRANSPORT, "capture", False):
        raise CapturedRequest(url, body, token, headers)
    values = {"Content-Type": "application/json", **(headers or {})}
    if token:
        values["Authorization"] = "Bearer " + token
    request = urllib.request.Request(url, data=None if body is None else json.dumps(body).encode(), headers=values, method=method)
    request_deadline = deadline if deadline is not None else getattr(TRANSPORT, "deadline", None)
    if request_deadline is not None:
        require(time.monotonic() < request_deadline, "graph HTTP deadline expired before request")
        timeout = min(timeout, 10, request_deadline - time.monotonic())
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = read_http_body(response, request_deadline)
            status, content_type = response.status, response.headers.get_content_type()
    except urllib.error.HTTPError as error:
        if request_deadline is None:
            raw = error.read().decode()
        else:
            with error:
                raw = read_http_body(error, request_deadline)
        status, content_type = error.code, error.headers.get_content_type()
    if content_type == "text/event-stream":
        events, done = [], False
        for block in raw.replace("\r\n", "\n").split("\n\n"):
            data = "\n".join(line[5:].lstrip() for line in block.splitlines() if line.startswith("data:"))
            if data == "[DONE]":
                require(not done, "duplicate DONE")
                done = True
            elif data:
                require(not done, "event after DONE")
                events.append(json.loads(data))
        observed = {"status": status, "events": events, "done": done}
        EVIDENCE.append({"url": url, "response": observed})
        return observed
    try:
        value = json.loads(raw)
    except ValueError:
        value = {"unparsed": raw}
    observed = {"status": status, "body": value}
    EVIDENCE.append({"url": url, "response": observed})
    return observed


def failed(reply):
    if reply["status"] >= 400:
        return True
    body = reply.get("body", {})
    if body.get("error") or body.get("result", {}).get("status") == "error":
        return True
    state = body.get("result", {}).get("status")
    if isinstance(state, dict) and state.get("state") in ("TASK_STATE_FAILED", "TASK_STATE_CANCELED"):
        return True
    return any(event.get("error") or event.get("type") in ("error", "RUN_ERROR", "Error", "Cancelled") or event.get("type") == "RUN_FINISHED" and event.get("outcome", {}).get("type") == "cancelled" for event in reply.get("events", []))


def a2a_read_diagnostic(reply):
    detail = {"http_status": reply.get("status")}
    body = reply.get("body", {})
    error = body.get("error")
    if isinstance(error, dict):
        detail["error"] = {"code": error.get("code") if type(error.get("code")) is int else "omitted",
            "message_present": "message" in error}
    result = body.get("result", {})
    if isinstance(result, dict):
        task = result.get("task", result)
        if isinstance(task, dict):
            state = task.get("status", {}).get("state")
            if state in ("TASK_STATE_SUBMITTED", "TASK_STATE_WORKING", "TASK_STATE_INPUT_REQUIRED", "TASK_STATE_COMPLETED", "TASK_STATE_FAILED", "TASK_STATE_CANCELED", "TASK_STATE_REJECTED", "TASK_STATE_AUTH_REQUIRED", "TASK_STATE_UNSPECIFIED"):
                detail["task_state"] = state
    return json.dumps(detail, separators=(",", ":"))


class Driver:
    def __init__(self, args, protocol, agent=None):
        self.args, self.protocol, self.agent = args, protocol, agent or args.agent_id
        self.token = os.environ[args.bearer_token_env]
        self.thread, self.run = str(uuid.uuid4()), None
        self.session, self.task, self.context = None, None, None
        marker = "caller-tools-acceptance:" + str(uuid.uuid4())
        self.history = [{"role": "user", "content": marker + ":earlier-user"},
            {"role": "assistant", "content": marker + ":earlier-assistant"},
            {"role": "user", "content": marker + ":current-user"}]
        self.tools = copy.deepcopy(TOOLS)
        self.choice, self.parallel = "auto", True
        self.last_reply = None
        self.acp_rpc_id = None
        self.agui_requests = {}
        self.agui_graph_projections = {}
        self.message_ids = {}
        self.graph_approval_receipts = []
        self.graph_approval_observations = []
        self.graph_selection = None

    def common(self):
        choice = {"type": self.choice} if self.choice != "named" else {"type": "named", "name": "test_lookup"}
        return {"tools": self.tools, "tool_choice": choice, "parallel_tool_calls": self.parallel, "history": self.history}

    def rpc(self, endpoint, method, params, token=None, headers=None, rpc_id=None):
        return http(self.args.base_url.rstrip("/") + endpoint, {"jsonrpc": "2.0", "id": rpc_id or str(uuid.uuid4()), "method": method, "params": params}, token or self.token, headers)

    def send(self, stream=False, token=None):
        token = token or self.token
        if self.protocol == "openai":
            choice = self.choice if self.choice != "named" else {"type": "function", "function": {"name": "test_lookup"}}
            body = {"model": self.args.openai_model or self.agent, "agent_id": self.agent, "messages": self.history,
                "tools": self.tools, "tool_choice": choice, "parallel_tool_calls": self.parallel, "stream": stream}
            reply = http(self.args.base_url.rstrip("/") + "/v1/chat/completions", body, token)
        elif self.protocol == "anthropic":
            messages = []
            for message in self.history:
                if message["role"] == "tool":
                    block = {"type": "tool_result", "tool_use_id": message["tool_call_id"], "content": message["content"]}
                    if messages and messages[-1]["role"] == "user" and isinstance(messages[-1]["content"], list):
                        messages[-1]["content"].append(block)
                    else:
                        messages.append({"role": "user", "content": [block]})
                elif message.get("tool_calls"):
                    blocks = ([{"type": "text", "text": message["content"]}] if message.get("content") else [])
                    blocks += [{"type": "tool_use", "id": call["id"], "name": call["function"]["name"], "input": json.loads(call["function"]["arguments"])} for call in message["tool_calls"]]
                    messages.append({"role": "assistant", "content": blocks})
                else:
                    messages.append(copy.deepcopy(message))
            choice = {"type": {"auto": "auto", "none": "none", "required": "any", "named": "tool"}[self.choice]}
            if self.choice == "named":
                choice["name"] = "test_lookup"
            if self.choice != "none":
                choice["disable_parallel_tool_use"] = not self.parallel
            body = {"model": self.args.anthropic_model, "agent_id": self.agent, "max_tokens": 256, "messages": messages,
                "tools": [{"name": tool["function"]["name"], "description": tool["function"]["description"], "input_schema": tool["function"]["parameters"], "strict": True} for tool in self.tools],
                "tool_choice": choice, "stream": stream}
            reply = http(self.args.base_url.rstrip("/") + "/v1/messages", body, token, {"anthropic-version": "2023-06-01"})
        elif self.protocol == "agui":
            history_key = json.dumps(self.history, sort_keys=True)
            if history_key not in self.agui_requests:
                self.agui_requests[history_key] = [str(uuid.uuid4()), self.run]
            self.run, previous = self.agui_requests[history_key]
            messages = [{"id": self.thread + ":" + str(index), "role": message["role"], "content": message.get("content") or "",
                **({"toolCalls": message["tool_calls"]} if message.get("tool_calls") else {}),
                **({"toolCallId": message["tool_call_id"]} if message.get("tool_call_id") else {})} for index, message in enumerate(self.history)]
            body = {"threadId": self.thread, "runId": self.run, "protocolVersion": "1.0", "messages": messages,
                "tools": [{"name": tool["function"]["name"], "description": tool["function"]["description"], "parameters": tool["function"]["parameters"]} for tool in self.tools],
                "forwardedProps": {"uar": {"client_tools": {"tool_choice": self.common()["tool_choice"], "parallel_tool_calls": self.parallel}}}}
            if previous:
                body["parentRunId"] = previous
            reply = http(self.args.base_url.rstrip("/") + "/api/uar/ag-ui/agents/" + urllib.parse.quote(self.agent, safe=""), body, token, {"Accept": "text/event-stream", **self.args.workspace_headers})
        elif self.protocol == "acp":
            if not self.session:
                created = self.rpc(self.args.acp_path, "sessions/create", {"agent_id": self.agent})
                require(not failed(created), "ACP session creation failed: " + json.dumps(created))
                self.session = created["body"]["result"]["session_id"]
            method = "runs/submit_tool_results" if self.history[-1]["role"] == "tool" else "runs/create"
            self.acp_rpc_id = self.acp_rpc_id or str(uuid.uuid4())
            reply = self.rpc(self.args.acp_path + ("/stream" if stream else ""), method,
                {"session_id": self.session, "input": self.history[0]["content"], "client_tools": self.common()}, token, rpc_id=self.acp_rpc_id)
        else:
            history_key = json.dumps(self.history, sort_keys=True)
            self.message_ids.setdefault(history_key, str(uuid.uuid4()))
            message = {"messageId": self.message_ids[history_key], "role": "ROLE_USER", "parts": [{"text": "caller-tools acceptance"}]}
            if self.task:
                message.update(taskId=self.task, contextId=self.context)
            endpoint = self.args.a2a_path or "/a2a/v1/agents/" + urllib.parse.quote(self.agent, safe="")
            params = {"message": message, "metadata": {**self.args.a2a_metadata, EXTENSION: self.common()}, "configuration": {"returnImmediately": False}}
            reply = self.rpc(endpoint, "SendStreamingMessage" if stream else "SendMessage", params, token, {"A2A-Version": "1.0", **self.args.workspace_headers})
            if not failed(reply):
                if "events" in reply:
                    snapshots = [event.get("result", {}).get("task") for event in reply["events"] if event.get("result", {}).get("task")]
                    require(snapshots, "A2A stream missing initial task snapshot: " + a2a_read_diagnostic(reply))
                    task = snapshots[0]
                else:
                    task = reply["body"]["result"].get("task", reply["body"]["result"])
                self.task, self.context = task["id"], task["contextId"]
                if stream:
                    current = self.rpc(endpoint, "GetTask", {"id": self.task}, token, {"A2A-Version": "1.0"})
                    require(current["status"] < 400 and not current.get("body", {}).get("error")
                        and isinstance(current.get("body", {}).get("result"), dict),
                        "A2A stream terminal task read failed: " + a2a_read_diagnostic(current))
                    task = current["body"]["result"]
                self.task, self.context = task["id"], task["contextId"]
                deadline = time.monotonic() + 150
                while task["status"]["state"] not in ("TASK_STATE_INPUT_REQUIRED", "TASK_STATE_COMPLETED", "TASK_STATE_FAILED", "TASK_STATE_CANCELED"):
                    require(time.monotonic() < deadline, "A2A terminal deadline expired")
                    time.sleep(0.2)
                    reply = self.rpc(endpoint, "GetTask", {"id": self.task}, token, {"A2A-Version": "1.0"})
                    require(reply["status"] < 400 and not reply.get("body", {}).get("error")
                        and isinstance(reply.get("body", {}).get("result"), dict),
                        "A2A task read failed: " + a2a_read_diagnostic(reply))
                    task = reply["body"]["result"]
                reply = {"status": 200, "body": {"result": task}}
        self.last_reply = reply
        return reply

    def calls(self, reply):
        if self.protocol == "openai":
            if "body" in reply:
                body = reply["body"]
                return body.get("choices", [{}])[0].get("message", {}).get("tool_calls", [])
            calls = {}
            for event in reply["events"]:
                for choice in event.get("choices", []):
                    for part in choice.get("delta", {}).get("tool_calls", []):
                        call = calls.setdefault(part["index"], {"id": "", "type": "function", "function": {"name": "", "arguments": ""}})
                        call["id"] += part.get("id", "")
                        call["function"]["name"] += part.get("function", {}).get("name", "")
                        call["function"]["arguments"] += part.get("function", {}).get("arguments", "")
            require(reply["done"], "OpenAI stream missing DONE")
            return [calls[index] for index in sorted(calls)]
        if self.protocol == "anthropic":
            blocks = reply.get("body", {}).get("content", [])
            if "events" in reply:
                indexed = {}
                for event in reply["events"]:
                    if event.get("type") == "content_block_start":
                        indexed[event["index"]] = dict(event["content_block"], arguments="")
                    if event.get("type") == "content_block_delta" and event.get("delta", {}).get("type") == "input_json_delta":
                        indexed[event["index"]]["arguments"] += event["delta"]["partial_json"]
                blocks = list(indexed.values())
                require(any(event.get("type") == "message_stop" for event in reply["events"]), "Anthropic stream missing message_stop")
            return [{"id": block["id"], "type": "function", "function": {"name": block["name"], "arguments": block.get("arguments") or json.dumps(block["input"])}} for block in blocks if block.get("type") == "tool_use"]
        if self.protocol == "agui":
            terminal = next((event for event in reply.get("events", []) if event.get("type") == "RUN_FINISHED"), None)
            require(terminal is not None or failed(reply), "AG-UI missing terminal")
            return (terminal or {}).get("metadata", {}).get("uar", {}).get("clientToolCalls", [])
        body = reply.get("body", {}).get("result", {})
        if self.protocol == "acp":
            if "events" in reply:
                body = next((event for event in reversed(reply["events"]) if "run_id" in event and "status" in event), {})
            if body.get("client_tools"):
                self.history = body["client_tools"]["history"][:-1]
                return body["client_tools"]["history"][-1]["tool_calls"]
            return []
        extension = body.get("metadata", {}).get(EXTENSION, {})
        if extension:
            self.history = extension["history"][:-1]
        return extension.get("pending_calls", [])

    def answer(self, calls, value="fixture-result", assistant_text=""):
        require(calls, "expected caller tool requests")
        require(all(call["id"].startswith("uar_client_") for call in calls), "caller IDs lack durable host-issued locator")
        require(all(call["function"]["name"] in ("test_lookup", "test_second") for call in calls), "host executable leaked as caller tool")
        for call in calls:
            arguments = json.loads(call["function"]["arguments"])
            require(isinstance(arguments.get("query"), str), "caller arguments lost declared schema")
        self.history += [{"role": "assistant", "content": assistant_text, "tool_calls": copy.deepcopy(calls)}]
        self.history += [{"role": "tool", "tool_call_id": call["id"], "content": value + ":" + str(index)} for index, call in enumerate(calls)]
        self.acp_rpc_id = None


def configure(args, scenario):
    settings = {"scenario": scenario}
    if scenario == "graph":
        require(args.graph_agent_id == "orchestrator-agent", "graph scenario requires the installed orchestrator-agent graph")
        require(args.graph_before_marker and args.graph_after_marker and args.graph_before_marker != args.graph_after_marker, "graph scenario requires distinct router and worker prompt markers")
        settings["graph_before_marker"] = args.graph_before_marker
        settings["graph_after_marker"] = args.graph_after_marker
    if args.mode.startswith("budget"):
        require(args.budget_provider_usage_json, "budget requires source-bound reservation limits and explicit provider wire usage; numeric bounds remain an operator prerequisite")
        usage = json.loads(args.budget_provider_usage_json.read_text())
        require(isinstance(usage, dict) and set(usage) == {"input_tokens", "output_tokens"}
            and all(type(value) is int and value >= 0 for value in usage.values()), "invalid provider wire usage")
        settings["provider_usage"] = usage
    if args.host_tool_json:
        settings["host_tool"] = json.loads(args.host_tool_json.read_text())
    reply = http(args.provider_base_url.rstrip("/") + "/_control", settings)
    require(reply["status"] == 200, "controlled provider configuration failed")


def trace(args):
    return http(args.provider_base_url.rstrip("/") + "/_trace")["body"]["requests"]


def graph_waiting_response(driver, response):
    if driver.protocol == "agui":
        return any(event.get("type") == "RUN_FINISHED" and event.get("outcome", {}).get("type") == "interrupt"
            for event in response.get("events", []))
    if driver.protocol == "a2a":
        task = response.get("body", {}).get("result", {})
        state = task.get("status", {}).get("state")
        return state in ("TASK_STATE_SUBMITTED", "TASK_STATE_WORKING") or state == "TASK_STATE_INPUT_REQUIRED" and not task.get("metadata", {}).get(EXTENSION, {}).get("pending_calls")
    return False


def graph_completed_retry(driver, stream=False):
    return graph_send(driver, stream=stream, allow_spawn=False)


def graph_owned_request(call, deadline, *arguments, **keywords):
    previous = getattr(TRANSPORT, "deadline", None)
    TRANSPORT.deadline = deadline
    try:
        return call(*arguments, **keywords)
    finally:
        TRANSPORT.deadline = previous


def graph_selection_receipt(driver):
    args = driver.args
    require(not driver.graph_approval_receipts or driver.graph_selection is not None,
        "graph continuation lacks its persisted initial router selection receipt")
    marker = driver.history[0]["content"].removesuffix(":earlier-user")
    peer = http(args.provider_base_url.rstrip("/") + "/_trace", timeout=10)["body"]
    settings = peer["settings"]
    require(settings.get("scenario") == "graph" and settings.get("graph_before_marker") == args.graph_before_marker
        and settings.get("graph_after_marker") == args.graph_after_marker, "redacted spawn lacks the controlled graph fixture binding")
    selected = []
    for entry in peer["requests"]:
        request = entry["request"]
        prompt = [message.get("content", "") for message in request.get("messages", []) if message.get("role") in ("system", "developer")]
        if "system" in request:
            prompt.append(request["system"])
        identity = json.dumps(prompt)
        if marker in json.dumps(request) and args.graph_before_marker in identity:
            require(not request.get("tools") and args.graph_after_marker not in identity, "redacted spawn router identity is not tool-free and distinct")
            selected.append(request)
    require(len(selected) == 1, "redacted spawn requires exactly one actual router selection for its conversation")
    receipt = {"marker": marker, "router_request": selected[0], "selected_worker": "general-purpose",
        "fixture_settings": settings, "selection_basis": "controlled peer graph produce path returns exactly general-purpose"}
    if driver.graph_selection is not None:
        require(driver.graph_selection == receipt, "graph continuation changed its saved controlled router selection")
    else:
        driver.graph_selection = receipt
    return receipt


def graph_child_receipt(driver, root, worker_offset):
    args = driver.args
    snapshot = http(args.base_url.rstrip("/") + "/api/uar/runs/" + urllib.parse.quote(root, safe="") + "/events",
        token=driver.token, headers=args.workspace_headers, timeout=10)
    require(snapshot["status"] == 200 and snapshot["body"].get("runId") == root
        and snapshot["body"].get("gapReason") is None, "graph child identity snapshot is absent or has a retention gap")
    children = []
    for event in snapshot["body"].get("events", []):
        if event.get("eventName", "").startswith("agui.subagent."):
            data = event.get("data", {})
            lifecycle = data.get("lifecycle", {})
            require(data.get("request_id") == root and lifecycle.get("parent_run_id") == root
                and lifecycle.get("artifact_id") == "general-purpose", "approved graph produced an unexpected child identity")
            require(lifecycle.get("root_thread_id") and lifecycle.get("parent_thread_id")
                and lifecycle.get("child_thread_id"), "graph child lifecycle lacks actual thread IDs")
            children.append(event)
    require(children and len({event["data"]["lifecycle"]["child_thread_id"] for event in children}) == 1,
        "graph approval did not produce exactly one public selected child")
    child_runs = {event["data"]["lifecycle"]["child_run_id"] for event in children if event["data"]["lifecycle"].get("child_run_id")}
    require(len(child_runs) == 1, "graph selected child has no unique actual model-run identity")
    peer = http(args.provider_base_url.rstrip("/") + "/_trace", timeout=10)["body"]
    worker = peer["requests"][worker_offset:]
    require(len(worker) == 1, "graph approved root did not make exactly one selected worker model call")
    request = worker[0]["request"]
    prompt = [message.get("content", "") for message in request.get("messages", []) if message.get("role") in ("system", "developer")]
    if "system" in request:
        prompt.append(request["system"])
    require(args.graph_after_marker in json.dumps(prompt) and args.graph_before_marker not in json.dumps(prompt)
        and driver.graph_selection["marker"] in json.dumps(request), "approved graph selected the wrong worker prompt/conversation")
    return {"public_events": snapshot, "selected_child_events": children, "worker_request": worker[0],
        "identity_basis": "owned approved root lifecycle and its isolated post-decision provider trace"}


def graph_send(driver, stream=False, allow_spawn=True):
    args = driver.args
    runs_url = args.base_url.rstrip("/") + "/api/uar/runs"
    baseline = http(runs_url, token=driver.token, headers=args.workspace_headers, timeout=10)
    require(baseline["status"] == 200, "graph approval cannot inspect the owner run baseline")
    old_ids = {run["run_id"] for run in baseline["body"]}
    old_effects = peer_state(args)["host_effects"] if not allow_spawn else None
    observation = {"baseline": baseline, "allow_spawn": allow_spawn, "approvals": [], "cleanup": []}
    driver.graph_approval_observations.append(observation)
    EVIDENCE.append({"graph_approval_observation": observation})
    root = None
    challenge = None
    failure = None
    response = None
    deadline = time.monotonic() + 90
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
        request = pool.submit(graph_owned_request, driver.send, deadline, stream=stream)
        requests = [request]
        try:
            while True:
                require(time.monotonic() < deadline, "graph request/approval coordinator deadline expired")
                if request.done():
                    response = request.result(timeout=180)
                    projection = driver.agui_graph_projections.get(json.dumps(driver.history, sort_keys=True))
                    if driver.protocol == "agui" and not allow_spawn and projection is not None:
                        require(response["status"] < 400 and not failed(response), "graph duplicate AG-UI submission failed before projection attachment")
                        driver.run = projection
                        attach = args.base_url.rstrip("/") + "/api/uar/ag-ui/agents/" + urllib.parse.quote(driver.agent, safe="")
                        attach += "/threads/" + urllib.parse.quote(driver.thread, safe="") + "/runs/" + urllib.parse.quote(projection, safe="") + "/stream"
                        observation["duplicate_submission_response"] = response
                        response = http(attach, token=driver.token, headers={"Accept": "text/event-stream", **args.workspace_headers}, timeout=10, deadline=deadline)
                        observation["projection_attach_response"] = response
                    if driver.protocol == "a2a" and challenge is not None and graph_waiting_response(driver, response):
                        task_endpoint = args.a2a_path or "/a2a/v1/agents/" + urllib.parse.quote(driver.agent, safe="")
                        response = http(args.base_url.rstrip("/") + task_endpoint,
                            {"jsonrpc": "2.0", "id": str(uuid.uuid4()), "method": "GetTask", "params": {"id": driver.task}},
                            driver.token, {"A2A-Version": "1.0", **args.workspace_headers}, timeout=10)
                        require(response["status"] < 400 and not response.get("body", {}).get("error")
                            and isinstance(response.get("body", {}).get("result"), dict),
                            "graph A2A attach read failed: " + a2a_read_diagnostic(response))
                        task = response["body"]["result"]
                        require(task.get("id") == driver.task and task.get("contextId") == driver.context,
                            "graph A2A attach changed task/context identity")
                        observation.setdefault("task_reads", []).append(response)
                    if not graph_waiting_response(driver, response):
                        if allow_spawn:
                            require(challenge is not None, "graph request completed without servicing its required spawn approval")
                        break
                    require(allow_spawn, "completed graph replay returned an approval interrupt")
                if root is None:
                    current = http(runs_url, token=driver.token, headers=args.workspace_headers, timeout=10)
                    require(current["status"] == 200, "graph approval cannot inspect current owner runs")
                    candidates = [run for run in current["body"] if run["run_id"] not in old_ids
                        and run["agent_id"] == "orchestrator-agent" and run["status"] in ("pending", "running", "paused")]
                    require(len(candidates) <= 1, "graph approval root identity is ambiguous")
                    if candidates:
                        root = candidates[0]["run_id"]
                        require(all(receipt["root_run_id"] != root for receipt in driver.graph_approval_receipts),
                            "graph continuation reused a prior approval root")
                        observation["root_snapshot"] = current
                        observation["root_run_id"] = root
                if root is not None:
                    endpoint = runs_url + "/" + urllib.parse.quote(root, safe="") + "/tool-approval"
                    pending_reply = http(endpoint + "/pending", token=driver.token, headers=args.workspace_headers, timeout=10)
                    require(pending_reply["status"] == 200 and pending_reply["body"].get("runId") == root,
                        "graph approval pending read lost the exact native root")
                    pending = pending_reply["body"].get("pending")
                    if pending is not None:
                        require(allow_spawn, "graph replay requested a new spawn approval")
                        require(pending.get("rootRunId") == root and pending.get("name") == "spawn_agent",
                            "graph approval is not the expected live root spawn")
                        approval_id = pending.get("approvalId")
                        require(approval_id and approval_id == pending.get("challengeId") and pending.get("toolCallId"),
                            "graph approval challenge identity is incomplete")
                        require(challenge == approval_id or all(receipt["pending"]["body"]["pending"]["approvalId"] != approval_id
                            for receipt in driver.graph_approval_receipts), "graph continuation reused a prior spawn approval")
                        arguments = json.loads(pending["argumentsJson"])
                        require(isinstance(arguments, dict) and set(arguments) == {"operation", "detailsAvailable"}
                            and arguments["operation"] == "spawn_agent" and arguments["detailsAvailable"] is False,
                            "graph spawn approval display differs from the exact redacted public contract")
                        if challenge != approval_id:
                            require(challenge is None, "graph request produced a second spawn challenge")
                            selection = graph_selection_receipt(driver)
                            worker_offset = len(trace(args))
                            if driver.protocol == "agui":
                                if not request.done():
                                    time.sleep(0.05)
                                    continue
                                original = request.result(timeout=180)
                                terminal = next((event for event in original.get("events", [])
                                    if event.get("type") == "RUN_FINISHED"), None)
                                interrupts = (terminal or {}).get("outcome", {}).get("interrupts", [])
                                require(terminal and terminal.get("threadId") == driver.thread and terminal.get("runId") == driver.run
                                    and len(interrupts) == 1 and interrupts[0].get("id") == approval_id
                                    and interrupts[0].get("reason") == "tool_approval", "graph AG-UI sealed interrupt does not match the exact approval")
                                parent_wire = driver.run
                                driver.run = str(uuid.uuid4())
                                driver.agui_graph_projections[json.dumps(driver.history, sort_keys=True)] = driver.run
                                resume = {"threadId": driver.thread, "runId": driver.run, "parentRunId": parent_wire,
                                    "protocolVersion": "1.0", "messages": [], "tools": [], "forwardedProps": {"uar": {}},
                                    "resume": [{"interruptId": approval_id, "status": "resolved", "payload": {"approved": True}}]}
                                native_url = args.base_url.rstrip("/") + "/api/uar/ag-ui/agents/" + urllib.parse.quote(driver.agent, safe="")
                                request = pool.submit(graph_owned_request, http, deadline, native_url, resume, driver.token,
                                    {"Accept": "text/event-stream", **args.workspace_headers})
                                requests.append(request)
                                decided = {"native_resume_request": resume, "sealed_interrupt_response": original}
                            else:
                                decided = http(endpoint, {"approval_id": approval_id, "approved": True}, driver.token,
                                    args.workspace_headers, timeout=10)
                                require(decided["status"] == 200 and decided["body"].get("resolved") is True
                                    and decided["body"].get("decision") == "allow", "graph spawn approval was not delivered")
                            challenge = approval_id
                            receipt = {"root_run_id": root, "pending": pending_reply, "public_display": arguments,
                                "hidden_arguments_verified": False, "controlled_selection": selection, "decision": decided}
                            observation["approvals"].append(receipt)
                            driver.graph_approval_receipts.append(receipt)
                time.sleep(0.05)
            if allow_spawn:
                require(challenge is not None, "graph request completed without servicing its required spawn approval")
                observation["approvals"][0]["child_identity"] = graph_child_receipt(driver, root, worker_offset)
                if driver.protocol == "agui":
                    records = http(endpoint, token=driver.token, headers=args.workspace_headers, timeout=10)
                    require(records["status"] == 200 and records["body"].get("runId") == root, "graph AG-UI approval receipt read failed")
                    approved = [record for record in records["body"].get("records", []) if record.get("challengeId") == challenge]
                    require(len(approved) == 1 and approved[0].get("state") == "approved"
                        and approved[0].get("decision", {}).get("approved") is True, "graph AG-UI native resume did not commit the exact approval")
                    observation["approvals"][0]["decision"]["approval_records"] = records
                    observation["approvals"][0]["decision"]["native_resume_response"] = response
                    final_roots = http(runs_url, token=driver.token, headers=args.workspace_headers, timeout=10)
                    require(final_roots["status"] == 200, "graph AG-UI cannot confirm its physical root")
                    require({run["run_id"] for run in final_roots["body"] if run["run_id"] not in old_ids
                        and run["agent_id"] == "orchestrator-agent"} == {root}, "graph AG-UI approval resume admitted another physical root")
                    observation["final_snapshot"] = final_roots
            else:
                final_runs = http(runs_url, token=driver.token, headers=args.workspace_headers, timeout=10)
                require(final_runs["status"] == 200, "graph replay cannot inspect final owner roots")
                observation["final_snapshot"] = final_runs
                require(not any(run["run_id"] not in old_ids and run["agent_id"] == "orchestrator-agent"
                    for run in final_runs["body"]), "completed graph replay admitted a new root")
                require(peer_state(args)["host_effects"] == old_effects, "completed graph replay repeated a real host effect")
        except BaseException as error:
            failure = error
        finally:
            if failure is not None and root is not None:
                try:
                    observation["cleanup"].append(http(runs_url + "/" + urllib.parse.quote(root, safe="") + "/cancel",
                        {}, driver.token, args.workspace_headers, timeout=10))
                except Exception as error:
                    observation["cleanup"].append({"cancel_error_type": type(error).__name__})
            for owned in requests:
                try:
                    owned.result(timeout=180)
                except Exception as error:
                    observation.setdefault("request_error_types", []).append(type(error).__name__)
                    if failure is None:
                        failure = error
            observation["requests_joined"] = len(requests)
    observation["executor_joined"] = True
    if failure is not None:
        raise failure
    driver.last_reply = response
    return response


def loop_case(args, protocol, scenario="single", choice="auto", parallel=True, stream=False, agent=None):
    configure(args, scenario)
    driver = Driver(args, protocol, agent)
    driver.choice, driver.parallel = choice, parallel
    first = graph_send(driver, stream=stream) if scenario == "graph" else driver.send(stream=stream)
    require(not failed(first), "initial request rejected: " + json.dumps(first))
    calls = driver.calls(first)
    if choice == "none":
        require(not calls, "none requested a caller tool")
        captured = trace(args)
        require(captured, "candidate never contacted real provider")
        choice_wire = captured[0]["request"].get("tool_choice")
        require(choice_wire == "none" or isinstance(choice_wire, dict) and choice_wire.get("type") == "none", "provider lost none selection")
        return {"provider_contacts": len(captured), "calls": 0}
    require(calls, "caller handoff missing")
    if protocol == "openai":
        if "body" in first:
            require(first["body"]["choices"][0]["finish_reason"] == "tool_calls", "OpenAI handoff finish reason lost")
        else:
            reasons = [choice.get("finish_reason") for event in first["events"] for choice in event.get("choices", [])]
            require("tool_calls" in reasons and "stop" not in reasons, "OpenAI stream falsely stopped tool handoff")
    if protocol == "anthropic" and "body" in first:
        require(first["body"].get("stop_reason") == "tool_use", "Anthropic handoff stop reason lost")
    captured_request = trace(args)[1 if scenario == "graph" else 0]["request"]
    if protocol != "agui":
        forwarded = [tool.get("function", tool) for tool in captured_request.get("tools", [])]
        lookup = next((tool for tool in forwarded if tool.get("name") == "test_lookup"), None)
        require(lookup is not None, "provider lost caller function declaration")
        require(lookup.get("parameters", lookup.get("input_schema")) == SCHEMA, "provider changed caller JSON schema")
        require(lookup.get("strict") is True, "provider lost strict declaration")
    if not parallel:
        require(len(calls) == 1, "parallel false emitted multiple calls")
        choice_wire = captured_request.get("tool_choice")
        require(captured_request.get("parallel_tool_calls") is False or isinstance(choice_wire, dict) and choice_wire.get("disable_parallel_tool_use") is True, "provider lost parallel false restriction")
    if choice == "named":
        require(all(call["function"]["name"] == "test_lookup" for call in calls), "named selection changed")
    if choice in ("required", "named"):
        captured = trace(args)
        require(captured, "no actual provider request captured")
        provider_choice = captured[0]["request"].get("tool_choice")
        require(provider_choice is not None, "provider lost explicit tool_choice")
        return {"provider_contacts": len(captured), "calls": len(calls), "provider_tool_choice": provider_choice}
    handoffs = 0
    while calls:
        handoffs += 1
        require(handoffs <= 4, "caller loop did not converge")
        driver.answer(calls)
        reply = graph_send(driver, stream=stream) if scenario == "graph" else driver.send(stream=stream)
        require(not failed(reply), "result submission rejected: " + json.dumps(reply))
        calls = driver.calls(reply)
    contacts = len(trace(args))
    require(contacts >= 2, "result continuation did not reach real provider")
    before = contacts
    repeated = graph_completed_retry(driver, stream=stream) if scenario == "graph" else driver.send(stream=stream)
    require(not failed(repeated), "identical completed result retry rejected")
    require(not driver.calls(repeated), "completed result retry reissued tool calls")
    require(len(trace(args)) == before, "completed result retry executed provider again")
    if scenario == "mixed":
        peer = http(args.provider_base_url.rstrip("/") + "/_trace")["body"]
        require(len(peer["host_effects"]) == 1, "real MCP host effect did not execute exactly once across completed retry")
    captured = json.dumps(trace(args))
    require("fixture-result" in captured, "provider did not receive caller tool result")
    if scenario != "graph":
        require(driver.history[0]["content"] in captured and driver.history[1]["content"] in captured, "provider lost earlier full history")
    observed = {"provider_contacts": contacts, "handoffs": handoffs, "retry_provider_contacts": 0}
    if scenario == "graph":
        require(len(driver.graph_approval_receipts) == 2, "graph warm loop lost its two independently governed spawn decisions")
        observed["graph_approvals"] = driver.graph_approval_receipts
    return observed


def rejection_case(args, protocol, mutation):
    configure(args, "parallel")
    if protocol == "openai" and mutation == "history":
        fresh = Driver(args, protocol)
        TRANSPORT.capture = True
        try:
            try:
                fresh.send()
            except CapturedRequest as captured:
                url, body, token, headers = captured.request
            else:
                raise AssertionError("fresh OpenAI caller-tool request was not captured")
        finally:
            TRANSPORT.capture = False
        body["message"] = "legacy alias conflicts with the supplied messages history"
        before = len(trace(args))
        alias_reply = http(url, body, token, headers)
        require(400 <= alias_reply["status"] < 500, "legacy message alias was not rejected with a client error: " + json.dumps(alias_reply))
        require(len(trace(args)) == before, "legacy message alias rejection contacted provider")
    driver = Driver(args, protocol)
    initial = driver.send()
    require(not failed(initial), "rejection setup failed")
    calls = driver.calls(initial)
    driver.answer(calls)
    if mutation == "conflict":
        completed = driver.send()
        require(not failed(completed) and not driver.calls(completed), "conflict setup did not complete")
        driver.history[-1]["content"] += " conflicting retry"
    elif mutation == "forged":
        driver.history[-1]["tool_call_id"] = "uar_client_" + str(uuid.uuid4()) + "_" + str(uuid.uuid4())
    elif mutation == "partial":
        require(len(calls) == 2, "parallel setup did not yield two calls")
        driver.history.pop()
    elif mutation == "duplicate":
        driver.history.append(copy.deepcopy(driver.history[-1]))
    elif mutation == "history":
        driver.history[0]["content"] += " altered"
    elif mutation == "schema":
        driver.tools[0]["function"]["description"] += " altered"
    before = len(trace(args))
    reply = driver.send(token=os.environ[args.other_bearer_token_env] if mutation == "owner" else None)
    require(failed(reply), "invalid continuation accepted: " + mutation + ":" + json.dumps(reply))
    require(len(trace(args)) == before, "rejected continuation contacted provider")
    observed = {"rejected": mutation, "provider_contacts_after_rejection": 0}
    if protocol == "openai" and mutation == "history":
        observed["legacy_message_alias_status"] = alias_reply["status"]
    return observed


def peer_state(args):
    return http(args.provider_base_url.rstrip("/") + "/_trace")["body"]


def wait_peer(args, predicate, message):
    deadline = time.monotonic() + 30
    while time.monotonic() < deadline:
        state = peer_state(args)
        if predicate(state):
            return state
        time.sleep(0.05)
    raise AssertionError(message)


def prepared_results(args, protocol, mixed=False):
    configure(args, "mixed" if mixed else "single")
    driver = Driver(args, protocol)
    first = driver.send()
    require(not failed(first), "race/disconnect initial handoff failed")
    driver.answer(driver.calls(first))
    return driver


def concurrent_case(args, protocol):
    driver = prepared_results(args, protocol, mixed=bool(args.host_tool_json))
    before = len(trace(args))
    before_effects = len(peer_state(args)["host_effects"])
    http(args.provider_base_url.rstrip("/") + "/_hold", {})
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            first = pool.submit(copy.deepcopy(driver).send)
            wait_peer(args, lambda state: state["held_requests"] >= 1, "first duplicate did not reach held real provider")
            second = pool.submit(copy.deepcopy(driver).send)
            # The first claim remains live while the second races the durable CAS.
            time.sleep(0.2)
            http(args.provider_base_url.rstrip("/") + "/_release", {})
            replies = [first.result(timeout=180), second.result(timeout=180)]
    finally:
        http(args.provider_base_url.rstrip("/") + "/_release", {})
    require(any(not failed(reply) for reply in replies), "both identical concurrent submissions failed")
    require(len(trace(args)) == before + 1, "concurrent identical results executed provider more than once")
    require(len(peer_state(args)["host_effects"]) == before_effects, "concurrent continuation repeated a host effect")
    repeated = driver.send()
    require(not failed(repeated), "completed concurrent result did not replay")
    require(len(trace(args)) == before + 1, "post-race completed retry contacted provider")
    return {"provider_contacts_for_two_submissions": 1, "host_effects_repeated": 0,
        "submission_outcomes": ["rejected" if failed(reply) else "completed-or-replayed" for reply in replies]}


def disconnect_case(args, protocol):
    driver = prepared_results(args, protocol, mixed=bool(args.host_tool_json))
    TRANSPORT.capture = True
    try:
        try:
            driver.send(stream=True)
        except CapturedRequest as captured:
            url, body, token, headers = captured.request
        else:
            raise AssertionError("native continuation request was not captured")
    finally:
        TRANSPORT.capture = False
    target = urllib.parse.urlsplit(url)
    connection_type = http_client.HTTPSConnection if target.scheme == "https" else http_client.HTTPConnection
    connection = connection_type(target.hostname, target.port, timeout=180)
    request_headers = {"Content-Type": "application/json", "Accept": "text/event-stream", **(headers or {})}
    if token:
        request_headers["Authorization"] = "Bearer " + token
    before = len(trace(args))
    effects = len(peer_state(args)["host_effects"])
    http(args.provider_base_url.rstrip("/") + "/_hold", {})
    try:
        connection.request("POST", target.path + (("?" + target.query) if target.query else ""), json.dumps(body).encode(), request_headers)
        wait_peer(args, lambda state: state["held_requests"] >= 1, "disconnect request never reached held provider")
        # Close the actual candidate HTTP socket while its upstream response is held.
        connection.close()
    finally:
        connection.close()
        http(args.provider_base_url.rstrip("/") + "/_release", {})
    wait_peer(args, lambda state: state["held_requests"] == 0, "provider remained held after disconnect release")
    retry = driver.send(stream=True)
    require(failed(retry) or not driver.calls(retry), "disconnected result retry returned a fresh caller handoff")
    require(len(trace(args)) == before + 1, "disconnected claimed continuation executed provider again")
    require(len(peer_state(args)["host_effects"]) == effects, "disconnect/retry repeated real MCP effect")
    return {"closed_actual_http_socket_with_provider_held": True,
        "retry": "fail-closed" if failed(retry) else "durable-replay", "provider_reexecutions": 0, "host_effect_reexecutions": 0}


def provider_failure_case(args, protocol, scenario):
    configure(args, scenario)
    driver = Driver(args, protocol)
    reply = driver.send(stream=True)
    require(failed(reply), "invalid/truncated upstream became public success: " + json.dumps(reply))
    for event in reply.get("events", []):
        for choice in event.get("choices", []):
            require(choice.get("finish_reason") not in ("stop", "tool_calls"), "OpenAI error emitted successful terminal")
        require(not (event.get("type") == "message_delta" and event.get("delta", {}).get("stop_reason") in ("end_turn", "tool_use")), "Anthropic error emitted successful stop reason")
        require(not (event.get("type") == "RUN_FINISHED" and event.get("outcome", {}).get("type", "success") == "success"), "AG-UI invalid upstream emitted success")
    require(trace(args), "negative stream never reached actual provider")
    return {"upstream": scenario, "public_failure": True, "provider_contacts": len(trace(args))}


def cancellation_case(args, protocol):
    driver = prepared_results(args, protocol, mixed=True)
    require(len(peer_state(args)["host_effects"]) == 1, "cancellation setup did not settle one real host effect")
    before = len(trace(args))
    runs_url = args.base_url.rstrip("/") + "/api/uar/runs"
    snapshot = http(runs_url, token=driver.token)
    require(snapshot["status"] == 200, "cannot retain owner-visible run baseline")
    old_runs = {run["run_id"] for run in snapshot["body"]}
    TRANSPORT.capture = True
    try:
        try:
            driver.send(stream=True)
        except CapturedRequest as captured:
            url, body, token, headers = captured.request
        else:
            raise AssertionError("cancellation continuation was not captured")
    finally:
        TRANSPORT.capture = False
    require(http(args.provider_base_url.rstrip("/") + "/_hold", {})["status"] == 200, "provider hold failed")
    native_run = None
    cancel_reply = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
        pending = None
        try:
            pending = pool.submit(http, url, body, token, headers)
            wait_peer(args, lambda state: state["held_requests"] >= 1, "cancel continuation did not reach held real provider")
            if protocol == "a2a":
                endpoint = args.a2a_path or "/a2a/v1/agents/" + urllib.parse.quote(driver.agent, safe="")
                cancel_reply = driver.rpc(endpoint, "CancelTask", {"id": driver.task}, headers={"A2A-Version": "1.0", **args.workspace_headers})
                require(cancel_reply["status"] == 200 and not cancel_reply.get("body", {}).get("error"), "native A2A cancellation failed")
                task = cancel_reply["body"]["result"]
                require(task["status"]["state"] == "TASK_STATE_CANCELED" and not task.get("metadata", {}).get("cleanup_unconfirmed"), "A2A cancellation did not confirm cleanup")
            else:
                current = http(runs_url, token=driver.token)
                require(current["status"] == 200, "cannot inspect actual owner cancellation identity")
                active = [run for run in current["body"] if run["run_id"] not in old_runs
                    and run["agent_id"] == driver.agent and run["status"] in ("pending", "running")]
                require(len(active) == 1, "held continuation has no unique new owner-visible native run")
                native_run = active[0]["run_id"]
                cancel_reply = http(runs_url + "/" + urllib.parse.quote(native_run, safe="") + "/cancel", {}, driver.token)
                require(cancel_reply["status"] == 200 and cancel_reply["body"].get("cancelled") is True, "native run cancellation was not accepted")
        finally:
            try:
                released = http(args.provider_base_url.rstrip("/") + "/_release", {})
                require(released["status"] == 200, "provider release failed after cancellation")
            finally:
                # Joining this owned request is mandatory even after a failed
                # cancellation assertion; no request worker escapes the case.
                if pending is not None:
                    reply = pending.result(timeout=180)
    wait_peer(args, lambda state: state["held_requests"] == 0, "cancellation left a held provider request")
    if protocol == "a2a":
        terminal = driver.rpc(endpoint, "GetTask", {"id": driver.task}, headers={"A2A-Version": "1.0", **args.workspace_headers})
        require(terminal["body"]["result"]["status"]["state"] == "TASK_STATE_CANCELED", "cancelled task changed to a successful terminal")
        require("TASK_STATE_COMPLETED" not in json.dumps(reply), "cancelled A2A stream emitted completion")
    else:
        require(failed(reply), "explicit cancellation emitted a successful native result")
        inspected = http(runs_url + "/" + urllib.parse.quote(native_run, safe=""), token=driver.token)
        require(inspected["status"] == 200 and inspected["body"]["status"] == "cancelled", "owner-visible run did not retain cancelled status")
    for event in reply.get("events", []):
        for choice in event.get("choices", []):
            require(choice.get("finish_reason") not in ("stop", "tool_calls"), "cancelled OpenAI response emitted success")
        require(not (event.get("type") == "message_delta" and event.get("delta", {}).get("stop_reason") in ("end_turn", "tool_use")), "cancelled Anthropic response emitted success")
        require(not (event.get("type") == "RUN_FINISHED" and event.get("outcome", {}).get("type", "success") == "success"), "cancelled AG-UI response emitted success")
        require(not (event.get("status") in ("done", "client_tools_required")), "cancelled ACP response emitted success")
    require(len(trace(args)) == before + 1, "cancellation executed extra provider work")
    require(len(peer_state(args)["host_effects"]) == 1, "cancellation repeated a settled host effect")
    contacts = len(trace(args))
    retried = http(url, body, token, headers)
    require(failed(retried) or protocol == "a2a" and "TASK_STATE_CANCELED" in json.dumps(retried), "cancelled claimed batch retry became success")
    require(len(trace(args)) == contacts and len(peer_state(args)["host_effects"]) == 1, "cancelled claimed batch retry repeated effects")
    return {"protocol": protocol, "native_run_id": native_run, "task_id": driver.task,
        "explicit_cancel_response": cancel_reply, "continuation_response": reply,
        "settled_host_effects": 1, "retry_provider_contacts": 0, "owned_request_joined": True, "provider_released": True}


def graph_sequence(args, marker):
    marker = marker.removesuffix(":earlier-user")
    requests = [entry["request"] for entry in trace(args) if marker in json.dumps(entry["request"])]
    sequence = []
    for request in requests:
        prompt = [message.get("content", "") for message in request.get("messages", [])
            if message.get("role") in ("system", "developer")]
        if "system" in request:
            prompt.append(request["system"])
        identity = json.dumps(prompt)
        router = args.graph_before_marker in identity
        worker = args.graph_after_marker in identity
        require(router != worker, "graph request must identify exactly one active router or worker prompt")
        if router:
            require(not request.get("tools"), "router unexpectedly received tools")
            sequence.append("router")
        elif worker:
            sequence.append("worker")
        else:
            raise AssertionError("graph provider request lost both configured prompt markers")
    require(sequence == ["router", "worker", "worker"], "graph cursor replayed or skipped a node: " + json.dumps(sequence))
    return sequence


def safety_input_case(args, protocol, mutation):
    configure(args, "single")
    driver = Driver(args, protocol, args.safety_agent_id)
    if mutation == "duplicate_names":
        driver.tools[1]["function"]["name"] = driver.tools[0]["function"]["name"]
    elif mutation == "host_collision":
        driver.tools[0]["function"]["name"] = json.loads(args.host_tool_json.read_text())["name"]
    elif mutation == "malformed_schema":
        driver.tools[0]["function"]["parameters"] = {"type": "not-a-json-schema-type"}
    reply = driver.send()
    require(failed(reply), "invalid caller declaration accepted: " + mutation)
    require(not trace(args), "invalid caller declaration reached provider: " + mutation)
    require(not peer_state(args)["host_effects"], "invalid caller declaration executed host tool")
    return {"rejected": mutation, "provider_contacts": 0, "host_effects": 0}


def safety_success_case(args, protocol, scenario, stream=False):
    configure(args, scenario)
    driver = Driver(args, protocol, args.safety_agent_id)
    reply = driver.send(stream=stream)
    require(not failed(reply), "safety caller handoff failed: " + json.dumps(reply))
    calls = driver.calls(reply)
    require(len(calls) == (2 if scenario == "parallel" else 1), "safety handoff lost the complete parallel batch")
    if scenario == "mixed_text":
        require("client-tools-mixed-text" in json.dumps(reply), "mixed text/tool response lost assistant text")
    require(not peer_state(args)["host_effects"], "UAR executed caller-owned function before handoff")
    driver.answer(calls, assistant_text="client-tools-mixed-text" if scenario == "mixed_text" else "")
    completed = driver.send(stream=stream)
    require(not failed(completed) and not driver.calls(completed), "safety caller result batch did not complete")
    require(not peer_state(args)["host_effects"], "UAR executed caller-owned function during continuation")
    require(len(trace(args)) == 2, "caller result batch did not use exactly one continuation call")
    return {"caller_calls": len(calls), "provider_contacts": 2, "host_effects": 0, "stream": stream}


def safety_provider_case(args, protocol, scenario):
    configure(args, scenario)
    driver = Driver(args, protocol, args.safety_agent_id)
    if scenario == "force_parallel":
        driver.parallel = False
    reply = driver.send(stream=True)
    require(failed(reply), "provider violation became a successful handoff: " + scenario + ":" + json.dumps(reply))
    for event in reply.get("events", []):
        for choice in event.get("choices", []):
            require(choice.get("finish_reason") not in ("stop", "tool_calls"), "provider violation emitted successful OpenAI terminal")
        require(not (event.get("type") == "message_delta" and event.get("delta", {}).get("stop_reason") in ("end_turn", "tool_use")), "provider violation emitted successful Anthropic terminal")
        require(not (event.get("type") == "RUN_FINISHED" and event.get("outcome", {}).get("type", "success") == "success"), "provider violation emitted successful AG-UI terminal")
    require(trace(args), "provider violation did not exercise the actual upstream path")
    require(not peer_state(args)["host_effects"], "provider violation executed a host tool")
    return {"provider_violation": scenario, "public_failure": True, "host_effects": 0}


def safety_policy_case(args, protocol):
    configure(args, "host_after_results")
    driver = Driver(args, protocol, args.safety_agent_id)
    first = driver.send()
    require(not failed(first), "current-policy setup failed before caller handoff")
    driver.answer(driver.calls(first))
    require(not peer_state(args)["host_effects"], "host effect executed before caller result submission")
    endpoint = args.base_url.rstrip("/") + "/api/agents/" + urllib.parse.quote(args.safety_agent_id, safe="")
    original_reply = http(endpoint, token=driver.token)
    require(original_reply["status"] == 200, "cannot retain original registered safety agent")
    original = original_reply["body"]
    host_name = json.loads(args.host_tool_json.read_text())["name"]
    require(host_name in original["policy"]["tools"]["allow"], "safety agent did not originally allow the real host tool")
    advertised = [tool.get("function", tool).get("name") for tool in trace(args)[0]["request"].get("tools", [])]
    require(host_name in advertised, "policy scenario never established the initially executable host tool")
    before = len(trace(args))
    try:
        changed = http(endpoint, {"policy": {"tools": {"allow": [], "deny": [host_name]}}}, driver.token, method="PATCH")
        require(changed["status"] == 200, "current-agent policy mutation failed")
        require(changed["body"]["policy"]["tools"]["deny"] == [host_name], "agent policy mutation did not persist the host denial")
        reply = driver.send()
        requests = trace(args)[before:]
        require(requests, "policy continuation did not reach the real provider host-call probe")
        for entry in requests:
            names = [tool.get("function", tool).get("name") for tool in entry["request"].get("tools", [])]
            require(host_name not in names, "continuation advertised a host tool denied by current agent policy")
        require(failed(reply), "malicious provider's denied host call became a successful continuation")
        require(not peer_state(args)["host_effects"], "current agent denial failed to prevent the real MCP effect")
    finally:
        restored = http(endpoint, original, driver.token, method="PUT")
        require(restored["status"] == 200, "failed to restore original safety agent after policy mutation")
        require(restored["body"]["policy"] == original["policy"], "restored safety agent policy differs from original")
    return {"current_agent_policy_reloaded": True, "denied_host_schema_absent": True,
        "malicious_provider_call_refused": True, "host_effects": 0, "original_policy_restored": True}


def safety_tenant_case(args, protocol):
    configure(args, "single")
    driver = Driver(args, protocol, args.safety_agent_id)
    first = driver.send()
    require(not failed(first), "cross-tenant setup did not reach caller handoff")
    driver.answer(driver.calls(first))
    before = len(trace(args))
    reply = driver.send(token=os.environ[args.other_tenant_bearer_token_env])
    require(failed(reply), "different tenant accepted the pending caller batch")
    require(len(trace(args)) == before, "cross-tenant result submission contacted provider")
    require(not peer_state(args)["host_effects"], "cross-tenant result submission executed a host effect")
    return {"different_tenant_rejected": True, "provider_contacts_after_rejection": 0, "host_effects": 0,
        "credential_provenance": "supervisor must attest same subject and distinct signed tenant claims"}


def safety_binding_case(args, protocol):
    configure(args, "host_after_results")
    driver = Driver(args, protocol, args.safety_agent_id)
    first = driver.send()
    require(not failed(first), "binding withdrawal setup failed before caller handoff")
    driver.answer(driver.calls(first))
    host_name = json.loads(args.host_tool_json.read_text())["name"]
    first_names = [tool.get("function", tool).get("name") for tool in trace(args)[0]["request"].get("tools", [])]
    require(host_name in first_names, "binding withdrawal never established live host advertisement")
    endpoint = args.provider_base_url.rstrip("/") + "/_host_binding"
    before = len(trace(args))
    try:
        disabled = http(endpoint, {"enabled": False})
        require(disabled["status"] == 200, "real MCP binding withdrawal failed")
        offset = len(peer_state(args)["binding_requests"])
        reply = driver.send()
        state = peer_state(args)
        observed = state["binding_requests"][offset:]
        require(any(entry.get("method") == "tools/list" and entry["enabled"] is False for entry in observed),
            "continuation did not rediscover the real withdrawn MCP binding")
        require(not any(entry.get("method") == "tools/call" for entry in observed), "continuation attempted the withdrawn MCP executable")
        require(failed(reply), "withdrawn executable binding became a successful continuation")
        require(not state["host_effects"], "withdrawn MCP binding executed a real effect")
        for entry in trace(args)[before:]:
            names = [tool.get("function", tool).get("name") for tool in entry["request"].get("tools", [])]
            require(host_name not in names, "continuation advertised a withdrawn MCP executable")
    finally:
        restored = http(endpoint, {"enabled": True})
        require(restored["status"] == 200, "failed to restore real MCP binding")
    return {"real_mcp_tools_list_refreshed": True, "withdrawn_executable_refused": True,
        "host_effects": 0, "host_binding_restored": True}


def mixed_host_exchange(request, host_name):
    messages = request.get("messages", [])
    for index, message in enumerate(messages):
        if message.get("role") != "assistant":
            continue
        for call in message.get("tool_calls", []):
            if call.get("function", {}).get("name") == host_name:
                for result_index in range(index + 1, len(messages)):
                    result = messages[result_index]
                    if result.get("role") == "tool" and result.get("tool_call_id") == call["id"]:
                        return {"call": call, "result": result, "call_index": index, "result_index": result_index}
    raise AssertionError("actual provider history lacks the ordered real host call/result exchange")


def restart_case(args, resume):
    budget_mode = args.mode.startswith("budget-")
    graph_mode = args.mode.startswith("graph-")
    mixed_crash = args.mode.startswith("mixed-crash-")
    selected_agent = args.budget_agent_id if budget_mode else args.graph_agent_id if graph_mode else args.agent_id
    require(selected_agent, "restart variant requires its configured budget or graph agent")
    if graph_mode:
        require(args.graph_agent_id == "orchestrator-agent" and args.graph_before_marker and args.graph_after_marker, "graph restart requires orchestrator-agent and router/worker markers")
    if not resume:
        configure(args, "mixed" if mixed_crash else "two_round" if budget_mode else "graph" if graph_mode else "single")
        saved = []
        for protocol in args.protocols:
            driver = Driver(args, protocol, selected_agent)
            effects_before = len(peer_state(args)["host_effects"]) if mixed_crash else None
            reply = graph_send(driver) if graph_mode else driver.send()
            require(not failed(reply), "restart seed failed")
            exchange = None
            if mixed_crash:
                peer = peer_state(args)
                require(len(peer["host_effects"]) == effects_before + 1, "mixed crash seed did not settle exactly one new real host effect")
                matching = [entry for entry in peer["requests"] if driver.history[0]["content"] in json.dumps(entry["request"])]
                require(len(matching) == 1, "mixed crash seed did not use exactly one real same-batch provider response")
                response = matching[0].get("response", {})
                host_name = json.loads(args.host_tool_json.read_text())["name"]
                host_calls = [call for call in response.get("tool_calls", []) if call.get("function", {}).get("name") == host_name]
                require(len(host_calls) == 1 and response.get("wire_body"), "mixed crash seed lacks its actual emitted host call/wire receipt")
                effect = peer["host_effects"][effects_before]
                require(json.loads(host_calls[0]["function"]["arguments"]) == effect["arguments"], "emitted mixed host call differs from actual MCP effect arguments")
                require(isinstance(effect.get("mcp_result"), dict), "mixed crash seed lacks the actual MCP result receipt")
                exchange = {"call": host_calls[0], "mcp_result": effect["mcp_result"],
                    "effect_receipt": effect, "provider_response": response}
            driver.answer(driver.calls(reply))
            entry = {key: value for key, value in driver.__dict__.items() if key not in ("args", "token", "last_reply")}
            if mixed_crash:
                entry["mixed_host_exchange"] = exchange
            saved.append(entry)
        state = {"instance": args.instance_identity, "drivers": saved}
        if mixed_crash:
            state.update({"mode": "mixed-crash", "host_effect_receipts": peer_state(args)["host_effects"]})
            require(len(state["host_effect_receipts"]) == len(args.protocols), "mixed crash seed host-effect total mismatch")
        args.restart_state.write_text(json.dumps(state, indent=2))
        args.restart_state.chmod(0o600)
        if mixed_crash:
            return {"seed_ready": True, "pending_batches": len(saved), "mixed_host_effects": len(state["host_effect_receipts"]),
                "host_effect_receipts": state["host_effect_receipts"],
                "next": "Supervisor must SIGKILL and join only its owned candidate, retain peer/config/store/JWT/state, restart, then invoke mixed-crash-resume."}
        return {"pending_batches": len(saved), "next": "Stop and join candidate, restart same durable database and credentials, then invoke restart-resume with a distinct recorded instance identity."}
    saved = json.loads(args.restart_state.read_text())
    require(saved["instance"] != args.instance_identity, "restart requires distinct externally recorded process identity")
    receipts = []
    acp_fresh_runs = []
    mixed_receipts = []
    graph_receipts = []
    if mixed_crash:
        require(saved.get("mode") == "mixed-crash", "mixed crash resume requires its exact mixed seed state")
        require(peer_state(args)["host_effects"] == saved["host_effect_receipts"], "real host effect receipts changed across candidate crash")
    for state in saved["drivers"]:
        driver = Driver(args, state["protocol"])
        driver.__dict__.update(state)
        contacts_before = len(trace(args))
        reply = graph_send(driver) if graph_mode else driver.send()
        if budget_mode:
            require(failed(reply) and "budget" in json.dumps(reply).lower(), "cold continuation did not retain cumulative budget")
            before = len(trace(args))
            rejected = driver.send()
            require(failed(rejected) and len(trace(args)) == before, "cold failed claim retried provider effects")
        else:
            require(not failed(reply), "cold durable result submission failed: " + json.dumps(reply))
            require(not driver.calls(reply), "cold continuation did not complete")
            if mixed_crash:
                resumed = trace(args)[contacts_before:]
                require(len(resumed) == 1, "mixed crash resume did not make exactly one new provider contact")
                request = resumed[0]["request"]
                marker = state["history"][0]["content"]
                require(marker in json.dumps(request), "mixed crash provider request lost the selected conversation marker")
                exchange = mixed_host_exchange(request, json.loads(args.host_tool_json.read_text())["name"])
                result_content = exchange["result"].get("content")
                decoded_result = json.loads(result_content) if isinstance(result_content, str) else result_content
                actual_call = copy.deepcopy(exchange["call"])
                saved_call = copy.deepcopy(state["mixed_host_exchange"]["call"])
                for call in (actual_call, saved_call):
                    call["function"]["arguments"] = json.loads(call["function"]["arguments"])
                require(actual_call == saved_call
                    and decoded_result == state["mixed_host_exchange"]["mcp_result"],
                    "mixed crash continuation changed its settled host call/result")
                messages = request.get("messages", [])
                for result in (message for message in state["history"] if message.get("role") == "tool"):
                    indices = [index for index, message in enumerate(messages) if message.get("role") == "tool"
                        and message.get("tool_call_id") == result["tool_call_id"] and message.get("content") == result["content"]]
                    require(len(indices) == 1 and exchange["result_index"] < indices[0], "mixed crash history lost host-before-caller result ordering")
                require(peer_state(args)["host_effects"] == saved["host_effect_receipts"], "mixed crash continuation repeated a settled host effect")
                mixed_receipts.append({"protocol": driver.protocol, "marker": marker, "host_call_id": exchange["call"]["id"],
                    "retained_host_result": exchange["result"], "provider_contacts": 1, "host_effects_repeated": 0})
            before = len(trace(args))
            again = graph_completed_retry(driver) if graph_mode else driver.send()
            require(not failed(again) and len(trace(args)) == before, "cold completed retry reexecuted")
            if mixed_crash:
                require(peer_state(args)["host_effects"] == saved["host_effect_receipts"], "mixed crash exact retry repeated host effects")
            if graph_mode:
                graph_sequence(args, driver.history[0]["content"])
                require(len(driver.graph_approval_receipts) == 2, "cold graph lost its initial/fresh continuation approval receipts")
                graph_receipts.append({"protocol": driver.protocol, "approvals": driver.graph_approval_receipts})
            elif driver.protocol == "acp" and not mixed_crash:
                old_ids = {call["id"] for message in driver.history for call in message.get("tool_calls", [])}
                marker = "caller-tools-cold-acp-fresh:" + str(uuid.uuid4())
                before_fresh = len(trace(args))
                fresh = driver.rpc(args.acp_path, "runs/create", {"session_id": driver.session, "input": marker})
                require(not failed(fresh), "cold restored ACP session rejected a fresh default-tools run")
                fresh_history = fresh.get("body", {}).get("result", {}).get("client_tools", {}).get("history", [])
                require(any(message.get("role") == "user" and marker in str(message.get("content", "")) for message in fresh_history),
                    "fresh ACP response history replayed the old prompt instead of the new input")
                fresh_calls = driver.calls(fresh)
                require(fresh_calls and all(call["id"].startswith("uar_client_") and call["id"] not in old_ids for call in fresh_calls),
                    "fresh ACP default-tools run replayed old caller handoff IDs")
                fresh_contacts = trace(args)[before_fresh:]
                require(fresh_contacts and any(marker in json.dumps(entry["request"]) for entry in fresh_contacts),
                    "fresh ACP input did not reach a new real provider request")
                acp_fresh_runs.append({"session_id": driver.session, "input_marker": marker,
                    "provider_contacts": len(fresh_contacts), "new_handoff_ids": [call["id"] for call in fresh_calls],
                    "client_tools_omitted": True, "new_prompt_in_response_history": True})
        receipts.append(driver.protocol)
    return {"cold_protocols": receipts, "acp_fresh_default_runs": acp_fresh_runs, "mixed_crash_continuations": mixed_receipts,
        "graph_approvals": graph_receipts,
        "lifecycle_identity": "operator supplied; retain supervisor stop/join/restart evidence separately"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--provider-base-url", required=True)
    parser.add_argument("--bearer-token-env", required=True)
    parser.add_argument("--other-bearer-token-env", required=True)
    parser.add_argument("--other-tenant-bearer-token-env", help="Supervisor-signed JWT for the same subject with a distinct tenant")
    parser.add_argument("--agent-id", required=True)
    parser.add_argument("--openai-model", help="OpenAI model/registered-agent selector; defaults to the selected scenario agent")
    parser.add_argument("--anthropic-model", required=True, help="Provider-qualified model selected for the bound Anthropic agent")
    parser.add_argument("--protocols", nargs="+", choices=PROTOCOLS, default=list(PROTOCOLS))
    parser.add_argument("--a2a-path")
    parser.add_argument("--a2a-metadata-json", type=Path)
    parser.add_argument("--acp-path", default="/acp")
    parser.add_argument("--workspace-id")
    parser.add_argument("--host-tool-json", type=Path)
    parser.add_argument("--budget-agent-id")
    parser.add_argument("--safety-agent-id", help="Dedicated mutable registered agent initially allowing the real host tool")
    parser.add_argument("--budget-provider-usage-json", type=Path, help="Explicit synthetic provider input_tokens/output_tokens; requires separately established R1/R2/L/S reservation proof")
    parser.add_argument("--graph-agent-id")
    parser.add_argument("--graph-before-marker", help="Installed RouterNode system prompt marker")
    parser.add_argument("--graph-after-marker", help="Persisted general-purpose worker system prompt marker")
    parser.add_argument("--mode", choices=("matrix", "safety", "cancellation", "mixed-crash-seed", "mixed-crash-resume", "restart-seed", "restart-resume", "budget", "graph", "budget-restart-seed", "budget-restart-resume", "graph-restart-seed", "graph-restart-resume"), default="matrix")
    parser.add_argument("--restart-state", type=Path)
    parser.add_argument("--instance-identity")
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    require(os.environ[args.bearer_token_env] != os.environ[args.other_bearer_token_env], "cross-owner credentials must differ")
    args.workspace_headers = {"x-uar-workspace-id": args.workspace_id} if args.workspace_id else {}
    args.a2a_metadata = json.loads(args.a2a_metadata_json.read_text()) if args.a2a_metadata_json else {}
    results, gaps = [], []
    def checkpoint(fatal_error=None):
        report = {"mode": args.mode, "results": results, "coverage_gaps": gaps,
            "http_evidence": EVIDENCE, "execution_complete": False,
            "verification": "Incomplete driver execution; retained observations do not establish acceptance."}
        if fatal_error is not None:
            report["fatal_error"] = fatal_error
        args.report.write_text(json.dumps(report, indent=2))
        args.report.chmod(0o600)
    def record(name, work):
        try:
            results.append({"scenario": name, "status": "passed", "observed": work()})
        except (AssertionError, KeyError, ValueError, OSError, urllib.error.URLError) as error:
            results.append({"scenario": name, "status": "failed", "error": str(error)})
        except Exception as error:
            failure = type(error).__name__ + ": " + str(error)
            results.append({"scenario": name, "status": "failed", "error": failure})
            checkpoint(failure)
            raise
        checkpoint()
    if args.mode == "cancellation":
        require(args.host_tool_json, "cancellation requires the actual safe real --host-tool-json")
        for protocol in args.protocols:
            record(protocol + ":explicit-cancellation", lambda protocol=protocol: cancellation_case(args, protocol))
    elif args.mode == "safety":
        require(args.safety_agent_id and args.host_tool_json and args.other_tenant_bearer_token_env, "safety mode requires --safety-agent-id, --host-tool-json and --other-tenant-bearer-token-env")
        require(os.environ[args.other_tenant_bearer_token_env] != os.environ[args.bearer_token_env], "cross-tenant credential must differ from the owner credential")
        for protocol in args.protocols:
            for mutation in ("duplicate_names", "host_collision", "malformed_schema"):
                record(protocol + ":safety:input:" + mutation, lambda protocol=protocol, mutation=mutation: safety_input_case(args, protocol, mutation))
            record(protocol + ":safety:parallel-batch", lambda protocol=protocol: safety_success_case(args, protocol, "parallel"))
            for stream in (False, True):
                record(protocol + ":safety:mixed-text:stream=" + str(stream), lambda protocol=protocol, stream=stream: safety_success_case(args, protocol, "mixed_text", stream))
            for scenario in ("provider_error", "malformed_arguments", "force_parallel"):
                record(protocol + ":safety:provider:" + scenario, lambda protocol=protocol, scenario=scenario: safety_provider_case(args, protocol, scenario))
            record(protocol + ":safety:current-agent-policy", lambda protocol=protocol: safety_policy_case(args, protocol))
            record(protocol + ":safety:cross-tenant", lambda protocol=protocol: safety_tenant_case(args, protocol))
            record(protocol + ":safety:withdrawn-executable", lambda protocol=protocol: safety_binding_case(args, protocol))
    elif "restart-" in args.mode or args.mode.startswith("mixed-crash-"):
        require(args.restart_state and args.instance_identity, "restart modes require --restart-state and --instance-identity")
        if args.mode.startswith("mixed-crash-"):
            require(args.host_tool_json, "mixed crash recovery requires the actual safe real --host-tool-json")
        record(args.mode, lambda: restart_case(args, args.mode.endswith("resume")))
    elif args.mode == "graph":
        require(args.graph_agent_id == "orchestrator-agent" and args.graph_before_marker and args.graph_after_marker, "graph mode requires orchestrator-agent and distinct router/worker prompt markers")
        for protocol in args.protocols:
            def graph(protocol=protocol):
                observed = loop_case(args, protocol, "graph", agent=args.graph_agent_id)
                observed["node_sequence"] = graph_sequence(args, "caller-tools-acceptance:")
                return observed
            record(protocol + ":graph-cursor", graph)
    elif args.mode == "budget":
        require(args.budget_agent_id, "budget mode requires an agent with a source-bound cumulative token ceiling satisfying R1 <= L, R2 <= L, S + R2 > L")
        for protocol in args.protocols:
            def budget(protocol=protocol):
                configure(args, "two_round")
                driver = Driver(args, protocol, args.budget_agent_id)
                first = driver.send()
                require(not failed(first), "budget setup failed before first handoff")
                driver.answer(driver.calls(first))
                second = driver.send()
                require(failed(second), "cumulative budget did not reject resumed model usage")
                require("budget" in json.dumps(second).lower(), "resumed failure did not identify the budget boundary")
                require(len(trace(args)) <= 2, "budget exceeded allowed provider contact count")
                return {"provider_contacts": len(trace(args)), "budget_agent": args.budget_agent_id}
            record(protocol + ":cumulative-budget", budget)
    else:
        for protocol in args.protocols:
            for stream in (False, True):
                record(protocol + ":loop:stream=" + str(stream), lambda protocol=protocol, stream=stream: loop_case(args, protocol, stream=stream))
            record(protocol + ":two-round", lambda protocol=protocol: loop_case(args, protocol, "two_round"))
            for choice in ("none", "required", "named"):
                record(protocol + ":choice:" + choice, lambda protocol=protocol, choice=choice: loop_case(args, protocol, choice=choice))
            record(protocol + ":parallel-false", lambda protocol=protocol: loop_case(args, protocol, "parallel", parallel=False))
            record(protocol + ":simultaneous-identical-results", lambda protocol=protocol: concurrent_case(args, protocol))
            record(protocol + ":disconnect-held-continuation", lambda protocol=protocol: disconnect_case(args, protocol))
            for failure in ("truncated_stream", "invalid_stream"):
                record(protocol + ":upstream:" + failure, lambda protocol=protocol, failure=failure: provider_failure_case(args, protocol, failure))
            for mutation in ("forged", "owner", "partial", "duplicate", "history", "schema", "conflict"):
                record(protocol + ":reject:" + mutation, lambda protocol=protocol, mutation=mutation: rejection_case(args, protocol, mutation))
            if args.host_tool_json:
                record(protocol + ":mixed-host-caller", lambda protocol=protocol: loop_case(args, protocol, "mixed"))
            else:
                gaps.append(protocol + ": mixed host/caller requires --host-tool-json naming a configured safe real host executable and its arguments.")
        gaps += ["Cold restart is a separate two-invocation restart-seed/restart-resume gate with externally retained stop/join evidence.", "Cumulative budget is a separate budget/budget-restart gate with a configured durable budget-limited agent.", "Graph cursor is a separate graph/graph-restart gate with configured node prompt markers."]
    report = {"mode": args.mode, "results": results, "coverage_gaps": gaps, "http_evidence": EVIDENCE, "execution_complete": True,
        "environment": {"base_url": args.base_url, "provider_base_url": args.provider_base_url, "agent_id": args.agent_id},
        "verification": "Executed HTTP observations only; candidate/database provenance and lifecycle evidence are supplied by the operator."}
    args.report.write_text(json.dumps(report, indent=2))
    args.report.chmod(0o600)
    print(json.dumps({"report": str(args.report), "passed": sum(item["status"] == "passed" for item in results), "failed": sum(item["status"] == "failed" for item in results), "coverage_gaps": len(gaps)}))
    raise SystemExit(1 if any(item["status"] == "failed" for item in results) else 2 if gaps else 0)


if __name__ == "__main__":
    main()
