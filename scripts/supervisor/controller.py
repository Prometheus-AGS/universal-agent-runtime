#!/usr/bin/env python3
"""Privileged, independently supervised exclusion controller (stdlib only).

This service is NEVER launched by an agent or by the runtime it fences. Native
mode observes whole-host boots and never reboots the machine. Kubernetes mode
runs on the node outside governed Pods, uses the node's CRI and cgroup v2, and
never interprets API disappearance as exclusion. Failures produce fixed labels.
"""
import argparse
import hashlib
import http.server
import json
import os
import pathlib
import platform
import plistlib
import sqlite3
import ssl
import subprocess
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid


class Refused(Exception):
    pass


def require(condition):
    if not condition:
        raise Refused()


def command(argv):
    # Fixed executables and discrete argv only. Never send subprocess stderr,
    # request contents, certificates, or Kubernetes tokens to response/logs.
    result = subprocess.run(argv, check=True, capture_output=True, timeout=30)
    require(len(result.stdout) <= 4 * 1024 * 1024)
    return result.stdout.decode("utf-8").strip()


def native_identity():
    kind = platform.system().lower()
    if kind == "linux":
        machine = pathlib.Path("/etc/machine-id").read_text().strip()
        boot = pathlib.Path("/proc/sys/kernel/random/boot_id").read_text().strip()
    elif kind == "darwin":
        rows = plistlib.loads(subprocess.run(
            ["/usr/sbin/ioreg", "-a", "-r", "-c", "IOPlatformExpertDevice"],
            check=True, capture_output=True, timeout=10).stdout)
        machine = rows[0]["IOPlatformUUID"]
        boot = command(["/usr/sbin/sysctl", "-n", "kern.bootsessionuuid"])
    elif kind == "windows":
        # OS-provided boot GUID avoids wall-clock changes or PID reuse.
        import ctypes
        class BootEnvironment(ctypes.Structure):
            _fields_ = [("identifier", ctypes.c_ubyte * 16),
                        ("firmware_type", ctypes.c_uint32),
                        ("boot_flags", ctypes.c_uint64)]
        info = BootEnvironment()
        length = ctypes.c_ulong()
        result = ctypes.windll.ntdll.NtQuerySystemInformation(
            90, ctypes.byref(info), ctypes.sizeof(info), ctypes.byref(length))
        require(result == 0 and length.value >= 20)
        boot = bytes(info.identifier).hex()
        machine = command([str(pathlib.Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"), "-NoProfile", "-NonInteractive", "-Command",
            "(Get-CimInstance Win32_ComputerSystemProduct).UUID"])
    else:
        raise Refused()
    require(machine and boot)
    return kind, machine, boot


def root_identity(pid, expected_uid):
    require(isinstance(pid, int) and pid > 1)
    if platform.system() == "Linux":
        stat = pathlib.Path(f"/proc/{pid}/stat").read_text()
        started = stat.rsplit(")", 1)[1].split()[19]
        require(pathlib.Path(f"/proc/{pid}").stat().st_uid == expected_uid)
        return f"{pid}:{started}"
    if platform.system() == "Darwin":
        require(int(command(["/bin/ps", "-p", str(pid), "-o", "uid="])) == expected_uid)
        started = command(["/bin/ps", "-p", str(pid), "-o", "lstart="])
        require(started)
        return f"{pid}:{started}"
    # Native service identity is fixed by controller config, never client input.
    output = command([str(pathlib.Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"), "-NoProfile", "-NonInteractive", "-Command",
        f"(Get-Process -Id {pid} -ErrorAction Stop).StartTime.ToUniversalTime().Ticks"])
    require(output.isdecimal())
    return f"{pid}:{output}"


def closed_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result)
        result[key] = value
    return result


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


class Controller:
    def __init__(self, config, qualification_candidate=False):
        self.config = config
        self.lock = threading.Lock()
        self.db = sqlite3.connect(config["journal"], check_same_thread=False)
        self.db.execute("PRAGMA synchronous=FULL")
        self.db.execute("CREATE TABLE IF NOT EXISTS domains (epoch TEXT PRIMARY KEY, certificate TEXT NOT NULL, registration TEXT NOT NULL, private_data TEXT NOT NULL)")
        self.db.execute("CREATE TABLE IF NOT EXISTS excluded_domains (registration TEXT PRIMARY KEY, observation TEXT NOT NULL)")
        self.db.execute("CREATE TABLE IF NOT EXISTS exclusions (challenge TEXT PRIMARY KEY, request TEXT NOT NULL, receipt TEXT NOT NULL)")
        self.db.execute("CREATE TABLE IF NOT EXISTS node_inhibitions (identity TEXT PRIMARY KEY, evidence TEXT NOT NULL)")
        self.db.commit()
        self.context = ssl.create_default_context(cafile=config["kubernetes_ca"]) if config.get("kubernetes_ca") else None
        # Qualification is an exact local release/platform receipt digest selected
        # by the operator AFTER the phase gate. No auto-enable/default receipt.
        path = pathlib.Path(config["qualification_file"])
        require(hashlib.sha256(path.read_bytes()).hexdigest() == config["qualification_sha256"])
        qualification = json.loads(path.read_text())
        require(qualification["controller_id"] == config["controller_id"])
        require(qualification["qualification_id"] == config["qualification_id"])
        require(qualification["mechanism"] == config["mechanism"])
        if qualification_candidate:
            # Operator-only bootstrap for an isolated real-platform gate.
            # This never creates or promotes a production qualification record.
            require(qualification["production_qualified"] is False)
            require(qualification.get("isolated_candidate") is True)
            require(config["qualification_id"].startswith("dot17-candidate-"))
            require(config["deployments"] and all(name.startswith("dot17-")
                for name in config["deployments"]))
        else:
            require(qualification["production_qualified"] is True)
        require(qualification["controller_sha256"] == hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest())

    def kubernetes(self, path, method="GET", body=None):
        base = self.config["kubernetes_api"]
        require(base.startswith("https://"))
        token = pathlib.Path(self.config["kubernetes_token"]).read_text().strip()
        request = urllib.request.Request(base.rstrip("/") + path,
            data=None if body is None else encoded(body).encode(), method=method,
            headers={"Authorization": "Bearer " + token, "Content-Type": "application/json"})
        class NoRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, req, fp, code, msg, headers, newurl):
                raise Refused()
        opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect(),
            urllib.request.HTTPSHandler(context=self.context))
        with opener.open(request, timeout=15) as response:
            data = response.read(4 * 1024 * 1024 + 1)
            require(len(data) <= 4 * 1024 * 1024)
            return json.loads(data)

    def cri(self, *args):
        binary = self.config["crictl"]
        require(pathlib.Path(binary).is_absolute())
        return command([binary, "--runtime-endpoint", self.config["cri_endpoint"], *args])

    def kubelet_state(self):
        binary = self.config["systemctl"]
        require(pathlib.Path(binary).is_absolute() and self.config["kubelet_unit"] == "kubelet.service")
        output = command([binary, "show", "--property=ActiveState,UnitFileState,MainPID,ControlGroup,KillMode",
            self.config["kubelet_unit"]])
        return dict(row.split("=", 1) for row in output.splitlines())

    def kubelet_domain(self):
        state = self.kubelet_state()
        require(state["ActiveState"] == "active" and state["KillMode"] == "control-group")
        pid = int(state["MainPID"])
        require(pid > 1 and state["ControlGroup"].startswith("/"))
        group = (pathlib.Path("/sys/fs/cgroup") / state["ControlGroup"].lstrip("/")).resolve(strict=True)
        require(str(group).startswith("/sys/fs/cgroup/") and group != pathlib.Path("/sys/fs/cgroup"))
        require(pathlib.Path(self.process_cgroup(pid)) == group)
        require(not pathlib.Path(self.process_cgroup(os.getpid())).is_relative_to(group))
        require((group / "cgroup.kill").exists() and (group / "cgroup.freeze").exists())
        stat = group.stat()
        return {"unit": self.config["kubelet_unit"], "cgroup": str(group), "device": stat.st_dev, "inode": stat.st_ino}

    def inhibit_kubelet(self, recorded):
        # Qualified node profile: kubelet is the sole workload restart authority.
        # Its exact retained cgroup is excluded and its unit stays runtime-masked.
        # This intentionally suspends node scheduling until operator recovery.
        require(recorded["unit"] == self.config["kubelet_unit"] == "kubelet.service")
        boot = pathlib.Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        identity = encoded({"boot": boot, "node": self.config["node_uid"], "kubelet": recorded})
        established = self.db.execute("SELECT evidence FROM node_inhibitions WHERE identity=?", (identity,)).fetchone()
        if established:
            state = self.kubelet_state()
            require(state["ActiveState"] in ("inactive", "failed") and state["MainPID"] == "0"
                and state["UnitFileState"] in ("masked", "masked-runtime"))
            return
        descriptor = os.open(recorded["cgroup"], os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            stat = os.fstat(descriptor)
            require(stat.st_dev == recorded["device"] and stat.st_ino == recorded["inode"])
            def write(name, value):
                fd = os.open(name, os.O_WRONLY | os.O_NOFOLLOW, dir_fd=descriptor)
                try:
                    require(os.write(fd, value) == len(value))
                finally:
                    os.close(fd)
            def events():
                fd = os.open("cgroup.events", os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptor)
                with os.fdopen(fd) as stream:
                    return dict(line.split() for line in stream)
            write("cgroup.freeze", b"1")
            deadline = time.monotonic() + 10
            while events().get("frozen") != "1":
                require(time.monotonic() < deadline)
                time.sleep(0.05)
            command([self.config["systemctl"], "mask", "--runtime", recorded["unit"]])
            require(self.kubelet_state()["UnitFileState"] in ("masked", "masked-runtime"))
            write("cgroup.kill", b"1")
            deadline = time.monotonic() + 10
            while events().get("populated") != "0":
                require(time.monotonic() < deadline)
                time.sleep(0.05)
            command([self.config["systemctl"], "stop", recorded["unit"]])
            state = self.kubelet_state()
            require(state["ActiveState"] in ("inactive", "failed") and state["MainPID"] == "0"
                and state["UnitFileState"] in ("masked", "masked-runtime"))
            self.db.execute("INSERT INTO node_inhibitions VALUES (?,?)", (identity,
                encoded({"cgroup_empty": True, "runtime_masked": True, "state": state})))
            self.db.commit()
        finally:
            os.close(descriptor)

    def binding(self, fingerprint, action, deployment):
        bound = self.config["clients"].get(fingerprint)
        require(bound is not None and action in bound["actions"])
        require(deployment in bound["deployments"])
        if action == "register":
            require(deployment == bound["registration_deployment"])
        return self.config["deployments"][deployment]

    def register(self, request, fingerprint):
        deployment = request["deployment_id"]
        settings = self.binding(fingerprint, "register", deployment)
        epoch = str(uuid.UUID(request["runtime_epoch"]))
        incarnation = str(uuid.UUID(request["team_incarnation"]))
        require(request["qualification_id"] == self.config["qualification_id"])
        old = self.db.execute("SELECT certificate,registration FROM domains WHERE epoch=?", (epoch,)).fetchone()
        if old:
            registration = json.loads(old[1])
            require(old[0] == fingerprint and registration["team_incarnation"] == incarnation
                and registration["deployment_id"] == deployment)
            return registration
        if self.config["mechanism"] == "native-boot-exclusion-v1":
            kind, machine, boot = native_identity()
            services = {"darwin": "com.prometheus.universal-agent-runtime", "linux": "uar.service",
                        "windows": "PrometheusUniversalAgentRuntime"}
            require(settings["service_name"] == services[kind])
            domain = {"kind": "native_boot", "platform": kind, "machine_id": machine,
                "boot_id": boot, "service_name": services[kind],
                "root_process_identity": root_identity(request["process_id"], settings.get("runtime_uid"))}
            private = {}
        elif self.config["mechanism"] == "kubernetes-cri-containment-v1":
            domain, private = self.register_pod(settings)
        else:
            raise Refused()
        registration = {"runtime_epoch": epoch, "team_incarnation": incarnation,
            "controller_id": self.config["controller_id"], "qualification_id": self.config["qualification_id"],
            "deployment_id": deployment, "domain": domain}
        self.db.execute("INSERT INTO domains VALUES (?,?,?,?)", (epoch, fingerprint, encoded(registration), encoded(private)))
        self.db.commit()
        return registration

    def register_pod(self, settings):
        require(platform.system() == "Linux")
        kubelet = self.kubelet_domain()
        namespace = urllib.parse.quote(settings["namespace"], safe="")
        name = urllib.parse.quote(settings["pod_name"], safe="")
        path = f"/api/v1/namespaces/{namespace}/pods/{name}"
        pod = self.kubernetes(path)
        require(pod["metadata"]["uid"] == settings["pod_uid"])
        require(not pod["metadata"].get("deletionTimestamp"))
        annotations = pod["metadata"].get("annotations", {})
        require("kubernetes.io/config.mirror" not in annotations
            and annotations.get("kubernetes.io/config.source", "api") == "api")
        spec = pod["spec"]
        require(not spec.get("hostPID") and not spec.get("hostIPC"))
        require(not any("hostPath" in volume for volume in spec.get("volumes", [])))
        for container in spec.get("containers", []) + spec.get("initContainers", []) + spec.get("ephemeralContainers", []):
            security = container.get("securityContext", {})
            require(security.get("allowPrivilegeEscalation") is False and not security.get("privileged"))
            capabilities = security.get("capabilities", {})
            require("ALL" in capabilities.get("drop", []) and not capabilities.get("add"))
        node = self.kubernetes("/api/v1/nodes/" + urllib.parse.quote(spec["nodeName"], safe=""))
        require(node["metadata"]["uid"] == self.config["node_uid"])
        boot = pathlib.Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        require(node["status"]["nodeInfo"]["bootID"] == boot)
        sandboxes = json.loads(self.cri("pods", "-o", "json"))["items"]
        candidates = [row for row in sandboxes if row.get("metadata", {}).get("uid") == settings["pod_uid"]]
        require(len(candidates) == 1)
        sandbox = candidates[0]["id"]
        rows = json.loads(self.cri("ps", "-a", "-o", "json"))["containers"]
        containers = [row["id"] for row in rows if row["podSandboxId"] == sandbox]
        require(containers)
        groups = []
        for container in containers:
            info = json.loads(self.cri("inspect", container))
            # Containerd's verbose PID is part of this mechanism's platform
            # qualification. An incompatible runtime is refused, not guessed.
            pid = int(info["info"]["pid"])
            require(pid > 1)
            groups.append(self.process_cgroup(pid))
        # Each workload container must be directly inside the same Pod cgroup.
        parent = pathlib.Path(os.path.commonpath(groups)).parent if len(groups) == 1 else pathlib.Path(os.path.commonpath(groups))
        pod_uid = settings["pod_uid"]
        require(pod_uid in parent.name or pod_uid.replace("-", "_") in parent.name)
        parent = parent.resolve(strict=True)
        require(str(parent).startswith("/sys/fs/cgroup/") and parent != pathlib.Path("/sys/fs/cgroup"))
        require((parent / "cgroup.kill").exists() and (parent / "cgroup.freeze").exists())
        require(not pathlib.Path(self.process_cgroup(os.getpid())).is_relative_to(parent))
        stat = parent.stat()
        domain = {"kind": "kubernetes", "cluster_id": self.config["cluster_id"], "namespace": settings["namespace"],
            "pod_uid": pod_uid, "node_uid": node["metadata"]["uid"], "node_boot_id": boot,
            "sandbox_id": sandbox, "container_ids": sorted(containers)}
        return domain, {"pod_path": path, "cgroup": str(parent), "device": stat.st_dev,
            "inode": stat.st_ino, "kubelet": kubelet}

    @staticmethod
    def process_cgroup(pid):
        rows = pathlib.Path(f"/proc/{pid}/cgroup").read_text().splitlines()
        rows = [row[3:] for row in rows if row.startswith("0::")]
        require(len(rows) == 1)
        path = pathlib.Path("/sys/fs/cgroup") / rows[0].lstrip("/")
        return str(path.resolve(strict=True))

    def exclude(self, request, fingerprint):
        registration = request["registration"]
        self.binding(fingerprint, "exclude", registration["deployment_id"])
        require(request["generation"] == 1)
        challenge_id = str(uuid.UUID(request["id"]))
        request_data = encoded(request)
        prior = self.db.execute("SELECT request,receipt FROM exclusions WHERE challenge=?", (challenge_id,)).fetchone()
        if prior:
            require(prior[0] == request_data)
            return json.loads(prior[1])
        row = self.db.execute("SELECT registration,private_data FROM domains WHERE epoch=?",
            (registration["runtime_epoch"],)).fetchone()
        require(row is not None and encoded(registration) == row[0])
        target = request["target"]
        if target["kind"] == "checkpoint_epoch":
            require(target["runtime_epoch"] == registration["runtime_epoch"])
        elif target["kind"] == "team_owner":
            require(target["fence"]["incarnationId"] == registration["team_incarnation"])
        else:
            raise Refused()
        domain = registration["domain"]
        established = self.db.execute("SELECT observation FROM excluded_domains WHERE registration=?", (encoded(registration),)).fetchone()
        if established:
            observation = json.loads(established[0])
        elif domain["kind"] == "native_boot":
            kind, machine, boot = native_identity()
            require(kind == domain["platform"] and machine == domain["machine_id"] and boot != domain["boot_id"])
            observation = {"kind": "native_boot", "platform": kind, "machine_id": machine,
                "previous_boot_id": domain["boot_id"], "observed_boot_id": boot}
        elif domain["kind"] == "kubernetes":
            observation = self.exclude_pod(domain, json.loads(row[1]))
        else:
            raise Refused()
        receipt = {"challenge": request, "controller_id": self.config["controller_id"],
            "qualification_id": self.config["qualification_id"], "observation": observation}
        self.db.execute("INSERT OR IGNORE INTO excluded_domains VALUES (?,?)", (encoded(registration), encoded(observation)))
        self.db.execute("INSERT INTO exclusions VALUES (?,?,?)", (challenge_id, request_data, encoded(receipt)))
        self.db.commit()
        return receipt

    def exclude_pod(self, domain, private):
        require(domain["node_uid"] == self.config["node_uid"] and domain["cluster_id"] == self.config["cluster_id"])
        boot = pathlib.Path("/proc/sys/kernel/random/boot_id").read_text().strip()
        # An unreachable controller/node never reaches this code. A different
        # node boot needs its own qualified infrastructure contract; do not emit
        # a made-up power-fencing receipt.
        require(boot == domain["node_boot_id"])
        self.inhibit_kubelet(private["kubelet"])
        group = pathlib.Path(private["cgroup"])
        descriptor = os.open(group, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            stat = os.fstat(descriptor)
            require(stat.st_dev == private["device"] and stat.st_ino == private["inode"])
            def write(name, value):
                fd = os.open(name, os.O_WRONLY | os.O_NOFOLLOW, dir_fd=descriptor)
                try:
                    require(os.write(fd, value) == len(value))
                finally:
                    os.close(fd)
            def events():
                fd = os.open("cgroup.events", os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptor)
                with os.fdopen(fd) as stream:
                    return dict(line.split() for line in stream)
            # Kubelet is already excluded. Freeze the exact Pod domain before
            # deletion so no original descendant can escape the kill window.
            write("cgroup.freeze", b"1")
            deadline = time.monotonic() + 10
            while events().get("frozen") != "1":
                require(time.monotonic() < deadline)
                time.sleep(0.05)
            try:
                self.kubernetes(private["pod_path"], "DELETE", {"apiVersion": "v1", "kind": "DeleteOptions",
                    "preconditions": {"uid": domain["pod_uid"]}, "gracePeriodSeconds": 0})
            except urllib.error.HTTPError as error:
                # Absence is not proof. We still need exact original cgroup and
                # CRI observations below; a replacement UID is never deleted.
                require(error.code in (404, 409))
            write("cgroup.kill", b"1")
            deadline = time.monotonic() + 10
            while events().get("populated") != "0":
                require(time.monotonic() < deadline)
                time.sleep(0.05)
            # The exact retained cgroup was observed empty before CRI cleanup
            # can remove it; disappearance is never used for that observation.
            self.cri("stopp", domain["sandbox_id"])
            sandbox = json.loads(self.cri("inspectp", domain["sandbox_id"]))
            require(sandbox["status"]["state"] == "SANDBOX_NOTREADY")
            for container in domain["container_ids"]:
                status = json.loads(self.cri("inspect", container))["status"]
                require(status["id"] == container and status["state"] == "CONTAINER_EXITED")
            # GC'd CRI records remain a refusal. The empty original cgroup
            # observation above is retained even if subsequent cleanup removes it.
            return {"kind": "kubernetes_runtime", "node_uid": domain["node_uid"],
                "node_boot_id": boot, "sandbox_id": domain["sandbox_id"],
                "stopped_container_ids": domain["container_ids"], "sandbox_stopped": True,
                "descendant_cgroup_empty": True, "restart_inhibited": True,
                "kubelet_unit": private["kubelet"]["unit"], "kubelet_cgroup_empty": True,
                "kubelet_runtime_masked": True}
        finally:
            os.close(descriptor)


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        try:
            self.connection.settimeout(30)
            size = int(self.headers.get("Content-Length", "0"))
            require(0 < size <= 65536 and not self.headers.get("Transfer-Encoding"))
            fingerprint = hashlib.sha256(self.connection.getpeercert(binary_form=True)).hexdigest()
            request = json.loads(self.rfile.read(size), object_pairs_hook=closed_object)
            with self.server.controller.lock:
                if self.path == "/v1/containment/register":
                    result = self.server.controller.register(request, fingerprint)
                elif self.path == "/v1/containment/exclude":
                    result = self.server.controller.exclude(request, fingerprint)
                else:
                    raise Refused()
            data = encoded(result).encode()
            self.send_response(200)
        except Exception:
            # A failed command may already have stopped the domain. Absence of a
            # committed observation is still NOT successful exclusion.
            data = b'{"error":"supervisor_exclusion_unproven"}'
            self.send_response(409)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


def windows_service(server):
    """SCM dispatcher for the independent controller; no external service wrapper."""
    import ctypes
    from ctypes import wintypes
    require(platform.system() == "Windows")
    api = ctypes.WinDLL("advapi32", use_last_error=True)
    class Status(ctypes.Structure):
        _fields_ = [(name, wintypes.DWORD) for name in (
            "service_type", "state", "accepted", "win32_exit", "service_exit", "checkpoint", "wait_hint")]
    control_type = ctypes.WINFUNCTYPE(None, wintypes.DWORD)
    entry_type = ctypes.WINFUNCTYPE(None, wintypes.DWORD, ctypes.POINTER(wintypes.LPWSTR))
    class Entry(ctypes.Structure):
        _fields_ = [("name", wintypes.LPWSTR), ("entry", entry_type)]
    api.RegisterServiceCtrlHandlerW.argtypes = [wintypes.LPCWSTR, control_type]
    api.RegisterServiceCtrlHandlerW.restype = wintypes.HANDLE
    api.SetServiceStatus.argtypes = [wintypes.HANDLE, ctypes.POINTER(Status)]
    api.SetServiceStatus.restype = wintypes.BOOL
    api.StartServiceCtrlDispatcherW.argtypes = [ctypes.POINTER(Entry)]
    api.StartServiceCtrlDispatcherW.restype = wintypes.BOOL
    handle = None
    def publish(state, accepted=0):
        status = Status(0x10, state, accepted, 0, 0, 0, 10000 if state in (2, 3) else 0)
        require(api.SetServiceStatus(handle, ctypes.byref(status)))
    @control_type
    def control(event):
        if event in (1, 5):
            publish(3)
            # shutdown must run outside serve_forever's own thread.
            threading.Thread(target=server.shutdown, daemon=True).start()
    @entry_type
    def entry(_argc, _argv):
        nonlocal handle
        handle = api.RegisterServiceCtrlHandlerW("UARSupervisor", control)
        if not handle:
            return
        publish(2)
        publish(4, 1 | 4)
        try:
            server.serve_forever()
        finally:
            server.server_close()
            publish(1)
    table = (Entry * 2)(Entry("UARSupervisor", entry), Entry(None, entry_type()))
    require(api.StartServiceCtrlDispatcherW(table))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True)
    parser.add_argument("--windows-service", action="store_true")
    parser.add_argument("--qualification-candidate", action="store_true",
        help="isolated deployment only; requires an explicitly unqualified candidate record")
    args = parser.parse_args()
    config = json.loads(pathlib.Path(args.config).read_text())
    controller = Controller(config, qualification_candidate=args.qualification_candidate)
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    context.minimum_version = ssl.TLSVersion.TLSv1_2
    context.load_cert_chain(config["server_certificate"], config["server_key"])
    context.load_verify_locations(config["client_ca"])
    context.verify_mode = ssl.CERT_REQUIRED
    server = http.server.HTTPServer((config["listen_host"], config["listen_port"]), Handler)
    server.socket = context.wrap_socket(server.socket, server_side=True)
    server.socket.settimeout(30)
    server.controller = controller
    if args.windows_service:
        windows_service(server)
    else:
        server.serve_forever()


if __name__ == "__main__":
    main()
