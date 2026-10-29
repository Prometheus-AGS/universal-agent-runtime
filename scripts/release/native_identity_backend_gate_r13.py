#!/usr/bin/env python3
"""Fixed r13 full backend gate after process-resident approval routing changes.

Requires separate approval of the completed frozen source and private settings.
Imports the unchanged baseline directly and runs all FOUR targets and its exact
customer build. Earlier results are retained history, never current pass credit.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[2]
HANDOFF = ROOT / ".agent-team/uar-core/handoffs/kubo-native-identity"
PRIVATE = Path("/Volumes/T7/pr13-native-identity-r9-private")
R12_OUTPUT = PRIVATE / "backend-evidence-r3"
R12_TREE = "fc8a9de51436f1656fea45f8ae9da0a4fe404f99"
R10_TREE = "9078a8706dfcbd5f474c9e5b88dbdda769d26fee"
BASE_PATH = "scripts/release/native_identity_backend_gate.py"
BASE_SHA256 = "6389700f122977be2e059c1ce95946603fc70e21a3bef503fe36079bb8efb634"
SUPPORT_PATH = "scripts/release/uar_1_1_1_support.py"
SUPPORT_SHA256 = "2ba73ad2ec3a6214fa81ae587750e5c18f29a9e0998a7b5dcb280129f97a1073"
CHANGE = "openspec/changes/preserve-native-tool-call-identity/"
PLAN = CHANGE + "cold-inspection-approval-routing-r13-plan.md"
SCRIPT = "scripts/release/native_identity_backend_gate_r13.py"
ROUTES = "src/uar/api/routes.rs"
ADDED_PRODUCERS = {ROUTES, PLAN, SCRIPT}
CHANGED_EXISTING = {
    "src/uar/runtime/manager.rs",
    "tests/support/fresh_process_resume_process.rs",
    "tests/support/approval_recovery_process.rs",
    "docs/operations/approval-recovery.md",
    CHANGE + "specs/tool-approval-workflow/spec.md",
    CHANGE + "design.md",
    CHANGE + "tasks.md",
}
FULL_TARGETS = (
    ("agent_threads", "server-full,postgres-backend,in-memory-backend", 10),
    ("fresh_process_resume", "server-full,postgres-backend", 1),
    ("stream_terminal_contract", "server-full", 1),
    ("approval_recovery", "server-full,postgres-backend,in-memory-backend", 1),
)
# Fixed historical input bytes from continuation-input-identities-r12.json.
# They preserve the failed attempt and source boundary, not a reusable pass.
HISTORY = {
    "receipt": (R12_OUTPUT / "receipt.json", 5462,
                "acc0c2321da3661e3366870d935bd1a25cbe7d03ef78564c05214d78a67bf163"),
    "continuation": (R12_OUTPUT / "continuation-receipt.json", 5236,
                     "7eac92f96390160a419e2fd7805e71506d608366cf696b66c4cc01247bbc596a"),
    "binding": (R12_OUTPUT / "continuation-binding.json", 9614,
                "70e009eafd3aad084f490fabebadcbc729aa51518d438776386ff49340776188"),
    "command": (R12_OUTPUT / "command-00136.json", 936,
                "f208bf4a20b2102dc7808bbbd0e0cd812bfc211527406f764c59926f67a88cf0"),
    "log": (R12_OUTPUT / "command-00136.log", 2199,
            "1e8710127883ef9d8d5d6d28615aa990733b547719748a4c258ac7470695fc8a"),
    "before_source": (R12_OUTPUT / "source-before-fresh_process_resume.json", 16594,
                      "46425bb5d79fe7923c40b4bcd84f34b1d80128e78df61e417af64c2c984dd981"),
    "after_source": (R12_OUTPUT / "source-after-fresh_process_resume.json", 16594,
                     "46425bb5d79fe7923c40b4bcd84f34b1d80128e78df61e417af64c2c984dd981"),
    "final_source": (R12_OUTPUT / "source-after-cleanup.json", 16594,
                     "46425bb5d79fe7923c40b4bcd84f34b1d80128e78df61e417af64c2c984dd981"),
    "manifest": (HANDOFF / "source-manifest-r12.json", 32652,
                 "c09f044c74a7f7aaad02ae82a23f00b32b92f9575211457b1f1198361005d764"),
    "settings": (PRIVATE / "backend-settings-r12.json", 1478,
                 "40048b1ae6af955731ed0554eb940f6a24810fbe1b07ee4897a6a3639a22872a"),
    "last_response": (PRIVATE / "backend-resources-r3/dot06-evidence/local-resume/current/last-response.json", 1299,
                      "9a286a2602538bf20c5cdf19975e5a1a94ed6224c16e463fc1aea19af4641645"),
}
MODEL_ROOT = "src/uar/runtime/matching/models"
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


class GateBlocked(RuntimeError):
    pass


def require(condition, reason):
    if not condition:
        raise GateBlocked(reason)


def bound_bytes(path, digest, size=None):
    require(path.is_file() and not path.is_symlink(), "required immutable input missing: " + str(path))
    raw = path.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == digest and (size is None or len(raw) == size),
            "immutable input changed: " + str(path))
    return raw


def preserved_history():
    bound_bytes(ROOT / BASE_PATH, BASE_SHA256)
    bound_bytes(ROOT / SUPPORT_PATH, SUPPORT_SHA256)
    return {name: bound_bytes(path, digest, size) for name, (path, size, digest) in HISTORY.items()}


def references():
    return {name: {"path": str(path), "bytes": size, "sha256": digest}
            for name, (path, size, digest) in HISTORY.items()}


def import_baseline():
    bound_bytes(ROOT / BASE_PATH, BASE_SHA256)
    bound_bytes(ROOT / SUPPORT_PATH, SUPPORT_SHA256)
    spec = importlib.util.spec_from_file_location("native_identity_backend_r13_baseline", ROOT / BASE_PATH)
    require(spec is not None and spec.loader is not None, "baseline module loader unavailable")
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    require(base.ROOT == ROOT and base.TARGETS == FULL_TARGETS, "baseline full target contract differs")
    return base


def file_inventory(manifest):
    result = {}
    for row in manifest["files"] + manifest["controlFiles"]:
        require(row["path"] not in result, "duplicate source/control inventory path")
        result[row["path"]] = {key: row[key] for key in ("bytes", "mode", "sha256")}
    return result


def historical_boundary(settings):
    raw = preserved_history()
    manifest = json.loads(raw["manifest"])
    source = json.loads(raw["final_source"])
    old_settings = json.loads(raw["settings"])
    receipt = json.loads(raw["receipt"])
    require(source == json.loads(raw["before_source"]) == json.loads(raw["after_source"]),
            "historical source snapshots differ")
    require(manifest["root"] == str(ROOT) and manifest["sourceComplete"] is True and manifest["noActiveWriters"] is True
            and manifest["stagedGitTree"] == source["staged_git_tree"] == receipt["staged_git_tree"] == R12_TREE
            and source["source_manifest_sha256"] == receipt["source_manifest_sha256"] == HISTORY["manifest"][2]
            and receipt["source_manifest"] == str(HISTORY["manifest"][0]), "historical source boundary differs")
    require(receipt["status"] == "blocked" and receipt["cleanup"] == "complete",
            "historical attempt is not the recorded blocked, cleaned outcome")
    files = file_inventory(manifest)
    require(len(manifest["files"]) == 47 and len(manifest["controlFiles"]) == 27
            and len(files) == 74 and files == source["files"], "historical inventory denominator differs")
    allowed = {"source_manifest", "source_manifest_sha256", "staged_git_tree", "output_dir", "resource_dir"}
    require(set(settings) - {"_settings_path"} == set(old_settings)
            and all(settings[key] == value for key, value in old_settings.items() if key not in allowed),
            "backend, platform, executable, cache or settings schema changed")
    require(old_settings["source_manifest"] == str(HISTORY["manifest"][0])
            and old_settings["source_manifest_sha256"] == HISTORY["manifest"][2]
            and old_settings["staged_git_tree"] == R12_TREE and old_settings["output_dir"] == str(R12_OUTPUT)
            and old_settings["resource_dir"] == str(PRIVATE / "backend-resources-r3"), "historical settings boundary differs")
    require(all(receipt[key] == settings[key] for key in ("platform", "surreal_image", "postgres_image")),
            "historical backend image or platform differs")
    require(manifest["pinnedDependencySources"] == source["pinned_dependency_sources"]
            and [{key: row[key] for key in ("path", "bytes", "sha256", "kind")} for row in manifest["externalBuildInputs"]]
            == source["external_build_inputs"], "historical dependency or external input identity differs")
    return {"manifest": manifest, "source": source, "files": files}


def install_freeze(base, history):
    original_freeze = base.SourceFreeze

    class FullGateFreeze(original_freeze):
        def capture(self, label):
            preserved_history()
            require(base.TARGETS == FULL_TARGETS, "full-target execution contract changed")
            value = super().capture(label)
            current = base.load(self.path)
            old = history["manifest"]
            files = file_inventory(current)
            require(current["baselineCommit"] == old["baselineCommit"] and current["branch"] == old["branch"],
                    "source baseline or branch changed")
            require(len(current["files"]) == 50 and len(current["controlFiles"]) == 27
                    and len(files) == 77 and files == value["files"], "r13 source/control denominator differs")
            require({row["path"] for row in current["files"]} == {row["path"] for row in old["files"]} | ADDED_PRODUCERS
                    and {row["path"] for row in current["controlFiles"]} == {row["path"] for row in old["controlFiles"]}
                    and set(files) == set(history["files"]) | ADDED_PRODUCERS,
                    "source inventory changes exceed the fixed r13 scope")
            require(all(files[path] == identity for path, identity in history["files"].items() if path not in CHANGED_EXISTING),
                    "immutable predecessor source/control bytes or modes changed")
            require(all(files[path]["mode"] == "0644" for path in CHANGED_EXISTING | ADDED_PRODUCERS),
                    "r13 changed source requires regular 0644 files")
            require(all(files[path] == identity for path, identity in MODEL_FILES.items()), "shipped model identity changed")
            require(current["pinnedDependencySources"] == old["pinnedDependencySources"]
                    and value["pinned_dependency_sources"] == history["source"]["pinned_dependency_sources"]
                    and current["externalBuildInputs"] == old["externalBuildInputs"]
                    and value["external_build_inputs"] == history["source"]["external_build_inputs"],
                    "dependency or external Cargo input continuity changed")
            for key in ("allowedUnstagedMetadataPaths", "allowedUntrackedMetadataRoots"):
                require(current[key] == old[key], "metadata exclusion scope changed")
            parts = self.git(ROOT, "diff", "--name-status", "--no-renames", "-z", R12_TREE, value["staged_git_tree"]).split("\0")
            require(parts[-1] == "" and len(parts[:-1]) % 2 == 0, "non-parseable complete staged-tree delta")
            changes = list(zip(parts[:-1:2], parts[1:-1:2]))
            # routes.rs is new to the exact-path inventory, but already tracked
            # in r12's complete Git tree. It is a modification, not a new file.
            expected = {("M", path) for path in CHANGED_EXISTING | {ROUTES}} | {("A", PLAN), ("A", SCRIPT)}
            require(len(changes) == len(expected) and set(changes) == expected, "complete staged-tree delta exceeds or omits r13 scope")
            entries = self.git(ROOT, "ls-tree", "-r", "-l", "-z", R10_TREE, "--", MODEL_ROOT).split("\0")
            require(entries[-1] == "", "non-parseable original model tree")
            blobs = {}
            for entry in entries[:-1]:
                metadata, path = entry.split("\t", 1)
                require(path not in blobs, "duplicate historical model path")
                blobs[path] = metadata.split()
            expected_blobs = {MODEL_ROOT + "/" + name: ["100644", "blob", blob, str(size)]
                              for name, size, _, blob in MODEL_ROWS}
            require(blobs == expected_blobs, "original shipped raw model blobs differ")
            base.save(self.resources.evidence / "r13-source-binding.json", {
                "kind": "r13-full-backend-gate-source-binding", "source_capture": label,
                "historical_inputs": references(), "historical_tree": R12_TREE,
                "historical_acceptance_adopted": False,
                "historical_ten_test_result": "retained history only; agent_threads must execute again",
                "current_targets": [row[0] for row in FULL_TARGETS] + ["customer-build"],
                "successor_tree": value["staged_git_tree"], "successor_manifest": str(self.path),
                "successor_manifest_sha256": self.expected, "allowed_tree_delta": changes,
                "shipped_model_controls": MODEL_FILES, "original_model_blobs": expected_blobs,
                "external_build_inputs": value["external_build_inputs"]})
            return value

    # No target-table mutation, acceptance override or continuation import.
    base.SourceFreeze = FullGateFreeze


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--settings", required=True)
    args = parser.parse_args()
    try:
        require(Path(args.settings) == PRIVATE / "backend-settings-r13.json", "r13 has one fixed private settings path")
        base = import_baseline()
        settings = base.configure(args.settings)
        require(settings["source_manifest"] == str(HANDOFF / "source-manifest-r13.json")
                and settings["output_dir"] == str(PRIVATE / "backend-evidence-r4")
                and settings["resource_dir"] == str(PRIVATE / "backend-resources-r4"), "r13 requires its fixed fresh manifest/evidence/resources")
        history = historical_boundary(settings)
        install_freeze(base, history)
    except Exception as error:
        print("R13 blocked before execution: " + str(error), file=sys.stderr)
        return 2
    # The baseline receipt is authoritative for these four current targets and
    # exact customer build. Historical artifacts cannot supply a passed gate.
    return base.execute(settings)


if __name__ == "__main__":
    raise SystemExit(main())
