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
import http.client
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


def http(url, body=None, token=None, headers=None):
    if getattr(TRANSPORT, "capture", False):
        raise CapturedRequest(url, body, token, headers)
    values = {"Content-Type": "application/json", **(headers or {})}
    if token:
        values["Authorization"] = "Bearer " + token
    request = urllib.request.Request(url, data=None if body is None else json.dumps(body).encode(), headers=values)
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            raw = response.read().decode()
            status, content_type = response.status, response.headers.get_content_type()
    except urllib.error.HTTPError as error:
        raw = error.read().decode()
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
        self.message_ids = {}

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
                "tools": [{"name": tool["function"]["name"], "description": tool["function"]["description"], "input_schema": SCHEMA, "strict": True} for tool in self.tools],
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
                "tools": [{"name": tool["function"]["name"], "description": tool["function"]["description"], "parameters": SCHEMA} for tool in self.tools],
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
                    require(snapshots, "A2A stream missing initial task snapshot")
                    task = snapshots[0]
                else:
                    task = reply["body"]["result"].get("task", reply["body"]["result"])
                self.task, self.context = task["id"], task["contextId"]
                if stream:
                    current = self.rpc(endpoint, "GetTask", {"id": self.task}, token, {"A2A-Version": "1.0"})
                    require(not failed(current), "A2A stream terminal task read failed")
                    task = current["body"]["result"]
                self.task, self.context = task["id"], task["contextId"]
                deadline = time.monotonic() + 150
                while task["status"]["state"] not in ("TASK_STATE_INPUT_REQUIRED", "TASK_STATE_COMPLETED", "TASK_STATE_FAILED", "TASK_STATE_CANCELED"):
                    require(time.monotonic() < deadline, "A2A terminal deadline expired")
                    time.sleep(0.2)
                    reply = self.rpc(endpoint, "GetTask", {"id": self.task}, token, {"A2A-Version": "1.0"})
                    require(not failed(reply), "A2A task read failed")
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

    def answer(self, calls, value="fixture-result"):
        require(calls, "expected caller tool requests")
        require(all(call["id"].startswith("uar_client_") for call in calls), "caller IDs lack durable host-issued locator")
        require(all(call["function"]["name"] in ("test_lookup", "test_second") for call in calls), "host executable leaked as caller tool")
        for call in calls:
            arguments = json.loads(call["function"]["arguments"])
            require(isinstance(arguments.get("query"), str), "caller arguments lost declared schema")
        self.history += [{"role": "assistant", "content": "", "tool_calls": copy.deepcopy(calls)}]
        self.history += [{"role": "tool", "tool_call_id": call["id"], "content": value + ":" + str(index)} for index, call in enumerate(calls)]
        self.acp_rpc_id = None


def configure(args, scenario):
    settings = {"scenario": scenario}
    if scenario == "graph":
        require(args.graph_before_marker and args.graph_after_marker, "graph scenario requires distinct node prompt markers")
        settings["graph_after_marker"] = args.graph_after_marker
    if args.host_tool_json:
        settings["host_tool"] = json.loads(args.host_tool_json.read_text())
    reply = http(args.provider_base_url.rstrip("/") + "/_control", settings)
    require(reply["status"] == 200, "controlled provider configuration failed")


def trace(args):
    return http(args.provider_base_url.rstrip("/") + "/_trace")["body"]["requests"]


def loop_case(args, protocol, scenario="single", choice="auto", parallel=True, stream=False, agent=None):
    configure(args, scenario)
    driver = Driver(args, protocol, agent)
    driver.choice, driver.parallel = choice, parallel
    first = driver.send(stream=stream)
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
    captured_request = trace(args)[0]["request"]
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
        reply = driver.send(stream=stream)
        require(not failed(reply), "result submission rejected: " + json.dumps(reply))
        calls = driver.calls(reply)
    contacts = len(trace(args))
    require(contacts >= 2, "result continuation did not reach real provider")
    before = contacts
    repeated = driver.send(stream=stream)
    require(not failed(repeated), "identical completed result retry rejected")
    require(not driver.calls(repeated), "completed result retry reissued tool calls")
    require(len(trace(args)) == before, "completed result retry executed provider again")
    if scenario == "mixed":
        peer = http(args.provider_base_url.rstrip("/") + "/_trace")["body"]
        require(len(peer["host_effects"]) == 1, "real MCP host effect did not execute exactly once across completed retry")
    captured = json.dumps(trace(args))
    require("fixture-result" in captured, "provider did not receive caller tool result")
    require(driver.history[0]["content"] in captured and driver.history[1]["content"] in captured, "provider lost earlier full history")
    return {"provider_contacts": contacts, "handoffs": handoffs, "retry_provider_contacts": 0}


def rejection_case(args, protocol, mutation):
    configure(args, "parallel")
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
    return {"rejected": mutation, "provider_contacts_after_rejection": 0}


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
    connection_type = http.client.HTTPSConnection if target.scheme == "https" else http.client.HTTPConnection
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


def graph_sequence(args, marker):
    requests = [entry["request"] for entry in trace(args) if marker in json.dumps(entry["request"])]
    sequence = []
    for request in requests:
        serialized = json.dumps(request)
        if args.graph_after_marker in serialized:
            sequence.append("after")
        elif args.graph_before_marker in serialized:
            sequence.append("before")
        else:
            raise AssertionError("graph provider request lost both configured node markers")
    require(sequence == ["before", "before", "after"], "graph cursor replayed or skipped a node: " + json.dumps(sequence))
    return sequence


def restart_case(args, resume):
    budget_mode = args.mode.startswith("budget-")
    graph_mode = args.mode.startswith("graph-")
    selected_agent = args.budget_agent_id if budget_mode else args.graph_agent_id if graph_mode else args.agent_id
    require(selected_agent, "restart variant requires its configured budget or graph agent")
    if graph_mode:
        require(args.graph_before_marker and args.graph_after_marker, "graph restart requires node markers")
    if not resume:
        configure(args, "two_round" if budget_mode else "graph" if graph_mode else "single")
        saved = []
        for protocol in args.protocols:
            driver = Driver(args, protocol, selected_agent)
            reply = driver.send()
            require(not failed(reply), "restart seed failed")
            driver.answer(driver.calls(reply))
            saved.append({key: value for key, value in driver.__dict__.items() if key not in ("args", "token", "last_reply")})
        args.restart_state.write_text(json.dumps({"instance": args.instance_identity, "drivers": saved}, indent=2))
        args.restart_state.chmod(0o600)
        return {"pending_batches": len(saved), "next": "Stop and join candidate, restart same durable database and credentials, then invoke restart-resume with a distinct recorded instance identity."}
    saved = json.loads(args.restart_state.read_text())
    require(saved["instance"] != args.instance_identity, "restart requires distinct externally recorded process identity")
    receipts = []
    for state in saved["drivers"]:
        driver = Driver(args, state["protocol"])
        driver.__dict__.update(state)
        reply = driver.send()
        if budget_mode:
            require(failed(reply) and "budget" in json.dumps(reply).lower(), "cold continuation did not retain cumulative budget")
            before = len(trace(args))
            rejected = driver.send()
            require(failed(rejected) and len(trace(args)) == before, "cold failed claim retried provider effects")
        else:
            require(not failed(reply), "cold durable result submission failed: " + json.dumps(reply))
            require(not driver.calls(reply), "cold continuation did not complete")
            before = len(trace(args))
            again = driver.send()
            require(not failed(again) and len(trace(args)) == before, "cold completed retry reexecuted")
            if graph_mode:
                graph_sequence(args, driver.history[0]["content"])
        receipts.append(driver.protocol)
    return {"cold_protocols": receipts, "lifecycle_identity": "operator supplied; retain supervisor stop/join/restart evidence separately"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--provider-base-url", required=True)
    parser.add_argument("--bearer-token-env", required=True)
    parser.add_argument("--other-bearer-token-env", required=True)
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
    parser.add_argument("--graph-agent-id")
    parser.add_argument("--graph-before-marker")
    parser.add_argument("--graph-after-marker")
    parser.add_argument("--mode", choices=("matrix", "restart-seed", "restart-resume", "budget", "graph", "budget-restart-seed", "budget-restart-resume", "graph-restart-seed", "graph-restart-resume"), default="matrix")
    parser.add_argument("--restart-state", type=Path)
    parser.add_argument("--instance-identity")
    parser.add_argument("--report", required=True, type=Path)
    args = parser.parse_args()
    require(os.environ[args.bearer_token_env] != os.environ[args.other_bearer_token_env], "cross-owner credentials must differ")
    args.workspace_headers = {"x-uar-workspace-id": args.workspace_id} if args.workspace_id else {}
    args.a2a_metadata = json.loads(args.a2a_metadata_json.read_text()) if args.a2a_metadata_json else {}
    results, gaps = [], []
    def record(name, work):
        try:
            results.append({"scenario": name, "status": "passed", "observed": work()})
        except (AssertionError, KeyError, ValueError, OSError, urllib.error.URLError) as error:
            results.append({"scenario": name, "status": "failed", "error": str(error)})
    if "restart-" in args.mode:
        require(args.restart_state and args.instance_identity, "restart modes require --restart-state and --instance-identity")
        record(args.mode, lambda: restart_case(args, args.mode.endswith("resume")))
    elif args.mode == "graph":
        require(args.graph_agent_id and args.graph_before_marker and args.graph_after_marker, "graph mode requires a two-model-node graph agent and distinct configured node prompt markers")
        for protocol in args.protocols:
            def graph(protocol=protocol):
                observed = loop_case(args, protocol, "graph", agent=args.graph_agent_id)
                observed["node_sequence"] = graph_sequence(args, "caller-tools-acceptance:")
                return observed
            record(protocol + ":graph-cursor", graph)
    elif args.mode == "budget":
        require(args.budget_agent_id, "budget mode requires an agent configured with a cumulative token ceiling below 10 tokens")
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
    report = {"mode": args.mode, "results": results, "coverage_gaps": gaps, "http_evidence": EVIDENCE,
        "environment": {"base_url": args.base_url, "provider_base_url": args.provider_base_url, "agent_id": args.agent_id},
        "verification": "Executed HTTP observations only; candidate/database provenance and lifecycle evidence are supplied by the operator."}
    args.report.write_text(json.dumps(report, indent=2))
    args.report.chmod(0o600)
    print(json.dumps({"report": str(args.report), "passed": sum(item["status"] == "passed" for item in results), "failed": sum(item["status"] == "failed" for item in results), "coverage_gaps": len(gaps)}))
    raise SystemExit(1 if any(item["status"] == "failed" for item in results) else 2 if gaps else 0)


if __name__ == "__main__":
    main()
