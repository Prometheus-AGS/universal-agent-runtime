"""Controlled MCP effects for fresh-process continuation qualification."""
import json
import os
import signal
import sys

trace_path = sys.argv[1]


def record(event):
    event["pid"] = os.getpid()
    with open(trace_path, "a", encoding="utf-8") as trace:
        trace.write(json.dumps(event) + "\n")
        trace.flush()
        os.fsync(trace.fileno())


record({"event": "started"})
for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {"protocolVersion": request["params"]["protocolVersion"],
                  "capabilities": {"tools": {}},
                  "serverInfo": {"name": "resume-fixture", "version": "1.0"}}
    elif method == "tools/list":
        result = {"tools": [{"name": "effect", "description": "Controlled resume effect",
                             "inputSchema": {"type": "object",
                                             "properties": {"mode": {"type": "string"}},
                                             "required": ["mode"], "additionalProperties": False},
                             "annotations": {"readOnlyHint": False, "destructiveHint": True,
                                             "idempotentHint": False, "openWorldHint": True}}]}
    elif method == "tools/call":
        mode = request["params"]["arguments"]["mode"]
        record({"event": "dispatch", "mode": mode})
        if mode == "hold_before_effect":
            while True:
                signal.pause()
        record({"event": "effect", "mode": mode})
        if mode == "hold_after_effect":
            while True:
                signal.pause()
        if mode == "unknown":
            os._exit(23)
        result = {"content": [{"type": "text", "text": "resume-effect-complete"}],
                  "structuredContent": {"mode": mode}, "isError": False}
    else:
        print(json.dumps({"jsonrpc": "2.0", "id": request_id,
                          "error": {"code": -32601, "message": "method not found"}}), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)
