#!/usr/bin/env python3
"""Fixed r12 continuation with the shipped local model inputs.

Preparation is not execution authority. A separately approved frozen invocation
adopts only the actual r10 ten-test prefix, preserves both failed attempts, and
executes the three remaining whole targets and the unchanged customer build.
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
R11_SCRIPT = "scripts/release/native_identity_backend_continuation_r11.py"
R11_SHA256 = "7810c7df129a741812470ee96a95bfbadf42a48187e629b15cf0a15a9b6b25f2"
R11_TREE = "65c31498c5c35bbbcb3c6a322d7b056dc5d4778f"
HELPER_NEW = "05570d521daf4746f8371cad944afeb5488c6f4ac60cecc3f7e25c7b4c39e616"
PLAN = "openspec/changes/preserve-native-tool-call-identity/startup-model-assets-r12-plan.md"
SCRIPT = "scripts/release/native_identity_backend_continuation_r12.py"
PREVIOUS_OUTPUT = PRIVATE / "backend-evidence-r2"
# Immutable retained bytes supplied by continuation-input-identities-r11.json.
R11_INPUTS = {
    "receipt": (PREVIOUS_OUTPUT / "receipt.json", 5462,
                "8eb0aec356ed53bd06883e8fa750500506541195dea4eec9affb9736e24dd066"),
    "continuation": (PREVIOUS_OUTPUT / "continuation-receipt.json", 3983,
                     "7669fafa01bd8bc7763680409a0122c818dffa0be3b59966316e53e1b7bfaaea"),
    "binding": (PREVIOUS_OUTPUT / "continuation-binding.json", 4435,
                "73d776d459c498b8b338bcd6bcf715b59824e0b4af159363130f3440f20b8001"),
    "command": (PREVIOUS_OUTPUT / "command-00130.json", 937,
                "b6dec087064dbbeb41444b9d62cde275ba302a0cede169babef190e4fffc5fb2"),
    "log": (PREVIOUS_OUTPUT / "command-00130.log", 2142,
            "cf54a9e0bd0f705e36b44b72963029b3f079e0c755035bca9c472d403b8878d0"),
    "before_source": (PREVIOUS_OUTPUT / "source-before-fresh_process_resume.json", 15006,
                      "94a60fe0ab3810484e45c8bd30990bf734319275817b10720399a1bc0065566d"),
    "after_source": (PREVIOUS_OUTPUT / "source-after-fresh_process_resume.json", 15006,
                     "94a60fe0ab3810484e45c8bd30990bf734319275817b10720399a1bc0065566d"),
    "final_source": (PREVIOUS_OUTPUT / "source-after-cleanup.json", 15006,
                     "94a60fe0ab3810484e45c8bd30990bf734319275817b10720399a1bc0065566d"),
    "manifest": (HANDOFF / "source-manifest-r11.json", 26785,
                 "aa8cbaa37a0690edd1b54b3775d68c8746ca302ceee9ec48698152f50f5427cd"),
    "settings": (PRIVATE / "backend-settings-r11.json", 1478,
                 "28fd90fcd32cadf61b4e432308803009cb6b24cb89394a2c0d13fd81cd7df080"),
}
MODEL_ROOT = "src/uar/runtime/matching/models"
# Names, raw bytes and original r10 Git blob identities from shipped-model-assets-r12.json.
MODEL_ROWS = (
    ("MODEL-NOTICE.txt", 331, "99d87511d89807668120d86f3bce9f3d138d8da2428cf66c0c82e39d26b1ad22", "370df919d1d2c1052156d519e62e47c0b30d8208"),
    ("bg-small-en-v1.5.onnx", 34014426, "6c9c6101a956d62dfb5e7190c538226c0c5bb9cb27b651234b6df063ee7dbfe4", "2d792d46dac2724cc6dffaaf38ecb4f5d51f4334"),
    ("config.json", 650, "f2c87ea17fdfa286f48d6e1506f2cdc838d303420dc55606190e67e5acd9e186", "5fbcd5887bf403b2aa098c5b91cf82015c497495"),
    ("special_tokens_map.json", 125, "b6d346be366a7d1d48332dbc9fdf3bf8960b5d879522b7799ddba59e76237ee3", "a8b3208c2884c4efb86e49300fdd3dc877220cdf"),
    ("tokenizer.json", 711396, "d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66", "688882a79f44442ddc1f60d70334a7ff5df0fb47"),
    ("tokenizer_config.json", 366, "9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3", "37fca74771bc76a8e01178ce3a6055a0995f8093"),
)
MODEL_FILES = {MODEL_ROOT + "/" + name: {"bytes": size, "mode": "0644", "sha256": digest}
               for name, size, digest, _ in MODEL_ROWS}


class ContinuationBlocked(RuntimeError):
    pass


def require(condition, reason):
    if not condition:
        raise ContinuationBlocked(reason)


def import_predecessor():
    path = ROOT / R11_SCRIPT
    require(path.is_file() and not path.is_symlink(), "r11 continuation source missing")
    require(hashlib.sha256(path.read_bytes()).hexdigest() == R11_SHA256, "r11 continuation source changed")
    spec = importlib.util.spec_from_file_location("native_identity_continuation_r11_retained", path)
    require(spec is not None and spec.loader is not None, "r11 module loader unavailable")
    previous = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(previous)
    require(previous.ROOT == ROOT and previous.SCRIPT == R11_SCRIPT, "r11 source root differs")
    return previous


def references():
    return {name: {"path": str(path), "bytes": size, "sha256": digest}
            for name, (path, size, digest) in R11_INPUTS.items()}


def retained_inputs(previous):
    previous.bound_bytes(ROOT / R11_SCRIPT, R11_SHA256)
    previous.bound_bytes(ROOT / previous.BASE_PATH, previous.BASE_SHA256)
    previous.bound_bytes(ROOT / previous.SUPPORT_PATH, previous.SUPPORT_SHA256)
    previous.prior_inputs()
    return {name: previous.bound_bytes(path, digest, size)
            for name, (path, size, digest) in R11_INPUTS.items()}


def failed_predecessor(previous, base, settings, prefix):
    raw = retained_inputs(previous)
    parsed = {name: json.loads(value) for name, value in raw.items() if name != "log"}
    receipt, report, binding = parsed["receipt"], parsed["continuation"], parsed["binding"]
    manifest, source, old_settings = parsed["manifest"], parsed["final_source"], parsed["settings"]
    refs = references()
    require(source == parsed["before_source"] == parsed["after_source"], "r11 source changed across its failed gate")
    require(receipt["schema_version"] == 1 and receipt["status"] == "blocked"
            and receipt["cleanup"] == "complete", "r11 is not the actual failed cleaned attempt")
    for key in ("cleanup_failure", "final_freeze_failure", "retention_failure", "lock_close_failure", "supervisor_failure"):
        require(key not in receipt, "r11 has an unresolved source, evidence or cleanup failure")
    require(receipt["receipt_write_failures"] == [] and receipt["diagnostic_write_failures"] == [],
            "r11 diagnostic retention was incomplete")
    cleanup = receipt["owned_resource_cleanup"]
    require(len(cleanup) == 6 and len({row["id"] for row in cleanup}) == 6
            and Counter(row["kind"] for row in cleanup) == {"container": 3, "volume": 2, "network": 1}
            and all(row["status"] == "removed" and "failure" not in row and "capture_failure" not in row for row in cleanup),
            "r11 does not establish six owned resource removals")
    for row in cleanup:
        if row["kind"] == "container":
            termination = row["termination"]
            require(termination["container"] == row["id"] and termination["completion"] == "process_exited"
                    and termination["exit_code"] == "0" and termination["attachment_failure"] is None,
                    "r11 container termination is incomplete")
    require(manifest["root"] == str(ROOT) and manifest["sourceComplete"] is True and manifest["noActiveWriters"] is True
            and manifest["stagedGitTree"] == source["staged_git_tree"] == receipt["staged_git_tree"] == R11_TREE
            and source["source_manifest_sha256"] == receipt["source_manifest_sha256"] == refs["manifest"]["sha256"]
            and receipt["source_manifest"] == refs["manifest"]["path"], "r11 source identity differs")
    allowed = {"source_manifest", "source_manifest_sha256", "staged_git_tree", "output_dir", "resource_dir"}
    require(set(settings) - {"_settings_path"} == set(old_settings)
            and all(settings[key] == value for key, value in old_settings.items() if key not in allowed),
            "r11 backend, executable, cache or settings schema changed")
    require(old_settings["source_manifest"] == refs["manifest"]["path"]
            and old_settings["source_manifest_sha256"] == refs["manifest"]["sha256"]
            and old_settings["staged_git_tree"] == R11_TREE
            and old_settings["output_dir"] == str(PREVIOUS_OUTPUT)
            and old_settings["resource_dir"] == str(PRIVATE / "backend-resources-r2"), "r11 settings identity differs")
    require(all(receipt[key] == old_settings[key] == settings[key]
                for key in ("platform", "surreal_image", "postgres_image")), "r11 backend image or platform differs")
    gates = receipt["gates"]
    require(len(gates) == 1 and gates[0]["name"] == "fresh_process_resume"
            and gates[0]["status"] == "blocked" and gates[0]["exit_code"] == 101
            and gates[0]["failure"] == receipt["failure"] == {"type": "Blocked", "reason": "fresh_process_resume failed"},
            "r11 does not contain exactly the recorded first failed gate")
    command = parsed["command"]
    require(command["argv"] == gates[0]["argv"] == base.gate_command(old_settings, "fresh_process_resume", "server-full,postgres-backend")
            and command["exit_code"] == 101 and command["status"] == "failed"
            and command["log_retained"] is True and command["log"] == refs["log"]["path"]
            and command["log_sha256"] == refs["log"]["sha256"] and command["log_bytes"] == refs["log"]["bytes"]
            and gates[0]["command_receipt"] == R11_INPUTS["command"][0].name, "r11 command or raw evidence differs")
    summaries = re.findall(r"test result: FAILED\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", raw["log"].decode())
    require(summaries == [("0", "1", "0", "0", "0")], "r11 failed target denominator differs")
    historical = {"name": "agent_threads", "executed_in_this_attempt": False, "coverage": prefix["coverage"]}
    require(report["kind"] == "r11-authoritative-failed-gate-continuation" and report["status"] == "blocked"
            and report["baseline_exit_code"] == 2 and report["current_cleanup"] == "complete"
            and report["current_failure"] == receipt["failure"]
            and report["raw_baseline_receipt"] == refs["receipt"]
            and report["source_continuity_binding"] == refs["binding"]
            and report["requested_current_execution"] == previous.EXECUTED_NAMES
            and len(report["current_gate_observations"]) == 1, "r11 authoritative receipt differs")
    observed = report["current_gate_observations"][0]
    require(all(observed[key] == gates[0][key] for key in ("name", "status", "exit_code", "argv", "command_receipt"))
            and observed["execution_evidence"] == "exit-observed" and observed["coverage_count"] is None,
            "r11 observed execution differs")
    require(report["retained_historical_prefix"] == {**historical, "source_tree": previous.OLD_TREE,
                "receipt": prefix["references"]["receipt"], "log": prefix["references"]["prefix_log"]}
            and report["prior_failure_preserved"] == prefix["references"]["failed_log"], "r11 historical prefix differs")
    require(binding["kind"] == "r11-fixed-failed-gate-continuation-binding"
            and binding["source_capture"] == "after-cleanup" and binding["historical_prefix"] == historical
            and binding["prior_inputs"] == prefix["references"] and binding["prior_tree"] == previous.OLD_TREE
            and binding["successor_tree"] == R11_TREE and binding["successor_manifest"] == refs["manifest"]["path"]
            and binding["successor_manifest_sha256"] == refs["manifest"]["sha256"]
            and binding["fixed_suffix"] == previous.EXECUTED_NAMES
            and binding["helper_preimage_sha256"] == previous.HELPER_OLD
            and binding["helper_successor_sha256"] == previous.HELPER_NEW,
            "r11 continuity binding differs")
    old_files = previous.file_inventory(manifest)
    require(len(manifest["files"]) == 45 and len(manifest["controlFiles"]) == 21
            and len(old_files) == 66 and old_files == source["files"]
            and set(old_files) == set(prefix["old_files"]) | {previous.PLAN, R11_SCRIPT}
            and all(old_files[path] == value for path, value in prefix["old_files"].items() if path != previous.HELPER)
            and old_files[previous.HELPER]["sha256"] == previous.HELPER_NEW
            and old_files[R11_SCRIPT]["sha256"] == R11_SHA256, "r11 source inventory continuity differs")
    require(manifest["pinnedDependencySources"] == prefix["manifest"]["pinnedDependencySources"] == source["pinned_dependency_sources"]
            and manifest["externalBuildInputs"] == prefix["manifest"]["externalBuildInputs"]
            and source["external_build_inputs"] == prefix["source"]["external_build_inputs"] == binding["external_build_inputs"],
            "r11 dependency or external input continuity differs")
    return {"manifest": manifest, "source": source, "files": old_files, "references": refs}


def tree_delta(freeze, before, after, expected):
    parts = freeze.git(ROOT, "diff", "--name-status", "--no-renames", "-z", before, after).split("\0")
    require(parts[-1] == "" and len(parts[:-1]) % 2 == 0, "non-parseable complete staged-tree delta")
    changes = list(zip(parts[:-1:2], parts[1:-1:2]))
    require(len(changes) == len(expected) and set(changes) == expected, "complete staged-tree delta exceeds the fixed r12 repair")
    return changes


def install_fixed_adapter(previous, base, prefix, predecessor):
    original_freeze = base.SourceFreeze

    class ContinuationFreeze(original_freeze):
        def capture(self, label):
            retained_inputs(previous)
            value = super().capture(label)
            current = base.load(self.path)
            old = predecessor["manifest"]
            require(current["baselineCommit"] == old["baselineCommit"] == prefix["manifest"]["baselineCommit"]
                    and current["branch"] == old["branch"] == prefix["manifest"]["branch"], "source baseline or branch changed")
            files = previous.file_inventory(current)
            require(len(current["files"]) == 47 and len(current["controlFiles"]) == 27
                    and len(files) == 74 and files == value["files"], "r12 source/control denominator differs")
            require({row["path"] for row in current["files"]} == {row["path"] for row in old["files"]} | {PLAN, SCRIPT}
                    and {row["path"] for row in current["controlFiles"]} == {row["path"] for row in old["controlFiles"]} | set(MODEL_FILES)
                    and set(files) == set(predecessor["files"]) | {PLAN, SCRIPT} | set(MODEL_FILES),
                    "r12 inventory changes exceed fixed source additions and model controls")
            require(all(files[path] == identity for path, identity in predecessor["files"].items() if path != previous.HELPER)
                    and files[previous.HELPER]["sha256"] == HELPER_NEW and files[previous.HELPER]["mode"] == "0644"
                    and files[PLAN]["mode"] == "0644" and files[SCRIPT]["mode"] == "0644",
                    "r12 changed preserved source or lacks the exact reviewed helper")
            require(all(files[path] == identity for path, identity in MODEL_FILES.items()), "shipped model control bytes or modes changed")
            require(current["pinnedDependencySources"] == old["pinnedDependencySources"]
                    and value["pinned_dependency_sources"] == predecessor["source"]["pinned_dependency_sources"]
                    and current["externalBuildInputs"] == old["externalBuildInputs"]
                    and value["external_build_inputs"] == predecessor["source"]["external_build_inputs"],
                    "dependency or external input continuity changed")
            for key in ("allowedUnstagedMetadataPaths", "allowedUntrackedMetadataRoots"):
                require(current[key] == old[key] == prefix["manifest"][key], "metadata exclusion scope changed")
            recent = tree_delta(self, R11_TREE, value["staged_git_tree"], {("M", previous.HELPER), ("A", PLAN), ("A", SCRIPT)})
            cumulative = tree_delta(self, previous.OLD_TREE, value["staged_git_tree"],
                                    {("M", previous.HELPER), ("A", previous.PLAN), ("A", R11_SCRIPT), ("A", PLAN), ("A", SCRIPT)})
            # The current files are hashed by SourceFreeze. These immutable raw
            # Git objects additionally establish that the assets predate r11/r12.
            entries = self.git(ROOT, "ls-tree", "-r", "-l", "-z", previous.OLD_TREE, "--", MODEL_ROOT).split("\0")
            require(entries[-1] == "", "non-parseable historical model tree")
            blobs = {}
            for entry in entries[:-1]:
                metadata, path = entry.split("\t", 1)
                require(path not in blobs, "duplicate historical model path")
                blobs[path] = metadata.split()
            expected_blobs = {MODEL_ROOT + "/" + name: ["100644", "blob", blob, str(size)]
                              for name, size, _, blob in MODEL_ROWS}
            require(blobs == expected_blobs, "historical raw model blobs differ")
            base.save(self.resources.evidence / "continuation-binding.json", {
                "kind": "r12-fixed-model-input-continuation-binding", "source_capture": label,
                "r10_inputs": prefix["references"], "r11_inputs": predecessor["references"],
                "r10_tree": previous.OLD_TREE, "r11_tree": R11_TREE,
                "successor_tree": value["staged_git_tree"], "successor_manifest": str(self.path),
                "successor_manifest_sha256": self.expected, "fixed_suffix": previous.EXECUTED_NAMES,
                "historical_prefix": {"name": "agent_threads", "executed_in_this_attempt": False, "coverage": prefix["coverage"]},
                "r11_tree_delta": recent, "r10_tree_delta": cumulative,
                "helper_preimage_sha256": previous.HELPER_NEW, "helper_successor_sha256": HELPER_NEW,
                "shipped_model_controls": MODEL_FILES, "original_model_blobs": expected_blobs,
                "external_build_inputs": value["external_build_inputs"]})
            return value

    base.SourceFreeze = ContinuationFreeze
    base.TARGETS = previous.SUFFIX


def continuation_result(previous, base, settings, prefix, predecessor, code, failure):
    output = Path(settings["output_dir"])
    report = {"schema_version": 1, "kind": "r12-authoritative-model-input-continuation", "status": "blocked",
              "finished_at": time.time(), "baseline_exit_code": code,
              "retained_historical_prefix": {"name": "agent_threads", "executed_in_this_attempt": False,
                  "source_tree": previous.OLD_TREE, "coverage": prefix["coverage"],
                  "receipt": prefix["references"]["receipt"], "log": prefix["references"]["prefix_log"]},
              "failed_predecessors": {"r10": {"receipt": prefix["references"]["receipt"], "log": prefix["references"]["failed_log"]},
                  "r11": {key: predecessor["references"][key] for key in ("receipt", "continuation", "binding", "log")}},
              "requested_current_execution": previous.EXECUTED_NAMES, "current_gate_observations": [],
              "limits": ["Only the historical ten-test prefix is adopted; failed attempts supply no passing recovery coverage.",
                         "The raw baseline four-target family label does not claim four current test executions.",
                         "Three full integration targets and the customer build stop at the first failure, missing evidence or drift.",
                         "Unbounded waits, permanent-I/O evidence limits, warnings/submission holds and broader release/consumer gates remain.",
                         "Source binding does not certify model loading or negative/tamper behavior."]}
    if failure is not None:
        report["invocation_failure"] = failure
    try:
        retained_inputs(previous)
        raw_path = output / "receipt.json"
        raw = base.load(raw_path)
        report["raw_baseline_receipt"] = previous.reference(base, raw_path)
        report["raw_baseline_family_scope_label"] = raw["scope"]
        require(raw["source_manifest_sha256"] == settings["source_manifest_sha256"]
                and raw["staged_git_tree"] == settings["staged_git_tree"], "current receipt source identity differs")
        gates = raw["gates"]
        require(len(gates) <= 4 and [row["name"] for row in gates] == previous.EXECUTED_NAMES[:len(gates)],
                "current gate sequence differs from the fixed suffix")
        for row in gates:
            name = row["name"]
            if name == "customer-build":
                expected = base.gate_command(settings)
            else:
                features = next(features for target, features, _ in previous.SUFFIX if target == name)
                expected = base.gate_command(settings, name, features)
            require(row["argv"] == expected, "current command differs from the fixed whole-target contract")
            report["current_gate_observations"].append({"name": name, "status": row["status"], "argv": row["argv"],
                "exit_code": row.get("exit_code"), "command_receipt": row.get("command_receipt"),
                "execution_evidence": "exit-observed" if row.get("exit_code") is not None else "not-established-by-gate-receipt",
                "coverage_count": row.get("coverage", {}).get("count")})
        binding = output / "continuation-binding.json"
        if binding.is_file():
            report["source_continuity_binding"] = previous.reference(base, binding)
        if code == 0:
            require(failure is None and raw["status"] == "passed" and raw["cleanup"] == "complete"
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
    print("Fixed r12 continuation " + report["status"] + "; authoritative private receipt: " + str(output / "continuation-receipt.json"))
    return 0 if report["status"] == "passed" else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--settings", required=True)
    args = parser.parse_args()
    try:
        require(Path(args.settings) == PRIVATE / "backend-settings-r12.json", "r12 has one fixed private settings path")
        previous = import_predecessor()
        base = previous.import_baseline()
        settings = base.configure(args.settings)
        require(settings["source_manifest"] == str(HANDOFF / "source-manifest-r12.json")
                and settings["output_dir"] == str(PRIVATE / "backend-evidence-r3")
                and settings["resource_dir"] == str(PRIVATE / "backend-resources-r3"), "r12 requires its fixed fresh manifest/evidence/resources")
        # acceptance('agent_threads') needs the original four-target table.
        # Parse once before selecting the suffix; later captures rebind its bytes.
        prefix = previous.retained_prefix(base, settings)
        predecessor = failed_predecessor(previous, base, settings, prefix)
        install_fixed_adapter(previous, base, prefix, predecessor)
    except Exception as error:
        print("Continuation blocked before execution: " + str(error), file=sys.stderr)
        return 2
    code, failure = None, None
    try:
        code = base.execute(settings)
    except BaseException as error:
        failure = {"type": type(error).__name__, "reason": str(error)}
    return continuation_result(previous, base, settings, prefix, predecessor, code, failure)


if __name__ == "__main__":
    raise SystemExit(main())
