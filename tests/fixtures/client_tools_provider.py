#!/usr/bin/env python3
"""Real HTTP provider collaborator for caller-tool acceptance; never a runtime mock.

Launch separately, configure the candidate's selected provider endpoint to this
listener, and give the acceptance driver its base URL. Control/trace endpoints
contain synthetic fixture data only. This process never starts the candidate.
"""
import argparse
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def frame(value, event=None):
    prefix = "event: " + event + "\n" if event else ""
    return (prefix + "data: " + json.dumps(value, separators=(",", ":")) + "\n\n").encode()


def tool_results(messages):
    results = []
    for message in messages:
        if message.get("role") == "tool" and message.get("tool_call_id", "").startswith("uar_client_"):
            results.append(message.get("content"))
        content = message.get("content")
        if isinstance(content, list):
            results.extend(part.get("content") for part in content if part.get("type") == "tool_result" and part.get("tool_use_id", "").startswith("uar_client_"))
    return results


def produce(request, settings):
    tools = request.get("tools", [])
    names = [tool.get("function", tool).get("name") for tool in tools]
    names = [name for name in names if name in ("test_lookup", "test_second")]
    choice = request.get("tool_choice", "auto")
    choice_type = choice.get("type") if isinstance(choice, dict) else choice
    named = choice.get("function", {}).get("name", choice.get("name")) if isinstance(choice, dict) else None
    results = tool_results(request.get("messages", []))
    scenario = settings.get("scenario", "single")
    if scenario == "graph" and settings.get("graph_after_marker") in json.dumps(request):
        return [], "client-tools-graph-after-node-complete"
    if scenario == "provider_error":
        return None, None
    if choice_type == "none" or not names:
        return [], "client-tools-final:no-call"
    required_results = 2 if scenario in ("two_round", "parallel", "force_parallel") else 1
    if len(results) >= required_results:
        return [], "client-tools-final:" + json.dumps(results, sort_keys=True)
    count = 2 if scenario in ("parallel", "force_parallel") and not results else 1
    disabled = request.get("parallel_tool_calls") is False or (isinstance(choice, dict) and choice.get("disable_parallel_tool_use") is True)
    if disabled and scenario != "force_parallel":
        count = 1
    chosen = named or ("test_second" if results and "test_second" in names else names[0])
    calls = [{"id": "fixture-call-" + str(len(results) + index), "type": "function", "function": {
        "name": (names[index % len(names)] if count > 1 else chosen),
        "arguments": '{"query":' if scenario == "malformed_arguments" else json.dumps({"query": "fixture-" + str(len(results) + index)})
    }} for index in range(count)]
    host_result = any(message.get("role") == "tool" and message.get("tool_call_id") == "fixture-host-call" for message in request.get("messages", []))
    if scenario == "mixed" and not results and not host_result:
        host = settings.get("host_tool")
        if not host or not any(tool.get("function", tool).get("name") == host["name"] for tool in tools):
            raise ValueError("mixed scenario requires an advertised host_tool name and arguments")
        calls.insert(0, {"id": "fixture-host-call", "type": "function", "function": {
            "name": host["name"], "arguments": json.dumps(host["arguments"])}})
    return calls, ""


def openai(request, calls, text):
    model = request.get("model", "fixture-model")
    message = {"role": "assistant", "content": text or None}
    if calls:
        message["tool_calls"] = calls
    usage = {"prompt_tokens": 2, "completion_tokens": 3, "total_tokens": 5}
    if not request.get("stream"):
        return json.dumps({"id": "fixture-completion", "object": "chat.completion", "model": model,
            "choices": [{"index": 0, "message": message, "finish_reason": "tool_calls" if calls else "stop"}], "usage": usage}).encode(), "application/json"
    def chunk(delta, finish=None):
        return frame({"id": "fixture-completion", "object": "chat.completion.chunk", "created": 0, "model": model,
            "choices": [{"index": 0, "delta": delta, "finish_reason": finish}]})
    body = chunk({"role": "assistant"})
    if calls:
        for index, call in enumerate(calls):
            arguments = call["function"]["arguments"]
            middle = len(arguments) // 2
            body += chunk({"tool_calls": [{"index": index, "id": call["id"], "type": "function",
                "function": {"name": call["function"]["name"], "arguments": arguments[:middle]}}]})
            body += chunk({"tool_calls": [{"index": index, "function": {"arguments": arguments[middle:]}}]})
    else:
        body += chunk({"content": text})
    body += chunk({}, "tool_calls" if calls else "stop")
    body += frame({"id": "fixture-completion", "object": "chat.completion.chunk", "model": model, "choices": [], "usage": usage})
    return body + b"data: [DONE]\n\n", "text/event-stream"


def anthropic(request, calls, text):
    blocks = [{"type": "tool_use", "id": call["id"], "name": call["function"]["name"],
        "input": json.loads(call["function"]["arguments"])} for call in calls] if calls else [{"type": "text", "text": text}]
    message = {"id": "fixture-message", "type": "message", "role": "assistant", "model": request.get("model"),
        "content": blocks, "stop_reason": "tool_use" if calls else "end_turn", "stop_sequence": None,
        "usage": {"input_tokens": 2, "output_tokens": 3}}
    if not request.get("stream"):
        return json.dumps(message).encode(), "application/json"
    start = dict(message, content=[], stop_reason=None)
    body = frame({"type": "message_start", "message": start}, "message_start")
    for index, block in enumerate(blocks):
        initial = dict(block, input={}) if calls else {"type": "text", "text": ""}
        body += frame({"type": "content_block_start", "index": index, "content_block": initial}, "content_block_start")
        delta = {"type": "input_json_delta", "partial_json": json.dumps(block["input"])} if calls else {"type": "text_delta", "text": text}
        body += frame({"type": "content_block_delta", "index": index, "delta": delta}, "content_block_delta")
        body += frame({"type": "content_block_stop", "index": index}, "content_block_stop")
    body += frame({"type": "message_delta", "delta": {"stop_reason": message["stop_reason"], "stop_sequence": None}, "usage": {"output_tokens": 3}}, "message_delta")
    return body + frame({"type": "message_stop"}, "message_stop"), "text/event-stream"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--port-file", required=True, type=Path)
    parser.add_argument("--trace", required=True, type=Path)
    args = parser.parse_args()
    lock = threading.Lock()
    settings, traces, effects = {"scenario": "single"}, [], []
    release = threading.Event()
    release.set()
    held = {"count": 0}
    args.trace.touch(mode=0o600)
    args.trace.chmod(0o600)

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_args):
            return

        def reply(self, status, body, content_type="application/json"):
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            if self.path == "/mcp":
                self.reply(405, b'{"error":"stateless MCP uses POST"}')
                return
            with lock:
                value = {"settings": dict(settings), "requests": list(traces), "host_effects": list(effects), "held_requests": held["count"]} if self.path == "/_trace" else {"object": "list", "data": [{"id": "fixture-model", "object": "model"}]}
            self.reply(200, json.dumps(value).encode())

        def do_POST(self):
            request = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
            with lock:
                if self.path == "/_hold":
                    release.clear()
                    self.reply(200, b'{"held":true}')
                    return
                if self.path == "/_release":
                    release.set()
                    self.reply(200, b'{"released":true}')
                    return
                if self.path == "/_control":
                    settings.clear()
                    settings.update(request)
                    traces.clear()
                    effects.clear()
                    self.reply(200, b'{"configured":true}')
                    return
                if self.path == "/mcp":
                    method, identifier = request.get("method"), request.get("id")
                    if method == "notifications/initialized":
                        self.reply(202, b"")
                        return
                    if method == "initialize":
                        result = {"protocolVersion": request.get("params", {}).get("protocolVersion", "2025-03-26"),
                            "capabilities": {"tools": {"listChanged": False}}, "serverInfo": {"name": "client-tools-acceptance", "version": "1.0"}}
                    elif method == "tools/list":
                        result = {"tools": [{"name": "effect", "description": "Record one acceptance effect", "inputSchema": {
                            "type": "object", "properties": {"query": {"type": "string"}}, "required": ["query"], "additionalProperties": False}}]}
                    elif method == "tools/call" and request.get("params", {}).get("name") == "effect":
                        receipt = {"sequence": len(effects) + 1, "arguments": request["params"].get("arguments", {})}
                        effects.append(receipt)
                        with args.trace.open("a") as output:
                            output.write(json.dumps({"kind": "real_mcp_effect", **receipt}) + "\n")
                        result = {"content": [{"type": "text", "text": json.dumps(receipt)}], "isError": False}
                    elif method == "ping":
                        result = {}
                    else:
                        self.reply(200, json.dumps({"jsonrpc": "2.0", "id": identifier, "error": {"code": -32601, "message": "Unknown MCP method or tool"}}).encode())
                        return
                    self.reply(200, json.dumps({"jsonrpc": "2.0", "id": identifier, "result": result}).encode())
                    return
                captured = {"path": self.path, "request": request, "scenario": settings.get("scenario")}
                traces.append(captured)
                with args.trace.open("a") as output:
                    output.write(json.dumps(captured) + "\n")
                current = dict(settings)
            waiting = not release.is_set()
            if waiting:
                with lock:
                    held["count"] += 1
                released = release.wait(60)
                with lock:
                    held["count"] -= 1
                if not released:
                    self.reply(503, b'{"error":{"message":"fixture hold deadline expired"}}')
                    return
            try:
                if current.get("scenario") in ("truncated_stream", "invalid_stream"):
                    if not request.get("stream"):
                        self.reply(200, b'{"id":', "application/json")
                    elif current["scenario"] == "invalid_stream":
                        self.reply(200, b'data: {bad}\n\n', "text/event-stream")
                    elif self.path.endswith("/messages"):
                        body = frame({"type": "message_start", "message": {"id": "truncated", "type": "message", "role": "assistant", "model": request.get("model"), "content": [], "usage": {"input_tokens": 2, "output_tokens": 0}}}, "message_start")
                        body += frame({"type": "content_block_start", "index": 0, "content_block": {"type": "tool_use", "id": "partial", "name": "test_lookup", "input": {}}}, "content_block_start")
                        body += frame({"type": "content_block_delta", "index": 0, "delta": {"type": "input_json_delta", "partial_json": '{"query":'}}, "content_block_delta")
                        self.reply(200, body, "text/event-stream")
                    else:
                        self.reply(200, frame({"id": "truncated", "object": "chat.completion.chunk", "model": request.get("model"), "choices": [{"index": 0, "delta": {"tool_calls": [{"index": 0, "id": "partial", "type": "function", "function": {"name": "test_lookup", "arguments": '{"query":'}}]}, "finish_reason": None}]}), "text/event-stream")
                    return
                calls, text = produce(request, current)
                if calls is None:
                    self.reply(503, b'{"error":{"message":"fixture provider unavailable","type":"server_error"}}')
                    return
                body, content_type = anthropic(request, calls, text) if self.path.endswith("/messages") else openai(request, calls, text)
                self.reply(200, body, content_type)
            except (ValueError, KeyError) as error:
                self.reply(422, json.dumps({"error": {"message": str(error)}}).encode())

    server = ThreadingHTTPServer((args.host, args.port), Handler)
    args.port_file.write_text(str(server.server_address[1]))
    server.serve_forever()


if __name__ == "__main__":
    main()
