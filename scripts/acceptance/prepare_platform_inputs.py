"""Explicitly approved serial platform preparation. Import performs no I/O."""
from __future__ import annotations

import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import sys
import time


ORDER = ("artifacts", "docker_build", "linux_prepare", "ios_prepare", "ios_target", "ios_build")
EFFECTS = ["run-frozen-native-sdk-grpc-artifact-preparation", "build-selected-local-linux-image",
           "derive-linux-build-receipt-config-and-exact-extraction-approval-from-successful-iid",
           "extract-owned-linux-artifact-without-start", "select-installed-ios-simulator-metadata",
           "explicit-install-selected-ios-target-under-pinned-toolchain",
           "build-ios-staticlib-and-build-for-testing-without-boot-or-test-execution",
           "retain-private-command-and-platform-artifact-receipts"]


class Refusal(RuntimeError):
    pass


def need(condition, code):
    if not condition:
        raise Refusal(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def absolute(value):
    path = Path(value)
    need(path.is_absolute() and ".." not in path.parts and path == path.resolve(), "resolved_absolute_path_required")
    return path


def file_hash(path):
    path = absolute(path)
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb") as stream:
        info = os.fstat(stream.fileno())
        need(stat.S_ISREG(info.st_mode) and info.st_size <= 17179869184, "bounded_regular_file_required")
        result = hashlib.sha256()
        while chunk := stream.read(1048576):
            result.update(chunk)
        return "sha256:" + result.hexdigest()


def reference(path):
    return {"path": str(path), "sha256": file_hash(path)}


def read_json(path, private=False):
    path = absolute(path)
    info = path.lstat()
    need(stat.S_ISREG(info.st_mode) and info.st_size <= 67108864, "bounded_regular_json_required")
    if private:
        need(info.st_uid == os.getuid() and info.st_mode & 0o077 == 0, "private_owned_json_required")
    return json.loads(path.read_bytes())


def frozen(ref):
    need(isinstance(ref, dict) and set(ref) == {"path", "sha256"}
         and re.fullmatch(r"sha256:[0-9a-f]{64}", ref["sha256"]), "invalid_frozen_reference")
    need(file_hash(ref["path"]) == ref["sha256"], "frozen_input_changed")
    return absolute(ref["path"])


def write(path, value):
    with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb") as stream:
        stream.write(canonical(value) + b"\n")
        stream.flush()
        os.fsync(stream.fileno())
    return reference(path)


def linux_commands(config):
    docker, output = config["docker_client"]["path"], Path(config["output"])
    return [[docker, "image", "inspect", config["image_id"]],
        [docker, "create", "--name", config["extraction_container_name"], "--cidfile", str(output / "extract.cid"),
         "--mount", "type=tmpfs,destination=/var/lib/uar", "--no-healthcheck", "--entrypoint", config["container_binary"], config["image_id"]],
        [docker, "inspect", "@owned-container-id@"],
        [docker, "cp", "-L", "@owned-container-id@:" + config["container_binary"], str(output / "uar")],
        [docker, "cp", "-L", "@owned-container-id@:" + config["container_python"], str(output / "image-python3")],
        [docker, "rm", "@owned-container-id@"],
        [docker, "ps", "--all", "--no-trunc", "--filter", "id=@owned-container-id@", "--format", "{{.ID}}"]]


class Controller:
    def __init__(self, path):
        self.config = c = read_json(path, private=True)
        need(c.get("schema_version") == "uar.platform-input-preparation.v1" and c.get("executable") is True,
             "explicit_platform_preparation_required")
        approval = read_json(frozen(c["approval"]), private=True)
        projection = {key: value for key, value in c.items() if key != "approval"}
        need(approval == {"operation": "prepare-uar-platform-inputs", "operator_approved": True,
                          "projection_sha256": sha(canonical(projection)), "effects": EFFECTS}, "exact_preparation_approval_differs")
        self.root = absolute(c["output_root"])
        need(not self.root.exists() and not self.root.is_symlink(), "exclusive_new_output_required")
        parent = self.root.parent
        need(parent.is_dir() and parent.stat().st_uid == os.getuid() and parent.stat().st_mode & 0o077 == 0,
             "existing_private_parent_required")
        limits = c["limits"]
        need(type(c["remaining_global_seconds"]) is int and 0 < limits["total_seconds"] <= c["remaining_global_seconds"]
             and type(limits["total_seconds"]) is int and type(limits["output_bytes"]) is int
             and 0 < limits["output_bytes"] <= 67108864, "explicit_remaining_global_and_output_bounds_required")
        need(set(c["commands"]) == set(ORDER), "exact_six_serial_commands_required")
        self.deadline = time.monotonic() + limits["total_seconds"]
        self.remaining_output = limits["output_bytes"]
        self.receipts, self.dynamic, self.owner = [], {}, None
        self.repo = Path(__file__).resolve().parents[2]
        self.inventory = read_json(frozen(c["source_inventory"]))
        need(self.inventory["root"] == str(self.repo) and self.inventory["source_identity"] == c["source_identity"],
             "source_inventory_identity_differs")
        self.sources = [{"path": row["path"], "sha256": row["sha256"]}
                        for row in self.inventory["files"] if row["kind"] == "regular"]
        need(self.sources == sorted(self.sources, key=lambda row: row["path"])
             and len({row["path"] for row in self.sources}) == len(self.sources)
             and sha(canonical(self.sources)) == c["source_identity"], "source_preimage_differs")
        required = {str(self.repo / value) for value in ("scripts/acceptance/prepare_platform_inputs.py",
            "scripts/acceptance/prepare_artifacts.py", "scripts/acceptance/linux_runtime.py", "scripts/acceptance/ios/runner.py")}
        need(required <= {row["path"] for row in self.sources}, "required_controller_helpers_not_regular_sources")
        self.source_identity()
        need({"python", "docker", "cargo", "git", "xcrun", "xcodebuild", "xcode_select", "rustup"} <= set(c["tools"]),
             "explicit_nested_tool_references_required")
        for ref in [*c["tools"].values(), *c["inputs"]]:
            frozen(ref)
        artifact_config = read_json(frozen(c["artifact_config"]), private=True)
        need(artifact_config["build_owner_lock"] == c["build_owner_lock"]
             and artifact_config["source_identity"] == c["source_identity"]
             and artifact_config["source_files"] == self.sources, "artifact_source_or_single_writer_lock_differs")
        need(Path(artifact_config["output_root"]) == self.root / "artifacts", "owned_artifact_output_differs")
        need(artifact_config["limits"]["total_seconds"] <= c["commands"]["artifacts"]["timeout_seconds"],
             "nested_artifact_allowance_exceeds_stage")
        need(artifact_config["limits"]["output_bytes"] <= c["limits"]["output_bytes"],
             "nested_artifact_output_exceeds_controller_bound")
        ids = {stage["id"] for stage in artifact_config["stages"]}
        need(ids == {"native", "rust_sdk", "grpc"}, "artifact_stage_scope_must_be_native_rust_sdk_grpc_only")
        for stage in artifact_config["stages"]:
            tool = stage["argv"][0]
            need(Path(tool).name not in {"sh", "bash", "zsh", "fish", "dash", "csh", "tcsh"}
                 and any(ref["path"] == tool and ref["sha256"] == stage["tool_sha256"] for ref in c["tools"].values()),
                 "nested_stage_must_use_explicit_frozen_nonshell_tool")
        self.artifact_config = artifact_config
        self.check_commands()
        self.root.mkdir(mode=0o700)
        (self.root / "logs").mkdir(mode=0o700)
        write(self.root / "ownership.json", {"source_identity": c["source_identity"], "config": reference(path),
            "projection_sha256": approval["projection_sha256"], "verification_gate": "CLOSED",
            "effects": EFFECTS})

    def source_identity(self):
        need(time.monotonic() < self.deadline, "preparation_allowance_exhausted")
        for row in self.sources:
            need(time.monotonic() < self.deadline, "preparation_allowance_exhausted")
            frozen(row)

    def check_commands(self):
        c, root = self.config, self.root
        python, docker = c["tools"]["python"]["path"], c["tools"]["docker"]["path"]
        need(c["tools"]["rustup"]["path"] == "/Users/gqadonis/.cargo/bin/rustup", "selected_rustup_path_differs")
        expected = {
            "artifacts": [python, str(self.repo / "scripts/acceptance/prepare_artifacts.py"), c["artifact_config"]["path"]],
            "linux_prepare": [python, str(self.repo / "scripts/acceptance/linux_runtime.py"), "--prepare", "--config",
                              str(root / "linux-config.json"), "--approval", str(root / "linux-approval.json")],
            "ios_prepare": [python, str(self.repo / "scripts/acceptance/ios/runner.py"), "prepare", "--selection",
                str(root / "ios-selection.json"), "--timeout-seconds", str(c["commands"]["ios_prepare"]["timeout_seconds"]), "--udid", c["ios_udid"]],
            "ios_target": [c["tools"]["rustup"]["path"], "target", "add", "@selected-target@", "--toolchain",
                           "nightly-2026-07-18-x86_64-apple-darwin"],
            "ios_build": [python, str(self.repo / "scripts/acceptance/ios/runner.py"), "build", "--selection",
                str(root / "ios-selection.json"), "--timeout-seconds", str(c["commands"]["ios_build"]["timeout_seconds"]),
                "--output", str(root / "ios-artifact"), "--source-inventory", c["source_inventory"]["path"]]}
        need(isinstance(c["ios_udid"], str) and re.fullmatch(r"[0-9A-Fa-f-]{36}", c["ios_udid"]), "explicit_selected_ios_udid_required")
        for name, spec in c["commands"].items():
            seconds = spec["timeout_seconds"]
            maximum = 14400 if name == "artifacts" else 120 if name == "ios_prepare" else 600 if name == "ios_target" else 1200 if name == "linux_prepare" else 7200
            need(type(seconds) is int and 0 < seconds <= maximum, "stage_allowance_outside_bound")
            need(spec["cwd"] == str(self.repo), "controller_command_cwd_differs")
            argv = spec["argv"]
            need(isinstance(argv, list) and argv and all(isinstance(value, str) and value and "\0" not in value for value in argv),
                 "exact_argv_required")
            if name in expected:
                need(argv == expected[name], "helper_command_differs_from_exact_cli")
            else:
                need(argv[:2] == [docker, "build"] and argv.count("--iidfile") == 1
                     and argv[argv.index("--iidfile") + 1] == str(root / "image.iid")
                     and argv.count("--platform") == 1
                     and argv[argv.index("--platform") + 1] == "linux/" + c["linux"]["architecture"]
                     and argv[-1] == str(self.repo) and "--push" not in argv, "exact_local_docker_build_with_owned_iid_required")
            environment = spec["environment"]
            need(isinstance(environment, dict) and environment and all(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key)
                 and isinstance(value, str) and "\0" not in value for key, value in environment.items()), "literal_selected_environment_required")
            need(not any("API_KEY" in key or "TOKEN" in key or "PASSWORD" in key or "SECRET" in key for key in environment),
                 "provider_or_secret_environment_forbidden")
            need("PATH" in environment and "@" not in environment["PATH"], "explicit_fixed_nested_tool_path_required")
        linux = c["linux"]
        need(set(linux) == {"architecture", "container_binary", "container_python", "extraction_container_name", "command_seconds", "output_bytes"}
             and linux["architecture"] in ("amd64", "arm64") and linux["container_python"] == "/usr/local/bin/python3"
             and re.fullmatch(r"uar-acceptance-linux-extract-[a-f0-9]{32}", linux["extraction_container_name"]), "exact_owned_linux_recipe_required")
        need(type(linux["command_seconds"]) is int and 0 < linux["command_seconds"] <= 120
             and type(linux["output_bytes"]) is int and 0 < linux["output_bytes"] <= 2097152,
             "linux_nested_bounds_invalid")

    def acquire_owner(self):
        path = absolute(self.config["build_owner_lock"])
        need(not path.is_relative_to(self.root) and path.parent.is_dir()
             and path.parent.stat().st_uid == os.getuid() and path.parent.stat().st_mode & 0o077 == 0,
             "existing_shared_private_lock_parent_required")
        descriptor = os.open(path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        self.owner = os.fdopen(descriptor, "r+b")
        info = os.fstat(descriptor)
        need(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid() and info.st_mode & 0o077 == 0, "shared_lock_not_private")
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def expand(self, value):
        # No shell interpolation, arbitrary expressions or ambient variable expansion.
        for name in ("@image-id@", "@selected-target@"):
            if name in value:
                need(name in self.dynamic, "dynamic_input_not_yet_observed")
                value = value.replace(name, self.dynamic[name])
        need(not re.search(r"@[A-Za-z][A-Za-z0-9_-]*@", value), "unsupported_dynamic_substitution")
        return value

    def stage(self, name):
        spec = self.config["commands"][name]
        argv = [self.expand(value) for value in spec["argv"]]
        env = {key: self.expand(value) for key, value in spec["environment"].items()}
        tool = self.config["tools"]["docker" if name == "docker_build" else "rustup" if name == "ios_target" else "python"]
        frozen(tool)
        need(argv[0] == tool["path"], "stage_tool_differs")
        stdout_path, stderr_path = self.root / "logs" / (name + ".stdout"), self.root / "logs" / (name + ".stderr")
        process, failure, code, cleanup = None, None, None, []
        start = time.time()
        stop = min(self.deadline, time.monotonic() + spec["timeout_seconds"])
        with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
            os.chmod(stdout_path, 0o600)
            os.chmod(stderr_path, 0o600)
            selector = selectors.DefaultSelector()
            try:
                need(time.monotonic() < stop, "total_allowance_exhausted_before_stage")
                process = subprocess.Popen(argv, cwd=spec["cwd"], env=env, stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True, close_fds=True)
                selector.register(process.stdout, selectors.EVENT_READ, out)
                selector.register(process.stderr, selectors.EVENT_READ, err)
                while selector.get_map():
                    need(time.monotonic() < stop, "stage_timed_out")
                    for key, _event in selector.select(timeout=0.1):
                        raw = os.read(key.fileobj.fileno(), min(65536, self.remaining_output + 1))
                        if not raw:
                            selector.unregister(key.fileobj)
                            continue
                        key.data.write(raw[:self.remaining_output])
                        self.remaining_output -= len(raw)
                        need(self.remaining_output >= 0, "aggregate_command_output_exhausted")
                code = process.wait(timeout=max(0.001, stop - time.monotonic()))
                need(code == 0, "preparation_stage_failed")
            except BaseException as error:
                failure = str(error) if isinstance(error, Refusal) else type(error).__name__
                raise
            finally:
                selector.close()
                if process is not None:
                    for sig in (signal.SIGTERM, signal.SIGKILL):
                        try:
                            os.killpg(process.pid, sig)
                            cleanup.append({"signal": sig.name, "result": "sent_to_owned_group"})
                        except ProcessLookupError:
                            cleanup.append({"signal": sig.name, "result": "group_absent"})
                    try:
                        process.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        cleanup.append({"result": "leader_join_unconfirmed"})
                    code = process.returncode
                    process.stdout.close()
                    process.stderr.close()
                for stream in (out, err):
                    stream.flush()
                    os.fsync(stream.fileno())
                receipt = {"stage": name, "command": argv, "operator_approved_command": argv,
                    "exit_code": code, "status": "FAILED" if failure else "FINISHED", "failure": failure,
                    "environment_names": sorted(env), "started_at_unix": start, "finished_at_unix": time.time(),
                    "stdout": reference(stdout_path), "stderr": reference(stderr_path), "owned_group_cleanup": cleanup}
                self.receipts.append(write(self.root / (name + "-command.json"), receipt))

    def linux_metadata(self):
        c = self.config
        iid = self.root / "image.iid"
        need(iid.is_file() and not iid.is_symlink() and iid.stat().st_size <= 128, "successful_build_iid_missing")
        image = iid.read_text().strip()
        need(re.fullmatch(r"sha256:[0-9a-f]{64}", image), "successful_build_iid_invalid")
        self.dynamic["@image-id@"] = image
        observed = read_json(self.root / "docker_build-command.json")
        need(observed["exit_code"] == 0 and observed["status"] == "FINISHED"
             and observed["command"] == observed["operator_approved_command"], "successful_docker_build_receipt_required")
        command = observed["command"]
        receipt = write(self.root / "linux-image-build.json", {"command": command,
            "operator_approved_command": command, "exit_code": 0, "source_identity": c["source_identity"],
            "image_id": image, "image_source_id": image, "iid": reference(iid),
            "command_receipt": self.receipts[-1]})
        config = {"schema_version": 1, "executable": True, **c["linux"], "source_identity": c["source_identity"],
            "image_id": image, "image_source_id": image, "docker_client": c["tools"]["docker"],
            "build_receipt": receipt, "state_parent": str(self.root), "output": str(self.root / "linux-artifact")}
        config_ref = write(self.root / "linux-config.json", config)
        write(self.root / "linux-approval.json", {"schema_version": 1, "operation": "prepare-linux-runtime-artifact",
            "operator_approved": True, "config_sha256": config_ref["sha256"], "source_identity": c["source_identity"],
            "commands": linux_commands(config)})

    def run(self):
        try:
            # Child owns the SAME flock during its native/SDK/gRPC build stages.
            # Holding it here would deadlock or refuse its independent helper.
            self.stage("artifacts")
            artifact_manifest = self.root / "artifacts/snapshot/preparation.json"
            artifact = read_json(artifact_manifest)
            need(artifact["source_identity"] == self.config["source_identity"], "prepared_native_source_differs")
            self.acquire_owner()
            self.source_identity()
            self.stage("docker_build")
            self.linux_metadata()
            self.stage("linux_prepare")
            linux_receipt = read_json(self.root / "linux-artifact/image-receipt.json")
            need(linux_receipt["source_identity"] == self.config["source_identity"]
                 and linux_receipt["image_id"] == self.dynamic["@image-id@"]
                 and linux_receipt["extraction_container_absent"] is True, "linux_preparation_receipt_differs")
            self.stage("ios_prepare")
            selection = read_json(self.root / "ios-selection.json")
            need(selection["udid"] == self.config["ios_udid"]
                 and selection["target"] in ("aarch64-apple-ios-sim", "x86_64-apple-ios"), "observed_ios_selection_differs")
            self.dynamic["@selected-target@"] = selection["target"]
            self.stage("ios_target")
            self.stage("ios_build")
            ios = read_json(self.root / "ios-artifact/artifact.json")
            need(ios["source_identity"] == self.config["source_identity"] and ios["selection"] == selection,
                 "ios_artifact_receipt_differs")
            self.source_identity()
            result = {"schema_version": "uar.platform-preparation-receipt.v1", "status": "PREPARED_NOT_RUNTIME_ACCEPTANCE",
                "source_identity": self.config["source_identity"], "source_inventory": self.config["source_inventory"],
                "native_sdk_grpc": reference(artifact_manifest), "linux": reference(self.root / "linux-artifact/image-receipt.json"),
                "ios": reference(self.root / "ios-artifact/artifact.json"), "selection": reference(self.root / "ios-selection.json"),
                "image_id": self.dynamic["@image-id@"], "selected_target": selection["target"],
                "command_receipts": self.receipts, "fixture_composed": False, "product_verification": False,
                "verification_gate": "CLOSED", "automatic_retry": False}
            return write(self.root / "platform-artifacts.json", result)
        except BaseException as error:
            write(self.root / "failed.json", {"status": "INCOMPLETE", "error_code": str(error)
                if isinstance(error, Refusal) else type(error).__name__, "command_receipts": self.receipts,
                "partial_state_retained": True, "automatic_retry": False, "verification_gate": "CLOSED"})
            raise
        finally:
            if self.owner is not None:
                self.owner.close()


def main():
    need(len(sys.argv) == 2, "expected_one_exact_private_configuration_path")
    result = Controller(sys.argv[1]).run()
    print(json.dumps({"status": "PREPARED_NOT_RUNTIME_ACCEPTANCE", "receipt": result, "gate": "CLOSED"}), flush=True)


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"status": "INCOMPLETE", "code": str(error) if isinstance(error, Refusal)
                          else type(error).__name__, "gate": "CLOSED"}), flush=True)
        sys.exit(2)
