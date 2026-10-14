"""Deferred installed-host schedule integration driver; never run before full phase.

The caller owns two-runtime startup, qualified supervisor exclusion, real provider
and approval/effect fixtures, and cold process restart. This driver sends real
HTTP requests only; it never modifies protected rows or manufactures authority.

Environment: UAR_SCHEDULE_BASE_URL, UAR_SCHEDULE_TEST_TOKEN,
UAR_SCHEDULE_OTHER_TOKEN, UAR_SCHEDULE_WORKSPACE, UAR_SCHEDULE_INSTANCE_ID.
Optional UAR_SCHEDULE_SECOND_URL reads the same durable state through runtime B.
Usage: MODE RECEIPT_PATH. Modes: seed, read, busy, terminal, edit, pause, resume, cancel.
"""
import datetime
import json
import os
from pathlib import Path
import sys
import urllib.error
import urllib.request

base = os.environ["UAR_SCHEDULE_BASE_URL"].rstrip("/")
token = os.environ["UAR_SCHEDULE_TEST_TOKEN"]
other = os.environ["UAR_SCHEDULE_OTHER_TOKEN"]
workspace = os.environ["UAR_SCHEDULE_WORKSPACE"]
instance = os.environ["UAR_SCHEDULE_INSTANCE_ID"]
prefix = "/api/uar/schedules"


def request(method, suffix="", payload=None, credential=token, endpoint=base):
    headers = {"Content-Type": "application/json", "x-uar-workspace-id": workspace}
    if credential:
        headers["Authorization"] = "Bearer " + credential
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(endpoint + prefix + suffix, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            return response.status, json.load(response)
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read())


def read(schedule_id, endpoint=base):
    code, value = request("GET", "/" + schedule_id, endpoint=endpoint)
    assert code == 200, value
    assert "prompt" not in value and "ownerGrant" not in value
    assert request("GET", "/" + schedule_id, credential=other)[0] == 404
    return value


mode, receipt_file = sys.argv[1:]
path = Path(receipt_file)
if mode == "seed":
    assert request("GET", credential=None)[0] == 401
    fixtures = json.loads(Path(__file__).with_name("dot14_calendar_cases.json").read_text())
    for recurrence in fixtures["invalidRecurrences"]:
        assert request("POST", payload={"instanceId": instance, "prompt": "invalid fixture", "recurrence": recurrence})[0] in (400, 422)
    past = (datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(seconds=1)).isoformat()
    assert request("POST", payload={"instanceId": instance, "prompt": "past fixture", "recurrence": {"kind": "once", "at": past}})[0] == 400
    # The caller may supply a daily/weekly recurrence to exercise busy-time and
    # DST paths under its controlled VM clock. No runtime test-clock API exists.
    at = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(seconds=30)).isoformat()
    recurrence = json.loads(os.environ.get("UAR_SCHEDULE_RECURRENCE", json.dumps({"kind": "once", "at": at})))
    code, record = request("POST", payload={"instanceId": instance, "prompt": os.environ.get("UAR_SCHEDULE_PROMPT", "Return the schedule fixture marker once."), "recurrence": recurrence})
    assert code == 201 and record["revision"] == 1, record
    path.write_text(json.dumps({"id": record["id"], "revision": record["revision"], "initial": record}, indent=2))
else:
    saved = json.loads(path.read_text())
    record = read(saved["id"])
    if second := os.environ.get("UAR_SCHEDULE_SECOND_URL"):
        peer = read(saved["id"], second.rstrip("/"))
        assert peer["id"] == record["id"] and peer["revision"] == record["revision"]
    ids = [receipt["occurrence"]["id"] for receipt in record["receipts"]]
    assert len(ids) == len(set(ids)), "duplicate logical occurrence receipt"
    if mode == "busy":
        assert record["active"] is not None and record["pending"] is not None, record
        assert record["active"]["id"] != record["pending"]["id"]
        assert any(skip["reason"] == "coalesced" for skip in record["skipped"]), record
        if saved.get("busyActive"):
            assert record["active"]["id"] == saved["busyActive"], "guard changed before safe resolution"
        saved["busyActive"] = record["active"]["id"]
        saved["latestPending"] = record["pending"]["id"]
        saved["receiptCountWhileBusy"] = len(record["receipts"])
    elif mode == "terminal":
        assert record["receipts"], record
        if saved.get("busyActive"):
            assert saved["busyActive"] in ids
            admitted = ([record["active"]["id"]] if record["active"] else []) + ids
            assert saved["latestPending"] in admitted, "latest pending was not admitted after safe resolution"
    elif mode == "edit":
        old = record["active"]
        recurrence = json.loads(os.environ["UAR_SCHEDULE_RECURRENCE"])
        code, changed = request("PUT", "/" + saved["id"], {"expectedRevision": record["revision"], "schedule": {"instanceId": instance, "recurrence": recurrence, "prompt": "Revised schedule fixture marker."}})
        assert code == 200 and changed["revision"] == record["revision"] + 1, changed
        assert changed["pending"] is None
        assert changed["active"] == old, "edit released or reinterpreted the admitted predecessor"
        assert request("POST", "/" + saved["id"] + "/pause", {"expectedRevision": record["revision"]})[0] == 409
        saved["revision"] = changed["revision"]
    elif mode in ("pause", "resume"):
        code, changed = request("POST", "/" + saved["id"] + "/" + mode, {"expectedRevision": record["revision"]})
        assert code == 200 and changed["paused"] == (mode == "pause"), changed
        assert changed["active"] == record["active"]
        if record["pending"]:
            assert changed["pending"]["dueAt"] == record["pending"]["dueAt"]
    elif mode == "cancel":
        code, changed = request("POST", "/" + saved["id"] + "/cancel", {"expectedRevision": record["revision"]})
        assert code == 200 and changed["cancelled"] and changed["pending"] is None, changed
        assert changed["active"] == record["active"]
    elif mode != "read":
        raise AssertionError("unknown fixture mode")
    saved["lastObserved"] = record
    path.write_text(json.dumps(saved, indent=2))
print(json.dumps({"mode": mode, "receipt": str(path), "result": "passed"}))
