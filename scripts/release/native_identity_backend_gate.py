#!/usr/bin/env python3
"""Prepared r9 backend gate. Running this file requires separate explicit approval.

This supervisor does not authorize itself. It runs only the four complete backend
integration targets and the exact customer build. Submission hooks, formatting,
protocol11, identity/client gates and publication are separate approvals.

Readiness and graceful stop/join deliberately retain the existing helper's
unbounded waits. A live progress receipt is not a finite cleanup guarantee.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import signal
import stat
import subprocess
import sys
import threading
import time
import tomllib
import urllib.parse
import uuid

sys.dont_write_bytecode = True
from uar_1_1_1_support import (AttachedContainer, Blocked, Resources,
                               SURREAL_IMAGE, postgres_final_server, surreal_listening)

ROOT = Path(__file__).resolve().parents[2]
POSTGRES_IMAGE = "docker.io/supabase/postgres:17.6.1.173@sha256:da28f45caeb2d0857a5f53759654859a46c469112f257290e28bc2c291727506"
AGENT_TESTS = {
    "tree_limits_refuse_the_fifth_concurrent_child",
    "tree_limits_refuse_the_fourth_child_nesting_level",
    "tree_limits_refuse_the_seventeenth_lifetime_child",
    "child_history_fork_none_contains_only_the_delegated_prompt",
    "child_history_fork_last_two_keeps_dialogue_and_drops_tool_traffic",
    "inter_agent_message_keeps_sender_identity_out_of_the_child_user_turn",
    "lifecycle_events_are_content_free_and_drive_both_agui_mappings",
    "in_memory_spawn_persists_identically_ordered_threads_and_edges",
    "surreal_spawn_persists_identically_ordered_threads_and_edges",
    "postgres_spawn_persists_identically_ordered_threads_and_edges",
}
TARGETS = (
    ("agent_threads", "server-full,postgres-backend,in-memory-backend", 10),
    ("fresh_process_resume", "server-full,postgres-backend", 1),
    ("stream_terminal_contract", "server-full", 1),
    ("approval_recovery", "server-full,postgres-backend,in-memory-backend", 1),
)
SINGLE_TESTS = {
    "fresh_process_resume": "durable_fresh_process_resume_production_boundaries",
    "stream_terminal_contract": "stream_terminal_contract_production_boundaries",
    "approval_recovery": "durable_approval_recovery_production_boundaries",
}
RESUME_CASES = ("resume", "target", "negative", "policy", "binding", "abrupt",
                "uncertain", "kill-before", "kill-after", "corrupt")
APPROVAL_CASES = ("pending-restart", "race", "deny", "expiration", "revoked",
                  "tightened", "binding", "abrupt", "unknown", "siblings", "legacy",
                  "legacy-deny", "invalidation-outage")
STREAM_PROTOCOLS = ("OpenAi", "Azure", "LiterAnthropic", "NativeAnthropic",
                    "GoogleAi", "Vertex", "Cohere", "Bedrock")
STREAM_CASES = ("valid", "split", "empty", "missing_terminal", "partial_after_terminal",
                "partial_event", "malformed", "invalid_utf8", "malformed_after_terminal",
                "tool", "tool_missing_terminal", "cancel")


def require(condition, message):
    if not condition:
        raise Blocked(message)


def sha256(path):
    with Path(path).open("rb") as source:
        digest = hashlib.sha256()
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def private_write(path, value):
    path = Path(path)
    require(not path.is_symlink(), "refusing symlink output")
    with path.open("w", encoding="utf-8") as output:
        os.chmod(path, 0o600)
        output.write(value)


def save(path, value):
    private_write(path, json.dumps(value, indent=2, sort_keys=True) + "\n")


def load(path):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError) as error:
        raise Blocked("missing or non-parseable JSON: " + str(path)) from error


def private_dir(path):
    Path(path).mkdir(mode=0o700)
    os.chmod(path, 0o700)
    return Path(path)


def private_input(path):
    path = Path(path)
    require(path.is_absolute() and not path.is_symlink(), "private input must be absolute and non-symlink")
    info = path.stat()
    require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
            and stat.S_IMODE(info.st_mode) == 0o600, "private input requires owner and mode0600")
    return path


def no_dotenv(path):
    for directory in (Path(path), *Path(path).parents):
        require(not (directory / ".env").exists() and not (directory / ".env").is_symlink(),
                "dotenv ancestry must be absent: " + str(directory))


class ScopedAttachment(AttachedContainer):
    """Reuse readiness/join semantics, replacing the ambient-environment launch."""
    def __init__(self, resources, name, recognize):
        self.resources, self.name = resources, name
        self.condition = threading.Condition()
        self.receipt = self.error = self.exit_code = None
        self.path = resources.evidence / (name + "-attach-" + uuid.uuid4().hex + ".log")
        self.process = subprocess.Popen(
            [resources.docker_binary, "start", "--attach", name],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, errors="replace", env=resources.child_env, cwd=resources.work,
            start_new_session=True)

        def consume():
            try:
                with self.path.open("w") as output:
                    os.chmod(self.path, 0o600)
                    for line in self.process.stdout:
                        output.write(resources.redact(line))
                        output.flush()
                        with self.condition:
                            if self.receipt is None and self.error is None:
                                try:
                                    self.receipt = recognize(line)
                                except Exception as error:
                                    self.error = type(error).__name__
                                self.condition.notify_all()
            except Exception as error:
                with self.condition:
                    self.error = type(error).__name__
                    self.condition.notify_all()
            finally:
                self.process.stdout.close()
                code = self.process.wait()
                with self.condition:
                    self.exit_code = code
                    self.condition.notify_all()

        self.reader = threading.Thread(target=consume, name="attach-" + name, daemon=False)
        self.reader.start()


class BackendResources(Resources):
    """Bounded adapter: no runtime leases, provider configuration or host stores."""
    def __init__(self, settings, output, resource_root, environment, work):
        manifest = {"resource_root": str(resource_root), "image": settings["postgres_image"],
                    "platform": settings["platform"]}
        super().__init__(manifest, {}, output)
        self.settings = settings
        self.child_env = dict(environment)
        self.work = work
        self.docker_binary = settings["docker_binary"]
        self.owner = uuid.uuid4().hex
        self.label = "uar.native-identity-backend-owner=" + self.owner
        self.pending = []
        self.cleanup_results = []
        self.command_records = []
        self.diagnostic_write_errors = []
        self.in_cleanup = False
        self.known_secrets.update(("dot06-signed-jwt-fixture", "dot06-synthetic-provider-key",
            "dot06-private-admin-key", "dot05-signed-principal-fixture",
            "0123456789abcdef0123456789abcdef"))

    def redact(self, value):
        return super().redact(value)

    def diagnostic_write(self, writer, path, value, *, strict=False):
        try:
            writer(path, value)
            return True
        except OSError as error:
            self.diagnostic_write_errors.append({"path": str(path), "type": type(error).__name__,
                                                 "reason": self.redact(error)})
            if strict and not self.in_cleanup:
                raise
            return False

    def run_command(self, argv, *, check=True, env=None, cwd=None, timeout=None, **kwargs):
        require(timeout is None and not kwargs, "undeclared command options")
        self.command_counter += 1
        stem = "command-%05d" % self.command_counter
        receipt_path = self.evidence / (stem + ".json")
        log_path = self.evidence / (stem + ".log")
        child_env = dict(self.child_env)
        if env:
            child_env.update(env)
        record = {"argv": [self.redact(item) for item in argv], "cwd": str(cwd or self.work),
                  "started_at": time.time(), "status": "running", "timeout_seconds": None,
                  "log": str(log_path)}
        self.command_records.append(record)
        write_error_start = len(self.diagnostic_write_errors)
        self.diagnostic_write(save, receipt_path, record, strict=True)
        process = None
        failure = None
        stdout = ""
        try:
            process = subprocess.Popen([str(item) for item in argv], cwd=cwd or self.work,
                env=child_env, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT, text=True, errors="replace", start_new_session=True)
            record["pid"] = process.pid
            # Once launched, diagnostic I/O must not abandon the owned process.
            self.diagnostic_write(save, receipt_path, record)
            try:
                stdout, _ = process.communicate()
            except BaseException:
                # Signal only this invocation's process group, then wait for actual exit.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                stdout, _ = process.communicate()
                raise
        except BaseException as error:
            failure = error
            record["failure"] = {"type": type(error).__name__, "reason": self.redact(error)}
        finally:
            retained = self.redact(stdout)
            written = self.diagnostic_write(private_write, log_path, retained)
            record.update(finished_at=time.time(), exit_code=process.returncode if process else None,
                          log_sha256=hashlib.sha256(retained.encode()).hexdigest() if written else None,
                          log_retained=written, log_bytes=len(retained.encode()),
                          status="failed" if failure or not process
                          or process.returncode else "passed")
            self.diagnostic_write(save, receipt_path, record)
        if failure:
            raise failure
        result = subprocess.CompletedProcess(argv, process.returncode, stdout, "")
        if check:
            require(result.returncode == 0, "command failed; see " + str(receipt_path))
        if not self.in_cleanup:
            require(len(self.diagnostic_write_errors) == write_error_start,
                    "command evidence write failed; no later gate may run")
        return result

    def docker(self, *args, **kwargs):
        return self.run_command([self.docker_binary, *args], **kwargs)

    def start_attached(self, name, recognize):
        require(name not in self.attachments, "duplicate attachment")
        owner = ScopedAttachment(self, name, recognize)
        self.attachments[name] = owner
        return owner

    def stop_container(self, name):
        # Preserve Resources' actual stop/wait/join boundary, independently of receipt I/O.
        self.docker("stop", "--timeout", "-1", name)
        status = self.docker("wait", name).stdout.strip()
        owner = self.attachments.pop(name, None)
        if owner is not None:
            # The inherited join raises for a diagnostic error after joining.
            # Keep the actual join and its diagnostic outcome separate so a
            # failed log cannot prevent removal of this terminated container.
            owner.reader.join()
        outcome = {"container": name, "exit_code": status,
                   "attachment_exit": owner.exit_code if owner is not None else None,
                   "attachment_joined": owner is not None,
                   "attachment_failure": owner.error if owner is not None else None,
                   "attachment_log": str(owner.path) if owner is not None else None,
                   "completion": "process_exited", "graceful": status == "0"}
        if owner is not None and owner.exit_code is None:
            outcome["attachment_failure"] = owner.error or "attachment_exit_unconfirmed"
        self.diagnostic_write(save, self.evidence / (name + "-exit-" + uuid.uuid4().hex + ".json"),
                              outcome)
        return outcome

    def reserve(self, kind, label):
        name = self.unique_name(label)
        self.pending.append((kind, name))
        save(self.evidence / "owned-resources.json", {
            "owner": self.owner, "attempted": self.pending, "cleanup": self.cleanup_results})
        return name

    def start(self):
        self.network = self.reserve("network", "network")
        self.docker("network", "create", "--label", self.label, self.network)
        self.network_created = True
        return self

    def env_file(self, name, values):
        for value in values.values():
            require("\n" not in value and "\r" not in value, "invalid environment file value")
        path = self.root / (name + ".env")
        private_write(path, "".join(key + "=" + value + "\n" for key, value in values.items()))
        return path

    def ensure_backend(self, kind):
        if kind in self.backends:
            return self.backends[kind]
        require(kind in ("surreal", "postgres"), "unsupported backend")
        password = secrets.token_hex(32)
        user = "root" if kind == "surreal" else "postgres"
        self.known_secrets.update((password, urllib.parse.quote(password, safe=""),
            base64.b64encode((user + ":" + password).encode()).decode()))
        volume = self.reserve("volume", kind + "-data")
        self.docker("volume", "create", "--label", self.label, volume)
        name = self.reserve("container", kind)
        common = ["create", "--name", name, "--label", self.label, "--platform", self.platform,
                  "--network", self.network, "--network-alias", kind]
        if kind == "surreal":
            credentials = self.env_file(kind, {"SURREAL_USER": user, "SURREAL_PASS": password})
            self.docker(*common, "--publish", "127.0.0.1::8000", "--user", "0",
                "--env-file", str(credentials), "--volume", volume + ":/data", SURREAL_IMAGE,
                "start", "--bind", "0.0.0.0:8000", "rocksdb:/data/phase.db")
            self.start_attached(name, surreal_listening).ready()
            # Scoped process environment has no proxy variables; auth stays in the HTTP header.
            address = self.docker("port", name, "8000/tcp").stdout.strip()
            require(re.fullmatch(r"127\.0\.0\.1:\d+", address), "Surreal loopback binding missing")
            host_url = self.authenticate_surreal(name, user, password)
            entry = {"name": name, "username": user, "password": password,
                     "host_url": host_url, "image": SURREAL_IMAGE, "durability": "owned-rocksdb-volume"}
        else:
            image = self.settings["postgres_image"]
            credentials = self.env_file(kind, {"POSTGRES_USER": "supabase_admin", "POSTGRES_PASSWORD": password})
            self.docker(*common, "--publish", "127.0.0.1::5432", "--env-file", str(credentials),
                "--volume", volume + ":/var/lib/postgresql/data", image)
            self.start_attached(name, postgres_final_server()).ready()
            address = self.authenticate_postgres(name, image, user, password)
            entry = {"name": name, "username": user, "password": password,
                     "host_address": address, "image": image}
        self.backends[kind] = entry
        actual_platform = self.docker("image", "inspect", "--format", "{{.Os}}/{{.Architecture}}", entry["image"]).stdout.strip()
        require(actual_platform == self.platform, "owned backend image platform differs")
        return entry

    def authenticate_postgres(self, name, image, user, password):
        address = self.docker("port", name, "5432/tcp").stdout.strip()
        require(re.fullmatch(r"127\.0\.0\.1:\d+", address), "PostgreSQL loopback binding missing")
        host, port = address.split(":")
        credentials = self.env_file("postgres-client", {"PGPASSWORD": password})
        client = self.reserve("container", "postgres-auth")
        # Match the existing helper's host-endpoint authentication, with explicit ownership,
        # platform and file-based credentials. The local desktop daemon must support host networking.
        sql = ("CREATE EXTENSION IF NOT EXISTS vector; SELECT current_setting('server_version_num')"
               " || '|' || extversion FROM pg_extension WHERE extname = 'vector';")
        self.docker("create", "--name", client, "--label", self.label, "--platform", self.platform,
            "--network", "host", "--env-file", str(credentials), "--entrypoint", "psql", image,
            "-h", host, "-p", port, "-U", user, "-d", "postgres", "-v", "ON_ERROR_STOP=1", "-At", "-c", sql)
        result = self.docker("start", "--attach", client)
        rows = re.findall(r"^170006\|([^\s]+)$", result.stdout, re.MULTILINE)
        require(len(rows) == 1, "owned PostgreSQL17.6/pgvector authentication unavailable")
        save(self.evidence / "postgres-authentication.json", {
            "server_version_num": 170006, "pgvector_version": rows[0], "host": address,
            "image": image, "platform": self.platform, "status": "passed"})
        return address

    def close(self):
        errors = []
        self.in_cleanup = True
        for kind, name in reversed(self.pending):
            item = {"kind": kind, "id": name, "status": "pending"}
            self.cleanup_results.append(item)
            self.diagnostic_write(save, self.evidence / "cleanup.json", self.cleanup_results)
            try:
                field = ".Config.Labels" if kind == "container" else ".Labels"
                inspect = self.docker(kind, "inspect", "--format",
                    '{{index ' + field + ' "uar.native-identity-backend-owner"}}', name, check=False)
                require(inspect.returncode == 0, "resource existence unresolved after attempted create: " + name)
                require(inspect.stdout.strip() == self.owner, "owned-resource label mismatch: " + name)
                if kind == "container":
                    try:
                        self.capture_container(name)
                    except BaseException as error:
                        item["capture_failure"] = {"type": type(error).__name__, "reason": self.redact(error)}
                        errors.append({"kind": kind, "id": name, "capture_failure": item["capture_failure"]})
                    # Existing stop/wait/join has no forced-kill escalation or finite deadline.
                    item["termination"] = self.stop_container(name)
                    if item["termination"]["attachment_failure"] is not None:
                        errors.append({"kind": kind, "id": name,
                                       "attachment_failure": item["termination"]["attachment_failure"]})
                    self.docker("rm", name)
                else:
                    self.docker(kind, "rm", name)
                item["status"] = "removed"
            except BaseException as error:
                item.update(status="unconfirmed", failure={"type": type(error).__name__,
                            "reason": self.redact(error)})
                errors.append(item)
            self.diagnostic_write(save, self.evidence / "cleanup.json", self.cleanup_results)
        require(not errors, "owned cleanup or attachment evidence incomplete; see cleanup.json")


def expected_stream_rows():
    return [(protocol, case) for protocol in STREAM_PROTOCOLS for case in STREAM_CASES
            if not (protocol == "Azure" and case not in ("valid", "missing_terminal"))
            and not (protocol == "Bedrock" and case in ("partial_event", "invalid_utf8", "malformed_after_terminal"))]


def artifact(path):
    require(Path(path).is_file() and not Path(path).is_symlink(), "required evidence missing: " + str(path))
    return {"path": str(path), "sha256": sha256(path)}


def verified_group(root, name, profile, case=None):
    path = root / name / "acceptance.json"
    data = load(path)
    require(data.get("verified") is True and data.get("profile") == profile,
            "unverified or mismatched group: " + name)
    if case is not None:
        require(data.get("case") == case, "group case mismatch: " + name)
    return {"name": name, "status": "passed", "evidence": artifact(path)}


def acceptance(target, text, environment):
    require(not re.search(r"\bSKIPPED\b", text, re.IGNORECASE), "target reports skipped coverage")
    summaries = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", text)
    expected_count = next(count for name, _, count in TARGETS if name == target)
    require(len(summaries) == 1 and tuple(map(int, summaries[0])) == (expected_count, 0, 0, 0, 0),
            "missing, filtered, ignored or non-parseable target summary")
    # --nocapture may place fixture output after 'test NAME ...' and before the final 'ok'.
    names = re.findall(r"(?m)^test ([A-Za-z0-9_:]+) \.\.\. ", text)
    expected_names = AGENT_TESTS if target == "agent_threads" else {SINGLE_TESTS[target]}
    require(len(names) == len(expected_names) and set(names) == expected_names,
            "complete exact test-name inventory missing")
    groups = []
    if target == "agent_threads":
        return {"tests": sorted(names), "count": 10, "skips": 0}
    if target == "fresh_process_resume":
        root = Path(environment["UAR_DOT06_EVIDENCE_ROOT"])
        for profile in ("local", "shared"):
            for case in RESUME_CASES:
                groups.append(verified_group(root, profile + "-" + case, profile, case))
        groups.append(verified_group(root, "shared-race", "shared", "race"))
        for name, profile in (("memory", "memory"), ("unattested", "shared"), ("postgres", "postgres")):
            groups.append(verified_group(root, name, profile))
        require(len(groups) == 24, "resume denominator differs")
    elif target == "approval_recovery":
        root = Path(environment["UAR_DOT07_EVIDENCE_ROOT"])
        for profile in ("local", "shared"):
            for case in APPROVAL_CASES:
                groups.append(verified_group(root, profile + "-" + case, profile, case))
        for profile in ("unattested", "postgres"):
            groups.append(verified_group(root, profile, profile, "unsupported"))
        for profile in ("local", "shared", "postgres", "memory"):
            groups.append(verified_group(root, profile + "-atomic-deadline", profile, "atomic-deadline"))
        for profile in ("local", "shared"):
            name = profile + "-live-identity"
            directory = root / name / "live-readback"
            result = load(directory / "result.json")
            joined = load(directory / "joined.json")
            require(joined.get("joined") is True and joined.get("exitCode") == 0,
                    "live identity readback was not joined")
            records, receipts = result.get("records"), result.get("receipts")
            require(isinstance(records, list) and len(records) == 1 and isinstance(receipts, list)
                    and len(receipts) == 1 and result.get("claims") == [], "live identity evidence incomplete")
            require(records[0].get("toolCallId") == "dot07-seed-0"
                    and records[0].get("state") == "approved"
                    and receipts[0].get("call_id") == "dot07-seed-0"
                    and records[0].get("rootRunId") == receipts[0].get("run_id")
                    and records[0].get("ownerKey") == receipts[0].get("principal_key"),
                    "live identity call binding absent")
            groups.append({"name": name, "status": "passed", "assertion_boundary": "full-target-success",
                           "evidence": [artifact(directory / "result.json"), artifact(directory / "joined.json")],
                           "limitation": "fixture emits readback, not a separate acceptance.json"})
        require(len(groups) == 34, "approval denominator differs")
    else:
        roots = re.findall(r"dot05 private fixture evidence: ([^\r\n]+)", text)
        require(len(roots) == 1, "stream fixture root missing or ambiguous")
        root = Path(roots[0]).resolve()
        require(root.parent == Path(environment["TMPDIR"]).resolve(), "stream evidence escaped owned TMPDIR")
        rows = expected_stream_rows()
        reported = re.findall(r"dot05 (\w+)/([\w_]+): legacy \+ local/shared graph accepted", text)
        require(reported == rows and len(rows) == 83
                and text.count("dot05 controlled scenario matrix: 83 protocol/scenario rows") == 1,
                "stream83-row inventory missing or differs")
        for index, (protocol, case) in enumerate(rows, 1):
            for route in ("legacy", "local", "shared"):
                path = root / ("case-" + str(index)) / route / "boundary-diagnostics.json"
                data = load(path)
                require(data.get("protocol") == protocol and data.get("case") == case
                        and data.get("route") == ("legacy_orchestrator" if route == "legacy" else "graph"),
                        "stream diagnostics identity differs")
                groups.append({"name": protocol + "/" + case + "/" + route,
                               "status": "passed", "assertion_boundary": "full-target-success", "evidence": artifact(path)})
        require(len(groups) == 249, "stream path denominator differs")
    return {"groups": groups, "count": len(groups), "coverage": "complete-source-defined-inventory"}


class SourceFreeze:
    """Bind the lead's immutable exact-path inventory and the complete Git index."""
    def __init__(self, settings, resources):
        self.settings, self.resources = settings, resources
        self.path = Path(settings["source_manifest"])
        self.expected = settings["source_manifest_sha256"]
        self.settings_path = settings["_settings_path"]
        self.settings_digest = sha256(self.settings_path)
        self.initial = None

    def git(self, repository, *args):
        return self.resources.run_command(
            [self.settings["git_binary"], "-C", str(repository), *args]).stdout

    def capture(self, label):
        require(sha256(self.settings_path) == self.settings_digest, "private settings changed")
        require(sha256(self.path) == self.expected, "source manifest changed")
        cargo_input = cargo_configuration_input(self.settings)
        manifest = load(self.path)
        require(manifest.get("schemaVersion") == 1 and manifest.get("sourceComplete") is True
                and manifest.get("noActiveWriters") is True and manifest.get("root") == str(ROOT),
                "source inventory is not complete and frozen for this checkout")
        require(manifest.get("stagedGitTree") == self.settings["staged_git_tree"], "inventory tree differs")
        require(self.git(ROOT, "rev-parse", "HEAD").strip() == manifest["baselineCommit"], "baseline commit changed")
        tree = self.git(ROOT, "write-tree").strip()
        require(tree == manifest["stagedGitTree"], "complete staged tree changed")
        entries = manifest.get("files", []) + manifest.get("controlFiles", [])
        require(entries, "empty source inventory")
        bound = {}
        for item in entries:
            relative = Path(item["path"])
            require(not relative.is_absolute() and ".." not in relative.parts and str(relative) not in ("", "."),
                    "unsafe source inventory path")
            path = ROOT / relative
            require(path.resolve().is_relative_to(ROOT) and not path.is_symlink() and path.is_file(),
                    "source inventory requires contained regular files")
            info = path.stat()
            actual = {"sha256": sha256(path), "bytes": info.st_size,
                      "mode": format(stat.S_IMODE(info.st_mode), "04o")}
            require(actual["sha256"] == item["sha256"] and actual["bytes"] == item["bytes"],
                    "source bytes changed: " + str(relative))
            if "mode" in item:
                require(actual["mode"] == item["mode"], "source mode changed: " + str(relative))
            require(str(relative) not in bound or bound[str(relative)] == actual, "conflicting inventory entries")
            bound[str(relative)] = actual
        require({".cargo/config.toml", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                 "scripts/release/native_identity_backend_gate.py", "scripts/release/uar_1_1_1_support.py",
                 "tests/agent_threads.rs", "tests/fresh_process_resume.rs", "tests/stream_terminal_contract.rs",
                 "tests/approval_recovery.rs", "tests/support/fresh_process_resume_process.rs",
                 "tests/support/approval_recovery_process.rs"}.issubset(bound), "required controls absent from inventory")
        metadata = set(manifest.get("allowedUnstagedMetadataPaths", []))
        require(metadata.issubset(METADATA_PATHS), "untyped metadata exclusion refused")
        staged = set(filter(None, self.git(ROOT, "diff", "--cached", "--name-only", "-z",
                                          manifest["baselineCommit"]).split("\0")))
        require(staged.issubset(set(bound) | metadata), "unexpected staged path outside inventory")
        unstaged = set(filter(None, self.git(ROOT, "diff", "--name-only", "-z").split("\0")))
        require(unstaged.issubset(metadata), "unexpected unstaged tracked source")
        untracked = set(filter(None, self.git(ROOT, "ls-files", "--others", "--exclude-standard", "-z").split("\0")))
        untracked_roots = manifest.get("allowedUntrackedMetadataRoots", [])
        require(isinstance(untracked_roots, list) and len(untracked_roots) == len(UNTRACKED_METADATA_ROOTS)
                and set(untracked_roots) == UNTRACKED_METADATA_ROOTS,
                "untracked metadata roots differ from the fixed preparation scope")
        untracked_metadata = {name for name in untracked
                              if any(name.startswith(prefix) for prefix in UNTRACKED_METADATA_ROOTS)}
        require(untracked.issubset(set(bound) | metadata | untracked_metadata),
                "unexpected untracked source outside inventory")
        dependencies = manifest.get("pinnedDependencySources", [])
        require(dependencies, "dependency identity inventory absent")
        observed_dependencies = []
        for item in dependencies:
            relative = Path(item["path"])
            require(not relative.is_absolute() and ".." not in relative.parts, "unsafe dependency path")
            path = ROOT / relative
            require(path.resolve().is_relative_to(ROOT), "dependency escaped source checkout")
            commit = self.git(path, "rev-parse", "HEAD").strip()
            require(commit == item["commit"], "dependency commit changed: " + str(relative))
            require(not self.git(path, "status", "--porcelain", "--untracked-files=all").strip(),
                    "dependency has dirty or untracked source: " + str(relative))
            observed_dependencies.append({"path": str(relative), "commit": commit})
        current_modules = self.git(ROOT, "submodule", "status", "--recursive").splitlines()
        parsed = []
        for line in current_modules:
            match = re.fullmatch(r" ([0-9a-f]{40}) ([^\s]+)(?: \(.*\))?", line)
            require(match is not None, "uninitialized or mismatched recursive dependency")
            parsed.append({"path": match.group(2), "commit": match.group(1)})
        require(sorted(parsed, key=lambda row: row["path"]) == sorted(observed_dependencies, key=lambda row: row["path"]),
                "recursive dependency denominator differs")
        # Selected test/readback entrypoints do not discover .env. Runtime children
        # use explicit --config; main only honors --env-file/UAR_ENV_FILE, neither
        # of which is supplied. Source ancestry is not a configuration input.
        no_dotenv(self.resources.work)
        value = {"staged_git_tree": tree, "source_manifest_sha256": self.expected,
                 "files": bound, "pinned_dependency_sources": observed_dependencies,
                 "external_build_inputs": [cargo_input]}
        if self.initial is not None:
            require(value == self.initial, "frozen source identity drifted")
        else:
            self.initial = value
        save(self.resources.evidence / ("source-" + label + ".json"), value)
        return value


# Only typed KBD projections may change without silently admitting source edits.
METADATA_PATHS = {
    ".kbd-orchestrator/current-waypoint.json",
    ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/progress.json",
    ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/tasks.md",
    ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/progress.json",
    ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/tasks.md",
    ".kbd-orchestrator/position-reminder.txt",
    ".kbd-orchestrator/position.json",
}
UNTRACKED_METADATA_ROOTS = {
    ".agent-team/uar-core/handoffs/kubo-native-identity/",
    ".agent-team/uar-core/reviews/kubo-native-identity-r7/",
    ".kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/children/kubo-native-tool-identity-pr13/",
}


def after_gate_freeze(freeze, label, record):
    try:
        freeze.capture(label)
    except BaseException as error:
        record["source_freeze_failure"] = {"type": type(error).__name__,
                                          "reason": freeze.resources.redact(error)}
        if "failure" not in record:
            record["status"] = "blocked"
            raise
        # The in-flight original exception stays primary; the drift is retained too.


def cargo_configuration_input(settings):
    """Adopt the existing cache configuration without exposing its contents."""
    home = Path(settings["cargo_home"])
    legacy = home / "config"
    require(not legacy.exists() and not legacy.is_symlink(),
            "unbound legacy Cargo configuration would override the selected config.toml")
    path = home / "config.toml"
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    require(digest == settings["cargo_config_sha256"], "selected Cargo configuration bytes changed")

    def parse(content):
        try:
            return tomllib.loads(content.decode("utf-8"))
        except (UnicodeError, tomllib.TOMLDecodeError) as error:
            raise Blocked("Cargo configuration is not parseable TOML") from error

    def has_runner(value):
        if isinstance(value, dict):
            return "runner" in value or any(has_runner(child) for child in value.values())
        if isinstance(value, list):
            return any(has_runner(child) for child in value)
        return False

    config = parse(raw)
    require(not has_runner(config), "Cargo configuration injects a test runner")
    environment = config.get("env", {})
    require(isinstance(environment, dict)
            and set(environment).issubset({"SCCACHE_DIR", "SCCACHE_CACHE_SIZE"}),
            "global Cargo environment contains an unapproved key")
    require(all(isinstance(value, str) and "\n" not in value and "\r" not in value
                for value in environment.values()),
            "Cargo cache environment requires single-line strings, not value tables")
    for name in ("config", "config.toml"):
        repository_config = ROOT / ".cargo" / name
        if repository_config.is_file():
            repository = parse(repository_config.read_bytes())
            require("env" not in repository and not has_runner(repository),
                    "repository Cargo environment or runner injection is forbidden")
    return {"path": str(path), "bytes": len(raw), "sha256": digest,
            "kind": "existing-cargo-cache-configuration"}


def configure(path):
    path = private_input(path)
    settings = load(path)
    keys = {"schema_version", "source_manifest", "source_manifest_sha256", "staged_git_tree",
            "output_dir", "resource_dir", "cargo_binary", "rustc_binary", "git_binary", "docker_binary",
            "cargo_home", "cargo_config_sha256", "rustup_home", "target_dir", "path", "docker_host", "docker_config", "platform",
            "postgres_image", "surreal_image"}
    require(isinstance(settings, dict) and set(settings) == keys, "private settings keys differ from template")
    require(settings["schema_version"] == 1, "private settings version differs")
    for key in keys - {"schema_version"}:
        require(isinstance(settings[key], str) and settings[key] and "\n" not in settings[key]
                and "\r" not in settings[key], "required private setting missing or invalid: " + key)
    require(re.fullmatch(r"[0-9a-f]{64}", settings["source_manifest_sha256"]), "real source manifest SHA256 required")
    require(re.fullmatch(r"[0-9a-f]{64}", settings["cargo_config_sha256"]), "real Cargo configuration SHA256 required")
    require(re.fullmatch(r"[0-9a-f]{40}", settings["staged_git_tree"]), "real staged source tree required")
    require(settings["platform"] == "linux/arm64", "this prepared gate selects linux/arm64")
    require(settings["surreal_image"] == SURREAL_IMAGE, "Surreal image must match authoritative3.3.0 pin")
    require(settings["postgres_image"] == POSTGRES_IMAGE,
            "PostgreSQL image must match the selected immutable Supabase17.6 registry reference")
    for key in ("source_manifest", "output_dir", "resource_dir", "cargo_binary", "rustc_binary", "git_binary",
                "docker_binary", "cargo_home", "rustup_home", "target_dir", "docker_config"):
        require(Path(settings[key]).is_absolute(), "absolute path required: " + key)
    require(Path(settings["target_dir"]) == ROOT / "target", "preserve this checkout's existing target path")
    require(settings["docker_host"].startswith("unix:///") and "@" not in settings["docker_host"],
            "owned backend gate requires explicitly selected local Unix Docker socket")
    require(all(part.startswith("/") for part in settings["path"].split(":")), "PATH must contain absolute entries only")
    for key in ("output_dir", "resource_dir"):
        value = Path(settings[key])
        require(not value.exists() and not value.is_symlink(), "output/resource path must be fresh: " + key)
        require(value.parent.is_dir() and not value.resolve().is_relative_to(ROOT), "fresh directories require existing parent outside source")
        no_dotenv(value.parent)
    output, resource = Path(settings["output_dir"]).resolve(), Path(settings["resource_dir"]).resolve()
    require(output != resource and not output.is_relative_to(resource) and not resource.is_relative_to(output),
            "output and resource directories must be disjoint")
    require(not path.resolve().is_relative_to(ROOT), "private settings must stay outside source")
    require(not any(re.search(r"(?:^|_)(?:JOB|CHILD)(?:_|$)", name) for name in os.environ),
            "inherited fixture JOB/CHILD mode is forbidden")
    # Only the existing, byte-bound global cache inputs may extend the clean env.
    cargo_configuration_input(settings)
    settings["_settings_path"] = str(path)
    return settings


def scoped_environment(settings, resource_root):
    home = private_dir(resource_root / "home")
    temporary = private_dir(resource_root / "tmp")
    return {"PATH": settings["path"], "LANG": "en_US.UTF-8", "LC_ALL": "en_US.UTF-8",
            "HOME": str(home), "TMPDIR": str(temporary), "XDG_CONFIG_HOME": str(home / "config"),
            "XDG_DATA_HOME": str(home / "data"), "XDG_CACHE_HOME": str(home / "cache"),
            "CARGO_HOME": settings["cargo_home"], "RUSTUP_HOME": settings["rustup_home"],
            "RUSTUP_TOOLCHAIN": "nightly-2026-07-18", "RUSTC": settings["rustc_binary"],
            "CARGO_TARGET_DIR": settings["target_dir"], "CARGO_TERM_COLOR": "never",
            "DOCKER_HOST": settings["docker_host"], "DOCKER_CONFIG": settings["docker_config"],
            "PYTHONDONTWRITEBYTECODE": "1"}


def gate_command(settings, target=None, features="server-full,postgres-backend"):
    command = [settings["cargo_binary"], "test" if target else "build", "--locked",
               "--manifest-path", str(ROOT / "Cargo.toml"), "--config", str(ROOT / ".cargo/config.toml"),
               "--no-default-features", "--features", features]
    if target:
        command += ["--test", target, "--", "--nocapture", "--test-threads=1"]
    else:
        command += ["--bin", "universal-agent-runtime"]
    return command


def retain_fixture_inventory(resources, directory):
    """Retain fixture bytes privately; log files receive the same secret redaction."""
    entries = []
    for path in sorted(Path(directory).rglob("*")):
        if path.is_file() and not path.is_symlink():
            if path.suffix in (".log", ".stdout", ".stderr"):
                private_write(path, resources.redact(path.read_text(errors="replace")))
            entries.append({"path": str(path), "bytes": path.stat().st_size, "sha256": sha256(path)})
    return entries


def execute(settings):
    import fcntl
    os.umask(0o077)
    output = private_dir(settings["output_dir"])
    receipt = {"schema_version": 1, "status": "preparing", "started_at": time.time(), "gates": [],
               "source_manifest": settings["source_manifest"], "source_manifest_sha256": settings["source_manifest_sha256"],
               "staged_git_tree": settings["staged_git_tree"], "platform": settings["platform"],
               "surreal_image": settings["surreal_image"], "postgres_image": settings["postgres_image"],
               "scope": "four backend targets and exact customer build only", "cleanup": "not-started",
               "limits": ["No finite readiness, command or graceful cleanup deadline.",
                          "Cargo lock and advisory supervisor lock do not exclude unrelated noncooperating writers.",
                          "Fixture cleanup remains owned by fixtures; supervisor does not discover or signal unrelated PIDs.",
                          "Private fixture configs and traces are retained; do not publish the resource directory.",
                          "A full or detached evidence volume can prevent final diagnostics; cleanup attempts remain independent and acceptance is blocked.",
                          "This gate does not certify packaging, genuine inference, publication, consumer pins or PR readiness."]}
    save(output / "receipt.json", receipt)
    resources = freeze = lock = None
    original_failure = cleanup_failure = None
    receipt_write_errors = []

    def retain_receipt(*, strict=False):
        # Cleanup cannot depend on a working evidence volume. Keep failures in
        # memory so the next ordinary receipt write can retain them if possible.
        try:
            save(output / "receipt.json", receipt)
        except OSError as error:
            receipt_write_errors.append({"path": str(output / "receipt.json"),
                "type": type(error).__name__, "reason": resources.redact(error) if resources else str(error)})
            if strict:
                raise

    try:
        resource_root = private_dir(settings["resource_dir"])
        work = private_dir(resource_root / "work")
        environment = scoped_environment(settings, resource_root)
        # Also scope imports/helpers whose internals inherit os.environ.
        os.environ.clear()
        os.environ.update(environment)
        resources = BackendResources(settings, output, resource_root, environment, work)
        target = Path(settings["target_dir"])
        require(target.is_dir(), "existing owned Cargo target cache required")
        lock = (target / ".native-identity-backend-gate.lock").open("a")
        os.chmod(lock.name, 0o600)
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        freeze = SourceFreeze(settings, resources)
        freeze.capture("before-setup")
        resources.start()
        surreal = resources.ensure_backend("surreal")
        postgres = resources.ensure_backend("postgres")
        database_url = "postgres://postgres:" + urllib.parse.quote(postgres["password"], safe="") + "@" + postgres["host_address"] + "/postgres"
        resources.known_secrets.add(database_url)
        infra = resource_root / "infrastructure.json"
        save(infra, {"surreal": {"url": surreal["host_url"], "username": surreal["username"], "password": surreal["password"]},
                     "postgres": {"url": database_url}})
        environment.update(DATABASE_URL=database_url, UAR_DOT06_INFRA_FILE=str(infra),
            UAR_DOT05_INFRA_FILE=str(infra), UAR_DOT06_NAMESPACE="dot06_" + uuid.uuid4().hex,
            UAR_DOT05_NAMESPACE="dot05_" + uuid.uuid4().hex,
            UAR_DOT06_EVIDENCE_ROOT=str(private_dir(resource_root / "dot06-evidence")),
            UAR_DOT07_EVIDENCE_ROOT=str(private_dir(resource_root / "dot07-evidence")),
            VERTEXAI_PROJECT="dot05-fixture", VERTEXAI_LOCATION="us-central1")
        require(surreal["durability"] == "owned-rocksdb-volume", "durable owned Surreal setup missing")
        environment["UAR_REMOTE_SURREAL_DURABILITY_ATTESTED"] = "1"
        freeze.capture("after-setup")
        receipt.update(status="running", environment_keys=sorted(environment),
                       private_infrastructure=str(infra), resource_root=str(resource_root),
                       command_wait_policy="no outer timeout; existing fixture deadlines unchanged")
        retain_receipt(strict=True)
        for target_name, features, _ in TARGETS:
            freeze.capture("before-" + target_name)
            command = gate_command(settings, target_name, features)
            record = {"name": target_name, "argv": command, "status": "running"}
            receipt["gates"].append(record)
            retain_receipt(strict=True)
            try:
                result = resources.run_command(command, env=environment, check=False)
                record.update(exit_code=result.returncode, command_receipt="command-%05d.json" % resources.command_counter)
                require(result.returncode == 0, target_name + " failed")
                record["coverage"] = acceptance(target_name, result.stdout, environment)
                record["status"] = "passed"
            except BaseException as error:
                record.update(status="blocked" if isinstance(error, Blocked) else "failed",
                              failure={"type": type(error).__name__, "reason": resources.redact(error)})
                raise
            finally:
                try:
                    after_gate_freeze(freeze, "after-" + target_name, record)
                finally:
                    retain_receipt(strict=True)
        freeze.capture("before-customer-build")
        command = gate_command(settings)
        record = {"name": "customer-build", "argv": command, "status": "running"}
        receipt["gates"].append(record)
        retain_receipt(strict=True)
        try:
            result = resources.run_command(command, env=environment, check=False)
            record.update(exit_code=result.returncode, command_receipt="command-%05d.json" % resources.command_counter)
            require(result.returncode == 0, "exact customer profile build failed")
            binary = Path(settings["target_dir"]) / "debug/universal-agent-runtime"
            require(binary.is_file(), "customer binary missing")
            before = sha256(binary)
            destination = output / "universal-agent-runtime"
            with binary.open("rb") as source, destination.open("xb") as copy:
                os.chmod(destination, 0o600)
                for chunk in iter(lambda: source.read(1024 * 1024), b""):
                    copy.write(chunk)
            require(sha256(destination) == before == sha256(binary), "customer artifact changed during copy")
            record.update(status="passed", artifact=artifact(destination), artifact_mode="0600",
                          profile="debug;server-full,postgres-backend;no-default-features")
        except BaseException as error:
            record.update(status="failed", failure={"type": type(error).__name__, "reason": resources.redact(error)})
            raise
        finally:
            try:
                after_gate_freeze(freeze, "after-customer-build", record)
            finally:
                retain_receipt(strict=True)
    except BaseException as error:
        caught = {"type": type(error).__name__, "reason": resources.redact(error) if resources else str(error)}
        gate_failure = receipt["gates"][-1].get("failure") if receipt["gates"] else None
        original_failure = gate_failure or caught
        if original_failure != caught:
            receipt["supervisor_failure"] = caught
        receipt["failure"] = original_failure
    finally:
        receipt["cleanup"] = "running" if resources else "not-created"
        receipt["receipt_write_failures"] = receipt_write_errors
        retain_receipt()
        if resources:
            try:
                resources.close()
                receipt["cleanup"] = "complete"
            except BaseException as error:
                cleanup_failure = {"type": type(error).__name__, "reason": resources.redact(error)}
                receipt.update(cleanup="unconfirmed", cleanup_failure=cleanup_failure)
            try:
                if freeze and freeze.initial is not None:
                    freeze.capture("after-cleanup")
            except BaseException as error:
                receipt["final_freeze_failure"] = {"type": type(error).__name__, "reason": resources.redact(error)}
            try:
                inventory = retain_fixture_inventory(resources, resources.root)
                inventory_path = output / "fixture-inventory.json"
                save(inventory_path, inventory)
                receipt["retained_private_files"] = {
                    **artifact(inventory_path), "bytes": inventory_path.stat().st_size,
                    "count": len(inventory),
                }
            except BaseException as error:
                receipt["retention_failure"] = {"type": type(error).__name__, "reason": resources.redact(error)}
            receipt["owned_resource_cleanup"] = resources.cleanup_results
            receipt["diagnostic_write_failures"] = resources.diagnostic_write_errors
        if lock:
            try:
                lock.close()
            except OSError as error:
                receipt["lock_close_failure"] = {"type": type(error).__name__,
                    "reason": resources.redact(error) if resources else str(error)}
        failed = (original_failure or cleanup_failure or receipt_write_errors
                  or receipt.get("diagnostic_write_failures") or receipt.get("final_freeze_failure")
                  or receipt.get("retention_failure") or receipt.get("lock_close_failure"))
        receipt.update(status="blocked" if failed else "passed", finished_at=time.time())
        retain_receipt()
        if receipt_write_errors:
            # A failure of this last write may only be reportable to the caller.
            # Do not retry or convert missing evidence into a passing outcome.
            receipt["status"] = "blocked"
    print("Backend gate " + receipt["status"] + "; private receipt path"
          + (" (may be incomplete)" if receipt_write_errors else "") + ": " + str(output / "receipt.json"))
    return 2 if receipt["status"] != "passed" else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--settings", required=True, help="absolute private0600 settings JSON")
    args = parser.parse_args()
    try:
        settings = configure(args.settings)
        return execute(settings)
    except (Blocked, OSError, ValueError) as error:
        # Input validation occurs before credentials or resources are generated.
        print("Backend gate blocked: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
