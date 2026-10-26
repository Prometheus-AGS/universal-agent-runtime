"""Read-only live identity receipt for the explicitly selected macOS Liter job.

Import performs no I/O. Invocation belongs to a later named approved packet.
No service start/stop, environment inspection, HTTP or inference is implemented.
"""
from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import plistlib
import re
import selectors
import subprocess
import sys
import time
import tomllib
from urllib.parse import urlsplit


LABEL = "ai.prometheus.liter-llm-api"
AUTH_NAMES = ["LITER_LLM_MASTER_KEY", "KIMI_CODING_KEY", "MINIMAX_KEY"]


class Refusal(RuntimeError):
    pass


def need(condition, code):
    if not condition:
        raise Refusal(code)


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def reference(ref):
    need(isinstance(ref, dict) and set(ref) == {"path", "sha256"}, "source_reference_invalid")
    path = Path(ref["path"])
    need(path.is_absolute() and path.is_file() and not path.is_symlink()
         and path.stat().st_size <= 1024 * 1024 * 1024, "frozen_source_unavailable")
    before = path.stat()
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    after = path.stat()
    need((before.st_ino, before.st_size, before.st_mtime_ns) == (after.st_ino, after.st_size, after.st_mtime_ns)
         and "sha256:" + digest == ref["sha256"], "frozen_source_changed")
    return path, after


def origin(url):
    parsed = urlsplit(url)
    need(parsed.scheme in ("http", "https") and parsed.hostname and not parsed.username
         and not parsed.password and not parsed.query and not parsed.fragment, "configured_origin_not_sanitized")
    return parsed.scheme + "://" + parsed.netloc


def bounded_command(argv, limits, deadline):
    remaining = min(limits["command_seconds"], deadline - time.monotonic())
    need(remaining > 0, "service_observation_wall_exhausted")
    process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        env={"LC_ALL": "C", "LANG": "C", "TZ": "UTC", "PATH": "/usr/bin:/bin:/usr/sbin:/sbin"},
        close_fds=True)
    stop = time.monotonic() + remaining
    output = bytearray()
    observed = 0
    selector = selectors.DefaultSelector()
    try:
        selector.register(process.stdout, selectors.EVENT_READ, "stdout")
        selector.register(process.stderr, selectors.EVENT_READ, "stderr")
        while selector.get_map():
            need(time.monotonic() < stop, "service_identity_command_timeout")
            events = selector.select(timeout=min(0.1, max(0, stop - time.monotonic())))
            for key, _mask in events:
                raw = os.read(key.fileobj.fileno(), min(16384, limits["output_bytes"] - observed + 1))
                if not raw:
                    selector.unregister(key.fileobj)
                    continue
                observed += len(raw)
                need(observed <= limits["output_bytes"], "service_identity_output_exceeded")
                if key.data == "stdout":
                    output.extend(raw)
        need(process.wait(timeout=max(0.001, stop - time.monotonic())) == 0, "service_identity_command_refused")
        return output.decode("utf-8", errors="strict")
    finally:
        selector.close()
        if process.poll() is None:
            process.kill()  # only the owned observation command, never the service
            process.wait()
        process.stdout.close()
        process.stderr.close()


def registered_job(raw, uid, launchagent):
    lines = raw.splitlines()
    need(lines and lines[0].strip() == f"gui/{uid}/{LABEL} = {{", "launchctl_print_format_unqualified")
    # Installed launchctl documents this human format as NOT an API. Accept only
    # its frozen top-level shape; nested environments/arguments are never emitted.
    def field(name):
        values = [line[len(name) + 4:] for line in lines if line.startswith("\t" + name + " = ")]
        need(len(values) == 1, "registered_job_field_missing_or_ambiguous")
        return values[0]
    need(field("path") == launchagent and field("state") == "running", "registered_job_identity_differs")
    pid = field("pid")
    need(re.fullmatch(r"[1-9][0-9]*", pid), "registered_job_pid_invalid")
    return int(pid)


def process_identity(raw, uid, expected_argv):
    rows = raw.strip().splitlines()
    need(len(rows) == 1, "selected_process_missing_or_ambiguous")
    pieces = rows[0].strip().split(None, 6)
    need(len(pieces) == 7 and pieces[0] == str(uid)
         and pieces[6] == " ".join(expected_argv), "selected_process_owner_or_command_differs")
    started = datetime.strptime(" ".join(pieces[1:6]), "%a %b %d %H:%M:%S %Y").replace(tzinfo=timezone.utc)
    return started.timestamp()


def listeners(raw, pid, uid, approved):
    current_pid, current_uid = None, None
    names, file_record = [], {}
    def finish():
        if file_record:
            need(current_pid == str(pid) and current_uid == str(uid), "listener_owner_differs")
            need(file_record.get("t") in ("IPv4", "IPv6"), "listener_type_unqualified")
            name = file_record.get("n")
            need(name in approved, "listener_binding_not_explicitly_approved")
            names.append(name)
    for line in raw.splitlines():
        need(line and line[0] in "pufnt", "listener_field_format_unqualified")
        key, value = line[0], line[1:]
        if key == "p":
            finish()
            file_record = {}
            need(current_pid is None, "listener_process_ambiguous")
            current_pid = value
        elif key == "u":
            need(current_uid is None, "listener_uid_ambiguous")
            current_uid = value
        elif key == "f":
            finish()
            file_record = {key: value}
        else:
            need(key not in file_record, "listener_field_ambiguous")
            file_record[key] = value
    finish()
    need(names, "selected_process_has_no_approved_listener")
    return sorted(set(names))


def observe(config, config_digest):
    need(config.get("schema_version") == 1 and config.get("executable") is True, "service_observer_not_adopted")
    uid = config["uid"]
    need(type(uid) is int and uid == os.getuid(), "service_observer_owner_differs")
    limits = config["limits"]
    for name, ceiling in (("command_seconds", 30), ("wall_seconds", 120), ("output_bytes", 1048576)):
        need(type(limits.get(name)) is int and 0 < limits[name] <= ceiling, "service_observer_limit_invalid")
    need(config["credential_env_names"] == AUTH_NAMES, "credential_reference_names_differs")
    approved_names = config["approved_listener_names"]
    need(isinstance(approved_names, list) and approved_names
         and set(approved_names) <= {"127.0.0.1:4000", "[::1]:4000", "*:4000"}, "listener_approval_missing")
    tools, sources = {}, {}
    for name, expected in (("launchctl", "/bin/launchctl"), ("ps", "/bin/ps"), ("lsof", "/usr/sbin/lsof")):
        path, _stat = reference(config["tools"][name])
        need(str(path) == expected and os.access(path, os.X_OK), "observation_tool_not_frozen_expected_binary")
        tools[name] = str(path)
    for name in ("launchagent", "launch_script", "service_config", "executable"):
        sources[name] = reference(config["sources"][name])
    args = config["expected_argv"]
    need(args == [str(sources["executable"][0]), "api", "--config", str(sources["service_config"][0])]
         and all(isinstance(value, str) and not re.search(r"\s", value) for value in args), "service_expected_argv_unqualified")
    need(sources["launchagent"][1].st_size <= 1048576 and sources["launch_script"][1].st_size <= 1048576
         and sources["service_config"][1].st_size <= 1048576, "configured_service_source_oversized")
    plist = plistlib.loads(sources["launchagent"][0].read_bytes())
    need(plist.get("Label") == LABEL and plist.get("ProgramArguments") == [str(sources["launch_script"][0])],
         "selected_launch_definition_differs")
    data = tomllib.loads(sources["service_config"][0].read_text())
    models = [{"alias": item["name"], "provider_model": item["provider_model"],
               "upstream_origin": origin(item["base_url"])} for item in data.get("models", [])]
    need(models == config["expected_models"], "configured_nonsecret_model_aliases_differ")
    patterns = [item["pattern"] for item in data.get("aliases", [])]
    need(patterns == config["expected_alias_patterns"], "configured_alias_patterns_differ")
    general = data.get("general", {})
    need(general.get("max_retries") == config["expected_max_retries"]
         and general.get("enable_cost_tracking") is config["expected_cost_tracking"], "gateway_accounting_settings_differ")
    deadline = time.monotonic() + limits["wall_seconds"]
    target = f"gui/{uid}/{LABEL}"
    launch = bounded_command([tools["launchctl"], "print", target], limits, deadline)
    pid = registered_job(launch, uid, str(sources["launchagent"][0]))
    del launch
    ps_argv = [tools["ps"], "-p", str(pid), "-o", "uid=,lstart=,command=", "-ww"]
    started = process_identity(bounded_command(ps_argv, limits, deadline), uid, args)
    names = listeners(bounded_command([tools["lsof"], "-n", "-P", "-a", "-p", str(pid), "-u", str(uid),
                        "-iTCP:4000", "-sTCP:LISTEN", "-F", "pufnt"], limits, deadline), pid, uid, approved_names)
    need(process_identity(bounded_command(ps_argv, limits, deadline), uid, args) == started,
         "selected_process_changed_during_observation")
    for name in sources:
        _path, stat = reference(config["sources"][name])
        need(stat.st_mtime <= started + 1, "selected_source_modified_after_process_start")
    return {"schema_version": "uar.configured-service-identity.v1", "status": "observed", "service_id": LABEL,
            "config_sha256": config_digest, "sourceconfig_sha256": config["sources"]["service_config"]["sha256"],
            "observed_at_unix": time.time(), "uid": uid, "service_target": target, "pid": pid,
            "process_started_unix": started, "expected_command_matched": True, "process_argv": args,
            "registered_launchagent_path": str(sources["launchagent"][0]),
            "sources": {name: {**config["sources"][name], "mtime_unix": sources[name][1].st_mtime} for name in sources},
            "listeners": names, "loopback_exclusive": all(name != "*:4000" for name in names),
            "configured_models": models, "configured_alias_patterns": patterns,
            "credential_env_names": AUTH_NAMES, "gateway_max_retries": config["expected_max_retries"],
            "gateway_cost_tracking": config["expected_cost_tracking"],
            "limits": ["file hashes/mtime and process command do not prove loaded in-memory config or current-source build",
                       "registered-job parser is OS-specific human format; unrecognized output is refused",
                       "gateway forwarding receipts count gateway requests, not physical provider attempts",
                       "embedding model availability, downstream identity, prices and charge are unverified"]}


def main():
    need(len(sys.argv) == 2, "expected_one_frozen_service_observer_config")
    path = Path(sys.argv[1])
    need(path.is_absolute() and path.is_file() and not path.is_symlink()
         and path.stat().st_mode & 0o077 == 0 and path.stat().st_size <= 1048576, "private_observer_config_unavailable")
    raw = path.read_bytes()
    config = json.loads(raw)
    receipt = observe(config, sha(raw))
    output = Path(config["receipt_path"])
    need(output.is_absolute() and output.parent.is_dir() and not output.parent.is_symlink()
         and output.parent.stat().st_mode & 0o077 == 0, "private_service_receipt_parent_unavailable")
    encoded = json.dumps(receipt, sort_keys=True).encode() + b"\n"
    need(len(encoded) <= config["limits"]["output_bytes"], "service_receipt_exceeds_allowance")
    with os.fdopen(os.open(output, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as stream:
        stream.write(encoded)
        stream.flush()
        os.fsync(stream.fileno())
    sys.stdout.buffer.write(encoded)
    sys.stdout.buffer.flush()


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"error": "configured_service_identity_unconfirmed", "code": str(error) if isinstance(error, Refusal)
                          else type(error).__name__}), flush=True)
        sys.exit(2)
