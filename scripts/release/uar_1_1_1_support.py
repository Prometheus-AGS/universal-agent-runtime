"""Owned real-process collaborators for the UAR 1.1.1 phase integration program.

Importing this module performs no I/O. Only the final phase runner invokes it.
"""
import base64
import hashlib
import hmac
import http.client
import ipaddress
import json
import math
import os
from pathlib import Path
import re
import secrets
import signal
import socket
import subprocess
import threading
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parents[2]
SURREAL_IMAGE = "surrealdb/surrealdb:v3.3.0@sha256:681c6c22c287421b5c7d99e0fde79b6e0d32c36c1ddeaab2762a1661cb04cd20"
GROUPS = ("binding", "streams", "mcp-outcomes", "credentials", "recovery", "combined")


class Blocked(RuntimeError):
    """A declared observation cannot be made with the supplied collaborators."""


class StreamDeadlineExceeded(TimeoutError):
    """The whole observer read expired, including an otherwise active stream."""

    def __init__(self, partial):
        super().__init__("observer stream total deadline exceeded (%s bytes retained)" % len(partial))
        self.partial = partial


def require(value, message):
    if not value:
        raise AssertionError(message)


def wait_for(label, probe, timeout=180):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        try:
            result = probe()
            if result:
                return result
            last = "probe_returned_false"
        except (OSError, urllib.error.URLError, subprocess.SubprocessError) as error:
            last = type(error).__name__
        time.sleep(0.2)
    raise Blocked(label + " did not become ready: " + str(last))


class Http:
    def __init__(self, base_url, token=None, admin_key=None):
        self.base_url = base_url.rstrip("/")
        self.token = token
        self.admin_key = admin_key

    def request(self, method, path, body=None, expected=200, headers=None,
                return_status=False, timeout=180):
        values = {"Accept": "application/json"}
        if self.token:
            values["Authorization"] = "Bearer " + self.token
        if self.admin_key:
            values["X-UAR-Admin-Key"] = self.admin_key
        values.update(headers or {})
        if body is not None and not isinstance(body, bytes):
            body = json.dumps(body, ensure_ascii=False).encode()
            values.setdefault("Content-Type", "application/json")
        request = urllib.request.Request(self.base_url + path, data=body,
                                         headers=values, method=method)
        accepted = expected if isinstance(expected, (tuple, list, set)) else (expected,)
        try:
            response = urllib.request.urlopen(request, timeout=timeout)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            status = response.code
            require(status in accepted,
                    "%s %s status %s; expected %s" % (method, path, status, accepted))
            try:
                result = json.loads(raw) if raw else None
            except (ValueError, UnicodeDecodeError):
                result = raw.decode("utf-8", errors="replace")
            return (status, result) if return_status else result

    def stream(self, path, timeout=180, *, total_timeout=240):
        headers = {"Accept": "text/event-stream"}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        require(math.isfinite(total_timeout) and total_timeout > 0,
                "observer stream requires a finite positive total deadline")
        deadline = time.monotonic() + total_timeout
        url = urllib.parse.urlsplit(self.base_url + path)
        require(url.scheme in ("http", "https") and url.hostname,
                "observer stream requires an HTTP endpoint")
        connection_type = http.client.HTTPSConnection if url.scheme == "https" else http.client.HTTPConnection
        connection = connection_type(url.hostname, url.port, timeout=min(timeout, total_timeout))
        expired = threading.Event()
        chunks = []
        timer = None
        try:
            connection.connect()
            transport = connection.sock
            require(transport is not None, "observer stream connection has no socket")

            def expire():
                expired.set()
                try:
                    transport.shutdown(socket.SHUT_RDWR)
                except OSError:
                    pass

            # Heartbeats keep an inactivity timeout alive. Shutdown also interrupts
            # response-header and HTTP chunk-framing reads at the total deadline.
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise StreamDeadlineExceeded(b"")
            timer = threading.Timer(remaining, expire)
            timer.daemon = True
            timer.start()
            target = urllib.parse.urlunsplit(("", "", url.path or "/", url.query, ""))
            connection.request("GET", target, headers=headers)
            with connection.getresponse() as response:
                require(response.status == 200, "observer stream status %s; expected200" % response.status)
                while True:
                    if expired.is_set() or time.monotonic() >= deadline:
                        raise StreamDeadlineExceeded(b"".join(chunks))
                    chunk = response.read1(65536)
                    if not chunk:
                        break
                    chunks.append(chunk)
            if expired.is_set() or time.monotonic() >= deadline:
                raise StreamDeadlineExceeded(b"".join(chunks))
            return b"".join(chunks).decode("utf-8")
        except Exception as error:
            partial = b"".join(chunks)
            if isinstance(error, http.client.IncompleteRead):
                partial += error.partial
                error.partial = partial
            if expired.is_set() and not isinstance(error, StreamDeadlineExceeded):
                raise StreamDeadlineExceeded(partial) from error
            raise
        finally:
            if timer is not None:
                timer.cancel()
                timer.join()
            connection.close()


class RuntimeLease:
    def __init__(self, resources, label, config, env=None, mounts=None):
        self.resources = resources
        self.name = resources.unique_name(label)
        self.config = config
        self.env = dict(env or {})
        self.mounts = list(mounts or [])
        self.base_url = None
        self.container = None
        self.volume = resources.unique_name(label + "-data")
        self.config_path = resources.root / (self.name + ".json")
        self.config_path.write_text(json.dumps(config))
        self.config_path.chmod(0o644)
        resources.register_secrets(config)
        resources.register_secrets(self.env)
        self.http_port = resources.free_port()
        self.grpc_port = resources.free_port()

    def start(self):
        r = self.resources
        r.docker("volume", "create", self.volume)
        if self.volume not in r.volumes:
            r.volumes.append(self.volume)
        args = ["run", "--detach", "--name", self.name, "--network", r.network,
                "--add-host", "host.docker.internal:host-gateway",
                "--publish", "127.0.0.1:%s:1906" % self.http_port,
                "--publish", "127.0.0.1:%s:50051" % self.grpc_port,
                "--volume", str(self.config_path) + ":/opt/uar/phase-config.json:ro",
                "--volume", self.volume + ":/var/lib/uar"]
        for mount in self.mounts:
            if isinstance(mount, (tuple, list)):
                mount = ":".join(str(part) for part in mount)
            args.extend(["--volume", str(mount)])
        for key, value in self.effective_environment().items():
            args.extend(["--env", key + "=" + str(value)])
        args.extend([r.image, "universal-agent-runtime", "--config", "/opt/uar/phase-config.json"])
        r.containers.append(self.name)
        self.container = r.docker(*args).stdout.strip()
        self.base_url = "http://127.0.0.1:%s" % self.http_port
        wait_for(self.name, self._health)
        return self

    def _health(self):
        status = self.resources.docker("inspect", "--format", "{{.State.Status}}", self.name).stdout.strip()
        if status == "exited":
            self.resources.capture_container(self.name)
            raise Blocked("runtime exited before health: " + self.name)
        status, _ = Http(self.base_url).request("GET", "/health", expected=(200, 503), return_status=True)
        return status == 200

    def restart(self):
        self.resources.docker("restart", self.name)
        wait_for(self.name + " restart", self._health)
        return self

    def effective_environment(self):
        # Image defaults are later loader sources than the mounted file. Bind
        # these two shipped defaults to this fixture's actual selected store.
        persistence = self.config["persistence"]
        values = {"UAR_PERSISTENCE__PROVIDER": persistence["provider"],
                  "UAR_PERSISTENCE__DATABASE_URL": persistence["database_url"]}
        values.update(self.env)
        return values

    def cold_restart(self):
        self.stop()
        return self.start()

    def http(self, token=None, admin_key=None):
        return Http(self.base_url, token, admin_key)

    def exec(self, *args, check=True):
        return self.resources.docker("exec", self.name, *args, check=check)

    def logs(self, *, redact=True):
        result = self.resources.docker("logs", self.name, check=False)
        raw = result.stdout + result.stderr
        # Leak assertions inspect the raw bytes in memory. Retained command and
        # container transcripts are always redacted by the resources layer.
        return self.resources.redact(raw) if redact else raw

    def stop(self):
        if self.container:
            self.resources.capture_container(self.name)
            result = self.resources.docker("rm", "--force", self.name, check=False)
            require(result.returncode == 0, "owned runtime container cleanup failed")
            self.container = None
            self.resources.containers.remove(self.name)

    def __enter__(self):
        return self.start()

    def __exit__(self, *_args):
        self.stop()


class Resources:
    def __init__(self, manifest, infrastructure, evidence):
        self.manifest = manifest
        self.infrastructure = infrastructure
        self.root = Path(manifest["resource_root"])
        self.evidence = Path(evidence)
        self.image = manifest["image"]
        self.platform = manifest["platform"]
        self.network = self.unique_name("network")
        self.network_created = False
        self.containers = []
        self.volumes = []
        self.processes = []
        self.files = []
        self.known_secrets = set()
        self.signing_secret = secrets.token_hex(32)
        self.admin_key = secrets.token_hex(24)
        self.known_secrets.update([self.signing_secret, self.admin_key])
        self.register_secrets(infrastructure)
        self.backends = {}
        self.command_counter = 0

    def unique_name(self, label):
        return "uar111-" + re.sub(r"[^a-z0-9-]", "-", label.lower())[:30] + "-" + uuid.uuid4().hex[:10]

    @staticmethod
    def free_port():
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            return sock.getsockname()[1]

    def register_secrets(self, value, key=""):
        if isinstance(value, dict):
            for child_key, child in value.items():
                self.register_secrets(child, str(child_key))
        elif isinstance(value, (list, tuple)):
            for child in value:
                self.register_secrets(child, key)
        elif isinstance(value, str) and value and any(
                word in key.lower() for word in ("secret", "password", "pass", "token", "api_key", "authorization")):
            self.known_secrets.add(value)

    def redact(self, value):
        value = str(value)
        for secret in sorted(self.known_secrets, key=len, reverse=True):
            value = value.replace(secret, "[redacted]")
        value = re.sub(r"(?i)Bearer\s+[^\s\"']+", "[redacted]", value)
        value = re.sub(r"eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+", "[redacted]", value)
        return re.sub(r"(://[^/@\s:]+:)[^/@\s]+@", r"\1[redacted]@", value)

    def run_command(self, argv, *, check=True, env=None, cwd=ROOT, timeout=900, **_kwargs):
        self.command_counter += 1
        identity = "%05d" % self.command_counter
        receipt = self.evidence / ("command-" + identity + ".json")
        record = {"argv": [self.redact(part) for part in argv], "cwd": str(cwd),
                  "started_at": time.time(), "status": "started"}
        receipt.write_text(json.dumps(record, indent=2))
        child_env = os.environ.copy()
        child_env.pop("DOCKER_DEFAULT_PLATFORM", None)
        if env:
            self.register_secrets(env)
            child_env.update({str(k): str(v) for k, v in env.items()})
        try:
            result = subprocess.run([str(x) for x in argv], cwd=cwd, env=child_env,
                                    text=True, capture_output=True, timeout=timeout)
        except BaseException as error:
            record.update(status="interrupted" if isinstance(error, KeyboardInterrupt) else "failed",
                          error=type(error).__name__, finished_at=time.time())
            receipt.write_text(json.dumps(record, indent=2))
            raise
        (self.evidence / ("command-" + identity + ".log")).write_text(self.redact(result.stdout + result.stderr))
        record.update(status="passed" if result.returncode == 0 else "failed",
                      exit_code=result.returncode, finished_at=time.time())
        receipt.write_text(json.dumps(record, indent=2))
        if check and result.returncode:
            raise RuntimeError("command failed (%s), see %s" % (result.returncode, receipt))
        return result

    def docker(self, *args, **kwargs):
        return self.run_command(["docker", *args], **kwargs)

    def start_process(self, argv, *, env=None, cwd=None, label="peer", **kwargs):
        path = self.root / (self.unique_name(label) + ".log")
        output = path.open("w")
        self.files.append(output)
        child_env = {k: v for k, v in os.environ.items()
                     if k in ("PATH", "LANG", "LC_ALL", "SYSTEMROOT", "TMPDIR")}
        child_home = self.root / self.unique_name("peer-home")
        child_home.mkdir(mode=0o700)
        child_env.update(HOME=str(child_home), XDG_DATA_HOME=str(child_home / "data"),
                         XDG_CACHE_HOME=str(child_home / "cache"))
        if env:
            self.register_secrets(env)
            child_env.update({str(k): str(v) for k, v in env.items()})
        process = subprocess.Popen([str(x) for x in argv], cwd=cwd or ROOT,
                                   env=child_env, stdout=kwargs.get("stdout", output),
                                   stderr=kwargs.get("stderr", subprocess.STDOUT), start_new_session=True)
        self.processes.append((process, path))
        return process

    def start(self):
        result = self.docker("network", "create", self.network, check=False)
        if result.returncode:
            raise Blocked("isolated Docker network unavailable")
        self.network_created = True
        return self

    def ensure_backend(self, kind):
        if kind in self.backends:
            return self.backends[kind]
        settings = self.infrastructure.get(kind)
        if not isinstance(settings, dict):
            raise Blocked("private infrastructure lacks " + kind)
        name = self.unique_name(kind)
        volume = self.unique_name(kind + "-data")
        user = settings.get("username", "root" if kind == "surreal" else "postgres")
        password = settings.get("password")
        if not isinstance(password, str) or not password:
            raise Blocked("isolated backend password missing: " + kind)
        self.known_secrets.add(password)
        self.docker("volume", "create", volume)
        self.volumes.append(volume)
        self.containers.append(name)
        if kind == "surreal":
            image = settings.get("image", SURREAL_IMAGE)
            require(image == SURREAL_IMAGE, "Surreal image must match versions.toml")
            self.docker("run", "--detach", "--name", name, "--network", self.network,
                        "--network-alias", "surreal", "--publish", "127.0.0.1::8000",
                        "--user", "0", "--volume", volume + ":/data", image,
                        "start", "--user", user, "--pass", password, "--bind", "0.0.0.0:8000", "rocksdb:/data/phase.db")
            address = self.docker("port", name, "8000/tcp").stdout.strip()
            wait_for("isolated Surreal", lambda: Http("http://" + address).request(
                "GET", "/health", return_status=True)[0] == 200)
            entry = dict(name=name, username=user, password=password, url="http://surreal:8000",
                         host_url="http://" + address, image=image)
        elif kind == "postgres":
            image = settings.get("image")
            if not isinstance(image, str) or "@sha256:" not in image:
                raise Blocked("private infrastructure must pin the isolated PostgreSQL image digest")
            self.docker("run", "--detach", "--name", name, "--network", self.network,
                        "--network-alias", "postgres", "--publish", "127.0.0.1::5432",
                        "--env", "POSTGRES_USER=" + user, "--env", "POSTGRES_PASSWORD=" + password,
                        "--volume", volume + ":/var/lib/postgresql/data", image)
            wait_for("isolated PostgreSQL", lambda: self.docker(
                "exec", name, "pg_isready", "-U", user, check=False).returncode == 0)
            address = self.docker("port", name, "5432/tcp").stdout.strip()
            entry = dict(name=name, username=user, password=password, image=image,
                         host_address=address)
        else:
            raise Blocked("unsupported isolated backend: " + kind)
        self.backends[kind] = entry
        return entry

    def base_config(self, backend="surreal", database=None):
        database = database or self.unique_name("db").replace("-", "_")
        security = dict(jwt_required=True, jwt_secret=self.signing_secret,
                        jwt_algorithm="HS256", jwt_issuer="phase-issuer", jwt_audience="phase-runtime",
                        jwt_validate_nbf=True, settings_mutation_auth_required=True,
                        settings_admin_key=self.admin_key, trusted_host_principals=[dict(
                            issuer="phase-issuer", subject="phase-host", tenant_id="phase-tenant", host_id="phase-host")])
        persistence = dict(provider=backend, vector_dimension=384, external_cache_enabled=False)
        if backend == "surreal":
            db = self.ensure_backend("surreal")
            persistence.update(database_url=db["url"], surreal_user=db["username"],
                               surreal_pass=db["password"], surreal_ns="uar111", surreal_db=database)
        elif backend == "postgres":
            db = self.ensure_backend("postgres")
            require(re.fullmatch(r"[a-zA-Z0-9_]+", database), "invalid owned database identifier")
            self.docker("exec", db["name"], "createdb", "-U", db["username"], database)
            persistence["database_url"] = "postgres://%s:%s@postgres:5432/%s" % (
                urllib.parse.quote(db["username"], safe=""), urllib.parse.quote(db["password"], safe=""), database)
        elif backend == "local":
            persistence.update(provider="surreal", database_url="surrealkv:///var/lib/uar/data/phase.db")
        else:
            raise Blocked("unsupported runtime persistence profile")
        return dict(server=dict(host="0.0.0.0", port=1906, grpc_port=50051, cors_allowed_origins=[]),
                    security=security, persistence=persistence, memory=dict(enabled=False), providers=[],
                    resilience=dict(rate_limit_enabled=False),
                    native_tools=dict(file_tools_enabled=False, web_fetch_enabled=False,
                                      terminal_exec_enabled=False, session_search_enabled=False))

    def jwt(self, subject, roles, issuer=None, tenant=None, host=False):
        def encode(value):
            return base64.urlsafe_b64encode(value).rstrip(b"=").decode()
        now = int(time.time())
        claims = dict(sub=subject, roles=roles, iss=issuer or "phase-issuer",
                      tenant_id=tenant or "phase-tenant", aud="phase-runtime", nbf=now - 5,
                      exp=now + 14400, uar_credential_kind="issuer")
        if host:
            claims["uar_instance_id"] = "phase-host"
        message = encode(json.dumps(dict(alg="HS256", typ="JWT")).encode()) + "." + encode(json.dumps(claims).encode())
        token = message + "." + encode(hmac.new(self.signing_secret.encode(), message.encode(), hashlib.sha256).digest())
        self.known_secrets.add(token)
        return token

    def relay(self, port):
        name = self.unique_name("relay")
        self.containers.append(name)
        script = ROOT / "scripts/release/uar_1_1_loopback_relay.py"
        self.docker("run", "--detach", "--name", name, "--network", self.network,
                    "--add-host", "host.docker.internal:host-gateway", "--entrypoint", "python3",
                    "--volume", str(script) + ":/relay.py:ro", self.image, "/relay.py",
                    "--listen-host", "0.0.0.0", "--listen-port", str(port),
                    "--upstream-host", "host.docker.internal", "--upstream-port", str(port),
                    "--ready-file", "/tmp/ready", "--pid-file", "/tmp/pid")
        wait_for("peer relay", lambda: self.docker("exec", name, "cat", "/tmp/ready", check=False).returncode == 0)
        data = json.loads(self.docker("inspect", name).stdout)[0]
        address = data["NetworkSettings"]["Networks"][self.network]["IPAddress"]
        require(any(ipaddress.ip_address(address) in ipaddress.ip_network(cidr)
                    for cidr in ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16")), "relay requires RFC1918 address")
        return "http://%s:%s" % (address, port)

    def surreal_query(self, database, sql, namespace="uar111"):
        db = self.ensure_backend("surreal")
        require(namespace == "uar111" and re.fullmatch(r"[a-zA-Z0-9_]+", database), "query outside owned namespace")
        authorization = base64.b64encode((db["username"] + ":" + db["password"]).encode()).decode()
        return Http(db["host_url"]).request("POST", "/sql", sql.encode(), headers={
            "Authorization": "Basic " + authorization, "Surreal-NS": namespace,
            "Surreal-DB": database, "Content-Type": "text/plain"})

    def postgres_query(self, database, sql):
        db = self.ensure_backend("postgres")
        require(re.fullmatch(r"[a-zA-Z0-9_]+", database), "query outside owned database")
        return self.docker("exec", db["name"], "psql", "-X", "-v", "ON_ERROR_STOP=1",
                           "-U", db["username"], "-d", database, "--csv", "-c", sql).stdout

    def new_runtime(self, label, config, env=None, mounts=None):
        return RuntimeLease(self, label, config, env, mounts)

    def capture_container(self, name):
        result = self.docker("logs", name, check=False)
        path = self.evidence / (name + "-" + uuid.uuid4().hex[:6] + ".log")
        path.write_text(self.redact(result.stdout + result.stderr))

    def close(self):
        errors = []
        for process, path in reversed(self.processes):
            try:
                if process.poll() is None:
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    try:
                        process.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        process.wait()
                if path.exists():
                    (self.evidence / path.name).write_text(self.redact(path.read_text(errors="replace")))
            except Exception as error:
                errors.append(type(error).__name__)
        for name in reversed(self.containers):
            try:
                self.capture_container(name)
            except Exception as error:
                errors.append(type(error).__name__)
            finally:
                try:
                    result = self.docker("rm", "--force", name, check=False)
                    if result.returncode:
                        errors.append("container_cleanup_failed")
                except Exception as error:
                    errors.append(type(error).__name__)
        for volume in reversed(self.volumes):
            try:
                result = self.docker("volume", "rm", volume, check=False)
                if result.returncode:
                    errors.append("volume_cleanup_failed")
            except Exception as error:
                errors.append(type(error).__name__)
        if self.network_created:
            try:
                result = self.docker("network", "rm", self.network, check=False)
                if result.returncode:
                    errors.append("network_cleanup_failed")
            except Exception as error:
                errors.append(type(error).__name__)
        for file in self.files:
            file.close()
        if errors:
            raise RuntimeError("owned resource cleanup incomplete: " + ",".join(errors))

    def context(self, group, evidence):
        return dict(group=group, resources=self, http=Http, jwt=self.jwt,
                    base_config=self.base_config, new_runtime=self.new_runtime,
                    run_command=self.run_command, resource_root=self.root,
                    evidence_root=Path(evidence), infrastructure=self.infrastructure,
                    image=self.image, platform=self.platform,
                    identity=dict(issuer="phase-issuer", tenant="phase-tenant", host_subject="phase-host"),
                    admin_key=self.admin_key, require=require, wait_for=wait_for)
