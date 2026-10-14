"""Deferred installed-host follow-up driver; never execute before full phase.

The caller supplies real owned logical source/destination instances and generates
durable source events through production APIs. It owns process crash/restart,
shared-runtime races, clock control and grant revocation. No protected storage
mutation, fabricated grants, mock transport, or runtime test-clock API is used.

Environment: UAR_FOLLOWUP_BASE_URL, UAR_FOLLOWUP_TEST_TOKEN,
UAR_FOLLOWUP_OTHER_TOKEN, UAR_FOLLOWUP_WORKSPACE, UAR_FOLLOWUP_SOURCE,
UAR_FOLLOWUP_DESTINATION, UAR_FOLLOWUP_CONVERSATION.
Optional: UAR_FOLLOWUP_SECOND_URL, UAR_FOLLOWUP_EVENT_KINDS (JSON list),
UAR_FOLLOWUP_QUIET_HOURS (JSON object or null), UAR_FOLLOWUP_SUPPRESSION_REASON.
Usage: MODE RECEIPT_PATH. Modes: seed, read, delivered, deferred, suppressed,
enable, disable, revoke, edit, acknowledge, hidden.
"""
import json
import os
from pathlib import Path
import sys
import urllib.error
import urllib.parse
import urllib.request

base = os.environ["UAR_FOLLOWUP_BASE_URL"].rstrip("/")
token = os.environ["UAR_FOLLOWUP_TEST_TOKEN"]
other = os.environ["UAR_FOLLOWUP_OTHER_TOKEN"]
workspace = os.environ["UAR_FOLLOWUP_WORKSPACE"]
source = os.environ["UAR_FOLLOWUP_SOURCE"]
destination = os.environ["UAR_FOLLOWUP_DESTINATION"]
conversation = os.environ["UAR_FOLLOWUP_CONVERSATION"]
prefix = "/api/uar/followups"


def segment(value):
    return urllib.parse.quote(value, safe="")


def request(method, suffix="", payload=None, credential=token, endpoint=base):
    headers = {"Content-Type": "application/json", "x-uar-workspace-id": workspace}
    if credential:
        headers["Authorization"] = "Bearer " + credential
    data = None if payload is None else json.dumps(payload).encode()
    req = urllib.request.Request(endpoint + prefix + suffix, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            raw = response.read()
            return response.status, json.loads(raw) if raw else None
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read())


def read(rule_id, endpoint=base):
    code, value = request("GET", "/" + segment(rule_id), endpoint=endpoint)
    assert code == 200, value
    assert "ownerGrant" not in value and "sourceAuthority" not in value
    assert all("eventKind" not in item and "occurrenceId" not in item for item in value["decisions"])
    assert request("GET", "/" + segment(rule_id), credential=other)[0] == 404
    return value


def inbox(endpoint=base):
    code, items = request("GET", "/inbox/" + segment(conversation), endpoint=endpoint)
    assert code == 200, items
    identities = [(item["ruleId"], item["occurrenceId"]) for item in items]
    assert len(identities) == len(set(identities)), "duplicate logical inbox delivery"
    return items


mode, receipt_file = sys.argv[1:]
path = Path(receipt_file)
if mode == "seed":
    assert request("GET", credential=None)[0] == 401
    body = {"sourceInstanceId": source, "destinationInstanceId": destination,
            "conversationId": conversation,
            "eventKinds": json.loads(os.environ.get("UAR_FOLLOWUP_EVENT_KINDS", '["turn_completed"]')),
            "quietHours": json.loads(os.environ.get("UAR_FOLLOWUP_QUIET_HOURS", "null"))}
    code, rule = request("POST", payload=body)
    assert code == 201 and rule["revision"] == 1 and not rule["enabled"], rule
    assert not rule["decisions"], "creation must not import historical source events"
    path.write_text(json.dumps({"id": rule["id"], "rule": body, "initial": rule}, indent=2))
else:
    saved = json.loads(path.read_text())
    rule = read(saved["id"])
    items = [item for item in inbox() if item["ruleId"] == saved["id"]]
    if second := os.environ.get("UAR_FOLLOWUP_SECOND_URL"):
        peer = read(saved["id"], second.rstrip("/"))
        assert peer["revision"] == rule["revision"]
        assert {item["id"] for item in inbox(second.rstrip("/")) if item["ruleId"] == saved["id"]} == {item["id"] for item in items}
    if mode in ("enable", "disable", "revoke"):
        code, changed = request("POST", "/" + segment(saved["id"]) + "/" + mode, {"expectedRevision": rule["revision"]})
        assert code == 200 and changed["revision"] == rule["revision"] + 1, changed
        assert changed["enabled"] == (mode == "enable")
        assert changed["revoked"] == (mode == "revoke")
        assert request("POST", "/" + segment(saved["id"]) + "/disable", {"expectedRevision": rule["revision"]})[0] == 409
        if mode != "enable":
            assert all(d["disposition"] not in ("queued", "deferred") for d in changed["decisions"])
    elif mode == "edit":
        body = dict(saved["rule"], enabled=rule["enabled"], quietHours=json.loads(os.environ["UAR_FOLLOWUP_QUIET_HOURS"]))
        code, changed = request("PUT", "/" + segment(saved["id"]), {"expectedRevision": rule["revision"], "rule": body})
        assert code == 200 and changed["revision"] == rule["revision"] + 1, changed
        assert all(d["ruleRevision"] == changed["revision"] for d in changed["decisions"] if d["disposition"] in ("queued", "deferred"))
        saved["rule"] = body
    elif mode == "delivered":
        assert items, rule
        delivered = {d["id"] for d in rule["decisions"] if d["disposition"] == "delivered"}
        assert all(item["id"] in delivered for item in items)
        assert request("GET", "/inbox/" + segment(conversation), credential=other) == (200, [])
        if saved.get("deliveredIds"):
            assert set(saved["deliveredIds"]).issubset({item["id"] for item in items}), "restart lost a committed item"
        saved["deliveredIds"] = [item["id"] for item in items]
    elif mode == "deferred":
        deferred = [d for d in rule["decisions"] if d["disposition"] == "deferred"]
        assert deferred and all(d["reason"] == "quiet_hours" and d["availableAt"] for d in deferred), rule
        assert not {d["id"] for d in deferred}.intersection(item["id"] for item in items)
    elif mode == "suppressed":
        reason = os.environ.get("UAR_FOLLOWUP_SUPPRESSION_REASON", "consent_disabled")
        suppressed = [d for d in rule["decisions"] if d["disposition"] == "suppressed" and d["reason"] == reason]
        assert suppressed, rule
        assert not {d["id"] for d in suppressed}.intersection(item["id"] for item in items)
    elif mode == "acknowledge":
        assert items, rule
        item = items[0]
        suffix = "/inbox/" + segment(conversation) + "/" + segment(item["id"]) + "/acknowledge"
        assert request("POST", suffix)[0] == 204
        acknowledged = next(value for value in inbox() if value["id"] == item["id"])
        assert acknowledged["acknowledgedAt"]
        assert request("POST", suffix)[0] == 204
        repeated = next(value for value in inbox() if value["id"] == item["id"])
        assert repeated["acknowledgedAt"] == acknowledged["acknowledgedAt"]
    elif mode == "hidden":
        assert not items, "revoked destination or consent still exposes metadata"
    elif mode != "read":
        raise AssertionError("unknown fixture mode")
    saved["lastObserved"] = rule
    path.write_text(json.dumps(saved, indent=2))
print(json.dumps({"mode": mode, "receipt": str(path), "result": "passed"}))
