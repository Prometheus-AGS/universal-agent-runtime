"""Exact-owned Postgres Docker lifecycle; execution needs named later approval.

Import has no I/O. All service/query operations occur only in explicit actions.
No operator database, role, container, image or existing data directory is reused.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import selectors
import subprocess
import sys
import threading
import time
from urllib.parse import unquote, urlsplit


class Refusal(RuntimeError):
    pass


def need(condition, code):
    if not condition:
        raise Refusal(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def read_private(path):
    path = Path(path)
    need(path.is_absolute() and path.is_file() and not path.is_symlink()
         and path.stat().st_uid == os.getuid() and path.stat().st_mode & 0o077 == 0
         and path.stat().st_size <= 1048576, "private_fixture_input_unavailable")
    return path.read_bytes()


def secret(name):
    need(isinstance(name, str) and re.fullmatch(r"[A-Z][A-Z0-9_]*", name), "fixture_environment_reference_invalid")
    value = os.environ.get(name)
    need(isinstance(value, str) and 0 < len(value) <= 16384 and not any(char in value for char in "\r\n\0"),
         "named_fixture_environment_unavailable")
    return value


def frozen_tool(ref):
    need(isinstance(ref, dict) and set(ref) == {"path", "sha256"}, "fixture_tool_reference_invalid")
    path = Path(ref["path"])
    need(path.is_absolute() and path.is_file() and not path.is_symlink() and os.access(path, os.X_OK),
         "fixture_tool_unavailable")
    with path.open("rb") as stream:
        need("sha256:" + hashlib.file_digest(stream, "sha256").hexdigest() == ref["sha256"], "fixture_tool_digest_changed")
    return str(path)


def write_private(path, value):
    raw = canonical(value) + b"\n"
    with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())


def command(argv, environment, deadline, per_command, limit, input_bytes=None):
    remaining = min(per_command, deadline - time.monotonic())
    need(remaining > 0, "fixture_operation_deadline_exhausted")
    process = subprocess.Popen(argv, stdin=subprocess.PIPE if input_bytes is not None else subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=environment, close_fds=True)
    stop = time.monotonic() + remaining
    stdout, stderr = bytearray(), bytearray()
    selector = selectors.DefaultSelector()
    try:
        if input_bytes is not None:
            need(len(input_bytes) <= 16384, "fixture_sql_exceeds_bound")
            process.stdin.write(input_bytes)
            process.stdin.close()
        selector.register(process.stdout, selectors.EVENT_READ, stdout)
        selector.register(process.stderr, selectors.EVENT_READ, stderr)
        while selector.get_map():
            need(time.monotonic() < stop, "fixture_command_timeout")
            for key, _event in selector.select(timeout=min(0.1, max(0, stop - time.monotonic()))):
                block = os.read(key.fileobj.fileno(), min(16384, limit - len(stdout) - len(stderr) + 1))
                if not block:
                    selector.unregister(key.fileobj)
                    continue
                key.data.extend(block)
                need(len(stdout) + len(stderr) <= limit, "fixture_command_output_exceeded")
        code = process.wait(timeout=max(0.001, stop - time.monotonic()))
        return code, bytes(stdout), bytes(stderr)
    finally:
        selector.close()
        if process.poll() is None:
            process.kill()  # only the owned CLI process, never a service PID
            process.wait()
        process.stdout.close()
        process.stderr.close()


class PostgresFixture:
    def __init__(self, path, expected_config_sha):
        raw = read_private(path)
        need(sha(raw) == expected_config_sha, "postgres_config_changed_during_dispatch")
        self.config = json.loads(raw)
        self.config_sha = sha(raw)
        c = self.config
        need(c.get("schema_version") == 1 and c.get("executable") is True, "postgres_fixture_not_adopted")
        self.context = json.loads(secret("UAR_ACCEPTANCE_CONTEXT"))
        need(self.context.get("backend") == "postgres_pgvector" and isinstance(self.context.get("run_id"), str)
             and bool(self.context["run_id"]), "postgres_fixture_context_invalid")
        self.scope = "uar_acceptance_" + hashlib.sha256(self.context["run_id"].encode()).hexdigest()[:20]
        parent = Path(secret(c["state_parent_env"]))
        need(parent.is_absolute() and parent.is_dir() and not parent.is_symlink()
             and parent.stat().st_uid == os.getuid() and parent.stat().st_mode & 0o077 == 0, "postgres_private_parent_invalid")
        self.root = parent / self.scope
        need(self.root.is_dir() and not self.root.is_symlink() and self.root.stat().st_mode & 0o077 == 0,
             "runtime_owned_root_not_prepared")
        self.marker = json.loads(read_private(self.root / "ownership.json"))
        need(self.marker.get("run_id") == self.context["run_id"] and self.marker.get("backend") == "postgres_pgvector"
             and self.marker.get("scope") == self.scope and re.fullmatch(r"[a-f0-9]{32}", self.marker.get("nonce", "")),
             "runtime_ownership_marker_differs")
        self.docker, self.psql = frozen_tool(c["docker"]), frozen_tool(c["psql"])
        self.image = c["image"]
        need(isinstance(self.image, str) and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:/-]*@sha256:[0-9a-f]{64}", self.image),
             "explicit_digest_pinned_pgvector_image_required")
        for name in ("host_port", "container_pg_port"):
            need(type(c[name]) is int and 1 <= c[name] <= 65535, "postgres_port_invalid")
        need(c["host_port"] >= 1024, "postgres_host_port_must_be_nonprivileged")
        self.mount_target, self.pgdata = PurePosixPath(c["mount_target"]), PurePosixPath(c["pgdata"])
        need(self.mount_target.is_absolute() and self.pgdata.is_absolute() and ".." not in self.mount_target.parts
             and ".." not in self.pgdata.parts and self.mount_target != PurePosixPath("/")
             and self.pgdata.is_relative_to(self.mount_target) and self.pgdata != self.mount_target,
             "explicit_pgdata_must_be_inside_owned_mount")
        self.data = self.root / "pg-data"
        need("," not in str(self.data) and "," not in str(self.mount_target), "unsupported_mount_path_separator")
        self.username, self.password, self.database = (secret(c[name]) for name in ("username_env", "password_env", "database_env"))
        need(re.fullmatch(r"uar_fixture_[a-f0-9]{20}", self.username)
             and re.fullmatch(r"uar_fixture_[a-f0-9]{20}", self.database), "postgres_credentials_are_not_owned_fixture_names")
        parsed = urlsplit(secret(c["connection_env"]))
        need(parsed.scheme in ("postgres", "postgresql") and parsed.hostname == "127.0.0.1"
             and parsed.port == c["host_port"] and unquote(parsed.username or "") == self.username
             and unquote(parsed.password or "") == self.password and unquote(parsed.path) == "/" + self.database
             and not parsed.query and not parsed.fragment, "postgres_connection_does_not_match_owned_fixture")
        self.limits = c["limits"]
        for name in ("command_seconds", "ready_seconds", "cleanup_seconds", "stop_seconds", "output_bytes", "total_seconds"):
            need(type(self.limits.get(name)) is int and self.limits[name] > 0, "explicit_postgres_resource_bound_missing")
        need(self.limits["stop_seconds"] < self.limits["cleanup_seconds"], "postgres_stop_exceeds_cleanup_allowance")
        need(self.limits["total_seconds"] <= 110 and self.limits["output_bytes"] <= 1048576
             and all(self.limits[name] < self.limits["total_seconds"] for name in ("command_seconds", "ready_seconds", "cleanup_seconds"))
             and self.limits["stop_seconds"] < self.limits["command_seconds"], "postgres_inner_limits_exceed_total_owner_bound")
        self.environment = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LC_ALL": "C", "LANG": "C"}
        for name, ref in c["docker_environment_env"].items():
            need(name in ("HOME", "DOCKER_HOST", "DOCKER_CONTEXT", "DOCKER_CONFIG", "DOCKER_TLS_VERIFY", "DOCKER_CERT_PATH", "DOCKER_API_VERSION"),
                 "unapproved_docker_environment_target")
            self.environment[name] = secret(ref)
        self.labels = {"uar.acceptance.run": self.context["run_id"], "uar.acceptance.scope": self.scope,
                       "uar.acceptance.nonce": self.marker["nonce"], "uar.acceptance.backend": "postgres_pgvector"}
        self.claim_path = self.root / "postgres-fixture-claim.json"
        self.receipt_path = self.root / "postgres-fixture-prepared.json"
        self.deadline = 0

    def cli(self, argv, environment=None, input_bytes=None, allow_refused=False):
        result = command(argv, environment or self.environment, self.deadline,
                         self.limits["command_seconds"], self.limits["output_bytes"], input_bytes)
        if not allow_refused:
            need(result[0] == 0, "owned_fixture_command_failed_private_details_withheld")
        return result

    def named_containers(self):
        _, raw, errors = self.cli([self.docker, "ps", "-a", "--filter", "name=^/" + self.scope + "$", "--format", "{{.ID}}"])
        need(not errors.strip(), "docker_name_listing_reported_private_diagnostics")
        identifiers = raw.decode("ascii").split()
        need(len(identifiers) <= 1 and all(re.fullmatch(r"[a-f0-9]{12,64}", value) for value in identifiers),
             "docker_exact_name_listing_ambiguous")
        return identifiers, {"exact_name": self.scope, "listing_sha256": sha(raw), "observed_at_unix": time.time(),
                             "container_absent": not identifiers}

    def absent(self):
        identifiers, evidence = self.named_containers()
        need(not identifiers, "owned_fixture_container_name_already_exists")
        return evidence

    def inspect(self, expected_id=None):
        _, raw, _ = self.cli([self.docker, "inspect", self.scope])
        rows = json.loads(raw)
        need(isinstance(rows, list) and len(rows) == 1, "postgres_container_inspection_ambiguous")
        row = rows[0]
        need(row.get("Name") == "/" + self.scope and re.fullmatch(r"[a-f0-9]{64}", row.get("Id", ""))
             and (expected_id is None or row["Id"] == expected_id), "postgres_container_identity_differs")
        need(row.get("Config", {}).get("Image") == self.image
             and all(row.get("Config", {}).get("Labels", {}).get(key) == value for key, value in self.labels.items()),
             "postgres_image_or_ownership_labels_differ")
        mounts = row.get("Mounts", [])
        need(len(mounts) == 1 and mounts[0].get("Type") == "bind" and mounts[0].get("RW") is True
             and mounts[0].get("Source") == str(self.data) and mounts[0].get("Destination") == str(self.mount_target)
             and self.data.is_dir() and not self.data.is_symlink(), "postgres_data_mount_not_exact_owned_durable_directory")
        ports = row.get("HostConfig", {}).get("PortBindings", {})
        need(ports == {str(self.config["container_pg_port"]) + "/tcp": [{"HostIp": "127.0.0.1", "HostPort": str(self.config["host_port"])}]},
             "postgres_port_binding_is_not_exact_owned_loopback")
        _, image_raw, _ = self.cli([self.docker, "image", "inspect", self.image])
        images = json.loads(image_raw)
        need(len(images) == 1 and self.image in images[0].get("RepoDigests", [])
             and row.get("Image") == images[0].get("Id"), "actual_postgres_image_digest_unconfirmed")
        return {"container_id": row["Id"], "container_name": self.scope, "image": self.image,
                "image_id": row["Image"], "labels": self.labels, "mount_source": str(self.data),
                "mount_target": str(self.mount_target), "pgdata": str(self.pgdata),
                "host": "127.0.0.1", "host_port": self.config["host_port"],
                "container_pg_port": self.config["container_pg_port"], "running": row.get("State", {}).get("Running") is True,
                "inspection_sha256": sha(raw)}

    def pg(self, query, allow_refused=False):
        environment = dict(self.environment)
        environment.update(PGHOST="127.0.0.1", PGPORT=str(self.config["host_port"]), PGUSER=self.username,
            PGPASSWORD=self.password, PGDATABASE=self.database, PGAPPNAME="uar-owned-postgres-fixture",
            PGCONNECT_TIMEOUT=str(min(self.limits["command_seconds"], max(1, int(self.deadline - time.monotonic())))))
        return self.cli([self.psql, "-X", "-w", "-q", "-A", "-t", "-v", "ON_ERROR_STOP=1"], environment, query.encode(), allow_refused)

    def prepare(self):
        self.deadline = time.monotonic() + self.limits["ready_seconds"]
        need(not self.claim_path.exists() and not self.receipt_path.exists() and not self.data.exists(), "postgres_fixture_is_not_fresh")
        self.absent()
        # Confirm the exact approved image is already local. --pull=never also
        # forbids a race from introducing an implicit unapproved registry pull.
        _, raw, _ = self.cli([self.docker, "image", "inspect", self.image])
        images = json.loads(raw)
        need(len(images) == 1 and self.image in images[0].get("RepoDigests", []), "approved_local_postgres_image_unavailable")
        self.data.mkdir(mode=0o700)
        claim = {"schema_version": 1, "config_sha256": self.config_sha, "run_id": self.context["run_id"],
                 "scope": self.scope, "nonce": self.marker["nonce"], "labels": self.labels,
                 "image": self.image, "mount_source": str(self.data), "state": "creation_reserved"}
        write_private(self.claim_path, claim)
        env_path = self.root / "postgres-container.env"
        with os.fdopen(os.open(env_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "w") as stream:
            for name, value in (("POSTGRES_USER", self.username), ("POSTGRES_PASSWORD", self.password),
                                ("POSTGRES_DB", self.database), ("PGDATA", str(self.pgdata))):
                need(not any(char in value for char in "\r\n\0"), "container_environment_value_invalid")
                stream.write(name + "=" + value + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        argv = [self.docker, "run", "--detach", "--pull=never", "--name", self.scope]
        for key, value in self.labels.items():
            argv.extend(("--label", key + "=" + value))
        argv.extend(("--mount", "type=bind,src=" + str(self.data) + ",dst=" + str(self.mount_target),
                     "--publish", f"127.0.0.1:{self.config['host_port']}:{self.config['container_pg_port']}",
                     "--env-file", str(env_path), self.image))
        _, raw, _ = self.cli(argv)
        container_id = raw.decode().strip()
        need(re.fullmatch(r"[a-f0-9]{64}", container_id), "created_container_id_unconfirmed")
        actual = self.inspect(container_id)
        need(actual["running"], "owned_postgres_container_not_running")
        query = "SELECT json_build_object('ready',1,'database',current_database(),'username',current_user,'data_directory',current_setting('data_directory'),'port',current_setting('port')::integer,'vector_available',EXISTS(SELECT 1 FROM pg_available_extensions WHERE name='vector'),'server_version_num',current_setting('server_version_num'));"
        while True:
            code, output, error = self.pg(query, allow_refused=True)
            if code == 0:
                ready = json.loads(output)
                break
            # Only concrete connection-refused startup is a wait condition.
            # Authentication, SQL, timeout and arbitrary server errors stop.
            need(code == 2 and b"connection refused" in error.lower(), "postgres_readiness_failed_private_details_withheld")
            need(time.monotonic() < self.deadline, "owned_postgres_connection_refused_until_deadline")
            time.sleep(min(0.1, max(0, self.deadline - time.monotonic())))
        need(ready.get("ready") == 1 and ready.get("database") == self.database and ready.get("username") == self.username
             and ready.get("data_directory") == str(self.pgdata) and ready.get("port") == self.config["container_pg_port"]
             and ready.get("vector_available") is True, "actual_owned_postgres_or_pgvector_unavailable")
        _, output, _ = self.pg("CREATE EXTENSION vector; SELECT json_build_object('extension',(SELECT extversion FROM pg_extension WHERE extname='vector'),'database',current_database(),'username',current_user);")
        installed = json.loads(output)
        need(isinstance(installed.get("extension"), str) and bool(installed["extension"])
             and installed.get("database") == self.database and installed.get("username") == self.username,
             "pgvector_extension_installation_unconfirmed")
        actual = self.inspect(container_id)
        receipt = {"schema_version": "uar.owned-postgres-fixture.v1", "status": "prepared", "run_id": self.context["run_id"],
                   "scope": self.scope, "config_sha256": self.config_sha, "ownership_nonce": self.marker["nonce"],
                   "actual": actual, "ready": ready, "pgvector": installed, "data_retained": True}
        write_private(self.receipt_path, receipt)
        return receipt

    def cleanup(self):
        self.deadline = time.monotonic() + self.limits["cleanup_seconds"]
        claim = None
        if self.claim_path.exists() or self.claim_path.is_symlink():
            claim = json.loads(read_private(self.claim_path))
            need(claim.get("config_sha256") == self.config_sha and claim.get("run_id") == self.context["run_id"]
                 and claim.get("scope") == self.scope and claim.get("image") == self.image
                 and claim.get("mount_source") == str(self.data)
                 and claim.get("nonce") == self.marker["nonce"] and claim.get("labels") == self.labels,
                 "postgres_cleanup_ownership_claim_differs")
        expected_id = None
        if self.receipt_path.exists():
            receipt = json.loads(read_private(self.receipt_path))
            need(claim is not None and receipt.get("config_sha256") == self.config_sha
                 and receipt.get("ownership_nonce") == self.marker["nonce"],
                 "postgres_cleanup_prepared_receipt_differs")
            expected_id = receipt["actual"]["container_id"]
        identifiers, evidence = self.named_containers()
        if not identifiers:
            # Creation can fail before a container exists. Successful exact-name
            # listing is positive absence evidence; inspect/transport failures
            # are never converted into absence, and no container ID is invented.
            receipt = {"schema_version": "uar.owned-postgres-fixture.v1", "status": "cleaned", "scope": self.scope,
                       "run_id": self.context["run_id"], "config_sha256": self.config_sha,
                       "ownership_nonce": self.marker["nonce"], "claim_validated": claim is not None,
                       "container_absent": True, "container_removed": False, "absence_evidence": evidence,
                       "data_retained": True, "data_path": str(self.data)}
            write_private(self.root / "postgres-fixture-cleaned.json", receipt)
            return receipt
        need(claim is not None, "container_exists_without_owned_creation_claim")
        actual = self.inspect(expected_id)
        # Immutable actual ID, never an ambient name or caller-provided selector.
        if actual["running"]:
            self.cli([self.docker, "stop", "--time", str(self.limits["stop_seconds"]), actual["container_id"]])
        self.inspect(actual["container_id"])
        self.cli([self.docker, "rm", actual["container_id"]])
        evidence = self.absent()
        receipt = {"schema_version": "uar.owned-postgres-fixture.v1", "status": "cleaned", "scope": self.scope,
                   "run_id": self.context["run_id"], "actual_before_stop": actual,
                   "config_sha256": self.config_sha, "ownership_nonce": self.marker["nonce"], "claim_validated": True,
                   "container_absent": True, "container_removed": True, "absence_evidence": evidence,
                   "data_retained": True, "data_path": str(self.data)}
        write_private(self.root / "postgres-fixture-cleaned.json", receipt)
        return receipt


def main():
    need(len(sys.argv) == 3 and sys.argv[1] in ("postgres-prepare", "postgres-cleanup"), "expected_exact_postgres_fixture_action_and_config")
    raw = read_private(sys.argv[2])
    total = json.loads(raw)["limits"]["total_seconds"]
    need(type(total) is int and 0 < total <= 110, "postgres_total_owner_deadline_invalid")
    guard = threading.Timer(total, lambda: os._exit(75))
    guard.daemon = True
    guard.start()
    try:
        fixture = PostgresFixture(sys.argv[2], sha(raw))
        receipt = fixture.prepare() if sys.argv[1] == "postgres-prepare" else fixture.cleanup()
        print(json.dumps(receipt, sort_keys=True), flush=True)
    finally:
        guard.cancel()


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"error": "owned_postgres_fixture_unconfirmed", "code": str(error) if isinstance(error, Refusal)
                          else type(error).__name__}), flush=True)
        sys.exit(2)
