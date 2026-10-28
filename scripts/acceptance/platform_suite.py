"""Explicitly approved serial macOS/Linux/iOS suite sharing one budget owner.

Importing does no I/O. Windows is blocked and Android deferred, never passed.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
import time
import uuid

from core import Blocked, Evidence, digest, require
from failure_sweep import execute_collection
from packet_processes import PacketProcesses
from user_outcomes import admission, load_cases, load_fixture


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def private_reference(ref):
    path = Path(ref["path"])
    require(path.is_absolute() and path == path.resolve() and path.is_file()
            and path.stat().st_uid == os.getuid() and path.stat().st_mode & 0o077 == 0
            and path.stat().st_size <= 64 * 1024 * 1024, "private bounded regular packet input required")
    raw = path.read_bytes()
    require(digest(raw) == ref["sha256"], "packet input identity differs")
    return json.loads(raw)


def load_config(path):
    source = Path(path)
    require(source.is_absolute() and source == source.resolve() and source.is_file()
            and source.stat().st_uid == os.getuid() and source.stat().st_mode & 0o077 == 0
            and source.stat().st_size <= 1024 * 1024, "private suite config required")
    config = json.loads(source.read_bytes())
    require(config.get("schema_version") == 1 and config.get("executable") is True,
            "suite configuration is not executable")
    projection = {key: value for key, value in config.items() if key != "approval"}
    approval = private_reference(config["approval"])
    require(approval == {"operation": "execute-uar-four-platform-suite", "operator_approved": True,
                         "projection_sha256": digest(canonical(projection))}, "exact suite approval missing")
    require(set(config["fixtures"]) == {"macos", "linux"}
            and config["windows"] == "BLOCKED_NO_EXECUTION_HOST"
            and config["android"] == "DEFERRED_OPERATOR_SCOPE", "platform scope differs")
    for key, ceiling in (("wall_seconds", 43200), ("ios_seconds", 7200), ("max_corrections", 13)):
        require(type(config[key]) is int and 0 < config[key] <= ceiling, "invalid finite suite bound")
    require(config["max_corrections"] <= config["remaining_global_repairs"], "repair slots exceed remaining global allowance")
    return config


def same_packet(left, right):
    require(left["source_identity"] == right["source_identity"], "platforms have different source snapshots")
    require(left["limits"] == right["limits"], "platforms have different shared allowances")
    require(left.get("runtime_platform", "macos-native") == "macos-native"
            and right.get("runtime_platform") == "linux-container", "actual platform launch modes differ")
    a, b = left["packet_processes"]["budget_proxy"], right["packet_processes"]["budget_proxy"]
    require(a["argv"][:2] == b["argv"][:2] and a["epoch_id"] == b["epoch_id"]
            and a["environment_env"] == b["environment_env"]
            and Path(a["argv"][2]).read_bytes() == Path(b["argv"][2]).read_bytes(),
            "platforms must share identical budget configuration and the same durable epoch")
    require(left["required_cases"] == right["required_cases"], "platform case denominators differ")


def run_ios(config, fixture, packet, evidence, shared):
    spec = config["ios"]
    for name in ("runner", "python", "selection", "source_inventory"):
        ref = spec[name]
        path = Path(ref["path"])
        require(path.is_absolute() and path.is_file() and path == path.resolve()
                and digest(path.read_bytes()) == ref["sha256"], "iOS frozen input differs")
    require(spec["runner"]["path"] in fixture["_frozen_paths"]
            and spec["python"]["path"] in fixture["_frozen_paths"], "iOS executable omitted from frozen inputs")
    private_path = Path(spec["private_config_path"])
    control_path = private_path.with_name(private_path.name+".collection.json")
    results = Path(spec["result_root"])
    require(results.is_absolute() and results == results.resolve() and not results.exists()
            and results.parent == Path(config["private_scope"]),
            "iOS results require a fresh exclusive scope")
    require(private_path.is_absolute() and private_path.parent == Path(config["private_scope"])
            and private_path.parent.is_dir() and not private_path.parent.is_symlink()
            and private_path.parent.stat().st_uid == os.getuid()
            and private_path.parent.stat().st_mode & 0o077 == 0, "iOS private scope differs")
    token = os.environ.get(fixture["provider"]["credential_env"])
    require(bool(token), "iOS ingress credential reference missing")
    correction_ref = spec.get("correction_authority")
    if correction_ref is not None:
        private_reference(correction_ref)
        require(any(row["path"] == correction_ref["path"] and row["sha256"] == correction_ref["sha256"]
                    for row in fixture["frozen_files"]), "iOS correction authority must be an approved frozen input")
    run_id = str(uuid.uuid4())
    corrections_before = shared["corrections"]
    control_raw = canonical({"run_id":run_id,"epoch_id":packet.spec["epoch_id"],
        "corrections_before":corrections_before,"max_corrections":shared["max_corrections"],
        "correction_authority":correction_ref,
        "immutable_files":[row for row in fixture["frozen_files"]
                           if Path(row["path"]).suffix in (".py",".mjs")
                           or row["path"] == spec["python"]["path"]]}) + b"\n"
    raw = canonical({"base_url": fixture["provider"]["config"]["base_url"], "proxy_token": token,
                     "model": fixture["provider"]["wire_model"],
                     "budget_proxy_config": fixture["packet_processes"]["budget_proxy"]["argv"][2],
                     "scope_id": config["scope_id"],
                     "context_window": next(row["context_window"] for row in fixture["provider"]["config"]["models"]
                                            if row["id"] == fixture["provider"]["wire_model"])}) + b"\n"
    remaining = min(config["ios_seconds"], config["wall_seconds"] - (time.monotonic() - shared["started"]),
                    packet.ctx.remaining())
    require(remaining >= 1, "no remaining allowance for iOS")
    argv = [spec["python"]["path"], spec["runner"]["path"], "run", "--selection", spec["selection"]["path"],
            "--source-inventory", spec["source_inventory"]["path"], "--output", spec["artifact_root"],
            "--private-config", str(private_path), "--results",str(results),
            "--collection-control",str(control_path),"--timeout-seconds", str(int(remaining))]
    process = None
    created = False
    control_created = False
    try:
        with os.fdopen(os.open(private_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600), "wb") as output:
            created = True
            output.write(raw)
            output.flush()
            os.fsync(output.fileno())
        with os.fdopen(os.open(control_path, os.O_CREAT | os.O_EXCL | os.O_WRONLY | os.O_NOFOLLOW, 0o600), "wb") as output:
            control_created = True
            output.write(control_raw)
            output.flush()
            os.fsync(output.fileno())
        evidence.write("ios-start", {"argv": argv, "epoch_id": packet.spec["epoch_id"]})
        process = packet.ctx.owned_process(argv)
        packet.ctx.wait_process(process, remaining)
        receipt = results / "receipt.json"
        value = json.loads(receipt.read_bytes())
        require(value.get("status") in ("PASS","FAIL","BLOCKED")
                and process.returncode == {"PASS":0,"FAIL":1,"BLOCKED":2}[value["status"]]
                and value.get("platform") == "ios-simulator" and value.get("run_id") == run_id
                and value.get("source") == fixture["source_identity"] and value.get("repetitions") == 3,
                "iOS actual receipt does not establish the admitted result")
        evidence.write("ios-result", {"status": value["status"], "receipt": {"path": str(receipt), "sha256": digest(receipt.read_bytes())}})
        return {"status": value["status"], "source_identity": value["source"], "artifact_identity": value["artifact"],
                "final_source_identity":value["final_source_identity"],
                "final_artifact_identity":value["final_artifact_identity"],
                "safety_stop":value.get("safety_stop",True),"inventory":value.get("inventory",[])}
    finally:
        try:
            if process is not None:
                packet.ctx.stop_process(process)
                packet.ctx.release_process(process)
        finally:
            # Recover durable charges even if the child was stopped before its summary.
            consumed = 0
            if results.exists():
                require(results == results.resolve() and results.is_dir()
                        and results.stat().st_uid == os.getuid() and results.stat().st_mode & 0o077 == 0,
                        "iOS evidence ownership unknown; repair accounting blocked")
                for charge in sorted(results.glob("correction-consumed-*.json")):
                    require(not charge.is_symlink() and charge.is_file(),"iOS correction charge ownership differs")
                    value = json.loads(charge.read_bytes())
                    consumed += 1
                    require(value.get("run_id") == run_id
                            and value.get("shared_corrections_consumed") == corrections_before+consumed
                            and corrections_before+consumed <= shared["max_corrections"],
                            "iOS shared repair accounting unknown")
                    shared["corrections"] = corrections_before+consumed
            if control_created:
                require(control_path.is_file() and not control_path.is_symlink()
                        and control_path.read_bytes() == control_raw,"iOS control file changed; cleanup unconfirmed")
                control_path.unlink()
            if created:
                require(private_path.is_file() and not private_path.is_symlink()
                        and private_path.read_bytes() == raw, "iOS private file changed; cleanup unconfirmed")
                private_path.unlink()


def execute(config, evidence):
    result = {"schema_version": 1, "phase_acceptance": False, "platforms": {
        "macos": {"status": "NOT_RUN"}, "linux": {"status": "NOT_RUN"},
        "ios": {"status": "NOT_RUN"}, "windows": {"status": "BLOCKED", "reason": "no execution host"},
        "android": {"status": "DEFERRED", "reason": "operator scope; historical OpenSSL failure retained"}}}
    packet = None
    shared = {"started": time.monotonic(), "requests": 0, "output_bytes": 0,
              "corrections": 0, "max_corrections": config["max_corrections"]}
    try:
        fixtures, secrets = {}, []
        for name in ("macos", "linux"):
            ref = config["fixtures"][name]
            private_reference(ref)
            fixtures[name], selected_secrets = load_fixture(Path(ref["path"]))
            secrets.extend(selected_secrets)
        evidence.secrets = sorted(set(secrets), key=len, reverse=True)
        same_packet(fixtures["macos"], fixtures["linux"])
        require(config["wall_seconds"] <= fixtures["macos"]["limits"]["wall_seconds"], "suite wall exceeds fixture")
        packet = PacketProcesses(fixtures["macos"], evidence, str(uuid.uuid4()), shared["started"])
        packet.start()
        for name in ("macos", "linux"):
            child = Evidence(evidence.root / name, evidence.secrets)
            try:
                code, summary = execute_collection(fixtures[name], child, str(uuid.uuid4()), [],
                    load_cases, admission, load_fixture, packet_owner=packet, shared_state=shared)
                child.seal(summary)
            except BaseException as error:
                child.seal({"status": "BLOCKED", "error_type": type(error).__name__, "phase_acceptance": False})
                raise
            result["platforms"][name] = {"status": summary["status"], "counts": summary.get("counts", {}),
                "source_identity": fixtures[name]["source_identity"], "artifact_identity": fixtures[name]["artifact_identity"],
                "evidence": str(child.root), "exit_code": code}
            if summary.get("final_source_identity", fixtures[name]["source_identity"]) != fixtures[name]["source_identity"]:
                result["requires_platform_rebind"] = True
                raise Blocked("corrected source requires fresh later-platform artifacts and authority; prior results retained")
            if summary.get("safety_stop") or summary.get("cleanup_unconfirmed"):
                raise Blocked("platform cleanup, ownership or allowance stop; independent continuation unsafe")
        try:
            packet.alive()
            result["platforms"]["ios"] = run_ios(config, fixtures["macos"], packet, evidence, shared)
            if result["platforms"]["ios"].get("final_source_identity") != fixtures["macos"]["source_identity"]:
                result["requires_platform_rebind"] = True
            if result["platforms"]["ios"].get("safety_stop"):
                result["safety_stop"] = True
        except Exception as error:
            result["platforms"]["ios"] = {"status": "BLOCKED", "error_type": type(error).__name__}
            result["safety_stop"] = True
    except BaseException as error:
        result.update(safety_stop=True, error_type=type(error).__name__)
    finally:
        if packet is not None:
            try:
                packet.stop()
            except BaseException as error:
                result.update(safety_stop=True, cleanup_unconfirmed=True, cleanup_error_type=type(error).__name__)
    selected = [result["platforms"][name]["status"] for name in ("macos", "linux", "ios")]
    result["selected_runnable_scope_status"] = ("FAIL" if "FAIL" in selected else "PASS" if set(selected) == {"PASS"} else "BLOCKED")
    if result.get("safety_stop") or result.get("requires_platform_rebind"):
        result["selected_runnable_scope_status"] = "BLOCKED"
    result["four_platform_certification"] = False
    result["corrections_consumed"] = shared["corrections"]
    evidence.seal(result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if not args.execute:
        raise Blocked("explicit approved --execute invocation required")
    config = load_config(args.config)
    evidence = Evidence(args.evidence, [])
    result = execute(config, evidence)
    print(json.dumps({"status": result["selected_runnable_scope_status"], "evidence": str(args.evidence), "four_platform_certification": False}))
    return {"PASS": 0, "FAIL": 1, "BLOCKED": 2}[result["selected_runnable_scope_status"]]


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception:
        print("platform suite blocked; no acceptance claim", file=sys.stderr)
        sys.exit(2)
