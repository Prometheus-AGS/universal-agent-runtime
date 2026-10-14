"""Deferred real MCP peer for Dot19. Authoring only until the full-phase gate.

Use --transport stdio|http --trace PATH; HTTP also requires --port-file PATH.
The real host owns admission/persistence. This peer records initialize capabilities,
server requests, returned elicitation decisions, and separately invoked effects.
The supervising gate kills transport/runtime after observing durable pending to
exercise disconnect/restart. No helper bypasses host approvals.
"""
import argparse
import json
import os
import pathlib
import select
import socket
import sys
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

args_parser = argparse.ArgumentParser()
args_parser.add_argument("--transport", choices=["stdio", "http"], required=True)
args_parser.add_argument("--trace", required=True)
args_parser.add_argument("--port-file")
args_parser.add_argument("--port", type=int, default=0)
args_parser.add_argument("--host", choices=["127.0.0.1", "0.0.0.0"], default="127.0.0.1")
args_parser.add_argument("--disconnect-file")
args = args_parser.parse_args()
wire = json.loads(pathlib.Path(__file__).with_name("dot19_mcp_wire.json").read_text())
trace_lock = threading.Lock()
pending_lock = threading.Lock()
pending = {}


def trace(event):
    event = dict(event, pid=os.getpid())
    with trace_lock, open(args.trace, "a", encoding="utf-8") as output:
        output.write(json.dumps(event) + "\n")
        output.flush()
        os.fsync(output.fileno())


def reply(request_id, result):
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def tool_result(value):
    return {"content": [{"type": "text", "text": json.dumps(value)}], "isError": False}


def receive(request):
    method = request.get("method")
    request_id = request.get("id")
    if method is None:
        with pending_lock:
            slot = pending.get(request_id)
            if slot is not None:
                slot["answer"] = request.get("result", {"error": request.get("error")})
                slot["ready"].set()
        trace({"event": "elicitation_response", "request_id": request_id, "message": request})
        return None, None
    if request_id is None:
        return None, None
    if method == "initialize":
        trace({"event": "initialize", "params": request["params"]})
        return reply(request_id, {"protocolVersion": request["params"]["protocolVersion"],
                                  "capabilities": {"tools": {}},
                                  "serverInfo": {"name": "dot19-elicitation", "version": "1"}}), None
    if method == "tools/list":
        schema = {"type": "object", "properties": {"mode": {"type": "string", "enum": ["form", "url", "mrtr"]}}}
        return reply(request_id, {"tools": [
            {"name": "elicit", "description": "Request a governed human input", "inputSchema": schema,
             "annotations": {"readOnlyHint": True}},
            {"name": "effect", "description": "Record an independently governed effect",
             "inputSchema": {"type": "object", "properties": {}},
             "annotations": {"readOnlyHint": False, "destructiveHint": True}}]}), None
    if method == "tools/call":
        trace({"event": "tool_call", "request_id": request_id, "params": request["params"]})
        if request["params"]["name"] == "effect":
            trace({"event": "effect", "request_id": request_id, "effect": True})
            return reply(request_id, tool_result({"effect": True})), None
        mode = request["params"].get("arguments", {}).get("mode", "form")
        if mode == "mrtr":
            result = json.loads(json.dumps(wire["rejectedAutomaticRetry"]))
            result["id"] = request_id
            return result, None
        elicitation = json.loads(json.dumps(wire["urlRequest" if mode == "url" else "formRequest"]))
        elicitation["id"] = "elicitation-" + str(uuid.uuid4())
        slot = {"request_id": request_id, "elicitation_id": elicitation["id"], "ready": threading.Event()}
        with pending_lock:
            pending[elicitation["id"]] = slot
        trace({"event": "elicitation_request", "message": elicitation})
        return elicitation, slot
    if method == "ping":
        return reply(request_id, {}), None
    return {"jsonrpc": "2.0", "id": request_id, "error": {"code": -32601, "message": "method not found"}}, None


def finish(slot):
    with pending_lock:
        pending.pop(slot["elicitation_id"], None)
    return reply(slot["request_id"], tool_result(slot["answer"]))


def controlled_disconnect():
    while True:
        if pathlib.Path(args.disconnect_file).exists():
            trace({"event": "controlled_disconnect"})
            os._exit(23)
        time.sleep(0.05)


if args.disconnect_file:
    threading.Thread(target=controlled_disconnect, daemon=True).start()

if args.transport == "stdio":
    for line in sys.stdin:
        message, slot = receive(json.loads(line))
        if message is not None:
            print(json.dumps(message), flush=True)
        with pending_lock:
            answered = [entry for entry in pending.values() if entry["ready"].is_set()]
        for entry in answered:
            print(json.dumps(finish(entry)), flush=True)
    trace({'event': 'stdio_eof'})
else:
    if not args.port_file:
        raise SystemExit("--port-file required for HTTP")

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, *_args):
            pass

        def do_GET(self):
            self.send_response(405)
            self.send_header("Content-Length", "0")
            self.end_headers()

        def do_DELETE(self):
            self.send_response(200)
            self.send_header("Content-Length", "0")
            self.end_headers()

        def do_POST(self):
            request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            message, slot = receive(request)
            if message is None:
                self.send_response(202)
                self.send_header("Content-Length", "0")
                self.end_headers()
                return
            if slot is None:
                encoded = json.dumps(message).encode()
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(encoded)))
                self.end_headers()
                self.wfile.write(encoded)
                return
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(("event: message\ndata: " + json.dumps(message) + "\n\n").encode())
            self.wfile.flush()
            # This is fixture coordination, not a replacement host deadline.
            # Abandon only this real HTTP request stream, leaving the peer alive.
            deadline = time.monotonic() + 720
            disposition = 'peer_wait_deadline'
            try:
                while time.monotonic() < deadline:
                    if args.disconnect_file and pathlib.Path(args.disconnect_file + '.abandon-request').exists():
                        trace({'event': 'request_abandoned', 'request_id': slot['elicitation_id'],
                               'transport': 'http', 'peer_process_retained': True})
                        disposition = 'controlled_stream_abandon'
                        break
                    readable, _, _ = select.select([self.connection], [], [], 0)
                    if readable and self.connection.recv(1, socket.MSG_PEEK | socket.MSG_DONTWAIT) == b'':
                        disposition = 'client_socket_eof'
                        trace({'event': 'http_handler_disconnected', 'request_id': slot['elicitation_id'],
                               'cause': disposition})
                        break
                    if slot["ready"].wait(0.05):
                        self.wfile.write(("event: message\ndata: " + json.dumps(finish(slot)) + "\n\n").encode())
                        self.wfile.flush()
                        disposition = 'response_written'
                        break
            except (BrokenPipeError, ConnectionResetError) as error:
                disposition = 'client_socket_error'
                trace({'event': 'http_handler_disconnected', 'request_id': slot['elicitation_id'],
                       'cause': disposition, 'errno': error.errno})
            finally:
                with pending_lock:
                    pending.pop(slot['elicitation_id'], None)
                self.close_connection = True
                trace({'event': 'http_handler_finished', 'request_id': slot['elicitation_id'],
                       'cause': disposition})

    server = ThreadingHTTPServer((args.host, args.port), Handler)
    port = pathlib.Path(args.port_file)
    temporary = port.with_suffix(".pending")
    temporary.write_text(str(server.server_port))
    os.replace(temporary, port)
    server.serve_forever()
