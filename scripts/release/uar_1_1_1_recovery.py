"""Installed-image recovery scenarios. Importing this module performs no I/O."""
import csv
import hashlib
import io
import json
from pathlib import Path
import sqlite3
import sys
import urllib.parse
import uuid

from uar_1_1_1_support import Blocked, require

REQUIRED_SCENARIOS = {"recovery": (
    "unattested_remote_upload_refusal", "attested_upload_restart_owner_isolation",
    "joined_quiesce_reclaim_controls",
    "qualified_supervisor_reclaim_controls",
)}
SOURCE = b"UAR v1.1.1 durable recovery fixture source."
TABLES = ("knowledge_bases", "knowledge_documents", "knowledge_chunks", "knowledge_ingestion")
OWNER_PATH = "/api/v1/collaboration/execution-owner"
CHALLENGE_PATH = "/api/uar/recovery/supervisor/challenges"
FENCE_FIELDS = ("catalogId", "serviceInstanceId", "incarnationId", "epoch")


def _data(value):
    return value["data"] if isinstance(value, dict) and set(value) == {"data"} else value


def _digest(value):
    if not isinstance(value, bytes):
        value = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(value).hexdigest()


def _literal(value, backend):
    if backend == "surreal":
        return json.dumps(value)
    return "'" + str(value).replace("'", "''") + "'"


def _database(config):
    persistence = config["persistence"]
    return persistence.get("surreal_db") or persistence["database_url"].rsplit("/", 1)[-1]


def _sql(context, backend, config, statement):
    resources = context["resources"]
    if backend == "postgres":
        return resources.postgres_query(_database(config), statement)
    result = resources.surreal_query(_database(config), statement, namespace="uar111")
    require(isinstance(result, list) and all(row.get("status") == "OK" for row in result), "Surreal fixture statement failed")
    return result


def _rows(context, backend, config, table, where=None):
    # All table names and predicates are fixture source, never manifest SQL.
    suffix = " WHERE " + where if where else ""
    if backend == "surreal":
        result = _sql(context, backend, config, "SELECT * FROM " + table + suffix + ";")
        require(len(result) == 1 and isinstance(result[0].get("result"), list), "Surreal row projection missing")
        rows = result[0]["result"]
    else:
        result = _sql(context, backend, config, "SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json)::text AS row FROM " + table + " t" + suffix + ";")
        csv_rows = list(csv.reader(io.StringIO(result)))
        require(len(csv_rows) >= 2 and csv_rows[0] == ["row"], "PostgreSQL row projection missing")
        rows = json.loads(csv_rows[1][0])
        require(isinstance(rows, list), "PostgreSQL rows missing")
    return sorted(rows, key=lambda row: json.dumps(row, sort_keys=True))


def _snapshot(context, backend, config, principal=False, owner=None):
    where = None if owner is None else "owner_id = " + _literal(owner, backend)
    return {table: _rows(context, backend, config, ("principal_" if principal else "") + table, where) for table in TABLES}


def _inventory(rows):
    return {table: {"count": len(values), "digest": _digest(values)} for table, values in rows.items()}


def _multipart():
    boundary = "uar111-" + uuid.uuid4().hex
    body = ("--%s\r\nContent-Disposition: form-data; name=\"file\"; filename=\"recovery.txt\"\r\nContent-Type: text/plain\r\n\r\n" % boundary).encode()
    return body + SOURCE + ("\r\n--%s--\r\n" % boundary).encode(), "multipart/form-data; boundary=" + boundary


def _kb(http, name):
    value = _data(http.request("POST", "/api/knowledge", {"name": name}, expected=201))
    require(isinstance(value, dict) and isinstance(value.get("id"), str), "knowledge base id missing")
    return value["id"]


def _upload(http, kb, expected):
    body, content_type = _multipart()
    return _data(http.request("POST", "/api/knowledge/%s/documents" % kb, body, expected=expected, headers={"Content-Type": content_type}))


def _unattested(context, owner):
    config = context["base_config"](backend="surreal")
    runtime = context["new_runtime"]("recovery-unattested", config, env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "0"})
    runtime.start()
    try:
        http = runtime.http(owner)
        kb = _kb(http, "unattested-" + uuid.uuid4().hex)
        before = _snapshot(context, "surreal", config, principal=True)
        _upload(http, kb, 503)
        require(_data(http.request("GET", "/api/knowledge/%s/documents" % kb)) == [], "refused upload created a document")
        after = _snapshot(context, "surreal", config, principal=True)
        require(before == after, "refused upload mutated durable rows")
        for table in TABLES[1:]:
            require(after[table] == [], "refused upload retained source/job/document/chunk")
        return {"status": 503, "knowledge_base": kb, "unchanged_inventory": _inventory(after)}
    finally:
        runtime.stop()


def _seed_legacy(context, backend, config, base):
    # These are the same four legacy tables/record forms as
    # tests/principal_knowledge_namespace.rs. The owner is the exact verified
    # principal key read from this run's real principal KB, not a guessed subject.
    owner, kb = base["owner_id"], base.get("logical_id", base.get("id"))
    document, chunk = str(uuid.uuid4()), str(uuid.uuid4())
    now = "2026-10-16T00:00:00Z"
    job = dict(owner_id=owner, document_id=document, kb_id=kb, generation="legacy-generation",
               state="pending", attempts=0, next_attempt_at=0, last_error=None)
    q = lambda value: _literal(value, backend)
    if backend == "surreal":
        key = "%s:%s:%s" % (len(owner.encode()), owner, kb)
        doc_key = "%s:%s:%s" % (len(owner.encode()), owner, document)
        payloads = [
            (TABLES[0], key, dict(logical_id=kb, owner_id=owner, name="legacy-private", config=base["config"], created_at=now, updated_at=now)),
            (TABLES[1], doc_key, dict(logical_id=document, owner_id=owner, kb_id=kb, filename="legacy.txt", mime_type="text/plain", status="pending", chunk_count=0, created_at=now, updated_at=now)),
            (TABLES[2], chunk, dict(logical_id=chunk, owner_id=owner, kb_id=kb, document_id=document, content="legacy-private", embedding=[1.0] * 384, created_at=now)),
            (TABLES[3], doc_key, dict(owner_id=owner, document_id=document, kb_id=kb, state="pending", next_attempt_at=0, data=json.dumps(job), source=list(b"legacy-source"))),
        ]
        sql = "BEGIN TRANSACTION; " + " ".join("CREATE type::record(%s,%s) CONTENT %s;" % (q(table), q(key), json.dumps(payload)) for table, key, payload in payloads) + " COMMIT TRANSACTION;"
    else:
        sql = "BEGIN; "
        sql += "INSERT INTO knowledge_bases(owner_id,id,name,config) VALUES(%s,%s,'legacy-private',%s::jsonb); " % (q(owner), q(kb), q(json.dumps(base["config"])))
        sql += "INSERT INTO knowledge_documents(owner_id,id,kb_id,filename,mime_type) VALUES(%s,%s,%s,'legacy.txt','text/plain'); " % (q(owner), q(document), q(kb))
        sql += "INSERT INTO knowledge_chunks(owner_id,id,kb_id,document_id,content,embedding) VALUES(%s,%s,%s,%s,'legacy-private',%s::vector); " % (q(owner), q(chunk), q(kb), q(document), q(json.dumps([1.0] * 384)))
        sql += "INSERT INTO knowledge_ingestion(owner_id,document_id,kb_id,state,next_attempt_at,data,source) VALUES(%s,%s,%s,'pending',0,%s::jsonb,decode(%s,'hex')); COMMIT;" % (q(owner), q(document), q(kb), q(json.dumps(job)), q(b"legacy-source".hex()))
    _sql(context, backend, config, sql)
    snapshot = _snapshot(context, backend, config, owner=owner)
    require(all(len(rows) == 1 for rows in snapshot.values()), "legacy fixture did not persist four rows")
    return document, snapshot


def _committed(context, backend, config, kb, document, owner, indexed=False):
    rows = _snapshot(context, backend, config, principal=True)
    require(len(rows[TABLES[0]]) == len(rows[TABLES[1]]) == len(rows[TABLES[3]]) == 1, "upload must retain one KB, document and job")
    doc, job = rows[TABLES[1]][0], rows[TABLES[3]][0]
    require(doc.get("logical_id", doc.get("id")) == document and doc["kb_id"] == kb, "document identity changed")
    require(job["document_id"] == document and job["kb_id"] == kb and doc["owner_id"] == job["owner_id"] == owner, "commit owner/job mismatch")
    source = bytes(job["source"]) if backend == "surreal" else bytes.fromhex(job["source"].removeprefix("\\x"))
    require(source == SOURCE, "atomic upload source bytes changed")
    data = json.loads(job["data"]) if isinstance(job["data"], str) else job["data"]
    require(data["document_id"] == document and data["owner_id"] == owner and data["namespace"] == "principal_v1", "job provenance changed")
    # Embedding fingerprint: every persisted chunk vector must equal the exact
    # vector the configured model returned; a silent model/dimension change or
    # a rewrite across restart changes the fingerprint. Asserted only after
    # indexing has legitimately completed: before the planned restart the peer
    # deliberately withholds the embedding response, so chunks cannot exist yet.
    fingerprint = [1.0] + [0.0] * 383
    chunks = rows[TABLES[2]]
    if indexed:
        require(chunks, "committed principal chunks missing")
        for row in chunks:
            vector = row.get("embedding")
            vector = json.loads(vector) if isinstance(vector, str) else vector
            require(isinstance(vector, list) and [float(component) for component in vector] == fingerprint,
                    "persisted embedding fingerprint differs from the configured model's vector")
    return rows, data


def _seed_accounting(context, backend, config, owner):
    # Same durable-seed pattern as _seed_legacy, for the accounting store's own
    # table. The record identity uses the exact verified principal key.
    scope_id = "acct-" + uuid.uuid4().hex
    identity = {"owner": owner, "scope": "session", "scope_id": scope_id}
    key = "%d:%s:%d:%s:%s" % (len(owner.encode()), owner, len("session"), "session", scope_id)
    record = dict(identity=identity, revision=0, generation=1,
                  limits={"cost_usd": 0.25, "tokens": 1000}, settled_cost_usd=0.0,
                  settled_tokens=0, reserved_cost_usd=0.0, reserved_tokens=0,
                  unknown_calls=0, pending_calls=0, reconciled=True, reconciliation_evidence=None)
    if backend == "surreal":
        _sql(context, backend, config, "CREATE type::record('accounting_scopes', %s) SET data=%s;" % (
            _literal(key, backend), _literal(json.dumps(record), backend)))
    else:
        _sql(context, backend, config, "INSERT INTO accounting_scopes(key,data) VALUES(%s,%s::jsonb);" % (
            _literal(key, backend), _literal(json.dumps(record), backend)))
    found = _rows(context, backend, config, "accounting_scopes")
    require(len(found) == 1, "accounting scope seed did not persist")
    return scope_id, record


def _embedding(context, config):
    resources = context["resources"]
    root = resources.root / resources.unique_name("embedding")
    root.mkdir(mode=0o700)
    peer = resources.start_event_process([sys.executable, str(Path(__file__).resolve()), "--embedding-peer", str(root)], label="recovery-embedding")
    port = peer.wait("ready").get("port")
    require(isinstance(port, int) and 0 < port < 65536, "embedding peer listener identity missing")
    url = resources.relay(port)
    config["llm"] = {"embedding": {"backend": "openai", "model": "uar111-recovery-vector", "api_key": "isolated-embedding-fixture", "base_url": url + "/v1", "vector_dimension": 384}}
    return peer


def _cold_boot(context, runtime, backend):
    runtime.stop()
    resources = context["resources"]
    resources.restart_backend(backend)
    runtime.start()


def _await_indexed(http, kb, document, generation):
    # This owner-authorized route subscribes before its initial authoritative
    # snapshot. Reconciled status frames are fresh reads after gaps, never raw
    # privileged live-query rows. No unscoped internal topic is exposed here.
    path = "/api/knowledge/%s/documents/%s/ingestion/events" % (kb, document)
    with http.subscribe(path) as subscription:
        while True:
            frame = subscription.next()
            payload = frame["data"]
            require(payload.get("version") == 1 and payload.get("topic") == "runtime_principal_ingestion_jobs",
                    "unsupported ingestion observation contract")
            if frame["event"] == "ingestion.gap":
                require(payload.get("requires_snapshot") is True, "ingestion gap omitted reconciliation requirement")
                if payload.get("reason") not in {"consumer_lagged", "lifecycle_transition"} or payload.get("ready") is False:
                    raise Blocked("ingestion subscription unavailable; completion remains unconfirmed")
                # The endpoint follows this event with an owner-authorized
                # reconciled snapshot. Continue reading that explicit event.
                continue
            require(frame["event"] == "ingestion.status", "unexpected ingestion event")
            job = payload.get("job")
            require(isinstance(job, dict) and job.get("document_id") == document and job.get("kb_id") == kb,
                    "ingestion snapshot does not bind the requested resource")
            require(job.get("generation") == generation, "restart replaced the admitted ingestion generation")
            require(job.get("state") != "failed", "ingestion terminal failure")
            if job.get("state") == "indexed":
                return job


def _attested(context, backend, owner, outsiders):
    config = context["base_config"](backend=backend)
    peer = _embedding(context, config)
    runtime = context["new_runtime"]("recovery-" + backend, config, env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"})
    runtime.start()
    try:
        http = runtime.http(owner)
        kb = _kb(http, "attested-" + backend + "-" + uuid.uuid4().hex)
        base = _snapshot(context, backend, config, principal=True)[TABLES[0]][0]
        legacy_document, legacy_before = _seed_legacy(context, backend, config, base)
        # Accounting retention is proven against the accounting store's own
        # table and the authoritative accounting API, across the cold boot.
        accounting_scope_id, accounting_record = _seed_accounting(context, backend, config, base["owner_id"])
        readback = http.request("GET", "/api/uar/accounting/scopes/session/" + accounting_scope_id, expected=200)
        require(readback == accounting_record, "accounting scope API readback differs from the seeded record")
        for suffix, method, body in (("", "GET", None), ("/ingestion", "GET", None), ("/retry", "POST", {})):
            http.request(method, "/api/knowledge/%s/documents/%s%s" % (kb, legacy_document, suffix), body, expected=404)
        document = _upload(http, kb, 202)["id"]
        peer.wait("contact")
        before, initial_job = _committed(context, backend, config, kb, document, base["owner_id"])
        require(initial_job["state"] in ("pending", "processing", "retryable"), "restart fixture did not interrupt a pending ingestion")
        # The peer holds the real HTTP response until the old runtime is dead.
        # A new dispatcher must rediscover the already committed job.
        runtime.force_stop_for_fault()
        peer.send("release")
        peer.wait("released")
        _cold_boot(context, runtime, backend)
        http = runtime.http(owner)
        job = _await_indexed(http, kb, document, initial_job["generation"])
        require(job["generation"] == initial_job["generation"], "restart created a replacement job")
        found = _data(http.request("POST", "/api/knowledge/%s/search" % kb, {"query": SOURCE.decode(), "limit": 5, "min_score": 0.0}))
        require(any(row.get("document_id") == document and SOURCE.decode() in row.get("content", "") for row in found["results"]), "same owner retrieval missing")
        require(all(isinstance(row.get("document_id"), str) and row["document_id"] for row in found["results"]), "grounded retrieval result missing its document citation")
        require(all("legacy-private" not in row.get("content", "") for row in found["results"]), "legacy content was adopted")
        denials = {}
        for label, token, status in outsiders:
            outsider = runtime.http(token)
            values = []
            for suffix, method, body in (("", "GET", None), ("/ingestion", "GET", None), ("/retry", "POST", {})):
                values.append(outsider.request(method, "/api/knowledge/%s/documents/%s%s" % (kb, document, suffix), body, expected=status, return_status=True)[0])
            values.append(outsider.request("POST", "/api/knowledge/%s/search" % kb, {"query": "recovery", "limit": 5, "min_score": 0.0}, expected=status, return_status=True)[0])
            denials[label] = values
        after, cold_job = _committed(context, backend, config, kb, document, base["owner_id"], indexed=True)
        require(cold_job["state"] == "indexed" and after[TABLES[2]], "cold principal publication missing")
        require(_snapshot(context, backend, config, owner=base["owner_id"]) == legacy_before, "legacy rows changed during restart/reupload")
        # Repeat cold boot after publication: observe retained final state, not a
        # digest sampled before the claimed restart.
        accounting_before = _rows(context, backend, config, "accounting_scopes")
        _cold_boot(context, runtime, backend)
        http = runtime.http(owner)
        final, final_job = _committed(context, backend, config, kb, document, base["owner_id"], indexed=True)
        require(final == after and final_job == cold_job, "indexed principal state changed across cold restart")
        legacy_after = _snapshot(context, backend, config, owner=base["owner_id"])
        require(legacy_after == legacy_before, "legacy rows changed on cold readback")
        accounting_after = _rows(context, backend, config, "accounting_scopes")
        require(accounting_after == accounting_before, "accounting scopes changed across cold restart")
        retained = http.request("GET", "/api/uar/accounting/scopes/session/" + accounting_scope_id, expected=200)
        require(retained == accounting_record, "accounting scope API readback changed across cold restart")
        accounting_denials = {}
        for label, token, status in outsiders:
            accounting_denials[label] = runtime.http(token).request("GET", "/api/uar/accounting/scopes/session/" + accounting_scope_id, expected=status, return_status=True)[0]
        return {"backend": backend, "document": document, "job_generation": job["generation"],
                "job_state": final_job["state"], "source_digest": _digest(SOURCE), "denials": denials,
                "committed_inventory": _inventory(before), "cold_inventory": _inventory(final),
                "embedding_fingerprint": "sha256:" + hashlib.sha256(json.dumps([1.0] + [0.0] * 383).encode()).hexdigest(),
                "grounded_citation": "every retrieval result carried its source document_id",
                "accounting_retention": {"scope": accounting_scope_id, "rows_unchanged_across_cold_boot": True,
                    "api_readback_unchanged": True, "outsider_denials": accounting_denials},
                "legacy_cold_readback": {"backend": backend, "legacy_document": legacy_document,
                    "before": _inventory(legacy_before), "after_cold_boot": _inventory(legacy_after),
                    "unchanged": True}}
    finally:
        runtime.stop()


def _fence(claim):
    return {key: claim[key] for key in FENCE_FIELDS}


def _reclaim_body(claim, replacement, evidence):
    return {"commandId": str(uuid.uuid4()), "catalogId": claim["catalogId"], "expectedEpoch": claim["epoch"], "replacementServiceInstanceId": replacement["serviceInstanceId"], "reason": "isolated phase recovery", "fencingEvidenceRef": evidence}


def _joined(context, owner):
    config = context["base_config"](backend="surreal")
    config["service_instance"] = {"instance_id": "uar111-joined-" + uuid.uuid4().hex}
    first = context["new_runtime"]("joined-first", config, env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"})
    second = context["new_runtime"]("joined-second", config, env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"})
    try:
        first.start()
        second.start()
        admin = second.http(owner, context["admin_key"])
        def snapshot():
            rows = _rows(context, "surreal", config, "uar_collaboration_state")
            require(len(rows) == 1, "catalog durable state missing")
            return rows[0]
        view = _data(admin.request("GET", OWNER_PATH))
        claim, replacement = view["claim"], view["currentFence"]
        require(not view["ownsExecution"] and claim["incarnationId"] != replacement["incarnationId"], "replacement must not own old execution")
        negatives = {}
        for label, evidence in (("absent", str(uuid.uuid4())), ("forged", "trusted-runtime-joined-" + uuid.uuid4().hex)):
            before = snapshot()
            negatives[label] = admin.request("POST", OWNER_PATH + "/reclaim", _reclaim_body(claim, replacement, evidence), expected=409, return_status=True)[0]
            require(snapshot() == before, "invalid evidence changed epoch/effects/accounting")
        joined = _data(first.http(owner, context["admin_key"]).request("POST", OWNER_PATH + "/quiesce", {}))
        evidence = joined["fencingEvidenceRef"]
        state = snapshot()
        proof = state["executionFencingEvidence"][evidence]
        require(proof["source"] == "trusted-runtime-joined" and proof["rootAndChildrenStopped"] is True and proof["fence"] == _fence(claim), "quiesce lacks exact joined root proof")
        # quiesce joins actual execution roots; force removal after that response
        # prevents normal server shutdown from auto-releasing the draining claim.
        old_container = first.container
        first.stop()
        require(context["resources"].docker("inspect", old_container, check=False).returncode != 0, "old runtime still exists")
        body = _reclaim_body(claim, replacement, evidence)
        receipt = _data(admin.request("POST", OWNER_PATH + "/reclaim", body))
        require(receipt["previousFence"] == _fence(claim) and receipt["replacementFence"]["epoch"] == claim["epoch"] + 1 and receipt["fencingEvidenceRef"] == evidence, "joined reclaim identity mismatch")
        before = snapshot()
        replay = _reclaim_body(receipt["replacementFence"], replacement, evidence)
        negatives["replayed"] = admin.request("POST", OWNER_PATH + "/reclaim", replay, expected=409, return_status=True)[0]
        require(snapshot() == before, "replayed evidence changed epoch/effects/accounting")
        return {"negative_statuses": negatives, "joined_evidence_digest": _digest(proof), "reclaim_receipt": receipt, "durable_catalog_digest": _digest(before)}
    finally:
        second.stop()
        first.stop()


def _private_json(path, digest):
    raw = Path(path).read_bytes()
    require(_digest(raw) == digest, "private qualification/identity artifact changed")
    return json.loads(raw)


LIMA_JOURNAL_INSTANCE = "dot17-linux-qualification"
LIMA_JOURNAL_PATH = "/var/lib/uar-supervisor/exclusions.sqlite3"
LIMA_JOURNAL_MAX_OUTPUT = 1024 * 1024
LIMA_JOURNAL_PROGRAM = r'''import json
import sqlite3
import sys
import urllib.parse

path, table, key, value = sys.argv[1:]
queries = {
    ("domains", "epoch"): "SELECT epoch, certificate, registration, private_data FROM domains WHERE epoch=?",
    ("exclusions", "challenge"): "SELECT challenge, request, receipt FROM exclusions WHERE challenge=?",
}
query = queries.get((table, key))
if path != "/var/lib/uar-supervisor/exclusions.sqlite3" or query is None:
    raise SystemExit(64)
uri = "file:" + urllib.parse.quote(path, safe="/") + "?mode=ro"
with sqlite3.connect(uri, uri=True) as connection:
    row = connection.execute(query, (value,)).fetchone()
encoded = json.dumps(None if row is None else list(row), separators=(",", ":"))
if len(encoded.encode()) > 1048576:
    raise SystemExit(65)
sys.stdout.write(encoded)
'''


def _local_journal(path, table, key, value):
    # Existing controller journal, opened read-only. No controller is created,
    # configured, qualified, or started by this fixture.
    uri = Path(path).resolve().as_uri() + "?mode=ro"
    with sqlite3.connect(uri, uri=True) as connection:
        return connection.execute("SELECT * FROM " + table + " WHERE " + key + "=?", (value,)).fetchone()


def _journal(context, fixture, table, key, value):
    transport = fixture.get("controller_journal_transport")
    if transport is None:
        path = fixture.get("controller_journal")
        require(isinstance(path, str) and path, "controller journal path missing")
        return _local_journal(path, table, key, value)
    require(isinstance(transport, dict) and transport.get("kind") == "lima", "unsupported controller journal transport")
    require(transport.get("instance") == LIMA_JOURNAL_INSTANCE, "unexpected Lima journal instance")
    require(transport.get("journal_path") == LIMA_JOURNAL_PATH, "unexpected Lima journal path")
    executable_path, lima_home_path = transport.get("executable"), transport.get("lima_home")
    require(isinstance(executable_path, str) and isinstance(lima_home_path, str), "Lima transport paths missing")
    executable, lima_home = Path(executable_path), Path(lima_home_path)
    require(executable.is_absolute() and executable.name == "limactl" and lima_home.is_absolute(), "Lima transport paths must be absolute")
    expected_hash = transport.get("executable_sha256")
    require(isinstance(expected_hash, str) and expected_hash.startswith("sha256:"), "Lima executable hash missing")
    require(_digest(executable.read_bytes()) == expected_hash, "Lima executable hash changed")
    require((table, key) in (("domains", "epoch"), ("exclusions", "challenge")), "unsupported controller journal row")
    result = context["resources"].run_command([
        str(executable), "--tty=false", "shell", "--workdir=/tmp", LIMA_JOURNAL_INSTANCE,
        "sudo", "/usr/bin/python3", "-c", LIMA_JOURNAL_PROGRAM, LIMA_JOURNAL_PATH,
        table, key, value,
    ], env={"LIMA_HOME": str(lima_home)})
    raw = result.stdout.encode()
    require(len(raw) <= LIMA_JOURNAL_MAX_OUTPUT, "Lima controller journal row too large")
    row = json.loads(result.stdout)
    expected_columns = {("domains", "epoch"): 4, ("exclusions", "challenge"): 3}[(table, key)]
    require(row is None or (isinstance(row, list) and len(row) == expected_columns and all(isinstance(column, str) for column in row)), "invalid Lima controller journal row")
    return None if row is None else tuple(row)


def _qualified_supervisor(context, _owner):
    fixture = context["infrastructure"].get("supervisor")
    required = ("qualification_file", "qualification_sha256", "identity_receipt_file", "identity_receipt_sha256", "runtime_url", "host_token", "ordinary_token", "admin_token", "admin_key", "team_owner_token")
    if not isinstance(fixture, dict) or any(not fixture.get(key) for key in required):
        raise Blocked("qualified supervisor external prerequisite missing: retained qualification and immutable same-image identity receipt, real controller journal, installed replacement runtime and scoped recovery credentials")
    if not fixture.get("controller_journal") and not fixture.get("controller_journal_transport"):
        raise Blocked("qualified supervisor controller journal access prerequisite missing")
    qualification = _private_json(fixture["qualification_file"], fixture["qualification_sha256"])
    identity = _private_json(fixture["identity_receipt_file"], fixture["identity_receipt_sha256"])
    require(qualification["production_qualified"] is True, "supervisor is not operator-qualified")
    require(qualification["controller_sha256"] == hashlib.sha256((Path(__file__).resolve().parents[1] / "supervisor/controller.py").read_bytes()).hexdigest(), "qualified controller source mismatch")
    manifest = context["resources"].manifest
    for key in ("image", "platform", "source_tree", "fixture_tree"):
        require(identity[key] == manifest[key], "supervisor runtime/source identity mismatch")
    image_id = context["resources"].docker("image", "inspect", "--format", "{{.Id}}", context["image"]).stdout.strip()
    require(identity["runtime_image_id"] == image_id, "supervisor installed image bytes differ from candidate")
    require(identity["isolated"] is True and identity["phase"] == "uar-outstanding-issues-v1-1-1", "supervisor target is not the phase-owned disposable deployment")
    require(identity["runtime_url"] == fixture["runtime_url"], "runtime endpoint differs from retained identity")
    registration = identity["predecessor_registration"]
    for key in ("controller_id", "qualification_id"):
        require(registration[key] == qualification[key], "qualification registration binding mismatch")
    mechanism = {"native_boot": "native-boot-exclusion-v1", "kubernetes": "kubernetes-cri-containment-v1"}[registration["domain"]["kind"]]
    require(mechanism == qualification["mechanism"], "qualified mechanism mismatch")
    stored = _journal(context, fixture, "domains", "epoch", registration["runtime_epoch"])
    require(stored is not None and json.loads(stored[2]) == registration, "actual contained predecessor registration missing")
    # Use the existing recovery_scenarios endpoint policy: TLS or local loopback,
    # no redirects or ambient proxies. Tokens stay private and are never output.
    from urllib import request as urlrequest
    from urllib import error as urlerror
    import ssl
    parsed = urllib.parse.urlsplit(fixture["runtime_url"])
    require(not parsed.username and not parsed.password and not parsed.query and not parsed.fragment and (parsed.scheme == "https" or (parsed.scheme == "http" and parsed.hostname in ("127.0.0.1", "localhost", "::1"))), "unsafe supervisor fixture endpoint")
    class NoRedirect(urlrequest.HTTPRedirectHandler):
        def redirect_request(self, *_args):
            raise AssertionError("supervisor redirect refused")
    opener = urlrequest.build_opener(urlrequest.ProxyHandler({}), NoRedirect(), urlrequest.HTTPSHandler(context=ssl.create_default_context()))
    def request(method, path, token, body=None, expected=200, admin=False, workspace=None):
        headers = {"Authorization": "Bearer " + fixture[token], "Content-Type": "application/json"}
        if admin:
            headers["x-uar-admin-key"] = fixture["admin_key"]
        if workspace:
            headers["x-uar-workspace-id"] = workspace
        req = urlrequest.Request(fixture["runtime_url"].rstrip("/") + path, data=None if body is None else json.dumps(body).encode(), headers=headers, method=method)
        try:
            response = opener.open(req, timeout=90)
        except urlerror.HTTPError as error:
            response = error
        with response:
            raw = response.read(2 * 1024 * 1024 + 1)
            require(response.status == expected and len(raw) <= 2 * 1024 * 1024, "supervisor HTTP result mismatch")
            return _data(json.loads(raw)) if raw and expected == 200 else None
    scopes = identity["team_scopes"]
    require(scopes and sum(len(scope["queued_attempt_ids"]) for scope in scopes) > 0 and sum(len(scope["uncertain_attempt_ids"]) for scope in scopes) > 0, "supervisor requires real queued and active predecessor work")
    def snapshot():
        view = request("GET", OWNER_PATH, "admin_token")
        teams = {}
        for scope in scopes:
            team = scope["team_id"]
            teams[team] = request("GET", "/api/v1/collaboration/team-instances/" + urllib.parse.quote(team, safe="") + "/execution", "team_owner_token", workspace=scope["workspace_id"])
        return {"ownership": view, "teams": teams}
    initial = snapshot()
    claim = initial["ownership"]["claim"]
    replacement = initial["ownership"]["currentFence"]
    require(_fence(claim) == identity["predecessor_fence"] and replacement == identity["replacement_fence"] and claim["incarnationId"] == registration["team_incarnation"] and claim["incarnationId"] != replacement["incarnationId"] and not initial["ownership"]["ownsExecution"], "supervisor execution fence mismatch")
    for scope in scopes:
        attempts = {row["id"]: row for row in initial["teams"][scope["team_id"]]["attempts"]}
        for attempt in scope["queued_attempt_ids"]:
            require(attempts[attempt]["status"] == "queued" and attempts[attempt]["executionFence"] == _fence(claim), "queued predecessor identity mismatch")
        for attempt in scope["uncertain_attempt_ids"]:
            require(attempts[attempt]["status"] in ("running", "cancellation_requested", "uncertain") and attempts[attempt]["executionFence"] == _fence(claim), "active predecessor identity mismatch")
    negatives = {}
    for label, evidence in (("absent", str(uuid.uuid4())), ("forged", "trusted-supervisor-excluded-" + uuid.uuid4().hex)):
        before = snapshot()
        request("POST", OWNER_PATH + "/reclaim", "admin_token", _reclaim_body(claim, replacement, evidence), expected=409, admin=True)
        require(snapshot() == before, "invalid supervisor proof changed epoch/effects/accounting")
        negatives[label] = 409
    target = {"kind": "team_owner", "fence": _fence(claim)}
    body = {"runtime_epoch": registration["runtime_epoch"], "target": target}
    before = snapshot()
    request("POST", CHALLENGE_PATH, "ordinary_token", body, expected=403)
    wrong = json.loads(json.dumps(body))
    wrong["target"]["fence"]["incarnationId"] = str(uuid.uuid4())
    request("POST", CHALLENGE_PATH, "host_token", wrong, expected=409)
    require(snapshot() == before, "unauthorized/forged challenge changed execution")
    challenge = request("POST", CHALLENGE_PATH, "host_token", body)["challenge"]
    require(challenge["registration"] == registration and challenge["target"] == target and challenge["generation"] == 1, "challenge identity mismatch")
    exclusion_path = CHALLENGE_PATH + "/" + challenge["id"] + "/exclude"
    request("POST", exclusion_path, "ordinary_token", {}, expected=403)
    result = request("POST", exclusion_path, "host_token", {})
    require(result["exclusion_recorded"] is True and result["effects_reconciled"] is False and result["execution_resumed"] is False, "exclusion overclaimed settlement")
    physical = _journal(context, fixture, "exclusions", "challenge", challenge["id"])
    require(physical is not None and json.loads(physical[1]) == challenge, "physical exclusion journal missing")
    physical_receipt = json.loads(physical[2])
    require(physical_receipt["challenge"] == challenge and physical_receipt["controller_id"] == registration["controller_id"] and physical_receipt["qualification_id"] == registration["qualification_id"], "physical receipt binding mismatch")
    observation = physical_receipt["observation"]
    domain = registration["domain"]
    if domain["kind"] == "kubernetes":
        require(observation["kind"] == "kubernetes_runtime" and all(observation.get(key) is True for key in ("sandbox_stopped", "descendant_cgroup_empty", "restart_inhibited", "kubelet_cgroup_empty", "kubelet_runtime_masked")), "whole-domain exclusion missing")
        for key in ("node_uid", "node_boot_id", "sandbox_id"):
            require(observation[key] == domain[key], "physical domain changed")
        require(sorted(observation["stopped_container_ids"]) == sorted(domain["container_ids"]) and observation["kubelet_unit"] == "kubelet.service", "contained descendants mismatch")
    else:
        require(observation["kind"] == "native_boot" and observation["platform"] == domain["platform"] and observation["machine_id"] == domain["machine_id"] and observation["previous_boot_id"] == domain["boot_id"] and observation["observed_boot_id"] != domain["boot_id"], "actual whole-host boot exclusion missing")
    before = snapshot()
    request("POST", exclusion_path, "host_token", {}, expected=409)
    require(snapshot() == before, "replayed challenge changed effects")
    receipt = request("POST", OWNER_PATH + "/reclaim", "admin_token", _reclaim_body(claim, replacement, challenge["id"]), admin=True)
    require(receipt["previousFence"] == _fence(claim) and receipt["replacementFence"] == dict(replacement, epoch=claim["epoch"] + 1) and receipt["fencingEvidenceRef"] == challenge["id"], "supervisor reclaim fence mismatch")
    queued = sorted(attempt for scope in scopes for attempt in scope["queued_attempt_ids"])
    uncertain = sorted(attempt for scope in scopes for attempt in scope["uncertain_attempt_ids"])
    require(sorted(receipt["transferredQueuedAttemptIds"]) == queued and sorted(receipt["uncertainAttemptIds"]) == uncertain, "reclaim lost queued/uncertain work")
    after = snapshot()
    for scope in scopes:
        previous = initial["teams"][scope["team_id"]]
        current = after["teams"][scope["team_id"]]
        require(current["committed"] == previous["committed"] and current["reserved"] == previous["reserved"], "exclusion settled accounting")
        attempts = {row["id"]: row for row in current["attempts"]}
        for attempt in scope["uncertain_attempt_ids"]:
            row = attempts[attempt]
            require(row["status"] == "uncertain" and row["effectDisposition"] == "uncertain" and row["accountingState"] == "reserved-unknown", "active effects were settled by exclusion")
    request("POST", OWNER_PATH + "/reclaim", "admin_token", _reclaim_body(receipt["replacementFence"], replacement, challenge["id"]), expected=409, admin=True)
    require(snapshot() == after, "replayed supervisor evidence changed epoch/effects")
    negatives.update(replayed=409, ordinary_challenge=403, wrong_incarnation=409, ordinary_exclusion=403, replayed_challenge=409)
    return {"qualification_digest": fixture["qualification_sha256"], "identity_digest": fixture["identity_receipt_sha256"], "physical_exclusion_digest": _digest(physical_receipt), "negative_statuses": negatives, "reclaim_receipt": receipt, "before_digest": _digest(initial), "after_digest": _digest(after)}


def run_group(context):
    require(context["group"] == "recovery", "wrong group")
    issuer, tenant = context["identity"]["issuer"], context["identity"]["tenant"]
    owner = context["jwt"]("recovery-owner", ["user"], issuer=issuer, tenant=tenant)
    outsiders = [
        ("other_principal", context["jwt"]("recovery-other", ["user"], issuer=issuer, tenant=tenant), 404),
        ("other_tenant", context["jwt"]("recovery-owner", ["user"], issuer=issuer, tenant="other-phase-tenant"), 404),
        ("other_issuer", context["jwt"]("recovery-owner", ["user"], issuer="other-phase-issuer", tenant=tenant), (401, 404)),
    ]
    refusal = _unattested(context, owner)
    uploads = []
    for backend in ("surreal", "postgres"):
        uploads.append(_attested(context, backend, owner, outsiders))
    joined = _joined(context, owner)
    observations = (refusal, uploads, joined)
    scenarios = [{"id": name, "status": "passed", "observations": observation} for name, observation in zip(REQUIRED_SCENARIOS["recovery"][:3], observations)]
    try:
        supervisor = _qualified_supervisor(context, owner)
    except Blocked as error:
        scenarios.append({"id": REQUIRED_SCENARIOS["recovery"][3], "status": "blocked", "observations": {"reason": str(error)}})
        return {"status": "blocked", "scenarios": scenarios}
    scenarios.append({"id": REQUIRED_SCENARIOS["recovery"][3], "status": "passed", "observations": supervisor})
    return {"status": "passed", "scenarios": scenarios}


def _embedding_peer(root):
    """Independent real HTTP collaborator with a release-controlled response."""
    import http.server
    import os
    import select
    import threading
    released = threading.Event()
    output_lock = threading.Lock()

    def emit(name, **fields):
        with output_lock:
            print("UAR_FIXTURE_EVENT:" + json.dumps(dict(version=1, event=name, **fields)), flush=True)
    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *_args):
            pass
        def do_POST(self):
            require(self.path == "/v1/embeddings", "unexpected embedding route")
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            require(body["model"] == "uar111-recovery-vector" and body["dimensions"] == 384 and isinstance(body["input"], list), "unexpected embedding request")
            inputs = body["input"]
            require(all(isinstance(value, str) for value in inputs), "unexpected embedding input")
            # Built-in skills use this peer during startup. Hold only the
            # recovery upload so startup reaches readiness before its durable
            # worker crosses the controlled restart edge.
            if any(SOURCE.decode() in value for value in inputs):
                emit("contact")
                released.wait()
            # Controlled vector transport tests UAR durability/retrieval, not
            # semantic quality of an external embedding model.
            result = {"data": [{"index": index, "embedding": [1.0] + [0.0] * 383} for index, _ in enumerate(inputs)]}
            data = json.dumps(result).encode()
            try:
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(data)))
                self.end_headers()
                self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError):
                pass
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    emit("ready", port=server.server_port)
    pending = b""
    try:
        while True:
            readable, _, _ = select.select([server, sys.stdin], [], [])
            if sys.stdin in readable:
                data = os.read(sys.stdin.fileno(), 4096)
                if not data:
                    break
                pending += data
                while b"\n" in pending:
                    line, pending = pending.split(b"\n", 1)
                    command = json.loads(line)
                    require(command == {"version": 1, "command": "release"}, "invalid embedding control command")
                    released.set()
                    emit("released")
            if server in readable:
                server.handle_request()
    finally:
        released.set()
        server.server_close()


if __name__ == "__main__":
    require(len(sys.argv) == 3 and sys.argv[1] == "--embedding-peer", "only the fixed embedding collaborator entrypoint is supported")
    _embedding_peer(Path(sys.argv[2]))
