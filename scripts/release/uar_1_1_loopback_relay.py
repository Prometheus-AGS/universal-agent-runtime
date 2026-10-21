#!/usr/bin/env python3
"""Forward one acceptance TCP port to the controlled MCP peer.

The release runner starts this process either inside a runtime on IPv4 loopback
or in a separate candidate-image container on the private acceptance network.
The external relay has no published host port. Both paths retain the supervised
dot19 peer's trace and credential checks without interpreting MCP messages.
"""

import argparse
import os
import select
import socket
import socketserver
from pathlib import Path


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--listen-host", choices=("127.0.0.1", "0.0.0.0"), default="127.0.0.1")
parser.add_argument("--listen-port", required=True, type=int)
parser.add_argument("--upstream-host", required=True)
parser.add_argument("--upstream-port", required=True, type=int)
parser.add_argument("--ready-file", required=True, type=Path)
parser.add_argument("--pid-file", required=True, type=Path)
args = parser.parse_args()


class RelayHandler(socketserver.BaseRequestHandler):
    def handle(self):
        upstream = socket.create_connection((args.upstream_host, args.upstream_port), timeout=10)
        try:
            sockets = [self.request, upstream]
            while True:
                readable, _, _ = select.select(sockets, [], [])
                for source in readable:
                    payload = source.recv(65_536)
                    if not payload:
                        return
                    target = upstream if source is self.request else self.request
                    target.sendall(payload)
        finally:
            upstream.close()


class LoopbackRelay(socketserver.ThreadingTCPServer):
    address_family = socket.AF_INET
    allow_reuse_address = True
    daemon_threads = True


with LoopbackRelay((args.listen_host, args.listen_port), RelayHandler) as relay:
    args.pid_file.write_text(str(os.getpid()), encoding="utf-8")
    args.ready_file.write_text("ready\n", encoding="utf-8")
    print("UAR_RELAY_READY", flush=True)
    relay.serve_forever()
