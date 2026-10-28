"""Read-only retained-data observations for U14/U15 on the two real adapters.

Use the frozen UAR_ACCEPTANCE_STARTUP_CONFIG Store connection contract. This
module performs no observation on import. Execution needs an approved packet.
"""
from __future__ import annotations

import hashlib
import json
import math
import sys
import uuid
from pathlib import Path

from observers_startup import Store, ObservationFailure, need, pg_string, secret_ref


def hashed(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def snapshot(store, context):
    values = context["values"]
    kb_id = str(uuid.UUID(values["kb_id"]))
    # Mounted authenticated knowledge is physically disjoint from legacy rows.
    if store.kind == "postgres_pgvector":
        result = store.pg_query("SELECT json_build_object('bases',COALESCE((SELECT json_agg(b) "
            "FROM principal_knowledge_bases b WHERE id=" + pg_string(kb_id) + "),'[]'),"
            "'documents',COALESCE((SELECT json_agg(d) FROM principal_knowledge_documents d WHERE kb_id="
            + pg_string(kb_id) + "),'[]'),'jobs',COALESCE((SELECT json_agg(json_build_object("
            "'owner_id',owner_id,'document_id',document_id,'kb_id',kb_id,'data',data,"
            "'source_hex',encode(source,'hex'))) FROM principal_knowledge_ingestion WHERE kb_id="
            + pg_string(kb_id) + "),'[]'),'chunks',COALESCE((SELECT json_agg(json_build_object("
            "'owner_id',owner_id,'id',id,'kb_id',kb_id,'document_id',document_id,'content',content,"
            "'metadata',metadata,'embedding',embedding::text)) FROM principal_knowledge_chunks WHERE kb_id="
            + pg_string(kb_id) + "),'[]'),'vector_storage',(SELECT format_type(a.atttypid,a.atttypmod) "
            "FROM pg_attribute a WHERE a.attrelid='principal_knowledge_chunks'::regclass "
            "AND a.attname='embedding'))")
    else:
        # A single read statement binds the KB marker and vectors together.
        result = store.sql("RETURN {bases:(SELECT * FROM principal_knowledge_bases "
            "WHERE logical_id=" + json.dumps(kb_id) + "),documents:(SELECT * FROM principal_knowledge_documents "
            "WHERE kb_id=" + json.dumps(kb_id) + "),jobs:(SELECT * FROM principal_knowledge_ingestion WHERE kb_id="
            + json.dumps(kb_id) + "),chunks:(SELECT * FROM principal_knowledge_chunks WHERE kb_id="
            + json.dumps(kb_id) + "),vector_storage:'array'};")
        if not isinstance(result, dict):
            raise ObservationFailure("Surreal snapshot did not return its transactional read value")
    bases = result["bases"]
    if not bases:
        need(not result["documents"] and not result["jobs"] and not result["chunks"],
             "deleted KB retains orphan durable rows")
        return {"kb_id": kb_id, "exists": False, "documents": [], "jobs": [], "chunks": []}
    need(len(bases) == 1, "run-owned KB has ambiguous physical ownership")
    kb = bases[0]
    need(kb["name"].startswith("acceptance-" + context["run_id"] + "-"),
         "observer refuses a KB outside the invocation's name scope")
    owner = kb["owner_id"]
    need(owner.startswith("v1:k:"), "knowledge observer requires a verified principal partition")
    docs = []
    for row in result["documents"]:
        need(row["owner_id"] == owner and row["kb_id"] == kb_id, "document scope differs")
        docs.append({"id": row.get("logical_id", row["id"]), "filename": row["filename"],
                     "status": row["status"], "chunk_count": row["chunk_count"]})
    jobs = []
    for row in result["jobs"]:
        job = json.loads(row["data"]) if isinstance(row["data"], str) else row["data"]
        need(row["owner_id"] == owner and job["owner_id"] == owner
             and job["kb_id"] == kb_id and job["document_id"] == row["document_id"],
             "retained source/job scope differs")
        source = bytes.fromhex(row["source_hex"]) if "source_hex" in row else bytes(row["source"])
        jobs.append({"document_id": row["document_id"], "job": job,
                     "source_sha256": hashed(source), "source_bytes": len(source)})
    chunks = []
    for row in result["chunks"]:
        need(row["owner_id"] == owner and row["kb_id"] == kb_id, "chunk scope differs")
        vector = json.loads(row["embedding"]) if isinstance(row["embedding"], str) else row["embedding"]
        need(vector and all(type(value) in (int, float) and math.isfinite(value) for value in vector)
             and any(value != 0 for value in vector), "stored embedding is empty, nonfinite or zero")
        chunks.append({"id": row.get("logical_id", row["id"]), "document_id": row["document_id"],
                       "dimension": len(vector), "embedding_sha256": hashed(json.dumps(vector).encode()),
                       "content_sha256": hashed(row["content"].encode()), "metadata": row["metadata"]})
    return {"kb_id": kb_id, "exists": True, "owner_id": owner, "config": kb["config"],
            "documents": sorted(docs, key=lambda row: row["id"]),
            "jobs": sorted(jobs, key=lambda row: row["document_id"]), "chunks": chunks,
            "vector_storage": result["vector_storage"]}


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    config = json.loads(Path(secret_ref("UAR_ACCEPTANCE_STARTUP_CONFIG")).read_text())
    store = Store(config, context)
    need(len(sys.argv) == 1, "knowledge observer takes no unbound actions")
    value = snapshot(store, context)
    print(json.dumps({"run_id": context["run_id"], "backend": context["backend"],
                      "storage_identity": store.storage_identity, **value}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "knowledge retained-data observation failed",
                          "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure)
                          else "upstream cause withheld because it may contain credentials"}))
        sys.exit(2)
