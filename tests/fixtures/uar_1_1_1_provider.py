#!/usr/bin/env python3
"""Controlled wire peer; policy, client construction and settlement remain in UAR.

The trace contains fixture input only and is private. Chunk sizes record writes
at the HTTP transport boundary, not a claim about TCP packet boundaries.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import socket
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

TEXT = "runtime-control:λ雪😀"
TOOL_MODES = {"application", "success", "omitted", "protocol", "timeout", "unknown", "combined", "tool", "partial_tool", "complete_tool_eof"}


def sse(value, event=None, newline="\n"):
    prefix = "" if event is None else "event: " + event + newline
    return (prefix + "data: " + json.dumps(value, separators=(",", ":"), ensure_ascii=False) + newline + newline).encode()


def has_result(request):
    for message in request.get("messages", []):
        if message.get("role") == "tool":
            return True
        content = message.get("content")
        if isinstance(content, list) and any(isinstance(part, dict) and part.get("type") == "tool_result" for part in content):
            return True
    return False


def prompt_skill_answer(request, prompt_marker, skill_marker):
    received = json.dumps(request, ensure_ascii=False, sort_keys=True)
    if prompt_marker not in received or skill_marker not in received:
        return "prompt-skill-marker-missing"
    return "prompt-skill-output:" + hashlib.sha256(
        (prompt_marker + "\x1f" + skill_marker).encode()).hexdigest()


def response_text(mode, request, prompt_marker=None, skill_marker=None):
    if mode == "prompt_skill":
        return prompt_skill_answer(request, prompt_marker or "", skill_marker or "")
    return "general-purpose" if mode == "graph" and "Route" in json.dumps(request) else TEXT


def anthropic_stream(mode, request=None, newline="\n", bom=False, prompt_marker=None, skill_marker=None):
    request = request or {}
    if mode == "empty":
        return b""
    frames = [sse({"type":"message_start","message":{"id":"uar111","model":"claude-haiku-4-5-20251001","role":"assistant","usage":{"input_tokens":1,"output_tokens":0}}}, "message_start", newline)]
    if mode == "metadata_only":
        return b"".join(frames)
    tool = mode in TOOL_MODES and not has_result(request)
    if tool:
        tool_mode = "application" if mode == "combined" else "success" if mode in {"tool", "partial_tool", "complete_tool_eof"} else mode
        frames += [sse({"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"call-uar111","name":"outcomes__effect","input":{}}}, "content_block_start", newline),
                   sse({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":json.dumps({"mode":tool_mode}) if mode != "partial_tool" else '{"mode":'}}, "content_block_delta", newline)]
    else:
        text = response_text(mode, request, prompt_marker, skill_marker)
        frames += [sse({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}, "content_block_start", newline),
                   sse({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":text}}, "content_block_delta", newline)]
    if mode == "text_only" or mode == "partial_tool":
        return b"".join(frames)
    frames += [sse({"type":"content_block_stop","index":0}, "content_block_stop", newline),
               sse({"type":"message_delta","delta":{"stop_reason":"tool_use" if tool else "end_turn","stop_sequence":None},"usage":{"output_tokens":1}}, "message_delta", newline)]
    if mode == "unknown":
        # MCP unknown mode still needs a valid provider terminal.
        pass
    if mode == "unknown_event":
        frames.insert(1, sse({"type":"future_event","payload":{"value":42}}, "future_event", newline))
    if mode == "known_malformed":
        frames.append(b"event: content_block_delta\ndata: {bad}\n\n")
    elif mode == "known_wrong_shape":
        frames.append(sse({"type":"content_block_delta","index":0,"delta":{}}, "content_block_delta", newline))
    elif mode == "overflow":
        frames.append(b"data: " + b"x" * (1024 * 1024 + 1))
    elif mode == "partial_bom":
        return b"\xef\xbb"
    elif mode == "invalid_utf8":
        frames.append(b"data: \xff\n\n")
    elif mode not in {"missing_terminal", "complete_tool_eof", "cancel"} and not (mode == "combined" and has_result(request)):
        frames.append(sse({"type":"message_stop"}, "message_stop", newline))
    if mode == "after_terminal":
        frames.append(sse({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"late"}}, "content_block_delta", newline))
    body = b"".join(frames)
    if mode == "sse_fields":
        # SSE comments/unknown fields and multiline JSON data remain valid.
        rendered = []
        for line in body.decode().split(newline):
            if line.startswith("data: "):
                value = line[6:]
                first, separator, rest = value.partition(",")
                line = ": fixture comment" + newline + "id: fixture" + newline + "retry: 1" + newline + "future-field: ignored" + newline + "data:  " + first + separator
                if separator:
                    line += newline + "data: " + rest + " "
            rendered.append(line)
        body = newline.join(rendered).encode()
    return (b"\xef\xbb\xbf" if bom else b"") + body


def openai_stream(mode, request, prompt_marker=None, skill_marker=None):
    if mode == "empty":
        return b""
    tool = mode in TOOL_MODES and not has_result(request)
    if tool:
        tool_mode = "application" if mode == "combined" else mode
        delta = {"tool_calls":[{"index":0,"id":"call-uar111","type":"function","function":{"name":"outcomes__effect","arguments":json.dumps({"mode":tool_mode})}}]}
    else:
        delta = {"content":response_text(mode, request, prompt_marker, skill_marker)}
    def frame(delta, finish):
        return sse({"id":"uar111","object":"chat.completion.chunk","created":0,"model":"gpt-4o-mini","choices":[{"index":0,"delta":delta,"finish_reason":finish}]})
    result = frame(delta, None) + frame({}, "tool_calls" if tool else "stop")
    if mode == "catalog_budget":
        result += sse({"id":"uar111","object":"chat.completion.chunk","created":0,"model":"gpt-4o-mini",
            "choices":[],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}})
    if mode != "missing_terminal" and not (mode == "combined" and has_result(request)):
        result += b"data: [DONE]\n\n"
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port-file", required=True, type=Path)
    parser.add_argument("--trace", required=True, type=Path)
    parser.add_argument("--mode-file", required=True, type=Path)
    args = parser.parse_args()
    lock = threading.Lock()
    connections_lock = threading.Lock()
    connections = set()
    stopping = threading.Event()
    args.trace.touch(mode=0o600)
    args.trace.chmod(0o600)

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"
        def setup(self):
            super().setup()
            with connections_lock:
                connections.add(self.connection)
                if stopping.is_set():
                    self.connection.shutdown(socket.SHUT_RDWR)

        def finish(self):
            try:
                super().finish()
            finally:
                with connections_lock:
                    connections.discard(self.connection)
                contact = getattr(self, "held_contact", None)
                if contact:
                    with lock:
                        print("UAR_FIXTURE_EVENT:" + json.dumps({"version": 1, "event": "response_released",
                              "contact_token": contact, "fixture_shutdown": stopping.is_set()}), flush=True)

        def log_message(self, *_args):
            pass
        def do_GET(self):
            body = json.dumps({"object":"list","data":[{"id":"gpt-4o-mini","object":"model"},{"id":"claude-sonnet-4-20250514","object":"model"}]}).encode()
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        def do_POST(self):
            request = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
            received_at = time.monotonic()
            settings = json.loads(args.mode_file.read_text())
            mode = settings.get("mode", "complete")
            self.held_contact = settings.get("contact_token") if mode == "cancel" else None
            body = anthropic_stream(mode, request, settings.get("newline", "\n"), settings.get("bom", False),
                                    settings.get("prompt_marker"), settings.get("skill_marker")) if self.path.endswith("/messages") else openai_stream(mode, request, settings.get("prompt_marker"), settings.get("skill_marker"))
            split = settings.get("split")
            fixed = settings.get("fixed")
            pieces = [body[index:index+fixed] for index in range(0,len(body),fixed)] if fixed else [bytes([byte]) for byte in body] if split == "bytewise" else [body[:split], body[split:]] if isinstance(split, int) else [body]
            correlation_id = hashlib.sha256(json.dumps(request, ensure_ascii=False, sort_keys=True).encode()).hexdigest()
            with lock, args.trace.open("a", encoding="utf-8") as trace:
                # Credentials are synthetic; store only equality labels, never header bytes.
                key = self.headers.get("x-api-key", self.headers.get("Authorization", ""))
                source = "empty" if not key or key == "Bearer " else "fixture" if key in {"runtime-fixture-key", "Bearer runtime-fixture-key"} else "other"
                trace.write(json.dumps({"kind":"request","correlation_id":correlation_id,"path":self.path,"mode":mode,"request":request,"credential_source":source,"sha256":hashlib.sha256(body).hexdigest(),"chunks":[len(part) for part in pieces],"peer_received_at":received_at,"time":received_at}) + "\n")
                trace.flush()
                os.fsync(trace.fileno())
                if settings.get("contact_token"):
                    print("UAR_FIXTURE_EVENT:" + json.dumps({"version": 1, "event": "request",
                          "contact_token": settings["contact_token"], "correlation_id": correlation_id}), flush=True)
            if settings.get("delay"):
                # Configured latency is a business fault simulation. Host EOF
                # can interrupt it; elapsed latency never proves cleanup.
                if stopping.wait(settings["delay"]):
                    return
            self.send_response(settings.get("http_status", 200))
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Transfer-Encoding", "chunked")
            self.end_headers()
            first_write_at = None
            last_write_at = None
            try:
                for piece in pieces:
                    if piece:
                        first_write_at = first_write_at or time.monotonic()
                        self.wfile.write(f"{len(piece):x}\r\n".encode() + piece + b"\r\n")
                        self.wfile.flush()
                        last_write_at = time.monotonic()
                        if split is not None or fixed:
                            if stopping.wait(0.001):
                                return
                if mode == "cancel":
                    # Keep this response incomplete until the client actually
                    # closes it or fixture EOF explicitly releases the socket.
                    # This worker owns the read and is joined by server_close.
                    unexpected = self.connection.recv(1)
                    if unexpected:
                        raise RuntimeError("held response received unexpected pipelined bytes")
                    self.close_connection = True
                    return
                self.wfile.write(b"0\r\n\r\n")
                self.wfile.flush()
                last_write_at = time.monotonic()
            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError):
                pass
            finally:
                with lock, args.trace.open("a", encoding="utf-8") as trace:
                    trace.write(json.dumps({"kind":"timing","correlation_id":correlation_id,
                                              "peer_first_write_at":first_write_at,
                                              "peer_last_write_at":last_write_at}) + "\n")
                    trace.flush()
                    os.fsync(trace.fileno())

    server = ThreadingHTTPServer(("0.0.0.0", 0), Handler)
    server.daemon_threads = False
    pending = args.port_file.with_suffix(".pending")
    pending.write_text(str(server.server_address[1]))
    pending.replace(args.port_file)
    print("UAR_FIXTURE_EVENT:" + json.dumps({"version": 1, "event": "ready",
          "port": server.server_address[1]}), flush=True)
    try:
        while True:
            readable, _, _ = select.select([server, sys.stdin], [], [])
            if sys.stdin in readable:
                command = os.read(sys.stdin.fileno(), 4096)
                if not command:
                    break
                raise RuntimeError("provider fixture accepts only inherited EOF shutdown")
            if server in readable:
                server.handle_request()
    finally:
        with connections_lock:
            stopping.set()
            owned_connections = list(connections)
        for connection in owned_connections:
            try:
                connection.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass  # already closed; worker join below remains authoritative
        server.server_close()
        print("UAR_FIXTURE_EVENT:" + json.dumps({"version": 1, "event": "closed",
              "completion": "listener_and_handlers_joined"}), flush=True)


if __name__ == "__main__":
    main()
