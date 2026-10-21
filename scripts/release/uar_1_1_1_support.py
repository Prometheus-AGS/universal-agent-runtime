"""Owned real-process collaborators for the UAR 1.1.1 phase integration program.

Importing this module performs no I/O. Only the final phase runner invokes it.
"""
import base64
import codecs
import hashlib
import hmac
import http.client
import ipaddress
import json
import math
import os
from pathlib import Path
import re
import secrets
import signal
import socket
import subprocess
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
SURREAL_IMAGE = "surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20"
GROUPS = ("binding", "streams", "mcp-outcomes", "credentials", "recovery", "combined")


class Blocked(RuntimeError):
    """A declared observation cannot be made with the supplied collaborators."""


class StreamDeadlineExceeded(TimeoutError):
    """The whole observer read expired, including an otherwise active stream."""

    def __init__(self, partial):
        super().__init__("observer stream total deadline exceeded (%s bytes retained)" % len(partial))
        self.partial = partial


def require(value, message):
    if not value:
        raise AssertionError(message)


class Http:
    def __init__(self, base_url, token=None, admin_key=None):
        self.base_url = base_url.rstrip("/")
        self.token = token
        self.admin_key = admin_key

    def request(self, method, path, body=None, expected=200, headers=None,
                return_status=False, timeout=180):
        values = {"Accept": "application/json"}
        if self.token:
            values["Authorization"] = "Bearer " + self.token
        if self.admin_key:
            values["X-UAR-Admin-Key"] = self.admin_key
        values.update(headers or {})
        if body is not None and not isinstance(body, bytes):
            body = json.dumps(body, ensure_ascii=False).encode()
            values.setdefault("Content-Type", "application/json")
        request = urllib.request.Request(self.base_url + path, data=body,
                                         headers=values, method=method)
        accepted = expected if isinstance(expected, (tuple, list, set)) else (expected,)
        try:
            response = urllib.request.urlopen(request, timeout=timeout)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            status = response.code
            if status not in accepted:
                # Bounded, redacted failure context: surface only the error
                # code (never credential-bearing bodies) so failures remain
                # diagnosable (round-6 fixture improvement).
                code = self._error_code(raw)
                require(False, "%s %s status %s code=%s; expected %s"
                        % (method, path, status, code, accepted))
            try:
                result = json.loads(raw) if raw else None
            except (ValueError, UnicodeDecodeError):
                result = raw.decode("utf-8", errors="replace")
            return (status, result) if return_status else result

    @staticmethod
    def _error_code(raw):
        try:
            parsed = json.loads(raw[:4096])
        except (ValueError, UnicodeDecodeError):
            return "<unparseable>"
        if not isinstance(parsed, dict):
            return "<non-object>"
        field = parsed.get("error")
        if isinstance(field, str) and re.fullmatch(r"[a-z][a-z0-9_]{0,95}", field):
            return field
        if isinstance(field, dict) and isinstance(field.get("code"), str):
            return field["code"][:96]
        return "<absent>" if field is None else "<no-code>"

    @staticmethod
    def sanitized_error(response):
        if not isinstance(response, dict):
            return {"error": "unparseable_error_body"}
        error = response.get("error")
        code = error.get("code") if isinstance(error, dict) else error
        result = {"error": code if isinstance(code, str) and re.fullmatch(r"[a-z][a-z0-9_]{0,95}", code)
                  else "unclassified_error"}
        operation = response.get("operation_id")
        if isinstance(operation, str) and re.fullmatch(r"[a-fA-F0-9-]{16,128}", operation):
            result["operation_id"] = operation
        return result

    def subscribe(self, path):
        """Open an owned SSE observer before reading the authoritative snapshot."""
        return SseSubscription(self, path)

    def stream(self, path, timeout=180, *, total_timeout=240, on_text=None):
        headers = {"Accept": "text/event-stream"}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        require(math.isfinite(total_timeout) and total_timeout > 0,
                "observer stream requires a finite positive total deadline")
        deadline = time.monotonic() + total_timeout
        url = urllib.parse.urlsplit(self.base_url + path)
        require(url.scheme in ("http", "https") and url.hostname,
                "observer stream requires an HTTP endpoint")
        connection_type = http.client.HTTPSConnection if url.scheme == "https" else http.client.HTTPConnection
        connection = connection_type(url.hostname, url.port, timeout=min(timeout, total_timeout))
        expired = threading.Event()
        chunks = []
        decoded = []
        decoder = codecs.getincrementaldecoder("utf-8")("strict")
        timer = None
        try:
            connection.connect()
            transport = connection.sock
            require(transport is not None, "observer stream connection has no socket")

            def expire():
                expired.set()
                try:
                    transport.shutdown(socket.SHUT_RDWR)
                except OSError:
                    pass

            # Heartbeats keep an inactivity timeout alive. Shutdown also interrupts
            # response-header and HTTP chunk-framing reads at the total deadline.
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise StreamDeadlineExceeded(b"")
            timer = threading.Timer(remaining, expire)
            timer.daemon = True
            timer.start()
            target = urllib.parse.urlunsplit(("", "", url.path or "/", url.query, ""))
            connection.request("GET", target, headers=headers)
            with connection.getresponse() as response:
                require(response.status == 200, "observer stream status %s; expected200" % response.status)
                while True:
                    if expired.is_set() or time.monotonic() >= deadline:
                        raise StreamDeadlineExceeded(b"".join(chunks))
                    chunk = response.read1(65536)
                    if not chunk:
                        break
                    chunks.append(chunk)
                    text = decoder.decode(chunk, final=False)
                    if text:
                        decoded.append(text)
                        if on_text is not None:
                            on_text(text)
            if expired.is_set() or time.monotonic() >= deadline:
                raise StreamDeadlineExceeded(b"".join(chunks))
            text = decoder.decode(b"", final=True)
            if text:
                decoded.append(text)
                if on_text is not None:
                    on_text(text)
            return "".join(decoded)
        except Exception as error:
            partial = b"".join(chunks)
            if isinstance(error, http.client.IncompleteRead):
                partial += error.partial
                error.partial = partial
            if on_text is not None:
                error.partial = partial
            if expired.is_set() and not isinstance(error, StreamDeadlineExceeded):
                raise StreamDeadlineExceeded(partial) from error
            raise
        finally:
            if timer is not None:
                timer.cancel()
                timer.join()
            connection.close()


class SseSubscription:
    """A blocking event reader with explicit close and no observation polling.

    The server's initial event establishes its subscribe-before-snapshot
    contract. Unexpected EOF is unavailable, never a reason to probe/retry.
    """
    def __init__(self, client, path):
        url = urllib.parse.urlsplit(client.base_url + path)
        require(url.scheme in ("http", "https") and url.hostname, "SSE endpoint must be HTTP")
        connection_type = http.client.HTTPSConnection if url.scheme == "https" else http.client.HTTPConnection
        self.connection = connection_type(url.hostname, url.port, timeout=None)
        self.response = None
        headers = {"Accept": "text/event-stream"}
        if client.token:
            headers["Authorization"] = "Bearer " + client.token
        if client.admin_key:
            headers["X-UAR-Admin-Key"] = client.admin_key
        target = urllib.parse.urlunsplit(("", "", url.path or "/", url.query, ""))
        try:
            self.connection.request("GET", target, headers=headers)
            self.response = self.connection.getresponse()
            require(self.response.status == 200, "SSE subscription status %s; expected 200" % self.response.status)
            require(self.response.getheader("Content-Type", "").startswith("text/event-stream"),
                    "SSE subscription returned a different content type")
        except BaseException:
            self.close()
            raise

    def next(self):
        event, identity, data = "message", None, []
        while True:
            raw = self.response.readline()
            if not raw:
                raise Blocked("SSE subscription ended without a terminal acknowledgement")
            line = raw.decode("utf-8").rstrip("\r\n")
            if not line:
                if data:
                    return {"event": event, "id": identity, "data": json.loads("\n".join(data))}
                event, identity = "message", None
                continue
            if line.startswith(":"):
                continue
            field, separator, value = line.partition(":")
            if separator and value.startswith(" "):
                value = value[1:]
            if field == "event":
                event = value
            elif field == "id":
                identity = value
            elif field == "data":
                data.append(value)

    def close(self):
        if self.response is not None:
            self.response.close()
            self.response = None
        self.connection.close()

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        self.close()


class AttachedContainer:
    """Own start/attach before execution and retain the actual exit acknowledgement.

    Docker's start --attach establishes output attachment before ContainerStart.
    Each owner consumes only this process generation; historical logs are never
    readiness input. A missing event remains an outstanding obligation.
    """
    def __init__(self, resources, name, recognize):
        self.resources, self.name = resources, name
        self.condition = threading.Condition()
        self.receipt = None
        self.error = None
        self.exit_code = None
        self.path = resources.evidence / (name + "-attach-" + uuid.uuid4().hex + ".log")
        child_env = os.environ.copy()
        child_env.pop("DOCKER_DEFAULT_PLATFORM", None)
        self.process = subprocess.Popen(["docker", "start", "--attach", name],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            text=True, errors="replace", env=child_env, start_new_session=True)

        def consume():
            try:
                with self.path.open("w") as output:
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

    def ready(self):
        with self.condition:
            self.condition.wait_for(lambda: self.receipt is not None
                                    or self.error is not None or self.exit_code is not None)
            if self.error is not None or self.exit_code is not None:
                raise Blocked("container exited or attachment failed before readiness: "
                              + self.name + "; see " + self.path.name)
            return self.receipt

    def join(self):
        self.reader.join()
        require(self.error is None, "container attachment failed: " + self.name)
        return self.exit_code


def surreal_listening(line):
    # Pinned-image startup evidence: function-first-local-r3/command-00488.log.
    return {"listener": "0.0.0.0:8000"} if "Started web server on 0.0.0.0:8000" in line else None


def postgres_final_server(*, bootstrap_required=True):
    # A restart of an already authenticated owned volume does not rerun initdb.
    bootstrap_complete = not bootstrap_required

    def recognize(line):
        nonlocal bootstrap_complete
        if "PostgreSQL init process complete; ready for start up." in line:
            bootstrap_complete = True
        # The bootstrap postmaster has a different PID. Both facts are required.
        if bootstrap_complete and re.search(r"\[1\].*database system is ready to accept connections", line):
            return {"bootstrap_complete": True, "postmaster_pid": 1,
                    "bootstrap_required": bootstrap_required}
        return None
    return recognize


class ProcessEvents:
    """Owned peer output pump: explicit events or actual exit, never probes."""
    def __init__(self, resources, process, path):
        self.process = process
        self.condition = threading.Condition()
        self.events = []
        self.exit_code = None
        self.error = None

        def consume():
            try:
                with path.open("a") as output:
                    for raw in process.stdout:
                        line = raw.decode("utf-8", errors="replace").rstrip("\r\n")
                        output.write(resources.redact(line) + "\n")
                        output.flush()
                        if line.startswith("UAR_FIXTURE_EVENT:"):
                            event = json.loads(line[len("UAR_FIXTURE_EVENT:"):])
                            require(isinstance(event, dict) and event.get("version") == 1
                                    and isinstance(event.get("event"), str), "invalid fixture event")
                            with self.condition:
                                self.events.append(event)
                                self.condition.notify_all()
            except BaseException as error:
                with self.condition:
                    self.error = type(error).__name__
                    self.condition.notify_all()
            finally:
                process.stdout.close()
                code = process.wait()
                with self.condition:
                    self.exit_code = code
                    self.condition.notify_all()

        self.reader = threading.Thread(target=consume, name="peer-events-" + str(process.pid), daemon=False)
        self.reader.start()

    def wait(self, name, predicate=None):
        matches = lambda event: event["event"] == name and (predicate is None or predicate(event))
        with self.condition:
            self.condition.wait_for(lambda: any(matches(event) for event in self.events)
                                    or self.exit_code is not None or self.error is not None)
            require(self.error is None and self.exit_code is None,
                    "fixture exited or output failed before " + name)
            index = next(index for index, event in enumerate(self.events) if matches(event))
            return self.events.pop(index)

    def send(self, command):
        self.process.stdin.write((json.dumps({"version": 1, "command": command}) + "\n").encode())
        self.process.stdin.flush()

    def join(self):
        self.reader.join()
        require(self.error is None, "fixture event reader failed: " + str(self.error))
        return self.exit_code


class RuntimeLease:
    def __init__(self, resources, label, config, env=None, mounts=None):
        self.resources = resources
        self.name = resources.unique_name(label)
        self.config = config
        self.config["skill_source_roots"] = []
        self.startup_generations = set()
        self.env = dict(env or {})
        self.mounts = list(mounts or [])
        self.base_url = None
        self.container = None
        self.volume = resources.unique_name(label + "-data")
        self.config_dir = resources.root / (self.name + "-config")
        self.config_dir.mkdir()
        self.config_path = None
        self.config_target = None
        self.launched_target = None
        self.launch_args = None
        self.probe_note = None
        self.write_config()
        resources.register_secrets(config)
        resources.register_secrets(self.env)
        self.http_port = resources.free_port()
        self.grpc_port = resources.free_port()

    def write_config(self):
        """Atomically write the configuration under a fresh name per version.

        Each content version gets its own file name (config-<sha12>.json). A
        name no container has ever resolved cannot be served from a stale
        file-view cache entry, which previously handed (re)starting processes
        a truncated generation of the config (round-3: EOF at column 1355 of a
        valid 1458-byte file; round-4: EOF at column 1217 with the probe view
        already fresh). Stage + fsync + os.replace keeps each name's bytes
        atomic."""
        self.config["skill_source_roots"] = []
        payload = json.dumps(self.config)
        name = "config-%s.json" % hashlib.sha256(payload.encode()).hexdigest()[:12]
        staged = self.config_dir / (name + ".staged")
        with open(staged, "w") as handle:
            handle.write(payload)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(staged, self.config_dir / name)
        self.config_path = self.config_dir / name
        self.config_target = name
        self.config_path.chmod(0o644)

    def start(self):
        r = self.resources
        r.docker("volume", "create", self.volume)
        if self.volume not in r.volumes:
            r.volumes.append(self.volume)
        args = ["create", "--name", self.name, "--network", r.network,
                "--add-host", "host.docker.internal:host-gateway",
                "--publish", "127.0.0.1:%s:1906" % self.http_port,
                "--publish", "127.0.0.1:%s:50051" % self.grpc_port,
                "--volume", str(self.config_dir) + ":/opt/uar/phase-config:ro",
                "--volume", self.volume + ":/var/lib/uar"]
        for mount in self.mounts:
            if isinstance(mount, (tuple, list)):
                mount = ":".join(str(part) for part in mount)
            args.extend(["--volume", str(mount)])
        for key, value in self.effective_environment().items():
            args.extend(["--env", key + "=" + str(value)])
        args.extend([r.image, "universal-agent-runtime", "--config", "/opt/uar/phase-config/" + self.config_target])
        r.containers.append(self.name)
        self.launch_args = list(args)
        self.container = r.docker(*args).stdout.strip()
        self.launched_target = self.config_target
        self.base_url = "http://127.0.0.1:%s" % self.http_port
        try:
            self.readiness = r.start_attached(self.name, self._ready_receipt).ready()
            self.startup_generations.add(self.readiness["startup_generation"])
            self.probe_note = self.readiness
            (r.evidence / (self.name + "-ready-" + self.readiness["startup_generation"] + ".json")).write_text(
                json.dumps(self.readiness, indent=2))
        except BaseException:
            view = self._config_view()
            r.capture_container(self.name, launch=self.launch_args,
                                probe=self.probe_note, config_view=view)
            raise
        return self

    def _config_view(self):
        """What a fresh container sees at the mounted config path, vs intended.

        Every startup failure records the observable bytes so retries (or the
        absence of them) cannot conceal stale or incorrect configuration."""
        r = self.resources
        intended = hashlib.sha256(json.dumps(self.config).encode()).hexdigest()
        probe = r.docker("run", "--rm", "--volume", str(self.config_dir) + ":/probe:ro",
                         "--entrypoint", "cat", r.image, "/probe/" + self.config_target,
                         check=False, timeout=60)
        seen = hashlib.sha256(probe.stdout.encode()).hexdigest() if probe.returncode == 0 else "unreadable"
        return {"intended_sha256": intended, "visible_sha256": seen, "stale": seen != intended}

    def _ready_receipt(self, line):
        if not line.startswith("UAR_READY:"):
            return None
        receipt = json.loads(line[len("UAR_READY:"):])
        require(receipt.get("version") == 1, "unsupported runtime readiness version")
        require(receipt.get("config_file") == self.launched_target,
                "runtime consumed a different immutable configuration")
        require(receipt.get("port") == 1906 and isinstance(receipt.get("process_id"), int)
                and receipt["process_id"] > 0, "invalid runtime process readiness")
        generation = receipt.get("startup_generation")
        require(isinstance(generation, str) and str(uuid.UUID(generation)) == generation
                and generation not in self.startup_generations, "reused or invalid startup generation")
        return receipt

    def restart(self):
        # A fresh attachment and container avoid replay of an older READY event.
        self.stop()
        return self.start()

    def effective_environment(self):
        # Image defaults are later loader sources than the mounted file. Bind
        # these two shipped defaults to this fixture's actual selected store.
        persistence = self.config["persistence"]
        values = {"UAR_PERSISTENCE__PROVIDER": persistence["provider"],
                  "UAR_PERSISTENCE__DATABASE_URL": persistence["database_url"],
                  "UAR_LIFECYCLE_STDOUT": "1", "HOME": "/var/lib/uar/fixture-home",
                  "XDG_DATA_HOME": "/var/lib/uar/fixture-home/data",
                  "XDG_CACHE_HOME": "/var/lib/uar/fixture-home/cache"}
        values.update(self.env)
        return values

    def cold_restart(self):
        self.stop()
        return self.start()

    def http(self, token=None, admin_key=None):
        return Http(self.base_url, token, admin_key)

    def exec(self, *args, check=True):
        return self.resources.docker("exec", self.name, *args, check=check)

    def logs(self, *, redact=True):
        result = self.resources.docker("logs", self.name, check=False)
        raw = result.stdout + result.stderr
        # Leak assertions inspect the raw bytes in memory. Retained command and
        # container transcripts are always redacted by the resources layer.
        return self.resources.redact(raw) if redact else raw

    def stop(self):
        if self.container:
            self.resources.capture_container(self.name, probe=self.probe_note)
            exit_code = self.resources.stop_container(self.name)
            result = self.resources.docker("rm", self.name, check=False)
            require(result.returncode == 0, "owned runtime container cleanup failed")
            self.container = None
            self.resources.containers.remove(self.name)
            require(exit_code == "0", "runtime cleanup acknowledged a failing process exit: " + exit_code)

    def force_stop_for_fault(self):
        """Explicit recovery fault injection; it does not certify rollback."""
        if self.container:
            r = self.resources
            r.docker("kill", "--signal", "KILL", self.name, timeout=None)
            status = r.docker("wait", self.name, timeout=None).stdout.strip()
            owner = r.attachments.pop(self.name, None)
            attachment_exit = owner.join() if owner is not None else None
            r.capture_container(self.name, probe=self.probe_note)
            (r.evidence / (self.name + "-injected-crash-" + uuid.uuid4().hex + ".json")).write_text(json.dumps(
                dict(container=self.name, exit_code=status, attachment_exit=attachment_exit,
                     completion="process_exited", graceful=False, forced=True,
                     reason="explicit_recovery_fault_injection", durable_effects="unknown_until_reconciled"), indent=2))
            r.docker("rm", self.name)
            self.container = None
            r.containers.remove(self.name)

    def __enter__(self):
        return self.start()

    def __exit__(self, *_args):
        self.stop()


class Resources:
    def __init__(self, manifest, infrastructure, evidence):
        self.manifest = manifest
        self.infrastructure = infrastructure
        self.root = Path(manifest["resource_root"])
        self.evidence = Path(evidence)
        self.image = manifest["image"]
        self.platform = manifest["platform"]
        self.network = self.unique_name("network")
        self.network_created = False
        self.containers = []
        self.volumes = []
        self.processes = []
        self.process_events = {}
        self.files = []
        self.known_secrets = set()
        self.signing_secret = secrets.token_hex(32)
        self.admin_key = secrets.token_hex(24)
        self.known_secrets.update([self.signing_secret, self.admin_key])
        self.register_secrets(infrastructure)
        self.backends = {}
        self.command_counter = 0
        self.attachments = {}

    def unique_name(self, label):
        return "uar111-" + re.sub(r"[^a-z0-9-]", "-", label.lower())[:30] + "-" + uuid.uuid4().hex[:10]

    @staticmethod
    def free_port():
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            return sock.getsockname()[1]

    def register_secrets(self, value, key=""):
        if isinstance(value, dict):
            for child_key, child in value.items():
                self.register_secrets(child, str(child_key))
        elif isinstance(value, (list, tuple)):
            for child in value:
                self.register_secrets(child, key)
        elif isinstance(value, str) and value and any(
                word in key.lower() for word in ("secret", "password", "pass", "token", "api_key", "authorization")):
            self.known_secrets.add(value)

    def redact(self, value):
        value = str(value)
        for secret in sorted(self.known_secrets, key=len, reverse=True):
            value = value.replace(secret, "[redacted]")
        value = re.sub(r"(?i)Bearer\s+[^\s\"']+", "[redacted]", value)
        value = re.sub(r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+", "[redacted]", value)
        return re.sub(r"(://[^/@\s:]+:)[^/@\s]+@", r"\1[redacted]@", value)

    def run_command(self, argv, *, check=True, env=None, cwd=ROOT, timeout=900, **_kwargs):
        self.command_counter += 1
        identity = "%05d" % self.command_counter
        receipt = self.evidence / ("command-" + identity + ".json")
        record = {"argv": [self.redact(part) for part in argv], "cwd": str(cwd),
                  "started_at": time.time(), "status": "started"}
        receipt.write_text(json.dumps(record, indent=2))
        child_env = os.environ.copy()
        child_env.pop("DOCKER_DEFAULT_PLATFORM", None)
        if env:
            self.register_secrets(env)
            child_env.update({str(k): str(v) for k, v in env.items()})
        try:
            result = subprocess.run([str(x) for x in argv], cwd=cwd, env=child_env,
                                    text=True, capture_output=True, timeout=timeout)
        except BaseException as error:
            incomplete = isinstance(error, (KeyboardInterrupt, subprocess.TimeoutExpired))
            record.update(status="incomplete" if incomplete else "failed",
                          outcome="unknown" if incomplete else "command_error",
                          error=type(error).__name__, finished_at=time.time())
            receipt.write_text(json.dumps(record, indent=2))
            if isinstance(error, subprocess.TimeoutExpired):
                raise Blocked("command observation deadline expired; resource outcome unknown; see " + str(receipt)) from error
            raise
        (self.evidence / ("command-" + identity + ".log")).write_text(self.redact(result.stdout + result.stderr))
        record.update(status="passed" if result.returncode == 0 else "failed",
                      exit_code=result.returncode, finished_at=time.time())
        receipt.write_text(json.dumps(record, indent=2))
        if check and result.returncode:
            raise RuntimeError("command failed (%s), see %s" % (result.returncode, receipt))
        return result

    def docker(self, *args, **kwargs):
        return self.run_command(["docker", *args], **kwargs)

    def start_attached(self, name, recognize):
        previous = self.attachments.pop(name, None)
        if previous is not None:
            previous.join()
        owner = AttachedContainer(self, name, recognize)
        self.attachments[name] = owner
        return owner

    def stop_container(self, name):
        # -1 disables Docker's implicit SIGKILL escalation. Cancellation is not
        # completion: wait and join acknowledge the actual child termination.
        self.docker("stop", "--timeout", "-1", name, timeout=None)
        status = self.docker("wait", name, timeout=None).stdout.strip()
        owner = self.attachments.pop(name, None)
        attachment_exit = owner.join() if owner is not None else None
        (self.evidence / (name + "-exit-" + uuid.uuid4().hex + ".json")).write_text(json.dumps(
            dict(container=name, exit_code=status, attachment_exit=attachment_exit,
                 completion="process_exited", graceful=status == "0"), indent=2))
        return status

    def authenticate_surreal(self, name, user, password):
        address = self.docker("port", name, "8000/tcp").stdout.strip()
        authorization = base64.b64encode((user + ":" + password).encode()).decode()
        result = Http("http://" + address).request("POST", "/sql", b"RETURN 1;", timeout=None,
            headers={"Authorization": "Basic " + authorization,
                     "Surreal-NS": "uar111", "Surreal-DB": "fixture_readiness",
                     "Content-Type": "text/plain"})
        require(isinstance(result, list) and result and all(row.get("status") == "OK" for row in result),
                "Surreal final-service authentication failed")
        return "http://" + address

    def restart_surreal(self, backend):
        previous = self.attachments.pop(backend["name"], None)
        if previous is not None:
            previous.join()
        self.start_attached(backend["name"], surreal_listening).ready()
        backend["host_url"] = self.authenticate_surreal(backend["name"], backend["username"], backend["password"])

    def restart_backend(self, kind):
        backend = self.ensure_backend(kind)
        self.stop_container(backend["name"])
        if kind == "surreal":
            self.restart_surreal(backend)
        else:
            require(kind == "postgres", "unsupported restart backend")
            self.start_attached(backend["name"], postgres_final_server(bootstrap_required=False)).ready()
            backend["host_address"] = self.authenticate_postgres(
                backend["name"], backend["image"], backend["username"], backend["password"])

    def authenticate_postgres(self, name, image, user, password):
        address = self.docker("port", name, "5432/tcp").stdout.strip()
        require(address.startswith("127.0.0.1:"), "isolated PostgreSQL published no host endpoint")
        host, _, port = address.rpartition(":")
        self.run_command(
            ["docker", "run", "--rm", "--network", "host", "--env", "PGPASSWORD=" + password,
             "--entrypoint", "psql", image, "-h", host, "-p", port,
             "-U", user, "-d", "postgres", "-v", "ON_ERROR_STOP=1", "-c", "select 1"],
            timeout=None)
        return address

    def start_event_process(self, argv, **kwargs):
        process = self.start_process(argv, stdout=subprocess.PIPE, stdin=subprocess.PIPE, **kwargs)
        owner = ProcessEvents(self, process, self.processes[-1][1])
        self.process_events[process.pid] = owner
        return owner

    def start_process(self, argv, *, env=None, cwd=None, label="peer", **kwargs):
        path = self.root / (self.unique_name(label) + ".log")
        output = path.open("w")
        self.files.append(output)
        child_env = {k: v for k, v in os.environ.items()
                     if k in ("PATH", "LANG", "LC_ALL", "SYSTEMROOT", "TMPDIR")}
        child_home = self.root / self.unique_name("peer-home")
        child_home.mkdir(mode=0o700)
        child_env.update(HOME=str(child_home), XDG_DATA_HOME=str(child_home / "data"),
                         XDG_CACHE_HOME=str(child_home / "cache"))
        if env:
            self.register_secrets(env)
            child_env.update({str(k): str(v) for k, v in env.items()})
        process = subprocess.Popen([str(x) for x in argv], cwd=cwd or ROOT,
                                   env=child_env, stdout=kwargs.get("stdout", output),
                                   stdin=kwargs.get("stdin"),
                                   stderr=kwargs.get("stderr", subprocess.STDOUT), start_new_session=True)
        self.processes.append((process, path))
        return process

    def start(self):
        result = self.docker("network", "create", self.network, check=False)
        if result.returncode:
            raise Blocked("isolated Docker network unavailable")
        self.network_created = True
        return self

    def ensure_backend(self, kind):
        if kind in self.backends:
            return self.backends[kind]
        settings = self.infrastructure.get(kind)
        if not isinstance(settings, dict):
            raise Blocked("private infrastructure lacks " + kind)
        name = self.unique_name(kind)
        volume = self.unique_name(kind + "-data")
        user = settings.get("username", "root" if kind == "surreal" else "postgres")
        password = settings.get("password")
        if not isinstance(password, str) or not password:
            raise Blocked("isolated backend password missing: " + kind)
        self.known_secrets.add(password)
        self.docker("volume", "create", volume)
        self.volumes.append(volume)
        self.containers.append(name)
        if kind == "surreal":
            image = settings.get("image", SURREAL_IMAGE)
            require(image == SURREAL_IMAGE, "Surreal image must match versions.toml")
            self.docker("create", "--name", name, "--network", self.network,
                        "--network-alias", "surreal", "--publish", "127.0.0.1::8000",
                        "--user", "0", "--volume", volume + ":/data", image,
                        "start", "--user", user, "--pass", password, "--bind", "0.0.0.0:8000", "rocksdb:/data/phase.db")
            self.start_attached(name, surreal_listening).ready()
            host_url = self.authenticate_surreal(name, user, password)
            entry = dict(name=name, username=user, password=password, url="http://surreal:8000",
                         host_url=host_url, image=image)
        elif kind == "postgres":
            image = settings.get("image")
            if not isinstance(image, str) or "@sha256:" not in image:
                raise Blocked("private infrastructure must pin the isolated PostgreSQL image digest")
            # The pinned Supabase image completes first-boot initialization only
            # when its bootstrap superuser is supabase_admin: migrate.sh
            # authenticates as supabase_admin and then creates the postgres role
            # with the same password. Provisioning POSTGRES_USER=postgres left
            # supabase_admin missing -> migrate.sh FATAL -> entrypoint abort ->
            # container exit 2 -> published bindings torn down -> the next
            # `docker port` reported "no public port" (round-2 root cause).
            self.docker("create", "--name", name, "--network", self.network,
                        "--network-alias", "postgres", "--publish", "127.0.0.1::5432",
                        "--env", "POSTGRES_USER=supabase_admin", "--env", "POSTGRES_PASSWORD=" + password,
                        "--volume", volume + ":/var/lib/postgresql/data", image)
            self.start_attached(name, postgres_final_server()).ready()
            # Exactly one authenticated operation after final-postmaster receipt.
            address = self.authenticate_postgres(name, image, user, password)
            entry = dict(name=name, username=user, password=password, image=image,
                         host_address=address)
        else:
            raise Blocked("unsupported isolated backend: " + kind)
        self.backends[kind] = entry
        return entry

    def base_config(self, backend="surreal", database=None):
        database = database or self.unique_name("db").replace("-", "_")
        security = dict(jwt_required=True, jwt_secret=self.signing_secret,
                        jwt_algorithm="HS256", jwt_issuer="phase-issuer", jwt_audience="phase-runtime",
                        jwt_validate_nbf=True, settings_mutation_auth_required=True,
                        settings_admin_key=self.admin_key, trusted_host_principals=[dict(
                            issuer="phase-issuer", subject="phase-host", tenant_id="phase-tenant", host_id="phase-host")])
        persistence = dict(provider=backend, vector_dimension=384, external_cache_enabled=False)
        if backend == "surreal":
            db = self.ensure_backend("surreal")
            persistence.update(database_url=db["url"], surreal_user=db["username"],
                               surreal_pass=db["password"], surreal_ns="uar111", surreal_db=database)
        elif backend == "postgres":
            db = self.ensure_backend("postgres")
            require(re.fullmatch(r"[a-zA-Z0-9_]+", database), "invalid owned database identifier")
            self.docker("exec", db["name"], "createdb", "-U", db["username"], database)
            persistence["database_url"] = "postgres://%s:%s@postgres:5432/%s" % (
                urllib.parse.quote(db["username"], safe=""), urllib.parse.quote(db["password"], safe=""), database)
        elif backend == "local":
            persistence.update(provider="surreal", database_url="surrealkv:///var/lib/uar/data/phase.db")
        else:
            raise Blocked("unsupported runtime persistence profile")
        return dict(server=dict(host="0.0.0.0", port=1906, grpc_port=50051, cors_allowed_origins=[]),
                    security=security, persistence=persistence, memory=dict(enabled=False), providers=[],
                    skill_source_roots=[],
                    resilience=dict(rate_limit_enabled=False),
                    native_tools=dict(file_tools_enabled=False, web_fetch_enabled=False,
                                      terminal_exec_enabled=False, session_search_enabled=False))

    def jwt(self, subject, roles, issuer=None, tenant=None, host=False):
        def encode(value):
            return base64.urlsafe_b64encode(value).rstrip(b"=").decode()
        now = int(time.time())
        claims = dict(sub=subject, roles=roles, iss=issuer or "phase-issuer",
                      tenant_id=tenant or "phase-tenant", aud="phase-runtime", nbf=now - 5,
                      exp=now + 14400, uar_credential_kind="issuer")
        if host:
            claims["uar_instance_id"] = "phase-host"
        message = encode(json.dumps(dict(alg="HS256", typ="JWT")).encode()) + "." + encode(json.dumps(claims).encode())
        token = message + "." + encode(hmac.new(self.signing_secret.encode(), message.encode(), hashlib.sha256).digest())
        self.known_secrets.add(token)
        return token

    def relay(self, port):
        name = self.unique_name("relay")
        self.containers.append(name)
        script = ROOT / "scripts/release/uar_1_1_loopback_relay.py"
        self.docker("create", "--name", name, "--network", self.network,
                    "--add-host", "host.docker.internal:host-gateway", "--entrypoint", "python3",
                    "--volume", str(script) + ":/relay.py:ro", self.image, "/relay.py",
                    "--listen-host", "0.0.0.0", "--listen-port", str(port),
                    "--upstream-host", "host.docker.internal", "--upstream-port", str(port),
                    "--ready-file", "/tmp/ready", "--pid-file", "/tmp/pid")
        self.start_attached(name, lambda line: {"listening": True}
                            if line.strip() == "UAR_RELAY_READY" else None).ready()
        data = json.loads(self.docker("inspect", name).stdout)[0]
        address = data["NetworkSettings"]["Networks"][self.network]["IPAddress"]
        require(any(ipaddress.ip_address(address) in ipaddress.ip_network(cidr)
                    for cidr in ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16")), "relay requires RFC1918 address")
        return "http://%s:%s" % (address, port)

    def surreal_query(self, database, sql, namespace="uar111"):
        db = self.ensure_backend("surreal")
        require(namespace == "uar111" and re.fullmatch(r"[a-zA-Z0-9_]+", database), "query outside owned namespace")
        authorization = base64.b64encode((db["username"] + ":" + db["password"]).encode()).decode()
        return Http(db["host_url"]).request("POST", "/sql", sql.encode(), headers={
            "Authorization": "Basic " + authorization, "Surreal-NS": namespace,
            "Surreal-DB": database, "Content-Type": "text/plain"})

    def postgres_query(self, database, sql):
        db = self.ensure_backend("postgres")
        require(re.fullmatch(r"[a-zA-Z0-9_]+", database), "query outside owned database")
        return self.docker("exec", db["name"], "psql", "-X", "-v", "ON_ERROR_STOP=1",
                           "-U", db["username"], "-d", database, "--csv", "-c", sql).stdout

    def new_runtime(self, label, config, env=None, mounts=None):
        return RuntimeLease(self, label, config, env, mounts)

    def capture_container(self, name, launch=None, probe=None, attempt=None, config_view=None):
        result = self.docker("logs", name, check=False)
        path = self.evidence / (name + "-" + uuid.uuid4().hex[:6] + ".log")
        path.write_text(self.redact(result.stdout + result.stderr))
        state = self.docker("inspect", "--format",
                            "{{.State.Status}}|{{.State.ExitCode}}|{{.State.Error}}", name, check=False)
        diagnostics = {"container": name, "captured_at": time.time(),
                       "container_logs": path.name,
                       "container_state": self.redact(state.stdout.strip()) if state.returncode == 0 else "inspect-unavailable",
                       "launch_command": self.redact(" ".join(str(part) for part in launch)) if launch else None,
                       "health_probe": self.redact(str(probe)) if probe is not None else None,
                       "attempt": attempt,
                       "config_view": config_view}
        (self.evidence / (path.stem + ".diag.json")).write_text(json.dumps(diagnostics, indent=2))

    def close(self):
        errors = []
        for process, path in reversed(self.processes):
            try:
                owner = self.process_events.get(process.pid)
                if owner is not None:
                    # Managed peers own stdin EOF, release held HTTP handlers,
                    # and join them before process termination.
                    try:
                        process.stdin.close()
                    except OSError as error:
                        errors.append(type(error).__name__)
                    stop_request = "inherited_eof"
                else:
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    stop_request = "SIGTERM"
                process.wait()
                if owner is not None:
                    owner.join()
                    self.process_events.pop(process.pid, None)
                (self.evidence / (path.stem + "-exit.json")).write_text(json.dumps(
                    dict(process_id=process.pid, exit_code=process.returncode,
                         completion="process_exited_and_output_joined", stop_request=stop_request), indent=2))
                if path.exists():
                    (self.evidence / path.name).write_text(self.redact(path.read_text(errors="replace")))
            except Exception as error:
                errors.append(type(error).__name__)
        for name in reversed(self.containers):
            try:
                self.capture_container(name)
            except Exception as error:
                errors.append(type(error).__name__)
            finally:
                try:
                    self.stop_container(name)
                    result = self.docker("rm", name, check=False)
                    if result.returncode:
                        errors.append("container_cleanup_failed")
                except Exception as error:
                    errors.append(type(error).__name__)
        for volume in reversed(self.volumes):
            try:
                result = self.docker("volume", "rm", volume, check=False)
                if result.returncode:
                    errors.append("volume_cleanup_failed")
            except Exception as error:
                errors.append(type(error).__name__)
        if self.network_created:
            try:
                result = self.docker("network", "rm", self.network, check=False)
                if result.returncode:
                    errors.append("network_cleanup_failed")
            except Exception as error:
                errors.append(type(error).__name__)
        for file in self.files:
            file.close()
        if errors:
            raise RuntimeError("owned resource cleanup incomplete: " + ",".join(errors))

    def context(self, group, evidence):
        return dict(group=group, resources=self, http=Http, jwt=self.jwt,
                    base_config=self.base_config, new_runtime=self.new_runtime,
                    run_command=self.run_command, resource_root=self.root,
                    evidence_root=Path(evidence), infrastructure=self.infrastructure,
                    image=self.image, platform=self.platform,
                    identity=dict(issuer="phase-issuer", tenant="phase-tenant", host_subject="phase-host"),
                    admin_key=self.admin_key, require=require)
