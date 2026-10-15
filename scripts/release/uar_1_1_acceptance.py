#!/usr/bin/env python3
"""Installed-image acceptance for the UAR v1.1.0 release candidate.

This script is deliberately executable only at the KBD full-phase boundary. It
starts the supplied image, a pinned SurrealDB persistence service, and the
repository's controlled provider/MCP peers. No UAR behavior is simulated here.

Examples:
  python3 scripts/release/uar_1_1_acceptance.py \
    --image ghcr.io/prometheus-ags/universal-agent-runtime@sha256:... \
    --evidence /tmp/uar-1.1.0-evidence
  python3 scripts/release/uar_1_1_acceptance.py \
    --image uar:release-candidate --local-tag --evidence /tmp/uar-evidence
"""

import argparse
import base64
import hashlib
import hmac
import importlib.util
import ipaddress
import json
import os
from pathlib import Path
import platform
import re
import secrets
import shutil
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid


ROOT = Path(__file__).resolve().parents[2]
SURREAL_IMAGE = "surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20"
HTTP_PORT = 1906
GRPC_PORT = 50051
RELEASE_SECRET = "uar-v1-1-0-release-acceptance-secret"
RELEASE_ISSUER = "uar-release-acceptance"
RELEASE_AUDIENCE = "uar-release-candidate"
TENANT = "release-tenant"
HOST_SUBJECT = "release-host"
HOST_ID = "release-host"
OWNER_SUBJECT = "release-owner"
PROTECTED_ROLE = "release:protected"
PROTECTED_SKILL = "release-protected-skill"
SURR_USER = "root"
SURR_PASS = "release-surreal-password"
GRPCURL_VERSION = "1.9.4"
GRPCURL_CHECKSUMS_URL = (
    "https://github.com/fullstorydev/grpcurl/releases/download/v1.9.4/"
    "grpcurl_1.9.4_checksums.txt"
)
GRPCURL_CHECKSUMS_SHA256 = "f861790e2dd2ac2a3cc29cb985edc5554fe8460fec1a238863ccf6f5b82e677c"


class AcceptanceFailure(RuntimeError):
    pass


def require(condition, message):
    if not condition:
        raise AcceptanceFailure(message)


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, "could not load " + str(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def b64url(raw):
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode()


def jwt(subject, roles, instance_id=None):
    now = int(time.time())
    header = {"alg": "HS256", "typ": "JWT"}
    payload = {
        "sub": subject,
        "uar_credential_kind": "issuer",
        "roles": roles,
        "tenant_id": TENANT,
        "iss": RELEASE_ISSUER,
        "aud": RELEASE_AUDIENCE,
        "nbf": now - 5,
        "exp": now + 3600,
    }
    if instance_id:
        payload["uar_instance_id"] = instance_id
    signing_input = (b64url(json.dumps(header, separators=(",", ":")).encode()) + "." +
                     b64url(json.dumps(payload, separators=(",", ":")).encode())).encode()
    signature = hmac.new(RELEASE_SECRET.encode(), signing_input, hashlib.sha256).digest()
    return signing_input.decode() + "." + b64url(signature)


def run(command, *, capture=True, check=True, cwd=ROOT, env=None):
    completed = subprocess.run(command, cwd=cwd, env=env, check=False, text=True,
                               capture_output=capture)
    if check and completed.returncode:
        raise AcceptanceFailure(json.dumps({"command": redacted_command(command),
                                            "stdout": redact_text(completed.stdout),
                                            "stderr": redact_text(completed.stderr)}, indent=2))
    return completed


def docker(*args, **kwargs):
    return run(["docker", *args], **kwargs)


def redact_text(value):
    redacted = re.sub(r"Bearer\s+[A-Za-z0-9._~+/-]+", "Bearer <redacted>", value)
    return re.sub(r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+", "<redacted-jwt>", redacted)


def redacted_command(command):
    result = []
    redact_next_environment = False
    for item in command:
        text = str(item)
        if redact_next_environment:
            name = text.split("=", 1)[0]
            result.append(name + "=<redacted>" if "=" in text else name)
            redact_next_environment = False
        else:
            result.append(redact_text(text))
            redact_next_environment = text == "--env"
    return result


def wait_for(description, probe, timeout=120):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        try:
            return probe()
        except (AcceptanceFailure, OSError, urllib.error.URLError, urllib.error.HTTPError,
                json.JSONDecodeError) as error:
            last = error
            time.sleep(0.25)
    raise AcceptanceFailure("timed out waiting for " + description + ": " + str(last))


class Http:
    def __init__(self, base_url, token=None, admin_key=None):
        self.base_url = base_url.rstrip("/")
        self.token = token
        self.admin_key = admin_key

    def request(self, method, path, body=None, expected=200, headers=None, return_status=False):
        request_headers = {"Accept": "application/json"}
        if self.token:
            request_headers["Authorization"] = "Bearer " + self.token
        if self.admin_key:
            request_headers["X-UAR-Admin-Key"] = self.admin_key
        if headers:
            request_headers.update(headers)
        data = None
        if body is not None:
            request_headers["Content-Type"] = "application/json"
            data = json.dumps(body, separators=(",", ":")).encode()
        request = urllib.request.Request(self.base_url + path, data=data, method=method,
                                         headers=request_headers)
        expected_statuses = expected if isinstance(expected, tuple) else (expected,)
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                raw = response.read()
                require(response.status in expected_statuses,
                        "%s %s returned %s, expected %s" % (method, path, response.status, expected))
                value = json.loads(raw) if raw else None
                return (response.status, value) if return_status else value
        except urllib.error.HTTPError as error:
            raw = error.read()
            require(error.code in expected_statuses,
                    "%s %s returned %s, expected %s: %s" %
                    (method, path, error.code, expected, raw.decode(errors="replace")))
            if error.code == 408:
                value = raw.decode(errors="replace")
            else:
                value = json.loads(raw) if raw else None
            return (error.code, value) if return_status else value


class Runtime:
    def __init__(self, image, name, network, config, evidence, env, *, loopback_only=False):
        self.image = image
        self.name = name
        self.network = network
        self.config = config
        self.evidence = evidence
        self.env = env
        self.loopback_only = loopback_only
        self.container = None
        self.base_url = None
        self.grpc_target = None

    def start(self):
        args = ["run", "--detach", "--name", self.name, "--network", self.network,
                "--add-host", "host.docker.internal:host-gateway",
                "--volume", str(self.config) + ":/opt/uar/release-config.yaml:ro",
                "--volume", str(ROOT / "scripts/release/uar_1_1_loopback_relay.py") +
                ":/opt/uar/uar_1_1_loopback_relay.py:ro"]
        if not self.loopback_only:
            # Explicit host bindings remain stable across Docker restarts;
            # every protocol client retains the original runtime URL.
            with socket.socket() as http_socket, socket.socket() as grpc_socket:
                http_socket.bind(("127.0.0.1", 0))
                grpc_socket.bind(("127.0.0.1", 0))
                http_port = http_socket.getsockname()[1]
                grpc_port = grpc_socket.getsockname()[1]
            args.extend(["--publish", "127.0.0.1:%d:1906" % http_port,
                         "--publish", "127.0.0.1:%d:50051" % grpc_port])
        for key, value in self.env.items():
            args.extend(["--env", key + "=" + value])
        args.extend([self.image, "universal-agent-runtime", "--config", "/opt/uar/release-config.yaml"])
        self.container = docker(*args).stdout.strip()
        if self.loopback_only:
            self.base_url = "http://127.0.0.1:" + str(HTTP_PORT)
            self.grpc_target = "127.0.0.1:" + str(GRPC_PORT)
        else:
            http_mapping = docker("port", self.name, str(HTTP_PORT) + "/tcp").stdout.strip()
            grpc_mapping = docker("port", self.name, str(GRPC_PORT) + "/tcp").stdout.strip()
            self.base_url = "http://" + http_mapping
            self.grpc_target = grpc_mapping
        try:
            wait_for(self.name + " health", lambda: self._health())
        except BaseException:
            self.stop()
            raise
        return self

    def _health(self):
        state = docker("inspect", "--format", "{{.State.Status}}", self.name).stdout.strip()
        if state == "exited":
            raise RuntimeError(self.name + " exited before health; see retained runtime log")
        if self.loopback_only:
            probe = docker("exec", self.name, "python3", "-c",
                           "import urllib.request; urllib.request.urlopen('http://127.0.0.1:1906/health', timeout=5).read()")
            return probe.stdout
        with urllib.request.urlopen(self.base_url + "/health", timeout=5) as response:
            require(response.status == 200, "runtime health endpoint did not return 200")
            return response.read()

    def restart(self):
        docker("restart", self.name)
        if not self.loopback_only:
            mapping = docker("port", self.name, "1906/tcp").stdout.strip()
            require("http://" + mapping == self.base_url,
                    "explicit runtime HTTP binding changed after restart: " + mapping)
        wait_for(self.name + " restart health", lambda: self._health())

    def logs(self):
        return docker("logs", self.name, check=False).stdout

    def gateway_url(self, port):
        address = docker("exec", self.name, "python3", "-c",
                         "import socket; print(socket.gethostbyname('host.docker.internal'))").stdout.strip()
        try:
            parsed = ipaddress.ip_address(address)
        except ValueError as error:
            raise AcceptanceFailure("Docker host gateway did not resolve to a literal IP address") from error
        require(parsed.is_private or parsed.is_loopback,
                "Docker host gateway is not a fixture-local address")
        return "http://%s:%d" % (address, port)

    def stop(self):
        if self.container:
            logs = docker("logs", self.name, check=False)
            (self.evidence / (self.name + ".log")).write_text(redact_text(logs.stdout + logs.stderr))
            docker("rm", "--force", self.name, check=False)
            self.container = None


class ExternalMcpRelay:
    """Private-network TCP relay to the host peer, with no published host port."""
    def __init__(self, image, network, evidence, peer_port):
        self.image = image
        self.network = network
        self.evidence = evidence
        self.peer_port = peer_port
        self.name = "uar-mcp-relay-" + uuid.uuid4().hex[:12]
        self.container = None
        self.url = None

    def start(self):
        self.container = self.name
        try:
            self.container = docker(
                "run", "--detach", "--name", self.name, "--network", self.network,
                "--add-host", "host.docker.internal:host-gateway",
                "--entrypoint", "python3",
                "--volume", str(ROOT / "scripts/release/uar_1_1_loopback_relay.py") +
                ":/opt/uar/uar_1_1_loopback_relay.py:ro",
                self.image, "/opt/uar/uar_1_1_loopback_relay.py",
                "--listen-host", "0.0.0.0", "--listen-port", str(self.peer_port),
                "--upstream-host", "host.docker.internal", "--upstream-port", str(self.peer_port),
                "--ready-file", "/tmp/uar-mcp-external-relay.ready",
                "--pid-file", "/tmp/uar-mcp-external-relay.pid").stdout.strip()
            wait_for(self.name + " private-network MCP relay", lambda: docker(
                "exec", self.name, "cat", "/tmp/uar-mcp-external-relay.ready").stdout, timeout=30)
            networks = json.loads(docker(
                "inspect", "--format", "{{json .NetworkSettings.Networks}}", self.name).stdout)
            address = ipaddress.IPv4Address(networks[self.network]["IPAddress"])
            # Match Rust IPv4::is_private rather than Python's broader special-use classification.
            require(any(address in ipaddress.IPv4Network(cidr)
                        for cidr in ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16")),
                    "acceptance MCP relay requires an RFC1918 Docker network address")
            self.url = "http://%s:%d" % (address, self.peer_port)
        except BaseException:
            self.stop()
            raise
        return self

    def stop(self):
        if self.container is None:
            return
        # Remove by name even if docker run failed after creating the container.
        logs = docker("logs", self.name, check=False)
        try:
            (self.evidence / (self.name + ".log")).write_text(redact_text(logs.stdout + logs.stderr))
        finally:
            docker("rm", "--force", self.name, check=False)
            self.container = None


class LoopbackRelay:
    """Candidate-namespace TCP relay for real MCP loopback alias coverage."""
    def __init__(self, runtime, upstream_url):
        parsed = urllib.parse.urlsplit(upstream_url)
        require(parsed.scheme == "http" and parsed.hostname and parsed.port,
                "MCP loopback relay requires an explicit HTTP upstream host and port")
        self.runtime = runtime
        self.upstream_host = parsed.hostname
        self.port = parsed.port
        self.ready_file = "/tmp/uar-mcp-loopback-relay-%d.ready" % self.port
        self.pid_file = "/tmp/uar-mcp-loopback-relay-%d.pid" % self.port
        self.url = "http://localhost:%d" % self.port

    def start(self):
        self.stop()
        docker("exec", "-d", self.runtime.name, "python3", "/opt/uar/uar_1_1_loopback_relay.py",
               "--listen-host", "127.0.0.1", "--listen-port", str(self.port),
               "--upstream-host", self.upstream_host,
               "--upstream-port", str(self.port), "--ready-file", self.ready_file,
               "--pid-file", self.pid_file)
        wait_for(self.runtime.name + " loopback MCP relay", lambda: docker(
            "exec", self.runtime.name, "cat", self.ready_file).stdout, timeout=30)

    def restart_runtime(self):
        self.runtime.restart()
        self.start()

    def stop(self):
        pid = docker("exec", self.runtime.name, "cat", self.pid_file, check=False).stdout.strip()
        if pid.isdecimal():
            command = docker(
                "exec", self.runtime.name, "python3", "-c",
                "import pathlib, sys; print(pathlib.Path('/proc', sys.argv[1], 'cmdline').read_bytes().decode(errors='replace'))",
                pid, check=False).stdout
            if "/opt/uar/uar_1_1_loopback_relay.py" in command:
                docker("exec", self.runtime.name, "kill", "-TERM", pid, check=False)
        docker("exec", self.runtime.name, "rm", "-f", self.ready_file, self.pid_file, check=False)


def yaml_config(shared_transports, *, host="0.0.0.0", database="release"):
    return """server:
  host: \"{host}\"
  port: 1906
  grpc_port: 50051
  cors_allowed_origins: []
  mcp_shared_transports_enabled: {shared}
security:
  jwt_required: true
  jwt_secret: \"{secret}\"
  jwt_algorithm: HS256
  jwt_issuer: \"{issuer}\"
  jwt_audience: \"{audience}\"
  jwt_validate_nbf: true
  settings_admin_key: \"release-settings-admin\"
  skill_required_roles:
    {skill}: [\"{role}\"]
  trusted_host_principals:
    - issuer: \"{issuer}\"
      subject: \"{host_subject}\"
      tenant_id: \"{tenant}\"
      host_id: \"{host_id}\"
persistence:
  provider: surreal
  database_url: \"http://surreal:8000\"
  surreal_user: \"{surreal_user}\"
  surreal_pass: \"{surreal_pass}\"
  surreal_ns: \"release\"
  surreal_db: \"{database}\"
  vector_dimension: 384
  external_cache_enabled: false
memory:
  enabled: false
providers: []
native_tools:
  file_tools_enabled: false
  web_fetch_enabled: false
  terminal_exec_enabled: false
  session_search_enabled: false
""".format(host=host, database=database, shared=str(shared_transports).lower(), secret=RELEASE_SECRET,
           issuer=RELEASE_ISSUER, audience=RELEASE_AUDIENCE, skill=PROTECTED_SKILL,
           role=PROTECTED_ROLE, host_subject=HOST_SUBJECT, tenant=TENANT, host_id=HOST_ID,
           surreal_user=SURR_USER, surreal_pass=SURR_PASS)


def agent_artifact(agent_id, preferred_skill, allowed_tools):
    return {
        "version": "1.0.0", "kind": "agent", "id": agent_id,
        "metadata": {"title": agent_id, "description": "release acceptance agent", "tags": ["release"]},
        "runtime": {"entry": "agent", "protocols": {}},
        "policy": {
            "provider": {"default": {"provider": "fixture", "model": "gpt-4o-mini"}, "fallbacks": []},
            "tools": {"allow": list(allowed_tools), "deny": [], "max_concurrent": 1, "execution_mode": "direct"},
            "skills": {"prefer": [preferred_skill], "max_active": 1},
        },
        "schemas": {"inputs": None, "outputs": None, "state": None},
        "prompt": {"system": "Follow the supplied controlled instruction.", "instructions": []},
        "memory": {"conversation": {"enabled": False},
                   "kb": {"enabled": False, "knowledge_bases": [], "citation_required": False}},
        "tools": {"bundles": []},
        "ui": {"forms": {"enabled": False}, "artifacts": {"enabled": False, "preferred_types": []}},
        "extensions": {},
    }


def start_surreal(name, network, evidence):
    container = docker("run", "--detach", "--name", name, "--network", network,
                       "--publish", "127.0.0.1::8000",
                       "--network-alias", "surreal", SURREAL_IMAGE, "start", "--log", "warn",
                       "--user", SURR_USER, "--pass", SURR_PASS, "--bind", "0.0.0.0:8000",
                       "rocksdb:/tmp/release.db").stdout.strip()
    address = docker("port", name, "8000/tcp").stdout.strip()
    def health():
        state = docker("inspect", "--format", "{{.State.Status}}", name).stdout.strip()
        if state == "exited":
            logs = docker("logs", name, check=False)
            (evidence / "surreal-startup.log").write_text(logs.stdout + logs.stderr)
            raise RuntimeError("SurrealDB exited before health; see surreal-startup.log")
        with urllib.request.urlopen("http://" + address + "/health", timeout=5) as response:
            require(response.status == 200, "SurrealDB health did not return 200")
            return response.read()
    wait_for("SurrealDB health", health, timeout=180)
    return container


def start_peer(workdir, expected_authorization_file=None):
    trace = workdir / "mcp-peer.jsonl"
    port_file = workdir / "mcp-peer.port"
    output = (workdir / "mcp-peer.stderr").open("w")
    command = [sys.executable, str(ROOT / "tests/fixtures/dot19_mcp_peer.py"),
               "--transport", "http", "--host", "0.0.0.0", "--port", "0",
               "--port-file", str(port_file), "--trace", str(trace)]
    if expected_authorization_file is not None:
        command.extend(["--expected-authorization-file", str(expected_authorization_file)])
    process = subprocess.Popen(command,
                               cwd=ROOT, stdout=subprocess.DEVNULL, stderr=output, text=True)
    wait_for("MCP peer port", lambda: int(port_file.read_text()), timeout=30)
    return process, output, trace, int(port_file.read_text())


def start_provider(workdir):
    provider_module = load_module("release_dot_provider", ROOT / "scripts/dot/provider.py")
    trace = workdir / "provider.jsonl"
    token = secrets.token_urlsafe(24)
    provider = provider_module.Provider(0, trace, control_token=token, bind_host="0.0.0.0")
    provider.start()
    return provider, trace


def remote_persistence_env():
    """Override the image's embedded SurrealKV environment for remote acceptance."""
    return {
        "UAR_PERSISTENCE__PROVIDER": "surreal",
        "UAR_PERSISTENCE__DATABASE_URL": "http://surreal:8000",
        "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1",
    }


def prepare_grpcurl(evidence):
    """Fetch the pinned official Linux client and retain its verified receipt."""
    machine = platform.machine().lower()
    architectures = {"aarch64": "arm64", "arm64": "arm64", "x86_64": "amd64", "amd64": "amd64"}
    architecture = architectures.get(machine)
    require(architecture is not None, "unsupported grpcurl client architecture: " + machine)
    tools_dir = evidence / "tools"
    tools_dir.mkdir(exist_ok=True)
    checksums = tools_dir / ("grpcurl_" + GRPCURL_VERSION + "_checksums.txt")
    archive_name = "grpcurl_%s_linux_%s.tar.gz" % (GRPCURL_VERSION, architecture)
    archive = tools_dir / archive_name
    binary = tools_dir / "grpcurl"
    with urllib.request.urlopen(GRPCURL_CHECKSUMS_URL, timeout=120) as response:
        checksums.write_bytes(response.read())
    observed_checksums_digest = hashlib.sha256(checksums.read_bytes()).hexdigest()
    require(observed_checksums_digest == GRPCURL_CHECKSUMS_SHA256,
            "official grpcurl checksum manifest did not match its pinned sha256")
    expected_archive_digest = None
    for line in checksums.read_text(encoding="utf-8").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1].lstrip("*") == archive_name:
            expected_archive_digest = fields[0]
            break
    require(expected_archive_digest is not None,
            "pinned grpcurl checksum manifest omitted the required Linux archive")
    archive_url = (
        "https://github.com/fullstorydev/grpcurl/releases/download/v%s/%s"
        % (GRPCURL_VERSION, archive_name)
    )
    with urllib.request.urlopen(archive_url, timeout=120) as response:
        archive.write_bytes(response.read())
    observed_archive_digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    require(observed_archive_digest == expected_archive_digest,
            "grpcurl archive did not match the official checksum manifest")
    with tarfile.open(archive, "r:gz") as package:
        member = package.getmember("grpcurl")
        require(member.isfile(), "pinned grpcurl archive has no executable grpcurl member")
        source = package.extractfile(member)
        require(source is not None, "pinned grpcurl archive could not read grpcurl member")
        binary.write_bytes(source.read())
    os.chmod(binary, 0o755)
    receipt = {
        "version": GRPCURL_VERSION,
        "architecture": architecture,
        "checksums_url": GRPCURL_CHECKSUMS_URL,
        "checksums_sha256": observed_checksums_digest,
        "archive_url": archive_url,
        "archive_sha256": observed_archive_digest,
        "binary": str(binary),
    }
    (tools_dir / "grpcurl-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    return binary


def restart_surreal_fault(surreal_name, runtime, client, skill_id):
    docker("stop", surreal_name)
    failure_body = {"skill_markdown": "---\nname: Persistence Fault Skill\nversion: 1\ndescription: fault\n---\nbody\n"}
    status, failed = client.request("POST", "/api/uar/skills/import-content", failure_body,
                                    expected=(408, 503), return_status=True)
    if status == 503:
        require(failed["error"] in ("skill_persistence_unavailable", "skill_persistence_failed"),
                "real persistence outage returned an unrelated import failure")
    else:
        require(failed == "Request timed out", "outage timeout returned an unrelated failure")
    docker("start", surreal_name)
    address = docker("port", surreal_name, "8000/tcp").stdout.strip()
    def restored_health():
        with urllib.request.urlopen("http://" + address + "/health", timeout=5) as response:
            require(response.status == 200, "restored SurrealDB health did not return 200")
            return response.read()
    wait_for("restored SurrealDB", restored_health, timeout=180)
    client.request("GET", "/api/uar/skills/persistence-fault-skill", expected=404)
    client.request("GET", "/api/uar/skills/" + skill_id, expected=200)
    return {"fault_status": status, "unpublished_skill_id": "persistence-fault-skill"}


def configure_runtime(runtime, peer_port, owner_token, host_token, grant_authorization, *, peer_url=None):
    owner = Http(runtime.base_url, owner_token, "release-settings-admin")
    host = Http(runtime.base_url, host_token, "release-settings-admin")
    suffix = uuid.uuid4().hex
    agent_a = "release-agent-a-" + suffix
    agent_b = "release-agent-b-" + suffix
    owner.request("POST", "/api/agents", agent_artifact(
        agent_a, PROTECTED_SKILL, ["qualification__elicit"]), expected=201)
    owner.request("POST", "/api/agents", agent_artifact(
        agent_b, PROTECTED_SKILL, ["qualification__elicit"]), expected=201)
    peer_url = peer_url or runtime.gateway_url(peer_port)
    mcp = {
        "name": "qualification", "enabled": True, "transport": "remote_http", "url": peer_url,
        "env": {}, "headers": {}, "sandboxed": False,
        "grant_policy": {"destination_id": "qualification", "trusted_hosts": [HOST_ID],
                         "required_scopes": ["tools:invoke"], "allow_private_http": True,
                         "allowed_agent_ids": [agent_a]},
    }
    host.request("PUT", "/api/uar/mcp/servers/qualification", mcp, expected=200)
    protected_markdown = (ROOT / "tests/fixtures/skill-upload/round-trip.SKILL.md").read_text()
    protected_markdown = protected_markdown.replace("name: Remote Receipt", "name: Release Protected Skill", 1)
    protected_markdown = protected_markdown.replace("  - records.read", "  - qualification__elicit", 1)
    imported = owner.request("POST", "/api/uar/skills/import-content",
                             {"skill_markdown": protected_markdown}, expected=201)
    require(imported["skill_id"] == PROTECTED_SKILL, "protected release skill derived an unexpected ID")
    owner.request("PUT", "/api/uar/agents/" + agent_a + "/skills",
                  {"skill_ids": [PROTECTED_SKILL]}, expected=200)
    owner.request("PUT", "/api/uar/agents/" + agent_b + "/skills",
                  {"skill_ids": [PROTECTED_SKILL]}, expected=200)
    grant = {"name": "qualification", "grant": {"credential_revision": "v1",
             "scopes": ["tools:invoke"], "headers": {"Authorization": grant_authorization},
             "expires_at_unix": int(time.time()) + 3600}}
    return owner, host, agent_a, agent_b, grant, mcp


def set_governance(client, enabled):
    settings = client.request("GET", "/api/uar/settings/governance", expected=200)
    row = next((item for item in settings if item["key"] == "governance.enabled"), None)
    require(row is not None, "governance.enabled setting is absent")
    updated = client.request("PUT", "/api/uar/settings/governance", {
        "data": {"enabled": enabled}, "expected_revisions": {"enabled": row["revision"]},
    }, expected=200)
    require(updated["status"] == "updated" and
            updated["governance_status"]["effective_enabled"] is enabled,
            "governance setting did not publish the requested runtime posture")


def configure_a2a_runtime(runtime, peer_port, owner_token, host_token):
    owner = Http(runtime.base_url, owner_token, "release-settings-admin")
    host = Http(runtime.base_url, host_token, "release-settings-admin")
    suffix = uuid.uuid4().hex
    protected_agent = "release-a2a-protected-" + suffix
    dynamic_agent = "release-a2a-dynamic-" + suffix
    baseline_skill = "release-baseline-skill"
    owner.request("POST", "/api/agents", agent_artifact(
        protected_agent, PROTECTED_SKILL, ["a2a-qualification__effect"]), expected=201)
    owner.request("POST", "/api/agents", agent_artifact(
        dynamic_agent, baseline_skill, ["activate_skill"]), expected=201)
    peer_url = runtime.gateway_url(peer_port)
    host.request("PUT", "/api/uar/mcp/servers/a2a-qualification", {
        "name": "a2a-qualification", "enabled": True, "transport": "remote_http", "url": peer_url,
        "env": {}, "headers": {}, "sandboxed": False,
    }, expected=200)
    source = (ROOT / "tests/fixtures/skill-upload/round-trip.SKILL.md").read_text()
    protected = source.replace("name: Remote Receipt", "name: Release Protected Skill", 1)
    protected = protected.replace("  - records.read", "  - a2a-qualification__effect", 1)
    protected = protected.replace("# Remote receipt", "# UAR_PROTECTED_PROMPT_BODY_379", 1)
    baseline = source.replace("name: Remote Receipt", "name: Release Baseline Skill", 1)
    imported_protected = owner.request(
        "POST", "/api/uar/skills/import-content", {"skill_markdown": protected}, expected=201)
    require(imported_protected["skill_id"] == PROTECTED_SKILL,
            "protected A2A skill derived an unexpected ID")
    imported_baseline = owner.request(
        "POST", "/api/uar/skills/import-content", {"skill_markdown": baseline}, expected=201)
    require(imported_baseline["skill_id"] == baseline_skill,
            "baseline A2A skill derived an unexpected ID")
    owner.request("PUT", "/api/uar/agents/" + protected_agent + "/skills",
                  {"skill_ids": [PROTECTED_SKILL]}, expected=200)
    owner.request("PUT", "/api/uar/agents/" + dynamic_agent + "/skills",
                  {"skill_ids": [baseline_skill, PROTECTED_SKILL]}, expected=200)
    return protected_agent, dynamic_agent


def run_loopback_a2a_tuple(image, runtime, evidence, grpcurl, *, governance, transport,
                           decision, peer_port, peer_trace, provider_trace,
                           trusted_token, limited_token, host_token, receipt_path):
    result_path = receipt_path.with_suffix(".client.json")
    client_tokens = {
        "UAR_RELEASE_TRUSTED_TOKEN": trusted_token,
        "UAR_RELEASE_LIMITED_TOKEN": limited_token,
        "UAR_RELEASE_HOST_TOKEN": host_token,
    }
    previous_tokens = {key: os.environ.get(key) for key in client_tokens}
    command = [
        "run", "--rm", "--network", "container:" + runtime.name,
        "--user", "%d:%d" % (os.getuid(), os.getgid()),
        "--entrypoint", "python3",
        "--volume", str(ROOT) + ":/acceptance:ro",
        "--volume", str(evidence) + ":" + str(evidence),
        "--volume", str(grpcurl) + ":/tools/grpcurl:ro",
        "--env", "UAR_RELEASE_TRUSTED_TOKEN",
        "--env", "UAR_RELEASE_LIMITED_TOKEN",
        "--env", "UAR_RELEASE_HOST_TOKEN",
        image,
        "/acceptance/scripts/release/uar_1_1_loopback_client.py",
        "--governance", governance,
        "--transport", transport,
        "--decision", decision,
        "--peer-port", str(peer_port),
        "--peer-url", runtime.gateway_url(peer_port),
        "--peer-trace", str(peer_trace),
        "--provider-trace", str(provider_trace),
        "--receipt", str(receipt_path),
        "--result", str(result_path),
    ]
    try:
        os.environ.update(client_tokens)
        docker(*command)
    finally:
        for key, value in previous_tokens.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value
    return json.loads(result_path.read_text())


def run_a2a_profiles(image, network, workdir, evidence):
    trusted_token = jwt(OWNER_SUBJECT, ["admin", PROTECTED_ROLE])
    limited_token = jwt(OWNER_SUBJECT, ["admin"])
    host_token = jwt(HOST_SUBJECT, ["admin", PROTECTED_ROLE], HOST_ID)
    grpcurl = prepare_grpcurl(evidence)
    results = []
    expected_tuples = {
        "%s-%s-%s" % (governance, transport, decision)
        for governance in ("enforced", "disabled")
        for transport in ("jsonrpc", "grpc")
        for decision in ("approve", "deny")
    }
    for governance in ("enforced", "disabled"):
        for transport in ("jsonrpc", "grpc"):
            for decision in ("approve", "deny"):
                tuple_id = "%s-%s-%s" % (governance, transport, decision)
                tuple_dir = workdir / ("a2a-" + tuple_id)
                tuple_dir.mkdir()
                provider, provider_trace = start_provider(tuple_dir)
                peer, peer_output, peer_trace, peer_port = start_peer(tuple_dir)
                config = tuple_dir / "runtime.yaml"
                config.write_text(yaml_config(True, host="127.0.0.1", database="a2a-" + tuple_id))
                runtime_env = remote_persistence_env()
                if governance == "disabled":
                    # Governance may be disabled only in an optional-JWT,
                    # loopback-only test process. Supplied tokens still verify;
                    # all shipped configurations continue to require JWTs.
                    runtime_env["UAR_SECURITY__JWT_REQUIRED"] = "false"
                runtime_env.update({
                    "LLM_MODEL": "fixture/gpt-4o-mini",
                    "LLM_API_KEY": "fixture-provider-key",
                    "LLM_BASE_URL": "http://host.docker.internal:%d/v1" % provider.server.server_port,
                    "LLM_PROTOCOL": "chat",
                })
                runtime = Runtime(image, "uar-a2a-" + uuid.uuid4().hex[:12], network, config, evidence,
                                  runtime_env, loopback_only=True).start()
                try:
                    receipt_path = evidence / ("a2a-" + tuple_id + ".json")
                    result = run_loopback_a2a_tuple(
                        image, runtime, evidence, grpcurl, governance=governance,
                        transport=transport, decision=decision, peer_port=peer_port,
                        peer_trace=peer_trace, provider_trace=provider_trace,
                        trusted_token=trusted_token, limited_token=limited_token,
                        host_token=host_token, receipt_path=receipt_path,
                    )
                    results.append({"tuple": tuple_id, "receipt": result["receipt"],
                                    "provider_trace": str(provider_trace), "peer_trace": str(peer_trace),
                                    "shared_transports_enabled": True,
                                    "client": "candidate-image-network-namespace"})
                finally:
                    runtime.stop()
                    peer.terminate()
                    peer.wait(timeout=30)
                    peer_output.close()
                    provider.stop()
    require({result["tuple"] for result in results} == expected_tuples,
            "A2A acceptance did not complete every transport/governance/decision tuple")
    return results


def safe_default_profile(image, evidence):
    defaults = load_module("release_default_profile", ROOT / "scripts/release/uar_1_1_defaults.py")
    return defaults.run_default_profile(
        image,
        evidence,
        docker=docker,
        require=require,
        wait_for=wait_for,
    )


def shipped_profile_checks(image, evidence, network):
    profiles = load_module("release_shipped_profiles", ROOT / "scripts/release/uar_1_1_profiles.py")
    return profiles.run_shipped_profile_checks(
        image,
        evidence,
        docker=docker,
        require=require,
        wait_for=wait_for,
        network=network,
        surreal_password=SURR_PASS,
    )


def run_mcp_profiles(image, network, workdir, evidence, surreal_name):
    provider, provider_trace = start_provider(workdir)
    private_dir = Path(tempfile.mkdtemp(prefix="uar-1.1.0-mcp-credentials-"))
    authorization = "Bearer " + secrets.token_urlsafe(32)
    expected_authorization = private_dir / "peer-authorization"
    expected_authorization.write_text(authorization)
    os.chmod(expected_authorization, 0o600)
    peer, peer_output, peer_trace, peer_port = start_peer(workdir, expected_authorization)
    results = []
    external_relay = ExternalMcpRelay(image, network, evidence, peer_port)
    try:
        external_relay.start()
        skill_fixture = load_module("release_skill_import", ROOT / "tests/fixtures/skill_remote_import.py")
        mcp_fixture = load_module("release_mcp_contract", ROOT / "tests/fixtures/uar_1_1_mcp_contract.py")
        prepare_agui = load_module("release_prepare_agui", ROOT / "scripts/release/prepare_agui.py")
        schema_module = prepare_agui.prepare(evidence / "agui-schema")
        owner_token = jwt(OWNER_SUBJECT, ["admin", PROTECTED_ROLE])
        limited_token = jwt(OWNER_SUBJECT, ["admin"])
        host_token = jwt(HOST_SUBJECT, ["admin", PROTECTED_ROLE], HOST_ID)
        for shared in (False, True):
            config = workdir / ("runtime-shared-%s.yaml" % str(shared).lower())
            database = "mcp-" + ("shared" if shared else "catalog")
            config.write_text(yaml_config(shared, database=database))
            runtime_env = remote_persistence_env()
            runtime_env.update({
                "LLM_MODEL": "fixture/gpt-4o-mini",
                "LLM_API_KEY": "fixture-provider-key",
                "LLM_BASE_URL": "http://host.docker.internal:%d/v1" % provider.server.server_port,
                "LLM_PROTOCOL": "chat",
            })
            runtime = Runtime(image, "uar-release-" + uuid.uuid4().hex[:12], network, config, evidence,
                              runtime_env).start()
            relay = None
            try:
                owner, host, agent_a, agent_b, grant, destination = configure_runtime(
                    runtime, peer_port, owner_token, host_token, authorization,
                    peer_url=external_relay.url)
                skill_receipt = skill_fixture.exercise(runtime.base_url,
                    lambda method, url, body, status: Http(runtime.base_url, owner_token,
                        "release-settings-admin").request(method, url.removeprefix(runtime.base_url), body, status),
                    runtime.restart)
                fault = restart_surreal_fault(surreal_name, runtime, owner, skill_receipt["skill_id"])
                def validate_event(event):
                    completed = subprocess.run(["node", str(ROOT / "scripts/release/validate_agui.mjs")],
                        input=json.dumps(event) + "\n", text=True, capture_output=True,
                        env=dict(os.environ, UAR_AGUI_SCHEMA_MODULE=str(schema_module)))
                    require(completed.returncode == 0, "pinned AG-UI event schema rejected frame: " + completed.stderr)
                external_receipt = mcp_fixture.run_profile(runtime.base_url,
                    {"Authorization": "Bearer " + host_token}, agent_a, agent_b, grant, peer_trace,
                    shared, validate_event, restart_runtime=runtime.restart,
                    admin_request=lambda method, path, body, status: host.request(method, path, body, status),
                    destination=destination)
                relay = LoopbackRelay(runtime, external_relay.url)
                relay.start()
                loopback_destination = json.loads(json.dumps(destination))
                loopback_destination["url"] = relay.url
                host.request("PUT", "/api/uar/mcp/servers/qualification", loopback_destination, 200)
                loopback_receipt = mcp_fixture.run_profile(runtime.base_url,
                    {"Authorization": "Bearer " + host_token}, agent_a, agent_b, grant, peer_trace,
                    shared, validate_event, restart_runtime=relay.restart_runtime,
                    admin_request=lambda method, path, body, status: host.request(method, path, body, status),
                    destination=loopback_destination)
                host.request("PUT", "/api/uar/mcp/servers/qualification", destination, 200)
                common = {"shared_transports_enabled": shared, "runtime": runtime.name,
                          "skill": skill_receipt, "persistence_fault": fault,
                          "provider_trace": str(provider_trace), "mcp_trace": str(peer_trace),
                          "external_relay": external_relay.name,
                          "role_limited_token_configured": bool(limited_token)}
                results.append(dict(common, destination_topology="external-private-network-relay",
                                    coverage="remote-grant-and-full-elicitation",
                                    destination_url=destination["url"], mcp=external_receipt))
                results.append(dict(common, destination_topology="runtime-loopback-relay",
                                    coverage="loopback-alias-and-full-elicitation",
                                    destination_url=loopback_destination["url"], mcp=loopback_receipt,
                                    restored_destination_url=destination["url"]))
            finally:
                try:
                    if relay is not None:
                        relay.stop()
                finally:
                    runtime.stop()
    finally:
        try:
            external_relay.stop()
        finally:
            peer.terminate()
            peer.wait(timeout=30)
            peer_output.close()
            provider.stop()
            shutil.rmtree(private_dir)
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--image", required=True, help="immutable published digest or explicitly allowed local tag")
    parser.add_argument("--local-tag", action="store_true", help="allow --image to be a local non-digest tag")
    parser.add_argument("--evidence", required=True, type=Path, help="empty or new evidence directory")
    parser.add_argument("--keep", action="store_true", help="retain Docker resources when an acceptance failure occurs")
    parser.add_argument("--gate", action="append", choices=("all", "defaults", "profiles", "mcp", "a2a"),
                        help="run one acceptance gate; omit for the complete release program")
    args = parser.parse_args()
    require("@sha256:" in args.image or args.local_tag,
            "--image must be immutable by digest; pass --local-tag only for a local candidate")
    require(shutil.which("docker") is not None, "Docker is required for installed-image acceptance")
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=False)
    os.chmod(evidence, 0o755)
    requested_gates = args.gate or ["all"]
    require(not ("all" in requested_gates and len(requested_gates) != 1),
            "--gate all cannot be combined with a partial gate")
    selected_gates = ("defaults", "profiles", "mcp", "a2a") if requested_gates == ["all"] else tuple(requested_gates)
    receipt = {"image": args.image, "image_kind": "local-tag" if args.local_tag else "immutable-digest",
               "started_at": int(time.time()), "result": "incomplete", "requested_gates": requested_gates,
               "selected_gates": list(selected_gates), "full_acceptance": requested_gates == ["all"]}
    network = "uar-release-" + uuid.uuid4().hex[:12]
    surreal_name = "surreal-release-" + uuid.uuid4().hex[:12]
    resources = []
    try:
        if "defaults" in selected_gates:
            receipt["safe_default"] = safe_default_profile(args.image, evidence)
        if any(gate in selected_gates for gate in ("profiles", "mcp", "a2a")):
            docker("network", "create", network)
            resources.append(("network", network))
            resources.append(("container", surreal_name))
            start_surreal(surreal_name, network, evidence)
        if "profiles" in selected_gates:
            receipt["shipped_profiles"] = shipped_profile_checks(args.image, evidence, network)
        if "mcp" in selected_gates or "a2a" in selected_gates:
            workdir = evidence / "work"
            workdir.mkdir()
            os.chmod(workdir, 0o755)
        if "mcp" in selected_gates:
            receipt["profiles"] = run_mcp_profiles(args.image, network, workdir, evidence, surreal_name)
        if "a2a" in selected_gates:
            receipt["a2a_profiles"] = run_a2a_profiles(args.image, network, workdir, evidence)
        receipt["result"] = "passed" if requested_gates == ["all"] else "partial-passed"
    except BaseException as error:
        receipt["failure"] = str(error)
        raise
    finally:
        receipt["finished_at"] = int(time.time())
        (evidence / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
        if not args.keep:
            for kind, value in reversed(resources):
                if kind == "container":
                    docker("rm", "--force", value, check=False)
                else:
                    docker("network", "rm", value, check=False)


if __name__ == "__main__":
    main()
