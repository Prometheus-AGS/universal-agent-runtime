"""Concrete mounted protocol adapters for the user-outcome packet.

These are clients of the installed host. They do not create mock drivers or
launch services. All effects are deferred until the exact approved invocation.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import tempfile
import time
from urllib.parse import quote
import uuid

from core import Blocked, Context, Failed, Response, require


def sse(response: Response) -> list[dict]:
    require(response.status == 200, "stream HTTP request failed")
    require("text/event-stream" in response.headers.get("content-type", ""),
            "stream returned a non-SSE response")
    result = []
    for block in re.split(r"\r?\n\r?\n", response.body.decode("utf-8")):
        fields = block.splitlines()
        event = next((line[6:].strip() for line in fields if line.startswith("event:")), "message")
        data = "\n".join(line[5:].lstrip() for line in fields if line.startswith("data:"))
        if not data:
            continue
        if re.search("thinking|reasoning|thought", event, re.I):
            continue
        if data == "[DONE]":
            result.append({"event": event, "data": data})
            continue
        try:
            value = json.loads(data)
        except ValueError as error:
            raise Failed("stream contains malformed JSON data") from error
        if isinstance(value, dict) and re.search("thinking|reasoning|thought", str(value.get("type", "")), re.I):
            continue
        result.append({"event": event, "data": value})
    require(bool(result), "stream contains no events")
    return result


def rpc(ctx: Context, path: str, method: str, params: dict, *, headers: dict | None = None) -> dict:
    request_id = str(uuid.uuid4())
    response = ctx.request("POST", path, {"jsonrpc": "2.0", "id": request_id,
                                          "method": method, "params": params}, headers)
    require(response.status == 200, "JSON-RPC transport failed")
    body = response.json()
    require(body.get("jsonrpc") == "2.0" and body.get("id") == request_id,
            "JSON-RPC response correlation failed")
    require(not body.get("error") and isinstance(body.get("result"), dict), "JSON-RPC operation failed")
    return body["result"]


def verify_run(ctx: Context, run_id: str, session_id: str | None, agent_id: str,
               model: str) -> dict:
    require(bool(run_id), "native run correlation is missing")
    response = ctx.request("GET", "/api/uar/runs/" + quote(run_id, safe=""))
    require(response.status == 200, "native run inspection unavailable")
    value = response.json()
    provider_id, model_id = model.split("/", 1)
    require(value.get("run_id") == run_id and value.get("agent_id") == agent_id
            and value.get("status") == "done", "run did not complete as the intended agent")
    if session_id is not None:
        require(value.get("conversation_id") == session_id, "run context changed")
    require(value.get("effective_model") == {"provider_id": provider_id, "model_id": model_id}
            and bool(value.get("agent_revision")), "effective model/agent revision differs")
    accounting_response = ctx.request("GET", "/api/uar/accounting/calls/" + quote(run_id, safe=""))
    require(accounting_response.status == 200, "real provider accounting unavailable")
    calls = accounting_response.json()
    require(isinstance(calls, list) and bool(calls), "no provider dispatch receipt")
    for call in calls:
        admission = call.get("admission", {})
        identity = admission.get("identity", {})
        require(identity.get("run_id") == run_id and identity.get("owner") and identity.get("call_id"),
                "provider receipt owner/run/call mismatch")
        require(admission.get("model") == model and call.get("state") == "settled"
                and call.get("observed_tokens", 0) > 0 and call.get("reconciliation_evidence"),
                "provider receipt is wrong-model, unsettled or lacks observed usage")
    provenance = ctx.observe("inference", {"run_id": run_id,
        "session_id": value.get("conversation_id"), "qualified_model": model})
    bindings = [ctx.fixture["provider"], *ctx.fixture.get("genuine_bindings", [])]
    expected = next((item for item in bindings if item["qualified_model"] == model), None)
    require(expected is not None and expected.get("kind") == "genuine", "binding is not a genuine fixture")
    require(provenance.get("run_id") == run_id and provenance.get("kind") == "genuine"
            and provenance.get("synthetic") is False and provenance.get("qualified_model") == model
            and provenance.get("session_id") == value.get("conversation_id")
            and provenance.get("base_url") == expected["config"]["base_url"]
            and provenance.get("configuration_digest") and provenance.get("provider_receipt"),
            "genuine dispatch provenance missing")
    return {"inspection": value, "calls": calls, "provenance": provenance}


def session_run(ctx: Context, session: str, previous: set[str]) -> str:
    response = ctx.request("GET", "/api/uar/runs")
    require(response.status == 200, "owner-scoped run listing unavailable")
    rows = response.json()
    require(isinstance(rows, list), "run listing is not an array")
    candidates = [row["run_id"] for row in rows if row.get("conversation_id") == session
                  and row.get("run_id") not in previous]
    require(len(candidates) == 1, "response cannot be bound to one owned native session turn")
    return candidates[0]


def openai_events(events: list[dict], mode: str) -> tuple[str, str]:
    text, run_ids = [], set()
    terminal = 0
    done = 0
    for event in events:
        data = event["data"]
        if data == "[DONE]":
            done += 1
            continue
        require(isinstance(data, dict), "invalid chat event data")
        require(not data.get("error") and event["event"] not in ("error", "agui.error", "agui.cancelled")
                and data.get("type") not in ("RUN_ERROR", "RUN_CANCELLED"), "positive stream emitted a failure")
        if mode in ("openai", "dual") and data.get("object") == "chat.completion.chunk":
            identifier = data.get("id", "")
            require(identifier.startswith("chatcmpl-"), "chat chunk lacks native run identity")
            run_ids.add(identifier.removeprefix("chatcmpl-"))
            for choice in data.get("choices", []):
                fragment = choice.get("delta", {}).get("content")
                if isinstance(fragment, str):
                    text.append(fragment)
                if choice.get("finish_reason") is not None:
                    require(choice["finish_reason"] == "stop", "chat stream ended without success")
                    terminal += 1
        elif mode == "agui" and event["event"] == "agui.message.delta":
            run_ids.add(data["request_id"])
            text.append(data["delta"]["text"])
        elif mode == "agui" and event["event"] == "agui.done":
            run_ids.add(data["request_id"])
            terminal += 1
        elif mode == "agui_spec" and data.get("type") == "TEXT_MESSAGE_CONTENT":
            text.append(data["delta"])
        elif mode == "agui_spec" and data.get("type") == "RUN_STARTED":
            run_ids.add(data["runId"])
        elif mode == "agui_spec" and data.get("type") == "RUN_FINISHED":
            run_ids.add(data["runId"])
            terminal += 1
    require(len(run_ids) == 1 and terminal == 1, "stream requires one correlated successful terminal")
    if mode in ("openai", "dual"):
        require(done == 1, "OpenAI stream lacks one DONE marker")
    return "".join(text), run_ids.pop()


def normalized(events: list[dict]) -> tuple[str, str]:
    fragments, run_ids = [], set()
    terminals = 0
    for item in events:
        data = item["data"]
        require(isinstance(data, dict), "invalid normalized stream event")
        kind, payload = data.get("type"), data.get("data", {})
        require(kind not in ("Error", "Cancelled"), "normalized stream emitted a failure")
        if payload.get("run_id"):
            run_ids.add(payload["run_id"])
        if kind == "ChatDelta":
            fragments.append(payload["text_delta"])
        if kind in ("RunDone", "RunDoneWithUsage"):
            terminals += 1
    require(terminals == 1 and len(run_ids) == 1, "normalized stream lacks a unique successful terminal")
    return "".join(fragments), run_ids.pop()


class MountedClient:
    def __init__(self, ctx: Context, profile: str, agent: str, model: str):
        self.ctx, self.profile, self.agent, self.model = ctx, profile, agent, model
        self.session = str(uuid.uuid4())
        self.task: str | None = None
        self.native_runs: set[str] = set()
        self.history: list[dict] = []
        self.wire_runs: list[str] = []

    def turn(self, text: str) -> dict:
        prefix = self.profile.split(":", 1)[0]
        if prefix in ("chat", "openai"):
            result = self.chat(text)
        elif prefix == "native":
            result = self.native(text)
        elif prefix == "anthropic":
            result = self.anthropic(text)
        elif prefix in ("a2a_legacy", "a2a_v1"):
            result = self.a2a(text)
        elif prefix == "acp":
            result = self.acp(text)
        else:
            raise Blocked("unimplemented protocol profile: " + self.profile)
        require(result["run_id"] not in self.native_runs, "follow-up reused an earlier native run")
        evidence = verify_run(self.ctx, result["run_id"], result.get("session_id"), self.agent, self.model)
        self.native_runs.add(result["run_id"])
        result["evidence"] = evidence
        self.ctx.record("interface-turn", {"profile": self.profile, "agent": self.agent, **result})
        return result

    def chat(self, text: str) -> dict:
        mode = self.profile.split(":")[1]
        path = "/api/chat/completion" if self.profile.startswith("chat:") else "/v1/chat/completions"
        body = {"agent_id": self.agent, "model": self.model, "session_id": self.session,
                "messages": [{"role": "user", "content": text}], "memory_enabled": False,
                "max_completion_tokens": self.ctx.fixture["limits"]["output_tokens"], "stream": mode != "json"}
        if mode != "json":
            body["stream_mode"] = mode
        response = self.ctx.request("POST", path, body)
        if mode == "json":
            require(response.status == 200, "chat request failed")
            data = response.json()
            require(data.get("session_id") == self.session and data.get("model") == self.model
                    and data.get("object") == "chat.completion", "chat envelope identity mismatch")
            choices = data.get("choices", [])
            require(len(choices) == 1 and choices[0].get("finish_reason") == "stop", "chat terminal failed")
            identifier = data.get("id", "")
            require(identifier.startswith("chatcmpl-"), "chat lacks native run correlation")
            content, run_id = choices[0]["message"]["content"], identifier.removeprefix("chatcmpl-")
        else:
            events = sse(response)
            self.ctx.record("stream-events", events)
            content, run_id = openai_events(events, mode)
        return {"content": content, "run_id": run_id, "session_id": self.session}

    def native(self, text: str) -> dict:
        mode = self.profile.split(":")[1]
        response = self.ctx.request("POST", "/api/uar/runs", {"agent_id": self.agent,
            "model": self.model, "session_id": self.session, "input": text})
        require(response.status in (200, 201, 202), "native run admission failed")
        run_id = response.json()["run_id"]
        suffix = "?stream_mode=agui_spec" if mode == "agui_spec" else ""
        stream = self.ctx.request("GET", "/api/uar/runs/" + quote(run_id, safe="") + "/stream" + suffix)
        events = sse(stream)
        self.ctx.record("stream-events", events)
        content, streamed_run = openai_events(events, mode)
        require(streamed_run == run_id, "native stream belongs to another run")
        return {"content": content, "run_id": run_id, "session_id": self.session}

    def anthropic(self, text: str) -> dict:
        mode = self.profile.split(":")[1]
        body = {"model": self.model, "session_id": self.session, "max_tokens": self.ctx.fixture["limits"]["output_tokens"],
                "messages": [*self.history, {"role": "user", "content": text}], "stream": mode == "stream"}
        if not self.profile.endswith(":implicit"):
            body["agent_id"] = self.agent
        response = self.ctx.request("POST", "/v1/messages", body, {"anthropic-version": "2023-06-01"})
        require(response.status == 200, "Anthropic request failed")
        if mode == "json":
            value = response.json()
            require(value.get("stop_reason") == "end_turn" and value.get("model") == self.model,
                    "Anthropic completion did not finish successfully on intended model")
            content = "".join(part["text"] for part in value.get("content", []) if part.get("type") == "text")
            identifier = value.get("id")
        else:
            events = sse(response)
            self.ctx.record("stream-events", events)
            require(not any(item["event"] == "error" or item["data"].get("type") == "error"
                            for item in events if isinstance(item["data"], dict)), "Anthropic stream failed")
            starts = [item["data"]["message"] for item in events if item["event"] == "message_start"]
            stops = [item for item in events if item["event"] == "message_stop"]
            reasons = [item["data"].get("delta", {}).get("stop_reason") for item in events if item["event"] == "message_delta"]
            require(len(starts) == 1 and len(stops) == 1 and reasons == ["end_turn"]
                    and starts[0].get("model") == self.model, "Anthropic stream terminal/model mismatch")
            identifier = starts[0]["id"]
            content = "".join(item["data"]["delta"]["text"] for item in events
                              if item["event"] == "content_block_delta"
                              and item["data"].get("delta", {}).get("type") == "text_delta")
        require(isinstance(identifier, str) and identifier.startswith("msg_"), "Anthropic message identity missing")
        self.history.extend([{"role": "user", "content": text}, {"role": "assistant", "content": content}])
        run_id = session_run(self.ctx, self.session, self.native_runs)
        return {"content": content, "run_id": run_id, "session_id": self.session, "message_id": identifier}

    def a2a(self, text: str) -> dict:
        version, mode = self.profile.split(":")
        v1 = version == "a2a_v1"
        path = ("/a2a/v1/agents/" if v1 else "/a2a/agents/") + quote(self.agent, safe="")
        headers = {"a2a-version": "1.0"} if v1 else {}
        if v1:
            message = {"messageId": str(uuid.uuid4()), "role": "ROLE_USER", "parts": [{"text": text}]}
            if self.task:
                message.update(taskId=self.task, contextId=self.session)
            params = {"message": message, "configuration": {"returnImmediately": False}}
            if mode == "stream":
                request_id = str(uuid.uuid4())
                response = self.ctx.request("POST", path, {"jsonrpc": "2.0", "id": request_id,
                    "method": "SendStreamingMessage", "params": params}, headers)
                events = sse(response)
                self.ctx.record("stream-events", events)
                task_id = None
                completed = False
                for item in events:
                    envelope = item["data"]
                    require(envelope.get("id") == request_id and not envelope.get("error"), "A2A stream failed or correlation differs")
                    payload = envelope.get("result", {})
                    if "task" in payload:
                        current = payload["task"]
                        task_id = current["id"]
                        completed = current["status"]["state"] == "TASK_STATE_COMPLETED"
                        require(current["status"]["state"] not in ("TASK_STATE_FAILED", "TASK_STATE_CANCELED", "TASK_STATE_REJECTED"), "A2A stream task failed")
                    if "statusUpdate" in payload:
                        current = payload["statusUpdate"]
                        require(task_id == current["taskId"], "A2A stream changed task")
                        state = current["status"]["state"]
                        require(state not in ("TASK_STATE_FAILED", "TASK_STATE_CANCELED", "TASK_STATE_REJECTED"), "A2A stream failed")
                        completed = state == "TASK_STATE_COMPLETED"
                require(task_id and completed, "A2A stream lacks successful terminal")
                task = rpc(self.ctx, path, "GetTask", {"id": task_id}, headers=headers)
            else:
                task = rpc(self.ctx, path, "SendMessage", params, headers=headers).get("task", {})
            require(task.get("status", {}).get("state") == "TASK_STATE_COMPLETED", "A2A v1 did not complete")
            context_key = "contextId"
        else:
            params = {"message": {"role": "user", "parts": [{"type": "text", "text": text}]}}
            if self.task:
                params.update(task_id=self.task, context_id=self.session)
            task = rpc(self.ctx, path, "message/send", params)
            for _ in range(self.ctx.fixture["limits"].get("poll_requests", 40)):
                state = task.get("status", {}).get("state")
                require(state not in ("failed", "canceled", "inputrequired"), "A2A legacy task failed or interrupted")
                if state == "completed":
                    break
                self.ctx.remaining()
                time.sleep(0.05)
                task = rpc(self.ctx, path, "tasks/get", {"id": task["id"]})
            require(task.get("status", {}).get("state") == "completed", "A2A legacy poll bound exhausted")
            context_key = "context_id"
        require(task.get("metadata", {}).get("agent_id") == self.agent, "A2A selected wrong agent")
        if self.task:
            require(task.get("id") == self.task and task.get(context_key) == self.session,
                    "A2A follow-up lost task/context")
        self.task, self.session = task["id"], task[context_key]
        message = task.get("status", {}).get("message", {})
        content = "".join(part["text"] for part in message.get("parts", []) if "text" in part)
        return {"content": content, "run_id": task.get("metadata", {}).get("run_id"), "task_id": self.task}

    def acp(self, text: str) -> dict:
        path = self.ctx.fixture.get("interfaces", {}).get("acp_path", "/acp").rstrip("/")
        if not self.task:
            session = rpc(self.ctx, path + "/", "sessions/create", {"agent_id": self.agent})
            self.session, self.task = session["session_id"], session["session_id"]
        params = {"session_id": self.session, "input": text}
        if self.profile.endswith(":stream"):
            response = self.ctx.request("POST", path + "/stream", {"jsonrpc": "2.0", "id": str(uuid.uuid4()),
                                                                       "method": "runs/create", "params": params})
            events = sse(response)
            self.ctx.record("stream-events", events)
            content, run_id = normalized(events)
        else:
            run = rpc(self.ctx, path + "/", "runs/create", params)
            require(run.get("status") == "running" and run.get("run_id"), "ACP run admission failed")
            run_id = run["run_id"]
            response = self.ctx.request("GET", "/api/uar/runs/" + quote(run_id, safe="") + "/stream")
            events = sse(response)
            self.ctx.record("stream-events", events)
            content, streamed = openai_events(events, "agui")
            require(streamed == run_id, "ACP native terminal correlation mismatch")
        return {"content": content, "run_id": run_id, "session_id": self.session}


def run_program(ctx: Context, name: str, inputs: dict) -> dict:
    argv = ctx.backend.get("clients", {}).get(name)
    if not isinstance(argv, list) or not argv:
        raise Blocked("missing concrete installed client command: " + name)
    frozen = set(ctx.fixture["_frozen_paths"])
    for index, argument in enumerate(argv):
        path = Path(argument)
        if index == 0 or (path.is_absolute() and path.is_file()):
            if not path.is_absolute() or str(path.resolve()) not in frozen:
                raise Blocked("client command executable/file input is not frozen")
    with tempfile.TemporaryDirectory(prefix="uar-outcome-client-") as directory:
        output = Path(directory) / "receipt.json"
        environment = {"UAR_ACCEPTANCE_CLIENT_INPUT": json.dumps(inputs),
                       "UAR_ACCEPTANCE_CLIENT_OUTPUT": str(output)}
        if name in ("grpc_legacy", "grpc_v1"):
            environment["UAR_ACCEPTANCE_GRPC_VERSION"] = "v1" if name == "grpc_v1" else "legacy"
        process = ctx.owned_process(argv, environment)
        ctx.wait_process(process, min(ctx.remaining(), ctx.fixture["limits"]["command_seconds"]))
        ctx.stop_process(process)
        if not output.is_file():
            raise Failed("concrete client produced no receipt")
        raw = output.read_bytes()
        require(len(raw) <= ctx.fixture["limits"]["response_bytes"], "client receipt exceeded byte bound")
        value = json.loads(raw)
        ctx.record("installed-client-receipt", {"client": name, "receipt": value, "returncode": process.returncode})
        if value.get("status") == "BLOCKED":
            raise Blocked("installed client prerequisite unavailable; see retained receipt: " + name)
        require(process.returncode == 0 and value.get("status") == "PASS", "installed client failed")
        ctx.release_process(process)
        return value
