"""Owned Linux runtime on a macOS controller. Importing performs no I/O.

All Docker invocations belong to the separately approved execution gate.
"""
from __future__ import annotations

import datetime
import json
import os
from pathlib import Path
import re
import time
from urllib.parse import urlsplit, urlunsplit

from observers_startup import need, save


def route_endpoint(value, spec):
    """Rewrite only explicitly declared owned endpoints; preserve public hosts."""
    if not isinstance(value, str) or "://" not in value:
        return value
    parsed = urlsplit(value)
    if parsed.hostname not in ("127.0.0.1", "localhost", "::1", spec["host_alias"]):
        return value
    need(parsed.hostname != "::1", "Linux endpoint requires explicit IPv4 owned binding")
    route = {"scheme": parsed.scheme, "host": "127.0.0.1", "port": parsed.port}
    need(route in spec["owned_endpoint_routes"], "Linux endpoint is outside approved owned routing inventory")
    # Preserve escaped credentials, path, scope/search_path query and fragment.
    authority = parsed.netloc.rsplit("@", 1)
    userinfo = authority[0] + "@" if len(authority) == 2 else ""
    return urlunsplit(parsed._replace(netloc=userinfo + spec["host_alias"] + ":" + str(parsed.port)))


def route_tree(value, spec):
    if isinstance(value, dict):
        return {key: route_tree(item, spec) for key, item in value.items()}
    if isinstance(value, list):
        return [route_tree(item, spec) for item in value]
    return route_endpoint(value, spec)


def elf_matches(raw, architecture):
    return (len(raw) >= 20 and raw[:6] == b"\x7fELF\x02\x01"
            and int.from_bytes(raw[18:20], "little") == {"arm64": 183, "amd64": 62}[architecture])


class LinuxRuntime:
    def __init__(self, observer):
        self.observer = observer
        self.spec = observer.config["linux_runtime"]
        self.docker = str(observer.frozen_file(self.spec["docker_client"]["path"]))
        need(observer.frozen[self.docker] == self.spec["docker_client"]["sha256"], "Linux Docker client differs")
        observer.frozen_file(str(Path(__file__).resolve()))
        receipt = json.loads(observer.frozen_file(self.spec["image_receipt"]["path"]).read_bytes())
        need(observer.frozen[str(Path(self.spec["image_receipt"]["path"]).resolve())] == self.spec["image_receipt"]["sha256"],
             "Linux image preparation receipt differs")
        need(receipt["image_id"] == self.spec["image_id"] and receipt["image_source_id"] == self.spec["image_source_id"]
             and receipt["artifact_identity"] == observer.fixture["artifact_identity"]
             and receipt["source_identity"] == observer.fixture["source_identity"]
             and receipt["binary_path"] == observer.config["server_binary"]
             and receipt["container_binary"] == self.spec["container_binary"]
             and receipt["architecture"] == self.spec["architecture"]
             and receipt["docker_client"] == self.spec["docker_client"]
             and receipt["container_python"] == "/usr/local/bin/python3",
             "Linux image, extracted binary, source or client attestation differs")
        need(elf_matches(observer.frozen_file(receipt["binary_path"]).read_bytes(), self.spec["architecture"]),
             "Linux artifact ELF architecture differs")

    def command(self, *args):
        return self.observer.command([self.docker, *args], require_clean_stderr=True)

    def labels(self, launch):
        return {**self.observer.store_labels(), "uar.acceptance.runtime": "linux-container",
                "uar.acceptance.launch": launch["launch_nonce"]}

    def mount_contract(self):
        return [(self.spec["repo_root"], self.spec["repo_root"], False),
                (str(self.observer.root), str(self.observer.root), True),
                (str(self.observer.root / "image-state"), "/var/lib/uar", True),
                *[(value, value, False) for value in self.spec["readonly_resource_roots"]]]

    def launch(self, launch, environment):
        observer = self.observer
        nonce = launch["launch_nonce"]
        cidfile = observer.root / ("runtime-" + nonce + ".cid")
        envfile = observer.root / ("runtime-" + nonce + ".env")
        need(not cidfile.exists() and not envfile.exists(), "Linux launch paths already exist")
        # Never forward controller PATH, DYLD, Docker credentials, or ambient keys.
        allowed = {"UAR_STATIC_DIR", "UAR_TEAM_EXECUTION_MAX_ACTIVE", "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED",
                   *observer.config.get("server_environment_env", {})}
        selected = {key: value for key, value in environment.items() if key in allowed}
        selected.update(HOME=str(observer.root / "home"), TMPDIR=str(observer.root / "tmp"))
        need(all(re.fullmatch(r"[A-Z][A-Z0-9_]*", key) and "\n" not in value and "\r" not in value
                 for key, value in selected.items()), "Linux env-file cannot encode multiline environment")
        fd = os.open(envfile, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, "w") as stream:
            stream.write("".join(key + "=" + value + "\n" for key, value in sorted(selected.items())))
        policy_path = observer.root / ("runtime-" + nonce + "-environment.json")
        save(policy_path, sorted(selected))
        (observer.root / "image-state").mkdir(mode=0o700, exist_ok=True)
        argv = [self.docker, "run", "--pull=never", "--cidfile", str(cidfile), "--name", "uar-runtime-" + nonce,
                "--user", str(os.getuid()) + ":" + str(os.getgid()), "--no-healthcheck",
                "--env-file", str(envfile), "--workdir", str(observer.root), "--entrypoint", self.spec["container_python"]]
        for key, value in self.labels(launch).items():
            argv.extend(("--label", key + "=" + value))
        for source, destination, writable in self.mount_contract():
            need("," not in source and "," not in destination, "Linux bind path cannot contain mount syntax")
            argv.extend(("--mount", "type=bind,source=" + source + ",target=" + destination + ("" if writable else ",readonly")))
        for value in observer.config["linux_ports"]:
            argv.extend(("--publish", "127.0.0.1:" + str(value) + ":" + str(value)))
        container_args = [str(Path(__file__).resolve()), "--container-exec", self.spec["container_binary"],
                          "--config", launch["config_path"], "--environment-policy", str(policy_path)]
        argv.extend((self.spec["image_id"], *container_args))
        launch.update(runtime_platform="linux-container", image_id=self.spec["image_id"],
                      image_source_id=self.spec["image_source_id"], architecture=self.spec["architecture"],
                      cidfile=str(cidfile), envfile=str(envfile), container_argv=argv, container_args=container_args,
                      container_user=str(os.getuid()) + ":" + str(os.getgid()), environment_policy=str(policy_path),
                      environment_policy_keys=sorted(selected),
                      launch_started_at=time.time())
        save(observer.root / ("launch-" + str(os.getpid()) + ".json"), launch)
        # The foreground frozen CLI replaces the wrapper and retains its PID.
        os.execve(self.docker, argv, os.environ.copy())

    def cid(self, launch):
        path = Path(launch["cidfile"])
        need(path == self.observer.root / ("runtime-" + launch["launch_nonce"] + ".cid")
             and path.is_file() and not path.is_symlink() and path.stat().st_uid == os.getuid()
             and path.stat().st_size <= 128, "Linux immutable ID file is unavailable or unowned")
        value = path.read_text().strip()
        need(re.fullmatch(r"[0-9a-f]{64}", value), "Linux immutable container ID is malformed")
        return value

    def inspect(self, launch, running=True):
        cid = self.cid(launch)
        rows = json.loads(self.command("inspect", cid))
        need(isinstance(rows, list) and len(rows) == 1, "Linux inspect identity is ambiguous")
        row = rows[0]
        need(row["Id"] == cid and row["Image"] == self.spec["image_id"]
             and row["Config"]["Image"] == self.spec["image_id"]
             and all(row["Config"]["Labels"].get(key) == value for key, value in self.labels(launch).items()),
             "Linux runtime is outside immutable image and launch ownership")
        need(row["Config"]["Entrypoint"] == [self.spec["container_python"]]
             and row["Config"]["Cmd"] == launch["container_args"]
             and row["Config"]["User"] == launch["container_user"]
             and row["Config"]["WorkingDir"] == str(self.observer.root), "Linux command/config path differs")
        policy_path = Path(launch["environment_policy"])
        need(policy_path == self.observer.root / ("runtime-" + launch["launch_nonce"] + "-environment.json")
             and self.observer.private_receipt(policy_path) == launch["environment_policy_keys"],
             "Linux scrubbed image environment policy changed")
        mounts = {(item["Source"], item["Destination"], item["RW"]) for item in row["Mounts"]}
        need(mounts == set(self.mount_contract()),
             "Linux runtime mounts differ from exact approved ownership")
        expected_ports = {str(port) + "/tcp": [{"HostIp": "127.0.0.1", "HostPort": str(port)}]
                          for port in self.observer.config["linux_ports"]}
        need(row["HostConfig"]["PortBindings"] == expected_ports, "Linux published listener bindings differ")
        # Container start must follow this exact foreground CLI launch receipt.
        started = row["State"]["StartedAt"]
        started_seconds = datetime.datetime.fromisoformat(started.replace("Z", "+00:00")).timestamp()
        need(launch["launch_started_at"] <= started_seconds <= time.time(), "Linux container start does not correlate to launch")
        need(not running or row["State"]["Running"] is True, "owned Linux runtime is not running")
        return row

    def stop(self, launch):
        cid = self.cid(launch)
        selectors = ["--filter", "id=" + cid]
        for key, value in self.labels(launch).items():
            selectors.extend(("--filter", "label=" + key + "=" + value))
        # Exact cid absence is idempotent; missing/invalid receipt remains failure.
        raw = self.command("ps", "--all", "--no-trunc", "--filter", "id=" + cid, "--format", "{{.ID}}")
        need(raw.decode().splitlines() in ([], [cid]), "Linux immutable ID lookup differs")
        if raw.strip():
            self.inspect(launch, running=False)
            selected = self.command("ps", "--all", "--no-trunc", *selectors, "--format", "{{.ID}}")
            need(selected.decode().splitlines() == [cid], "Linux cleanup selector ownership differs")
            # Stop joins the product first; remove targets only its inspected ID.
            self.command("stop", "--time", "10", cid)
            self.command("wait", cid)
            self.command("rm", cid)
        remaining = self.command("ps", "--all", "--no-trunc", "--filter", "id=" + cid, "--format", "{{.ID}}")
        need(not remaining.strip(), "owned Linux runtime remains; refusing store teardown")
        return {"container_id": cid, "joined": True, "absent": True}


def prepare_commands(config, output):
    """Exact future command packet. Container-ID substitution is closed."""
    docker = config["docker_client"]["path"]
    return [
        [docker, "image", "inspect", config["image_id"]],
        [docker, "create", "--name", config["extraction_container_name"], "--cidfile", str(output / "extract.cid"),
         "--mount", "type=tmpfs,destination=/var/lib/uar", "--no-healthcheck",
         "--entrypoint", config["container_binary"], config["image_id"]],
        [docker, "inspect", "@owned-container-id@"],
        [docker, "cp", "-L", "@owned-container-id@:" + config["container_binary"], str(output / "uar")],
        [docker, "cp", "-L", "@owned-container-id@:" + config["container_python"], str(output / "image-python3")],
        [docker, "rm", "@owned-container-id@"],
        [docker, "ps", "--all", "--no-trunc", "--filter", "id=@owned-container-id@", "--format", "{{.ID}}"],
    ]


def prepare(config_path, approval_path):
    """Future approved extraction only. Never build, install, pull or start."""
    import hashlib
    import subprocess
    import tempfile

    def digest(raw):
        return "sha256:" + hashlib.sha256(raw).hexdigest()

    def regular(path):
        need(path.is_absolute() and path.is_file() and not path.is_symlink(), "Linux preparation input must be a regular absolute file")
        return path.read_bytes()

    raw_config = regular(config_path)
    need(config_path.stat().st_uid == os.getuid() and config_path.stat().st_mode & 0o077 == 0
         and approval_path.is_file() and not approval_path.is_symlink()
         and approval_path.stat().st_uid == os.getuid() and approval_path.stat().st_mode & 0o077 == 0,
         "Linux preparation config and approval must be private owned files")
    config = json.loads(raw_config)
    need(config.get("schema_version") == 1 and config.get("executable") is True,
         "Linux preparation config is inert or unsupported")
    need(re.fullmatch(r"sha256:[0-9a-f]{64}", config.get("source_identity", ""))
         and re.fullmatch(r"sha256:[0-9a-f]{64}", config.get("image_id", ""))
         and config.get("architecture") in ("amd64", "arm64")
         and config.get("container_python") == "/usr/local/bin/python3",
         "Linux preparation needs exact source/image/architecture/Python")
    need(Path(config["container_binary"]).is_absolute()
         and re.fullmatch(r"uar-acceptance-linux-extract-[a-f0-9]{32}", config["extraction_container_name"]),
         "Linux preparation extraction identity is not explicit")
    client = config["docker_client"]
    need(digest(regular(Path(client["path"]))) == client["sha256"], "Linux preparation Docker client changed")
    build_ref = config["build_receipt"]
    raw_build = regular(Path(build_ref["path"]))
    need(digest(raw_build) == build_ref["sha256"], "Linux image build receipt changed")
    build = json.loads(raw_build)
    need(build.get("exit_code") == 0 and build.get("command") == build.get("operator_approved_command")
         and isinstance(build.get("command"), list) and build["command"]
         and all(build.get(key) == config[key] for key in ("source_identity", "image_id", "image_source_id")),
         "Linux image requires actual approved successful build receipt")
    parent = Path(config["state_parent"])
    need(parent.is_absolute() and parent.is_dir() and not parent.is_symlink()
         and parent.stat().st_uid == os.getuid() and parent.stat().st_mode & 0o077 == 0,
         "Linux extraction requires an existing private owned parent")
    output = Path(config["output"])
    need(output.parent == parent and output.name == "linux-artifact" and not output.exists(),
         "Linux extraction output must be new exact linux-artifact directory")
    commands = prepare_commands(config, output)
    approval = json.loads(regular(approval_path))
    need(approval == {"schema_version": 1, "operation": "prepare-linux-runtime-artifact", "operator_approved": True,
                      "config_sha256": digest(raw_config), "source_identity": config["source_identity"], "commands": commands},
         "Linux preparation exact command approval differs")
    seconds = config["command_seconds"]
    output_limit = config["output_bytes"]
    need(type(seconds) is int and 0 < seconds <= 120 and type(output_limit) is int and 0 < output_limit <= 2 * 1024 * 1024,
         "Linux extraction command/output bounds must be explicit")
    output.mkdir(mode=0o700)
    save(output / "ownership.json", {"source_identity": config["source_identity"], "config_sha256": digest(raw_config),
                                     "extraction_container_name": config["extraction_container_name"]})
    receipts = []

    def command(index, cid=None):
        argv = [item.replace("@owned-container-id@", cid) if cid is not None else item for item in commands[index]]
        need(not any("@owned-container-id@" in item for item in argv), "Linux extraction immutable ID is unresolved")
        incomplete = None
        with tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                                       start_new_session=True)
            try:
                deadline = time.monotonic() + seconds
                while process.poll() is None:
                    need(stdout.tell() <= output_limit and stderr.tell() <= output_limit,
                         "Linux preparation command output exceeded bound")
                    need(time.monotonic() < deadline, "Linux preparation command timed out")
                    try:
                        process.wait(timeout=0.05)
                    except subprocess.TimeoutExpired:
                        pass
                process.wait()
            except Exception as error:
                incomplete = type(error).__name__
            finally:
                if process.poll() is None:
                    import signal
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=seconds)
            stdout.seek(0)
            stderr.seek(0)
            raw, errors = stdout.read(output_limit + 1), stderr.read(output_limit + 1)
        receipt = {"command": argv, "exit_code": process.returncode, "stdout_sha256": digest(raw), "stderr_sha256": digest(errors),
                   "status": "INCOMPLETE" if incomplete is not None else "FINISHED", "incomplete_error_type": incomplete}
        receipts.append(receipt)
        save(output / "commands.json", receipts)
        need(incomplete is None and len(raw) <= output_limit and len(errors) <= output_limit
             and process.returncode == 0 and not errors.strip(),
             "Linux preparation command failed; private digests retained")
        return raw

    cid = None
    removed = False
    try:
        images = json.loads(command(0))
        need(isinstance(images, list) and len(images) == 1, "Linux image inspect returned ambiguous identity")
        image = images[0]
        need(image["Id"] == config["image_id"] and image["Os"] == "linux"
             and image["Architecture"] == config["architecture"]
             and (config["image_source_id"] == image["Id"] or config["image_source_id"] in (image.get("RepoDigests") or [])),
             "Linux image source identity or architecture differs")
        need(set(image["Config"].get("Volumes") or {}) <= {"/var/lib/uar"},
             "Linux extraction image declares an unowned anonymous volume")
        # Image source is bound both by the approved build receipt and immutable
        # content ID; a local image needs no fabricated registry RepoDigest.
        command(1)
        cid_path = output / "extract.cid"
        need(cid_path.is_file() and not cid_path.is_symlink() and cid_path.stat().st_size <= 128,
             "Linux extraction did not retain an immutable ID")
        cid = cid_path.read_text().strip()
        need(re.fullmatch(r"[a-f0-9]{64}", cid), "Linux extraction immutable ID is malformed")
        rows = json.loads(command(2, cid))
        need(len(rows) == 1 and rows[0]["Id"] == cid and rows[0]["Image"] == config["image_id"]
             and rows[0]["Name"] == "/" + config["extraction_container_name"]
             and rows[0]["State"]["Status"] == "created" and rows[0]["State"]["Running"] is False,
             "Linux extraction container is not the exact unstarted owned image")
        command(3, cid)
        command(4, cid)
        binary = output / "uar"
        python = output / "image-python3"
        need(elf_matches(regular(binary), config["architecture"]) and elf_matches(regular(python), config["architecture"]),
             "Linux image binary or Python ELF architecture differs")
        binary.chmod(0o500)
        python.chmod(0o400)
        command(5, cid)
        removed = True
        need(not command(6, cid).strip(), "Linux extraction container remains")
        artifact_identity = digest(binary.read_bytes())
        receipt = {"schema_version": 1, "status": "LINUX_IMAGE_PREPARED_NOT_RUNTIME_ACCEPTANCE",
                   **{key: config[key] for key in ("source_identity", "image_id", "image_source_id", "architecture",
                                                  "container_binary", "container_python", "docker_client")},
                   "binary_path": str(binary), "artifact_identity": artifact_identity,
                   "image_python_sha256": digest(python.read_bytes()), "image_build_receipt": build_ref,
                   "extraction_container_id": cid, "extraction_container_absent": True,
                   "config_sha256": digest(raw_config), "commands": receipts}
        save(output / "image-receipt.json", receipt)
        save(output / "build-receipt.json", {**build, "artifact_path": str(binary), "artifact_identity": artifact_identity,
                                             "image_receipt_path": str(output / "image-receipt.json")})
        return {"status": receipt["status"], "artifact_path": str(binary), "image_receipt": str(output / "image-receipt.json"),
                "build_receipt": str(output / "build-receipt.json"), "runtime_verified": False}
    except Exception:
        # If create was interrupted, recover only its exact cidfile. Never look
        # up or remove by a shared name, image, ancestor or broad label.
        if cid is None:
            cid_path = output / "extract.cid"
            if cid_path.is_file() and not cid_path.is_symlink() and cid_path.stat().st_size <= 128:
                candidate = cid_path.read_text().strip()
                if re.fullmatch(r"[a-f0-9]{64}", candidate):
                    cid = candidate
        if cid is not None and not removed:
            rows = json.loads(command(2, cid))
            need(len(rows) == 1 and rows[0]["Id"] == cid and rows[0]["Image"] == config["image_id"]
                 and rows[0]["Name"] == "/" + config["extraction_container_name"]
                 and rows[0]["State"]["Running"] is False, "Linux extraction cleanup ownership is unconfirmed")
            command(5, cid)
            need(not command(6, cid).strip(), "Linux extraction cleanup did not confirm absence")
        raise


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--prepare", action="store_true")
    mode.add_argument("--container-exec", type=Path)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--approval", type=Path)
    parser.add_argument("--environment-policy", type=Path)
    args = parser.parse_args()
    if args.container_exec is not None:
        import sys
        need(sys.platform == "linux" and args.approval is None and args.environment_policy is not None,
             "Linux container exec requires explicit owned environment policy")
        policy = json.loads(args.environment_policy.read_bytes())
        need(isinstance(policy, list) and all(isinstance(key, str) for key in policy), "Linux environment policy differs")
        environment = {key: value for key, value in os.environ.items()
                       if key in policy or not key.startswith(("UAR_", "LLM_")) and key not in
                       ("PORT", "CONFIG_FILE", "JWT_REQUIRED", "RATE_LIMIT_ENABLED", "DATABASE_URL", "PGOPTIONS", "MCP_CONFIG_PATH", "MCP_CONFIG_DIR")}
        need(args.container_exec.is_absolute() and args.container_exec.read_bytes()[:4] == b"\x7fELF",
             "Linux container cannot execute a host Mach-O artifact")
        os.execve(str(args.container_exec), [str(args.container_exec), "--config", str(args.config)], environment)
    need(args.approval is not None and args.environment_policy is None, "Linux preparation requires exact approval")
    print(json.dumps(prepare(args.config, args.approval), sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"status": "BLOCKED", "error_type": type(error).__name__,
                          "detail": "Linux preparation failed; no runtime acceptance"}))
        raise SystemExit(2)
