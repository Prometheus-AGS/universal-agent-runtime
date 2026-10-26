"""U14/U15 retained knowledge and real embedding regeneration, U16 per adapter.

No case executes on import. All genuine calls, mutations and process failures
below require the separately approved whole-phase acceptance invocation.
"""
from __future__ import annotations

import copy
import hashlib
import os
import secrets
import signal
import time
import uuid
from urllib.parse import quote

from core import Blocked, Case, Context, Outcome, Predicate, require
from cases_prompt import answer_json, configure_provider, prompt


ROOT = "/api/uar/knowledge-bases"
SETTINGS = "/api/uar/settings"


def path(kb_id):
    return ROOT + "/" + quote(kb_id, safe="")


def spaces(ctx):
    declared = ctx.fixture.get("knowledge", {}).get("embedding_spaces", [])
    if len(declared) != 3 or any(row.get("kind") != "genuine" for row in declared):
        raise Blocked("knowledge requires three frozen genuine embedding spaces A/B/C")
    first, second, third = [row["fingerprint"] for row in declared]
    require(first != second and first["dimension"] == second["dimension"]
            and third["dimension"] != second["dimension"],
            "embedding A/B must differ with equal dimensions; C must change dimension")
    return declared


def select_space(ctx, binding):
    desired = binding["fingerprint"]
    data = {"embedding.backend": desired["backend"], "embedding.model": desired["model"],
            "embedding.vector_dimension": desired["dimension"]}
    revisions = {}
    for field in data:
        current = ctx.request("GET", SETTINGS + "/llm." + field)
        require(current.status == 200, "real embedding setting is unavailable")
        revisions[field] = current.json()["revision"]
    changed = ctx.request("PUT", SETTINGS + "/llm", {"data": data, "expected_revisions": revisions})
    require(changed.status == 200 and changed.json().get("status") == "updated",
            "real global embedding selection was not persisted")
    serving = ctx.request("GET", SETTINGS + "/embedding/provenance")
    require(serving.status == 200 and serving.json().get("status") == "serving"
            and serving.json().get("space") == desired, "runtime did not activate the exact real embedding space")
    # Real frozen production backend plus observed nonzero model vectors is the
    # embedding evidence. The fixture must not use a fake embedding endpoint.
    require(binding.get("synthetic") is False and binding.get("model_source_identity"),
            "embedding binding lacks frozen genuine model source identity")
    if serving.json().get("endpoint"):
        require(binding.get("endpoint") == serving.json()["endpoint"]
                and binding.get("source_kind") == "approved_hosted_endpoint",
                "actual hosted embedding endpoint differs from the frozen approved source")
    else:
        provenance = serving.json().get("provenance") or {}
        require(binding.get("source_kind") == "local_asset_manifest"
                and provenance.get("asset_manifest_sha256") == binding["model_source_identity"],
                "actual local embedding assets differ from the frozen model manifest")
    ctx.record("embedding-selection", {"binding": binding, "serving": serving.json()})
    return desired


def create_kb(ctx):
    name = "acceptance-" + ctx.run_id + "-" + secrets.token_hex(8)
    response = ctx.request("POST", ROOT + "/", {"name": name,
                           "config": {"chunk_strategy": "fixed", "chunk_size": 512,
                                      "retrieval_min_score": 0.0, "retrieval_top_k": 10}})
    require(response.status == 201, "knowledge base create failed")
    return response.json()["id"]


def upload(ctx, kb_id, filename, text, expected=202):
    boundary = "uar-" + secrets.token_hex(16)
    body = ("--" + boundary + '\r\nContent-Disposition: form-data; name="file"; filename="'
            + filename + '"\r\nContent-Type: text/plain\r\n\r\n').encode() + text.encode() \
            + ("\r\n--" + boundary + "--\r\n").encode()
    response = ctx.request_bytes("POST", path(kb_id) + "/documents", body,
                                 "multipart/form-data; boundary=" + boundary,
                                 receipt_body={"filename": filename, "source_bytes": len(text.encode()),
                                     "source_sha256": "sha256:" + hashlib.sha256(text.encode()).hexdigest()})
    require(response.status == expected, "durable multipart upload returned an unexpected result")
    return response.json() if expected == 202 else None


def observe(ctx, kb_id):
    result = ctx.observe("knowledge", {"kb_id": kb_id})
    require(result.get("run_id") == ctx.run_id and result.get("backend") == ctx.backend["name"]
            and result.get("kb_id") == kb_id and result.get("storage_identity"),
            "knowledge observation lacks run/store/KB binding")
    return result


def wait_indexed(ctx, kb_id, documents):
    deadline = time.monotonic() + min(ctx.remaining(), ctx.fixture["knowledge"]["index_seconds"])
    while time.monotonic() < deadline:
        states = [ctx.request("GET", path(kb_id) + "/documents/" + row["id"] + "/ingestion")
                  for row in documents]
        require(all(row.status == 200 for row in states), "accepted durable job disappeared")
        jobs = [row.json() for row in states]
        require(not any(row["state"] == "failed" for row in jobs), "durable ingestion exhausted attempts")
        if all(row["state"] == "indexed" for row in jobs):
            return jobs
        time.sleep(min(0.1, ctx.remaining()))
    raise Blocked("indexing did not settle within the frozen finite allowance")


def verify_vectors(snapshot, expected, sources, generation=None):
    require(snapshot["exists"] and snapshot["config"]["indexed_embedding"] == expected,
            "durable completed index identity differs from actual model")
    by_document = {row["document_id"]: row for row in snapshot["jobs"]}
    require(set(by_document) == set(sources), "retained document cohort changed")
    for document, text in sources.items():
        row = by_document[document]
        require(row["source_sha256"] == "sha256:" + hashlib.sha256(text.encode()).hexdigest()
                and row["source_bytes"] == len(text.encode()) and row["job"]["state"] == "indexed",
                "retained source identity or durable indexed state differs")
        if generation:
            require(row["job"].get("regeneration") == generation, "job belongs to an old cohort")
    chunks = snapshot["chunks"]
    require(chunks and set(row["document_id"] for row in chunks) == set(sources),
            "indexed documents lack actual stored vectors")
    require(all(row["dimension"] == expected["dimension"]
                and row["metadata"].get("embedding_space") == expected for row in chunks),
            "actual vectors contain incompatible dimensions or embedding identities")
    if generation:
        require(all(row["metadata"].get("knowledge_regeneration") == generation for row in chunks),
                "completed KB exposes old generation chunks")
    if snapshot["backend"] == "postgres_pgvector":
        require(snapshot["vector_storage"] == "vector", "Postgres schema still fixes a global vector dimension")


def search(ctx, kb_id, subject, document, code):
    response = ctx.request("POST", path(kb_id) + "/search", {
        "query": "What is the private access code for " + subject + "?", "limit": 5, "min_score": 0.0})
    require(response.status == 200, "knowledge search unavailable")
    results = response.json().get("results", [])
    require(results and results[0].get("document_id") == document and code in results[0]["content"],
            "ranked current document fact was not retrieved")
    return results


def bind_agent(ctx, kb_id):
    response = ctx.request("GET", "/api/agents/default-agent")
    require(response.status == 200, "agent template unavailable")
    artifact = copy.deepcopy(response.json())
    agent_id = "acceptance-knowledge-" + str(uuid.uuid4())
    artifact.update(id=agent_id, version="1.0.0", extensions={})
    artifact["metadata"].update(title=agent_id, description="Run-owned grounded knowledge outcome")
    artifact["runtime"]["entry"] = "default"
    artifact["prompt"] = {"system": 'Answer only from retrieved documents. Return only JSON with keys '
                          'code and citation. Use the exact private code and its numbered source marker '
                          'such as [1]. If the requested subject is absent, return '
                          '{"code":"UNKNOWN","citation":null}. Never infer a missing private code.',
                          "instructions": []}
    artifact["memory"]["kb"].update(enabled=True, knowledge_bases=[kb_id])
    artifact["policy"]["tools"]["allow"] = []
    require(ctx.request("POST", "/api/agents", artifact).status == 201, "grounded agent create failed")
    return agent_id


def grounded(ctx, agent_id, kb_id, subject, document, code):
    result = prompt(ctx, "What is the private access code for " + subject + "?", agent_id=agent_id)
    answer = answer_json(result)
    require(answer.get("code") == code, "real answer did not recover the private document fact")
    events = ctx.request("GET", "/api/uar/runs/" + quote(result["run_id"], safe="") + "/events")
    require(events.status == 200 and events.json().get("gapReason") is None,
            "grounding event history is unavailable or incomplete")
    citations = [citation for event in events.json().get("events", [])
                 if event["eventName"] == "agui.rag_citations"
                 for citation in event["data"].get("citations", [])]
    require(any(citation.get("document_id") == document
                and citation.get("knowledge_base_id") == kb_id and code in citation.get("snippet", "")
                and answer.get("citation") == "[" + str(citation["marker"]) + "]"
                for citation in citations), "answer citation lacks matching real retrieved source provenance")
    return result


def lifecycle(ctx):
    configure_provider(ctx)
    expected = select_space(ctx, spaces(ctx)[0])
    kb_id = create_kb(ctx)
    first_subject, second_subject = "Parcel-" + secrets.token_hex(8), "Depot-" + secrets.token_hex(8)
    first_code, second_code, replacement = [secrets.token_hex(16) for _ in range(3)]
    first_text = f"The private access code for {first_subject} is {first_code}."
    second_text = f"The private access code for {second_subject} is {second_code}."
    first = upload(ctx, kb_id, "parcel.txt", first_text)
    second = upload(ctx, kb_id, "depot.txt", second_text)
    wait_indexed(ctx, kb_id, [first, second])
    sources = {first["id"]: first_text, second["id"]: second_text}
    verify_vectors(observe(ctx, kb_id), expected, sources)
    agent = bind_agent(ctx, kb_id)
    search(ctx, kb_id, first_subject, first["id"], first_code)
    grounded(ctx, agent, kb_id, first_subject, first["id"], first_code)
    grounded(ctx, agent, kb_id, second_subject, second["id"], second_code)
    # The mounted API exposes replacement as delete + upload, not PUT document.
    require(ctx.request("DELETE", path(kb_id) + "/documents/" + first["id"]).status == 204,
            "old document deletion failed")
    updated_text = f"The private access code for {first_subject} is {replacement}."
    updated = upload(ctx, kb_id, "parcel.txt", updated_text)
    wait_indexed(ctx, kb_id, [updated, second])
    sources = {updated["id"]: updated_text, second["id"]: second_text}
    verify_vectors(observe(ctx, kb_id), expected, sources)
    current = search(ctx, kb_id, first_subject, updated["id"], replacement)
    require(all(first_code not in row["content"] for row in current), "replaced fact remains searchable")
    grounded(ctx, agent, kb_id, first_subject, updated["id"], replacement)
    require(ctx.request("DELETE", path(kb_id) + "/documents/" + updated["id"]).status == 204,
            "current document deletion failed")
    verify_vectors(observe(ctx, kb_id), expected, {second["id"]: second_text})
    missing = prompt(ctx, "What is the private access code for " + first_subject + "?", agent_id=agent)
    require(answer_json(missing) == {"code": "UNKNOWN", "citation": None},
            "deleted private fact remained available to a fresh grounded turn")
    grounded(ctx, agent, kb_id, second_subject, second["id"], second_code)
    ctx.restart()
    verify_vectors(observe(ctx, kb_id), expected, {second["id"]: second_text})
    grounded(ctx, agent, kb_id, second_subject, second["id"], second_code)
    require(ctx.request("DELETE", path(kb_id)).status == 204, "KB deletion failed")
    require(observe(ctx, kb_id)["exists"] is False, "deleted KB retained source/index rows")
    require(ctx.request("POST", path(kb_id) + "/search", {"query": second_subject}).status == 404,
            "deleted KB remains searchable")
    ctx.restart()
    require(observe(ctx, kb_id)["exists"] is False, "KB deletion did not survive restart")
    require(ctx.request("DELETE", "/api/agents/" + agent).status == 204, "owned grounded agent cleanup failed")
    return Outcome(Predicate(True, "mounted durable lifecycle completed on this actual adapter"),
                   Predicate(True, "private facts, exact citations, replacement, deletion and cold restart agree"),
                   {"kb_id": kb_id, "database_parity_family": "U16"})


def wait_regeneration(ctx, kb_id, generation, target):
    deadline = time.monotonic() + min(ctx.remaining(), ctx.fixture["knowledge"]["regeneration_seconds"])
    while time.monotonic() < deadline:
        response = ctx.request("GET", path(kb_id) + "/regeneration")
        require(response.status == 200, "durable regeneration status unavailable")
        cohort = response.json()
        require(cohort["generation"] == generation and cohort["target"] == target,
                "regeneration identity changed unexpectedly")
        require(cohort["state"] != "failed", "regeneration exhausted an actual member attempt")
        if cohort["state"] == "complete":
            require(cohort["completed_documents"] == len(cohort["document_ids"]), "false complete progress")
            return cohort
        unavailable = ctx.request("POST", path(kb_id) + "/search", {"query": "private access code"})
        require(unavailable.status == 409 or unavailable.status == 200,
                "in-progress query produced an unexpected response")
        # A concurrent completion may legitimately return 200, but its rows
        # must all carry the target generation; no mixed success is allowed.
        if unavailable.status == 200:
            require(all(row.get("metadata", {}).get("knowledge_regeneration") == generation
                        and row["metadata"].get("embedding_space") == target
                        for row in unavailable.json()["results"]), "search exposed a partial or old vector generation")
        time.sleep(min(0.1, ctx.remaining()))
    raise Blocked("regeneration did not settle within the frozen finite allowance")


def regenerate(ctx, kb_id, previous, binding, sources, interrupt=False):
    target = select_space(ctx, binding)
    incompatible = ctx.request("POST", path(kb_id) + "/search", {"query": "private code"})
    require(incompatible.status == 409, "embedding switch silently searched the old space")
    started = ctx.request("POST", path(kb_id) + "/regeneration")
    require(started.status == 202, "retained-source regeneration was not accepted")
    cohort = started.json()
    require(cohort["target"] == target and cohort["previous"] == previous
            and set(cohort["document_ids"]) == set(sources), "regeneration cohort/target differs")
    generation = cohort["generation"]
    if interrupt:
        unavailable = ctx.request("POST", path(kb_id) + "/search", {"query": "private access code"})
        if unavailable.status != 409:
            raise Blocked("fixture did not retain an unavailable running cohort long enough for interruption")
        # Kill only this invocation's server, then read the durable interrupted
        # state while it is down. Graceful join cannot masquerade as interruption.
        process = ctx.server
        if process is None or process not in ctx.processes or process.poll() is not None:
            raise Blocked("mid-regeneration interruption requires the actual owned live server")
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=ctx.fixture["limits"]["cleanup_seconds"])
        frozen = observe(ctx, kb_id)
        marker = frozen["config"].get("regeneration", {})
        if marker.get("state") != "running" or marker.get("completed_documents", 0) >= len(sources):
            raise Blocked("fixture completed too quickly to establish a real interrupted rebuild")
        require(marker.get("generation") == generation
                and any(row["job"]["state"] in ("pending", "processing", "retryable")
                        for row in frozen["jobs"]), "no durable unfinished generation remained at process death")
        ctx.record("interrupted-regeneration", {"pid": process.pid, "returncode": process.returncode,
                                                "snapshot": frozen})
        ctx.restart()
        live = ctx.request("GET", SETTINGS + "/embedding/provenance")
        require(live.status == 200 and live.json().get("space") == target,
                "restart did not hydrate the durable target embedding model/dimension")
    complete = wait_regeneration(ctx, kb_id, generation, target)
    snapshot = observe(ctx, kb_id)
    verify_vectors(snapshot, target, sources, generation)
    require(snapshot["config"]["regeneration"] == complete,
            "public progress differs from durable completed cohort")
    return target, snapshot


def regeneration(ctx):
    configure_provider(ctx)
    bindings = spaces(ctx)
    expected = select_space(ctx, bindings[0])
    kb_id = create_kb(ctx)
    count = ctx.fixture["knowledge"].get("regeneration_documents", 8)
    if type(count) is not int or not 8 <= count <= 128:
        raise Blocked("regeneration fixture needs 8..128 bounded retained documents")
    documents, sources, facts = [], {}, []
    for index in range(count):
        subject, code = "Cargo-" + secrets.token_hex(10), secrets.token_hex(16)
        # Enough actual chunk work for an honest interruption; no synthetic
        # embeddings, source replacement, job pausing or fabricated readiness.
        text = (f"The private access code for {subject} is {code}.\n" * 32)
        document = upload(ctx, kb_id, "cargo-" + str(index) + ".txt", text)
        documents.append(document)
        sources[document["id"]] = text
        facts.append((subject, document["id"], code))
    wait_indexed(ctx, kb_id, documents)
    initial = observe(ctx, kb_id)
    verify_vectors(initial, expected, sources)
    agent = bind_agent(ctx, kb_id)
    grounded(ctx, agent, kb_id, *facts[0])
    expected, same_dimension = regenerate(ctx, kb_id, expected, bindings[1], sources)
    require({row["embedding_sha256"] for row in initial["chunks"]}
            != {row["embedding_sha256"] for row in same_dimension["chunks"]},
            "same-dimension model switch did not rebuild actual stored vectors")
    search(ctx, kb_id, *facts[0])
    grounded(ctx, agent, kb_id, *facts[0])
    expected, changed_dimension = regenerate(ctx, kb_id, expected, bindings[2], sources, interrupt=True)
    search(ctx, kb_id, *facts[-1])
    grounded(ctx, agent, kb_id, *facts[-1])
    ctx.restart()
    verify_vectors(observe(ctx, kb_id), expected, sources,
                   changed_dimension["config"]["regeneration"]["generation"])
    grounded(ctx, agent, kb_id, *facts[0])
    require(ctx.request("DELETE", path(kb_id)).status == 204
            and ctx.request("DELETE", "/api/agents/" + agent).status == 204,
            "run-owned regeneration resources could not be removed")
    return Outcome(Predicate(True, "durable same/new dimension cohorts completed with actual interruption/restart"),
                   Predicate(True, "retained bytes, real vectors, exact generations and grounded answers recovered"),
                   {"kb_id": kb_id, "database_parity_family": "U16"})


def adapter_contract(ctx):
    """One shared contract; the runner must retain a distinct receipt per store.

    This U16 witness supplements, never replaces, parameterization of every
    other durable outcome family across both mandatory adapters.
    """
    require(ctx.backend["name"] in ("postgres_pgvector", "surreal_disk")
            and ctx.backend.get("durable") is True, "U16 requires an admitted durable adapter")
    configure_provider(ctx)
    expected = select_space(ctx, spaces(ctx)[0])
    kb_id = create_kb(ctx)
    subject, code = "Parity-" + secrets.token_hex(8), secrets.token_hex(16)
    text = f"The private access code for {subject} is {code}."
    document = upload(ctx, kb_id, "parity.txt", text)
    wait_indexed(ctx, kb_id, [document])
    sources = {document["id"]: text}
    agent = bind_agent(ctx, kb_id)
    saved_agent = ctx.request("GET", "/api/agents/" + agent)
    require(saved_agent.status == 200, "U16 saved agent unavailable")
    before = observe(ctx, kb_id)
    verify_vectors(before, expected, sources)
    search(ctx, kb_id, subject, document["id"], code)
    grounded(ctx, agent, kb_id, subject, document["id"], code)
    old_pid = ctx.server.pid
    ctx.restart()
    require(ctx.server.pid != old_pid, "U16 did not cross an owned process boundary")
    after = observe(ctx, kb_id)
    verify_vectors(after, expected, sources)
    require(before["storage_identity"] == after["storage_identity"]
            and before["owner_id"] == after["owner_id"] and before["config"] == after["config"]
            and before["documents"] == after["documents"] and before["jobs"] == after["jobs"]
            and sorted(before["chunks"], key=lambda row: row["id"])
                == sorted(after["chunks"], key=lambda row: row["id"]),
            "U16 logical records, retained source, vectors or revision changed across restart")
    agent_after = ctx.request("GET", "/api/agents/" + agent)
    require(agent_after.status == 200 and agent_after.json() == saved_agent.json(),
            "U16 persisted grounding agent revision changed across restart")
    ranked = search(ctx, kb_id, subject, document["id"], code)
    result = grounded(ctx, agent, kb_id, subject, document["id"], code)
    receipt = {"contract": "knowledge-retained-grounding-v1", "backend": ctx.backend["name"],
               "storage_identity": after["storage_identity"], "kb_id": kb_id,
               "document_id": document["id"], "source_sha256": after["jobs"][0]["source_sha256"],
               "indexed_embedding": expected, "actual_dimensions": sorted({row["dimension"] for row in after["chunks"]}),
               "ranked_document_id": ranked[0]["document_id"], "expected_code": code,
               "actual_answer": answer_json(result), "old_pid": old_pid, "new_pid": ctx.server.pid,
               "same_logical_records_and_revision": True}
    ctx.record("adapter-contract", receipt)
    require(ctx.request("DELETE", path(kb_id)).status == 204
            and ctx.request("DELETE", "/api/agents/" + agent).status == 204,
            "U16 owned durable records could not be removed")
    return Outcome(Predicate(True, "this actual durable adapter preserved the common logical contract"),
                   Predicate(True, "same retained facts and exact citations remain useful after real restart"), receipt)


def cases():
    return [Case("U14.knowledge-lifecycle", "U14", lifecycle, durable=True),
            Case("U15.embedding-regeneration", "U15", regeneration, durable=True),
            Case("U16.knowledge-adapter-contract", "U16", adapter_contract, durable=True)]
