#!/usr/bin/env python3
"""Real integration of this phase's known scenarios, after full implementation.

No process starts on import. The manifest binds the complete implementation
inventory, frozen sources and either a local candidate or immutable child image.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

from uar_1_1_1_support import Blocked, GROUPS, ROOT, Resources, require

PHASE = "uar-outstanding-issues-v1-1-1"
REPOSITORY = "Prometheus-AGS/universal-agent-runtime"
IMAGE_REPOSITORY = "ghcr.io/prometheus-ags/universal-agent-runtime"
SOURCE_PATHS = ("src", "Cargo.toml", "Cargo.lock", "build.rs", "Dockerfile",
                "migrations", "policies", "versions.toml", "proto", "static", "catalog",
                "package.json", "pnpm-lock.yaml", "vendor", "crates", "frontend",
                "docs/agents", "scripts", ".cargo", ".dockerignore", ".gitmodules",
                "rust-toolchain.toml")
FIXTURE_PATHS = ("tests", "scripts/release")
ADAPTERS = {"binding": "runtime", "streams": "runtime", "mcp-outcomes": "runtime",
            "credentials": "credentials", "recovery": "recovery", "combined": "runtime"}
NATIVE_CASES = {
    "binding": (("bauar_full_harness_cursor",
                 "phase_runtime_binding_refusals_use_real_host_store_and_provider"),),
    "mcp-outcomes": (("bauar_full_harness_cursor",
                      "phase_runtime_native_incidental_error_retains_success"),),
    "credentials": (("credentials_api_integration_test",
                     "phase_native_sdk_supplied_credentials_use_real_provider_and_durable_store"),
                    ("credentials_api_integration_test",
                     "phase_memory_credential_migration_uses_real_http_and_atomic_store")),
}


def digest_bytes(value):
    return "sha256:" + hashlib.sha256(value).hexdigest()


def source_identity(paths):
    listing = subprocess.run(["git", "ls-files", "-z", "--cached", "--others",
                              "--exclude-standard", "--", *paths], cwd=ROOT,
                             capture_output=True, check=True).stdout
    records = []
    for raw in sorted(set(part for part in listing.split(b"\0") if part)):
        path = ROOT / raw.decode()
        if path.is_symlink():
            import os
            data = ("symlink:" + os.readlink(path)).encode()
        elif path.is_file():
            data = path.read_bytes()
        elif path.is_dir():
            commit = subprocess.run(["git", "-C", str(path), "rev-parse", "HEAD"],
                                    capture_output=True, check=True).stdout.strip()
            dirty = subprocess.run(["git", "-C", str(path), "status", "--porcelain"],
                                   capture_output=True, check=True).stdout
            require(not dirty, "pinned submodule has unrecorded changes: " + raw.decode())
            data = b"gitlink:" + commit
        else:
            data = b"deleted"
        records.append(raw + b"\0" + hashlib.sha256(data).digest())
    return digest_bytes(b"\n".join(records))


def load_adapter(kind):
    path = ROOT / ("scripts/release/uar_1_1_1_" + kind + ".py")
    require(path.is_file(), "missing implemented scenario adapter: " + str(path))
    name = "uar111_" + kind
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, "adapter cannot be loaded")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def verify_publication(resources, manifest):
    """Bind the immutable image to GitHub's actual successful publishing run."""
    reference = manifest.get("publication_receipt")
    require(isinstance(reference, dict), "published acceptance requires a publication receipt")
    path = Path(reference["path"])
    require(path.is_absolute(), "absolute publication receipt path required")
    raw = path.read_bytes()
    require(digest_bytes(raw) == reference["digest"], "publication receipt changed")
    publication = json.loads(raw)
    require(publication.get("schema_version") == 1
            and publication.get("repository") == REPOSITORY
            and publication.get("tag") == "v1.1.1"
            and publication.get("source_commit") == manifest["source_commit"]
            and publication.get("index_digest") == manifest["index_digest"],
            "publication receipt does not bind the selected release")
    platform = manifest["platform"]
    children = publication["children"]
    require(set(children) == {"linux/arm64", "linux/amd64"}, "publication lacks both child digests")
    require(manifest["image"] == IMAGE_REPOSITORY + "@" + children[platform],
            "tested child differs from the publishing receipt")
    run_id = publication["workflow_run_id"]
    require(type(run_id) is int and run_id > 0, "invalid publishing run identity")
    prefix = "repos/" + REPOSITORY

    def github(route):
        return json.loads(resources.run_command(["gh", "api", prefix + route]).stdout)

    remote_tag = github("/git/ref/tags/v1.1.1")
    require(remote_tag["object"]["type"] == "tag", "published tag must be annotated")
    tag = github("/git/tags/" + remote_tag["object"]["sha"])
    require(tag["object"]["type"] == "commit"
            and tag["object"]["sha"] == manifest["source_commit"], "remote release tag source differs")
    run = github("/actions/runs/" + str(run_id))
    require(run["id"] == run_id and run["head_sha"] == manifest["source_commit"]
            and run["event"] == "push"
            and run["path"].split("@", 1)[0] == ".github/workflows/deploy.yml"
            and run["status"] == "completed" and run["conclusion"] == "success"
            and run["run_attempt"] == publication["workflow_run_attempt"]
            and run["repository"]["full_name"].lower() == REPOSITORY.lower(),
            "GitHub publishing run does not prove the selected tag/source")
    artifacts = github("/actions/runs/" + str(run_id) + "/artifacts?per_page=100")
    observed = {}
    for selected in ("linux/arm64", "linux/amd64"):
        name = "digests-" + selected.replace("/", "-")
        matches = [item for item in artifacts["artifacts"] if item["name"] == name]
        require(len(matches) == 1 and not matches[0]["expired"],
                "actual publishing digest artifact unavailable: " + selected)
        artifact = matches[0]
        require(artifact["id"] == publication["artifact_ids"][selected],
                "digest artifact belongs to another run/source")
        destination = resources.root / resources.unique_name("publishing-digest")
        destination.mkdir(mode=0o700)
        resources.run_command(["gh", "run", "download", str(run_id), "--repo", REPOSITORY,
                               "--name", name, "--dir", str(destination)])
        files = list(destination.rglob("*"))
        expected = children[selected]
        require(expected.startswith("sha256:") and len(expected) == 71,
                "invalid published child digest")
        require(len(files) == 1 and files[0].is_file() and not files[0].is_symlink()
                and files[0].name == expected.removeprefix("sha256:")
                and files[0].read_bytes() == b"", "actual workflow export differs from selected child")
        observed[selected] = dict(artifact_id=artifact["id"], name=name, child_digest=expected,
                                  artifact_digest=artifact.get("digest"))
    return dict(receipt=reference, source_commit=manifest["source_commit"],
                workflow_run_id=run_id, workflow_run_attempt=run["run_attempt"],
                workflow_url=run["html_url"], head_branch=run["head_branch"], artifacts=observed)


def inspect_image(resources, manifest):
    provenance = None
    if manifest["stage"] == "published":
        provenance = verify_publication(resources, manifest)
        require("@sha256:" in manifest["image"], "published UAR requires an immutable child digest")
        resources.docker("pull", "--platform", manifest["platform"], manifest["image"], timeout=1800)
        repository = manifest["image"].split("@", 1)[0]
        index = json.loads(resources.docker("buildx", "imagetools", "inspect", "--raw",
                                           repository + "@" + manifest["index_digest"]).stdout)
        expected = manifest["image"].split("@", 1)[1]
        architecture = manifest["platform"].split("/", 1)[1]
        require(any(item.get("digest") == expected and item.get("platform", {}).get("architecture") == architecture
                    for item in index.get("manifests", [])), "child is not a member of the released index/platform")
        for selected, artifact in provenance["artifacts"].items():
            child_architecture = selected.split("/", 1)[1]
            matches = [item for item in index["manifests"]
                       if item.get("platform", {}).get("os") == "linux"
                       and item.get("platform", {}).get("architecture") == child_architecture]
            require(len(matches) == 1 and matches[0]["digest"] == artifact["child_digest"],
                    "released index does not contain the publishing run's exact children")
    else:
        require("@" not in manifest["image"], "local candidate must use an explicit local image label")
        resources.docker("buildx", "build", "--load", "--platform", manifest["platform"],
                         "--tag", manifest["image"], "--file", "Dockerfile", ".", timeout=14400)
    data = json.loads(resources.docker("image", "inspect", manifest["image"]).stdout)[0]
    actual = data["Os"] + "/" + data["Architecture"]
    require(actual == manifest["platform"], "runtime image architecture mismatch")
    return {"image_id": data["Id"], "repo_digests": data.get("RepoDigests", []), "platform": actual,
            "publication_provenance": provenance}


def check_result(group, module, result):
    required = module.REQUIRED_SCENARIOS[group]
    require(isinstance(required, (tuple, list)) and required, "mandatory scenario inventory is empty")
    require(len(required) == len(set(required)), "duplicate mandatory scenario identity")
    scenarios = result.get("scenarios", [])
    ids = [item.get("id") for item in scenarios]
    require(len(ids) == len(set(ids)), "duplicate observed scenario identity")
    require(set(ids) == set(required), "scenario inventory mismatch for " + group)
    for item in scenarios:
        if item.get("status") == "blocked":
            raise Blocked("required scenario blocked: " + str(item.get("id")))
        require(item.get("status") == "passed", "required scenario failed: " + str(item.get("id")))
        require(isinstance(item.get("observations"), (dict, list)) and item["observations"],
                "scenario lacks observable production evidence: " + str(item.get("id")))
    return result


def run_native_integration(resources, group):
    """Named real host integrations for seams without an installed HTTP input."""
    observations = []
    for target, case in NATIVE_CASES.get(group, ()):
        argv = ["cargo", "test", "--locked", "--no-default-features", "--features",
                "server-full,postgres-backend,in-memory-backend", "--test",
                target, case, "--", "--exact", "--test-threads=1"]
        result = resources.run_command(argv, timeout=14400, env={
            "CARGO_TARGET_DIR": str(ROOT / "target/uar-1-1-1-phase"),
            "UAR_PHASE_RESOURCE_ROOT": str(resources.root),
            "UAR_PHASE_EVIDENCE_ROOT": str(resources.evidence),
            "UAR_LOCAL_ONLY": "1",
        })
        require("test result: ok. 1 passed; 0 failed;" in result.stdout,
                "the selected real integration case did not execute and pass: " + case)
        observations.append(dict(target=target, case=case, argv=argv, status="passed",
                                 scope="native host with real provider/store collaborators; supplemental to image observations"))
    return observations


def read_manifest(path):
    value = json.loads(path.read_text())
    require(value.get("schema_version") == 1 and value.get("phase") == PHASE
            and value.get("version") == "1.1.1", "wrong manifest identity/version")
    require(value.get("stage") in ("local", "published"), "explicit execution stage required")
    require(value.get("platform") in ("linux/arm64", "linux/amd64"), "unsupported runtime platform")
    groups = value.get("groups")
    if groups != list(GROUPS):
        require(isinstance(groups, list) and len(groups) == 1 and groups[0] in GROUPS,
                "select all phase groups or one previously failed group")
        parent = value.get("repair_parent")
        require(parent, "repair rerun requires the retained parent receipt")
        receipt = json.loads(Path(parent["path"]).read_text())
        require(digest_bytes(Path(parent["path"]).read_bytes()) == parent["digest"], "parent receipt digest changed")
        require(all(receipt.get(key) == value.get(key)
                    for key in ("phase", "version", "stage", "platform", "image")),
                "repair parent belongs to another execution artifact")
        require(receipt.get("source_tree") == value["source_tree"] or value.get("repair_source_change"),
                "repair must disclose source changes")
        require(receipt.get("fixture_tree") == value["fixture_tree"] or value.get("repair_source_change"),
                "repair must disclose fixture changes")
        require(receipt.get("groups", {}).get(groups[0], {}).get("status") in ("failed", "blocked", "interrupted"),
                "repair selection is not a failed parent gate")
    for key in ("resource_root", "evidence_root", "infrastructure_file"):
        require(Path(value[key]).is_absolute(), "absolute private manifest paths required")
    inventory_ref = value["implementation_inventory"]
    raw = Path(inventory_ref["path"]).read_bytes()
    require(digest_bytes(raw) == inventory_ref["digest"], "implementation inventory changed")
    inventory = json.loads(raw)
    require(inventory.get("phase") == PHASE and inventory.get("implementation_complete") is True,
            "entire phase implementation is not complete")
    require(inventory.get("boundary") == "open" and inventory.get("source_tree") == value["source_tree"]
            and inventory.get("fixture_tree") == value["fixture_tree"], "candidate/inventory identity mismatch")
    require(inventory.get("items") and all(item.get("implemented") is True and not item.get("unresolved")
                                        for item in inventory["items"]), "unfinished implementation item")
    require(source_identity(SOURCE_PATHS) == value["source_tree"], "production source changed after freeze")
    require(source_identity(FIXTURE_PATHS) == value["fixture_tree"], "fixture sources changed after freeze")
    if value["stage"] == "published":
        require(isinstance(value.get("index_digest"), str) and value["index_digest"].startswith("sha256:"),
                "published index digest required")
        head = subprocess.run(["git", "rev-parse", "v1.1.1^{commit}"], cwd=ROOT,
                              capture_output=True, check=True).stdout.decode().strip()
        require(head == value["source_commit"], "published commit differs from immutable tag")
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    args = parser.parse_args()
    manifest = read_manifest(args.manifest)
    root, evidence = Path(manifest["resource_root"]), Path(manifest["evidence_root"])
    require(root != evidence and root not in evidence.parents and evidence not in root.parents,
            "private resources and retained evidence must be separate roots")
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    evidence.mkdir(mode=0o700, parents=True, exist_ok=False)
    infrastructure = json.loads(Path(manifest["infrastructure_file"]).read_text())
    resources = Resources(manifest, infrastructure, evidence)
    receipt = dict(phase=PHASE, version="1.1.1", status="started", started_at=time.time(),
                   source_commit=manifest["source_commit"], source_tree=manifest["source_tree"],
                   fixture_tree=manifest["fixture_tree"], stage=manifest["stage"], image=manifest["image"],
                   platform=manifest["platform"], index_digest=manifest.get("index_digest"),
                   repair_parent=manifest.get("repair_parent"),
                   repair_source_change=manifest.get("repair_source_change"),
                   groups={}, execution_mode=None)
    path = evidence / "receipt.json"

    def save():
        path.write_text(resources.redact(json.dumps(receipt, indent=2, ensure_ascii=False)) + "\n")

    save()
    exit_code = 1
    try:
        resources.start()
        receipt["artifact"] = inspect_image(resources, manifest)
        host = resources.docker("info", "--format", "{{.Architecture}}").stdout.strip()
        native = {"arm64": "arm64", "aarch64": "arm64", "amd64": "amd64", "x86_64": "amd64"}.get(host)
        receipt["execution_mode"] = "native" if native == manifest["platform"].split("/")[1] else "emulated"
        save()
        modules = {}
        for group in manifest["groups"]:
            group_root = evidence / group
            group_root.mkdir()
            receipt["groups"][group] = dict(status="started", started_at=time.time())
            save()
            try:
                if manifest["stage"] == "local" and group in NATIVE_CASES:
                    receipt["groups"][group]["native_integration"] = run_native_integration(resources, group)
                    save()
                kind = ADAPTERS[group]
                if kind not in modules:
                    modules[kind] = load_adapter(kind)
                module = modules[kind]
                context = resources.context(group, group_root)
                context.update(stage=manifest["stage"], manifest=manifest)
                result = module.run_group(context)
                receipt["groups"][group]["result"] = result
                save()
                check_result(group, module, result)
                if group == "credentials":
                    compiler_module = load_adapter("compiler")
                    compiler_result = compiler_module.run_group(context)
                    receipt["groups"][group]["compiler_profile"] = compiler_result
                    save()
                    check_result(group, compiler_module, compiler_result)
                receipt["groups"][group].update(status="passed", result=result, finished_at=time.time())
                save()
            except BaseException as error:
                partial = getattr(error, "partial", None)
                if isinstance(partial, bytes):
                    partial_path = group_root / "interrupted-stream.log"
                    partial_path.write_text(resources.redact(partial.decode("utf-8", errors="replace")))
                    partial_path.chmod(0o600)
                    receipt["groups"][group]["partial_stream"] = str(partial_path)
                receipt["groups"][group].update(status="blocked" if isinstance(error, Blocked) else
                                               "interrupted" if isinstance(error, KeyboardInterrupt) else "failed",
                                               error=resources.redact(str(error)), finished_at=time.time())
                save()
                if isinstance(error, (KeyboardInterrupt, SystemExit)):
                    raise
        require(source_identity(SOURCE_PATHS) == manifest["source_tree"]
                and source_identity(FIXTURE_PATHS) == manifest["fixture_tree"], "source changed during acceptance")
        statuses = {item["status"] for item in receipt["groups"].values()}
        if "failed" in statuses:
            receipt["status"], exit_code = "failed", 1
        elif "blocked" in statuses:
            receipt["status"], exit_code = "blocked", 2
        else:
            receipt["status"], exit_code = "passed", 0
    except Blocked as error:
        receipt.update(status="blocked", error=resources.redact(str(error)))
        exit_code = 2
    except KeyboardInterrupt:
        receipt.update(status="interrupted", error="operator interruption")
        exit_code = 130
    except Exception as error:
        receipt.update(status="failed", error=resources.redact(str(error)))
    finally:
        try:
            resources.close()
        except BaseException as error:
            receipt["cleanup_error"] = resources.redact(str(error))
            receipt["status"] = "failed"
            exit_code = 1
        receipt.update(finished_at=time.time(), exit_code=exit_code)
        save()
        files = {}
        for artifact in sorted(evidence.rglob("*")):
            if artifact.is_file() and artifact.name != "hashes.json":
                files[str(artifact.relative_to(evidence))] = digest_bytes(artifact.read_bytes())
        (evidence / "hashes.json").write_text(json.dumps(files, indent=2) + "\n")
    print(json.dumps(dict(status=receipt["status"], exit_code=exit_code, receipt=str(path))))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
