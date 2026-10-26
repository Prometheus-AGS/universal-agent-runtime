"""Author an exact source inventory, without running product verification.

Generated credentials, build outputs and authority are separate derived inputs.
This program does not certify implementation or grant execution permission.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


EXCLUDED = {".git", ".kbd-orchestrator", ".prometheus", ".agent-team", ".codex",
            ".claude", ".agents", ".cursor", ".opencode", "compass-out", "target",
            "node_modules", ".pnpm-store", "dist", "coverage", "test-results",
            "playwright-report", ".next", ".turbo", "__pycache__", ".venv", "venv", "openspec",
            ".acceptance-artifacts"}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def selected(relative):
    # Static and website output belong to the installed artifact closure, not
    # a source identity which would change merely because the SPA was built.
    return not any(part in EXCLUDED for part in relative.parts) and relative.parts[0] != "static"


def files(checkout, root, visited):
    resolved = checkout.resolve()
    if resolved in visited:
        return
    visited.add(resolved)
    raw = subprocess.run(["git", "-C", str(checkout), "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
                         check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout
    for name in sorted(set(os.fsdecode(item) for item in raw.split(b"\0") if item)):
        path = checkout / name
        relative = path.relative_to(root)
        if not selected(relative):
            continue
        if path.is_symlink():
            yield {"path": str(path), "kind": "symlink", "target": os.readlink(path),
                   "resolved_path": str(path.resolve()),
                   "sha256": sha(path.read_bytes()) if path.is_file() else None}
        elif path.is_file():
            raw_file = path.read_bytes()
            yield {"path": str(path), "kind": "regular", "bytes": len(raw_file), "sha256": sha(raw_file)}
        elif path.is_dir():
            # A tracked directory is normally a submodule. Its dirty working
            # bytes, including the local Liter patch, matter beyond its gitlink.
            yield from files(path, root, visited)
        else:
            yield {"path": str(path), "kind": "missing", "sha256": None}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    phase = root / ".kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness"
    if not args.output.is_absolute() or args.output.parent != phase or args.output.exists():
        raise ValueError("inventory requires a new exact file under the active phase")
    waypoint = json.loads((root / ".kbd-orchestrator/current-waypoint.json").read_bytes())
    manifest = json.loads((root / "scripts/acceptance/fixtures/scenarios.json").read_bytes())
    records = sorted(files(root, root, set()), key=lambda item: item["path"])
    regular = [{"path": item["path"], "sha256": item["sha256"]} for item in records if item["kind"] == "regular"]
    unresolved = [item for item in records if item["kind"] != "regular"]
    head = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], check=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout.decode().strip()
    value = {"schema_version": 1, "status": "SOURCE_SNAPSHOT_WITH_DERIVED_INPUTS_PENDING",
             "verification_gate": "CLOSED", "verification_ready": False, "phase_acceptance": False,
             "root": str(root), "head": head,
             "position": {key: waypoint.get(key) for key in ("activePhase", "stageId", "revision", "planRevision", "currentTask")},
             "source_identity": sha(canonical(regular)), "source_files": [item["path"] for item in regular],
             "files": records, "regular_file_count": len(regular), "unresolved_source_entries": unresolved,
             "source_scope": "Git tracked/nonignored working files recursively through selected submodules; exact dirty bytes",
             "excluded_path_components": sorted(EXCLUDED), "additional_excluded_output": "root static/",
             "governance_scope": "Canonical KBD/authority, append-only memory and team handoffs remain separate records, not build-source identity",
             "planned_cases": manifest["source_authored_case_count"], "planned_rows": manifest["planned_backend_repetition_rows"],
             "pending_derived_inputs": ["Operator-approved successful customer build and immutable installed artifact snapshot",
                "Complete installed SDK/browser/encoder/Python/Node/tool/model-asset closures",
                "Owned private credential/configuration preparation and immutable local datastore images",
                "Current Cohere PAYG conservative charge ceilings and explicit resource approval",
                "Reconciled prior consumption, unknown-reservation disposition and actual signed adoption",
                "Final deterministic fixture composition, separate derived-input receipt and exact gate instruction"],
             "limitation": "Source identity is not an installed artifact, usable fixture, whole-phase execution approval or runtime PASS"}
    with args.output.open("xb") as handle:
        handle.write(canonical(value) + b"\n")
        handle.flush()
        os.fsync(handle.fileno())
    args.output.chmod(0o400)
    print(json.dumps({"inventory": str(args.output), "source_identity": value["source_identity"],
                      "regular_files": len(regular), "unresolved_entries": len(unresolved), "gate": "CLOSED"}))


if __name__ == "__main__":
    main()
