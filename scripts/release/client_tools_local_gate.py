#!/usr/bin/env python3
"""One explicitly authorized local caller-tool gate, including owned lifecycle.

Uses an already built binary. All generated state stays under the repository's
ignored target directory. No import starts a process or accesses the network.
"""
import argparse
import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
PROVIDER_ID = "fixture"
MODEL_ID = "gpt-4o-mini"
MCP_SERVER = "acceptance_peer"
ISSUER = "uar-client-tools-local-gate"
AUDIENCE = "uar-client-tools-candidate"
TENANT = "client-tools-local-tenant"
STOP_SIGNALS = []


class GateFailure(RuntimeError):
    pass


def require(condition, message):
    if not condition:
        raise GateFailure(message)


def private_json(path, value):
    with open(path, "x", opener=lambda name, flags: os.open(name, flags, 0o600)) as output:
        json.dump(value, output, indent=2)
        output.write("\n")


def jwt(secret, subject, roles):
    def encoded(value):
        return base64.urlsafe_b64encode(json.dumps(value, separators=(",", ":")).encode()).rstrip(b"=").decode()
    now = int(time.time())
    header = encoded({"alg": "HS256", "typ": "JWT"})
    claims = encoded({"sub": subject, "name": subject, "roles": roles, "tenant_id": TENANT,
        "uar_credential_kind": "issuer", "iss": ISSUER, "aud": AUDIENCE, "nbf": now - 5, "exp": now + 21600})
    data = (header + "." + claims).encode()
    signature = base64.urlsafe_b64encode(hmac.new(secret.encode(), data, hashlib.sha256).digest()).rstrip(b"=").decode()
    return data.decode() + "." + signature


def clean_environment(evidence):
    # Allowlist rather than inheriting provider credentials, config selectors,
    # proxy settings, launch authority or persistence URLs from the operator.
    environment = {name: os.environ[name] for name in ("PATH", "LANG", "LC_ALL", "SYSTEMROOT") if name in os.environ}
    environment.update({"HOME": str(evidence / "home"), "TMPDIR": str(evidence / "tmp"), "RUST_LOG": "info", "UAR_LOCAL_ONLY": "true"})
    return environment


def available_port():
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", 0))
        return reservation.getsockname()[1]


def http(base, path, token=None, body=None, expected=200):
    require(not STOP_SIGNALS, "operator requested shutdown")
    headers = {"Accept": "application/json"}
    if token:
        headers["Authorization"] = "Bearer " + token
    if body is not None:
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(base + path, data=None if body is None else json.dumps(body).encode(), headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            status, raw = response.status, response.read()
    except urllib.error.HTTPError as error:
        status, raw = error.code, error.read()
    require(status == expected, "%s returned HTTP %s, expected %s" % (path, status, expected))
    return json.loads(raw) if raw else None


def wait_for(label, child, probe, seconds=180):
    deadline, last = time.monotonic() + seconds, None
    while time.monotonic() < deadline:
        require(not STOP_SIGNALS, "operator requested shutdown")
        require(child.poll() is None, label + " process exited before readiness; inspect its retained log")
        try:
            value = probe()
            if value is not None:
                return value
        except (GateFailure, OSError, ValueError, urllib.error.URLError) as error:
            last = type(error).__name__ + ": " + str(error)
        time.sleep(0.2)
    raise GateFailure(label + " readiness deadline exceeded: " + str(last))


def artifact(agent_id, actual_tool):
    return {
        "version": "1.0.0", "kind": "agent", "id": agent_id,
        "metadata": {"title": agent_id, "description": "Isolated caller-tool production HTTP acceptance", "tags": ["acceptance"]},
        "runtime": {"entry": "agent", "protocols": {}},
        "policy": {"provider": {"default": {"provider": PROVIDER_ID, "model": MODEL_ID}, "fallbacks": []},
            "tools": {"allow": [actual_tool], "deny": [], "max_concurrent": 2, "execution_mode": "direct"},
            "skills": {"prefer": [], "max_active": 1}},
        "schemas": {"inputs": None, "outputs": None, "state": None},
        "prompt": {"system": "Follow the controlled acceptance provider. Caller functions belong to the application. Execute only the explicitly admitted safe MCP effect when requested.", "instructions": []},
        "memory": {"conversation": {"enabled": False}, "kb": {"enabled": False, "knowledge_bases": [], "citation_required": False}},
        "tools": {"bundles": []},
        "ui": {"forms": {"enabled": False}, "artifacts": {"enabled": False, "preferred_types": []}},
        "extensions": {},
    }


def configuration(evidence, http_port, grpc_port, peer_url, secret):
    # JSON is valid YAML; the .yaml suffix selects the source-supported YAML
    # config loader without introducing an external YAML package dependency.
    return {
        "server": {"host": "127.0.0.1", "port": http_port, "grpc_port": grpc_port,
            "cors_allowed_origins": [], "mcp_shared_transports_enabled": True},
        "security": {"jwt_required": True, "jwt_secret": secret, "jwt_algorithm": "HS256",
            "jwt_issuer": ISSUER, "jwt_audience": AUDIENCE, "jwt_validate_nbf": True,
            "settings_admin_key": secrets.token_urlsafe(32)},
        "persistence": {"provider": "surreal", "database_url": "surrealkv://" + str(evidence / "store" / "uar.db"),
            "surreal_ns": "client_tools_gate", "surreal_db": "client_tools_gate", "vector_dimension": 384, "external_cache_enabled": False},
        "memory": {"enabled": False, "db_path": str(evidence / "store" / "memory.db")},
        "llm": {"model": PROVIDER_ID + "/" + MODEL_ID, "base_url": peer_url + "/v1", "api_key": "local-fixture-only",
            "protocol": "chat", "max_retries": 0, "timeout_secs": 90,
            "embedding": {"backend": "openai", "model": "text-embedding-3-small", "base_url": peer_url + "/v1",
                "api_key": "local-fixture-only", "vector_dimension": 384, "models_dir": str(evidence / "models")}},
        "providers": [{"id": PROVIDER_ID, "display_name": "Isolated loopback fixture", "base_url": peer_url + "/v1",
            "protocol": "chat", "api_key": "local-fixture-only", "default_model": MODEL_ID, "enabled": True,
            "models": [{"id": MODEL_ID, "supports_tools": True, "supports_streaming": True, "enabled": True}]}],
        "native_tools": {"file_tools_enabled": False, "web_fetch_enabled": False, "terminal_exec_enabled": False, "session_search_enabled": False},
        "acp": {"enabled": True, "path": "/acp", "auth_required": True},
    }


class Supervisor:
    def __init__(self, evidence, environment):
        self.evidence, self.environment = evidence, environment
        self.children, self.logs, self.receipts = [], [], []

    def start(self, name, command, extra_environment=None):
        require(not STOP_SIGNALS, "operator requested shutdown")
        log = open(self.evidence / (name + ".log"), "xb", opener=lambda name, flags: os.open(name, flags, 0o600))
        self.logs.append(log)
        child = subprocess.Popen([str(part) for part in command], cwd=self.evidence,
            env={**self.environment, **(extra_environment or {})}, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        self.children.append((name, child))
        self.receipts.append({"process": name, "pid": child.pid, "command": [str(part) for part in command]})
        return child

    def stop_join(self):
        clean = True
        for name, child in reversed(self.children):
            errors, joined, forced, kill_attempted = [], False, False, False
            try:
                child.send_signal(signal.SIGTERM)
            except OSError as error:
                errors.append("terminate: " + type(error).__name__ + ": " + str(error))
            try:
                child.wait(timeout=45)
                joined = True
            except (OSError, subprocess.TimeoutExpired) as error:
                errors.append("graceful join: " + type(error).__name__ + ": " + str(error))
            if not joined:
                kill_attempted = True
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                    forced = True
                except OSError as error:
                    errors.append("kill: " + type(error).__name__ + ": " + str(error))
                try:
                    child.wait(timeout=15)
                    joined = True
                except (OSError, subprocess.TimeoutExpired) as error:
                    errors.append("final join: " + type(error).__name__ + ": " + str(error))
            expected = (0, -signal.SIGTERM) if name == "provider-peer" else (0, 2) if name == "matrix" else (0,)
            accepted = joined and child.returncode in expected and not kill_attempted and not errors
            clean = clean and accepted
            self.receipts.append({"process": name, "pid": child.pid, "joined": joined,
                "exit_code": child.returncode if joined else None, "forced_kill_attempted": kill_attempted,
                "forced_kill": forced, "clean_shutdown": accepted, "cleanup_errors": errors})
        for log in self.logs:
            try:
                log.close()
            except OSError as error:
                clean = False
                self.receipts.append({"log": str(log.name), "closed": False,
                    "cleanup_error": type(error).__name__ + ": " + str(error)})
        return clean


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/uar-post-lifecycle-main/debug/universal-agent-runtime")
    parser.add_argument("--evidence", type=Path, help="Fresh directory under this repository's ignored target/ tree")
    parser.add_argument("--matrix-timeout-secs", type=int, default=2700)
    args = parser.parse_args()
    os.umask(0o077)
    evidence = (args.evidence or ROOT / "target" / ("client-tools-local-gate-" + str(uuid.uuid4()))).resolve()
    require(evidence.is_relative_to((ROOT / "target").resolve()), "evidence must be under repository target/")
    require(not evidence.exists(), "evidence directory must be fresh; existing private state is never reused")
    evidence.mkdir(parents=True, mode=0o700)
    for directory in ("tmp", "store", "models", "home"):
        (evidence / directory).mkdir(mode=0o700)
    binary = args.binary.resolve()
    environment = clean_environment(evidence)
    supervisor = Supervisor(evidence, environment)
    def interrupted(signum, _frame):
        # Record only: raising here can abandon finally during a second signal.
        STOP_SIGNALS.append(signum)
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    report = {"status": "failed", "evidence": str(evidence), "auth_enabled": True,
        "persistence": "fresh embedded SurrealKV", "external_gate_inputs_remaining": [
            "Real cold restart modes retain the same store and provider peer and need separate explicit gate authorization.",
            "Graph cursor modes require a configured real graph agent with before/after node prompt markers.",
            "Budget and cold budget modes require a configured real cumulative-budget agent."]}
    exit_code = 1
    try:
        require(binary.is_file() and os.access(binary, os.X_OK), "prebuilt candidate binary is missing or not executable")
        secret = secrets.token_urlsafe(48)
        owner_token = jwt(secret, "client-tools-gate-admin-" + uuid.uuid4().hex, ["admin"])
        other_token = jwt(secret, "client-tools-gate-other-" + uuid.uuid4().hex, [])
        private_json(evidence / "credentials.json", {"owner": owner_token, "other": other_token})
        port_file = evidence / "provider.port"
        peer = supervisor.start("provider-peer", [sys.executable, ROOT / "tests/fixtures/client_tools_provider.py",
            "--host", "127.0.0.1", "--port", "0", "--port-file", port_file, "--trace", evidence / "provider-trace.jsonl"])
        peer_port = wait_for("provider-peer", peer, lambda: int(port_file.read_text()) if port_file.exists() else None, seconds=30)
        peer_url = "http://127.0.0.1:" + str(peer_port)
        wait_for("provider HTTP", peer, lambda: http(peer_url, "/_trace"), seconds=30)
        http_port, grpc_port = available_port(), available_port()
        require(http_port != grpc_port, "ephemeral listener allocation collided; no candidate was started")
        private_json(evidence / "config.yaml", configuration(evidence, http_port, grpc_port, peer_url, secret))
        private_json(evidence / "mcp.json", {"mcpServers": {MCP_SERVER: {"url": peer_url + "/mcp", "preapproved_tools": ["effect"]}}})
        candidate = supervisor.start("candidate", [binary, "--config", evidence / "config.yaml"],
            {"MCP_CONFIG_PATH": str(evidence / "mcp.json")})
        base = "http://127.0.0.1:" + str(http_port)
        wait_for("candidate authenticated readiness", candidate, lambda: http(base, "/readyz", owner_token))
        tools = wait_for("configured real MCP tool", candidate, lambda: http(base, "/api/tools", owner_token))
        matches = [entry for entry in tools.get("tools", []) if entry.get("source") == "mcp_server"
            and entry.get("server") == MCP_SERVER and entry.get("name") == "effect"]
        require(len(matches) == 1, "registry did not expose exactly one configured MCP effect; no namespace is guessed")
        actual_tool = matches[0]["namespaced_name"]
        private_json(evidence / "tools-catalog.json", tools)
        private_json(evidence / "host-tool.json", {"name": actual_tool, "arguments": {"query": "safe-local-acceptance-effect"}})
        agent_id = "client-tools-local-" + uuid.uuid4().hex
        agent = artifact(agent_id, actual_tool)
        saved = http(base, "/api/agents", owner_token, agent, expected=201)
        require(saved.get("id") == agent_id, "persisted agent receipt changed selected ID")
        private_json(evidence / "agent.json", saved)
        # Verify auth is actually required; a liveness endpoint alone proves none.
        http(base, "/api/tools", expected=401)
        matrix_command = [sys.executable, ROOT / "scripts/release/client_tools_acceptance.py",
            "--base-url", base, "--provider-base-url", peer_url,
            "--bearer-token-env", "UAR_CLIENT_TOOLS_GATE_OWNER", "--other-bearer-token-env", "UAR_CLIENT_TOOLS_GATE_OTHER",
            "--agent-id", agent_id, "--openai-model", agent_id, "--anthropic-model", PROVIDER_ID + "/" + MODEL_ID,
            "--protocols", "openai", "anthropic", "agui", "a2a", "acp", "--acp-path", "/acp",
            "--host-tool-json", evidence / "host-tool.json", "--mode", "matrix", "--report", evidence / "matrix.json"]
        matrix = supervisor.start("matrix", matrix_command,
            {"UAR_CLIENT_TOOLS_GATE_OWNER": owner_token, "UAR_CLIENT_TOOLS_GATE_OTHER": other_token})
        matrix_deadline = time.monotonic() + args.matrix_timeout_secs
        while matrix.poll() is None:
            require(not STOP_SIGNALS, "operator requested shutdown")
            require(time.monotonic() < matrix_deadline, "matrix deadline exceeded")
            time.sleep(0.2)
        require(not STOP_SIGNALS, "operator requested shutdown")
        require((evidence / "matrix.json").is_file(), "matrix exited without its evidence report")
        matrix_report = json.loads((evidence / "matrix.json").read_text())
        results = matrix_report.get("results", [])
        require(results and all(item.get("status") == "passed" for item in results), "one or more actual matrix cases failed; inspect matrix.json and retained logs")
        require(matrix.returncode in (0, 2), "matrix returned an unexpected exit status")
        report.update(status="matrix_passed_remaining_gates", provider_base_url=peer_url, candidate_base_url=base,
            agent_id=agent_id, advertised_host_tool=actual_tool, matrix_exit_code=matrix.returncode,
            matrix_cases=len(results), matrix_coverage_gaps=matrix_report.get("coverage_gaps", []))
        exit_code = 2
    except (GateFailure, OSError, ValueError, KeyError, subprocess.TimeoutExpired, urllib.error.URLError, KeyboardInterrupt) as error:
        # Never include child environment/config/credential contents in diagnostics.
        report["failure"] = type(error).__name__ + ": " + str(error)
    finally:
        try:
            report["owned_shutdown_clean"] = supervisor.stop_join()
            if not report["owned_shutdown_clean"]:
                report["status"] = "failed_cleanup"
                exit_code = 1
        except (OSError, subprocess.TimeoutExpired) as error:
            report["status"] = "failed_cleanup"
            report["cleanup_failure"] = type(error).__name__ + ": " + str(error)
            exit_code = 1
        if STOP_SIGNALS:
            report["received_shutdown_signals"] = list(STOP_SIGNALS)
            report["status"] = "interrupted" if report.get("owned_shutdown_clean") else "failed_cleanup"
            exit_code = 1
        report["lifecycle"] = supervisor.receipts
        private_json(evidence / "local-gate.json", report)
    print(json.dumps({"status": report["status"], "report": str(evidence / "local-gate.json"), "exit_code": exit_code}))
    raise SystemExit(exit_code)


if __name__ == "__main__":
    main()
