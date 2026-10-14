#!/usr/bin/env python3
"""Loopback-only HTTPS endpoint. No upstream connections or operator credentials."""
import http.server
import json
import pathlib
import ssl
import sys
import threading
import time

root = pathlib.Path(sys.argv[1])
case = sys.argv[2]
lock = threading.Lock()
count = 0


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_args):
        pass

    def do_POST(self):
        global count
        request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        with lock:
            count += 1
            index = count
            with (root / "requests.jsonl").open("a") as output:
                output.write(json.dumps({"path": self.path, "request": request}) + "\n")
        data = (root / ("first.bin" if index == 1 else "second.bin")).read_bytes()
        self.send_response(200)
        self.send_header("Content-Type", "application/vnd.amazon.eventstream" if (root / "content-type").read_text() == "bedrock" else "text/event-stream")
        self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()
        try:
            pieces = (bytes([b]) for b in data) if case in ("split", "tool") else [data]
            for piece in pieces:
                if piece:
                    self.wfile.write(f"{len(piece):x}\r\n".encode() + piece + b"\r\n")
                    self.wfile.flush()
                if case in ("split", "tool"):
                    time.sleep(0.001)
            if case == "cancel":
                time.sleep(120)
                return
            self.wfile.write(b"0\r\n\r\n")
            self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError, ssl.SSLError):
            pass


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
server.daemon_threads = True
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(root / "server.pem", root / "server.key")
server.socket = context.wrap_socket(server.socket, server_side=True)
ready_port = root / "port.pending"
ready_port.write_text(str(server.server_address[1]))
ready_port.replace(root / "port")
server.serve_forever()
