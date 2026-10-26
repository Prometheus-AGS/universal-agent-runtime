"""Shared, standard-library user-outcome acceptance primitives.

Importing this module has no network, subprocess or filesystem side effects.
All execution belongs to the separately approved acceptance packet.
"""
from __future__ import annotations

from dataclasses import dataclass, field
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import threading
import time
from typing import Callable, Any
import urllib.error
import urllib.parse
import urllib.request


class Blocked(RuntimeError):
    """An unavailable prerequisite is never successful acceptance."""


class Failed(RuntimeError):
    """A product or protocol predicate failed."""


def require(condition: Any, message: str) -> None:
    if not condition:
        raise Failed(message)


def digest(raw: bytes) -> str:
    return "sha256:" + hashlib.sha256(raw).hexdigest()


@dataclass(frozen=True)
class Predicate:
    passed: bool
    detail: str


@dataclass(frozen=True)
class Outcome:
    protocol: Predicate
    useful: Predicate
    facts: dict = field(default_factory=dict)


@dataclass(frozen=True)
class Case:
    case_id: str
    family: str
    run: Callable[[Context], Outcome]
    repetitions: int = 3
    durable: bool = False


@dataclass(frozen=True)
class Response:
    status: int
    headers: dict[str, str]
    body: bytes

    def json(self) -> Any:
        return json.loads(self.body)


class Evidence:
    """Exclusive redacted receipts with a final hash-chain manifest.

    File permissions prevent accidental edits; hashes expose later changes.
    This is local tamper evidence, not an external signature or WORM store.
    """

    def __init__(self, root: Path, secrets: list[str]):
        root.mkdir(mode=0o700, parents=True, exist_ok=False)
        self.root = root
        self.secrets = sorted(set(filter(None, secrets)), key=len, reverse=True)
        self.receipts: list[dict] = []
        self.sealed = False

    def redact(self, value: Any) -> Any:
        if isinstance(value, dict):
            return {str(key): ("[REDACTED]" if re.search(
                r"authorization|cookie|password|secret|api.?key|access.?token|refresh.?token|reasoning|thinking|thoughts|chain.of.thought",
                str(key), re.I) else self.redact(item)) for key, item in value.items()}
        if isinstance(value, (list, tuple)):
            return [self.redact(item) for item in value]
        if isinstance(value, str):
            for secret in self.secrets:
                value = value.replace(secret, "[REDACTED]")
            return re.sub(r"(?i)bearer\s+[^\s\"<>]+", "Bearer [REDACTED]", value)
        return value

    def write(self, name: str, value: Any) -> str:
        if self.sealed:
            raise RuntimeError("evidence is sealed")
        safe = re.sub(r"[^a-zA-Z0-9_.-]", "_", name)[:100]
        filename = f"{len(self.receipts):05d}-{safe}.json"
        raw = json.dumps(self.redact(value), sort_keys=True, ensure_ascii=False,
                         indent=2, allow_nan=False).encode() + b"\n"
        path = self.root / filename
        with path.open("xb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        path.chmod(0o400)
        previous = self.receipts[-1]["chain"] if self.receipts else ""
        entry = {"path": filename, "sha256": digest(raw), "previous": previous}
        entry["chain"] = digest(json.dumps(entry, sort_keys=True).encode())
        self.receipts.append(entry)
        return filename

    def seal(self, summary: dict) -> None:
        self.write("summary", summary)
        manifest = {"schema_version": 1, "receipts": list(self.receipts)}
        path = self.root / "manifest.json"
        with path.open("xb") as stream:
            stream.write(json.dumps(manifest, sort_keys=True, indent=2).encode() + b"\n")
            stream.flush()
            os.fsync(stream.fileno())
        path.chmod(0o400)
        self.sealed = True
        self.root.chmod(0o500)


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Context:
    def __init__(self, fixture: dict, backend: dict, run_id: str, evidence: Evidence):
        self.fixture = fixture
        self.backend = backend
        self.run_id = run_id
        self.evidence = evidence
        self.attempt = 0
        self.case_id = "admission"
        self.started = time.monotonic()
        self.requests = 0
        self.output_bytes = 0
        self.processes: list[subprocess.Popen] = []
        self.server: subprocess.Popen | None = None
        self.cleanup_deadline: float | None = None
        # An owned runtime must not discover the operator's ambient provider
        # keys, HOME configuration or unrelated UAR overrides.
        system_names = ("PATH", "SYSTEMROOT", "WINDIR", "COMSPEC", "LANG", "LC_ALL",
                        "TZ", "SSL_CERT_FILE", "SSL_CERT_DIR")
        self.environment = {name: os.environ[name] for name in system_names if name in os.environ}
        for key, reference in backend.get("environment_env", {}).items():
            if not os.environ.get(reference):
                raise Blocked(f"missing environment reference: {reference}")
            self.environment[key] = os.environ[reference]
            self.environment[reference] = os.environ[reference]
        references = list(backend.get("headers_env", {}).values()) + fixture.get("secret_env", [])
        references += [binding["credential_env"] for binding in
                       [fixture["provider"], *fixture.get("genuine_bindings", [])]
                       if binding.get("credential_env")]
        for reference in references:
            if reference not in os.environ:
                raise Blocked(f"missing named environment reference: {reference}")
            self.environment[reference] = os.environ[reference]
        parent = Path(self.environment.get("UAR_ACCEPTANCE_STATE_PARENT", ""))
        if not parent.is_absolute():
            raise Blocked("an absolute run-owned state parent must be explicitly configured")
        scope = "uar_acceptance_" + hashlib.sha256(run_id.encode()).hexdigest()[:20]
        self.environment["HOME"] = str(parent / scope / "home")
        self.environment["TMPDIR"] = str(parent / scope / "tmp")
        self.environment["UAR_ACCEPTANCE_FIXTURE_PATH"] = fixture["_fixture_path"]
        self.environment["UAR_ACCEPTANCE_CONTEXT"] = json.dumps({
            "run_id": run_id, "backend": backend["name"], "case_id": "admission",
            "attempt": 0, "values": {},
        })
        self.environment["UAR_ACCEPTANCE_RUN_ID"] = run_id
        self.environment["UAR_ACCEPTANCE_BACKEND"] = backend["name"]
        self.opener = urllib.request.build_opener(NoRedirect)

    def remaining(self) -> float:
        for process in self.processes:
            for stream in process._acceptance_output:
                if not stream.closed and os.fstat(stream.fileno()).st_size > self.fixture["limits"]["response_bytes"]:
                    raise Failed("owned process output exceeded allowance")
        remaining = (self.cleanup_deadline - time.monotonic() if self.cleanup_deadline is not None
                     else self.fixture["limits"]["wall_seconds"] - (time.monotonic() - self.started))
        if remaining <= 0:
            raise Blocked("wall-clock allowance exhausted; no retry authorized")
        return remaining

    def record(self, name: str, value: Any) -> str:
        return self.evidence.write(f"{self.backend['name']}-{self.case_id}-{self.attempt}-{name}", value)

    def require(self, condition: Any, message: str) -> None:
        require(condition, message)

    def block(self, message: str) -> None:
        raise Blocked(message)

    def request(self, method: str, path: str, payload: Any = None,
                headers: dict | None = None) -> Response:
        raw = None if payload is None else json.dumps(payload, allow_nan=False).encode()
        return self.request_bytes(method, path, raw, "application/json", headers, payload)

    def request_bytes(self, method: str, path: str, raw: bytes | None,
                      content_type: str, headers: dict | None = None,
                      receipt_body: Any = None) -> Response:
        """Upload bounded multipart/binary data under the same host/evidence fence."""
        if not path.startswith("/") or path.startswith("//"):
            raise Blocked("HTTP path must remain on the fixture origin")
        if self.requests >= self.fixture["limits"]["http_requests"]:
            raise Blocked("HTTP request allowance exhausted")
        self.requests += 1
        if raw is not None and len(raw) > self.fixture["limits"]["response_bytes"]:
            raise Blocked("request byte allowance exhausted")
        request_headers = {"Content-Type": content_type}
        for header, variable in self.backend.get("headers_env", {}).items():
            value = os.environ.get(variable)
            if not value:
                raise Blocked(f"missing credential environment reference: {variable}")
            request_headers[header] = value
        request_headers.update(headers or {})
        self.record("request", {"method": method, "path": path, "headers": request_headers,
                                "body": receipt_body if receipt_body is not None else
                                ({"bytes": len(raw), "digest": digest(raw)} if raw is not None else None)})
        started = time.monotonic()
        request = urllib.request.Request(self.backend["base_url"].rstrip("/") + path,
                                         data=raw, headers=request_headers, method=method)
        # A socket timeout measures inactivity, so SSE heartbeats can keep an
        # aggregate read alive past the packet wall. Interrupt open/headers/body
        # in this serial main-thread runner without leaving a worker behind.
        if (os.name != "posix" or threading.current_thread() is not threading.main_thread()
                or not all(hasattr(signal, name) for name in
                           ("SIGALRM", "ITIMER_REAL", "getitimer", "setitimer",
                            "pthread_sigmask", "SIG_BLOCK"))):
            raise Blocked("bounded HTTP requires POSIX interval timers on the main thread")
        if signal.SIGALRM in signal.pthread_sigmask(signal.SIG_BLOCK, []):
            raise Blocked("bounded HTTP requires an unblocked SIGALRM")
        if any(signal.getitimer(signal.ITIMER_REAL)):
            raise Blocked("bounded HTTP cannot replace an existing active interval timer")
        allowance = min(self.remaining(), self.fixture["limits"]["request_seconds"])
        deadline = time.monotonic() + allowance
        prefix = bytearray()
        limit = self.fixture["limits"]["response_bytes"]

        def interrupt_request(signum, frame):
            raise TimeoutError("absolute HTTP deadline exhausted")

        previous_handler = signal.signal(signal.SIGALRM, interrupt_request)
        try:
            try:
                # Never rearm per chunk: every blocking stage shares this wall.
                signal.setitimer(signal.ITIMER_REAL, allowance)
                try:
                    reply = self.opener.open(request, timeout=allowance)
                except urllib.error.HTTPError as error:
                    reply = error
                with reply:
                    while len(prefix) <= limit:
                        self.remaining()
                        if time.monotonic() >= deadline:
                            raise TimeoutError("absolute HTTP deadline exhausted")
                        chunk = reply.read1(min(65536, limit + 1 - len(prefix)))
                        if not chunk:
                            break
                        prefix.extend(chunk)
                    self.remaining()
                    if time.monotonic() >= deadline:
                        raise TimeoutError("absolute HTTP deadline exhausted")
                    body = bytes(prefix)
                    response = Response(reply.code, {k.lower(): v for k, v in reply.headers.items()}, body)
            finally:
                try:
                    signal.setitimer(signal.ITIMER_REAL, 0)
                finally:
                    signal.signal(signal.SIGALRM, previous_handler)
        except Blocked:
            self.output_bytes += len(prefix)
            self.record("transport-failure", {"type": "Blocked", "cause": "allowance exhausted",
                        "partial_bytes": len(prefix), "partial_digest": digest(bytes(prefix)),
                        "elapsed_seconds": time.monotonic() - started})
            raise
        except (OSError, urllib.error.URLError, TimeoutError) as error:
            self.output_bytes += len(prefix)
            self.record("transport-failure", {"type": type(error).__name__, "message": str(error),
                        "partial_bytes": len(prefix), "partial_digest": digest(bytes(prefix)),
                        "elapsed_seconds": time.monotonic() - started})
            # Global wall exhaustion is BLOCKED, including an alarm surfaced as
            # URLError by urllib. A request-only deadline remains a transport FAIL.
            self.remaining()
            raise Failed("HTTP transport failed; receipt retained") from error
        self.output_bytes += len(body)
        self.record("response", {"status": response.status, "headers": response.headers,
                                  "body": self.public_body(body),
                                  "elapsed_seconds": time.monotonic() - started})
        require(len(body) <= limit, "response byte allowance exceeded")
        require(self.output_bytes <= self.fixture["limits"]["total_output_bytes"],
                "cumulative output allowance exceeded")
        return response

    @staticmethod
    def public_body(body: bytes) -> Any:
        try:
            return json.loads(body)
        except (ValueError, UnicodeDecodeError):
            # Retain structured SSE data before cleanup without capturing opaque
            # private reasoning or guessing at malformed event payloads.
            try:
                text = body.decode("utf-8")
            except UnicodeDecodeError:
                text = ""
            lines = [line for line in text.splitlines() if line.strip()]
            if lines:
                try:
                    records = [json.loads(line) for line in lines]
                    if all(isinstance(record, dict) for record in records):
                        return {"json_records": records, "source_digest": digest(body)}
                except ValueError:
                    pass
            events = []
            unknown = 0
            for block in re.split(r"\r?\n\r?\n", text):
                lines = block.splitlines()
                event = next((line[6:].strip() for line in lines if line.startswith("event:")), "message")
                data = "\n".join(line[5:].lstrip() for line in lines if line.startswith("data:"))
                if not data:
                    if block.strip() and not all(line.startswith(":") for line in lines):
                        unknown += 1
                    continue
                if re.search(r"reasoning|thought|thinking", event, re.I):
                    events.append({"event": event, "data": "[PRIVATE REASONING WITHHELD]"})
                    continue
                if data == "[DONE]":
                    events.append({"event": event, "data": "[DONE]"})
                    continue
                try:
                    parsed = json.loads(data)
                except ValueError:
                    unknown += 1
                    continue
                kind = parsed.get("type", "") if isinstance(parsed, dict) else ""
                if re.search(r"reasoning|thought|thinking", str(kind), re.I):
                    parsed = "[PRIVATE REASONING WITHHELD]"
                events.append({"event": event, "data": parsed})
            if events:
                return {"sse_events": events, "unparsed_blocks": unknown,
                        "evidence_complete": unknown == 0, "source_digest": digest(body)}
            # Non-SSE text may contain private model reasoning; retain identity.
            return {"unparsed_bytes": len(body), "digest": digest(body),
                    "evidence_complete": not body,
                    "cause": "unrecovered" if body else None,
                    "redaction": "opaque non-JSON response withheld"}

    def owned_process(self, argv: list[str], env: dict | None = None) -> subprocess.Popen:
        if not argv or not all(isinstance(item, str) for item in argv):
            raise Blocked("owned command must be a nonempty argv list")
        self.remaining()
        # Child output is private until bounded, redacted and retained by observe().
        output = tempfile.NamedTemporaryFile(prefix="uar-acceptance-stdout-")
        error = tempfile.NamedTemporaryFile(prefix="uar-acceptance-stderr-")
        try:
            process = subprocess.Popen(argv, env={**self.environment, **(env or {})},
                                       stdin=subprocess.DEVNULL, stdout=output, stderr=error,
                                       start_new_session=True)
        except BaseException:
            output.close()
            error.close()
            raise
        process._acceptance_output = (output, error)
        self.processes.append(process)
        self.environment["UAR_ACCEPTANCE_OWNED_PROCESS_IDS"] = json.dumps(
            [item.pid for item in self.processes if item.poll() is None])
        self.record("process-start", {"pid": process.pid, "argv": argv})
        return process

    def wait_process(self, process: subprocess.Popen, seconds: float) -> None:
        """Await a finite command while bounding its private output files."""
        deadline = time.monotonic() + seconds
        while process.poll() is None:
            for stream in process._acceptance_output:
                if os.fstat(stream.fileno()).st_size > self.fixture["limits"]["response_bytes"]:
                    self.stop_process(process)
                    raise Failed("owned command output exceeded allowance")
            remaining = min(deadline - time.monotonic(), self.remaining())
            if remaining <= 0:
                raise subprocess.TimeoutExpired(process.args, seconds)
            try:
                process.wait(timeout=min(0.1, remaining))
            except subprocess.TimeoutExpired:
                continue
        process.wait()

    def stop_process(self, process: subprocess.Popen) -> None:
        if process not in self.processes:
            raise Blocked("refusing to stop a process not owned by this invocation")
        # The foreground Docker client is only the host observation process.
        # Stop and confirm its exact owned container before joining the client;
        # a client exit alone cannot prove Linux runtime shutdown.
        if (process is self.server
                and self.backend.get("runtime_platform") == "linux-container"):
            receipt = self.observe("runtime_stop", {"process_pid": process.pid})
            require(receipt.get("run_id") == self.run_id
                    and receipt.get("container_absent") is True,
                    "owned Linux runtime shutdown is unconfirmed")
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=self.fixture["limits"]["cleanup_seconds"])
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=self.fixture["limits"]["cleanup_seconds"])
        else:
            process.wait()
            # A finished command may have left descendants in its owned group.
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        self.record("process-wait", {"pid": process.pid, "returncode": process.returncode})
        for label, stream in zip(("stdout", "stderr"), process._acceptance_output):
            if not stream.closed:
                stream.seek(0)
                raw = stream.read(self.fixture["limits"]["response_bytes"] + 1)
                self.record("process-" + label, {"pid": process.pid,
                            "body": self.public_body(raw),
                            "truncated": len(raw) > self.fixture["limits"]["response_bytes"]})

    def observe(self, name: str, values: dict | None = None) -> dict:
        command = self.backend.get("observers", {}).get(name)
        if not command:
            raise Blocked(f"missing implemented observation command: {name}")
        context = {"run_id": self.run_id, "backend": self.backend["name"],
                   "case_id": self.case_id, "attempt": self.attempt, "values": values or {}}
        process = self.owned_process(command, {"UAR_ACCEPTANCE_CONTEXT": json.dumps(context)})
        try:
            self.wait_process(process, min(self.remaining(), self.fixture["limits"]["command_seconds"]))
        except subprocess.TimeoutExpired as error:
            self.stop_process(process)
            self.record("observer-timeout", {"name": name})
            raise Failed(f"observer timed out: {name}") from error
        output, error_output = process._acceptance_output
        output.seek(0)
        error_output.seek(0)
        limit = self.fixture["limits"]["response_bytes"]
        raw, errors = output.read(limit + 1), error_output.read(limit + 1)
        self.output_bytes += len(raw) + len(errors)
        self.record("observation", {"name": name, "returncode": process.returncode,
                                    "stdout": self.public_body(raw),
                                    "stderr": self.public_body(errors)})
        require(process.returncode == 0, f"observation failed: {name}")
        require(len(raw) <= limit and len(errors) <= limit, "observation output exceeded allowance")
        require(self.output_bytes <= self.fixture["limits"]["total_output_bytes"],
                "cumulative observer output allowance exceeded")
        try:
            value = json.loads(raw)
        except (ValueError, UnicodeDecodeError) as error:
            raise Blocked(f"observation lacks JSON receipt: {name}") from error
        if not isinstance(value, dict):
            raise Blocked(f"observation receipt must be an object: {name}")
        # Joined finite observers must not retain two open temporary files each
        # across hundreds of cases. Quiesce its owned group before releasing it.
        self.stop_process(process)
        self.release_process(process)
        return value

    def release_process(self, process: subprocess.Popen) -> None:
        """Release joined output captures without losing live process ownership."""
        require(process in self.processes and process.poll() is not None,
                "only a joined invocation-owned process can be released")
        for stream in process._acceptance_output:
            stream.close()
        self.processes.remove(process)
        self.environment["UAR_ACCEPTANCE_OWNED_PROCESS_IDS"] = json.dumps(
            [item.pid for item in self.processes if item.poll() is None])

    def start(self) -> None:
        command = self.backend.get("server", {}).get("argv")
        if not command:
            raise Blocked("fixture must own its real server process")
        self.server = self.owned_process(command)
        self.environment["UAR_ACCEPTANCE_SERVER_PID"] = str(self.server.pid)
        output, errors = self.server._acceptance_output
        self.environment["UAR_ACCEPTANCE_SERVER_STDOUT_PATH"] = output.name
        self.environment["UAR_ACCEPTANCE_SERVER_STDERR_PATH"] = errors.name
        ready = self.observe("ready")
        require(ready.get("ready") is True, "owned server did not become ready")

    def restart(self) -> None:
        if self.server is None:
            raise Blocked("cold restart requires the runner-owned server")
        previous = self.server.pid
        self.stop_process(self.server)
        self.release_process(self.server)
        self.start()
        self.record("cold-restart", {"old_pid": previous, "new_pid": self.server.pid,
                                     "same_durable_profile": self.backend["profile_id"]})

    def cleanup(self) -> None:
        # Cleanup has its own explicitly bounded allowance so exhausted useful
        # work cannot prevent deleting exact-owned resources.
        self.cleanup_deadline = time.monotonic() + self.fixture["limits"]["cleanup_seconds"]
        failures = []
        for process in reversed(self.processes):
            try:
                self.stop_process(process)
            except (OSError, subprocess.TimeoutExpired) as error:
                failures.append({"pid": process.pid, "error": str(error)})
            finally:
                for stream in process._acceptance_output:
                    stream.close()
        self.record("owned-process-cleanup", {"failures": failures})
        if failures:
            raise Blocked("owned process cleanup could not be confirmed")
        receipt = self.observe("cleanup", {"processes_joined": True})
        require(receipt.get("run_id") == self.run_id and receipt.get("residuals") == [],
                "owned resource cleanup not confirmed")
        # observe() already joins/releases its finite process and output files.
