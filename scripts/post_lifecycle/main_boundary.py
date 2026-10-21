#!/usr/bin/env python3
"""Single, explicitly approved post-lifecycle boundary entry point.

This file does not grant permission. The reviewed manifest must bind the final
implementation inventory, source, inputs and exact child argv before invocation.
No source inspection subprocesses, package installs, service changes or retries
are hidden here. The Rust integration target owns the declared main/peer children.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

SCENARIOS = ["LIAB-01", "LIAB-02", "ERROR-01", "ERROR-02", "COUNT-01",
             "HISTORY-01", "KB-01", "CAS-01", "BTHREE-01", "IDENTITY-01"]
BUILD = ["cargo", "build", "--locked", "--no-default-features", "--features",
         "server-full", "--bin", "universal-agent-runtime"]
TEST = ["cargo", "test", "--locked", "--no-default-features", "--features",
        "server-full", "--test", "post_lifecycle_main", "--", "--test-threads=1"]


def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def write_new(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def run_child(argv, env, directory, timeout):
    start = time.monotonic()
    with (directory / "stdout.log").open("xb") as out, (directory / "stderr.log").open("xb") as err:
        child = subprocess.Popen(argv, env=env, stdout=out, stderr=err)
        try:
            code = child.wait(timeout=timeout)
            outcome = "PASS" if code == 0 else "FAIL"
        except subprocess.TimeoutExpired:
            child.terminate()
            try:
                code = child.wait(timeout=30)
            except subprocess.TimeoutExpired:
                child.kill()
                code = child.wait()
            outcome = "BLOCKED"
    return {"argv": argv, "exitCode": code, "outcome": outcome,
            "elapsedSeconds": time.monotonic() - start,
            "descendantOwnership": "PENDING_IF_TIMEOUT" if outcome == "BLOCKED" else "see awaited fixture teardown",
            "stdoutSha256": digest(directory / "stdout.log"),
            "stderrSha256": digest(directory / "stderr.log")}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--evidence-root", required=True, type=Path)
    args = parser.parse_args()
    manifest_bytes = args.manifest.read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest.get("kind") == "uar.main-boundary/v1", "wrong manifest kind")
    require(manifest.get("implementationComplete") is True, "implementation inventory incomplete")
    require(manifest.get("verificationApprovalRef"), "exact-command verification approval absent")
    require(manifest.get("storeFixtureApprovalRef"), "isolated fixture permission absent")
    require(manifest.get("scenarioIds") == SCENARIOS, "scenario set/order differs from adopted packet")
    require(manifest.get("buildArgv") == BUILD and manifest.get("testArgv") == TEST,
            "build/test argv differs from adopted packet")
    require(manifest.get("externalSpendAllowed") is False, "external spend is excluded")
    root = Path(manifest["projectRoot"]).resolve()
    require(root == Path.cwd().resolve(), "wrong project cwd")
    candidate = (root / "target/uar-post-lifecycle-main/debug/universal-agent-runtime").resolve()
    require(Path(manifest["candidatePath"]).resolve() == candidate, "wrong main candidate path")
    require(manifest.get("sourceInventoryDigest") and manifest.get("contractDigest"), "identity missing")
    inputs = manifest.get("boundInputs", [])
    require(inputs, "no source/config/fixture bindings")
    for item in inputs:
        require(digest(Path(item["path"])) == item["sha256"], "bound input changed: " + item["path"])
    require(not args.evidence_root.exists(), "append-only evidence destination already exists")
    os.umask(0o077)
    args.evidence_root.mkdir(parents=True, mode=0o700)
    evidence = args.evidence_root.resolve()
    os.chmod(evidence, 0o700)
    write_new(evidence / "manifest.json", manifest)
    setup = {"outcome": "PASS", "scope": "local reviewed input bindings; actual process setup is retained in target events", "manifestSha256": hashlib.sha256(manifest_bytes).hexdigest(),
             "sourceInventoryDigest": manifest["sourceInventoryDigest"],
             "contractDigest": manifest["contractDigest"], "boundInputsChecked": len(inputs)}
    write_new(evidence / "setup.json", setup)
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = "target/uar-post-lifecycle-main"
    env["UAR_MAIN_CANDIDATE"] = str(candidate)
    env["UAR_BOUNDARY_MANIFEST"] = str(args.manifest.resolve())
    env["UAR_BOUNDARY_EVIDENCE"] = str(evidence)
    timeout = int(manifest.get("commandTimeoutSeconds", 1800))
    require(0 < timeout <= 14400, "invalid finite command timeout")
    build_dir = evidence / "raw/build"
    build_dir.mkdir(parents=True)
    build = run_child(BUILD, env, build_dir, timeout)
    write_new(evidence / "build.json", build)
    if build["outcome"] != "PASS":
        write_new(evidence / "body.json", {"outcome": build["outcome"], "testStarted": False})
        write_new(evidence / "teardown.json", {"outcome": "PASS", "fixtureChildrenStarted": False})
        return 1
    built_hash = digest(candidate)
    write_new(evidence / "source-build-boot.json", {
        "sourceInventoryDigest": manifest["sourceInventoryDigest"],
        "builtMainSha256": built_hash, "bootEvidence": "see process receipts; not inferred from build"})
    env["UAR_BOUNDARY_BUILT_SHA256"] = built_hash
    test_dir = evidence / "raw/test"
    test_dir.mkdir(parents=True)
    body = run_child(TEST, env, test_dir, timeout)
    write_new(evidence / "body.json", body)
    cases_file = evidence / "cases.json"
    teardown_file = evidence / "teardown.json"
    complete = cases_file.exists() and teardown_file.exists()
    if complete:
        cases = json.loads(cases_file.read_text())
        teardown = json.loads(teardown_file.read_text())
        complete = (set(cases) == set(SCENARIOS) and len(cases) == len(SCENARIOS) and teardown.get("outcome") == "PASS"
                    and all(cases[s].get("outcome") == "PASS" for s in SCENARIOS if s != "BTHREE-01")
                    and cases["BTHREE-01"].get("outcome") in ["PASS", "NOT_EXERCISED"])
    # No test summary, failed teardown or a zero-test Cargo exit can count as success.
    for item in inputs:
        complete = complete and digest(Path(item["path"])) == item["sha256"]
    write_new(evidence / "outcome.json", {"outcome": "PASS" if complete and body["outcome"] == "PASS" else "BLOCKED",
              "testExitCode": body["exitCode"], "completeScenarioAndTeardownSummary": complete})
    return 0 if complete and body["outcome"] == "PASS" else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError, json.JSONDecodeError) as error:
        # The exception contains paths/categories only, never manifest bodies or env values.
        print("BOUNDARY BLOCKED: " + str(error), file=sys.stderr)
        sys.exit(2)
