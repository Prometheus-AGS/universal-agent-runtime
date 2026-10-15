"""Installed-image checks for UAR's production default configuration.

This module is called only by ``uar_1_1_acceptance.py`` after the KBD phase
implementation boundary.  It deliberately starts the candidate image without
a UAR config file, command override, port publication, or model settings.
The image binds its default loopback listener, so every HTTP request is issued
by Python already present in the runtime image through ``docker exec``.
"""

import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import secrets
import time
import uuid


DEFAULT_STARTUP_TIMEOUT_SECONDS = 30
EGRESS_OBSERVATION_SECONDS = 300
EGRESS_SAMPLE_SECONDS = 10


_EGRESS_CAPTURE = r"""
import ipaddress
import json
import pathlib
import socket
import struct
import sys
import time

duration = int(sys.argv[1])
sock = socket.socket(socket.AF_PACKET, socket.SOCK_RAW, socket.htons(3))
sock.settimeout(0.25)
started = time.monotonic()
pathlib.Path('/tmp/uar-egress-ready').write_text(str(started))
observations = []
while time.monotonic() - started < duration:
    try:
        packet, address = sock.recvfrom(65535)
    except TimeoutError:
        continue
    if address[0] == 'lo' or address[2] != 4:
        continue
    row = {'seconds': time.monotonic() - started, 'interface': address[0],
           'bytes': len(packet), 'ether_type': packet[12:14].hex(),
           'classification': 'unexpected_egress'}
    if packet[12:14] == b'\x08\x06' and len(packet) >= 42:
        operation = struct.unpack('!H', packet[20:22])[0]
        sender, target = socket.inet_ntoa(packet[28:32]), socket.inet_ntoa(packet[38:42])
        row.update(operation=operation, sender=sender, target=target)
        if operation == 1 and sender == target:
            row['classification'] = 'gratuitous_arp'
    elif packet[12:14] == b'\x86\xdd' and len(packet) >= 55:
        source = socket.inet_ntop(socket.AF_INET6, packet[22:38])
        destination = socket.inet_ntop(socket.AF_INET6, packet[38:54])
        protocol, offset = packet[20], 54
        # The observed multicast-listener report uses a hop-by-hop header.
        if protocol == 0 and len(packet) >= offset + 2:
            protocol, extension_length = packet[offset], (packet[offset + 1] + 1) * 8
            offset += extension_length
        row.update(source=source, destination=destination, protocol=protocol)
        if protocol == 58 and len(packet) > offset + 1:
            kind, code = packet[offset], packet[offset + 1]
            row.update(icmpv6_type=kind, icmpv6_code=code)
            link_multicast = ipaddress.ip_address(destination) in ipaddress.ip_network('ff02::/16')
            if code == 0 and link_multicast:
                if kind in (133, 135, 136) and packet[21] == 255:
                    row['classification'] = 'ipv6_neighbor_discovery'
                elif kind == 143 and destination == 'ff02::16' and packet[21] == 1:
                    row['classification'] = 'ipv6_multicast_listener_report'
    elif packet[12:14] == b'\x08\x00' and len(packet) >= 34:
        row.update(source=socket.inet_ntoa(packet[26:30]),
                   destination=socket.inet_ntoa(packet[30:34]), protocol=packet[23])
    observations.append(row)
    pathlib.Path('/tmp/uar-egress-observations.json').write_text(json.dumps(observations))
received, dropped = struct.unpack('II', sock.getsockopt(263, 6, 8))
result = {'duration_seconds': time.monotonic() - started,
          'received_packets': received, 'dropped_packets': dropped,
          'observations': observations}
pathlib.Path('/tmp/uar-egress-result.json').write_text(json.dumps(result))
"""


_HTTP_REQUEST = r"""
import json
import sys
import urllib.error
import urllib.request

method, path, token, admin_key, origin, encoded_body = sys.argv[1:]
headers = {"Accept": "application/json"}
if token:
    headers["Authorization"] = "Bearer " + token
if admin_key:
    headers["X-UAR-Admin-Key"] = admin_key
if origin:
    headers["Origin"] = origin
data = encoded_body.encode("utf-8") if encoded_body else None
if data is not None:
    headers["Content-Type"] = "application/json"
request = urllib.request.Request(
    "http://127.0.0.1:1906" + path,
    data=data,
    method=method,
    headers=headers,
)
try:
    with urllib.request.urlopen(request, timeout=20) as response:
        status = response.status
        response_headers = dict(response.headers.items())
        raw = response.read()
except urllib.error.HTTPError as error:
    status = error.code
    response_headers = dict(error.headers.items())
    raw = error.read()
try:
    body = json.loads(raw)
except (UnicodeDecodeError, json.JSONDecodeError):
    body = raw.decode("utf-8", errors="replace")
print(json.dumps({"status": status, "headers": response_headers, "body": body}))
"""


_LISTENERS = r"""
import ipaddress
import json

def is_loopback(address, version):
    raw = bytes.fromhex(address)
    if version == 4:
        return ipaddress.IPv4Address(raw[::-1]).is_loopback
    network_order = b"".join(raw[index:index + 4][::-1] for index in range(0, len(raw), 4))
    return ipaddress.IPv6Address(network_order).is_loopback

def listeners(path, version):
    rows = []
    for line in open(path, encoding="utf-8"):
        fields = line.split()
        if len(fields) < 4 or fields[3] != "0A":
            continue
        address, port = fields[1].split(":", 1)
        loopback = is_loopback(address, version)
        rows.append({
            "family": "ipv%d" % version,
            "address_hex": address,
            "port": int(port, 16),
            "loopback": loopback,
        })
    return rows

print(json.dumps(listeners("/proc/net/tcp", 4) + listeners("/proc/net/tcp6", 6), sort_keys=True))
"""


_NETWORK_COUNTERS = r"""
import json

counters = {}
for line in open("/proc/net/dev", encoding="utf-8"):
    if ":" not in line:
        continue
    interface, fields = line.split(":", 1)
    fields = fields.split()
    if interface.strip() == "lo" or len(fields) < 16:
        continue
    counters[interface.strip()] = {
        "rx_bytes": int(fields[0]),
        "rx_packets": int(fields[1]),
        "tx_bytes": int(fields[8]),
        "tx_packets": int(fields[9]),
    }
print(json.dumps(counters, sort_keys=True))
"""


def _b64url(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


def _token(secret, subject, roles):
    now = int(time.time())
    header = {"alg": "HS256", "typ": "JWT"}
    payload = {
        "sub": subject,
        "tenant_id": "uar-default-profile",
        "roles": roles,
        "nbf": now - 5,
        "exp": now + 900,
    }
    signing_input = (
        _b64url(json.dumps(header, separators=(",", ":")).encode("utf-8"))
        + "."
        + _b64url(json.dumps(payload, separators=(",", ":")).encode("utf-8"))
    )
    signature = hmac.new(secret.encode("utf-8"), signing_input.encode("ascii"), hashlib.sha256).digest()
    return signing_input + "." + _b64url(signature)


def _container_state(docker, container):
    inspected = docker("inspect", "--format", "{{.State.Status}}", container, check=False)
    if inspected.returncode != 0:
        return None
    return inspected.stdout.strip()


def _assert_unconfigured_startup_refused(image, docker, require):
    """Prove the image does not silently accept published fallback credentials."""
    container = "uar-default-refusal-" + uuid.uuid4().hex[:12]
    try:
        docker("run", "--detach", "--name", container, image)
        deadline = time.monotonic() + DEFAULT_STARTUP_TIMEOUT_SECONDS
        while time.monotonic() < deadline:
            if _container_state(docker, container) == "exited":
                logs = docker("logs", container, check=False)
                diagnostic = (logs.stdout + logs.stderr).strip()
                require(
                    "security.jwt_secret uses the built-in fallback" in diagnostic,
                    "unconfigured default image exited without the required fallback-credential diagnostic",
                )
                return {
                    "status": "refused",
                    "diagnostic": "fallback_jwt_secret_rejected",
                    "startup_mode": "image-default-command-without-config-or-environment",
                }
            time.sleep(0.25)
        raise RuntimeError("unconfigured default image remained running; required credential startup refusal was absent")
    finally:
        docker("rm", "--force", "--volumes", container, check=False)


def _container_request(docker, container, method, path, token="", admin_key="", origin="", body=None):
    encoded_body = "" if body is None else json.dumps(body, separators=(",", ":"))
    completed = docker(
        "exec",
        container,
        "python3",
        "-c",
        _HTTP_REQUEST,
        method,
        path,
        token,
        admin_key,
        origin,
        encoded_body,
    )
    return json.loads(completed.stdout)


def _packet_counters(docker, container):
    completed = docker("exec", container, "python3", "-c", _NETWORK_COUNTERS)
    return json.loads(completed.stdout)


def _listeners(docker, container):
    completed = docker("exec", container, "python3", "-c", _LISTENERS)
    return json.loads(completed.stdout)


def _header(headers, expected_name):
    for name, value in headers.items():
        if name.lower() == expected_name.lower():
            return value
    return None


def _assert_loopback_listeners(docker, container, require):
    listeners = _listeners(docker, container)
    require(listeners, "default profile exposed no TCP listener inventory")
    require(all(listener["loopback"] for listener in listeners), "default profile bound a non-loopback TCP listener")
    ports = {listener["port"] for listener in listeners}
    require(1906 in ports, "default profile did not listen on HTTP port 1906")
    require(50051 in ports, "default profile did not listen on A2A gRPC port 50051")
    return listeners


def _observe_egress(docker, container, require, started, startup_counters, evidence, wait_for):
    """Retain interface counters and classify every captured outgoing frame."""
    observations = [{"seconds": 0, "counters": startup_counters}]
    require(observations[0]["counters"], "default container has no non-loopback network interface to observe")
    next_sample = EGRESS_SAMPLE_SECONDS
    while next_sample <= EGRESS_OBSERVATION_SECONDS:
        delay = started + next_sample - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        observations.append({"seconds": next_sample, "counters": _packet_counters(docker, container)})
        next_sample += EGRESS_SAMPLE_SECONDS
    initial = observations[0]["counters"]
    for observation in observations[1:]:
        require(
            observation["counters"].keys() == initial.keys(),
            "non-loopback interface set changed during default egress observation",
        )
    capture = wait_for("default egress capture completion", lambda: json.loads(docker(
        "exec", container, "cat", "/tmp/uar-egress-result.json").stdout), timeout=10)
    (Path(evidence) / "default-egress.json").write_text(json.dumps(
        {"capture": capture, "interface_counters": observations}, indent=2) + "\n")
    require(capture["duration_seconds"] >= EGRESS_OBSERVATION_SECONDS, "egress capture ended early")
    require(capture["dropped_packets"] == 0, "egress capture dropped packets")
    require(all(row["classification"] != "unexpected_egress" for row in capture["observations"]),
            "default profile emitted application or unclassified traffic; see default-egress.json")
    return {
        "method": "AF_PACKET outgoing frames plus /proc/net/dev counters",
        "started_before_health_probe": True,
        "duration_seconds": EGRESS_OBSERVATION_SECONDS,
        "excluded_control_traffic": ["gratuitous_arp", "ipv6_neighbor_discovery", "ipv6_multicast_listener_report"],
        "capture": capture,
        "observations": observations,
    }


def _expected_effective_settings(effective, require):
    require(effective.get("schema_version") == 1, "effective settings returned an unexpected schema version")
    llm = effective.get("llm")
    require(llm == {"provider_host": "disabled", "model": ""}, "default profile selected a model")
    memory = effective.get("memory_embedder")
    require(memory == {"provider_host": "disabled", "model": ""}, "default profile enabled memory embedding")
    knowledge = effective.get("knowledge_base_embedder")
    require(
        isinstance(knowledge, dict)
        and knowledge.get("provider_host") in ("disabled", "local")
        and isinstance(knowledge.get("model"), str),
        "default profile returned an invalid knowledge-base embedder setting",
    )
    revision = effective.get("revision")
    require(
        isinstance(revision, str)
        and revision.startswith("sha256:")
        and len(revision) == len("sha256:") + 64,
        "effective settings omitted its sha256 revision",
    )


def _assert_agent_card(card, require):
    require(card.get("version") == "1.1.0", "public A2A agent card did not advertise version 1.1.0")
    interfaces = card.get("supportedInterfaces")
    require(isinstance(interfaces, list), "public A2A agent card omitted supported interfaces")
    required = {
        ("JSONRPC", "1.0", "http://127.0.0.1:1906/a2a/v1/compiler"),
        ("GRPC", "1.0", "http://127.0.0.1:50051"),
    }
    advertised = {
        (item.get("protocolBinding"), item.get("protocolVersion"), item.get("url"))
        for item in interfaces
        if isinstance(item, dict)
    }
    require(required <= advertised, "public A2A agent card omitted a required v1 interface")
    return interfaces


def _update_catalog_and_restore(docker, container, require, admin_token, settings_key, initial_effective):
    name = "default-revision-probe"
    path = "/api/uar/mcp/servers/" + name
    entry = {
        "name": name, "enabled": True, "transport": "remote_http",
        "url": "http://127.0.0.1:9/mcp", "headers": {},
        "grant_policy": {"destination_id": name, "trusted_hosts": ["revision-probe"],
                         "required_scopes": [], "allow_private_http": True,
                         "allowed_agent_ids": []},
    }
    # A deny-all catalog registration changes effective configuration without
    # opening a connection or disabling the default authentication/governance.
    added = _container_request(docker, container, "PUT", path, token=admin_token, body=entry)
    require(added["status"] == 200, "catalog update failed: " + json.dumps(added))
    changed = _container_request(docker, container, "GET", "/api/uar/settings/effective", token=admin_token)
    require(changed["status"] == 200, "effective settings unavailable after catalog update")
    require(any(server["name"] == name for server in changed["body"]["mcp_servers"]),
            "effective settings omitted the new catalog entry")
    require(changed["body"]["revision"] != initial_effective["revision"],
            "effective revision did not change with catalog configuration")
    removed = _container_request(docker, container, "DELETE", path, token=admin_token)
    require(removed["status"] == 200 and removed["body"].get("deleted") is True,
            "catalog restore failed: " + json.dumps(removed))
    restored = _container_request(docker, container, "GET", "/api/uar/settings/effective", token=admin_token)
    require(restored["status"] == 200 and restored["body"]["revision"] == initial_effective["revision"],
            "effective revision did not restore after catalog removal")
    return {"mutation": "deny-all MCP catalog entry", "before": initial_effective["revision"],
            "changed": changed["body"]["revision"], "restored": restored["body"]["revision"]}


def _with_secret_environment(jwt_secret, settings_key, callback):
    previous_jwt_secret = os.environ.get("UAR_SECURITY__JWT_SECRET")
    previous_settings_key = os.environ.get("UAR_SECURITY__SETTINGS_ADMIN_KEY")
    try:
        os.environ["UAR_SECURITY__JWT_SECRET"] = jwt_secret
        os.environ["UAR_SECURITY__SETTINGS_ADMIN_KEY"] = settings_key
        return callback()
    finally:
        if previous_jwt_secret is None:
            os.environ.pop("UAR_SECURITY__JWT_SECRET", None)
        else:
            os.environ["UAR_SECURITY__JWT_SECRET"] = previous_jwt_secret
        if previous_settings_key is None:
            os.environ.pop("UAR_SECURITY__SETTINGS_ADMIN_KEY", None)
        else:
            os.environ["UAR_SECURITY__SETTINGS_ADMIN_KEY"] = previous_settings_key


def _retain_logs(docker, container, path, *secrets_to_redact):
    logs = docker("logs", container, check=False)
    content = logs.stdout + logs.stderr
    for secret in secrets_to_redact:
        content = content.replace(secret, "<redacted>")
    path.write_text(content, encoding="utf-8")


def _explicit_cors_profile(image, evidence, docker, require, wait_for, jwt_secret, settings_key):
    config = Path(evidence) / "default-profile-allowed-origin.yaml"
    allowed_origin = "https://default-profile-allowed.invalid"
    config.write_text(
        "server:\n"
        "  host: \"127.0.0.1\"\n"
        "  cors_allowed_origins:\n"
        "    - \"%s\"\n" % allowed_origin,
        encoding="utf-8",
    )
    container = "uar-default-cors-" + uuid.uuid4().hex[:12]
    try:
        def start():
            return docker(
                "run", "--detach", "--name", container,
                "--env", "UAR_SECURITY__JWT_SECRET",
                "--env", "UAR_SECURITY__SETTINGS_ADMIN_KEY",
                "--volume", str(config) + ":/opt/uar/default-profile-allowed-origin.yaml:ro",
                image, "universal-agent-runtime", "--config", "/opt/uar/default-profile-allowed-origin.yaml",
            )

        _with_secret_environment(jwt_secret, settings_key, start)

        def healthy():
            if _container_state(docker, container) == "exited":
                raise RuntimeError("runtime exited before health; see retained default-profile logs")
            response = _container_request(docker, container, "GET", "/health")
            require(response["status"] == 200, "explicit CORS profile health did not return 200")
            return response

        wait_for("explicit CORS profile loopback health", healthy, timeout=120)
        positive = _container_request(docker, container, "GET", "/health", origin=allowed_origin)
        require(positive["status"] == 200, "explicit CORS profile health did not return 200")
        require(
            _header(positive["headers"], "Access-Control-Allow-Origin") == allowed_origin,
            "explicit CORS profile did not allow its exact configured origin",
        )
        unauthenticated = _container_request(docker, container, "GET", "/api/uar/settings/effective")
        require(unauthenticated["status"] == 401, "explicit CORS profile did not require authentication")
        return {"allowed_origin": allowed_origin, "anonymous_effective_status": 401}
    finally:
        _retain_logs(docker, container, Path(evidence) / "default-cors.log", jwt_secret, settings_key)
        docker("rm", "--force", "--volumes", container, check=False)


def run_default_profile(image, evidence, *, docker, require, wait_for):
    """Exercise the real image defaults and return receipt-safe observations.

    ``evidence`` is accepted for the release runner's uniform module contract;
    the returned value is embedded in its receipt and contains no credential.
    """
    startup_refusal = _assert_unconfigured_startup_refused(image, docker, require)
    container = "uar-default-profile-" + uuid.uuid4().hex[:12]
    jwt_secret = secrets.token_urlsafe(32)
    settings_key = secrets.token_urlsafe(32)
    try:
        def start():
            return docker(
                "run",
                "--detach",
                "--name",
                container,
                "--env",
                "UAR_SECURITY__JWT_SECRET",
                "--env",
                "UAR_SECURITY__SETTINGS_ADMIN_KEY",
                image,
            )

        # The image accepts these two deployment secrets through its documented
        # environment source. Their values stay out of Docker command receipts.
        _with_secret_environment(jwt_secret, settings_key, start)

        admin_token = _token(jwt_secret, "uar-default-admin", ["admin"])
        user_token = _token(jwt_secret, "uar-default-user", ["user"])
        docker("exec", "--detach", "--user", "0", container, "python3", "-c",
               _EGRESS_CAPTURE, str(EGRESS_OBSERVATION_SECONDS))
        wait_for("default egress capture readiness", lambda: docker(
            "exec", container, "cat", "/tmp/uar-egress-ready").stdout, timeout=10)
        egress_started = time.monotonic()
        startup_packets = _packet_counters(docker, container)
        require(startup_packets, "default container has no non-loopback interface after startup")

        def healthy():
            if _container_state(docker, container) == "exited":
                raise RuntimeError("runtime exited before health; see retained default-profile logs")
            response = _container_request(docker, container, "GET", "/health")
            require(response["status"] == 200, "default runtime health did not return 200")
            return response

        wait_for("default image loopback health", healthy, timeout=120)
        listeners = _assert_loopback_listeners(docker, container, require)
        anonymous_effective = _container_request(docker, container, "GET", "/api/uar/settings/effective")
        require(anonymous_effective["status"] == 401, "anonymous effective settings did not return 401")
        anonymous_agents = _container_request(docker, container, "GET", "/api/agents")
        require(anonymous_agents["status"] == 401, "default authentication middleware admitted anonymous agent discovery")
        user_effective = _container_request(
            docker, container, "GET", "/api/uar/settings/effective", token=user_token
        )
        require(user_effective["status"] == 403, "non-admin effective settings did not return 403")
        effective_response = _container_request(
            docker, container, "GET", "/api/uar/settings/effective", token=admin_token
        )
        require(effective_response["status"] == 200, "admin effective settings did not return 200")
        _expected_effective_settings(effective_response["body"], require)

        capabilities = _container_request(
            docker, container, "GET", "/api/uar/capabilities", token=user_token
        )
        require(capabilities["status"] == 200, "capabilities did not return 200")
        require(
            isinstance(capabilities["body"].get("uar_version"), str)
            and capabilities["body"]["uar_version"],
            "capabilities omitted the UAR version",
        )
        require(
            capabilities["body"].get("authentication", {}).get("principalMode") == "token-subject",
            "default profile did not report token-subject authentication",
        )
        agent_card = _container_request(docker, container, "GET", "/.well-known/agent-card.json")
        require(agent_card["status"] == 200 and isinstance(agent_card["body"], dict), "public A2A agent card was unavailable")
        interfaces = _assert_agent_card(agent_card["body"], require)

        cors = _container_request(
            docker,
            container,
            "GET",
            "/health",
            origin="https://unlisted.invalid",
        )
        require(cors["status"] == 200, "loopback health did not return 200 for the CORS probe")
        require(
            _header(cors["headers"], "Access-Control-Allow-Origin") is None,
            "default profile allowed an unlisted browser origin",
        )
        no_model = _container_request(
            docker,
            container,
            "POST",
            "/v1/chat/completions",
            token=user_token,
            body={"messages": [{"role": "user", "content": "default profile model probe"}]},
        )
        require(no_model["status"] == 422, "unconfigured default model did not fail explicitly")
        require(
            isinstance(no_model["body"], dict)
            and no_model["body"].get("error", {}).get("code") == "model_not_configured",
            "unconfigured default model returned the wrong failure",
        )

        observation = _observe_egress(docker, container, require, egress_started, startup_packets, evidence, wait_for)
        revision = _update_catalog_and_restore(
            docker, container, require, admin_token, settings_key, effective_response["body"]
        )
        positive_cors = _explicit_cors_profile(
            image, evidence, docker, require, wait_for, jwt_secret, settings_key
        )
        return {
            "startup_refusal": startup_refusal,
            "runtime_profile": {
                "config_file": None,
                "command_override": None,
                "port_publication": None,
                "auth_source": [
                    "UAR_SECURITY__JWT_SECRET",
                    "UAR_SECURITY__SETTINGS_ADMIN_KEY",
                ],
                "auth_principal_mode": capabilities["body"]["authentication"]["principalMode"],
                "loopback_namespace_client": "docker exec python3",
            },
            "effective": effective_response["body"],
            "effective_revision_update": revision,
            "version": capabilities["body"]["uar_version"],
            "listeners": listeners,
            "agent_card": {"version": agent_card["body"]["version"], "supported_interfaces": interfaces},
            "auth": {"anonymous_effective_status": 401, "anonymous_agents_status": 401,
                     "non_admin_effective_status": 403},
            "cors": {"unlisted_origin": "not_allowed", "explicit_profile": positive_cors},
            "no_model": {"status": no_model["status"], "code": no_model["body"]["error"]["code"]},
            "egress": observation,
        }
    finally:
        _retain_logs(docker, container, Path(evidence) / "default-runtime.log", jwt_secret, settings_key)
        docker("rm", "--force", "--volumes", container, check=False)
