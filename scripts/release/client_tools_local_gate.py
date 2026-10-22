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
import shutil
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


def jwt(secret, subject, roles, tenant=TENANT):
    def encoded(value):
        return base64.urlsafe_b64encode(json.dumps(value, separators=(",", ":")).encode()).rstrip(b"=").decode()
    now = int(time.time())
    header = encoded({"alg": "HS256", "typ": "JWT"})
    claims = encoded({"sub": subject, "name": subject, "roles": roles, "tenant_id": tenant,
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


def http(base, path, token=None, body=None, expected=200, extra_headers=None, timeout=5, method=None):
    require(not STOP_SIGNALS, "operator requested shutdown")
    headers = {"Accept": "application/json", **(extra_headers or {})}
    if token:
        headers["Authorization"] = "Bearer " + token
    if body is not None:
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(base + path, data=None if body is None else json.dumps(body).encode(), headers=headers, method=method)
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
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
            "cors_allowed_origins": [], "mcp_shared_transports_enabled": True, "ingest_directory": str(evidence / "ingest")},
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
            "models": [{"id": MODEL_ID, "pricing_identity": {"provider_id": "openai", "model_id": MODEL_ID},
                "supports_tools": True, "supports_streaming": True, "enabled": True}]}],
        "native_tools": {"file_tools_enabled": False, "web_fetch_enabled": False, "terminal_exec_enabled": False, "session_search_enabled": False},
        "acp": {"enabled": True, "path": "/acp", "auth_required": True},
        "resilience": {"rate_limit_enabled": True, "requests_per_second": 200.0, "burst_size": 400.0},
    }


class Supervisor:
    def __init__(self, evidence, environment):
        self.evidence, self.environment = evidence, environment
        self.children, self.logs, self.receipts = [], [], []
        self.joined_children = {}

    def start(self, name, command, extra_environment=None):
        require(not STOP_SIGNALS, "operator requested shutdown")
        log = open(self.evidence / (name + ".log"), "xb", opener=lambda name, flags: os.open(name, flags, 0o600))
        self.logs.append(log)
        child = subprocess.Popen([str(part) for part in command], cwd=self.evidence,
            env={**self.environment, **(extra_environment or {})}, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        self.children.append((name, child))
        self.receipts.append({"process": name, "pid": child.pid, "command": [str(part) for part in command]})
        return child

    def stop_child(self, name, child):
        if id(child) in self.joined_children:
            return self.joined_children[id(child)]
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
        cleanup_confirmed = joined and not kill_attempted and not errors
        exit_expected = joined and child.returncode in expected
        accepted = cleanup_confirmed and exit_expected
        receipt = {"process": name, "pid": child.pid, "joined": joined,
            "exit_code": child.returncode if joined else None, "forced_kill_attempted": kill_attempted,
            "forced_kill": forced, "cleanup_confirmed": cleanup_confirmed,
            "exit_status_expected": exit_expected, "clean_shutdown": accepted, "cleanup_errors": errors}
        self.receipts.append(receipt)
        if joined:
            self.joined_children[id(child)] = receipt
        return receipt

    def crash_candidate(self, name, child, generation):
        require(any(owned is child and owned_name == name for owned_name, owned in self.children), "crash target is not an owned candidate")
        require(name == "candidate" or name.startswith("candidate-"), "crash target is not a candidate")
        require(child.poll() is None, "candidate exited before expected recovery crash")
        errors, delivered, joined = [], False, False
        started = time.time()
        try:
            # Kill only the owned candidate PID; the controlled peer remains alive.
            child.kill()
            delivered = True
        except OSError as error:
            errors.append("expected crash signal: " + type(error).__name__ + ": " + str(error))
        try:
            child.wait(timeout=15)
            joined = True
        except (OSError, subprocess.TimeoutExpired) as error:
            errors.append("expected crash join: " + type(error).__name__ + ": " + str(error))
        confirmed = delivered and joined and child.returncode == -signal.SIGKILL and not errors
        receipt = {"action": "expected_crash_for_recovery", "process": name, "pid": child.pid,
            "generation": generation, "signal": signal.SIGKILL, "started_at_unix": started,
            "joined_at_unix": time.time() if joined else None, "joined": joined,
            "exit_code": child.returncode if joined else None, "signal_delivered": delivered,
            "expected_crash_confirmed": confirmed, "exit_status_expected": confirmed,
            "clean_shutdown": False, "cleanup_confirmed": False, "cleanup_errors": errors}
        self.receipts.append(receipt)
        if joined:
            self.joined_children[id(child)] = receipt
        return receipt

    def stop_join(self):
        clean = True
        for name, child in reversed(self.children):
            receipt = self.stop_child(name, child)
            if receipt.get("action") == "expected_crash_for_recovery":
                # A confirmed intentional crash is separately accounted, never
                # relabeled graceful cleanup. All other children still stop normally.
                clean = clean and receipt["expected_crash_confirmed"] and receipt["joined"]
            else:
                clean = clean and receipt["cleanup_confirmed"]
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
    parser.add_argument("--extended", action="store_true", help="After matrix passes, run safety, cancellation, ordinary/graph/budget warm-cold and mixed crash recovery gates")
    args = parser.parse_args()
    os.umask(0o077)
    evidence = (args.evidence or ROOT / "target" / ("client-tools-local-gate-" + str(uuid.uuid4()))).resolve()
    require(evidence.is_relative_to((ROOT / "target").resolve()), "evidence must be under repository target/")
    require(not evidence.exists(), "evidence directory must be fresh; existing private state is never reused")
    evidence.mkdir(parents=True, mode=0o700)
    for directory in ("tmp", "store", "models", "home", "ingest"):
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
        # Match the shipped container's cwd-relative mandatory Cedar assets.
        policy_directory = evidence / "policies"
        shutil.copytree(ROOT / "policies", policy_directory)
        policies = sorted(policy_directory.glob("*.cedar"))
        require(policies, "shipped policy tree contains no Cedar policy files")
        report["shipped_policy_files"] = [{"path": str(path.relative_to(evidence)),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in policies]

        secret = secrets.token_urlsafe(48)
        owner_subject = "client-tools-gate-admin-" + uuid.uuid4().hex
        owner_token = jwt(secret, owner_subject, ["admin"])
        other_token = jwt(secret, "client-tools-gate-other-" + uuid.uuid4().hex, [])
        credentials = {"owner": owner_token, "other": other_token}
        if args.extended:
            other_tenant_token = jwt(secret, owner_subject, ["admin"], tenant=TENANT + "-other")
            credentials["other_tenant"] = other_tenant_token
        private_json(evidence / "credentials.json", credentials)
        port_file = evidence / "provider.port"
        peer = supervisor.start("provider-peer", [sys.executable, ROOT / "tests/fixtures/client_tools_provider.py",
            "--host", "127.0.0.1", "--port", "0", "--port-file", port_file, "--trace", evidence / "provider-trace.jsonl"])
        peer_port = wait_for("provider-peer", peer, lambda: int(port_file.read_text()) if port_file.exists() else None, seconds=30)
        peer_url = "http://127.0.0.1:" + str(peer_port)
        wait_for("provider HTTP", peer, lambda: http(peer_url, "/_trace"), seconds=30)
        http_port, grpc_port = available_port(), available_port()
        require(http_port != grpc_port, "ephemeral listener allocation collided; no candidate was started")
        config = configuration(evidence, http_port, grpc_port, peer_url, secret)
        if args.extended:
            config["providers"].append({**config["providers"][0], "id": "openai",
                "display_name": "Catalog budget model through loopback fixture", "models": []})
        private_json(evidence / "config.yaml", config)
        admin_key = config["security"]["settings_admin_key"]
        report["isolated_rate_limit"] = config["resilience"]
        private_json(evidence / "mcp.json", {"mcpServers": {MCP_SERVER: {"url": peer_url + "/mcp", "preapproved_tools": ["effect"]}}})
        candidate = supervisor.start("candidate", [binary, "--config", evidence / "config.yaml"],
            {"MCP_CONFIG_PATH": str(evidence / "mcp.json")})
        base = "http://127.0.0.1:" + str(http_port)
        wait_for("candidate authenticated readiness", candidate, lambda: http(base, "/readyz", owner_token))
        # Admission requires a persisted proof from the actual protected admin
        # inference endpoint, not an enabled model flag authored into config.
        verification = http(base, "/api/providers/" + PROVIDER_ID + "/test", owner_token,
            {"model": MODEL_ID}, extra_headers={"X-UAR-Admin-Key": admin_key}, timeout=75)
        require(verification.get("ok") is True and verification.get("received_text") is True
            and verification.get("provider_id") == PROVIDER_ID and verification.get("model_id") == MODEL_ID,
            "actual provider verification did not return the selected model proof")
        private_json(evidence / "model-verification.json", verification)
        private_json(evidence / "model-verification-provider-trace.json", http(peer_url, "/_trace"))
        reset = http(peer_url, "/_control", body={"scenario": "single"})
        require(reset.get("configured") is True and not http(peer_url, "/_trace").get("requests"),
            "provider verification trace was not reset before matrix setup")

        if args.extended:
            budget_verification = http(base, "/api/providers/openai/test", owner_token, {"model": MODEL_ID},
                extra_headers={"X-UAR-Admin-Key": admin_key}, timeout=75)
            require(budget_verification.get("ok") is True and budget_verification.get("received_text") is True,
                "catalog budget model did not pass actual provider verification")
            private_json(evidence / "budget-model-verification.json", budget_verification)
            http(peer_url, "/_control", body={"scenario": "single"})

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
            "--agent-id", agent_id, "--openai-model", PROVIDER_ID + "/" + MODEL_ID,
            "--anthropic-model", PROVIDER_ID + "/" + MODEL_ID,
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
        if args.extended:
            require(len(results) == 95, "extended gates require the complete 95-case five-interface matrix")
            driver_environment = {"UAR_CLIENT_TOOLS_GATE_OWNER": owner_token, "UAR_CLIENT_TOOLS_GATE_OTHER": other_token,
                "UAR_CLIENT_TOOLS_GATE_OTHER_TENANT": other_tenant_token}
            command_prefix = matrix_command[:matrix_command.index("--mode")]
            generation = 1
            config_identity = hashlib.sha256((evidence / "config.yaml").read_bytes()).hexdigest()
            extended_reports = []
            report["extended_reports"] = extended_reports

            def run_extended(mode, extra=(), provider=PROVIDER_ID):
                report["extended_stage"] = mode
                command = list(command_prefix)
                for flag in ("--openai-model", "--anthropic-model"):
                    command[command.index(flag) + 1] = provider + "/" + MODEL_ID
                report_path = evidence / (mode + ".json")
                command += ["--mode", mode, "--report", report_path, *extra]
                child = supervisor.start(mode, command, driver_environment)
                deadline = time.monotonic() + args.matrix_timeout_secs
                while child.poll() is None:
                    require(not STOP_SIGNALS, "operator requested shutdown")
                    require(time.monotonic() < deadline, mode + " deadline exceeded")
                    time.sleep(0.2)
                require(child.returncode == 0, mode + " driver failed; inspect its retained log/report")
                receipt = json.loads(report_path.read_text())
                require(receipt.get("execution_complete") is True and receipt.get("results")
                    and all(item.get("status") == "passed" for item in receipt["results"])
                    and not receipt.get("coverage_gaps"), mode + " did not establish all selected observations")
                if mode in ("safety", "cancellation"):
                    require(len(receipt["results"]) == {"safety": 60, "cancellation": 5}[mode],
                        mode + " omitted required five-interface cases")
                extended_reports.append({"mode": mode, "report": str(report_path)})
                return receipt

            def restart_candidate():
                nonlocal candidate, generation
                stopped = supervisor.stop_child("candidate" if generation == 1 else "candidate-" + str(generation), candidate)
                require(stopped["cleanup_confirmed"] and stopped["exit_status_expected"], "candidate did not cleanly release before cold restart")
                require(hashlib.sha256((evidence / "config.yaml").read_bytes()).hexdigest() == config_identity,
                    "candidate config changed across cold restart")
                generation += 1
                candidate = supervisor.start("candidate-" + str(generation), [binary, "--config", evidence / "config.yaml"],
                    {"MCP_CONFIG_PATH": str(evidence / "mcp.json")})
                wait_for("cold candidate authenticated readiness", candidate, lambda: http(base, "/readyz", owner_token))

            safety_agent_id = "client-tools-safety-" + uuid.uuid4().hex
            safety_agent = http(base, "/api/agents", owner_token, artifact(safety_agent_id, actual_tool), expected=201)
            require(safety_agent.get("id") == safety_agent_id, "persisted safety agent changed selected ID")
            private_json(evidence / "safety-agent.json", safety_agent)
            run_extended("safety", ["--safety-agent-id", safety_agent_id,
                "--other-tenant-bearer-token-env", "UAR_CLIENT_TOOLS_GATE_OTHER_TENANT"])

            run_extended("cancellation")

            ordinary_state = evidence / "ordinary-restart-state.json"
            run_extended("restart-seed", ["--restart-state", ordinary_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])
            restart_candidate()
            run_extended("restart-resume", ["--restart-state", ordinary_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])

            worker = http(base, "/api/agents/general-purpose", owner_token)
            private_json(evidence / "graph-worker-before.json", worker)
            worker_marker = "CLIENT_TOOLS_GRAPH_WORKER_" + uuid.uuid4().hex
            router_marker = "You are a routing assistant. Respond with exactly one option from the provided list."
            worker["policy"]["provider"]["default"] = {"provider": PROVIDER_ID, "model": MODEL_ID}
            worker["prompt"]["system"] += "\n" + worker_marker
            saved_worker = http(base, "/api/agents/general-purpose", owner_token, worker, expected=200, method="PUT")
            private_json(evidence / "graph-worker-after.json", saved_worker)
            graph_flags = ["--graph-agent-id", "orchestrator-agent", "--graph-before-marker", router_marker,
                "--graph-after-marker", worker_marker]
            run_extended("graph", graph_flags)
            graph_state = evidence / "graph-restart-state.json"
            run_extended("graph-restart-seed", [*graph_flags, "--restart-state", graph_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])
            restart_candidate()
            cold_worker = http(base, "/api/agents/general-purpose", owner_token)
            require(worker_marker in cold_worker["prompt"]["system"], "persisted graph worker marker was lost during restart")
            run_extended("graph-restart-resume", [*graph_flags, "--restart-state", graph_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])

            models = http(base, "/api/providers/openai/models", owner_token, extra_headers={"X-UAR-Admin-Key": admin_key})
            model = next((item for item in models if item.get("id") == MODEL_ID and item.get("enabled")), None)
            require(model is not None and model.get("context_window") == 128000 and model.get("max_output_tokens") == 16384,
                "catalog budget reservation proof no longer matches the running model metadata")
            reservation, settled_usage, ceiling = 144384, 5, 144386
            require(reservation <= ceiling and settled_usage + reservation > ceiling, "invalid cumulative budget inequalities")
            private_json(evidence / "budget-proof.json", {"model": model, "reservation_source": "catalog_context_and_output",
                "R1": reservation, "R2": reservation, "S": settled_usage, "L": ceiling})
            private_json(evidence / "budget-provider-usage.json", {"input_tokens": 2, "output_tokens": 3})
            budget_agent_id = "client-tools-budget-" + uuid.uuid4().hex
            budget_agent = artifact(budget_agent_id, actual_tool)
            budget_agent["policy"]["provider"]["default"] = {"provider": "openai", "model": MODEL_ID}
            budget_agent["extensions"]["budgets"] = {"max_tokens_per_session": ceiling}
            private_json(evidence / "budget-agent.json", http(base, "/api/agents", owner_token, budget_agent, expected=201))
            budget_flags = ["--budget-agent-id", budget_agent_id, "--budget-provider-usage-json", evidence / "budget-provider-usage.json"]
            run_extended("budget", budget_flags, provider="openai")
            budget_state = evidence / "budget-restart-state.json"
            run_extended("budget-restart-seed", [*budget_flags, "--restart-state", budget_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)], provider="openai")
            restart_candidate()
            run_extended("budget-restart-resume", [*budget_flags, "--restart-state", budget_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)], provider="openai")
            mixed_state = evidence / "mixed-crash-state.json"
            mixed_seed = run_extended("mixed-crash-seed", ["--restart-state", mixed_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])
            require(len(mixed_seed["results"]) == 1, "mixed crash seed must retain its aggregate five-interface receipt")
            seed_observed = mixed_seed["results"][0].get("observed", {})
            require(seed_observed.get("seed_ready") is True and seed_observed.get("pending_batches") == 5
                and seed_observed.get("mixed_host_effects") == 5,
                "mixed crash seed did not confirm all five durable handoffs and settled host effects")
            require(mixed_state.is_file(), "mixed crash seed omitted protected state")
            report["mixed_crash_state_sha256"] = hashlib.sha256(mixed_state.read_bytes()).hexdigest()
            require(not STOP_SIGNALS, "operator requested shutdown")
            candidate_name = "candidate" if generation == 1 else "candidate-" + str(generation)
            crashed = supervisor.crash_candidate(candidate_name, candidate, generation)
            report["expected_crash_for_recovery"] = crashed
            require(crashed["expected_crash_confirmed"] and crashed["joined"], "expected candidate crash was not confirmed and joined")
            require(peer.poll() is None, "controlled peer exited during candidate-only crash")
            require(hashlib.sha256((evidence / "config.yaml").read_bytes()).hexdigest() == config_identity,
                "candidate config changed across recovery crash")
            generation += 1
            candidate = supervisor.start("candidate-" + str(generation), [binary, "--config", evidence / "config.yaml"],
                {"MCP_CONFIG_PATH": str(evidence / "mcp.json")})
            wait_for("crash successor authenticated readiness", candidate, lambda: http(base, "/readyz", owner_token))
            require(hashlib.sha256(mixed_state.read_bytes()).hexdigest() == report["mixed_crash_state_sha256"],
                "protected mixed crash state changed before recovery")
            run_extended("mixed-crash-resume", ["--restart-state", mixed_state,
                "--instance-identity", "candidate:" + str(candidate.pid) + ":generation-" + str(generation)])
            report.update(status="extended_passed_remaining_release_gates", extended_reports=extended_reports,
                completed_local_gates=["matrix", "safety", "cancellation", "ordinary-cold", "graph-warm", "graph-cold", "budget-warm", "budget-cold", "mixed-crash-recovery"],
                external_gate_inputs_remaining=["Broader release/backend-matrix acceptance remains outside this local gate."])
    except (GateFailure, OSError, ValueError, KeyError, subprocess.TimeoutExpired, urllib.error.URLError, KeyboardInterrupt) as error:
        # Never include child environment/config/credential contents in diagnostics.
        report["failure"] = type(error).__name__ + ": " + str(error)
        if args.extended and report["status"] == "matrix_passed_remaining_gates":
            report["status"] = "failed_extended"
        exit_code = 1
    finally:
        try:
            primary_status = report["status"]
            report["owned_shutdown_clean"] = supervisor.stop_join()
            report["owned_shutdown_clean_excludes_expected_crash"] = any(
                receipt.get("action") == "expected_crash_for_recovery" for receipt in supervisor.receipts)
            exits = [receipt["exit_status_expected"] for receipt in supervisor.receipts if "exit_status_expected" in receipt]
            report["owned_exit_statuses_expected"] = all(exits)
            report["cleanup_status"] = "confirmed" if report["owned_shutdown_clean"] else "failed_or_uncertain"
            if not report["owned_shutdown_clean"]:
                if primary_status in ("matrix_passed_remaining_gates", "extended_passed_remaining_release_gates"):
                    report["status"] = "failed_cleanup"
                exit_code = 1
            elif not report["owned_exit_statuses_expected"]:
                if primary_status in ("matrix_passed_remaining_gates", "extended_passed_remaining_release_gates"):
                    report["status"] = "failed_child_exit"
                exit_code = 1
        except (OSError, subprocess.TimeoutExpired) as error:
            if report["status"] in ("matrix_passed_remaining_gates", "extended_passed_remaining_release_gates"):
                report["status"] = "failed_cleanup"
            report["cleanup_status"] = "failed_or_uncertain"
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
