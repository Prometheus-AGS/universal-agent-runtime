"""Deferred real-runtime AG-UI fixture; never supplies a fake executor or broker.
Run only after the lead's full-phase implementation-complete inventory.
Environment: UAR_URL, UAR_TOKEN, UAR_AGENT; optional UAR_WORKSPACE.
Commands: submit <thread> <run> <prompt>; resume <thread> <new-run> <prior-run>
<interrupt> <approve|deny>; reconnect <thread> <run> [last-event-id].
"""
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request

base = os.environ["UAR_URL"].rstrip("/")
agent = urllib.parse.quote(os.environ["UAR_AGENT"], safe="")
headers = {"Authorization": "Bearer " + os.environ["UAR_TOKEN"], "Accept": "text/event-stream"}
if os.environ.get("UAR_WORKSPACE"):
    headers["x-uar-workspace-id"] = os.environ["UAR_WORKSPACE"]
mode, thread, run = sys.argv[1:4]
url = base + "/api/uar/ag-ui/agents/" + agent
body = None
if mode == "submit":
    body = {"threadId": thread, "runId": run, "protocolVersion": "1.0", "messages": [
        {"id": run + "-user", "role": "user", "content": sys.argv[4]}]}
elif mode == "resume":
    body = {"threadId": thread, "runId": run, "parentRunId": sys.argv[4], "messages": [],
            "resume": [{"interruptId": sys.argv[5], "status": "resolved", "payload": {"approved": sys.argv[6] == "approve"}}]}
else:
    assert mode == "reconnect"
    url += "/threads/" + urllib.parse.quote(thread, safe="") + "/runs/" + urllib.parse.quote(run, safe="") + "/stream"
    if len(sys.argv) > 4:
        headers["Last-Event-ID"] = sys.argv[4]
if body is not None:
    headers["Content-Type"] = "application/json"
request = urllib.request.Request(url, headers=headers, data=None if body is None else json.dumps(body).encode())
closed = False
with urllib.request.urlopen(request, timeout=120) as response:
    assert response.status == 200
    assert response.headers.get_content_type() == "text/event-stream"
    for raw in response:
        line = raw.decode().strip()
        if not line.startswith("data:"):
            continue
        event = json.loads(line[5:].strip())
        assert not closed, "event emitted after terminal outcome"
        assert isinstance(event.get("type"), str)
        if event["type"] in ("RUN_STARTED", "RUN_FINISHED"):
            assert event["threadId"] == thread and event["runId"] == run
        if event["type"] == "RUN_FINISHED":
            outcome = event.get("outcome", {"type": "success"})
            assert outcome["type"] in ("success", "interrupt", "cancelled")
            if outcome["type"] == "interrupt":
                assert outcome["interrupts"]
                assert len({entry["id"] for entry in outcome["interrupts"]}) == len(outcome["interrupts"])
            closed = True
        if event["type"] == "RUN_ERROR":
            assert isinstance(event["message"], str)
            closed = True
        print(json.dumps(event, separators=(",", ":")), flush=True)
assert closed, "stream ended without AG-UI terminal event"
