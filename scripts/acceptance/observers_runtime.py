"""Concrete owned-fixture observers. Importing starts no process or service.

Execution is exclusively part of the separately approved whole-phase packet.
"""
from __future__ import annotations

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

from observers_startup import (ObservationFailure, Store, need, pg_string,
                               run_scope, save, secret_ref)
from linux_runtime import LinuxRuntime, route_endpoint, route_tree

LIMIT = 2 * 1024 * 1024


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Observer:
    def __init__(self, config, context, fixture):
        self.config, self.context, self.fixture = config, context, fixture
        self.scope = run_scope(context)
        self.backend = next((item for item in fixture["backends"] if item["name"] == context["backend"]), None)
        need(self.backend is not None, "observer backend is absent from frozen fixture")
        self.timeout = config.get("command_seconds", 30)
        need(type(self.timeout) is int and 0 < self.timeout <= 120, "invalid observer command bound")
        self.frozen = {str(Path(item["path"]).resolve()): item["sha256"] for item in fixture["frozen_files"]}
        parent = Path(secret_ref(config["state_dir_env"]))
        need(str(parent) == secret_ref("UAR_ACCEPTANCE_STATE_PARENT"), "observer and core state parents differ")
        need(parent.is_absolute() and parent.is_dir() and not parent.is_symlink(), "private state parent must already exist")
        need(parent.stat().st_mode & 0o077 == 0, "state parent must be private")
        self.root = parent / self.scope
        os.environ["UAR_ACCEPTANCE_OWNED_STATE_DIR"] = str(self.root)
        os.environ["UAR_ACCEPTANCE_OWNED_SCOPE"] = self.scope
        os.environ["UAR_ACCEPTANCE_OWNED_LABEL"] = "uar.acceptance.scope=" + self.scope
        os.environ["UAR_ACCEPTANCE_OWNED_RUN_LABEL"] = "uar.acceptance.run=" + self.context["run_id"]
        os.environ["UAR_ACCEPTANCE_OWNED_BACKEND_LABEL"] = "uar.acceptance.backend=" + self.context["backend"]
        os.environ["UAR_ACCEPTANCE_SURREAL_CID_FILE"] = str(self.root / "surreal-container.cid")
        os.environ.pop("UAR_ACCEPTANCE_SURREAL_CONTAINER_ID", None)
        os.environ["UAR_ACCEPTANCE_SURREAL_VOLUME"] = str(self.root / "surreal-data") + ":/data"
        os.environ["UAR_ACCEPTANCE_MEMORY_PATH"] = str(self.root / "memory")
        os.environ["UAR_ACCEPTANCE_TOOLS_EFFECTS_ROOT"] = str(self.root / "tools-effects")
        self.known_secrets = []
        refs = list(self.backend.get("headers_env", {}).values())
        refs.extend(reference for key, reference in self.backend.get("environment_env", {}).items()
                    if re.search(r"key|password|secret|token|credential|connection|authorization", key + reference, re.I))
        refs.extend(binding.get("credential_env") for binding in [fixture.get("provider", {}), *fixture.get("genuine_bindings", [])])
        refs.extend(fixture.get("secret_env", []))
        for section in (config.get("postgres", {}), config.get("surreal", {})):
            refs.extend(value for key, value in section.items() if key in ("connection_env", "password_env"))
        for reference in refs:
            if isinstance(reference, str) and os.environ.get(reference):
                self.known_secrets.append(os.environ[reference])
        connection_ref = config.get("postgres", {}).get("connection_env")
        if connection_ref and os.environ.get(connection_ref):
            password = urllib.parse.urlsplit(os.environ[connection_ref]).password
            if password:
                self.known_secrets.extend((password, urllib.parse.unquote(password)))
        self.opener = urllib.request.build_opener(NoRedirect)

    def redact(self, value):
        if isinstance(value, dict):
            return {key: ("[REDACTED]" if re.search(r"password|secret|authorization|cookie|api.?key|reasoning|thinking|thought", key, re.I)
                          else self.redact(item)) for key, item in value.items()}
        if isinstance(value, list):
            return [self.redact(item) for item in value]
        if isinstance(value, str):
            for secret in sorted(self.known_secrets, key=len, reverse=True):
                value = value.replace(secret, "[REDACTED]")
            value = re.sub(r"([a-zA-Z][a-zA-Z0-9+.-]*://)[^/\s@]+@", r"\1[REDACTED]@", value)
        return value

    def frozen_file(self, value):
        path = Path(value)
        need(path.is_absolute() and path.is_file() and not path.is_symlink(), "required frozen file is unavailable")
        need(self.frozen.get(str(path.resolve())) == sha(path.read_bytes()), "frozen file bytes changed or were omitted")
        return path

    def render(self, value):
        if isinstance(value, dict):
            return {key: self.render(item) for key, item in value.items()}
        if isinstance(value, list):
            return [self.render(item) for item in value]
        if value == "@scope@":
            return self.scope
        if isinstance(value, str) and value.startswith("@env:") and value.endswith("@"):
            return secret_ref(value[5:-1])
        need(not isinstance(value, str) or "@scope@" not in value and "@env:" not in value,
             "substitutions must occupy an entire JSON leaf or argv element")
        return value

    def owned(self):
        need(self.root.is_dir() and not self.root.is_symlink() and self.root.stat().st_mode & 0o077 == 0,
             "exact run-owned private directory is unavailable")
        marker = json.loads((self.root / "ownership.json").read_bytes())
        need(marker.get("run_id") == self.context["run_id"] and marker.get("backend") == self.context["backend"]
             and marker.get("scope") == self.scope and marker.get("profile_id") == self.backend["profile_id"]
             and re.fullmatch(r"[a-f0-9]{32}", marker.get("nonce", "")), "fixture ownership marker differs")
        os.environ["UAR_ACCEPTANCE_OWNED_NONCE_LABEL"] = "uar.acceptance.nonce=" + marker["nonce"]
        return marker

    def command(self, argv, input_bytes=None, environment=None, require_clean_stderr=False):
        argv = self.render(argv)
        need(isinstance(argv, list) and argv and all(isinstance(item, str) for item in argv), "exact finite command argv required")
        self.frozen_file(argv[0])
        for argument in argv[1:]:
            path = Path(argument)
            if path.is_absolute() and path.is_file() and not path.is_relative_to(self.root):
                self.frozen_file(argument)
        with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
            process = subprocess.Popen(argv, stdin=subprocess.PIPE if input_bytes is not None else subprocess.DEVNULL,
                stdout=output, stderr=errors, env=environment or os.environ.copy(), start_new_session=True)
            try:
                if input_bytes is not None:
                    process.stdin.write(input_bytes)
                    process.stdin.close()
                deadline = time.monotonic() + self.timeout
                while process.poll() is None:
                    need(output.tell() <= LIMIT and errors.tell() <= LIMIT, "finite command output allowance exceeded")
                    if time.monotonic() >= deadline:
                        raise ObservationFailure("finite observer command timed out")
                    try:
                        process.wait(timeout=0.05)
                    except subprocess.TimeoutExpired:
                        pass
                process.wait()
            finally:
                if process.poll() is None:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=self.timeout)
            output.seek(0)
            errors.seek(0)
            raw, stderr = output.read(LIMIT + 1), errors.read(LIMIT + 1)
        need(len(raw) <= LIMIT and len(stderr) <= LIMIT, "finite command output exceeded bound")
        if process.returncode != 0:
            # Arbitrary stderr can include DSNs or credentials. Preserve its exact
            # byte identity while explicitly leaving opaque upstream cause open.
            raise ObservationFailure("finite command failed; stdout=" + sha(raw) + "; stderr=" + sha(stderr)
                                     + "; opaque upstream cause unrecovered")
        need(not require_clean_stderr or not stderr.strip(),
             "container listing emitted diagnostics; stderr=" + sha(stderr))
        return raw

    def pg(self, query):
        settings = self.config["postgres"]
        environment = os.environ.copy()
        environment.update(PGDATABASE=secret_ref(settings["connection_env"]),
                           PGOPTIONS="-c search_path=" + self.scope,
                           PGAPPNAME="uar-acceptance-runtime-observer")
        raw = self.command([settings["psql_binary"], "-X", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1"],
                           query.encode(), environment)
        return json.loads(raw)

    def surreal(self, query, database=True):
        settings = self.config["surreal"]
        url = secret_ref(settings["url_env"])
        parsed = urllib.parse.urlsplit(url)
        need(parsed.scheme == "http" and parsed.hostname in ("localhost", "127.0.0.1", "::1")
             and parsed.path in ("", "/") and not parsed.username and not parsed.password
             and not parsed.query and not parsed.fragment, "Surreal requires owned loopback origin")
        credentials = secret_ref(settings["username_env"]) + ":" + secret_ref(settings["password_env"])
        namespace = settings["namespace"]
        need(re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]*", namespace), "invalid owned namespace identifier")
        headers = {"Authorization": "Basic " + base64.b64encode(credentials.encode()).decode(),
                   "Surreal-NS": namespace, "Accept": "application/json", "Content-Type": "text/plain"}
        if database:
            headers["Surreal-DB"] = self.scope
        request = urllib.request.Request(url.rstrip("/") + "/sql", data=query.encode(), headers=headers, method="POST")
        with self.opener.open(request, timeout=self.timeout) as response:
            raw = response.read(LIMIT + 1)
        need(len(raw) <= LIMIT, "Surreal result exceeded bound")
        results = json.loads(raw)
        need(isinstance(results, list) and results and all(item.get("status") == "OK" for item in results),
             "Surreal checked statement failed")
        return results[-1]["result"]

    def api(self, path):
        need(path.startswith("/") and not path.startswith("//"), "API path must remain on owned origin")
        headers = {header: secret_ref(reference) for header, reference in self.backend.get("headers_env", {}).items()}
        request = urllib.request.Request(self.backend["base_url"].rstrip("/") + path, headers=headers)
        with self.opener.open(request, timeout=self.timeout) as response:
            raw = response.read(LIMIT + 1)
        need(len(raw) <= LIMIT, "owned API result exceeded bound")
        return json.loads(raw)

    def store(self):
        self.owned()
        return Store(self.config, self.context)

    def lifecycle(self, name):
        self.owned()
        argv = self.lifecycle_settings().get(name)
        if argv:
            raw = self.command(argv)
            save(self.root / ("lifecycle-" + name + "-" + uuid.uuid4().hex + ".json"),
                 {"operation": name, "stdout_digest": sha(raw), "run_id": self.context["run_id"]})
            return raw
        return None

    def lifecycle_settings(self):
        return self.config.get("store_lifecycle", {}).get(self.context["backend"], {})

    def private_receipt(self, path):
        need(path.is_file() and not path.is_symlink() and path.stat().st_uid == os.getuid()
             and path.stat().st_mode & 0o077 == 0 and path.stat().st_size <= LIMIT,
             "owned private lifecycle receipt is unavailable")
        return json.loads(path.read_bytes())

    def postgres_handshake(self, status="prepared"):
        marker = self.owned()
        path = self.frozen_file(self.config["postgres_store"]["config_path"])
        native = json.loads(path.read_bytes())
        need(native.get("connection_env") == self.config["postgres"]["connection_env"]
             and native.get("psql", {}).get("path") == self.config["postgres"]["psql_binary"],
             "Postgres lifecycle and observer target different native inputs")
        receipt = self.private_receipt(self.root / ("postgres-fixture-" + status + ".json"))
        need(receipt.get("schema_version") == "uar.owned-postgres-fixture.v1"
             and receipt.get("status") == status and receipt.get("run_id") == self.context["run_id"]
             and receipt.get("scope") == self.scope and receipt.get("ownership_nonce") == marker["nonce"]
             and receipt.get("config_sha256") == sha(path.read_bytes()),
             "Postgres native lifecycle handshake does not match actual frozen configuration and owner")
        if status == "prepared":
            actual = receipt.get("actual", {})
            need(re.fullmatch(r"[a-f0-9]{64}", actual.get("container_id", ""))
                 and actual.get("image") == native.get("image") and actual.get("running") is True
                 and actual.get("labels") == self.store_labels()
                 and actual.get("mount_source") == str(self.root / "pg-data")
                 and actual.get("host_port") == native.get("host_port") and receipt.get("data_retained") is True,
                 "Postgres prepared handshake lacks actual exact-owned durable container evidence")
            ready = receipt.get("ready", {})
            need(ready.get("ready") == 1 and ready.get("data_directory") == native.get("pgdata")
                 and ready.get("port") == native.get("container_pg_port")
                 and ready.get("database") == secret_ref(native["database_env"])
                 and ready.get("username") == secret_ref(native["username_env"])
                 and ready.get("vector_available") is True and bool(receipt.get("pgvector", {}).get("extension")),
                 "Postgres prepared handshake lacks actual owned SQL and pgvector readiness")
        else:
            need(receipt.get("container_absent") is True
                 and receipt.get("absence_evidence", {}).get("container_absent") is True
                 and receipt.get("absence_evidence", {}).get("exact_name") == self.scope
                 and re.fullmatch(r"sha256:[a-f0-9]{64}", receipt.get("absence_evidence", {}).get("listing_sha256", ""))
                 and receipt.get("data_retained") is True
                 and receipt.get("data_path") == str(self.root / "pg-data"),
                 "Postgres teardown did not confirm actual container absence")
        return receipt

    def store_labels(self):
        marker = self.owned()
        return {"uar.acceptance.run": self.context["run_id"], "uar.acceptance.scope": self.scope,
                "uar.acceptance.nonce": marker["nonce"], "uar.acceptance.backend": self.context["backend"]}

    def surreal_cid(self):
        path = self.root / "surreal-container.cid"
        if not path.exists():
            return None
        need(path.is_file() and not path.is_symlink() and path.stat().st_uid == os.getuid()
             and path.stat().st_size <= 128, "Surreal immutable ID file differs from owned lifecycle")
        value = path.read_text().strip()
        if not value:
            return None
        need(re.fullmatch(r"[a-f0-9]{64}", value), "Surreal immutable container ID is malformed")
        return value

    def surreal_container_ids(self):
        settings = self.config["surreal"]["disk_container"]
        selector_sets = [["--filter", "name=^/" + self.scope + "$"]]
        label_filters = []
        for key, value in self.store_labels().items():
            label_filters.extend(("--filter", "label=" + key + "=" + value))
        selector_sets.append(label_filters)
        cid = self.surreal_cid()
        if cid:
            selector_sets.append(["--filter", "id=" + cid])
        found = set()
        digests = []
        for selectors in selector_sets:
            raw = self.command([settings["docker_binary"], "ps", "-a", "--no-trunc", *selectors, "--format", "{{.ID}}"], require_clean_stderr=True)
            ids = raw.decode().splitlines()
            need(all(re.fullmatch(r"[a-f0-9]{64}", item) for item in ids), "Surreal container listing was not actual immutable IDs")
            found.update(ids)
            digests.append(sha(raw))
        return found, digests

    def disk(self, require_running=True):
        """Inspect actual immutable identity and exact nonce-bearing ownership."""
        settings = self.config["surreal"]["disk_container"]
        container = secret_ref(settings["container_env"])
        need(container == self.scope, "Surreal disk container must have exact invocation-owned name")
        cid = self.surreal_cid()
        raw = self.command([settings["docker_binary"], "inspect", cid or container])
        rows = json.loads(raw)
        need(len(rows) == 1, "disk container identity is ambiguous")
        row = rows[0]
        need(row.get("Name") == "/" + self.scope and re.fullmatch(r"[a-f0-9]{64}", row.get("Id", ""))
             and (cid is None or row["Id"] == cid), "Surreal immutable container identity differs")
        need(not require_running or row.get("State", {}).get("Running") is True,
             "owned disk container is stopped")
        image = settings["image"]
        need("@sha256:" in image and row.get("Config", {}).get("Image") == image, "Surreal container image differs from frozen pin")
        args = row.get("Args", [])
        need(args == self.render(settings["args"]), "actual Surreal daemon arguments differ from frozen command")
        need(all(row.get("Config", {}).get("Labels", {}).get(key) == value for key, value in self.store_labels().items()),
             "Surreal container lacks exact run/scope/nonce/backend ownership labels")
        endpoint = next((arg for arg in args if isinstance(arg, str) and arg.startswith(("rocksdb:", "surrealkv:"))), None)
        need(endpoint is not None and not any(arg in ("memory", "mem://") for arg in args), "Surreal daemon did not select a disk engine")
        container_path = endpoint.split(":", 1)[1]
        mount = next((item for item in row.get("Mounts", []) if item.get("RW") is True
                      and (container_path == item.get("Destination") or container_path.startswith(item.get("Destination", "") + "/"))), None)
        need(mount is not None, "Surreal engine path has no retained writable mount")
        expected_source = self.root / "surreal-data"
        need(mount.get("Type") == "bind" and Path(mount.get("Source", "")).resolve() == expected_source.resolve()
             and expected_source.is_dir() and not expected_source.is_symlink(), "Surreal disk mount is outside exact run-owned directory")
        bindings = row.get("HostConfig", {}).get("PortBindings", {}).get("8000/tcp", [])
        endpoint_url = urllib.parse.urlsplit(secret_ref(self.config["surreal"]["url_env"]))
        need(len(bindings) == 1 and bindings[0].get("HostIp") == "127.0.0.1"
             and bindings[0].get("HostPort") == str(endpoint_url.port), "actual Surreal loopback port differs")
        os.environ["UAR_ACCEPTANCE_SURREAL_CONTAINER_ID"] = row["Id"]
        return {"container_id": row["Id"], "image": image, "engine_endpoint": endpoint,
                "labels": self.store_labels(), "disk_path": str(expected_source), "container_path": container_path,
                "actual_process_pid": row.get("State", {}).get("Pid"), "inspect_digest": sha(raw)}

    def prepare(self):
        self.root.mkdir(mode=0o700, exist_ok=False)
        marker = {"run_id": self.context["run_id"], "backend": self.context["backend"], "scope": self.scope,
                  "profile_id": self.backend["profile_id"], "nonce": uuid.uuid4().hex}
        save(self.root / "ownership.json", marker)
        (self.root / "surreal-data").mkdir(mode=0o700)
        (self.root / "ingest").mkdir(mode=0o700)
        (self.root / "home").mkdir(mode=0o700)
        (self.root / "tmp").mkdir(mode=0o700)
        lifecycle = self.lifecycle_settings()
        need(bool(lifecycle.get("prepare_argv")) == bool(lifecycle.get("cleanup_argv")),
             "store lifecycle must pair explicit preparation and cleanup")
        if self.context["backend"] == "surreal_disk":
            need(lifecycle.get("prepare_argv") and lifecycle.get("cleanup_argv"),
                 "disk Surreal requires paired exact-owned container lifecycle")
        self.lifecycle("prepare_argv")
        if self.context["backend"] == "postgres_pgvector":
            self.postgres_handshake()
            exists = self.pg("SELECT to_json(EXISTS(SELECT 1 FROM pg_namespace WHERE nspname=" + pg_string(self.scope) + "))")
            need(exists is False, "refusing an existing Postgres fixture schema")
            self.pg('CREATE SCHEMA "' + self.scope + '"; SELECT to_json(true)')
        else:
            namespace = self.config["surreal"]["namespace"]
            need(re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]*", namespace), "invalid owned namespace identifier")
            deadline = time.monotonic() + self.config.get("ready_seconds", 60)
            while True:
                try:
                    namespaces = self.surreal("INFO FOR ROOT;", database=False)
                    break
                except (OSError, urllib.error.URLError):
                    need(time.monotonic() < deadline, "owned Surreal service did not become available")
                    time.sleep(0.1)
            need(namespace not in namespaces.get("namespaces", {}), "fresh run-owned Surreal container already has a namespace")
            self.surreal("DEFINE NAMESPACE " + namespace + ";", database=False)
            before = self.surreal("INFO FOR NS;", database=False)
            need(self.scope not in before.get("databases", {}), "refusing an existing Surreal fixture database")
            self.surreal("DEFINE DATABASE " + self.scope + ";", database=False)
            self.disk()
        store = self.store()
        counts = store.counts()
        need(all(value == 0 for value in counts.values()), "new owned application store is not empty")
        receipt = {**marker, "store_identity": store.storage_identity, "counts": counts,
                   "empty_owned_store": all(value == 0 for value in counts.values())}
        save(self.root / "prepared.json", receipt)
        return {**receipt, "owned": True}

    def identities(self):
        source_files = self.config["source_files"]
        need(isinstance(source_files, list) and source_files, "frozen source inventory missing")
        source = [{"path": str(self.frozen_file(path)), "sha256": self.frozen[str(Path(path).resolve())]}
                  for path in sorted(source_files)]
        binary = self.frozen_file(self.config["server_binary"])
        template = self.frozen_file(self.config["server_template"])
        configurations = [{"path": str(self.frozen_file(path)), "sha256": self.frozen[str(Path(path).resolve())]}
                          for path in sorted(self.config["configuration_files"])]
        need(configurations and str(template) in {item["path"] for item in configurations},
             "global configuration inventory omits selected server template")
        return {"source_identity": sha(canonical(source)), "artifact_identity": sha(binary.read_bytes()),
                "configuration_revision": sha(canonical(configurations)),
                "schema_revision": sha(canonical(self.config["schema_contract"]))}

    def generated_config(self):
        config = self.render(json.loads(self.frozen_file(self.config["server_template"]).read_bytes()))
        persistence = config["persistence"]
        if self.context["backend"] == "postgres_pgvector":
            need(persistence["provider"] == "postgres", "server template selects wrong store")
            original = urllib.parse.urlsplit(secret_ref(self.config["postgres"]["connection_env"]))
            need(original.scheme in ("postgres", "postgresql"), "Postgres connection must be explicit URL")
            query = [(key, value) for key, value in urllib.parse.parse_qsl(original.query) if key != "options" and not key.startswith("options[")]
            query.append(("options", "-c search_path=" + self.scope + ",public"))
            persistence["database_url"] = urllib.parse.urlunsplit(original._replace(query=urllib.parse.urlencode(query)))
        else:
            settings = self.config["surreal"]
            need(persistence["provider"] == "surreal", "server template selects wrong store")
            persistence.update(database_url=secret_ref(settings["url_env"]), surreal_ns=settings["namespace"],
                               surreal_db=self.scope, surreal_user=secret_ref(settings["username_env"]),
                               surreal_pass=secret_ref(settings["password_env"]), surreal_auth_level="root")
            self.disk()
        config["server"]["ingest_directory"] = str(self.root / "ingest")
        memory = config.get("memory", {})
        if memory.get("enabled"):
            need(memory.get("namespace") == self.scope and memory.get("database") == self.scope,
                 "enabled memory must use exact owned namespace/database")
            need(memory.get("surreal_endpoint") is None
                 and Path(memory.get("db_path", "")).resolve().is_relative_to(self.root.resolve())
                 and Path(memory.get("db_path", "")).resolve() != self.root.resolve(),
                 "enabled memory storage must be inside owned private directory")
        if self.config.get("runtime_platform", "macos-native") == "linux-container":
            config = route_tree(config, self.config["linux_runtime"])
            config["server"]["host"] = "0.0.0.0"
        return config

    def runtime_endpoint(self, value):
        if self.config.get("runtime_platform", "macos-native") == "linux-container":
            return route_endpoint(value, self.config["linux_runtime"])
        return value

    def serve(self):
        self.owned()
        identities = self.identities()
        need(all(identities[key] == self.fixture[key] for key in identities), "frozen identity contract differs")
        config = self.generated_config()
        mcp = self.render(json.loads(self.frozen_file(self.config["mcp_template"]).read_bytes()))
        mcp_path = self.root / "mcp.json"
        if mcp_path.exists():
            need(not mcp_path.is_symlink() and mcp_path.stat().st_mode & 0o077 == 0
                 and canonical(json.loads(mcp_path.read_bytes())) == canonical(mcp),
                 "existing owned MCP configuration differs from frozen template")
        else:
            save(mcp_path, mcp)
        static = Path(self.config["static_directory"])
        need(static.is_absolute() and static.is_dir() and not static.is_symlink(),
             "explicit installed static directory is unavailable")
        static_files = self.config["static_files"]
        need(isinstance(static_files, list) and static_files
             and str(static / "index.html") in static_files, "complete installed static closure must include index.html")
        for value in static_files:
            need(self.frozen_file(value).resolve().is_relative_to(static.resolve()),
                 "installed static input escapes the selected directory")
        team_max_active = self.config["team_execution_max_active"]
        need(type(team_max_active) is int and 3 <= team_max_active <= 4,
             "team fixture must explicitly admit three or four active members")
        binary = self.frozen_file(self.config["server_binary"])
        path = self.root / ("server-" + uuid.uuid4().hex + ".json")
        save(path, config)
        launch = {**identities, "pid": os.getpid(), "run_id": self.context["run_id"], "backend": self.context["backend"],
                  "launch_nonce": uuid.uuid4().hex,
                  "config_path": str(path), "rendered_config_digest": sha(path.read_bytes()),
                  "binary_path": str(binary), "argv": [str(binary), "--config", str(path)],
                  "mcp_path": str(mcp_path), "mcp_digest": sha(mcp_path.read_bytes()),
                  "static_directory": str(static), "team_execution_max_active": team_max_active}
        save(self.root / ("launch-" + str(os.getpid()) + ".json"), launch)
        # Config derives from the frozen template; ambient UAR/legacy overrides
        # must not redirect this owned process into another operator datastore.
        environment = {key: value for key, value in os.environ.items()
                       if not key.startswith(("UAR_", "LLM_")) and key not in
                       ("PORT", "CONFIG_FILE", "JWT_REQUIRED", "RATE_LIMIT_ENABLED", "DATABASE_URL", "PGOPTIONS", "MCP_CONFIG_PATH", "MCP_CONFIG_DIR")}
        for key, reference in self.config.get("server_environment_env", {}).items():
            need(not key.startswith("UAR_") and key not in ("CONFIG_FILE", "PORT", "DATABASE_URL", "PGOPTIONS", "JWT_REQUIRED", "MCP_CONFIG_PATH", "MCP_CONFIG_DIR"),
                 "server environment may not override rendered UAR configuration")
            environment[key] = secret_ref(reference)
        environment["UAR_STATIC_DIR"] = str(static)
        environment["UAR_TEAM_EXECUTION_MAX_ACTIVE"] = str(team_max_active)
        if self.context["backend"] == "surreal_disk":
            environment["UAR_REMOTE_SURREAL_DURABILITY_ATTESTED"] = "1"
        # Relative product defaults also belong to this invocation, including
        # credentials/files created beneath an otherwise implicit working dir.
        os.chdir(self.root)
        if self.config.get("runtime_platform", "macos-native") == "linux-container":
            LinuxRuntime(self).launch(launch, environment)
        os.execve(str(binary), launch["argv"], environment)

    def launch(self):
        self.owned()
        pid = int(secret_ref("UAR_ACCEPTANCE_SERVER_PID"))
        need(pid > 1, "invalid owned runtime PID")
        value = json.loads((self.root / ("launch-" + str(pid) + ".json")).read_bytes())
        need(value.get("pid") == pid and value.get("run_id") == self.context["run_id"]
             and value.get("backend") == self.context["backend"], "runtime launch receipt mismatch")
        os.kill(pid, 0)
        need(sha(self.frozen_file(value["binary_path"]).read_bytes()) == value["artifact_identity"], "installed binary changed since launch")
        path = Path(value["config_path"])
        need(path.parent == self.root and path.is_file() and not path.is_symlink()
             and path.stat().st_mode & 0o077 == 0
             and sha(path.read_bytes()) == value["rendered_config_digest"], "rendered configuration changed")
        mcp_path = Path(value["mcp_path"])
        need(mcp_path == self.root / "mcp.json" and mcp_path.is_file() and not mcp_path.is_symlink()
             and sha(mcp_path.read_bytes()) == value["mcp_digest"], "owned MCP launch configuration changed")
        if self.config.get("runtime_platform", "macos-native") == "linux-container":
            live = LinuxRuntime(self).inspect(value)
            value["container_id"] = live["Id"]
            value["container_started_at"] = live["State"]["StartedAt"]
        return value

    def stop_runtime(self):
        self.owned()
        need(self.config.get("runtime_platform") == "linux-container", "container stop requires Linux runtime profile")
        pid = int(secret_ref("UAR_ACCEPTANCE_SERVER_PID"))
        need(self.context.get("values", {}).get("process_pid") == pid,
             "Linux stop request differs from selected foreground runtime PID")
        receipt = self.private_receipt(self.root / ("launch-" + str(pid) + ".json"))
        need(receipt.get("pid") == pid and receipt.get("run_id") == self.context["run_id"]
             and receipt.get("backend") == self.context["backend"] and receipt.get("runtime_platform") == "linux-container",
             "Linux stop receipt does not belong to selected runtime")
        result = LinuxRuntime(self).stop(receipt)
        result.update(run_id=self.context["run_id"], container_absent=True)
        save(self.root / ("runtime-stopped-" + receipt["launch_nonce"] + ".json"), result)
        return result

    def ready(self):
        seconds = self.config.get("ready_seconds", 60)
        need(type(seconds) is int and 0 < seconds <= 120, "invalid readiness bound")
        deadline = time.monotonic() + seconds
        last = "unavailable"
        while time.monotonic() < deadline:
            try:
                self.launch()
                health, ready = self.api("/healthz"), self.api("/readyz")
                if health.get("status") == "ok" and ready.get("status") == "ready":
                    descriptor = self.api("/api/uar/full-harness/v1/capabilities")
                    need(bool(descriptor.get("runtime_epoch")), "actual process runtime epoch missing")
                    return {"ready": True, "pid": self.launch()["pid"], "runtime_epoch": descriptor["runtime_epoch"],
                            "health": health, "readiness": ready}
                last = "actual health/readiness status was not ready"
            except (OSError, urllib.error.URLError, ValueError):
                last = "bounded owned readiness transport unavailable"
            time.sleep(0.1)
        raise ObservationFailure(last)

    def schema(self, store):
        contract = self.config["schema_contract"][self.context["backend"]]
        required = contract["tables"]
        need(isinstance(required, dict) and required, "schema table contract missing")
        if self.context["backend"] == "postgres_pgvector":
            rows = store.pg_query("SELECT COALESCE(json_agg(json_build_object('table',table_name,'column',column_name) ORDER BY table_name,ordinal_position),'[]') "
                                  "FROM information_schema.columns WHERE table_schema=current_schema()")
            observed = {}
            for row in rows:
                observed.setdefault(row["table"], []).append(row["column"])
            for table, columns in required.items():
                need(table in observed and set(columns).issubset(observed[table]), "actual Postgres schema violates frozen table/column contract")
            installed = store.pg_query("SELECT COALESCE(json_agg(json_build_object('version',version,'checksum',encode(checksum,'hex'),'success',success) ORDER BY version),'[]') FROM _sqlx_migrations")
            need(installed and all(row.get("success") is True for row in installed), "successful installed migration ledger unavailable")
            sources = contract.get("migration_sources", [])
            need(bool(sources), "frozen SQLx migration source denominator missing")
            expected = {}
            for source in sources:
                path = self.frozen_file(source)
                prefix = path.name.split("_", 1)[0]
                need(prefix.isdigit(), "unsupported source migration filename")
                version = int(prefix)
                need(version not in expected, "duplicate frozen migration version")
                expected[version] = hashlib.sha384(path.read_bytes()).hexdigest()
            need({row["version"]: row["checksum"] for row in installed} == expected,
                 "installed migration versions/checksums differ from frozen source denominator")
            return {"tables": observed, "migrations": installed}
        info = store.sql("INFO FOR DB;")
        tables = info.get("tables", {})
        need(all(table in tables for table in required), "actual Surreal schema lacks frozen required tables")
        observed = {}
        for table in required:
            need(re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_]*", table), "unsupported schema table identifier")
            value = store.sql("INFO FOR TABLE " + table + ";")
            need(all(column in value.get("fields", {}) for column in required[table]), "Surreal defined-field contract differs")
            observed[table] = value
        return {"tables": observed}

    def admission(self):
        launch = self.launch()
        actual = self.identities()
        need(all(launch.get(key) == expected == self.fixture[key] for key, expected in actual.items()), "observed source/artifact/config/schema identity mismatch")
        prepared = json.loads((self.root / "prepared.json").read_bytes())
        store = self.store()
        need(prepared.get("store_identity") == store.storage_identity and prepared.get("empty_owned_store") is True,
             "admitted store differs from actual empty prepared store")
        schema = self.schema(store)
        live = self.api("/api/config/persistence")
        rendered = json.loads(Path(launch["config_path"]).read_bytes())["persistence"]
        need(live.get("provider") == rendered["provider"], "running host selected a different persistence adapter")
        actual_url, configured_url = urllib.parse.urlsplit(live["database_url"]), urllib.parse.urlsplit(rendered["database_url"])
        configured_scheme = {"http": "ws", "https": "wss"}.get(configured_url.scheme, configured_url.scheme)
        need(actual_url.scheme == configured_scheme and actual_url.hostname == configured_url.hostname
             and actual_url.port == configured_url.port and actual_url.path.rstrip("/") == configured_url.path.rstrip("/")
             and urllib.parse.parse_qsl(actual_url.query) == urllib.parse.parse_qsl(configured_url.query),
             "running host persistence endpoint/database options differ from rendered owned configuration")
        # The URL endpoint is scrubbed by production. Actual scoped Store identity,
        # generated DSN/DB and populated application tables complete this binding.
        need(store.counts().get("agents", 0) > 0, "actual prepared store has no runtime-seeded agent records")
        runtime = self.api("/api/uar/full-harness/v1/capabilities")
        value = {**actual, "run_id": self.context["run_id"], "backend": self.context["backend"],
            "profile_id": self.backend["profile_id"], "durable": True,
            "empty_owned_store": prepared["empty_owned_store"], "store_identity": store.storage_identity,
            "runtime_epoch": runtime["runtime_epoch"], "actual_schema": schema,
            "actual_schema_digest": sha(canonical(schema)), "live_persistence": live,
            "rendered_config_digest": launch["rendered_config_digest"]}
        if self.context["backend"] == "postgres_pgvector":
            extension = store.pg_query("SELECT COALESCE(json_agg(json_build_object('name',e.extname,'version',e.extversion,'schema',n.nspname)),'[]') "
                                       "FROM pg_extension e JOIN pg_namespace n ON n.oid=e.extnamespace WHERE e.extname='vector'")
            need(len(extension) == 1, "actual pgvector extension absent")
            need(extension[0]["schema"] in (self.scope, "public"), "actual pgvector schema is outside runtime search path")
            value.update(pgvector=True, extension=extension)
        else:
            disk = self.disk()
            value.update(ephemeral=False, disk_path=disk["disk_path"], disk=disk)
        return value

    def inference(self, values):
        self.launch()
        run_id, session_id, model = values["run_id"], values["session_id"], values["qualified_model"]
        primary = values.get("primary_qualified_model", model)
        role = values.get("role", "primary")
        need(role in ("primary", "auxiliary"), "inference role is not explicit")
        need(role != "primary" or model == primary, "primary inference subgroup differs from primary model")
        primary_provider, primary_model = primary.split("/", 1)
        provider_id, model_id = model.split("/", 1)
        quote = lambda value: urllib.parse.quote(value, safe="")
        run = self.api("/api/uar/runs/" + quote(run_id))
        need(run.get("run_id") == run_id and run.get("conversation_id") == session_id and run.get("status") == "done"
             and run.get("effective_model") == {"provider_id": primary_provider, "model_id": primary_model}, "actual primary inference binding/terminal differs")
        provider = self.api("/api/uar/providers/" + quote(provider_id))
        need(provider.get("id") == provider_id and provider.get("enabled") is True, "actual provider is unavailable")
        bindings = [self.fixture["provider"], *self.fixture.get("genuine_bindings", [])]
        binding = next((item for item in bindings if item.get("qualified_model") == model), None)
        need(binding is not None and binding.get("kind") == "genuine", "actual dispatched model has no frozen genuine binding")
        expected_provider = binding["config"]
        need(all(provider.get(key) == expected_provider.get(key) for key in ("id", "base_url", "protocol", "enabled"))
             and any(item.get("id") == model_id and item.get("enabled") is True for item in provider.get("models", [])),
             "actual provider configuration differs from frozen genuine binding")
        endpoint = urllib.parse.urlsplit(provider["base_url"])
        need(endpoint.scheme in ("http", "https") and endpoint.hostname and not endpoint.username and not endpoint.password
             and not endpoint.query and not endpoint.fragment, "actual provider endpoint is not admissible")
        origin = urllib.parse.urlunsplit((endpoint.scheme, endpoint.netloc, "", "", ""))
        approved = self.config["approved_genuine_origins"]
        need(any(item.get("origin") == origin and provider_id in item.get("provider_ids", []) for item in approved),
             "actual inference endpoint is not an operator-approved genuine origin")
        verification = provider.get("model_verifications", {}).get(model_id)
        need(isinstance(verification, dict) and verification.get("configuration_digest"), "current provider configuration verification missing")
        calls = self.api("/api/uar/accounting/calls/" + quote(run_id))
        need(isinstance(calls, list) and calls, "actual dispatched call receipts missing")
        requested_ids = values.get("call_ids")
        need(requested_ids is None or isinstance(requested_ids, list) and requested_ids
             and all(isinstance(item, str) and item for item in requested_ids), "invalid inference call subset")
        subgroup = [call for call in calls if call.get("admission", {}).get("model") == model
                    and (requested_ids is None or call.get("admission", {}).get("identity", {}).get("call_id") in requested_ids)]
        need(bool(subgroup), "requested actual model dispatch subgroup missing")
        if requested_ids is not None:
            need({call.get("admission", {}).get("identity", {}).get("call_id") for call in subgroup} == set(requested_ids),
                 "actual provider call subset omitted requested identities")
        for call in subgroup:
            identity = call.get("admission", {}).get("identity", {})
            need(identity.get("run_id") == run_id and identity.get("owner") and identity.get("call_id")
                 and call.get("admission", {}).get("model") == model and call.get("state") == "settled"
                 and call.get("observed_tokens", 0) > 0 and call.get("reconciliation_evidence"),
                 "actual call is missing correlated settled provider usage")
        forwarding = self.forwarding(model, binding, run_id, len(subgroup))
        return {"run_id": run_id, "session_id": session_id, "qualified_model": model, "kind": "genuine", "synthetic": False,
            "base_url": provider["base_url"], "configuration_digest": verification["configuration_digest"],
            "primary_qualified_model": primary, "role": role,
            "provider_receipt": {"calls": subgroup, "run": run, "provider_verification": verification,
                                 "approved_origin": origin, "forwarding": forwarding,
                                 "meaning": "settled runtime dispatch through the owned approved genuine forwarder"}}

    def forwarding(self, model, binding, run_id, call_count):
        """Read actual shared forwarding state; loopback alone is no genuine proof."""
        contract = self.config["budget_proxy"]
        config_path = self.frozen_file(contract["config_path"])
        config = json.loads(config_path.read_bytes())
        provider_id, _ = model.split("/", 1)
        binding_id = contract["provider_bindings"][provider_id]
        selected = next((item for item in config["bindings"] if item["id"] == binding_id), None)
        need(selected is not None and selected["kind"] == "model", "provider lacks a model forwarding binding")
        endpoint = urllib.parse.urlsplit(binding["config"]["base_url"])
        metadata_hop = None
        if endpoint.port != selected["listen_port"]:
            cap = self.fixture["capabilities"]
            need(model == cap["structured_binding_model"] and cap.get("structured_proxy_budget_hop_approved") is True
                 and endpoint.port == cap["genuine_proxy_port"] and endpoint.path.rstrip("/") == "/v1"
                 and Path(cap["structured_proxy_budget_config"]) == config_path,
                 "unapproved intermediate positive-response listener")
            hop = urllib.parse.urlsplit(cap["genuine_upstream"])
            need(hop.scheme == "http" and hop.hostname == "127.0.0.1" and hop.port == selected["listen_port"],
                 "structured metadata forwarder bypasses shared admission")
            effects = self.root / "tools-effects"
            listener_path = effects / ("genuine-proxy-" + str(endpoint.port) + ".json")
            need(listener_path.is_file() and not listener_path.is_symlink(), "metadata forwarding owner receipt is missing")
            metadata_hop = json.loads(listener_path.read_bytes())
            need(metadata_hop.get("port") == endpoint.port and metadata_hop.get("upstream") == cap["genuine_upstream"]
                 and metadata_hop.get("pid") in json.loads(secret_ref("UAR_ACCEPTANCE_OWNED_PROCESS_IDS")),
                 "metadata forwarding process is not invocation-owned")
            os.kill(metadata_hop["pid"], 0)
            endpoint = hop
        expected_endpoint = urllib.parse.urlsplit(self.runtime_endpoint("http://127.0.0.1:" + str(selected["listen_port"]) + endpoint.path))
        need(endpoint.scheme == "http" and endpoint.hostname == ("127.0.0.1" if metadata_hop is not None else expected_endpoint.hostname)
             and endpoint.port == selected["listen_port"]
             and selected["request_path"] == endpoint.path.rstrip("/") + "/chat/completions",
             "actual provider can bypass its named forwarding listener")
        # The qualified provider selector may be rewritten to a provider-native
        # alias; the frozen explicit allowlist owns those actual wire names.
        need(bool(selected["allowed_models"]), "approved actual wire model allowlist is empty")
        root = Path(config["ledger_root"])
        expected = "uar_budget_" + hashlib.sha256(config["epoch_id"].encode()).hexdigest()[:20]
        need(root.is_absolute() and root.name == expected and root.is_dir() and not root.is_symlink()
             and root.stat().st_mode & 0o077 == 0, "shared forwarding ledger ownership differs")
        pid = int(secret_ref("UAR_ACCEPTANCE_BUDGET_PID"))
        nonce = secret_ref("UAR_ACCEPTANCE_BUDGET_NONCE")
        identity = secret_ref("UAR_ACCEPTANCE_BUDGET_INPUT_IDENTITY")
        os.kill(pid, 0)
        ready_path = root / ("ready-" + nonce + ".json")
        need(ready_path.is_file() and not ready_path.is_symlink(), "owned forwarding readiness file is missing")
        ready = json.loads(ready_path.read_bytes())
        need(ready.get("pid") == pid and ready.get("nonce") == nonce
             and ready.get("config_sha256") == sha(config_path.read_bytes())
             and ready.get("epoch", {}).get("input_identity") == identity,
             "forwarding readiness is not this packet-owned process")
        database = root / "ledger.sqlite3"
        need(database.is_file() and not database.is_symlink(), "actual forwarding ledger is missing")
        with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True, timeout=self.timeout) as connection:
            owner_row = connection.execute("SELECT value FROM meta WHERE key='owner'").fetchone()
            need(owner_row is not None and json.loads(owner_row[0]) == {"pid": pid, "nonce": nonce},
                 "forwarding ledger has a different actual owner")
            rows = connection.execute("SELECT request_id,receipt FROM reservations WHERE binding_id=? AND kind='model' AND state='completed'",
                                      (binding_id,)).fetchall()
        receipts = [(identifier, json.loads(value)) for identifier, value in rows]
        completed = [(identifier, value) for identifier, value in receipts
                     if value.get("owner_nonce") == nonce and value.get("model") in selected["allowed_models"]
                     and value.get("owner_pid") == pid and value.get("input_identity") == identity
                     and value.get("binding_id") == binding_id and value.get("upstream_url") == selected["upstream_url"]
                     and value.get("request_path") == selected["request_path"]
                     and 200 <= value.get("http_status", 0) < 300
                     and value.get("response_sha256", "").startswith("sha256:")]
        need(len(completed) >= call_count, "approved genuine upstream has insufficient actual successful forwarding receipts")
        store = self.store()
        if self.context["backend"] == "postgres_pgvector":
            records = store.pg_query("SELECT COALESCE(json_agg(data),'[]') FROM durable_run_records WHERE run_id=" + pg_string(run_id))
        else:
            records = store.sql("SELECT VALUE data FROM durable_run_records WHERE run_id=" + json.dumps(run_id) + ";")
            records = [json.loads(value) if isinstance(value, str) else value for value in records]
        need(len(records) == 1 and records[0].get("run_id") == run_id, "forwarding proof lacks exact durable run")
        context = records[0]["run"].get("context", {})
        wire_hashes = {item.get("serialized_sha256") for item in context.get("request_budgets", [])
                       if item.get("destination_model") == model and item.get("serialized_request") is True}
        matched = [(identifier, value) for identifier, value in completed
                   if value.get("request_wire_sha256") in wire_hashes]
        need(not wire_hashes or bool(matched), "actual serialized run request lacks matching genuine upstream forwarding")
        if metadata_hop is not None:
            trace = self.root / "tools-effects" / "mcp-receipts.jsonl"
            need(trace.is_file() and not trace.is_symlink() and trace.stat().st_size <= LIMIT,
                 "structured metadata forwarding trace is missing or exceeds allowance")
            records = [json.loads(line) for line in trace.read_text().splitlines()]
            chained = []
            for identifier, value in matched:
                for row in records:
                    if row.get("operation") != "genuine-wire" or row.get("request_sha256") != value["request_wire_sha256"]:
                        continue
                    hop_receipt = row.get("budget_hop") or {}
                    need(hop_receipt.get("binding_id") == binding_id and hop_receipt.get("owner_pid") == pid
                         and hop_receipt.get("owner_nonce") == nonce and hop_receipt.get("input_identity") == identity,
                         "structured metadata hop bound a different forwarding authority")
                    reply = next((item for item in records if item.get("operation") == "genuine-wire-response"
                                  and item.get("request_id") == row.get("request_id")), None)
                    if reply and reply.get("status") == value["http_status"] and reply.get("sha256") == value["response_sha256"]:
                        chained.append(identifier)
            need(bool(chained), "structured response lacks a complete metadata-to-budget-to-upstream byte chain")
        return {"binding_id": binding_id, "upstream_url": selected["upstream_url"], "epoch_id": config["epoch_id"],
                "pid": pid, "nonce": nonce, "input_identity": identity,
                "metadata_hop": metadata_hop,
                "completed_binding_requests": len(completed), "matched_wire_request_ids": [row[0] for row in matched],
                "correlation": "exact serialized run wire hash" if matched else "owned binding plus settled runtime accounting; no per-call wire identity available",
                "limit": "helper and graph calls without serialized receipts do not acquire an exact forwarding call identity"}

    def session(self, values):
        store = self.store()
        session_id = str(uuid.UUID(values["session_id"]))
        metadata = store.session(session_id)
        need(metadata is not None, "actual durable session unavailable")
        data = store.session_data(metadata["owner_id"], session_id)
        need(data is not None and isinstance(data.get("messages"), list), "actual canonical durable messages unavailable")
        self.launch()
        runtime = self.api("/api/uar/full-harness/v1/capabilities")
        return {"session_id": session_id, "backend": self.context["backend"], "durable": True,
                "owner_id": metadata["owner_id"], "storage_key": metadata["storage_key"],
                "store_identity": store.storage_identity, "canonical_messages": data["messages"],
                "runtime_epoch": runtime["runtime_epoch"]}

    def cleanup_capture(self):
        store = self.store()
        counts = store.counts()
        jobs = {}
        for table in ("knowledge_ingestion", "principal_knowledge_ingestion"):
            if table not in counts:
                continue
            if self.context["backend"] == "postgres_pgvector":
                rows = store.pg_query("SELECT COALESCE(json_agg(json_build_object('owner_id',owner_id,'document_id',document_id,'kb_id',kb_id,'state',state,'next_attempt_at',next_attempt_at,"
                    "'generation',data->>'generation','attempts',data->>'attempts','last_error',left(data->>'last_error',1024),'regeneration',data->>'regeneration')),'[]') "
                    "FROM (SELECT * FROM " + table + " ORDER BY owner_id,document_id LIMIT 100) j")
            else:
                rows = store.sql("SELECT owner_id, document_id, kb_id, state, next_attempt_at, data FROM " + table + " LIMIT 100;")
                for row in rows:
                    data = json.loads(row.pop("data"))
                    need(isinstance(data, dict), "persisted typed ingestion metadata is not an object")
                    for key in ("generation", "attempts", "last_error", "regeneration"):
                        row[key] = data.get(key)
                    if isinstance(row.get("last_error"), str):
                        row["last_error"] = row["last_error"][:1024]
            jobs[table] = {"rows": rows, "omitted_rows": max(0, counts[table] - len(rows))}
        return self.redact({"run_id": self.context["run_id"], "backend": self.context["backend"],
                            "capture_status": "available", "counts_before_drop": counts,
                            "typed_jobs_before_drop": jobs, "store_identity": store.storage_identity})

    def cleanup_failure(self, error):
        # Never persist arbitrary upstream exception text, SQL rows or credentials.
        return {"error_type": type(error).__name__, "detail_digest": sha(str(error).encode()),
                "cause": "capture or teardown unavailable; opaque upstream detail withheld"}

    def teardown_store(self):
        need(self.lifecycle_settings().get("cleanup_argv"), "owned datastore cleanup lifecycle is missing")
        if self.context["backend"] == "postgres_pgvector":
            self.lifecycle("cleanup_argv")
            receipt = self.postgres_handshake("cleaned")
            return {"container_absent": True, "receipt_digest": sha(canonical(receipt)),
                    "container_removed": receipt.get("container_removed"),
                    "absence_evidence": receipt["absence_evidence"]}
        identifiers, before = self.surreal_container_ids()
        removed = False
        if identifiers:
            actual = self.disk(require_running=False)
            need(identifiers == {actual["container_id"]}, "Surreal cleanup ownership selectors disagree")
            # The only cleanup selector is the immutable ID from actual inspect.
            argv = self.lifecycle_settings()["cleanup_argv"]
            need(argv == [self.config["surreal"]["disk_container"]["docker_binary"], "rm", "--force",
                          "@env:UAR_ACCEPTANCE_SURREAL_CONTAINER_ID@"],
                 "Surreal cleanup must remove only the inspected immutable container ID")
            self.lifecycle("cleanup_argv")
            removed = True
        remaining, after = self.surreal_container_ids()
        need(not remaining, "exact owned store container remains after cleanup")
        return {"container_absent": True, "container_removed": removed,
                "absence_evidence": {"container_absent": True, "before_listing_digests": before,
                                     "after_listing_digests": after}}

    def cleanup(self, values):
        self.owned()
        need(values.get("processes_joined") is True, "runtime must be joined before datastore cleanup")
        # A live owned runtime still has authority to write; teardown is refused.
        for path in self.root.glob("launch-*.json"):
            receipt = self.private_receipt(path)
            if self.config.get("runtime_platform", "macos-native") == "linux-container":
                LinuxRuntime(self).stop(receipt)
            pid = receipt["pid"]
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                continue
            raise ObservationFailure("recorded runtime process is still alive; refusing store teardown")
        failures = {}
        try:
            retained = self.cleanup_capture()
            need(len(canonical(retained)) <= LIMIT, "cleanup metadata exceeded bound")
        except Exception as error:
            failures["capture"] = self.cleanup_failure(error)
            retained = {"run_id": self.context["run_id"], "backend": self.context["backend"],
                        "capture_status": "unavailable", "counts_before_drop": None,
                        "typed_jobs_before_drop": None, "store_identity": None,
                        "failure": failures["capture"]}
        capture_saved = False
        try:
            # Parent receives bounded redacted evidence before any destructive action.
            print(json.dumps({"cleanup_before_drop": retained}), file=sys.stderr, flush=True)
            save(self.root / "cleanup-before-drop.json", retained)
            capture_saved = True
        except Exception as error:
            failures["capture_retention"] = self.cleanup_failure(error)
        dropped = False
        if retained["capture_status"] == "available" and capture_saved:
            try:
                if self.context["backend"] == "postgres_pgvector":
                    self.pg('DROP SCHEMA "' + self.scope + '" CASCADE; SELECT to_json(true)')
                    absent = self.pg("SELECT to_json(NOT EXISTS(SELECT 1 FROM pg_namespace WHERE nspname=" + pg_string(self.scope) + "))")
                else:
                    self.surreal("REMOVE DATABASE " + self.scope + ";", database=False)
                    absent = self.scope not in self.surreal("INFO FOR NS;", database=False).get("databases", {})
                need(absent is True, "owned schema/database removal was not confirmed")
                dropped = True
            except Exception as error:
                failures["store_drop"] = self.cleanup_failure(error)
        teardown = None
        try:
            # Independent of capture/readiness: remove owned service, retain disk.
            teardown = self.teardown_store()
        except Exception as error:
            failures["store_lifecycle"] = self.cleanup_failure(error)
        active = [] if teardown is not None else ["owned-store-absence-unconfirmed"]
        if not failures and dropped and teardown is not None:
            marker = self.owned()
            need(marker["scope"] == self.root.name and self.root.parent == Path(secret_ref(self.config["state_dir_env"])),
                 "private cleanup path escaped the exact owned root")
            shutil.rmtree(self.root)
            need(not self.root.exists(), "owned private configuration directory remains")
            return {"run_id": self.context["run_id"], "residuals": [], "active_residuals": [],
                    "retained_before_drop": retained, "store_teardown": teardown, "private_state_retained": False}
        result = {"run_id": self.context["run_id"], "residuals": [str(self.root), *active],
                  "active_residuals": active, "private_state_retained": True,
                  "retained_before_drop": retained, "store_teardown": teardown, "failures": failures}
        # No empty jobs/counts are invented when startup or forensic capture failed.
        try:
            save(self.root / ("cleanup-incomplete-" + uuid.uuid4().hex + ".json"), self.redact(result))
        except Exception as error:
            result["failures"]["final_retention"] = self.cleanup_failure(error)
        return result


def main():
    context = json.loads(secret_ref("UAR_ACCEPTANCE_CONTEXT"))
    fixture_path = Path(secret_ref("UAR_ACCEPTANCE_FIXTURE_PATH"))
    fixture = json.loads(fixture_path.read_bytes())
    config_path = Path(secret_ref("UAR_ACCEPTANCE_RUNTIME_CONFIG"))
    config = json.loads(config_path.read_bytes())
    need(config.get("schema_version") == 1 and config.get("executable") is True, "runtime observer config is not executable schema1")
    observer = Observer(config, context, fixture)
    observer.frozen_file(str(config_path))
    observer.frozen_file(str(Path(__file__).resolve()))
    observer.frozen_file(str(Path(__file__).resolve().with_name("observers_startup.py")))
    action = sys.argv[1] if len(sys.argv) == 2 else ""
    if action == "serve":
        observer.serve()
        raise ObservationFailure("server exec unexpectedly returned")
    values = context.get("values", {})
    if action == "prepare":
        result = observer.prepare()
    elif action == "ready":
        result = observer.ready()
    elif action == "admission":
        result = observer.admission()
    elif action == "inference":
        result = observer.inference(values)
    elif action == "session":
        result = observer.session(values)
    elif action == "cleanup":
        result = observer.cleanup(values)
    elif action == "stop-runtime":
        result = observer.stop_runtime()
    else:
        raise ObservationFailure("unknown concrete runtime observer action")
    print(json.dumps(observer.redact(result), sort_keys=True, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"error": "runtime observer failed; no successful observation", "error_type": type(error).__name__,
            "detail": str(error) if isinstance(error, ObservationFailure) else "upstream cause unrecovered; private error withheld"}))
        sys.exit(2)
