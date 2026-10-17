"""Real installed-image credential observations. Importing performs no I/O."""
import base64
import copy
import csv
import hashlib
import hmac
import http.server
import io
import json
import secrets
import threading
import time
import urllib.parse
import uuid
from concurrent.futures import ThreadPoolExecutor

from uar_1_1_1_support import Blocked, Http, SURREAL_IMAGE, require, wait_for

BACKENDS = ("local", "surreal", "postgres")
MIGRATION_CASES = ("metadata-upgrade", "preview-authority-pagination", "legacy-conditional-copy",
                   "fixed-local-conditional-copy", "revision-drift", "mapping-drift", "destination-conflict",
                   "concurrent-replay", "lost-response-retry", "immutable-rotation-deletion-restart",
                   "historical-receipt", "preview-status-inventory", "secret-redaction")
REQUIRED_SCENARIOS = {"credentials": tuple(
    ["identity-isolation", "local-installation-stability", "shared-source-grants", "absent-service-policy",
     "store-decrypt-failures", "single-owner-compatibility", "local-quarantine", "messages-facade-policy", "response-correction-policy"] +
    [backend + "-" + case for backend in BACKENDS for case in MIGRATION_CASES])}
PREFIX = "/api/uar/credentials"
INSTALLATION = "9b7bc6a2-8ecf-4fc1-b404-329509870bd0"


def local_token(resources, subject, secret=None):
    def enc(value):
        return base64.urlsafe_b64encode(json.dumps(value, separators=(",", ":")).encode()).rstrip(b"=").decode()
    message = enc({"alg": "HS256", "typ": "JWT"}) + "." + enc({
        "sub": subject, "roles": ["user"], "exp": int(time.time()) + 7200})
    signature = hmac.new((secret or resources.signing_secret).encode(), message.encode(), hashlib.sha256).digest()
    token = message + "." + base64.urlsafe_b64encode(signature).rstrip(b"=").decode()
    resources.known_secrets.add(token)
    return token


def encoded_owner(kind, issuer, tenant, subject):
    tenant = "local" if tenant is None else "t:%s:%s" % (len(tenant.encode()), tenant)
    return "v1:k:%s:i:%s:%s:%s:s:%s:%s" % (kind, len(issuer.encode()), issuer, tenant, len(subject.encode()), subject)


class Peer:
    def __init__(self, resources):
        self.seen = []
        self.labels = {}
        self.lock = threading.Lock()
        owner = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def log_message(self, *_args):
                pass

            def do_POST(self):
                body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
                with owner.lock:
                    owner.seen.append(owner.labels.get(self.headers.get("Authorization", ""), "unknown"))
                choice = {"index": 0, "delta": {"content": "credential-observed"}, "finish_reason": None}
                chunk = dict(id="credential-peer", object="chat.completion.chunk", created=1,
                             model=body.get("model", "gpt-4o-mini"), choices=[choice])
                if body.get("stream"):
                    first = json.dumps(chunk)
                    choice.update(delta={}, finish_reason="stop")
                    output = ("data: " + first + "\n\ndata: " + json.dumps(chunk) + "\n\ndata: [DONE]\n\n").encode()
                    content = "text/event-stream"
                else:
                    output = json.dumps(dict(id="credential-peer", object="chat.completion", created=1,
                        model=body.get("model", "gpt-4o-mini"), choices=[dict(index=0,
                        message=dict(role="assistant", content="credential-observed"), finish_reason="stop")],
                        usage=dict(prompt_tokens=1, completion_tokens=1, total_tokens=2))).encode()
                    content = "application/json"
                self.send_response(200)
                self.send_header("Content-Type", content)
                self.send_header("Content-Length", str(len(output)))
                self.end_headers()
                self.wfile.write(output)

        self.server = http.server.ThreadingHTTPServer(("0.0.0.0", 0), Handler)
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=False)
        self.thread.start()
        self.url = resources.relay(self.server.server_port) + "/v1"

    def key(self, resources, label):
        value = "phase-credential-" + secrets.token_hex(16)
        resources.known_secrets.add(value)
        self.labels["Bearer " + value] = label
        return value

    def close(self):
        self.server.shutdown()
        self.server.server_close()
        self.thread.join(timeout=10)
        require(not self.thread.is_alive(), "provider peer did not join")


def artifact():
    return dict(version="1.0.0", kind="agent", id="credential-" + uuid.uuid4().hex,
        metadata=dict(title="Credential observation", description="Owned phase fixture", tags=[]),
        runtime=dict(entry="agent", protocols={}), policy=dict(provider=dict(
            default=dict(provider="openai", model="gpt-4o-mini"), fallbacks=[]),
            tools=dict(allow=[], deny=[], max_concurrent=1, execution_mode="direct"),
            skills=dict(prefer=[], max_active=1)), schemas=dict(inputs=None, outputs=None, state=None),
        prompt=dict(system="Answer the controlled observation.", instructions=[]),
        memory=dict(conversation=dict(enabled=False), kb=dict(enabled=False, knowledge_bases=[], citation_required=False)),
        tools=dict(bundles=[]), ui=dict(forms=dict(enabled=False), artifacts=dict(enabled=False, preferred_types=[])), extensions={})


def run_account(node, token, peer, label=None):
    before = len(peer.seen)
    status, value = node.http(token).request("POST", "/api/uar/runs", {"artifact": artifact(), "input": "Observe account"},
        expected=(200, 201, 202, 400, 403, 409, 500, 503), return_status=True)
    if status < 300:
        stream = node.http(token).stream("/api/uar/runs/" + value["run_id"] + "/stream")
        if label is not None:
            require("agui.done" in stream and "agui.error" not in stream, "authorized run did not finish")
        else:
            require("agui.error" in stream or "run.error" in stream or "credential" in stream,
                    "denied run lacked terminal failure")
    elif label is not None:
        raise AssertionError("authorized run admission failed")
    observed = peer.seen[before:]
    require(observed and all(value == label for value in observed) if label is not None else not observed,
            "provider account/contact observation mismatch")
    return {"http_status": status, "provider_contacts": len(observed), "account": label or "denied"}


def configure(resources, peer, backend="surreal", shared=True):
    config = resources.base_config(backend)
    config["security"]["provider_credentials"] = dict(mode="shared" if shared else "single_owner", installation_id=INSTALLATION)
    config["llm"] = dict(model="openai/gpt-4o-mini", base_url=peer.url,
                         api_key=peer.key(resources, "config"))
    config["resilience"] = dict(rate_limit_enabled=False)
    return config


def rewrite(node):
    node.config_path.write_text(json.dumps(node.config))
    node.restart()


class Store:
    """Direct fixture metadata/copy access, only in runner-owned databases."""
    def __init__(self, resources, backend, node):
        self.r, self.backend, self.node = resources, backend, node
        persistence = node.config["persistence"]
        self.database = persistence.get("surreal_db", "uar") if backend != "postgres" else urllib.parse.urlparse(persistence["database_url"]).path[1:]

    def query(self, surreal, postgres, read=False):
        if self.backend == "postgres":
            output = self.r.postgres_query(self.database, postgres)
            if not read:
                return None
            rows = list(csv.reader(io.StringIO(output)))
            return json.loads(rows[1][0]) if len(rows) > 1 else None
        if self.backend == "surreal":
            response = self.r.surreal_query(self.database, surreal)
        else:
            # No two owners open the SurrealKV directory at once.
            self.node.stop()
            name = self.r.unique_name("kv-metadata")
            password = secrets.token_hex(20)
            self.r.known_secrets.add(password)
            self.r.containers.append(name)
            try:
                self.r.docker("run", "--detach", "--name", name, "--network", self.r.network,
                    "--publish", "127.0.0.1::8000", "--user", "65532:65532", "--volume", self.node.volume + ":/data",
                    SURREAL_IMAGE, "start", "--user", "root", "--pass", password, "--bind", "0.0.0.0:8000",
                    "surrealkv:///data/data/phase.db")
                address = self.r.docker("port", name, "8000/tcp").stdout.strip()
                wait_for("quiesced SurrealKV metadata host", lambda: Http("http://" + address).request("GET", "/health"))
                authorization = base64.b64encode(("root:" + password).encode()).decode()
                response = Http("http://" + address).request("POST", "/sql", surreal.encode(), headers={
                    "Authorization": "Basic " + authorization, "Surreal-NS": "uar", "Surreal-DB": self.database,
                    "Content-Type": "text/plain"})
            finally:
                self.r.docker("rm", "--force", name, check=False)
                self.node.start()
        require(isinstance(response, list) and all(value.get("status") == "OK" for value in response), "fixture SQL failed")
        return response[-1].get("result") if read else None

    def seed(self, owner, provider, revision, source_owner=None, scope="user", scope_id="source"):
        table = "provider_principal_credentials" if source_owner else "provider_credentials"
        rid = json.dumps([source_owner, scope, scope_id, provider], separators=(",", ":")) if source_owner else scope + ":" + scope_id + ":" + provider
        values = dict(scope=scope, scope_id=scope_id, provider_id=provider, row_revision=revision)
        if source_owner:
            values["owner_key"] = source_owner
        assignments = ", ".join(k + " = " + ("NONE" if v is None else json.dumps(v)) for k, v in values.items())
        columns = list(values)
        literals = ["NULL" if values[k] is None else "'" + str(values[k]).replace("'", "''") + "'" for k in columns]
        self.query("LET $row = (SELECT * FROM provider_principal_credentials WHERE owner_key = " + json.dumps(owner) + " AND provider_id = 'seed')[0]; CREATE type::record(" + json.dumps(table) + ", " + json.dumps(rid) + ") SET " + assignments + ", api_key_encrypted=$row.api_key_encrypted, api_key_hint=$row.api_key_hint, created_at=$row.created_at, updated_at=$row.updated_at RETURN NONE;",
            "INSERT INTO " + table + " (" + ",".join(columns) + ",api_key_encrypted,api_key_hint,created_at,updated_at) SELECT " + ",".join(literals) + ",api_key_encrypted,api_key_hint,created_at,updated_at FROM provider_principal_credentials WHERE owner_key='" + owner.replace("'", "''") + "' AND provider_id='seed';")

    def revision(self, provider, revision, fixed=False):
        table = "provider_principal_credentials" if fixed else "provider_credentials"
        self.query("UPDATE " + table + " SET row_revision=" + ("NONE" if revision is None else json.dumps(revision)) + " WHERE provider_id=" + json.dumps(provider) + " RETURN NONE;",
            "UPDATE " + table + " SET row_revision=" + ("NULL" if revision is None else "'" + revision + "'") + " WHERE provider_id='" + provider + "';")

    def historical_journal(self, provider):
        self.query("UPDATE provider_credential_owner_migrations UNSET source_kind, source_owner_key, source_scope_id, source_revision, destination_revision, precondition WHERE provider_id=" + json.dumps(provider) + " RETURN NONE;",
            "UPDATE provider_credential_owner_migrations SET receipt=receipt-'source_kind'-'source_owner_key'-'source_scope_id'-'source_revision'-'destination_revision'-'precondition' WHERE receipt->>'provider_id'='" + provider + "';")

    def retained(self, provider):
        value = self.query("RETURN array::len((SELECT id FROM provider_credentials WHERE provider_id=" + json.dumps(provider) + "));",
            "SELECT to_json(count(*)) FROM provider_credentials WHERE provider_id='" + provider + "';", read=True)
        require(value == 1, "migration removed or duplicated source")

    def remove_source(self, provider):
        self.query("DELETE provider_credentials WHERE provider_id=" + json.dumps(provider) + " RETURN NONE;",
            "DELETE FROM provider_credentials WHERE provider_id='" + provider + "';")


def credentials(node, token):
    return node.http(token)


def result(results, case, observations):
    results.append(dict(id=case, status="passed", observations=observations))


def preview(node, host):
    return node.http(host).request("GET", PREFIX + "/migration-preview")


def item(node, host, provider):
    values = preview(node, host)["items"]
    matches = [value for value in values if value["provider_id"] == provider]
    require(len(matches) == 1, "migration source inventory ambiguity")
    return matches[0]


def install_mappings(node, legacy, principal):
    legacy_file = node.resources.root / (node.name + "-legacy.json")
    principal_file = node.resources.root / (node.name + "-principal.json")
    legacy_file.write_text(json.dumps(legacy))
    principal_file.write_text(json.dumps(principal))
    legacy_file.chmod(0o644)
    principal_file.chmod(0o644)
    node.stop()
    node.mounts = [(legacy_file, "/opt/uar/legacy-map.json", "ro"), (principal_file, "/opt/uar/principal-map.json", "ro")]
    node.env["CREDENTIAL_OWNER_MAPPING_FILE"] = "/opt/uar/legacy-map.json"
    node.config["security"]["provider_credentials"]["principal_mapping_file"] = "/opt/uar/principal-map.json"
    node.config_path.write_text(json.dumps(node.config))
    node.start()


def apply(node, token, provider, tag=None, fixed=False, old_owner=None, expected=200, extra=None):
    body = dict(scope="user", provider_id=provider, reason="operator_approved_local_namespace_migration" if fixed else "phase approved nonsecret migration")
    body.update(dict(source_owner_key=old_owner, source_scope_id="source") if fixed else dict(legacy_scope_id="source"))
    body.update(extra or {})
    return node.http(token).request("POST", PREFIX + ("/migrate-principal" if fixed else "/migrate-legacy"), body,
        expected=expected, headers={"If-Match": tag} if tag is not None else {})


def migration_cases(ctx, peer, backend, results):
    r = ctx["resources"]
    config = configure(r, peer, backend)
    encryption_key = secrets.token_hex(32)
    r.known_secrets.add(encryption_key)
    node = r.new_runtime("credentials-" + backend, config, env={"CREDENTIAL_ENCRYPTION_KEY": encryption_key,
        "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"}).start()
    user = r.jwt("phase-user", ["user"])
    host = r.jwt("phase-host", ["admin"], host=True)
    owner = node.http(user).request("GET", PREFIX + "/principal")["owner_key"]
    old_owner = encoded_owner("local-jwt", "urn:uar:local:jwt", None, "source")
    seed_key = peer.key(r, backend + "-seed")
    node.http(user).request("PUT", PREFIX + "/seed", {"api_key": seed_key})
    store = Store(r, backend, node)
    names = ["upgrade", "legacy-copy", "drift", "map-drift", "occupied", "race", "lost", "history", "unmapped", "missing"]
    legacy = [dict(scope="user", legacy_scope_id="source", provider_id=name,
                   destination_owner_key=owner, approval_id="phase-" + name) for name in names if name != "unmapped"]
    fixed_revision = str(uuid.uuid4())
    fixed = [dict(source_owner_key=old_owner, scope="user", source_scope_id="source", provider_id="fixed-copy",
                  source_revision=fixed_revision, destination_owner_key=owner, approval_id="phase-fixed")]
    try:
        for name in names:
            if name != "missing":
                store.seed(owner, name, None if name == "upgrade" else str(uuid.uuid4()))
        store.seed(owner, "fixed-copy", fixed_revision, old_owner)
        install_mappings(node, legacy, fixed)
        require(item(node, host, "upgrade")["status"] == "revision_unavailable", "historical revision was invented")
        node.stop()
        command = ["run", "--rm", "--network", r.network, "--volume", str(node.config_path) + ":/opt/uar/config.json:ro",
                   "--volume", node.volume + ":/var/lib/uar"]
        for key, value in node.effective_environment().items():
            command += ["--env", key + "=" + str(value)]
        for mount in node.mounts:
            command += ["--volume", ":".join(str(part) for part in mount)]
        command += [r.image, "universal-agent-runtime", "--config", "/opt/uar/config.json", "credentials", "initialize-revisions", "--writers-quiesced"]
        initialized = r.docker(*command)
        require(initialized.returncode == 0, "explicit metadata initialization failed")
        node.start()
        upgraded = item(node, host, "upgrade")
        require(upgraded["status"] == "ready" and upgraded["source_revision"], "explicit upgrade left revision unavailable")
        node.stop()
        missing_ack = r.docker(*command[:-1], check=False)
        require(missing_ack.returncode != 0, "metadata initialization accepted absent quiescence acknowledgement")
        repeated = r.docker(*command)
        require(repeated.returncode == 0, "metadata initialization retry failed")
        node.start()
        require(item(node, host, "upgrade")["source_revision"] == upgraded["source_revision"], "metadata initialization rotated an existing revision")
        result(results, backend + "-metadata-upgrade", {"before": "revision_unavailable", "after": "ready", "command_exit": initialized.returncode})

        denied = node.http(user).request("GET", PREFIX + "/migration-preview", expected=403)
        require(denied["error"]["code"] == "credential_preview_denied", "ordinary preview authority accepted")
        pages, cursor, seen = 0, None, []
        while True:
            path = PREFIX + "/migration-preview?limit=2" + ("&cursor=" + urllib.parse.quote(cursor, safe="") if cursor else "")
            page = node.http(host).request("GET", path)
            require(page["consistency"] == "page_observation" and page["counts"]["page_total"] <= 2, "preview bound incorrect")
            require(sum(page["counts"]["by_status"].values()) == page["counts"]["inventory_total"], "preview counts disagree")
            seen.extend((v["source_kind"], v["source_owner_key"] or "", v["scope"], v["source_scope_id"], v["provider_id"]) for v in page["items"])
            pages += 1
            cursor = page["next_cursor"]
            if not cursor:
                break
        require(len(seen) == len(set(seen)) and seen == sorted(seen), "keyset pagination duplicates or reorders")
        for query in ("limit=0", "limit=501", "unknown=1", "cursor=garbage", "cursor=" + "a" * 4097):
            node.http(host).request("GET", PREFIX + "/migration-preview?" + query, expected=400)
        stale_cursor = node.http(host).request("GET", PREFIX + "/migration-preview?limit=1")["next_cursor"]
        legacy[0]["approval_id"] += "-reapproved"
        install_mappings(node, legacy, fixed)
        node.http(host).request("GET", PREFIX + "/migration-preview?cursor=" + urllib.parse.quote(stale_cursor, safe=""), expected=409)
        result(results, backend + "-preview-authority-pagination", {"pages": pages, "rows": len(seen), "ordinary_status": 403, "stale_status": 409})

        tag = item(node, host, "legacy-copy")["precondition"]
        for invalid in ('*', 'W/' + tag, tag + "," + tag, 'unquoted'):
            apply(node, user, "legacy-copy", invalid, expected=400)
        apply(node, user, "legacy-copy", tag, expected=400, extra={"destination_owner_key": owner})
        other = r.jwt("foreign", ["user"], tenant="other-tenant")
        apply(node, other, "legacy-copy", tag, expected=403)
        receipt = apply(node, user, "legacy-copy", tag)
        require(receipt["source_revision"] and receipt["destination_revision"] and receipt["precondition"] == tag, "conditional receipt incomplete")
        store.retained("legacy-copy")
        result(results, backend + "-legacy-conditional-copy", {"receipt": {key: receipt[key] for key in ("source_revision", "destination_revision", "precondition")}, "wrong_owner": 403, "source_retained": True})

        fixed_tag = item(node, host, "fixed-copy")["precondition"]
        fixed_receipt = apply(node, user, "fixed-copy", fixed_tag, fixed=True, old_owner=old_owner)
        require(fixed_receipt["source_kind"] == "principal_fixed_local" and fixed_receipt["source_scope_id"] == "source", "fixed-local receipt source discriminator lost")
        node.cold_restart()
        require(apply(node, user, "fixed-copy", fixed_tag, fixed=True, old_owner=old_owner) == fixed_receipt, "fixed-local cold replay changed receipt")
        result(results, backend + "-fixed-local-conditional-copy", {"source_kind": fixed_receipt["source_kind"], "source_revision": fixed_receipt["source_revision"]})

        drift_tag = item(node, host, "drift")["precondition"]
        before_times = item(node, host, "drift")
        store.revision("drift", str(uuid.uuid4()))
        after_times = item(node, host, "drift")
        require(before_times["created_at"] == after_times["created_at"] and before_times["updated_at"] == after_times["updated_at"], "drift control changed timestamps")
        apply(node, user, "drift", drift_tag, expected=412)
        node.http(user).request("GET", PREFIX + "/drift", expected=404)
        result(results, backend + "-revision-drift", {"status": 412, "timestamps_equal": True, "destination_absent": True})

        mapping_tag = item(node, host, "map-drift")["precondition"]
        next(v for v in legacy if v["provider_id"] == "map-drift")["approval_id"] += "-changed"
        install_mappings(node, legacy, fixed)
        apply(node, user, "map-drift", mapping_tag, expected=412)
        result(results, backend + "-mapping-drift", {"status": 412, "new_tag_differs": item(node, host, "map-drift")["precondition"] != mapping_tag})

        node.http(user).request("PUT", PREFIX + "/occupied", {"api_key": seed_key})
        occupied = item(node, host, "occupied")
        require(occupied["status"] == "destination_conflict", "occupied destination reported ready")
        apply(node, user, "occupied", occupied["precondition"], expected=409)
        result(results, backend + "-destination-conflict", {"status": 409, "preview_status": occupied["status"]})

        race_tag = item(node, host, "race")["precondition"]
        with ThreadPoolExecutor(max_workers=2) as executor:
            futures = [executor.submit(apply, node, user, "race", race_tag) for _ in range(2)]
            concurrent = [future.result() for future in futures]
        require(concurrent[0] == concurrent[1], "concurrent retry did not return immutable receipt")
        result(results, backend + "-concurrent-replay", {"attempts": 2, "same_receipt": True})

        import socket
        lost_tag = item(node, host, "lost")["precondition"]
        address = urllib.parse.urlparse(node.base_url)
        body = json.dumps(dict(scope="user", legacy_scope_id="source", provider_id="lost", reason="phase approved nonsecret migration")).encode()
        with socket.create_connection((address.hostname, address.port), timeout=10) as connection:
            connection.sendall(("POST " + PREFIX + "/migrate-legacy HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer " + user + "\r\nContent-Type: application/json\r\nIf-Match: " + lost_tag + "\r\nContent-Length: " + str(len(body)) + "\r\nConnection: close\r\n\r\n").encode() + body)
        recovered = apply(node, user, "lost", lost_tag)
        require(apply(node, user, "lost", lost_tag) == recovered, "lost response recovery changed receipt")
        result(results, backend + "-lost-response-retry", {"response_discarded": True, "retry_receipt_stable": True})

        node.http(user).request("PUT", PREFIX + "/legacy-copy", {"api_key": peer.key(r, backend + "-rotated")})
        require(apply(node, user, "legacy-copy", tag) == receipt, "replay overwrote rotation")
        require(item(node, host, "legacy-copy")["status"] == "already_applied_destination_changed", "rotation preview status missing")
        node.http(user).request("DELETE", PREFIX + "/legacy-copy", expected=204)
        store.remove_source("legacy-copy")
        node.cold_restart()
        require(apply(node, user, "legacy-copy", tag) == receipt, "cold replay changed receipt")
        node.http(user).request("GET", PREFIX + "/legacy-copy", expected=404)
        require(item(node, host, "legacy-copy")["status"] == "already_applied_destination_missing", "deleted destination reported ready")
        result(results, backend + "-immutable-rotation-deletion-restart", {"same_receipt": True, "destination_absent_after_restart": True})

        old_receipt = apply(node, user, "history")
        store.historical_journal("history")
        historical = apply(node, user, "history")
        require(historical["completed_at"] == old_receipt["completed_at"] and "precondition" not in historical, "historical receipt mutated")
        apply(node, user, "history", tag, expected=412)
        require(item(node, host, "history")["status"] == "already_applied_revision_unknown", "historical destination revision invented")
        result(results, backend + "-historical-receipt", {"unconditioned_replay": 200, "conditioned_replay": 412, "status": "already_applied_revision_unknown"})

        next(v for v in legacy if v["provider_id"] == "race")["approval_id"] += "-changed"
        fixed[0]["source_revision"] = str(uuid.uuid4())
        install_mappings(node, legacy, fixed)
        statuses = {v["provider_id"]: v["status"] for v in preview(node, host)["items"]}
        for provider, expected in {"race": "journal_conflict", "fixed-copy": "journal_conflict", "missing": "missing_source", "unmapped": "unmapped_quarantined", "occupied": "destination_conflict", "lost": "already_applied", "history": "already_applied_revision_unknown", "legacy-copy": "already_applied_destination_missing"}.items():
            require(statuses[provider] == expected, "preview precedence mismatch: " + provider)
        apply(node, user, "race", race_tag, expected=409)
        apply(node, user, "fixed-copy", fixed_tag, fixed=True, old_owner=old_owner, expected=409)
        apply(node, user, "missing", expected=404)
        # A fresh mapped fixed source with a mismatched approved revision is source_changed.
        store.seed(owner, "fixed-changed", str(uuid.uuid4()), old_owner)
        fixed.append(dict(fixed[0], provider_id="fixed-changed", approval_id="phase-fixed-changed"))
        install_mappings(node, legacy, fixed)
        require(item(node, host, "fixed-changed")["status"] == "source_changed", "source_changed status missing")
        result(results, backend + "-preview-status-inventory", {"statuses": statuses, "source_changed": True})

        raw_logs = r.docker("logs", node.name, check=False)
        visible = json.dumps(preview(node, host)) + json.dumps(receipt) + raw_logs.stdout + raw_logs.stderr
        preview_json = json.dumps(preview(node, host))
        require(seed_key not in visible and encryption_key not in visible and "api_key_encrypted" not in preview_json and "api_key_hint" not in preview_json and "phase approved nonsecret migration" not in preview_json and '"reason"' not in preview_json, "credential secret or free-form reason leaked into metadata/evidence")
        result(results, backend + "-secret-redaction", {"public_metadata_secret_free": True, "backend": backend})
    finally:
        node.stop()


def identity_and_policy_cases(ctx, peer, results):
    r = ctx["resources"]
    config = configure(r, peer)
    encryption_key = secrets.token_hex(32)
    r.known_secrets.add(encryption_key)
    node = r.new_runtime("credential-policy", config, env={"CREDENTIAL_ENCRYPTION_KEY": encryption_key,
        "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"}).start()
    user = r.jwt("same", ["user"])
    host = r.jwt("phase-host", ["admin"], host=True)
    other = r.jwt("same", ["user"], tenant="foreign-tenant")
    try:
        first_owner = node.http(user).request("GET", PREFIX + "/principal")
        expected = encoded_owner("issuer", "phase-issuer", "phase-tenant", "same")
        require(first_owner["owner_key"] == expected, "external owner bytes changed")
        user_key = peer.key(r, "principal")
        node.http(user).request("PUT", PREFIX + "/openai", {"api_key": user_key})
        allowed = run_account(node, user, peer, "principal")
        node.http(other).request("GET", PREFIX + "/openai", expected=404)
        denied = run_account(node, other, peer)
        node.config["security"]["jwt_issuer"] = "different-issuer"
        node.config["security"]["trusted_host_principals"][0]["issuer"] = "different-issuer"
        rewrite(node)
        issuer_other = r.jwt("same", ["user"], issuer="different-issuer")
        node.http(issuer_other).request("GET", PREFIX + "/openai", expected=404)
        run_account(node, issuer_other, peer)
        node.config["security"]["jwt_issuer"] = "phase-issuer"
        node.config["security"]["trusted_host_principals"][0]["issuer"] = "phase-issuer"
        rewrite(node)
        result(results, "identity-isolation", {"external_bytes_unchanged": True, "allowed": allowed, "wrong_tenant": denied, "wrong_issuer_contacts": 0})

        node.http(user).request("DELETE", PREFIX + "/openai", expected=204)
        denied = run_account(node, user, peer)
        policy = node.config["security"]["provider_credentials"]
        grant = dict(issuer="phase-issuer", tenant_id="phase-tenant", provider_id="openai", source="config")
        policy["shared_fallbacks"] = [grant]
        rewrite(node)
        configured = run_account(node, user, peer, "config")
        before = len(peer.seen)
        node.http(user).request("POST", "/v1/messages", {"model":"openai/gpt-4o-mini", "max_tokens":32, "messages":[{"role":"user","content":"credential observation"}]})
        require(peer.seen[before:] and all(label == "config" for label in peer.seen[before:]), "messages facade selected unauthorized account")
        before = len(peer.seen)
        facade_status, _ = node.http(other).request("POST", "/v1/messages", {"model":"openai/gpt-4o-mini", "max_tokens":32, "messages":[{"role":"user","content":"denied observation"}]}, expected=(400,403,500,503), return_status=True)
        require(len(peer.seen) == before, "denied messages facade contacted provider")
        result(results, "messages-facade-policy", {"allowed_account":"config", "denied_status":facade_status, "denied_contacts":0})
        node.config["sycophancy"] = dict(enabled=True, auto_correct=True, auto_correct_threshold=0.0, log_only=False)
        rewrite(node)
        node.http(user).request("PUT", PREFIX + "/openai", {"api_key": user_key})
        verification = node.http(host, r.admin_key).request("POST", "/api/uar/providers/openai/test",
            {"model": "gpt-4o-mini"}, expected=200)
        require(verification["ok"] and verification["received_text"], "provider model verification failed")
        before = len(peer.seen)
        node.http(user).request("POST", "/api/chat/completion", {"model":"openai/gpt-4o-mini", "stream":True,
            "messages":[{"role":"user","content":"controlled correction observation"}]})
        wait_for("authorized poststream correction", lambda: len(peer.seen) >= before + 2, timeout=60)
        require(all(label == "principal" for label in peer.seen[before:]), "correction selected a house account")
        result(results, "response-correction-policy", {"provider_contacts":len(peer.seen)-before, "account":"principal", "auto_correct":True})
        node.config["sycophancy"]["enabled"] = False
        rewrite(node)
        node.http(user).request("DELETE", PREFIX + "/openai", expected=204)
        seed_key = peer.key(r, "system")
        node.http(user).request("PUT", PREFIX + "/seed", {"api_key": seed_key})
        store = Store(r, "surreal", node)
        store.query("LET $row=(SELECT * FROM provider_principal_credentials WHERE owner_key=" + json.dumps(expected) + " AND provider_id='seed')[0]; UPSERT type::record('provider_credentials','system:system:openai') SET scope='system',scope_id='system',provider_id='openai',api_key_encrypted=$row.api_key_encrypted,api_key_hint=$row.api_key_hint,created_at=$row.created_at,updated_at=$row.updated_at,row_revision=" + json.dumps(str(uuid.uuid4())) + " RETURN NONE;", "")
        # Config permission must not select the system row even when present.
        configured_again = run_account(node, user, peer, "config")
        policy["shared_fallbacks"] = [dict(grant, source="system")]
        rewrite(node)
        system = run_account(node, user, peer, "system")
        run_account(node, other, peer)
        result(results, "shared-source-grants", {"ungranted": denied, "config": configured, "config_skips_system": configured_again, "system": system, "foreign_tenant_contacts": 0})

        node.env.pop("CREDENTIAL_ENCRYPTION_KEY")
        node.stop(); node.start()
        # System grant alone cannot authorize a config key with service absent.
        no_service_denied = run_account(node, user, peer)
        policy["shared_fallbacks"] = [grant]
        rewrite(node)
        no_service_allowed = run_account(node, user, peer, "config")
        node.http(user).request("GET", PREFIX + "/openai", expected=503)
        result(results, "absent-service-policy", {"system_only": no_service_denied, "config_exact": no_service_allowed, "crud_unavailable": 503})

        policy["mode"] = "single_owner"
        policy["shared_fallbacks"] = []
        rewrite(node)
        remote = run_account(node, user, peer, "config")
        node.config["security"].pop("provider_credentials")
        rewrite(node)
        omitted = run_account(node, user, peer, "config")
        result(results, "single-owner-compatibility", {"explicit_remote": remote, "omitted_default": omitted})

        # Restore encrypted shared policy, then corrupt only the selected row.
        node.env["CREDENTIAL_ENCRYPTION_KEY"] = encryption_key
        node.config["security"]["provider_credentials"] = dict(mode="shared", installation_id=INSTALLATION)
        node.stop(); node.config_path.write_text(json.dumps(node.config)); node.start()
        node.http(user).request("PUT", PREFIX + "/openai", {"api_key": user_key})
        store.query("UPDATE provider_principal_credentials SET api_key_encrypted='invalid-phase-ciphertext' WHERE owner_key=" + json.dumps(expected) + " AND provider_id='openai' RETURN NONE;", "")
        decrypt_denied = run_account(node, user, peer)
        backend = r.ensure_backend("surreal")
        r.docker("stop", backend["name"])
        try:
            store_denied = run_account(node, user, peer)
        finally:
            r.docker("start", backend["name"])
            backend["host_url"] = "http://" + r.docker("port", backend["name"], "8000/tcp").stdout.strip()
            wait_for("credential backend recovery", lambda: Http(backend["host_url"]).request(
                "GET", "/health", return_status=True)[0] == 200)
        result(results, "store-decrypt-failures", {"decrypt": decrypt_denied, "backend_outage": store_denied})
    finally:
        node.stop()

    local_config = configure(r, peer, shared=False)
    local_config["security"].pop("jwt_issuer", None)
    local_config["security"].pop("jwt_audience", None)
    local_config["security"].pop("trusted_host_principals", None)
    local_config["security"]["provider_credentials"].pop("installation_id")
    local_node = r.new_runtime("local-owner", local_config, env={"CREDENTIAL_ENCRYPTION_KEY": encryption_key,
        "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"}).start()
    local = local_token(r, "local-source")
    try:
        failure = local_node.http(local).request("GET", PREFIX + "/principal", expected=503)
        require(failure["error"]["code"] == "credential_installation_id_required", "missing installation misclassified")
        local_node.config["security"]["provider_credentials"]["installation_id"] = INSTALLATION
        rewrite(local_node)
        bound = local_node.http(local).request("GET", PREFIX + "/principal")
        local_node.http(local).request("PUT", PREFIX + "/seed", {"api_key": peer.key(r, "local")})
        local_node.cold_restart()
        require(local_node.http(local).request("GET", PREFIX + "/principal") == bound, "restart changed owner")
        rotated_signing = secrets.token_hex(32)
        r.known_secrets.add(rotated_signing)
        local_node.config["security"]["jwt_secret"] = rotated_signing
        rewrite(local_node)
        local = local_token(r, "local-source", rotated_signing)
        require(local_node.http(local).request("GET", PREFIX + "/principal") == bound, "signing-key rotation changed owner")
        local_node.config["security"]["provider_credentials"]["installation_id"] = str(uuid.uuid4())
        rewrite(local_node)
        require(local_node.http(local).request("GET", PREFIX + "/principal")["owner_key"] != bound["owner_key"], "installation collision")
        local_node.http(local).request("GET", PREFIX + "/seed", expected=404)
        local_node.config["security"]["provider_credentials"]["installation_id"] = INSTALLATION
        rewrite(local_node)
        result(results, "local-installation-stability", {"missing_id_status": 503, "restart_stable": True, "signing_rotation_stable": True, "different_installation_isolated": True})
        store = Store(r, "surreal", local_node)
        old = encoded_owner("local-jwt", "urn:uar:local:jwt", None, "local-source")
        store.seed(bound["owner_key"], "quarantined", str(uuid.uuid4()), old, scope_id="local-source")
        for method, body in (("GET", None), ("PUT", {"api_key": "phase-attempt"}), ("DELETE", None)):
            denied = local_node.http(local).request(method, PREFIX + "/quarantined", body, expected=409)
            require(denied["error"]["code"] == "credential_migration_required", "old-local source bypassed")
        local_node.http(local).request("GET", PREFIX + "/seed")
        local_node.config["security"]["provider_credentials"]["mode"] = "shared"
        rewrite(local_node)
        local_node.http(local).request("GET", PREFIX + "/principal", expected=403)
        shared_local = run_account(local_node, local, peer)
        result(results, "local-quarantine", {"read_write_delete_status": 409, "bound_destination_remains_usable": True,
            "shared_local_identity_status": 403, "shared_local_run": shared_local})
    finally:
        local_node.stop()


def run_group(context):
    require(context["group"] == "credentials", "credentials adapter received another group")
    results = []
    peer = None
    try:
        peer = Peer(context["resources"])
        identity_and_policy_cases(context, peer, results)
        for backend in BACKENDS:
            migration_cases(context, peer, backend, results)
        return dict(status="passed", scenarios=results,
                    scope="installed image HTTP/provider/store/process observations; native SDK supplemental cases are separate")
    except BaseException as error:
        status = "interrupted" if isinstance(error, KeyboardInterrupt) else "blocked" if isinstance(error, Blocked) else "failed"
        completed = {value["id"] for value in results}
        for case in REQUIRED_SCENARIOS["credentials"]:
            if case not in completed:
                results.append(dict(id=case, status=status, observations=dict(error_type=type(error).__name__,
                    detail=context["resources"].redact(str(error)))))
        return dict(status=status, scenarios=results)
    finally:
        if peer is not None:
            peer.close()
