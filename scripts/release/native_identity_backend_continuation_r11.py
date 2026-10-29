#!/usr/bin/env python3
"""Fixed r11 continuation of the actual failed r10 backend attempt.

Preparation is not execution authority. This command requires its own explicit
approval after the complete successor freeze. It adopts the recorded ten-test
agent_threads prefix and executes only the three remaining integration targets
and the unchanged customer build. It does not rewrite any predecessor artifact.
"""
import argparse
from collections import Counter
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import time

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
HANDOFF = ROOT / ".agent-team/uar-core/handoffs/kubo-native-identity"
PRIVATE = Path("/Volumes/T7/pr13-native-identity-r9-private")
PRIOR_OUTPUT = PRIVATE / "backend-evidence-r1"
OLD_TREE = "9078a8706dfcbd5f474c9e5b88dbdda769d26fee"
BASE_PATH = "scripts/release/native_identity_backend_gate.py"
BASE_SHA256 = "6389700f122977be2e059c1ce95946603fc70e21a3bef503fe36079bb8efb634"
SUPPORT_PATH = "scripts/release/uar_1_1_1_support.py"
SUPPORT_SHA256 = "2ba73ad2ec3a6214fa81ae587750e5c18f29a9e0998a7b5dcb280129f97a1073"
HELPER = "tests/support/fresh_process_resume_process.rs"
HELPER_OLD = "e6b44614e943b95fcc12444c87fd86310582a5439e1a838a6ed28ade8566ed46"
HELPER_NEW = "1cb606ac8f19fd647a064c5906656c6e9eb5395caf7b41e1df2fe887e1c6ad91"
PLAN = "openspec/changes/preserve-native-tool-call-identity/fixture-ingestion-continuation-r11-plan.md"
SCRIPT = "scripts/release/native_identity_backend_continuation_r11.py"
FULL_TARGETS = (
    ("agent_threads", "server-full,postgres-backend,in-memory-backend", 10),
    ("fresh_process_resume", "server-full,postgres-backend", 1),
    ("stream_terminal_contract", "server-full", 1),
    ("approval_recovery", "server-full,postgres-backend,in-memory-backend", 1),
)
SUFFIX = FULL_TARGETS[1:]
EXECUTED_NAMES = [row[0] for row in SUFFIX] + ["customer-build"]
# Exact retained bytes from continuation-input-identities-r10.json. The private
# settings digest is separately retained in operator-approval-backend-r10.json.
PRIOR_INPUTS = {
    "receipt": (PRIOR_OUTPUT / "receipt.json", 6968,
                "de891a24fd668d7b4f183e30531580d996332e7ea7b6d4dface00bacdd48b7f8"),
    "prefix_command": (PRIOR_OUTPUT / "command-00127.json", 947,
                       "0aa45de8566997c6beba3ec8e0c7f97055357d232bd6981cd935dff45fd0f92d"),
    "prefix_log": (PRIOR_OUTPUT / "command-00127.log", 7933,
                   "18c3625c58c1587cfe8388a20fe853375a80db647a1573cdf3ab0902ddcb46e0"),
    "failed_command": (PRIOR_OUTPUT / "command-00204.json", 939,
                       "d52e2353e1d182901f28024020ce3b5b78d78b7751c65d44a233a223e4407139"),
    "failed_log": (PRIOR_OUTPUT / "command-00204.log", 2142,
                   "b888754dde259d3f6166059031e3678af6f6006e702c2dac1b988423cae04ca9"),
    "prefix_source": (PRIOR_OUTPUT / "source-after-agent_threads.json", 14566,
                      "c82e51d868c46f4183f34a10c4169ba8e69b4c2a0a04d2538249b720b1a205df"),
    "final_source": (PRIOR_OUTPUT / "source-after-cleanup.json", 14566,
                     "c82e51d868c46f4183f34a10c4169ba8e69b4c2a0a04d2538249b720b1a205df"),
    "manifest": (HANDOFF / "source-manifest-r10.json", 26462,
                 "30463729e14578d625887b50434846a2d42f2386f5bba2dd120e4ef97b75f11d"),
    "settings": (PRIVATE / "backend-settings.json", None,
                 "c505ce72cad4cda772934a93b0e053bcdd75623717edd6794106a7ade84e3d00"),
}


class ContinuationBlocked(RuntimeError):
    pass


def require(condition, reason):
    if not condition:
        raise ContinuationBlocked(reason)


def bound_bytes(path, expected, size=None):
    require(path.is_file() and not path.is_symlink(), "required retained input missing: " + str(path))
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == expected, "retained input bytes changed: " + str(path))
    require(size is None or len(raw) == size, "retained input size changed: " + str(path))
    return raw


def prior_inputs():
    return {name: bound_bytes(path, digest, size)
            for name, (path, size, digest) in PRIOR_INPUTS.items()}


def input_references():
    return {name: {"path": str(path), "sha256": digest, "bytes": size}
            for name, (path, size, digest) in PRIOR_INPUTS.items()}


def import_baseline():
    # Verify executable Python sources before importing them, not after they run.
    bound_bytes(ROOT / BASE_PATH, BASE_SHA256)
    bound_bytes(ROOT / SUPPORT_PATH, SUPPORT_SHA256)
    spec = importlib.util.spec_from_file_location("native_identity_backend_gate_r10", ROOT / BASE_PATH)
    require(spec is not None and spec.loader is not None, "baseline module loader unavailable")
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    require(base.ROOT == ROOT and base.TARGETS == FULL_TARGETS, "baseline target contract changed")
    return base


def file_inventory(manifest):
    result = {}
    for row in manifest["files"] + manifest["controlFiles"]:
        path = row["path"]
        require(path not in result, "duplicate source/control inventory path")
        result[path] = {key: row[key] for key in ("bytes", "mode", "sha256")}
    return result


def retained_prefix(base, settings):
    raw = prior_inputs()
    parsed = {name: json.loads(value) for name, value in raw.items() if not name.endswith("_log")}
    receipt, manifest, source = parsed["receipt"], parsed["manifest"], parsed["prefix_source"]
    require(source == parsed["final_source"], "historical source changed after the passing prefix")
    require(receipt["schema_version"] == 1 and receipt["status"] == "blocked"
            and receipt["cleanup"] == "complete", "predecessor is not the actual failed, cleaned attempt")
    require(receipt["source_manifest"] == str(PRIOR_INPUTS["manifest"][0])
            and receipt["source_manifest_sha256"] == PRIOR_INPUTS["manifest"][2]
            and receipt["staged_git_tree"] == OLD_TREE == manifest["stagedGitTree"]
            and source["source_manifest_sha256"] == PRIOR_INPUTS["manifest"][2]
            and source["staged_git_tree"] == OLD_TREE, "historical source binding differs")
    require(manifest["root"] == str(ROOT) and manifest["sourceComplete"] is True
            and manifest["noActiveWriters"] is True, "historical manifest is not the completed source boundary")
    for key in ("cleanup_failure", "final_freeze_failure", "retention_failure", "lock_close_failure", "supervisor_failure"):
        require(key not in receipt, "predecessor has unresolved evidence or cleanup failure")
    require(receipt["receipt_write_failures"] == [] and receipt["diagnostic_write_failures"] == [],
            "predecessor diagnostic writes were incomplete")
    cleanup = receipt["owned_resource_cleanup"]
    require(len(cleanup) == 6 and len({row["id"] for row in cleanup}) == 6
            and Counter(row["kind"] for row in cleanup) == {"container": 3, "volume": 2, "network": 1}
            and all(row["status"] == "removed" and "failure" not in row and "capture_failure" not in row
                    for row in cleanup), "predecessor does not establish six owned removals")
    for row in cleanup:
        if row["kind"] == "container":
            termination = row["termination"]
            require(termination["container"] == row["id"] and termination["completion"] == "process_exited"
                    and termination["exit_code"] == "0" and termination["attachment_failure"] is None,
                    "predecessor container termination is incomplete")
    gates = receipt["gates"]
    require([gate["name"] for gate in gates] == ["agent_threads", "fresh_process_resume"],
            "predecessor gate sequence differs")
    require(gates[0]["status"] == "passed" and gates[0]["exit_code"] == 0
            and gates[1]["status"] == "blocked" and gates[1]["exit_code"] == 101
            and receipt["failure"] == gates[1]["failure"] == {
                "type": "Blocked", "reason": "fresh_process_resume failed"}, "predecessor outcomes differ")
    old_settings = parsed["settings"]
    allowed_changes = {"source_manifest", "source_manifest_sha256", "staged_git_tree", "output_dir", "resource_dir"}
    require(set(settings) - {"_settings_path"} == set(old_settings), "continuation settings schema differs")
    require(all(settings[key] == value for key, value in old_settings.items() if key not in allowed_changes),
            "continuation changed a backend, platform, executable or cache input")
    require(old_settings["staged_git_tree"] == OLD_TREE
            and old_settings["source_manifest_sha256"] == PRIOR_INPUTS["manifest"][2]
            and old_settings["source_manifest"] == str(PRIOR_INPUTS["manifest"][0])
            and old_settings["output_dir"] == str(PRIOR_OUTPUT), "historical settings binding differs")
    for key in ("platform", "surreal_image", "postgres_image"):
        require(receipt[key] == old_settings[key] == settings[key], "historical backend setting differs")
    for index, name in enumerate(("prefix", "failed")):
        target, features, _ = FULL_TARGETS[index]
        command = parsed[name + "_command"]
        require(gates[index]["argv"] == command["argv"] == base.gate_command(old_settings, target, features),
                "historical command differs from the whole-target contract")
        require(gates[index]["command_receipt"] == PRIOR_INPUTS[name + "_command"][0].name
                and command["exit_code"] == gates[index]["exit_code"]
                and command["status"] == ("passed" if index == 0 else "failed")
                and command["log_retained"] is True
                and command["log"] == str(PRIOR_INPUTS[name + "_log"][0])
                and command["log_sha256"] == PRIOR_INPUTS[name + "_log"][2]
                and command["log_bytes"] == len(raw[name + "_log"]), "historical command evidence differs")
    # This is the unchanged baseline oracle, before selecting the fixed suffix.
    coverage = base.acceptance("agent_threads", raw["prefix_log"].decode(), {})
    require(coverage == gates[0]["coverage"], "historical prefix coverage differs from its raw log")
    failure_summary = re.findall(r"test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out",
                                 raw["failed_log"].decode())
    require(failure_summary == [("0", "1", "0", "0", "0")], "predecessor failed target summary differs")
    old_files = file_inventory(manifest)
    require(len(old_files) == 64 and old_files == source["files"], "historical source/control denominator differs")
    require(old_files[HELPER]["sha256"] == HELPER_OLD
            and old_files[BASE_PATH]["sha256"] == BASE_SHA256
            and old_files[SUPPORT_PATH]["sha256"] == SUPPORT_SHA256, "historical source preimages differ")
    require(manifest["pinnedDependencySources"] == source["pinned_dependency_sources"],
            "historical dependency identities disagree")
    external = [{key: row[key] for key in ("path", "bytes", "sha256", "kind")}
                for row in manifest["externalBuildInputs"]]
    require(external == source["external_build_inputs"]
            and external == [base.cargo_configuration_input(settings)], "external build input continuity differs")
    return {"manifest": manifest, "source": source, "coverage": coverage,
            "references": input_references(), "old_files": old_files}


def install_fixed_adapter(base, proof):
    original_freeze = base.SourceFreeze

    class ContinuationFreeze(original_freeze):
        def capture(self, label):
            # Rebind historical bytes every time; no supplied pass object can stand in for them.
            prior_inputs()
            bound_bytes(ROOT / BASE_PATH, BASE_SHA256)
            bound_bytes(ROOT / SUPPORT_PATH, SUPPORT_SHA256)
            value = super().capture(label)
            current = base.load(self.path)
            previous = proof["manifest"]
            old = proof["source"]
            require(current["baselineCommit"] == previous["baselineCommit"]
                    and current["branch"] == previous["branch"], "source baseline or branch changed")
            files = file_inventory(current)
            require(len(current["files"]) == 45 and len(current["controlFiles"]) == 21
                    and len(files) == 66 and files == value["files"], "successor source/control denominator differs")
            old_source_paths = {row["path"] for row in previous["files"]}
            old_control_paths = {row["path"] for row in previous["controlFiles"]}
            require(HELPER in old_control_paths
                    and {row["path"] for row in current["files"]} == old_source_paths | {HELPER, PLAN, SCRIPT}
                    and {row["path"] for row in current["controlFiles"]} == old_control_paths - {HELPER},
                    "successor inventory changes exceed the fixed repair and additions")
            require(set(files) == set(proof["old_files"]) | {PLAN, SCRIPT}, "source path continuity differs")
            require(all(files[path] == identity for path, identity in proof["old_files"].items() if path != HELPER),
                    "previously accepted source/control bytes or modes changed")
            require(files[HELPER]["sha256"] == HELPER_NEW and files[HELPER]["mode"] == "0644"
                    and files[PLAN]["mode"] == "0644" and files[SCRIPT]["mode"] == "0644",
                    "successor does not contain the exact reviewed helper and regular additions")
            require(current["pinnedDependencySources"] == previous["pinnedDependencySources"]
                    and value["pinned_dependency_sources"] == old["pinned_dependency_sources"]
                    and current["externalBuildInputs"] == previous["externalBuildInputs"]
                    and value["external_build_inputs"] == old["external_build_inputs"],
                    "dependency or external input continuity changed")
            require(current["allowedUnstagedMetadataPaths"] == previous["allowedUnstagedMetadataPaths"]
                    and current["allowedUntrackedMetadataRoots"] == previous["allowedUntrackedMetadataRoots"],
                    "metadata exclusion scope changed")
            delta = self.git(ROOT, "diff", "--name-status", "--no-renames", "-z", OLD_TREE, value["staged_git_tree"])
            parts = delta.split("\0")
            require(parts[-1] == "" and len(parts[:-1]) % 2 == 0, "non-parseable complete staged-tree delta")
            changes = list(zip(parts[:-1:2], parts[1:-1:2]))
            require(len(changes) == 3 and set(changes) == {("M", HELPER), ("A", PLAN), ("A", SCRIPT)},
                    "complete staged-tree delta exceeds the fixed continuation")
            binding = {"kind": "r11-fixed-failed-gate-continuation-binding", "source_capture": label,
                       "prior_inputs": proof["references"], "prior_tree": OLD_TREE,
                       "successor_tree": value["staged_git_tree"], "successor_manifest": str(self.path),
                       "successor_manifest_sha256": self.expected,
                       "historical_prefix": {"name": "agent_threads", "executed_in_this_attempt": False,
                                             "coverage": proof["coverage"]},
                       "fixed_suffix": EXECUTED_NAMES, "allowed_tree_delta": changes,
                       "helper_preimage_sha256": HELPER_OLD, "helper_successor_sha256": HELPER_NEW,
                       "external_build_inputs": value["external_build_inputs"]}
            base.save(self.resources.evidence / "continuation-binding.json", binding)
            return value

    # Fixed in source, with no caller target, command, skip flag or result callback.
    base.SourceFreeze = ContinuationFreeze
    base.TARGETS = SUFFIX


def reference(base, path):
    return {**base.artifact(path), "bytes": path.stat().st_size}


def continuation_result(base, settings, proof, code, invocation_failure):
    output = Path(settings["output_dir"])
    raw_path = output / "receipt.json"
    report = {"schema_version": 1, "kind": "r11-authoritative-failed-gate-continuation", "status": "blocked",
              "finished_at": time.time(), "baseline_exit_code": code,
              "retained_historical_prefix": {"name": "agent_threads", "executed_in_this_attempt": False,
                  "source_tree": OLD_TREE, "coverage": proof["coverage"],
                  "receipt": proof["references"]["receipt"], "log": proof["references"]["prefix_log"]},
              "requested_current_execution": EXECUTED_NAMES, "current_gate_observations": [],
              "prior_failure_preserved": proof["references"]["failed_log"],
              "limits": ["Historical prefix is adopted only through the bound unchanged-source proof.",
                         "The raw baseline receipt retains its four-target family scope label; it does not claim four current test executions.",
                         "Current execution consists of three full integration targets followed by the customer build, stopping at first failure.",
                         "Unbounded waits, permanent-I/O evidence limits, warning/submission holds and broader release/consumer gates remain.",
                         "This source adapter has no runtime negative/tamper certification."]}
    if invocation_failure is not None:
        report["invocation_failure"] = invocation_failure
    try:
        prior_inputs()
        raw = base.load(raw_path)
        report["raw_baseline_receipt"] = reference(base, raw_path)
        report["raw_baseline_family_scope_label"] = raw["scope"]
        require(raw["source_manifest_sha256"] == settings["source_manifest_sha256"]
                and raw["staged_git_tree"] == settings["staged_git_tree"], "current receipt source identity differs")
        gates = raw["gates"]
        require([row["name"] for row in gates] == EXECUTED_NAMES[:len(gates)] and len(gates) <= 4,
                "current receipt includes an unexpected target or execution order")
        for row in gates:
            name = row["name"]
            if name == "customer-build":
                expected = base.gate_command(settings)
            else:
                features = next(features for target, features, _ in SUFFIX if target == name)
                expected = base.gate_command(settings, name, features)
            require(row["argv"] == expected, "current executed command differs from fixed suffix")
            report["current_gate_observations"].append({"name": name, "status": row["status"],
                "execution_evidence": "exit-observed" if row.get("exit_code") is not None else "not-established-by-gate-receipt",
                "exit_code": row.get("exit_code"), "argv": row["argv"],
                "coverage_count": row.get("coverage", {}).get("count"),
                "command_receipt": row.get("command_receipt")})
        binding_path = output / "continuation-binding.json"
        if binding_path.is_file():
            report["source_continuity_binding"] = reference(base, binding_path)
        if code == 0:
            require(invocation_failure is None and raw["status"] == "passed" and raw["cleanup"] == "complete"
                    and len(gates) == 4 and all(row["status"] == "passed" and row["exit_code"] == 0 for row in gates)
                    and "source_continuity_binding" in report, "successful continuation lacks complete current evidence")
            report["status"] = "passed"
        else:
            report["current_failure"] = raw.get("failure")
            report["current_cleanup"] = raw.get("cleanup")
    except BaseException as error:
        report["continuation_evidence_failure"] = {"type": type(error).__name__, "reason": str(error)}
    try:
        base.save(output / "continuation-receipt.json", report)
    except OSError:
        print("Continuation blocked: authoritative receipt could not be retained; existing raw evidence is unchanged.", file=sys.stderr)
        return 2
    print("Fixed continuation " + report["status"] + "; authoritative private receipt: "
          + str(output / "continuation-receipt.json"))
    return 0 if report["status"] == "passed" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--settings", required=True)
    args = parser.parse_args()
    try:
        require(Path(args.settings) == PRIVATE / "backend-settings-r11.json", "this continuation has one fixed private settings path")
        base = import_baseline()
        settings = base.configure(args.settings)
        require(settings["source_manifest"] == str(HANDOFF / "source-manifest-r11.json")
                and settings["output_dir"] == str(PRIVATE / "backend-evidence-r2")
                and settings["resource_dir"] == str(PRIVATE / "backend-resources-r2"),
                "this continuation requires its fresh r11 manifest and r2 evidence/resources")
        proof = retained_prefix(base, settings)
        install_fixed_adapter(base, proof)
    except Exception as error:
        print("Continuation blocked before execution: " + str(error), file=sys.stderr)
        return 2
    code, failure = None, None
    try:
        code = base.execute(settings)
    except BaseException as error:
        failure = {"type": type(error).__name__, "reason": str(error)}
    return continuation_result(base, settings, proof, code, failure)


if __name__ == "__main__":
    raise SystemExit(main())
