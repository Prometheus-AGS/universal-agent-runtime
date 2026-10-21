"""Controlled MCP peer. The fsynced trace independently proves the lost-response effect."""
import json
import os
import sys
import time

trace_path = sys.argv[1]

for line in sys.stdin:
    request = json.loads(line)
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request["params"]["protocolVersion"],
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "truthful-outcomes-fixture", "version": "1.0"},
        }
    elif method == "tools/list":
        result = {"tools": [{
            "name": "effect",
            "description": "Controlled truthful-outcome acceptance effect",
            "inputSchema": {
                "type": "object", "properties": {"mode": {"type": "string"}},
                "required": ["mode"], "additionalProperties": False,
            },
            "annotations": {
                "readOnlyHint": False, "destructiveHint": True,
                "idempotentHint": False, "openWorldHint": True,
            },
        }]}
    elif method == "tools/call":
        mode = request["params"]["arguments"]["mode"]
        with open(trace_path, "a", encoding="utf-8") as trace:
            trace.write(json.dumps({"mode": mode, "pid": os.getpid(), "effect": True}) + "\n")
            trace.flush()
            os.fsync(trace.fileno())
        if mode == "unknown":
            os._exit(23)
        if mode == "timeout":
            time.sleep(90)
        if mode == "protocol":
            print(json.dumps({"jsonrpc": "2.0", "id": request_id,
                              "error": {"code": -32099, "message": "fixture protocol refusal"}}), flush=True)
            continue
        result = {
            "content": [
                {"type": "text", "text": "fixture-response-control:λ"},
                {"type": "image", "mimeType": "image/png", "data": "iVBORw0KGgo="},
                {"type": "resource", "resource": {
                    "uri": "fixture://outcomes/result", "mimeType": "text/plain",
                    "text": "embedded-resource-control",
                }},
            ],
            "structuredContent": {"mode": mode, "nested": {"control": 42}},
            "_meta": {"fixture": "metadata-control"},
        }
        if mode != "omitted":
            result["isError"] = mode == "application"
    else:
        print(json.dumps({"jsonrpc": "2.0", "id": request_id,
                          "error": {"code": -32601, "message": "method not found"}}), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)
