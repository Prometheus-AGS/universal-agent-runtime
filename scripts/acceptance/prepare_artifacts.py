"""Future-approved serialized artifact preparation; importing performs no I/O.

Exact configuration and separate approval bind effects. No service/model launch,
installation, discovery, default command, fallback output or inferred closure.
"""
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

CLOSURES = ("native", "python", "node", "python_sdk", "typescript_sdk", "rust_sdk",
            "grpc", "playwright", "browser", "video_encoder", "classifier_wasm", "local_embedding_assets")


class Refusal(RuntimeError):
    pass


def need(condition, reason):
    if not condition:
        raise Refusal(reason)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def digest(path):
    with path.open("rb") as stream:
        return "sha256:" + hashlib.file_digest(stream, "sha256").hexdigest()


def absolute(value):
    path = Path(value)
    need(path.is_absolute() and ".." not in path.parts, "absolute normalized input required")
    return path


def private_json(path):
    path = absolute(path)
    info = path.lstat()
    need(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid() and info.st_mode & 0o077 == 0
         and info.st_size <= 64 * 1024 * 1024, "private bounded JSON input required")
    return json.loads(path.read_bytes())


def write(path, value):
    with path.open("xb") as stream:
        os.chmod(path, 0o600)
        stream.write(canonical(value) + b"\n")
        stream.flush()
        os.fsync(stream.fileno())


def reference(path):
    return {"path": str(path), "sha256": digest(path)}


class Preparation:
    def __init__(self, config_path):
        self.config = private_json(config_path)
        c = self.config
        need(c.get("schema_version") == 1 and c.get("executable") is True,
             "artifact preparation not adopted")
        approval = private_json(c["approval"]["path"])
        need(digest(absolute(c["approval"]["path"])) == c["approval"]["sha256"], "approval bytes differ")
        projection = {key: value for key, value in c.items() if key != "approval"}
        need(approval == {"operation": "prepare-uar-acceptance-artifacts", "operator_approved": True,
                          "projection_sha256": sha(canonical(projection))}, "exact preparation approval missing")
        self.root = absolute(c["output_root"])
        need(self.root.parent.is_dir() and not self.root.parent.is_symlink()
             and self.root.parent.stat().st_uid == os.getuid()
             and self.root.parent.stat().st_mode & 0o077 == 0, "owned private output parent required")
        need(not self.root.exists() and not self.root.is_symlink(), "refusing existing output scope")
        limits = c["limits"]
        for name, maximum in (("total_seconds", 14400), ("stage_seconds", 7200),
                              ("output_bytes", 67108864), ("file_count", 1000000),
                              ("file_bytes", 17179869184), ("tree_bytes", 137438953472)):
            need(type(limits[name]) is int and 0 < limits[name] <= maximum, "invalid finite preparation bound")
        need(isinstance(c.get("os_baseline"), dict) and c["os_baseline"].get("excluded") is True
             and bool(c["os_baseline"].get("description")), "explicit excluded OS baseline required")
        self.limits = limits
        self.deadline = time.monotonic() + limits["total_seconds"]
        self.outputs, self.receipts = {}, []
        lock_path = absolute(c["build_owner_lock"])
        need(lock_path.parent.is_dir() and not lock_path.parent.is_symlink()
             and lock_path.parent.stat().st_uid == os.getuid()
             and lock_path.parent.stat().st_mode & 0o077 == 0
             and not lock_path.is_relative_to(self.root), "shared private build-owner lock required")
        descriptor = os.open(lock_path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        self.owner_lock = os.fdopen(descriptor, "r+b")
        info = os.fstat(descriptor)
        need(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid() and info.st_mode & 0o077 == 0,
             "build-owner lock is not private")
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        self.root.mkdir(mode=0o700)
        write(self.root / "ownership.json", {"operation": "prepare-uar-acceptance-artifacts",
              "source_identity": c["source_identity"], "projection_sha256": approval["projection_sha256"]})
        (self.root / "logs").mkdir(mode=0o700)
        self.snapshot = self.root / "snapshot"
        self.snapshot.mkdir(mode=0o700)

    def bounded(self):
        need(time.monotonic() < self.deadline, "preparation total deadline exhausted")

    def source_identity(self):
        rows = self.config["source_files"]
        need(all(set(row) == {"path", "sha256"} for row in rows), "source entries must match composer path/hash preimage")
        need(rows and rows == sorted(rows, key=lambda row: row["path"])
             and len({row["path"] for row in rows}) == len(rows), "source denominator must be sorted and unique")
        for row in rows:
            self.bounded()
            path = absolute(row["path"])
            need(path.is_file() and digest(path) == row["sha256"], "frozen source input differs")
        identity = sha(canonical(rows))
        need(identity == self.config["source_identity"], "source identity differs from composer preimage")
        return identity

    def stage(self, spec):
        self.bounded()
        name = spec["id"]
        need(re.fullmatch(r"[a-z][a-z0-9_-]{0,63}", name), "invalid stage ID")
        argv = spec["argv"]
        need(isinstance(argv, list) and argv and all(isinstance(arg, str) and arg and "\0" not in arg for arg in argv),
             "exact argv required; shell syntax is not interpreted")
        tool = absolute(argv[0])
        need(tool.is_file() and os.access(tool, os.X_OK) and digest(tool) == spec["tool_sha256"], "frozen stage tool differs")
        cwd = absolute(spec["cwd"])
        need(cwd.is_dir() and not cwd.is_symlink(), "stage working directory unavailable")
        environment = {}
        for key, variable in spec["environment_env"].items():
            need(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", key)
                 and re.fullmatch(r"[A-Z][A-Z0-9_]*", variable), "environment must use named references")
            value = os.environ.get(variable)
            need(isinstance(value, str) and "\0" not in value, "approved environment reference unavailable")
            environment[key] = value
        # No inherited shell/profile, provider credentials, PATH or ambient flags.
        stdout_path, stderr_path = (self.root / "logs" / (name + suffix) for suffix in (".stdout", ".stderr"))
        written, stdout = 0, bytearray()
        process = None
        code = None
        failure = None
        started = time.time()
        with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
            os.chmod(stdout_path, 0o600)
            os.chmod(stderr_path, 0o600)
            selector = selectors.DefaultSelector()
            try:
                process = subprocess.Popen(argv, cwd=cwd, env=environment, stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True, close_fds=True)
                selector.register(process.stdout, selectors.EVENT_READ, out)
                selector.register(process.stderr, selectors.EVENT_READ, err)
                stop = min(self.deadline, time.monotonic() + self.limits["stage_seconds"])
                while selector.get_map():
                    need(time.monotonic() < stop, "artifact stage deadline exhausted")
                    for key, _event in selector.select(timeout=0.1):
                        block = os.read(key.fileobj.fileno(), min(65536, self.limits["output_bytes"] - written + 1))
                        if not block:
                            selector.unregister(key.fileobj)
                            continue
                        remaining = self.limits["output_bytes"] - written
                        key.data.write(block[:remaining])
                        if key.data is out:
                            stdout.extend(block[:remaining])
                        written += len(block)
                        need(written <= self.limits["output_bytes"], "artifact stage output bound exhausted")
                code = process.wait(timeout=max(0.001, stop - time.monotonic()))
                need(code == 0, "artifact stage exited unsuccessfully")
            except BaseException as error:
                failure = {"type": type(error).__name__, "detail_digest": sha(str(error).encode())}
                raise
            finally:
                selector.close()
                if process is not None:
                    # Terminate remaining descendants too, even if the leader exited.
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    process.wait()
                    process.stdout.close()
                    process.stderr.close()
                out.flush(); err.flush()
                os.fsync(out.fileno()); os.fsync(err.fileno())
                receipt = {"stage": name, "command": argv, "operator_approved_command": argv,
                           "cwd": str(cwd), "environment_names": sorted(environment), "exit_code": code,
                           "started_at_unix": started, "finished_at_unix": time.time(), "failure": failure,
                           "stdout": reference(stdout_path), "stderr": reference(stderr_path)}
                write(self.root / (name + "-build.json"), receipt)
                self.receipts.append(receipt)
        os.chmod(stdout_path, 0o400)
        os.chmod(stderr_path, 0o400)
        for output in spec["outputs"]:
            identifier = output["id"]
            need(identifier not in self.outputs, "duplicate output binding")
            if "cargo_target" in output:
                candidates = []
                for line in stdout.decode("utf-8").splitlines():
                    row = json.loads(line)
                    if (row.get("reason") == "compiler-artifact" and row.get("target", {}).get("name") == output["cargo_target"]
                            and output["cargo_kind"] in row.get("target", {}).get("kind", []) and row.get("executable")):
                        candidates.append(row["executable"])
                need(len(set(candidates)) == 1, "exact Cargo emitted executable unavailable")
                path = absolute(candidates[0])
            else:
                path = absolute(output["path"])
            need(path.exists() and not path.is_symlink(), "expected emitted output unavailable")
            self.outputs[identifier] = {"path": path, "stage": name}

    def inventory_tree(self, root):
        rows, files, size = [], 0, 0
        stack = [(root, Path("."))]
        while stack:
            self.bounded()
            path, relative = stack.pop()
            info = path.lstat()
            files += 1
            need(files <= self.limits["file_count"], "tree entry allowance exhausted")
            if stat.S_ISLNK(info.st_mode):
                rows.append({"relative": str(relative), "kind": "symlink", "target": os.readlink(path)})
            elif stat.S_ISDIR(info.st_mode):
                rows.append({"relative": str(relative), "kind": "directory"})
                stack.extend((item, relative / item.name) for item in sorted(path.iterdir(), reverse=True))
            else:
                need(stat.S_ISREG(info.st_mode) and info.st_size <= self.limits["file_bytes"], "unsupported or oversized closure input")
                size += info.st_size
                need(size <= self.limits["tree_bytes"], "tree byte allowance exhausted")
                rows.append({"relative": str(relative), "kind": "file", "sha256": digest(path),
                             "bytes": info.st_size, "executable": bool(info.st_mode & 0o111)})
        return sorted(rows, key=lambda row: row["relative"])

    def capture(self):
        trees = self.config["trees"]
        roots, inventories, mapped = {}, {}, {}
        for tree in trees:
            name = tree["id"]
            need(re.fullmatch(r"[a-z][a-z0-9_-]{0,63}", name) and name not in roots, "invalid tree binding")
            if "output" in tree:
                produced = self.outputs[tree["output"]]["path"]
                root = absolute(tree["path"]) if "path" in tree else produced
                need(produced == root or produced.is_relative_to(root), "built tree does not contain its actual output")
            else:
                root = absolute(tree["path"])
            need(root.exists() and not root.is_symlink(), "approved closure root missing or indirect")
            need(not root.is_relative_to(self.root) and not self.root.is_relative_to(root), "snapshot overlaps selected tree")
            need(all(not root.is_relative_to(other) and not other.is_relative_to(root) for other in roots.values()),
                 "overlapping tree selections are ambiguous")
            rows = self.inventory_tree(root)
            if "output" not in tree:
                need(sha(canonical(rows)) == tree["tree_sha256"], "approved existing resource tree differs")
            roots[name], inventories[name] = root, rows
            mapped[name] = self.snapshot / name / "payload"
        total_files = sum(len(rows) for rows in inventories.values())
        total_bytes = sum(row.get("bytes", 0) for rows in inventories.values() for row in rows)
        need(total_files <= self.limits["file_count"] and total_bytes <= self.limits["tree_bytes"], "snapshot aggregate bound exhausted")
        def destination(path):
            for name, root in roots.items():
                if path == root or path.is_relative_to(root):
                    return mapped[name] / path.relative_to(root)
            raise Refusal("symlink target is outside explicitly approved resource trees")
        files_by_tree = {}
        links = []
        for name, root in roots.items():
            mapped[name].parent.mkdir(mode=0o700)
            captured = []
            for row in inventories[name]:
                self.bounded()
                original, target = root / row["relative"], mapped[name] / row["relative"]
                if row["kind"] == "directory":
                    target.mkdir(mode=0o700, parents=True, exist_ok=True)
                elif row["kind"] == "symlink":
                    # Resolve the full chain; no dangling or external target allowed.
                    resolved = original.resolve(strict=True)
                    links.append((target, destination(resolved), str(original), row["target"]))
                else:
                    target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                    with original.open("rb") as source, target.open("xb") as output:
                        copied = 0
                        while block := source.read(1024 * 1024):
                            self.bounded()
                            copied += len(block)
                            need(copied <= row["bytes"], "resource changed while copied")
                            output.write(block)
                        output.flush(); os.fsync(output.fileno())
                    need(copied == row["bytes"] and digest(target) == row["sha256"], "copied closure bytes differ")
                    os.chmod(target, 0o555 if row["executable"] else 0o444)
                    captured.append(reference(target))
            need(self.inventory_tree(root) == inventories[name], "resource tree changed during snapshot")
            files_by_tree[name] = captured
        for target, resolved, original, link in links:
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            target.symlink_to(os.path.relpath(resolved, target.parent))
        # Include every cross-tree link target transitively in the same closure.
        dependencies = {name: set() for name in roots}
        for target, resolved, original, link in links:
            owner = next(name for name in roots if target == mapped[name] or target.is_relative_to(mapped[name]))
            dependency = next(name for name in roots if resolved == mapped[name] or resolved.is_relative_to(mapped[name]))
            dependencies[owner].add(dependency)
        closures = {}
        need(set(self.config["closures"]) == set(CLOSURES), "all twelve explicit closure groups required")
        for name in CLOSURES:
            members = self.config["closures"][name]
            need(members and all(member in roots for member in members), "closure tree selection missing")
            expanded = set(members)
            pending = list(members)
            while pending:
                for dependency in dependencies[pending.pop()]:
                    if dependency not in expanded:
                        expanded.add(dependency)
                        pending.append(dependency)
            rows = sorted({row["path"]: row for member in expanded for row in files_by_tree[member]}.values(), key=lambda row: row["path"])
            need(rows, "closure has no captured regular bytes")
            manifest = self.snapshot / (name + "-closure.json")
            write(manifest, {"kind": name, "files": rows})
            closures[name] = {"complete": True, "files": [row["path"] for row in rows], "manifest": reference(manifest)}
        bindings = {}
        for key, entry in self.config["installed"].items():
            path = mapped[entry["tree"]] / entry["relative"]
            need(".." not in Path(entry["relative"]).parts and not Path(entry["relative"]).is_absolute()
                 and path.is_file() and path.resolve().is_relative_to(self.snapshot), "installed binding escapes snapshot or lacks bytes")
            bindings[key] = str(path.resolve())
        server = Path(bindings["server_binary"])
        need(str(server) in closures["native"]["files"], "server is outside native closure")
        native = next(row for row in self.receipts if row["stage"] == self.config["native_stage"])
        server_entry = self.config["installed"]["server_binary"]
        server_tree = next(tree for tree in trees if tree["id"] == server_entry["tree"])
        need("output" in server_tree and self.outputs[server_tree["output"]]["stage"] == native["stage"]
             and (roots[server_tree["id"]] / server_entry["relative"]).resolve()
                 == self.outputs[server_tree["output"]]["path"].resolve(),
             "native receipt must name the actual native-stage emitted server")
        artifact = reference(server)
        receipt = self.snapshot / "native-build-receipt.json"
        write(receipt, {**native, "source_identity": self.config["source_identity"], "artifact_identity": artifact["sha256"],
                        "artifact_path": artifact["path"]})
        manifest = {"schema_version": 1, "status": "prepared", "source_identity": self.config["source_identity"],
                    "source_files": self.config["source_files"], "installed_artifact": artifact,
                    "build_receipt": reference(receipt), "closures": closures, "installed": bindings,
                    "os_baseline": self.config["os_baseline"], "symlinks": [
                        {"path": str(target), "resolved_path": str(resolved), "source_path": original, "source_link": link}
                        for target, resolved, original, link in links],
                    "limit": "closure covers explicitly approved resource trees; OS baseline excluded, no all-machine claim"}
        write(self.snapshot / "preparation.json", manifest)
        # Seal only after all capture/receipts succeed. Partial scopes remain private.
        for current, dirs, files in os.walk(self.snapshot, followlinks=False):
            for name in files:
                path = Path(current) / name
                if not path.is_symlink():
                    os.chmod(path, 0o555 if path.stat().st_mode & 0o111 else 0o444)
        for current, dirs, files in os.walk(self.snapshot, topdown=False, followlinks=False):
            os.chmod(current, 0o500)
        return reference(self.snapshot / "preparation.json")

    def run(self):
        try:
            self.source_identity()
            stages = self.config["stages"]
            need(stages and len(stages) <= 32 and len({stage["id"] for stage in stages}) == len(stages), "finite unique serial stages required")
            for stage in stages:
                self.stage(stage)
                self.source_identity()
            result = self.capture()
            self.source_identity()
            write(self.root / "complete.json", {"status": "prepared", "manifest": result})
            return {"status": "prepared", "manifest": result, "private_logs": str(self.root / "logs")}
        except BaseException as error:
            write(self.root / "failed.json", {"status": "failed", "error_type": type(error).__name__,
                  "detail_digest": sha(str(error).encode()), "completed_stages": [row["stage"] for row in self.receipts],
                  "private_state_retained": True})
            raise
        finally:
            self.owner_lock.close()


def main():
    need(len(sys.argv) == 2, "expected exact artifact preparation configuration path")
    print(json.dumps(Preparation(sys.argv[1]).run(), sort_keys=True), flush=True)


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"status": "failed", "error_type": type(error).__name__,
                          "detail": "artifact preparation stopped; inspect owned private receipts"}), flush=True)
        sys.exit(2)
