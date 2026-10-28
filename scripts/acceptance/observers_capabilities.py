"""Concrete tool/file, run, memory and narrow store-fault observations.

Source authoring only; every read/mutation below requires the future approved
packet. No arbitrary external observer command or success label is accepted.
"""
from __future__ import annotations

import base64
import json
import os
from pathlib import Path
import re
import secrets
import sqlite3
import sys
import urllib.error
from urllib.parse import quote
import uuid

from observers_runtime import canonical, sha
from observers_strategies import StrategyObserver
from observers_startup import ObservationFailure, need, pg_string, save, secret_ref
from tools_fixture import root_path, target, color_image


class CapabilitiesObserver(StrategyObserver):
    def effects(self):
        self.owned()
        return root_path(str(self.root / "tools-effects"))

    def response(self, path):
        try:
            return 200, self.api(path)
        except urllib.error.HTTPError as error:
            return error.code, None

    def prepare_effect(self, values):
        root = self.effects()
        name = values["name"]
        path = target(root, name)
        fact, number = secrets.token_hex(16), secrets.randbelow(900) + 100
        initial = canonical({"fact": fact, "number": number}) + b"\n"
        with path.open("xb") as stream:
            stream.write(initial)
            stream.flush()
            os.fsync(stream.fileno())
        save(root / (name + ".expected.json"), {"initial_base64": base64.b64encode(initial).decode(),
                                                "fact": fact, "number": number})
        return {"name": name, "path": str(path), "root": str(root), "fact": fact, "number": number,
                "initial_sha256": sha(initial), "initial_bytes": len(initial)}

    def effect(self, values):
        root = self.effects()
        name, marker = values["name"], values["marker"]
        need(re.fullmatch(r"[a-f0-9]{24}", marker), "invalid exact-owned effect marker")
        expected = json.loads((root / (name + ".expected.json")).read_text())
        initial = base64.b64decode(expected["initial_base64"])
        actual = target(root, name).read_bytes()
        need(len(actual) <= 1024 * 1024, "owned effect output exceeds allowance")
        receipt_path = root / "mcp-receipts.jsonl"
        receipts = []
        if receipt_path.exists():
            raw = receipt_path.read_bytes()
            need(len(raw) <= 2 * 1024 * 1024, "effect receipt allowance exhausted")
            receipts = [json.loads(line) for line in raw.splitlines() if line]
        selected = [row for row in receipts if row.get("name") == name and row.get("marker") == marker]
        return {"name": name, "path": str(target(root, name)), "sha256": sha(actual),
                "unchanged": actual == initial, "exactly_one_append": actual == initial + marker.encode() + b"\n",
                "marker_occurrences": actual.count(marker.encode()), "mcp_receipts": selected}

    def run_evidence(self, values):
        run_id = str(uuid.UUID(values["run_id"]))
        run, store = self.record(run_id)
        session_id = run.get("conversation_id")
        need(isinstance(session_id, str), "capability run lacks actual session")
        metadata = store.session(session_id)
        need(metadata is not None, "actual durable capability session absent")
        session = store.session_data(metadata["owner_id"], session_id)
        need(session is not None, "actual durable capability history absent")
        if store.kind == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM canonical_tool_receipts WHERE run_id=" + pg_string(run_id))
            admission = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM tool_admission_evidence WHERE root_run_id=" + pg_string(run_id))
        else:
            rows = [json.loads(row) if isinstance(row, str) else row for row in store.sql(
                "SELECT VALUE data FROM canonical_tool_receipts WHERE run_id=" + json.dumps(run_id) + ";")]
            admission = [json.loads(row) if isinstance(row, str) else row for row in store.sql(
                "SELECT VALUE data FROM tool_admission_evidence WHERE root_run_id=" + json.dumps(run_id) + ";")]
        calls, results = [], []
        image_messages = 0
        for index, message in enumerate(session["messages"]):
            content = message.get("content")
            if isinstance(content, list) and any(part.get("type") == "image_url" for part in content):
                image_messages += 1
            for call in message.get("tool_calls") or []:
                args = json.loads(call["function"]["arguments"])
                calls.append({"id": call["id"], "name": call["function"]["name"], "position": index,
                              "arguments": args})
            if message.get("tool_call_id"):
                results.append({"id": message["tool_call_id"], "position": index,
                                "content_sha256": sha(canonical(content))})
        receipts = []
        for row in rows:
            need(row["run_id"] == run_id and row.get("principal_key") in (None, metadata["owner_id"]),
                 "tool receipt principal differs from actual session")
            typed = row.get("typed_value")
            receipts.append({key: row.get(key) for key in ("run_id", "call_id", "tool", "outcome", "completeness", "typed_value_sha256")}
                | {"status": typed.get("status") if isinstance(typed, dict) else None,
                   "failure": typed.get("failure", {}).get("code") if isinstance(typed, dict) and isinstance(typed.get("failure"), dict) else None})
        events = self.api("/api/uar/runs/" + run_id + "/events")
        return {"run_id": run_id, "session_id": session_id, "store_identity": store.storage_identity,
                "status": run.get("status"), "calls": calls, "results": results, "canonical_tools": receipts,
                "tool_admission": admission,
                "image_messages": image_messages, "message_count": len(session["messages"]),
                "history_sha256": sha(canonical(session["messages"])), "events": events,
                "context_reduction": run.get("context", {}).get("context_reduction"),
                "activated_skills": run.get("context", {}).get("activated_skills"),
                "skill_matching": run.get("context", {}).get("skill_matching")}

    def provider_ready(self, values):
        need(type(values["pid"]) is int and values["pid"] > 1, "invalid owned provider process identity")
        path = self.effects() / ("provider-fault-" + str(values["pid"]) + ".json")
        need(path.is_file(), "owned provider fault listener has not bound yet")
        value = json.loads(path.read_text())
        need(value["pid"] == values["pid"] and value["origin"].startswith("http://127.0.0.1:"),
             "negative provider fixture identity differs")
        trace = self.effects() / "mcp-receipts.jsonl"
        rows = [json.loads(line) for line in trace.read_text().splitlines()] if trace.exists() else []
        return {**value, "provider_denial_calls": len([row for row in rows if row.get("operation") == "provider-denial"])}

    def isolation(self, values):
        session = str(uuid.UUID(values["session_id"]))
        store = self.store()
        if store.kind == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM sessions WHERE data->>'id'=" + pg_string(session))
        else:
            rows = store.sql("SELECT * FROM sessions WHERE logical_id=" + json.dumps(session) + ";")
        owners = [row.get("owner_id") for row in rows]
        need(all(isinstance(owner, str) and owner.startswith("v1:k:") for owner in owners),
             "isolated conversations lack credential-principal ownership")
        return {"session_id": session, "storage_identity": store.storage_identity, "owner_count": len(set(owners)),
                "records": [{"owner_id": row["owner_id"], "history_sha256": sha(canonical(row["messages"])),
                             "private_marker_present": values["secret"] in json.dumps(row["messages"])} for row in rows]}

    def store_fault(self, values):
        name = values["name"]
        need(isinstance(name, str) and name.startswith("acceptance-" + self.context["run_id"] + "-"),
             "store fault may target only the exact new run-owned KB name")
        store = self.store()
        operation = values["operation"]
        need(operation in ("install", "remove", "read"), "unsupported bounded store fault action")
        if store.kind == "postgres_pgvector":
            if operation == "install":
                store.pg_query("CREATE FUNCTION uar_acceptance_write_refusal() RETURNS trigger LANGUAGE plpgsql AS $body$ "
                    "BEGIN IF NEW.name=" + pg_string(name) + " THEN RAISE EXCEPTION 'uar_acceptance_write_refused'; "
                    "END IF; RETURN NEW; END $body$; CREATE TRIGGER uar_acceptance_write_refusal BEFORE INSERT "
                    "ON principal_knowledge_bases FOR EACH ROW EXECUTE FUNCTION uar_acceptance_write_refusal(); "
                    "SELECT json_build_object('installed',true)")
            elif operation == "remove":
                store.pg_query("DROP TRIGGER uar_acceptance_write_refusal ON principal_knowledge_bases; "
                               "DROP FUNCTION uar_acceptance_write_refusal(); SELECT json_build_object('removed',true)")
            rows = store.pg_query("SELECT COALESCE(json_agg(json_build_object('id',id,'name',name)),'[]') "
                                  "FROM principal_knowledge_bases WHERE name=" + pg_string(name))
        else:
            if operation == "install":
                store.sql("DEFINE EVENT uar_acceptance_write_refusal ON principal_knowledge_bases "
                          "WHEN $event='CREATE' AND $after.name=" + json.dumps(name)
                          + " THEN { THROW 'uar_acceptance_write_refused'; };")
            elif operation == "remove":
                store.sql("REMOVE EVENT uar_acceptance_write_refusal ON principal_knowledge_bases;")
            rows = store.sql("SELECT logical_id,name FROM principal_knowledge_bases WHERE name=" + json.dumps(name) + ";")
        return {"name": name, "operation": operation, "storage_identity": store.storage_identity,
                "records": rows, "backend": self.context["backend"]}

    def memory(self, values):
        launch = self.launch()
        config = json.loads(Path(launch["config_path"]).read_text())
        memory = config.get("memory", {})
        need(memory.get("enabled") is True and memory.get("surreal_endpoint") is None,
             "actual memory service is unavailable or uses an unowned remote store")
        path = Path(memory["db_path"]).resolve()
        need(path.is_relative_to(self.root.resolve()) and path != self.root.resolve(), "memory disk escaped owned scope")
        files = list(path.rglob("*")) if path.is_dir() else [path]
        total = sum(item.stat().st_size for item in files if item.is_file())
        need(total > 0, "actual memory disk has no persisted bytes")
        policy = self.api("/api/memory/policy")
        memory_id = values["id"]
        status, row = self.response("/api/memory/records/" + quote(memory_id, safe=""))
        control_status, control = self.response("/api/memory/records/" + quote(memory_id, safe="") + "/control")
        return {"id": memory_id, "record_status": status, "record_sha256": sha(canonical(row)) if row else None,
                "content_sha256": sha(row["content"].encode()) if row else None,
                "owner_id": row.get("user_id") if row else None, "policy": policy,
                "control_status": control_status, "control": control,
                "storage_identity": sha(canonical({"path": str(path), "namespace": memory["namespace"], "database": memory["database"]})),
                "disk_bytes": total, "server_pid": launch["pid"]}

    def image(self, values):
        colors = secrets.SystemRandom().sample(["red", "green", "blue", "yellow"], 3)
        image = color_image(colors)
        path = self.effects() / (values["name"] + ".png")
        need(re.fullmatch(r"[a-z0-9-]{1,80}", values["name"]), "invalid owned image name")
        with path.open("xb") as stream:
            stream.write(image)
        return {"colors": colors, "sha256": sha(image), "path": str(path),
                "data_url": "data:image/png;base64," + base64.b64encode(image).decode()}

    def wire(self, values):
        root = self.effects()
        trace = root / "mcp-receipts.jsonl"
        need(trace.is_file() and trace.stat().st_size <= 2 * 1024 * 1024, "actual genuine wire receipts unavailable or oversized")
        rows = [json.loads(line) for line in trace.read_text().splitlines()]
        expected = values["response_format"]
        name = expected["json_schema"]["name"]
        requests = [row for row in rows if row.get("operation") == "genuine-wire"
                    and (row.get("response_format") or {}).get("json_schema", {}).get("name") == name]
        need(requests, "actual provider wire did not carry the unique schema contract")
        approved = self.fixture["capabilities"]["genuine_upstream"]
        from budget_proxy import owned_hop
        cap = self.fixture["capabilities"]
        need(cap.get("structured_proxy_budget_hop_approved") is True, "structured budget hop approval missing")
        config, binding, ready = owned_hop(cap["structured_proxy_budget_config"], approved)
        ledger_path = Path(config["ledger_root"]) / "ledger.sqlite3"
        with sqlite3.connect(ledger_path.as_uri() + "?mode=ro", uri=True) as db:
            reservations = [(request_id, state, json.loads(receipt)) for request_id, state, receipt
                            in db.execute("SELECT request_id,state,receipt FROM reservations WHERE binding_id=?", (binding["id"],))]
        results = []
        for row in requests:
            need(row["response_format"] == expected and row["upstream"] == approved
                 and row["model"] == values["wire_model"]
                 and ((row.get("max_tokens") or 0) > 0 or (row.get("max_completion_tokens") or 0) > 0),
                 "actual forwarded schema/model/approved origin/output ceiling differs")
            response = next((reply for reply in rows if reply.get("operation") == "genuine-wire-response"
                             and reply["request_id"] == row["request_id"]), None)
            need(response and response["status"] == 200 and response["bytes"] > 0,
                 "genuine upstream did not return an actual complete successful response")
            hop = row.get("budget_hop") or {}
            need(hop.get("binding_id") == binding["id"] and hop.get("input_identity") == ready["epoch"]["input_identity"]
                 and hop.get("owner_pid") == ready["pid"] and hop.get("owner_nonce") == ready["nonce"],
                 "metadata proxy did not forward through the approved owned budget listener")
            matches = [(request_id, receipt) for request_id, state, receipt in reservations if state == "completed"
                       and receipt.get("request_wire_sha256") == row["request_sha256"]
                       and receipt.get("model") == row["model"] and receipt.get("http_status") == response["status"]
                       and receipt.get("response_sha256") == response["sha256"]
                       and receipt.get("response_bytes") == response["bytes"]
                       and receipt.get("upstream_url") == binding["upstream_url"]
                       and receipt.get("owner_nonce") == ready["nonce"]]
            need(len(matches) == 1, "actual approved HTTPS forwarding receipt is absent or ambiguous")
            results.append({"request": row, "response": response, "budget_reservation_id": matches[0][0],
                            "approved_upstream_receipt": matches[0][1]})
        return {"schema_name": name, "actual_genuine_wire": results}


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    fixture = json.loads(Path(secret_ref("UAR_ACCEPTANCE_FIXTURE_PATH")).read_bytes())
    config = json.loads(Path(secret_ref("UAR_ACCEPTANCE_RUNTIME_CONFIG")).read_bytes())
    observer = CapabilitiesObserver(config, context, fixture)
    observer.frozen_file(str(Path(__file__).resolve()))
    observer.frozen_file(str(Path(__file__).resolve().with_name("tools_fixture.py")))
    action = sys.argv[1] if len(sys.argv) == 2 else ""
    operation = getattr(observer, action, None)
    need(action in ("prepare_effect", "effect", "run_evidence", "provider_ready", "store_fault", "memory", "image", "isolation", "wire")
         and callable(operation), "unknown concrete capability observer action")
    value = operation(context.get("values", {}))
    print(json.dumps(observer.redact({"run_id": context["run_id"], "backend": context["backend"], **value}), sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "capability observation failed", "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure)
                          else "upstream cause withheld because it may contain credentials"}))
        sys.exit(2)
