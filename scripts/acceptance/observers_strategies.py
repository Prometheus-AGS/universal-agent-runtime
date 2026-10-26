"""Concrete U09–13 observers over the exact owned runtime and durable store.

No operation executes on import. Runtime observer configuration, this program
and imported observer programs must be frozen by the approved packet.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import sys
from urllib.parse import quote
import uuid

from observers_runtime import Observer, canonical, sha
from observers_startup import ObservationFailure, need, pg_string, secret_ref


class StrategyObserver(Observer):
    def record(self, run_id):
        self.launch()
        store = self.store()
        if self.context["backend"] == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM durable_run_records WHERE run_id="
                                  + pg_string(run_id))
        else:
            rows = store.sql("SELECT VALUE data FROM durable_run_records WHERE run_id=" + json.dumps(run_id) + ";")
            rows = [json.loads(row) if isinstance(row, str) else row for row in rows]
        need(len(rows) == 1 and rows[0].get("run_id") == run_id and rows[0].get("owner_key"),
             "actual owner-partitioned run record unavailable or ambiguous")
        run = rows[0].get("run", {})
        need(run.get("run_id") == run_id, "durable run identity differs")
        return run, store

    def run(self, values):
        run_id = values["run_id"]
        run, store = self.record(run_id)
        context = run.get("context", {})
        inspection = self.api("/api/uar/runs/" + quote(run_id, safe=""))
        need(inspection.get("run_id") == run_id and inspection.get("status") == run.get("status"),
             "mounted and durable run identities or terminal states differ")
        result = {"run_id": run_id, "session_id": run.get("conversation_id"),
                  "status": run.get("status"), "agent_id": run.get("agent_id"),
                  "store_identity": store.storage_identity}
        activations = context.get("activated_skills")
        if isinstance(activations, list):
            result["activated_skills"] = activations
        budget = context.get("request_budget")
        if isinstance(budget, dict):
            result["request_budget"] = budget
        matching = context.get("skill_matching")
        if isinstance(matching, dict):
            fields = ("requested_method", "effective_method", "requested_model", "effective_model",
                      "threshold", "margin", "top_k")
            result["skill_matching"] = {key: matching.get(key) for key in fields}
            result["skill_matching"]["candidates"] = [
                {key: candidate.get(key) for key in ("skill_id", "score", "accepted")}
                for candidate in matching.get("candidates", context.get("skill_candidates", []))]
        receipt = context.get("context_reduction")
        if isinstance(receipt, dict):
            fields = ("schema_version", "requested_strategy", "source_sha256", "prepared_sha256",
                      "source_messages", "prepared_messages", "source_tokens", "prepared_tokens",
                      "eligible_spans", "changed", "summary_generated", "final_sha256", "final_messages",
                      "final_tokens", "input_allowance", "protected_groups", "protected_groups_preserved")
            result["context_reduction"] = {key: receipt.get(key) for key in fields}
            result["context_reduction"]["tier_calls"] = [
                {key: call.get(key) for key in ("tier", "model", "completed", "input_tokens", "output_tokens",
                                               "input_sha256", "output_sha256", "count_quality", "counting_revision")}
                for call in receipt.get("tier_calls", [])]
        calls = self.api("/api/uar/accounting/calls/" + quote(run_id, safe=""))
        need(isinstance(calls, list), "actual provider accounting unavailable")
        result["calls"] = [{"call_id": call.get("admission", {}).get("identity", {}).get("call_id"),
                            "model": call.get("admission", {}).get("model"), "state": call.get("state"),
                            "observed_tokens": call.get("observed_tokens"),
                            "reconciliation_evidence": bool(call.get("reconciliation_evidence"))}
                           for call in calls]
        return result

    def budget(self, values):
        response = values.get("response", {})
        completion_id = response.get("id", "")
        if completion_id.startswith("chatcmpl-"):
            run_id = completion_id.removeprefix("chatcmpl-")
        else:
            rows = self.api("/api/uar/runs")
            need(isinstance(rows, list), "mounted run catalog unavailable")
            need(isinstance(values.get("session_id"), str) and values["session_id"],
                 "refused request requires its unique caller-owned session correlation")
            matches = [row for row in rows if row.get("conversation_id") == values.get("session_id")]
            need(len(matches) == 1, "refused request has no unique actual run correlation")
            run_id = matches[0]["run_id"]
        observed = self.run({"run_id": run_id})
        budget = observed.get("request_budget", {})
        need(budget.get("count_quality") in ("approximate", "exact", "validated_upper_bound"),
             "actual destination lacks counting-quality evidence")
        calls = observed["calls"]
        result = {"kind": values["kind"], "run_id": run_id, "count_quality": budget["count_quality"],
                  "counted_input_tokens": budget.get("counted_input_tokens"),
                  "input_allowance": budget.get("input_allowance"),
                  "provider_calls": len(calls),
                  "settled_calls": sum(call.get("state") == "settled" for call in calls)}
        if values["kind"] == "exact_boundary":
            content = response.get("choices", [{}])[0].get("message", {}).get("content", "")
            try:
                result["useful_result"] = json.loads(content) == values["expected_json"]
            except (ValueError, TypeError):
                result["useful_result"] = False
        else:
            # These are retained boundary outcomes, not fixture declarations.
            _, store = self.record(run_id)
            if self.context["backend"] == "postgres_pgvector":
                effects = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM canonical_tool_receipts WHERE run_id=" + pg_string(run_id))
                admissions = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM tool_admission_evidence WHERE run_id="
                    + pg_string(run_id) + " OR root_run_id=" + pg_string(run_id))
            else:
                def decode(rows):
                    return [json.loads(row) if isinstance(row, str) else row for row in rows]
                effects = decode(store.sql("SELECT VALUE data FROM canonical_tool_receipts WHERE run_id=" + json.dumps(run_id) + ";"))
                admissions = decode(store.sql("SELECT VALUE data FROM tool_admission_evidence WHERE run_id="
                    + json.dumps(run_id) + " OR root_run_id=" + json.dumps(run_id) + ";"))
            need(isinstance(effects, list) and isinstance(admissions, list), "actual durable host admission/effect tables unavailable")
            result["host_effects"] = len(effects)
            result["host_admissions"] = len(admissions)
            result["effect_evidence_source"] = "durable_tool_admission_and_canonical_receipt_tables"
            result["refused_before_dispatch"] = (budget.get("outcome") == "protected_overflow"
                and budget.get("failure_code") in ("context_overflow", "protected_history_overflow")
                and budget.get("representation") == "protected_preparation_candidate"
                and observed["status"] == "error" and not calls and not effects and not admissions)
        return result

    def session_markers(self, values):
        store = self.store()
        metadata = store.session(str(uuid.UUID(values["session_id"])))
        need(metadata is not None, "actual owned canonical session missing")
        session = store.session_data(metadata["owner_id"], metadata["session_id"])
        need(session is not None and isinstance(session.get("messages"), list), "canonical session messages missing")
        messages = session["messages"]
        encoded = canonical(messages)
        markers = values.get("markers", [])
        need(isinstance(markers, list) and len(markers) <= 30
             and all(isinstance(marker, str) and 0 < len(marker) <= 256 for marker in markers),
             "bounded caller sentinel list required")
        calls, results, multimodal = [], [], 0
        for message in messages:
            calls.extend(call.get("id") for call in message.get("tool_calls") or [])
            if message.get("role") == "tool":
                results.append(message.get("tool_call_id"))
            if isinstance(message.get("content"), list):
                multimodal += 1
            elif isinstance(message.get("content"), dict) and message["content"].get("type") == "parts":
                multimodal += 1
        need(len(calls) == len(set(calls)) and set(calls) == set(results) and len(results) == len(set(results)),
             "canonical tool calls/results are orphaned or duplicated")
        return {"session_id": metadata["session_id"], "owner_id": metadata["owner_id"],
                "store_identity": store.storage_identity, "canonical_sha256": sha(encoded),
                "messages": len(messages), "marker_present": {marker: marker.encode() in encoded for marker in markers},
                "tool_calls": calls, "tool_results": results, "multimodal_messages": multimodal}

    def skill_scope(self, values):
        self.launch()
        skill_id = values["skill_id"]
        need(re.fullmatch(r"agents::acceptance-[0-9a-f]{32}", skill_id),
             "scope observation requires the exact run-owned discovery fixture identity")
        manifest = self.root / ".agents" / "skills" / skill_id.removeprefix("agents::") / "SKILL.md"
        need(manifest.is_file() and not manifest.is_symlink(), "owned discovery manifest unavailable")
        session_id = str(uuid.UUID(values["session_id"]))
        store = self.store()
        if self.context["backend"] == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(definition),'[]') FROM skills WHERE skill_id="
                                  + pg_string(skill_id))
        else:
            rows = store.sql("SELECT * FROM skills WHERE skill_id=" + json.dumps(skill_id) + ";")
        need(len(rows) == 1 and rows[0].get("skill_id") == skill_id,
             "actual durable discovery skill missing or duplicated")
        own = [row for row in rows[0].get("scoped_config", [])
               if row.get("scope") == {"kind": "conversation", "id": session_id}]
        need(len(own) == 1, "exact owned conversation scope missing or duplicated")
        return {"skill_id": skill_id, "session_id": session_id, "enabled": own[0].get("enabled"),
                "store_identity": store.storage_identity}

    def seed_discovery(self):
        self.owned()
        need(self.config.get("server_environment_env", {}).get("HOME") == "UAR_ACCEPTANCE_OWNED_STATE_DIR",
             "discovery requires HOME bound to the exact run-owned directory")
        need(self.generated_config().get("skill_source_roots") == [],
             "standard discovery fixture must exclude ambient project skill roots")
        token = uuid.uuid4().hex
        root = self.root / ".agents" / "skills"
        root.mkdir(parents=True, mode=0o700, exist_ok=True)
        top = "acceptance-" + token
        alias = "linked-" + token
        target = self.root / ("skill-target-" + token)
        manifests = [(root / top, "owned-top"), (root / top / "skills" / "nested", "owned-nested"),
                     (target, "owned-linked")]
        files = []
        for directory, name in manifests:
            directory.mkdir(parents=True, mode=0o700, exist_ok=False)
            path = directory / "SKILL.md"
            raw = f"---\nname: {name}\ndescription: Exact owned discovery fixture {token}.\n---\n\nHarmless discovery fixture.\n".encode()
            with path.open("xb") as stream:
                stream.write(raw)
            files.append({"relative_path": str(path.relative_to(self.root)), "sha256": sha(raw)})
        (root / alias).symlink_to(target, target_is_directory=True)
        ignored = root / ("ignored-" + token)
        ignored.mkdir(mode=0o700)
        (ignored / "NOT_A_SKILL.md").write_text("This file is not a skill manifest.\n")
        return {"expected_ids": ["agents::" + top, "agents::" + top + "/skills/nested", "agents::" + alias],
                "excluded_id": "agents::ignored-" + token, "files": files,
                "one_hop_alias": str((root / alias).relative_to(self.root)), "owned_home": str(self.root)}

    def discovery(self, values):
        self.launch()
        skills = self.api("/api/uar/skills/")
        need(isinstance(skills, list), "actual skill catalog unavailable")
        selected = []
        for expected in values["expected_ids"]:
            rows = [row for row in skills if row.get("skill_id") == expected]
            need(len(rows) == 1 and rows[0].get("provider_id") == "agent-skills",
                 "expected standard skill missing, duplicated or from wrong provider")
            row = rows[0]
            selected.append({key: row.get(key) for key in ("skill_id", "artifact_digest", "provider_id", "origin", "enabled")})
        need(not any(row.get("skill_id") == values["excluded_id"] for row in skills),
             "nonmanifest confounder was admitted as a skill")
        return {"skills": selected, "source": "actual-mounted-registry", "excluded_id": values["excluded_id"]}


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    fixture = json.loads(Path(secret_ref("UAR_ACCEPTANCE_FIXTURE_PATH")).read_bytes())
    path = Path(secret_ref("UAR_ACCEPTANCE_RUNTIME_CONFIG"))
    config = json.loads(path.read_bytes())
    need(config.get("schema_version") == 1 and config.get("executable") is True, "observer config is not executable schema1")
    observer = StrategyObserver(config, context, fixture)
    for source in (path, Path(__file__).resolve(), Path(__file__).with_name("observers_runtime.py").resolve(),
                   Path(__file__).with_name("observers_startup.py").resolve()):
        observer.frozen_file(str(source))
    action = sys.argv[1] if len(sys.argv) == 2 else ""
    values = context.get("values", {})
    if action == "run":
        result = observer.run(values)
    elif action == "budget":
        result = observer.budget(values)
    elif action == "session":
        result = observer.session_markers(values)
    elif action == "skill-scope":
        result = observer.skill_scope(values)
    elif action == "seed-discovery":
        result = observer.seed_discovery()
    elif action == "discovery":
        result = observer.discovery(values)
    else:
        raise ObservationFailure("unknown concrete strategy observer action")
    print(json.dumps(observer.redact(result), sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "strategy observer failed; no successful observation",
                          "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure) else "private upstream error withheld"}))
        sys.exit(2)
