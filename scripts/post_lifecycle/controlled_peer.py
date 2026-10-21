#!/usr/bin/env python3
"""Unexecuted bounded loopback protocol fixture; never a provider-exact oracle.

Owns no UAR settings, credentials or store. The separately approved boundary
runner owns launch, readiness, release and joined process exit. Only counters,
case labels and digests are recorded; request headers/content are not logged.
"""
import argparse
import collections
import hashlib
import http.server
import json
import math
import os
from pathlib import Path
import re
import signal
import socket
import threading
import time

LABEL = re.compile(r"^[A-Za-z0-9_-]{1,64}$")
CASE = re.compile(r"UAR_CASE:([A-Za-z0-9_-]{1,64})")
HOLD = re.compile(r"UAR_HOLD:([A-Za-z0-9_-]{1,64}):")
MODES = {"success", "provider_failure", "disconnect", "tool", "child"}


class State:
    def __init__(self, args):
        self.args = args
        self.condition = threading.Condition()
        self.cases = {}
        self.held = {}
        self.counts = collections.Counter()
        self.case_counts = {}
        self.stopping = False
        self.sequence = 0
        self.log_lock = threading.Lock()

    def record(self, event, **fields):
        with self.log_lock:
            self.sequence += 1
            row = {"sequence": self.sequence, "event": event,
                   "monotonic": time.monotonic(), **fields}
            with open(self.args.receipt_file, "a", encoding="utf-8") as output:
                output.write(json.dumps(row, sort_keys=True) + "\n")

    def snapshot(self):
        with self.condition:
            return {"kind": "uar.controlled-peer-status/v1",
                    "counts": {key: self.counts[key] for key in (
                        "model_requests", "embedding_requests",
                        "embedding_inputs", "embedding_responses")},
                    "cases": {key: {field: value[field] for field in (
                        "model_requests", "tool_intents")}
                              for key, value in self.case_counts.items()},
                    "held": {key: dict(value) for key, value in self.held.items()},
                    "stopping": self.stopping}


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"
    server_version = "UARControlledPeer/1"
    timeout = 5.0

    def log_message(self, *_args):
        pass

    @property
    def state(self):
        return self.server.state

    def reply(self, status, body, content_type="application/json", headers=None):
        data = body if isinstance(body, bytes) else json.dumps(body).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        for name, value in (headers or {}).items():
            self.send_header(name, value)
        self.end_headers()
        try:
            self.wfile.write(data)
            self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError):
            self.state.record("client_disconnect")

    def read_json(self):
        length = int(self.headers.get("Content-Length", "0"))
        if not 0 < length <= 1024 * 1024:
            raise ValueError("content length out of fixture bounds")
        raw = self.rfile.read(length)
        if len(raw) != length:
            raise ValueError("incomplete request")
        value = json.loads(raw)
        if not isinstance(value, dict):
            raise ValueError("expected object")
        return value

    def do_GET(self):
        if self.path == "/__uar_fixture/status":
            self.reply(200, self.state.snapshot())
        else:
            self.reply(404, {"error": "unknown fixture route"})

    def do_POST(self):
        try:
            body = self.read_json()
            if self.path == "/__uar_fixture/control":
                self.control(body)
            elif self.path in ("/v1/embeddings", "/embeddings"):
                self.embeddings(body)
            elif self.path in ("/v1/chat/completions", "/chat/completions"):
                self.chat(body)
            else:
                self.reply(404, {"error": "unknown fixture route"})
        except (ValueError, TypeError, KeyError, OverflowError):
            self.reply(400, {"error": "invalid fixture request"})

    def control(self, body):
        action = body.get("action")
        with self.state.condition:
            if action == "configure_chat":
                case = body["case"]
                mode = body["mode"]
                if not LABEL.fullmatch(case) or mode not in MODES:
                    raise ValueError("invalid case/mode")
                arguments = body.get("arguments", {})
                if not isinstance(arguments, dict):
                    raise ValueError("invalid tool arguments")
                self.state.cases[case] = {"mode": mode, "tool": body.get("tool"),
                                          "arguments": arguments,
                                          "wait_tool": body.get("wait_tool")}
                self.state.case_counts.setdefault(case, collections.Counter())
                result = {"configured": case, "mode": mode}
            elif action == "arm_embeddings":
                group = body["group"]
                expected = body["expected"]
                if (not LABEL.fullmatch(group) or type(expected) is not int
                        or not 1 <= expected <= 8 or group in self.state.held):
                    raise ValueError("invalid or reused hold group")
                self.state.held[group] = {"expected": expected, "arrived": 0,
                                          "released": False, "expired": 0}
                result = {"armed": group, "expected": expected}
            elif action == "release_embeddings":
                group = body["group"]
                self.state.held[group]["released"] = True
                self.state.condition.notify_all()
                result = {"released": group}
            else:
                raise ValueError("unsupported control action")
        self.state.record("control", action=action)
        self.reply(200, result)

    def embeddings(self, body):
        inputs = body["input"]
        if isinstance(inputs, str):
            inputs = [inputs]
        if not isinstance(inputs, list) or not all(isinstance(text, str) for text in inputs):
            raise ValueError("invalid embedding input")
        dimensions = body.get("dimensions", self.state.args.dimensions)
        if dimensions != self.state.args.dimensions:
            self.reply(400, {"error": "fixture dimension mismatch"})
            return
        groups = sorted({match.group(1) for text in inputs for match in HOLD.finditer(text)})
        with self.state.condition:
            self.state.counts["embedding_requests"] += 1
            self.state.counts["embedding_inputs"] += len(inputs)
            for group in groups:
                if group not in self.state.held:
                    self.reply(409, {"error": "embedding hold group not armed"})
                    return
                self.state.held[group]["arrived"] += 1
            self.state.condition.notify_all()
        self.state.record("embedding_arrived", groups=groups, inputs=len(inputs))
        deadline = time.monotonic() + self.state.args.hold_seconds
        with self.state.condition:
            while any(not self.state.held[group]["released"] for group in groups):
                remaining = deadline - time.monotonic()
                if remaining <= 0 or self.state.stopping:
                    for group in groups:
                        if not self.state.held[group]["released"]:
                            self.state.held[group]["expired"] += 1
                    self.reply(504, {"error": "bounded embedding hold expired"})
                    self.state.record("embedding_hold_expired", groups=groups)
                    return
                self.state.condition.wait(remaining)
        data = []
        for index, text in enumerate(inputs):
            vector = [0.0] * dimensions
            for word in re.findall(r"\w+", text.casefold()):
                digest = hashlib.sha256(word.encode("utf-8")).digest()
                vector[int.from_bytes(digest[:4], "big") % dimensions] += 1.0
            norm = math.sqrt(sum(value * value for value in vector))
            if norm == 0:
                vector[0] = 1.0
            else:
                vector = [value / norm for value in vector]
            data.append({"object": "embedding", "index": index, "embedding": vector})
        with self.state.condition:
            self.state.counts["embedding_responses"] += 1
        self.state.record("embedding_replied", inputs=len(inputs), groups=groups)
        self.reply(200, {"object": "list", "data": data, "model": body.get("model", "fixture"),
                         "usage": {"prompt_tokens": len(inputs), "total_tokens": len(inputs)}})

    def chat(self, body):
        messages = body["messages"]
        if not isinstance(messages, list):
            raise ValueError("messages must be an array")
        # The marker lives in the owned fixture prompt, never in auth/settings.
        found = []
        for message in reversed(messages):
            if message.get("role") == "user":
                found = CASE.findall(json.dumps(message.get("content"), ensure_ascii=False))
                if found:
                    break
        case = found[-1] if found else "default"
        with self.state.condition:
            settings = dict(self.state.cases.get(case, {"mode": "success"}))
            counts = self.state.case_counts.setdefault(case, collections.Counter())
            counts["model_requests"] += 1
            self.state.counts["model_requests"] += 1
        mode = settings["mode"]
        self.state.record("model_request", case=case, mode=mode)
        if mode == "provider_failure":
            self.reply(503, {"error": {"type": "server_error", "code": "fixture_provider_failure",
                                        "message": "controlled unrelated provider failure"}})
            return
        model = body.get("model", "fixture")
        delta = {"role": "assistant", "content": "controlled-success:" + case}
        finish = "stop"
        tool_results = [message for message in messages if message.get("role") == "tool"]
        has_tool_result = bool(tool_results)
        should_wait = mode == "child" and len(tool_results) == 1 and settings.get("wait_tool") is not None
        if mode in ("tool", "child") and (not has_tool_result or should_wait):
            tool = settings.get("wait_tool") if should_wait else settings.get("tool")
            advertised = {entry.get("function", {}).get("name") for entry in body.get("tools", [])}
            if not isinstance(tool, str) or tool not in advertised:
                self.reply(422, {"error": {"code": "fixture_tool_not_advertised"}})
                return
            arguments = dict(settings.get("arguments", {}))
            if should_wait:
                def thread_identity(value):
                    if isinstance(value, str):
                        try:
                            return thread_identity(json.loads(value))
                        except (ValueError, TypeError):
                            return None
                    if isinstance(value, dict):
                        if isinstance(value.get("thread_id"), str):
                            return value["thread_id"]
                        for nested in value.values():
                            identity = thread_identity(nested)
                            if identity is not None:
                                return identity
                    if isinstance(value, list):
                        for nested in value:
                            identity = thread_identity(nested)
                            if identity is not None:
                                return identity
                    return None
                child_thread = thread_identity(tool_results[0].get("content"))
                if child_thread is None:
                    self.reply(422, {"error": {"code": "fixture_child_ack_missing_identity"}})
                    return
                arguments = {"thread_ids": [child_thread], "timeout_ms": 1000}
            delta = {"role": "assistant", "tool_calls": [{"index": 0, "id": "call-" + case,
                     "type": "function", "function": {"name": tool,
                         "arguments": json.dumps(arguments, separators=(",", ":"))}}]}
            finish = "tool_calls"
            with self.state.condition:
                counts["tool_intents"] += 1
            self.state.record("tool_intent", case=case, tool=tool)
        if body.get("stream") is not True:
            if mode == "disconnect":
                self.reply(400, {"error": {"code": "fixture_disconnect_requires_stream"}})
                return
            message = dict(delta)
            if "tool_calls" in message:
                for call in message["tool_calls"]:
                    call.pop("index", None)
            self.reply(200, {"id": "fixture-" + case, "object": "chat.completion", "created": 1,
                             "model": model, "choices": [{"index": 0, "message": message,
                             "finish_reason": finish}], "usage": {"prompt_tokens": 1,
                             "completion_tokens": 1, "total_tokens": 2}})
            return
        def frame(value, reason=None, usage=None):
            packet = {"id": "fixture-" + case, "object": "chat.completion.chunk", "created": 1,
                      "model": model, "choices": [{"index": 0, "delta": value, "finish_reason": reason}]}
            if usage is not None:
                packet["usage"] = usage
            return ("data: " + json.dumps(packet, separators=(",", ":")) + "\n\n").encode("utf-8")
        first = frame(delta)
        if mode == "disconnect":
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Content-Length", str(len(first) + 128))
            self.end_headers()
            try:
                self.wfile.write(first)
                self.wfile.flush()
                self.connection.shutdown(socket.SHUT_RDWR)
            except (BrokenPipeError, ConnectionResetError, OSError):
                pass
            self.close_connection = True
            self.state.record("upstream_truncated", case=case)
            return
        output = first + frame({}, finish, {"prompt_tokens": 1, "completion_tokens": 1,
                                           "total_tokens": 2}) + b"data: [DONE]\n\n"
        self.reply(200, output, "text/event-stream")
        self.state.record("model_replied", case=case, terminal=finish)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--port-file", required=True)
    parser.add_argument("--receipt-file", required=True)
    parser.add_argument("--dimensions", type=int, required=True)
    parser.add_argument("--hold-seconds", type=float, default=20.0)
    args = parser.parse_args()
    if not 0 <= args.port <= 65535 or not 1 <= args.dimensions <= 65536 or not 0 < args.hold_seconds <= 45:
        parser.error("invalid bounded fixture settings")
    if Path(args.port_file).exists() or Path(args.receipt_file).exists():
        parser.error("fixture output already exists; preserve previous evidence")
    Path(args.receipt_file).parent.mkdir(parents=True, exist_ok=True)
    Path(args.receipt_file).touch(exist_ok=False)
    server = http.server.ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.daemon_threads = False
    server.block_on_close = True
    server.state = State(args)
    def stop(_signum, _frame):
        with server.state.condition:
            server.state.stopping = True
            server.state.condition.notify_all()
        threading.Thread(target=server.shutdown, daemon=True).start()
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    server.state.record("started", host="127.0.0.1", port=server.server_port,
                        dimensions=args.dimensions, hold_seconds=args.hold_seconds)
    port_file = Path(args.port_file)
    port_file.parent.mkdir(parents=True, exist_ok=True)
    pending = port_file.with_name(port_file.name + ".pending")
    with pending.open("x", encoding="utf-8") as output:
        output.write(str(server.server_port) + "\n")
    os.replace(pending, port_file)
    try:
        server.serve_forever(poll_interval=0.2)
    finally:
        with server.state.condition:
            server.state.stopping = True
            server.state.condition.notify_all()
        server.server_close()
        server.state.record("stopped", status=server.state.snapshot())


if __name__ == "__main__":
    main()
