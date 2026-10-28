"""Concrete initial-attachment metadata and body-free history observation.

No code executes on import. Raw skill/history/prompt text is never emitted.
"""
from __future__ import annotations

import json
from pathlib import Path
import re
import sys
from urllib.parse import quote
import uuid

from observers_runtime import Observer, canonical, sha
from observers_startup import ObservationFailure, need, pg_string
from observers_startup import secret_ref


class RetentionObserver(Observer):
    def observe_run(self, values):
        self.launch()
        run_id = values["run_id"]
        session_id = str(uuid.UUID(values["session_id"]))
        skill_ids = values["skill_ids"]
        need(isinstance(skill_ids, list) and len(skill_ids) == 2 and len(set(skill_ids)) == 2
             and all(isinstance(value, str) and re.fullmatch(r"acceptance-retention-[ab]-[0-9a-f]{16}", value) for value in skill_ids),
             "exact two owned long skill identities required")
        sentinels = values["sentinels"]
        need(isinstance(sentinels, list) and len(sentinels) == 4
             and all(isinstance(marker, str) and re.fullmatch(r"body-(head|tail)-[0-9a-f]{32}", marker) for marker in sentinels),
             "exact owned body sentinel set required")
        store = self.store()
        if self.context["backend"] == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM durable_run_records WHERE run_id=" + pg_string(run_id))
        else:
            rows = store.sql("SELECT VALUE data FROM durable_run_records WHERE run_id=" + json.dumps(run_id) + ";")
            rows = [json.loads(row) if isinstance(row, str) else row for row in rows]
        need(len(rows) == 1 and rows[0].get("run_id") == run_id and rows[0].get("owner_key"),
             "actual owner-partitioned run record missing or ambiguous")
        run = rows[0]["run"]
        visible = self.api("/api/uar/runs/" + quote(run_id, safe=""))
        need(run.get("run_id") == visible.get("run_id") == run_id and run.get("status") == visible.get("status") == "done"
             and run.get("conversation_id") == visible.get("conversation_id") == session_id,
             "authenticated mounted run does not correlate to completed durable row")
        receipt = run.get("context", {}).get("skill_reattachment")
        need(isinstance(receipt, dict) and receipt.get("schema_version") == 1
             and receipt.get("scope") == "initial_assembly" and receipt.get("count_quality") == "approximate",
             "actual content-free initial attachment receipt missing")
        fragments = receipt.get("fragments")
        need(isinstance(fragments, list) and {row.get("skill_id") for row in fragments} == set(skill_ids),
             "actual receipt lacks exactly the selected owned skill bodies")
        checked = []
        for skill_id in skill_ids:
            if self.context["backend"] == "postgres_pgvector":
                definitions = store.pg_query("SELECT COALESCE(json_agg(definition),'[]') FROM skills WHERE skill_id=" + pg_string(skill_id))
            else:
                definitions = store.sql("SELECT * FROM skills WHERE skill_id=" + json.dumps(skill_id) + ";")
            need(len(definitions) == 1 and definitions[0].get("skill_id") == skill_id,
                 "actual owned current skill definition absent or ambiguous")
            skill = definitions[0]
            expected = f'[SKILL: {skill["title"]}]\n{skill["prompt_overlay"]}'
            fragment = next(row for row in fragments if row["skill_id"] == skill_id)
            need(fragment.get("original_body_sha256") == sha(expected.encode()),
                 "receipt original body does not match the actual current durable skill")
            checked.append(skill_id)
        metadata = store.session(session_id)
        need(metadata is not None, "actual owned canonical session missing")
        session = store.session_data(metadata["owner_id"], session_id)
        need(session is not None and isinstance(session.get("messages"), list), "actual canonical history unavailable")
        encoded = canonical(session["messages"])
        presence = {marker: marker.encode() in encoded for marker in sentinels}
        need(not any(presence.values()), "canonical history retained an injected skill body sentinel")
        activated = run.get("context", {}).get("activated_skills")
        need(isinstance(activated, list) and set(skill_ids).issubset(activated), "actual owned skills were not activated")
        # Allow only the implemented receipt fields, never arbitrary run context.
        fields = ("schema_version", "scope", "model", "count_quality", "counting_revision", "per_skill_tokens", "total_tokens",
                  "context_limit", "reserve_tokens", "request_input_limit", "history_tokens", "request_tokens", "rendered_tokens", "rendered_sha256")
        fragment_fields = ("fragment_id", "skill_id", "original_body_tokens", "retained_body_tokens", "original_body_sha256",
                           "retained_body_sha256", "original_rendered_tokens", "retained_rendered_tokens",
                           "original_rendered_sha256", "retained_rendered_sha256", "retained", "truncated")
        safe = {key: receipt.get(key) for key in fields}
        safe["fragments"] = [{key: row.get(key) for key in fragment_fields} for row in fragments]
        return {"run_id": run_id, "session_id": session_id, "store_identity": store.storage_identity,
                "receipt": safe, "original_hashes_match": set(checked) == set(skill_ids),
                "history_body_free": not any(presence.values()), "history_sha256": sha(encoded),
                "history_messages": len(session["messages"]), "sentinel_present": presence}


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    fixture = json.loads(Path(secret_ref("UAR_ACCEPTANCE_FIXTURE_PATH")).read_bytes())
    path = Path(secret_ref("UAR_ACCEPTANCE_RUNTIME_CONFIG"))
    config = json.loads(path.read_bytes())
    need(config.get("schema_version") == 1 and config.get("executable") is True, "runtime observer configuration required")
    observer = RetentionObserver(config, context, fixture)
    for source in (path, Path(__file__).resolve(), Path(__file__).with_name("observers_runtime.py").resolve(),
                   Path(__file__).with_name("observers_startup.py").resolve()):
        observer.frozen_file(str(source))
    need(len(sys.argv) == 2 and sys.argv[1] == "run", "unknown retention observer action")
    print(json.dumps(observer.redact(observer.observe_run(context.get("values", {}))), sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "skill retention observation failed; no successful evidence",
                          "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure) else "private upstream cause withheld"}))
        sys.exit(2)
