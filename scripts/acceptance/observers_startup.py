"""Source-backed U21 observers; execution requires the approved acceptance packet.

Only the age action mutates product data: it changes last_activity on one
already-persisted genuine session. It never deletes or recreates that session.
"""
from __future__ import annotations

import base64
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import urllib.parse
import urllib.request
import uuid


LIMIT = 2 * 1024 * 1024


class ObservationFailure(RuntimeError):
    """Closed observer diagnostic containing no upstream credential material."""


def need(condition, message):
    if not condition:
        raise ObservationFailure(message)


def secret_ref(name):
    need(isinstance(name, str) and name in os.environ, "missing named environment reference")
    return os.environ[name]


def pg_string(value):
    return "'" + value.replace("'", "''") + "'"


def run_scope(context):
    return "uar_acceptance_" + hashlib.sha256(context["run_id"].encode()).hexdigest()[:20]


def session_storage_key(owner_id, session_id):
    need(isinstance(owner_id, str) and bool(owner_id), "session owner must be explicit")
    key = str(len(owner_id.encode())) + ":" + owner_id + ":" + session_id
    return "principal-session:" + key if owner_id.startswith("v1:k:") else key


class Store:
    def __init__(self, config, context):
        self.config = config
        self.context = context
        self.scope = run_scope(context)
        self.kind = context["backend"]
        self.timeout = config.get("command_seconds", 30)
        need(isinstance(self.timeout, int) and 0 < self.timeout <= 120, "invalid observer command bound")
        if self.kind == "postgres_pgvector":
            self.pg = config["postgres"]
            need(self.pg["schema_scope"] == "run_sha256_20", "Postgres must use the exact run-derived schema")
            self.identity = self.pg_query("SELECT json_build_object('database',current_database(),"
                                          "'schema',current_schema(),'address',inet_server_addr()::text,"
                                          "'port',inet_server_port())")
            need(self.identity.get("schema") == self.scope, "Postgres actual search_path differs")
        elif self.kind == "surreal_disk":
            self.surreal = config["surreal"]
            need(self.surreal["database_scope"] == "run_sha256_20", "Surreal must use the exact run-derived DB")
            url = secret_ref(self.surreal["url_env"])
            parsed = urllib.parse.urlparse(url)
            need(parsed.scheme in ("http", "https") and parsed.hostname in ("localhost", "127.0.0.1", "::1")
                 and parsed.username is None and not parsed.query and parsed.path in ("", "/"),
                 "Surreal observer requires an isolated loopback HTTP origin")
            self.url = url.rstrip("/") + "/sql"
            self.sql("INFO FOR DB;")
            self.identity = {"origin": url.rstrip("/"), "namespace": self.surreal["namespace"],
                             "database": self.scope}
        else:
            raise ObservationFailure("unsupported durable adapter")
        self.storage_identity = "sha256:" + hashlib.sha256(json.dumps(
            {"backend": self.kind, "actual": self.identity}, sort_keys=True).encode()).hexdigest()

    def pg_query(self, query):
        environment = dict(os.environ)
        # Credentials remain in the child environment, never argv or public output.
        environment["PGDATABASE"] = secret_ref(self.pg["connection_env"])
        environment["PGOPTIONS"] = "-c search_path=" + self.scope
        environment["PGAPPNAME"] = "uar-acceptance-startup-observer"
        binary = Path(self.pg["psql_binary"])
        need(binary.is_absolute() and binary.is_file(), "named frozen psql executable is unavailable")
        result = subprocess.run([str(binary), "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1"],
                                input=query.encode(), stdout=subprocess.PIPE,
                                stderr=subprocess.PIPE, env=environment, timeout=self.timeout)
        need(result.returncode == 0, "Postgres source observer query failed (private error withheld)")
        need(len(result.stdout) <= LIMIT, "Postgres observer result exceeded bound")
        return json.loads(result.stdout)

    def sql(self, query):
        credentials = secret_ref(self.surreal["username_env"]) + ":" + secret_ref(self.surreal["password_env"])
        headers = {"Authorization": "Basic " + base64.b64encode(credentials.encode()).decode(),
                   "Surreal-NS": self.surreal["namespace"], "Surreal-DB": self.scope,
                   "Accept": "application/json", "Content-Type": "text/plain"}
        request = urllib.request.Request(self.url, data=query.encode(), headers=headers, method="POST")
        # Redirects must not carry observer authority outside the owned origin.
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, req, fp, code, msg, headers, newurl):
                return None
        with urllib.request.build_opener(NoRedirect).open(request, timeout=self.timeout) as reply:
            raw = reply.read(LIMIT + 1)
        need(len(raw) <= LIMIT, "Surreal observer result exceeded bound")
        statements = json.loads(raw)
        need(isinstance(statements, list) and statements, "Surreal observer response missing statements")
        need(all(row.get("status") == "OK" for row in statements), "Surreal observer query failed")
        return statements[-1]["result"]

    def counts(self):
        if self.kind == "postgres_pgvector":
            tables = self.pg_query("SELECT COALESCE(json_agg(table_name),'[]') FROM information_schema.tables "
                                   "WHERE table_schema=current_schema() AND table_type='BASE TABLE'")
            result = {"agents": 0, "sessions": 0, "settings": 0}
            for name in tables:
                if name == "_sqlx_migrations":
                    continue
                need(re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]*", name), "unsupported application table identifier")
                result[name] = self.pg_query('SELECT count(*) FROM "' + name + '"')
            return result
        info = self.sql("INFO FOR DB;")
        tables = info.get("tables", {})
        result = {"agents": 0, "sessions": 0, "settings": 0}
        for name in tables:
            need(re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]*", name), "unsupported application table identifier")
            rows = self.sql("SELECT count() AS count FROM " + name + " GROUP ALL;")
            result[name] = rows[0]["count"] if rows else 0
        return result

    def session_data(self, owner_id, session_id):
        """Read one full persisted session by exact owner partition; no writes."""
        session_id = str(uuid.UUID(session_id))
        key = session_storage_key(owner_id, session_id)
        if self.kind == "postgres_pgvector":
            rows = self.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM sessions WHERE id=" + pg_string(key))
        else:
            rows = self.sql("SELECT * FROM type::record('sessions'," + json.dumps(key) + ");")
            rows = [{**row, "id": row.get("logical_id")} for row in rows]
        need(len(rows) <= 1, "exact owner session partition is ambiguous")
        if not rows:
            return None
        need(rows[0].get("owner_id") == owner_id and rows[0].get("id") == session_id,
             "persisted session owner partition mismatch")
        return rows[0]

    def session(self, session_id, owner_id=None):
        if self.kind == "postgres_pgvector":
            rows = self.pg_query("SELECT COALESCE(json_agg(json_build_object('storage_key',id,"
                                 "'owner_id',data->>'owner_id','session_id',data->>'id',"
                                 "'last_activity',data->>'last_activity')),'[]') FROM sessions "
                                 "WHERE data->>'id'=" + pg_string(session_id)
                                 + (" AND data->>'owner_id'=" + pg_string(owner_id) if owner_id is not None else ""))
        else:
            rows = self.sql("SELECT owner_id, logical_id AS session_id, last_activity FROM sessions WHERE logical_id="
                            + json.dumps(session_id)
                            + (" AND owner_id=" + json.dumps(owner_id) if owner_id is not None else "") + ";")
        need(len(rows) <= 1, "run-owned logical session is ambiguous")
        if not rows:
            return None
        row = rows[0]
        owner = row.get("owner_id")
        need(isinstance(owner, str) and owner and row["session_id"] == session_id,
             "persisted session lacks logical ownership")
        key = session_storage_key(owner, session_id)
        if self.kind == "postgres_pgvector":
            need(row["storage_key"] == key, "persisted session partition differs")
        row["storage_key"] = key
        return row

    def age(self, session_id):
        before = self.session(session_id)
        need(before is not None, "maintenance fixture requires an actual persisted genuine session")
        stamp = "2000-01-01T00:00:00Z"
        if self.kind == "postgres_pgvector":
            rows = self.pg_query("WITH aged AS (UPDATE sessions SET data=jsonb_set(data,'{last_activity}',"
                                   + pg_string(json.dumps(stamp)) + "::jsonb),updated_at=NOW() WHERE id="
                                   + pg_string(before["storage_key"]) + " AND data->>'id'=" + pg_string(session_id)
                                   + " RETURNING id,data) SELECT COALESCE(json_agg(json_build_object("
                                   "'storage_key',id,'owner_id',data->>'owner_id','session_id',data->>'id',"
                                   "'last_activity',data->>'last_activity')),'[]') FROM aged")
            need(len(rows) == 1, "maintenance age fixture did not update exactly one session")
            aged = rows[0]
        else:
            rows = self.sql("UPDATE type::record('sessions'," + json.dumps(before["storage_key"])
                            + ") SET last_activity=" + json.dumps(stamp) + " WHERE logical_id="
                            + json.dumps(session_id) + " RETURN AFTER;")
            need(len(rows) == 1, "maintenance age fixture did not update exactly one session")
            aged = {"storage_key": before["storage_key"], "owner_id": rows[0].get("owner_id"),
                    "session_id": rows[0].get("logical_id"), "last_activity": rows[0].get("last_activity")}
        need(aged.get("owner_id") == before["owner_id"] and aged.get("session_id") == session_id
             and isinstance(aged.get("last_activity"), str) and aged["last_activity"].startswith("2000-01-01"),
             "durable maintenance age change was not observable")
        return before, aged


def private_state(config, store):
    parent = Path(secret_ref(config["state_dir_env"]))
    need(parent.is_absolute(), "observer state parent must be absolute")
    root = (parent / store.scope).resolve()
    need(root.name == store.scope and root.is_dir(),
         "observer state directory must be the exact existing run-owned scope")
    need(root.stat().st_mode & 0o077 == 0, "observer state directory must be private")
    return root


def save(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, sort_keys=True)
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o600)


def logs():
    paths = {}
    for name in ("STDOUT", "STDERR"):
        value = os.environ.get("UAR_ACCEPTANCE_SERVER_" + name + "_PATH")
        need(value, "missing actual owned-server log capture")
        path = Path(value)
        need(path.is_file(), "owned server log capture is unavailable")
        paths[name] = path
    return paths


def worker_line(paths, cursors, session_id, owner_id):
    for name, path in paths.items():
        with path.open("rb") as stream:
            stream.seek(cursors[name])
            raw = stream.read(LIMIT + 1)
        need(len(raw) <= LIMIT, "maintenance log allowance exceeded")
        for line in raw.splitlines():
            text = line.decode("utf-8", errors="replace")
            if "persisted session purged by TTL sweep" in text and session_id in text and owner_id in text:
                # Retain bounded identity, not arbitrary adjacent log content.
                return {"marker": "persisted session purged by TTL sweep", "session_id": session_id,
                        "owner_id": owner_id, "stream": name,
                        "line_sha256": "sha256:" + hashlib.sha256(line).hexdigest()}
    return None


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    config = json.loads(Path(secret_ref("UAR_ACCEPTANCE_STARTUP_CONFIG")).read_text())
    need(config.get("schema_version") == 1, "startup observer configuration schema differs")
    store = Store(config, context)
    root = private_state(config, store)
    action = sys.argv[1] if len(sys.argv) == 2 else ""
    values = context.get("values", {})
    initial_path = root / "startup-initial.json"
    common = {"run_id": context["run_id"], "backend": context["backend"],
              "storage_identity": store.storage_identity}
    if action == "capture":
        counts = store.counts()
        need(all(count == 0 for count in counts.values()), "initial application store was not empty")
        value = {**common, "counts": counts, "empty_before_first_start": True,
                 "captured_at": datetime.now(timezone.utc).isoformat()}
        save(initial_path, value)
    elif action == "initial":
        value = json.loads(initial_path.read_text())
        need(all(value.get(key) == expected for key, expected in common.items()),
             "initial store receipt belongs to another invocation or adapter")
        need(store.counts()["agents"] >= 5, "runtime did not seed the observed store")
    elif action == "maintenance":
        need(values.get("storage_identity") == store.storage_identity, "maintenance store identity differs")
        session_id = str(uuid.UUID(values["expired_session_id"]))
        path = root / ("maintenance-" + session_id + ".json")
        paths = logs()
        if values.get("action") == "prepare":
            cursors = {name: log.stat().st_size for name, log in paths.items()}
            before, aged = store.age(session_id)
            value = {**common, "expired_session_id": session_id, "persisted_before": before,
                     "aged_record": aged, "log_cursors": cursors,
                     "prepared_at": datetime.now(timezone.utc).isoformat()}
            save(path, value)
        else:
            prepared = json.loads(path.read_text())
            need(prepared["storage_identity"] == store.storage_identity, "maintenance baseline store differs")
            seconds = config.get("maintenance_seconds", 60)
            need(isinstance(seconds, int) and 0 < seconds <= 120, "invalid maintenance wait bound")
            deadline = time.monotonic() + seconds
            receipt = None
            record = store.session(session_id)
            while time.monotonic() < deadline:
                record = store.session(session_id)
                receipt = worker_line(paths, prepared["log_cursors"], session_id,
                                      prepared["persisted_before"]["owner_id"])
                if record is None and receipt is not None:
                    break
                # Polling observes real state; elapsed time never establishes success.
                time.sleep(0.1)
            need(record is None and receipt is not None, "actual production retention deletion was not established")
            value = {**common, "worker": "run-retention", "expired_session_id": session_id,
                     "persisted_before": prepared["aged_record"], "persisted_after": None,
                     "deleted_by_worker": True, "worker_receipt": receipt}
    elif action == "read":
        need(values.get("storage_identity") == store.storage_identity, "restart store identity differs")
        session_id = str(uuid.UUID(values["expired_session_id"]))
        value = {**common, "expired_session_id": session_id, "persisted_record": store.session(session_id)}
    else:
        raise ObservationFailure("startup observer action must be capture, initial, maintenance or read")
    print(json.dumps(value, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        # Upstream errors may include database URLs or authentication data.
        print(json.dumps({"error": "startup durable observer failed; no successful observation",
                          "error_type": type(error).__name__,
                          "detail": str(error) if isinstance(error, ObservationFailure)
                          else "upstream cause withheld because it may contain credentials"}))
        sys.exit(2)
