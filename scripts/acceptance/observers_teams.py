"""Read actual owned catalog, threads, tool receipts and context consumption.

No private instructions/history/raw tool payload is printed. Source-only until
an exact whole-phase execution packet is approved.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys

from observers_startup import Store, ObservationFailure, need, pg_string, secret_ref


def sha(value):
    return "sha256:" + hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def decode(value):
    return json.loads(value) if isinstance(value, str) else value


def observe(store, context):
    values = context["values"]
    team_id = values["team_id"]
    if store.kind == "postgres_pgvector":
        catalog = store.pg_query("SELECT data FROM uar_collaboration_state WHERE id='catalog'")
    else:
        rows = store.sql("SELECT * FROM type::record('uar_collaboration_state','catalog');")
        need(len(rows) == 1, "durable team catalog absent")
        catalog = rows[0]
    teams = [row for row in catalog["teamInstances"].values() if row["id"] == team_id]
    need(len(teams) == 1, "team identity is absent or ambiguous")
    team = teams[0]
    need(team["input"].get("acceptance_run_id") == context["run_id"], "team is outside this invocation")
    owner, workspace = team["ownerId"], team["workspaceId"]
    need(workspace == values["workspace_id"] and owner == values["owner_id"], "team owner/workspace binding differs")
    def owned(row):
        authority = row.get("authority", row.get("scope", row))
        return (authority.get("ownerId"), authority.get("workspaceId"), authority.get("teamId")) == (owner, workspace, team_id)
    attempts = [row for row in catalog.get("teamExecutionAttempts", {}).values() if owned(row)]
    run_ids = [row["runId"] for row in attempts]
    if not run_ids:
        threads, canonical = [], []
    elif store.kind == "postgres_pgvector":
        ids = ",".join(pg_string(value) for value in run_ids)
        threads = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM agent_threads WHERE root_run_id IN (" + ids + ")")
        canonical = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM canonical_tool_receipts WHERE run_id IN (" + ids + ")")
    else:
        ids = json.dumps(run_ids)
        threads = [decode(row) for row in store.sql("SELECT VALUE data FROM agent_threads WHERE root_run_id IN " + ids + ";")]
        canonical = [decode(row) for row in store.sql("SELECT VALUE data FROM canonical_tool_receipts WHERE run_id IN " + ids + ";")]
    thread_rows = []
    for record in threads:
        thread = record["thread"]
        need(thread["root_run_id"] in run_ids, "observed thread belongs to another run")
        attempt = next(row for row in attempts if row["runId"] == thread["root_run_id"])
        need(thread["root_thread_id"] == attempt["rootId"], "persisted team ancestry differs")
        result = thread.get("result") or {}
        thread_rows.append({key: thread[key] for key in ("thread_id", "root_thread_id", "root_run_id",
            "parent_thread_id", "canonical_path", "run_id", "status", "history_revision")}
            | {"revision": record["revision"], "result_kind": result.get("kind"),
               "yield_control": result.get("control") if result.get("kind") == "yielded" else None})
    tool_rows = []
    for row in canonical:
        root = next((record["thread"] for record in threads
                     if record["thread"]["root_run_id"] == row["run_id"]
                     and record["thread"]["parent_thread_id"] is None), None)
        need(root and row["owner_id"] == root["owner_id"], "canonical tool owner differs from its actual root")
        tool_rows.append({key: row.get(key) for key in ("run_id", "sequence", "call_id", "tool", "source",
                         "outcome", "completeness", "typed_value_sha256")})
    contexts = [row for row in catalog.get("teamContextReceipts", {}).values() if owned(row)]
    messages = [row for row in catalog.get("teamPeerMessages", {}).values() if owned(row)]
    message_ids = {row["messageId"] for row in messages}
    deliveries = [row for row in catalog.get("teamMessageDeliveries", {}).values() if row["messageId"] in message_ids]
    # Context source IDs, byte counts and consumption times establish provenance;
    # emitting a different member's prompt or private raw history is unnecessary.
    safe_contexts = [{key: row[key] for key in ("authority", "rootId", "approvalScopeId", "instructionOrder",
                        "self", "coordinatorMemberId", "roster", "authorizationRevision", "selections",
                        "targetOutcomes", "targetOutcomeDataTrust", "countQuality")} for row in contexts]
    safe_attempts = []
    for row in attempts:
        safe = {key: row.get(key) for key in ("id", "runId", "rootId", "approvalScopeId", "ownerId",
            "workspaceId", "teamId", "taskId", "memberId", "memberRevision", "ownershipEpoch",
            "bindingRevision", "executionEpoch", "executionFence", "status", "executionOutcome", "effectDisposition",
            "accountingState", "contextArtifactIds", "continuationOfWaitId", "effectiveModels", "createdAt", "updatedAt")}
        safe["output_digest"] = sha(row.get("output"))
        safe_attempts.append(safe)
    artifacts = [row for row in catalog.get("teamArtifacts", {}).values() if owned(row)]
    sessions = []
    for attempt in attempts:
        session_id = "team-attempt:" + attempt["id"]
        if store.kind == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM sessions WHERE data->>'id'="
                                  + pg_string(session_id))
        else:
            rows = store.sql("SELECT * FROM sessions WHERE logical_id=" + json.dumps(session_id)
                             + ";")
        need(len(rows) <= 1, "attempt session is ambiguous")
        if not rows:
            continue
        session = rows[0]
        need(session.get("owner_id", "").startswith("v1:k:"), "attempt session lacks credential-principal ownership")
        need(all(row.get("principal_key") in (None, session["owner_id"]) for row in canonical
                 if row["run_id"] == attempt["runId"]), "tool receipt credential principal differs from attempt session")
        calls, results, selected_artifacts, selected_messages = [], [], set(), set()
        for index, message in enumerate(session["messages"]):
            for call in message.get("tool_calls") or []:
                calls.append({"id": call["id"], "name": call["function"]["name"], "position": index})
            if message.get("tool_call_id"):
                results.append({"id": message["tool_call_id"], "position": index})
            if message.get("role") == "user" and isinstance(message.get("content"), str):
                try:
                    data = json.loads(message["content"])
                except ValueError:
                    continue
                if isinstance(data, dict) and "taskInput" in data and "artifacts" in data:
                    selected_artifacts.update(row["id"] for row in data["artifacts"])
                    selected_messages.update(row["messageId"] for row in data.get("messages", []))
        sentinel = values.get("foreign_sentinel")
        sessions.append({"attempt_id": attempt["id"], "session_id": session_id,
                         "principal_owner_id": session["owner_id"],
                         "history_digest": sha(session["messages"]), "message_count": len(session["messages"]),
                         "calls": calls, "results": results,
                         "context_artifact_ids": sorted(selected_artifacts),
                         "context_message_ids": sorted(selected_messages),
                         "foreign_sentinel_present": bool(sentinel and sentinel in json.dumps(session))})
    return {"run_id": context["run_id"], "backend": context["backend"], "storage_identity": store.storage_identity,
            "team_id": team_id, "owner_id": owner, "workspace_id": workspace, "team_revision": team["revision"],
            "attempts": safe_attempts, "threads": thread_rows, "canonical_tools": tool_rows,
            "contexts": safe_contexts, "deliveries": deliveries, "sessions": sessions,
            "messages": [{key: row[key] for key in ("messageId", "senderMemberId", "senderAttemptId", "recipientMemberId", "mode")}
                         | {"payload_digest": sha(row["payload"])} for row in messages],
            "artifacts": [{key: row[key] for key in ("id", "taskId", "memberId", "attemptId")}
                          | {"content_digest": sha(row["content"])} for row in artifacts],
            "waits": [row for row in catalog.get("teamWaits", {}).values() if owned(row)],
            "continuations": [row for row in catalog.get("teamContinuations", {}).values() if owned(row)],
            "peer_commands": [row for row in catalog.get("teamPeerCommands", {}).values() if owned(row)],
            "execution_claim": catalog.get("executionClaim"),
            "fencing_evidence": [{key: row[key] for key in ("id", "fence", "rootAndChildrenStopped", "source", "recordedAt")}
                                 for row in catalog.get("executionFencingEvidence", {}).values()]}


def main():
    need(len(sys.argv) == 1, "team observer takes no unbound actions")
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    config = json.loads(Path(secret_ref("UAR_ACCEPTANCE_STARTUP_CONFIG")).read_text())
    print(json.dumps(observe(Store(config, context), context), sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "durable team observation failed", "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure)
                          else "upstream cause withheld because it may contain credentials"}))
        sys.exit(2)
