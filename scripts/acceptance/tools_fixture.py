"""Legitimate owned-file MCP operations and bounded negative provider transport.

This is a real tool peer, never an assistant-response generator. Importing this
module starts no process. Execution belongs only to the exact approved packet.
"""
from __future__ import annotations

import hashlib
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import re
import struct
import sys
import zlib
import urllib.request
import urllib.error
from urllib.parse import urlsplit


def root_path(raw):
    root = Path(raw)
    if not root.is_absolute() or root.is_symlink() or root.name != "tools-effects":
        raise ValueError("owned effects root unavailable")
    if (not root.parent.name.startswith("uar_acceptance_") or not root.parent.is_dir()
            or root.parent.is_symlink() or root.parent.stat().st_mode & 0o077):
        raise ValueError("effects parent lacks private run ownership")
    root.mkdir(mode=0o700, exist_ok=True)
    if not root.is_dir() or root.stat().st_mode & 0o077:
        raise ValueError("effects root lacks private run ownership")
    return root.resolve()


def target(root, name):
    if not isinstance(name, str) or not re.fullmatch(r"[a-z0-9-]{1,80}", name):
        raise ValueError("invalid owned record name")
    path = root / (name + ".txt")
    if path.is_symlink() or (path.exists() and not path.is_file()):
        raise ValueError("owned record is not a regular file")
    return path


def record(root, event):
    with (root / "mcp-receipts.jsonl").open("a", encoding="utf-8") as stream:
        stream.write(json.dumps({"pid": os.getpid(), **event}, sort_keys=True) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def operate(root, arguments):
    path = target(root, arguments.get("name"))
    operation = arguments.get("operation")
    marker = arguments.get("marker", "")
    if operation not in ("read", "append", "commit_disconnect"):
        raise ValueError("unsupported file operation")
    if not re.fullmatch(r"[a-f0-9]{24}", marker):
        raise ValueError("invalid run-owned effect marker")
    if operation != "read":
        with path.open("a", encoding="utf-8") as stream:
            stream.write(marker + "\n")
            stream.flush()
            os.fsync(stream.fileno())
    content = path.read_bytes()
    record(root, {"name": arguments["name"], "operation": operation, "marker": marker,
                  "bytes": len(content), "sha256": "sha256:" + hashlib.sha256(content).hexdigest(),
                  "effect_committed": operation != "read"})
    if operation == "commit_disconnect":
        # A real committed effect followed by real transport loss; no result is
        # fabricated and no automatic retry/idempotency promise is supplied.
        os._exit(23)
    return {"content": content.decode(), "bytes": len(content)}


def mcp(root):
    for line in sys.stdin:
        request = json.loads(line)
        if "id" not in request:
            continue
        method = request.get("method")
        if method == "initialize":
            result = {"protocolVersion": request["params"]["protocolVersion"], "capabilities": {"tools": {}},
                      "serverInfo": {"name": "uar-owned-file-operations", "version": "1.0.0"}}
        elif method == "tools/list":
            result = {"tools": [{"name": "owned_record", "description": "Append a marker or read a real run-owned record.",
                "inputSchema": {"type": "object", "properties": {
                    "name": {"type": "string", "pattern": "^[a-z0-9-]{1,80}$"},
                    "operation": {"enum": ["read", "append", "commit_disconnect"]},
                    "marker": {"type": "string", "pattern": "^[a-f0-9]{24}$"}},
                    "required": ["name", "operation", "marker"], "additionalProperties": False},
                "annotations": {"readOnlyHint": False, "destructiveHint": False,
                                "idempotentHint": False, "openWorldHint": False}}]}
        elif method == "tools/call":
            try:
                if request["params"]["name"] != "owned_record":
                    raise ValueError("unknown tool")
                value = operate(root, request["params"]["arguments"])
                result = {"content": [{"type": "text", "text": json.dumps(value)}], "isError": False}
            except Exception:
                result = {"content": [{"type": "text", "text": "owned file operation refused"}], "isError": True}
        elif method == "ping":
            result = {}
        else:
            print(json.dumps({"jsonrpc": "2.0", "id": request["id"],
                              "error": {"code": -32601, "message": "method not found"}}), flush=True)
            continue
        print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)


def color_image(colors):
    """Create a deterministic PNG data fixture; text never names its colors."""
    rgb = {"red": (255, 0, 0), "green": (0, 180, 0), "blue": (0, 0, 255), "yellow": (255, 255, 0)}
    width, height = 192, 64
    row = b"".join(bytes(rgb[color]) * 64 for color in colors)
    def chunk(kind, data):
        return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress((b"\0" + row) * height)) + chunk(b"IEND", b""))


def provider_fault(root, port=0):
    if type(port) is not int or (port != 0 and not 1024 <= port <= 65535):
        raise ValueError("invalid explicit provider fault port")
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            size = min(int(self.headers.get("Content-Length", "0")), 1024 * 1024)
            self.rfile.read(size)
            record(root, {"operation": "provider-denial", "path": self.path, "status": 401})
            self.send_response(401)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"error":{"code":"fixture_provider_denied","message":"run-owned denial"}}')
        def log_message(self, *_):
            pass
    server = HTTPServer(("127.0.0.1", port), Handler)
    with (root / ("provider-fault-" + str(os.getpid()) + ".json")).open("x") as stream:
        json.dump({"pid": os.getpid(), "origin": "http://127.0.0.1:" + str(server.server_port)}, stream)
    server.serve_forever(poll_interval=0.1)


def genuine_proxy(root, port, upstream, budget_config=None):
    """Forward to the frozen genuine endpoint and retain bounded wire metadata.

    No assistant result is generated here. Credential headers are forwarded
    only to the exact HTTPS origin and are never recorded.
    """
    endpoint = urlsplit(upstream)
    hop = None
    if budget_config is not None:
        from budget_proxy import owned_hop
        _config, binding, ready = owned_hop(budget_config, upstream)
        hop = {"binding_id": binding["id"], "config_sha256": ready["config_sha256"],
               "input_identity": ready["epoch"]["input_identity"], "owner_pid": ready["pid"],
               "owner_nonce": ready["nonce"], "upstream_url": binding["upstream_url"]}
    elif endpoint.scheme != "https" or not endpoint.hostname or endpoint.username or endpoint.password or endpoint.query or endpoint.fragment:
        raise ValueError("genuine upstream must be HTTPS or an exact owned approved budget hop")
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req, fp, code, msg, headers, newurl):
            return None
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect)
    class Handler(BaseHTTPRequestHandler):
        def do_POST(self):
            if self.path != "/v1/chat/completions":
                self.send_error(404)
                return
            size = int(self.headers.get("Content-Length", "0"))
            if not 0 < size <= 4 * 1024 * 1024:
                self.send_error(413)
                return
            raw = self.rfile.read(size)
            body = json.loads(raw)
            request_id = os.urandom(16).hex()
            record(root, {"operation": "genuine-wire", "request_id": request_id, "upstream": upstream,
                          "model": body.get("model"), "response_format": body.get("response_format"),
                          "stream": body.get("stream"), "max_tokens": body.get("max_tokens"),
                          "max_completion_tokens": body.get("max_completion_tokens"),
                          "request_sha256": "sha256:" + hashlib.sha256(raw).hexdigest(), "budget_hop": hop})
            request = urllib.request.Request(upstream.rstrip("/") + "/chat/completions", data=raw,
                headers={"Content-Type": "application/json", "Authorization": self.headers.get("Authorization", "")}, method="POST")
            try:
                response = opener.open(request, timeout=90)
            except urllib.error.HTTPError as error:
                response = error
            self.send_response(response.status)
            self.send_header("Content-Type", response.headers.get("Content-Type", "application/json"))
            self.end_headers()
            digest = hashlib.sha256()
            count = 0
            with response:
                while True:
                    chunk = response.read1(16384)
                    if not chunk:
                        break
                    count += len(chunk)
                    if count > 8 * 1024 * 1024:
                        raise ValueError("genuine response exceeded fixture allowance")
                    digest.update(chunk)
                    self.wfile.write(chunk)
                    self.wfile.flush()
            record(root, {"operation": "genuine-wire-response", "request_id": request_id,
                          "status": response.status, "bytes": count, "sha256": "sha256:" + digest.hexdigest()})
        def log_message(self, *_):
            pass
    server = HTTPServer(("127.0.0.1", port), Handler)
    with (root / ("genuine-proxy-" + str(port) + ".json")).open("x") as stream:
        json.dump({"pid": os.getpid(), "port": port, "upstream": upstream}, stream)
    server.serve_forever(poll_interval=0.1)


def main():
    if len(sys.argv) in (5, 6) and sys.argv[1] == "genuine-proxy":
        genuine_proxy(root_path(sys.argv[2]), int(sys.argv[3]), sys.argv[4], sys.argv[5] if len(sys.argv) == 6 else None)
        return
    if len(sys.argv) == 4 and sys.argv[1] == "provider-fault":
        provider_fault(root_path(sys.argv[2]), int(sys.argv[3]))
        return
    if len(sys.argv) != 3:
        raise ValueError("expected mcp or provider-fault and exact owned root")
    root = root_path(sys.argv[2])
    if sys.argv[1] == "mcp":
        mcp(root)
    elif sys.argv[1] == "provider-fault":
        provider_fault(root)
    else:
        raise ValueError("unsupported owned fixture mode")


if __name__ == "__main__":
    main()
