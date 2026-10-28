#!/usr/bin/env python3
"""Serial user-outcome runner. No commands execute on import.

Run only as an exact operator-approved gate after the whole-phase boundary.
"""
from __future__ import annotations

import argparse
import importlib
import json
import math
import os
from pathlib import Path
import re
import sys
import time
import uuid
from urllib.parse import urlsplit

from core import Blocked, Case, Context, Evidence, Failed, Outcome, digest
from packet_processes import PacketProcesses


BACKENDS = {"postgres_pgvector", "surreal_disk"}
LIMITS = ("wall_seconds", "request_seconds", "command_seconds", "cleanup_seconds",
          "http_requests", "response_bytes", "total_output_bytes", "output_tokens")


def load_fixture(path: Path) -> tuple[dict, list[str]]:
    raw = path.read_bytes()
    fixture = json.loads(raw)
    fixture["_fixture_digest"] = digest(raw)
    fixture["_fixture_path"] = str(path.resolve())
    return admit_fixture(fixture)


def admit_fixture(fixture: dict, *, scoped: bool = False) -> tuple[dict, list[str]]:
    """Admit selected inputs without starting processes or preparing private state.

    The historical loader retains its complete two-store contract. Scoped
    callers supply a projection whose closure and environment references have
    already been selected from an explicit workflow resource manifest.
    """
    if fixture.get("schema_version") != 1 or fixture.get("executable") is not True:
        raise Blocked("fixture is an example or lacks an executable version-1 contract")
    for name in ("source_identity", "artifact_identity", "configuration_revision", "schema_revision"):
        if not isinstance(fixture.get(name), str) or not fixture[name]:
            raise Blocked(f"fixture lacks frozen {name}")
    if fixture.get("positive_repetitions") != 3:
        raise Blocked("positive outcomes require three fresh repetitions")
    limits = fixture.get("limits", {})
    if any(type(limits.get(name)) not in (int, float) or limits[name] <= 0 for name in LIMITS):
        raise Blocked("all finite execution bounds must be positive")
    if any(type(limits[name]) is not int for name in
           ("http_requests", "response_bytes", "total_output_bytes", "output_tokens")):
        raise Blocked("count and byte allowances must be integers")
    if any(not math.isfinite(limits[name]) for name in LIMITS):
        raise Blocked("execution bounds must be finite")
    profiles = fixture.get("backends", [])
    if scoped and (len(profiles) != 1 or profiles[0].get("name") not in BACKENDS):
        raise Blocked("workflow fixture requires exactly one supported durable backend")
    if not scoped and (len(profiles) != 2 or {item.get("name") for item in profiles} != BACKENDS):
        raise Blocked("both Postgres+pgvector and disk-backed Surreal fixtures are mandatory")
    secrets = []
    for variable in fixture.get("secret_env", []):
        value = os.environ.get(variable)
        if not value:
            raise Blocked(f"missing named secret environment reference: {variable}")
        secrets.append(value)
    required_hooks = {"prepare", "admission", "ready", "inference", "session", "cleanup"}
    if scoped:
        required_hooks.remove("session")
    for backend in profiles:
        if backend.get("durable") is not True or not backend.get("profile_id"):
            raise Blocked("backend fixture must identify an owned durable profile")
        url = urlsplit(backend.get("base_url", ""))
        if url.scheme != "http" or url.hostname not in ("127.0.0.1", "localhost", "::1") or url.username or url.password or url.query or url.fragment:
            raise Blocked("owned server origin must be explicit loopback HTTP without credentials")
        if not backend.get("server", {}).get("argv"):
            raise Blocked("fixture lacks explicit owned server argv")
        if not required_hooks.issubset(backend.get("observers", {})):
            raise Blocked("fixture lacks implemented lifecycle/provenance observers")
        for variable in backend.get("headers_env", {}).values():
            value = os.environ.get(variable)
            if not value:
                raise Blocked(f"missing credential environment reference: {variable}")
            secrets.append(value)
        if "UAR_ACCEPTANCE_STATE_PARENT" not in backend.get("environment_env", {}):
            raise Blocked("backend must explicitly bind its isolated state parent")
        for key, variable in backend.get("environment_env", {}).items():
            value = os.environ.get(variable)
            if not value:
                raise Blocked(f"missing process environment reference: {variable}")
            if re.search(r"key|password|token|credential|authorization|secret", key + " " + variable, re.I):
                secrets.append(value)
        if backend.get("environment"):
            raise Blocked("use environment references, never inline server environment values")
    bindings = [fixture.get("provider", {}), *fixture.get("genuine_bindings", [])]
    for binding in bindings:
        if binding.get("kind") != "genuine" or not binding.get("qualified_model"):
            raise Blocked("positive inference requires a declared genuine binding")
        config = binding.get("config", {})
        if config.get("api_key") is not None:
            raise Blocked("provider credentials must be environment references")
        if binding["qualified_model"] != f"{config.get('id')}/{config.get('default_model')}":
            raise Blocked("provider/model declaration disagrees with configured identity")
        if not config.get("base_url") or config.get("enabled") is not True:
            raise Blocked("genuine provider endpoint/enabled config missing")
        variable = binding.get("credential_env")
        if variable:
            value = os.environ.get(variable)
            if not value:
                raise Blocked(f"missing provider credential environment reference: {variable}")
            secrets.append(value)
    # Freeze scripts/configs/binary inputs without executing a source-identity probe.
    files = fixture.get("frozen_files", [])
    if not files:
        raise Blocked("fixture must freeze executable/module/observer/config source files")
    frozen_paths = set()
    for item in files:
        candidate = Path(item["path"])
        if not candidate.is_absolute() or not candidate.is_file() or candidate.is_symlink():
            raise Blocked("frozen input must be an absolute regular file")
        if digest(candidate.read_bytes()) != item["sha256"]:
            raise Blocked("frozen source/artifact input changed: " + str(candidate))
        frozen_paths.add(str(candidate.resolve()))
    root = Path(__file__).resolve().parent
    for name in ("core.py", "cases_prompt.py", "user_outcomes.py", "packet_processes.py"):
        if str(root / name) not in frozen_paths:
            raise Blocked("fixture omits runner input: " + name)
    for backend in profiles:
        commands = [backend["server"]["argv"], *backend["observers"].values(),
                    *backend.get("clients", {}).values()]
        for command in commands:
            if not isinstance(command, list) or not command or not all(isinstance(v, str) for v in command):
                raise Blocked("every process must declare an exact argv list")
            executable = Path(command[0])
            if not executable.is_absolute() or str(executable.resolve()) not in frozen_paths:
                raise Blocked("command executable is not an absolute frozen input")
            for argument in command[1:]:
                candidate = Path(argument)
                if candidate.is_absolute() and candidate.is_file() and str(candidate.resolve()) not in frozen_paths:
                    raise Blocked("command file argument is not a frozen input")
    packet = fixture.get("packet_processes", {}).get("budget_proxy", {})
    command = packet.get("argv")
    if (not isinstance(command, list) or len(command) != 3
            or not all(isinstance(value, str) for value in command)
            or not isinstance(packet.get("epoch_id"), str)
            or type(packet.get("ready_seconds")) is not int or not 0 < packet["ready_seconds"] <= 120
            or not isinstance(packet.get("listen_ports"), list) or not packet["listen_ports"]):
        raise Blocked("shared forwarding command and exact epoch/listener contract are required")
    if "UAR_ACCEPTANCE_STATE_PARENT" not in packet.get("environment_env", {}):
        raise Blocked("shared forwarding owner lacks its separately bound private parent")
    for argument in command:
        candidate = Path(argument)
        if not candidate.is_absolute() or not candidate.is_file() or str(candidate.resolve()) not in frozen_paths:
            raise Blocked("shared forwarding executable/program/config is not an absolute frozen file")
    for key, variable in packet["environment_env"].items():
        value = os.environ.get(variable)
        if not value:
            raise Blocked("shared forwarding environment reference is missing: " + variable)
        if re.search(r"key|password|token|credential|authorization|secret", key + variable, re.I):
            secrets.append(value)
    service = fixture.get("packet_processes", {}).get("existing_service", {})
    command = service.get("argv")
    if not isinstance(command, list) or len(command) != 3 or not all(isinstance(value, str) for value in command):
        raise Blocked("configured Liter identity observation needs its exact frozen argv")
    for argument in command:
        candidate = Path(argument)
        if not candidate.is_absolute() or not candidate.is_file() or str(candidate.resolve()) not in frozen_paths:
            raise Blocked("configured Liter observation command or input is not an absolute frozen file")
    for key, variable in service.get("environment_env", {}).items():
        value = os.environ.get(variable)
        if not value:
            raise Blocked("configured service environment reference is missing: " + variable)
        if re.search(r"key|password|token|credential|authorization|secret", key + variable, re.I):
            secrets.append(value)
    fixture["_frozen_paths"] = sorted(frozen_paths)
    return fixture, secrets


def load_cases(fixture: dict) -> list[Case]:
    result = []
    names = fixture.get("modules", [])
    if not names:
        raise Blocked("no implemented case modules declared")
    for name in names:
        if not re.fullmatch(r"cases_[a-z][a-z0-9_]*", name):
            raise Blocked("case module must be a local cases_* module")
        path = Path(__file__).resolve().parent / (name + ".py")
        if str(path) not in fixture["_frozen_paths"]:
            raise Blocked("case module is not a frozen source input: " + name)
        module = importlib.import_module(name)
        definitions = module.cases()
        if not isinstance(definitions, list) or not all(isinstance(case, Case) for case in definitions):
            raise Blocked("case module did not supply implemented Case objects")
        result.extend(definitions)
    ids = [case.case_id for case in result]
    if len(set(ids)) != len(ids):
        raise Blocked("duplicate case IDs")
    if any(case.repetitions != 3 or not callable(case.run) for case in result):
        raise Blocked("positive cases need implemented runners and three repetitions")
    return result


def admission(ctx: Context) -> None:
    prepared = ctx.observe("prepare", {"profile_id": ctx.backend["profile_id"]})
    if prepared.get("run_id") != ctx.run_id or prepared.get("owned") is not True:
        raise Blocked("fixture did not create isolated run-owned resources")
    if "startup_initial_capture" in ctx.backend.get("observers", {}):
        ctx.observe("startup_initial_capture", {"profile_id": ctx.backend["profile_id"]})
    ctx.start()
    value = ctx.observe("admission", {"profile_id": ctx.backend["profile_id"]})
    expected = {"backend": ctx.backend["name"], "profile_id": ctx.backend["profile_id"],
                "run_id": ctx.run_id, "durable": True, "empty_owned_store": True,
                "source_identity": ctx.fixture["source_identity"],
                "artifact_identity": ctx.fixture["artifact_identity"],
                "configuration_revision": ctx.fixture["configuration_revision"],
                "schema_revision": ctx.fixture["schema_revision"]}
    if any(value.get(key) != item for key, item in expected.items()):
        raise Blocked("actual backend/source/artifact/schema admission disagrees with fixture")
    if not value.get("runtime_epoch") or not value.get("store_identity"):
        raise Blocked("actual durable store/runtime identity missing")
    if ctx.backend["name"] == "postgres_pgvector" and value.get("pgvector") is not True:
        raise Blocked("Postgres fixture lacks actual pgvector admission")
    if ctx.backend["name"] == "surreal_disk" and (value.get("ephemeral") is not False
                                                   or not value.get("disk_path")):
        raise Blocked("Surreal fixture is not confirmed disk-backed")


def execute(fixture: dict, evidence: Evidence, run_id: str, selected: list[str],
            collect_results: bool = False) -> tuple[int, dict]:
    if collect_results:
        module = Path(__file__).resolve().parent / "failure_sweep.py"
        if str(module) not in fixture["_frozen_paths"]:
            return 2, {"run_id": run_id, "status": "BLOCKED", "phase_acceptance": False,
                       "error": "collection controller must be a frozen source input"}
        from failure_sweep import execute_collection
        return execute_collection(fixture, evidence, run_id, selected,
                                  load_cases, admission, load_fixture)
    results = []
    summary = {"run_id": run_id, "status": "BLOCKED", "results": results,
               "scope": "explicit_subset" if selected else "declared_manifest",
               "phase_acceptance": False}
    contexts = []
    packet = None
    exit_code = 2
    try:
        cases = load_cases(fixture)
        requested = selected or fixture.get("required_cases", [])
        if not requested or len(set(requested)) != len(requested):
            raise Blocked("required case manifest must be explicit and unique")
        available = {case.case_id: case for case in cases}
        missing = sorted(set(requested) - set(available))
        if missing:
            raise Blocked("unimplemented required scenarios: " + ", ".join(missing))
        declared = fixture.get("required_cases", [])
        if set(selected) - set(declared):
            raise Blocked("selected case is outside the frozen required manifest")
        if not selected:
            expected_families = {f"U{number:02d}" for number in range(1, 26)}
            families = {available[name].family for name in requested}
            if families != expected_families:
                raise Blocked("full manifest must retain all 25 required outcome families")
        inventory = [{"backend": backend["name"], "case_id": name, "family": available[name].family,
                      "attempt": attempt, "status": "NOT_RUN"}
                     for backend in fixture["backends"] for name in requested for attempt in range(1, 4)]
        summary["inventory"] = inventory
        evidence.write("instantiated-case-manifest", inventory)
        global_started = time.monotonic()
        packet = PacketProcesses(fixture, evidence, run_id, global_started)
        packet.start()
        global_requests = global_output = 0
        for backend in fixture["backends"]:
            ctx = Context(fixture, backend, run_id, evidence)
            ctx.environment["UAR_ACCEPTANCE_BUDGET_PID"] = str(packet.process.pid)
            ctx.environment["UAR_ACCEPTANCE_BUDGET_NONCE"] = packet.ready["nonce"]
            ctx.environment["UAR_ACCEPTANCE_BUDGET_INPUT_IDENTITY"] = packet.ready["epoch"]["input_identity"]
            ctx.started = global_started
            ctx.requests, ctx.output_bytes = global_requests, global_output
            contexts.append(ctx)
            admission(ctx)
            for row in (item for item in inventory if item["backend"] == backend["name"]):
                ctx.case_id, ctx.attempt = row["case_id"], row["attempt"]
                packet.alive()
                ctx.remaining()
                try:
                    outcome = available[row["case_id"]].run(ctx)
                    if not isinstance(outcome, Outcome):
                        raise Blocked("implemented case returned no outcome predicates")
                    passed = outcome.protocol.passed is True and outcome.useful.passed is True
                    result = {**row, "status": "PASS" if passed else "FAIL",
                              "protocol": vars(outcome.protocol), "useful": vars(outcome.useful),
                              "facts": outcome.facts}
                    ctx.record("outcome", result)
                    results.append(result)
                    row["status"] = result["status"]
                    if not passed:
                        raise Failed("case outcome predicate failed")
                except (Blocked, Failed) as error:
                    if row["status"] == "NOT_RUN":
                        row["status"] = "BLOCKED" if isinstance(error, Blocked) else "FAIL"
                        result = {**row, "error_type": type(error).__name__, "error": str(error)}
                        results.append(result)
                        ctx.record("outcome", result)
                    raise
            global_requests, global_output = ctx.requests, ctx.output_bytes
            contexts.remove(ctx)
            try:
                ctx.cleanup()
            except BaseException:
                summary["cleanup_unconfirmed"] = True
                raise Blocked("backend cleanup failed; no automatic cleanup retry or next backend")
        summary["status"] = "PASS"
        # Phase acceptance also requires the separately approved supporting gates.
        exit_code = 0
    except (Blocked, Failed) as error:
        summary.update(status="BLOCKED" if isinstance(error, Blocked) else "FAIL",
                       error_type=type(error).__name__, error=str(error))
        evidence.write("first-failure", summary)
        exit_code = 2 if isinstance(error, Blocked) else 1
    except BaseException as error:
        summary.update(status="FAIL", error_type=type(error).__name__, error=str(error))
        evidence.write("runner-failure", summary)
        exit_code = 1
    finally:
        for ctx in reversed(contexts):
            try:
                ctx.cleanup()
            except BaseException as error:
                evidence.write("cleanup-failure", {"type": type(error).__name__, "message": str(error)})
                summary["status"] = "BLOCKED"
                summary["cleanup_unconfirmed"] = True
                exit_code = 2
        if packet is not None:
            try:
                packet.stop()
            except BaseException as error:
                evidence.write("packet-cleanup-failure", {"type": type(error).__name__, "message": str(error)})
                summary["status"] = "BLOCKED"
                summary["cleanup_unconfirmed"] = True
                exit_code = 2
    return exit_code, summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixture", type=Path, required=True)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument("--collect-results", action="store_true",
                        help="opt-in full serial sweep; requires frozen failure_sweep contract")
    parser.add_argument("--execute", action="store_true", help="only under exact operator gate approval")
    args = parser.parse_args()
    if not args.execute:
        print("BLOCKED: execution requires the exact operator-approved --execute command", file=sys.stderr)
        return 2
    # This is execution-time admission; it does not confer authority to run itself.
    run_id = str(uuid.uuid4())
    evidence = Evidence(args.evidence, [])
    try:
        fixture, secrets = load_fixture(args.fixture)
        evidence.secrets = sorted(set(secrets), key=len, reverse=True)
    except (Blocked, OSError, ValueError, KeyError, TypeError) as error:
        # Malformed fixtures may contain inline secrets: retain only the error
        # class and source bytes' digest, never the unadmitted payload/error text.
        summary = {"run_id": run_id, "status": "BLOCKED", "stage": "fixture_admission",
                   "error_type": type(error).__name__, "services_started": False}
        try:
            summary["fixture_digest"] = digest(args.fixture.read_bytes())
        except OSError:
            summary["fixture_digest"] = None
        evidence.seal(summary)
        print(json.dumps({"status": "BLOCKED", "evidence": str(args.evidence), "run_id": run_id}))
        return 2
    evidence.write("fixture", {"digest": fixture["_fixture_digest"], "fixture": fixture})
    code, summary = execute(fixture, evidence, run_id, args.case, args.collect_results)
    evidence.seal(summary)
    print(json.dumps({"status": summary["status"], "evidence": str(args.evidence), "run_id": run_id}))
    return code


if __name__ == "__main__":
    sys.exit(main())
