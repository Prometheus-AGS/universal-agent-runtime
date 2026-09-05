#!/usr/bin/env python3
"""Minimal real stdio MCP server for projected-runtime integration tests."""

import atexit
import json
import os
import pathlib
import sys
import time


RECEIPT_DIR = pathlib.Path(os.environ["MCP_FIXTURE_RECEIPT_DIR"])
RECEIPT_DIR.mkdir(parents=True, exist_ok=True)
PID = os.getpid()


def append_receipt(name: str) -> None:
    with (RECEIPT_DIR / name).open("a", encoding="utf-8") as receipt:
        receipt.write(f"{PID}\n")
        receipt.flush()


append_receipt("started.log")
atexit.register(append_receipt, "stopped.log")


def reply(request_id: object, result: object) -> None:
    try:
        sys.stdout.write(
            json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}) + "\n"
        )
        sys.stdout.flush()
    except BrokenPipeError:
        devnull = os.open(os.devnull, os.O_WRONLY)
        os.dup2(devnull, sys.stdout.fileno())
        os.close(devnull)
        raise SystemExit(0)


for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if method == "initialize":
        delay_ms = int(os.environ.get("MCP_FIXTURE_INITIALIZE_DELAY_MS", "0"))
        if delay_ms:
            time.sleep(delay_ms / 1000)
        reply(
            message["id"],
            {
                "protocolVersion": message.get("params", {}).get(
                    "protocolVersion", "2024-11-05"
                ),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "uar-stdio-fixture", "version": "1.0.0"},
            },
        )
    elif method == "tools/list":
        reply(
            message["id"],
            {
                "tools": [
                    {
                        "name": "echo",
                        "description": "Echo text through the stdio MCP fixture",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"text": {"type": "string"}},
                            "required": ["text"],
                            "additionalProperties": False,
                        },
                        "annotations": {"readOnlyHint": True},
                    }
                ]
            },
        )
    elif method == "tools/call":
        text = message.get("params", {}).get("arguments", {}).get("text", "")
        reply(
            message["id"],
            {"content": [{"type": "text", "text": text}], "isError": False},
        )
    elif method == "ping":
        reply(message["id"], {})
