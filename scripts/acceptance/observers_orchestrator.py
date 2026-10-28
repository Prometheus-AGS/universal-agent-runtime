"""Read owned AG-UI correlations, actual child threads/results and call groups.

Both supported durable adapters are concrete. No history/prompt/tool bodies are
emitted, and importing this program executes no commands or network requests.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
from urllib.parse import quote

from observers_runtime import Observer, canonical, sha
from observers_startup import ObservationFailure, need, pg_string, secret_ref, session_storage_key


class OrchestratorObserver(Observer):
    def rows(self, store, table, filters, order=None):
        need(table in ("agui_runs", "durable_run_records", "durable_run_events", "agent_threads", "accounting_calls"),
             "unbound observer table")
        need(all(re.fullmatch(r"[a-z_]+", key) for key in filters), "unbound filter column")
        if self.context["backend"] == "postgres_pgvector":
            where = " AND ".join(key + "=" + pg_string(value) for key, value in filters.items())
            ordered = " ORDER BY " + order if order else ""
            result = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM (SELECT data FROM "
                                    + table + " WHERE " + where + ordered + ") AS owned_rows")
        else:
            where = " AND ".join(key + "=" + json.dumps(value) for key, value in filters.items())
            ordered = " ORDER BY " + order + " ASC" if order else ""
            result = store.sql("SELECT VALUE data FROM " + table + " WHERE " + where + ordered + ";")
        return [json.loads(row) if isinstance(row, str) else row for row in result]

    def run_record(self, store, run_id, owner):
        rows = self.rows(store, "durable_run_records", {"run_id": run_id})
        need(len(rows) == 1 and rows[0].get("run_id") == run_id and rows[0].get("owner_key") == owner,
             "actual durable run is absent, ambiguous or outside verified history owner")
        run = rows[0]["run"]
        need(run.get("run_id") == run_id and run.get("status") == "done", "actual run did not complete successfully")
        visible = self.api("/api/uar/runs/" + quote(run_id, safe=""))
        need(visible.get("run_id") == run_id and visible.get("agent_id") == run.get("agent_id")
             and visible.get("conversation_id") == run.get("conversation_id") and visible.get("status") == "done",
             "authenticated mounted inspection does not authorize/correlate the durable run")
        return run

    def session(self, store, owner, session_id):
        key = session_storage_key(owner, session_id)
        if self.context["backend"] == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM sessions WHERE id=" + pg_string(key))
        else:
            rows = store.sql("SELECT * FROM type::record('sessions'," + json.dumps(key) + ");")
            rows = [{**row, "id": row.get("logical_id")} for row in rows]
        need(len(rows) == 1 and rows[0].get("owner_id") == owner and rows[0].get("id") == session_id,
             "actual graph dialogue session is missing or outside the exact principal partition")
        return rows[0]

    def observe_run(self, values):
        self.launch()
        identities = self.fixture.get("orchestrator", {})
        principal = identities.get("expected_principal_key")
        history_owner = identities.get("expected_history_owner_key")
        subject = identities.get("expected_subject")
        need(all(isinstance(value, str) and value for value in (principal, history_owner, subject)),
             "frozen orchestrator authority identities are required")
        need(principal.startswith("v1:k:") and history_owner.startswith("v1:"), "unsupported owner identity encoding")
        need(values.get("agent_id") == "orchestrator-agent", "observer requires exact supported root agent")
        store = self.store()
        rows = self.rows(store, "agui_runs", {"owner_key": principal, "agent_id": values["agent_id"],
                                               "thread_id": values["thread_id"]})
        matches = [row for row in rows if row.get("wire_run_id") == values["wire_run_id"]]
        need(len(matches) == 1, "actual official wire/native correlation absent or ambiguous")
        binding = matches[0]
        need(binding.get("owner") == principal and binding.get("thread_id") == values["thread_id"]
             and binding.get("agent_id") == values["agent_id"] and binding.get("revision", 0) >= 2
             and bool(binding.get("input_digest")), "official binding identity or durable completion missing")
        terminal = binding.get("terminal") or {}
        need(terminal.get("event", {}).get("type") == "RUN_FINISHED"
             and terminal.get("event", {}).get("outcome", {}).get("type") == "success"
             and not terminal.get("interrupt_ids"), "wire terminal is not successful finished inference")
        native = binding["native_run_id"]
        root = self.run_record(store, native, history_owner)
        need(root.get("agent_id") == values["agent_id"] and root.get("conversation_id") == binding["session_id"],
             "wire run did not bind to exact native agent/session")
        events = self.rows(store, "durable_run_events", {"run_id": native}, "event_id")
        selections = [row for row in events if row.get("event", {}).get("type") == "AgentSelected"]
        need(len(selections) == 1, "durable normalized selection is absent or duplicated")
        selected = selections[0]["event"]["data"]
        expected = values["selection"]
        need(selected.get("run_id") == native and selected.get("source_agent_id") == values["agent_id"]
             and selected.get("selected_agent_id") == values["selected_agent_id"]
             and selected.get("selected_thread_id") == values["selected_thread_id"]
             and selected.get("selected_agent_name") == expected["agent"]["title"]
             and selected.get("selection_method") == expected["selectionMethod"] == "intent"
             and selected.get("roster_revision") == expected["rosterRevision"],
             "wire selection differs from actual persisted typed event")
        need(any(row["event_id"] > selections[0]["event_id"] and row.get("event", {}).get("type") == "ChatDelta"
                 for row in events)
             and any(row["event_id"] > selections[0]["event_id"] and row.get("event", {}).get("type") in
                     ("RunDone", "RunDoneWithUsage") for row in events),
             "durable selection does not precede answer and successful terminal")
        need(not any(row.get("event", {}).get("type") in ("Error", "Cancelled") for row in events),
             "root durable history includes failure")
        records = self.rows(store, "agent_threads", {"owner_id": subject, "root_run_id": native})
        children = [row for row in records if row.get("thread", {}).get("thread_id") == selected["selected_thread_id"]]
        need(len(children) == 1, "selected child has no exact persisted owned thread")
        child = children[0]["thread"]
        roots = [row["thread"] for row in records if row.get("thread", {}).get("parent_thread_id") is None]
        need(len(roots) == 1, "actual root thread missing or ambiguous")
        root_thread = roots[0]
        need(root_thread.get("run_id") == native and child.get("root_run_id") == native
             and child.get("root_thread_id") == root_thread["thread_id"]
             and child.get("parent_thread_id") == root_thread["thread_id"]
             and child.get("artifact_id") == values["selected_agent_id"]
             and child.get("status") == "completed" and child.get("run_id") != native,
             "child thread lineage, actual agent, execution identity or completion differs")
        child_run = self.run_record(store, child["run_id"], history_owner)
        need(child_run.get("agent_id") == values["selected_agent_id"], "child native run executed another artifact")
        registered = self.api("/api/agents/" + quote(values["selected_agent_id"], safe=""))
        snapshot = child_run.get("context", {}).get("agent_snapshot", {})
        captured = snapshot.get("artifact", {})
        need(registered.get("id") == child["artifact_id"]
             and registered.get("metadata", {}).get("title") == selected["selected_agent_name"]
             and captured.get("id") == child["artifact_id"] and bool(snapshot.get("revision")),
             "selected current registered definition or actual child execution snapshot differs")
        result = child.get("result") or {}
        need(result.get("kind") == "completed" and isinstance(result.get("output"), str),
             "selected child lacks actual complete result")
        try:
            useful = json.loads(result["output"]) == values["expected_result"]
        except ValueError:
            useful = False
        need(useful, "actual persisted child output fails private arithmetic/fresh fact predicate")
        parent_session = self.session(store, principal, binding["session_id"])
        child_session = self.session(store, principal, child_run["conversation_id"])
        def carries(session):
            return any(message.get("role") == "user" and values["carry"] in json.dumps(message.get("content"), ensure_ascii=False)
                       for message in session.get("messages", []))
        need(carries(parent_session) and carries(child_session), "fresh user fact was not retained in child fork dialogue")
        calls = self.rows(store, "accounting_calls", {"owner_key": principal, "run_id": native})
        need(len(calls) >= 2, "shared root accounting lacks routing and delegated model activity")
        call_ids = []
        safe_calls = []
        for call in calls:
            admission = call.get("admission", {})
            identity = admission.get("identity", {})
            need(identity.get("owner") == principal and identity.get("run_id") == native
                 and bool(identity.get("call_id")) and call.get("state") == "settled"
                 and call.get("observed_tokens", 0) > 0 and bool(call.get("reconciliation_evidence")),
                 "actual shared accounting group contains wrong-owner/unsettled/no-usage call")
            need(admission.get("model") == self.fixture["provider"]["qualified_model"],
                 "actual root accounting used an undeclared model")
            call_ids.append(identity["call_id"])
            safe_calls.append({"identity": identity, "model": admission["model"], "state": call["state"],
                               "observed_tokens": call["observed_tokens"], "scopes": admission.get("scopes")})
        need(len(call_ids) == len(set(call_ids)), "shared group contains duplicate provider call identities")
        # Independent child run/result proves execution. Shared accounting cannot
        # truthfully assign each call to one child; retain this limitation explicitly.
        return {"wire_run_id": binding["wire_run_id"], "wire_thread_id": binding["thread_id"],
                "root_native_run_id": native, "session_id": binding["session_id"],
                "owner": principal, "history_owner": history_owner, "thread_owner": subject,
                "ordinal": binding["ordinal"], "previous_wire_run_id": binding.get("previous_wire_run_id"),
                "binding_revision": binding["revision"], "input_digest": binding["input_digest"],
                "root_thread_id": root_thread["thread_id"], "child_thread_id": child["thread_id"],
                "child_run_id": child["run_id"], "child_agent_id": child["artifact_id"],
                "child_session_id": child_run["conversation_id"], "child_useful": useful,
                "child_execution_revision": snapshot["revision"],
                "registered_revision": registered.get("extensions", {}).get("uar.catalog", {}).get("revision"),
                "child_output_sha256": sha(result["output"].encode()),
                "carry_in_child_user_history": carries(child_session),
                "graph_fork_context": {"source": "actual-owned-canonical-dialogue",
                    "parent_history_sha256": sha(canonical(parent_session.get("messages", []))),
                    "child_history_sha256": sha(canonical(child_session.get("messages", []))),
                    "child_messages": len(child_session.get("messages", []))},
                "selection": selected, "selection_event_id": selections[0]["event_id"],
                "terminal_source_cursor": terminal["source_cursor"],
                "accounting_attribution": "shared_root_group", "root_call_ids": sorted(call_ids),
                "calls": sorted(safe_calls, key=lambda call: call["identity"]["call_id"]),
                "store_identity": store.storage_identity}


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    fixture = json.loads(Path(secret_ref("UAR_ACCEPTANCE_FIXTURE_PATH")).read_bytes())
    path = Path(secret_ref("UAR_ACCEPTANCE_RUNTIME_CONFIG"))
    config = json.loads(path.read_bytes())
    need(config.get("schema_version") == 1 and config.get("executable") is True, "runtime observer config unavailable")
    observer = OrchestratorObserver(config, context, fixture)
    for source in (path, Path(__file__).resolve(), Path(__file__).with_name("observers_runtime.py").resolve(),
                   Path(__file__).with_name("observers_startup.py").resolve()):
        observer.frozen_file(str(source))
    need(len(sys.argv) == 2 and sys.argv[1] == "run", "unknown orchestrator observer action")
    result = observer.observe_run(context.get("values", {}))
    print(json.dumps(observer.redact(result), sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "orchestrator observation failed; no successful evidence",
                          "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure) else "private upstream cause withheld"}))
        sys.exit(2)
