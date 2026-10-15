#!/usr/bin/env python3
"""Installed-image probes for authentication in shipped UAR runtime profiles.

The release orchestrator owns Docker lifecycle and supplies its process helpers.
This module deliberately projects deployment manifests onto the candidate image;
it never invokes Compose, Kubernetes, OpenTofu, or a source build.
"""

import hashlib
import json
import re
import secrets
import uuid
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class ProfileFailure(RuntimeError):
    pass


def _source_hash(path):
    source = ROOT / path
    if not source.is_file():
        raise ProfileFailure("shipped profile source is absent: " + path)
    return hashlib.sha256(source.read_bytes()).hexdigest()


def _masked_environment(environment):
    masked = {}
    for name, value in sorted(environment.items()):
        upper = name.upper()
        if any(token in upper for token in ("SECRET", "PASSWORD", "PASS", "TOKEN", "API_KEY")):
            masked[name] = "<generated-secret>"
        elif name.endswith("DATABASE_URL"):
            masked[name] = "<local-postgres-url>"
        elif name.endswith("REDIS_URL"):
            masked[name] = "redis://redis:6379"
        else:
            masked[name] = value
    return masked


def _write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _materialize_config(source_path, destination, upload_dir):
    """Keep non-secret source configuration while forcing secret values from env.

    The candidate receives only the staged config. Every value that could be a
    source credential is replaced by the documented environment substitution.
    """
    content = (ROOT / source_path).read_text(encoding="utf-8")
    content = re.sub(r"^(\s*(?:jwt_secret|surreal_pass|database_url|api_key|bearer_token):\s*).*$",
                     r'\1"<environment-substitution>"', content, flags=re.MULTILINE)
    content = re.sub(r"^(\s*upload_dir:\s*).*$", r'\1"' + upload_dir + r'"',
                     content, flags=re.MULTILINE)
    destination.write_text(content, encoding="utf-8")


def _source_boolean(path, expression):
    content = (ROOT / path).read_text(encoding="utf-8")
    match = re.search(expression, content, flags=re.MULTILINE)
    if match is None:
        raise ProfileFailure("could not derive shipped jwt_required from " + path)
    return match.group(1) == "true"


def _resolved_jwt_required(profile):
    auth = profile["jwt_required"]
    kind = auth["kind"]
    if kind == "yaml":
        return _source_boolean(auth["source"], r"^\s*jwt_required:\s*(true|false)\s*(?:#.*)?$")
    if kind == "literal-env":
        return _source_boolean(auth["source"], auth["expression"])
    if kind == "env-default":
        return _source_boolean(auth["source"], auth["expression"])
    if kind == "runtime-default":
        return _source_boolean("src/config.rs", r'\.set_default\("security\.jwt_required",\s*(true|false)\)\?')
    raise ProfileFailure("unknown jwt_required derivation for " + profile["name"])


def _resolved_external_cache(profile):
    cache = profile["external_cache_enabled"]
    kind = cache["kind"]
    if kind == "yaml":
        return _source_boolean(cache["source"], r"^\s*external_cache_enabled:\s*(true|false)\s*(?:#.*)?$")
    if kind in ("literal-env", "env-default"):
        return _source_boolean(cache["source"], cache["expression"])
    if kind == "runtime-default":
        return _source_boolean("src/config.rs", r'\.set_default\("persistence\.external_cache_enabled",\s*(true|false)\)\?')
    raise ProfileFailure("unknown external_cache_enabled derivation for " + profile["name"])


def _provenance_sources(profile):
    sources = list(profile["sources"])
    for derivation in (profile["jwt_required"], profile["external_cache_enabled"]):
        source = "src/config.rs" if derivation["kind"] == "runtime-default" else derivation["source"]
        if source not in sources:
            sources.append(source)
    return sources


def _profile_inventory():
    """Return each shipped runtime projection and only its deliberate overrides."""
    return [
        {
            "name": "config-embedded",
            "sources": ["config.embedded.yaml"],
            "config": "config.embedded.yaml",
            "port": 1906,
            "postgres": None,
            "redis": None,
            "environment": {
                "UAR_PERSISTENCE__PROVIDER": "surreal",
                "UAR_PERSISTENCE__DATABASE_URL": "surrealkv:///var/lib/uar/data/profile-embedded.db",
            },
            "jwt_required": {"kind": "yaml", "source": "config.embedded.yaml"},
            "external_cache_enabled": {"kind": "yaml", "source": "config.embedded.yaml"},
            "substitutions": [
                "jwt secret and settings admin key are generated for the probe",
                "relative embedded database and upload paths are projected below writable /var/lib/uar",
            ],
        },
        {
            "name": "config-remote-surreal",
            "sources": ["config.remote.surreal.yaml"],
            "config": "config.remote.surreal.yaml",
            "port": 1906,
            "postgres": None,
            "redis": None,
            "environment": {
                "UAR_PERSISTENCE__PROVIDER": "surreal",
                "UAR_PERSISTENCE__DATABASE_URL": "http://surreal:8000",
                "UAR_PERSISTENCE__SURREAL_USER": "root",
                "UAR_PERSISTENCE__SURREAL_PASS": "<generated>",
                "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1",
            },
            "jwt_required": {"kind": "yaml", "source": "config.remote.surreal.yaml"},
            "external_cache_enabled": {"kind": "yaml", "source": "config.remote.surreal.yaml"},
            "substitutions": [
                "loopback Surreal endpoint is projected to the runner's pinned local Surreal service",
                "Surreal credentials are generated by the runner service setup",
            ],
        },
        {
            "name": "config-remote-postgres",
            "sources": ["config.remote.postgres.yaml"],
            "config": "config.remote.postgres.yaml",
            "port": 1906,
            "postgres": "pgvector/pgvector:pg17",
            "redis": None,
            "environment": {
                "UAR_PERSISTENCE__PROVIDER": "postgres",
            },
            "jwt_required": {"kind": "yaml", "source": "config.remote.postgres.yaml"},
            "external_cache_enabled": {"kind": "yaml", "source": "config.remote.postgres.yaml"},
            "substitutions": [
                "loopback Postgres URL is projected to an isolated local pgvector Postgres service",
                "jwt secret and settings admin key are generated for the probe",
            ],
        },
        {
            "name": "config-test",
            "sources": ["config.test.yaml"],
            "config": "config.test.yaml",
            "port": 3001,
            "postgres": "pgvector/pgvector:pg17",
            "redis": "redis/redis-stack:latest",
            "environment": {
                "UAR_PERSISTENCE__PROVIDER": "postgres",
            },
            "jwt_required": {"kind": "yaml", "source": "config.test.yaml"},
            "external_cache_enabled": {"kind": "yaml", "source": "config.test.yaml"},
            "substitutions": [
                "committed test JWT secret is overridden by a generated secret",
                "localhost Postgres and Redis endpoints are projected to isolated local services",
            ],
        },
        {
            "name": "compose-dev",
            "sources": ["docker-compose.dev.yaml", "docker-compose.override.yaml"],
            "config": None,
            "port": 1906,
            "postgres": "pgvector/pgvector:pg17",
            "redis": "redis/redis-stack:latest",
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "LLM_MODEL": "",
            },
            "jwt_required": {"kind": "runtime-default"},
            "external_cache_enabled": {"kind": "runtime-default"},
            "substitutions": [
                "Compose build service is replaced by the supplied candidate image",
                "Compose Postgres and Redis endpoints are projected to isolated local services",
            ],
        },
        {
            "name": "compose-test",
            "sources": ["docker-compose.test.yaml"],
            "config": None,
            "port": 3001,
            "postgres": "pgvector/pgvector:pg17",
            "redis": "redis/redis-stack:latest",
            "environment": {
                "UAR_SERVER__PORT": "3001",
                "UAR_SERVER__GRPC_PORT": "50052",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "UAR_FILE_PROCESSING__UNSTRUCTURED__API_URL": "http://unstructured:8000/general/v0/general",
                "LLM_PROTOCOL": "auto",
                "LLM_MODEL": "",
            },
            "jwt_required": {"kind": "runtime-default"},
            "external_cache_enabled": {"kind": "runtime-default"},
            "substitutions": [
                "Compose build service is replaced by the supplied candidate image",
                "test Postgres and Redis endpoints are projected to isolated local services",
                "unstructured endpoint remains a local service name and is not contacted by this settings probe",
            ],
        },
        {
            "name": "compose-prod",
            "sources": ["docker-compose.prod.yaml"],
            "config": None,
            "port": 1906,
            "postgres": None,
            "redis": "redis:7-alpine",
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_SERVER__GRPC_PORT": "50051",
                "UAR_SERVER__HOST": "0.0.0.0",
                "UAR_PERSISTENCE__PROVIDER": "surreal",
                "UAR_PERSISTENCE__DATABASE_URL": "http://surreal:8000",
                "UAR_PERSISTENCE__SURREAL_USER": "root",
                "UAR_PERSISTENCE__SURREAL_PASS": "<generated>",
                "UAR_MEMORY__ENABLED": "false",
                "UAR_MEMORY__EMBEDDING_PROVIDER": "local",
                "UAR_MEMORY__SURREAL_ENDPOINT": "ws://surreal:8000",
                "UAR_UNSTRUCTURED__API_URL": "http://unstructured:8000/general/v0/general",
                "LLM_PROTOCOL": "auto",
                "LLM_MODEL": "",
                "UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1",
            },
            "jwt_required": {
                "kind": "env-default", "source": "docker-compose.prod.yaml",
                "expression": r'UAR_SECURITY__JWT_REQUIRED:\s*"?\$\{UAR_SECURITY__JWT_REQUIRED:-(true|false)\}',
            },
            "external_cache_enabled": {
                "kind": "env-default", "source": "docker-compose.prod.yaml",
                "expression": r'UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED:\s*"?\$\{UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED:-(true|false)\}',
            },
            "substitutions": [
                "Compose image is replaced by the supplied candidate image",
                "Surreal and Redis endpoints are projected to runner-local services",
                "unstructured endpoint remains local and is not contacted by this settings probe",
            ],
        },
        {
            "name": "compose-prod-postgres",
            "sources": ["docker-compose.prod.postgres.yaml"],
            "config": None,
            "port": 1906,
            "postgres": "tribehealth/uar-postgres:pg17",
            "postgres_fallback": "pgvector/pgvector:pg17",
            "redis": "redis:7-alpine",
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_SERVER__GRPC_PORT": "50051",
                "UAR_SERVER__HOST": "0.0.0.0",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "UAR_MEMORY__ENABLED": "false",
                "UAR_MEMORY__EMBEDDING_PROVIDER": "local",
                "UAR_MEMORY__SURREAL_ENDPOINT": "ws://surreal:8000",
                "UAR_UNSTRUCTURED__API_URL": "http://unstructured:8000/general/v0/general",
                "LLM_PROTOCOL": "auto",
                "LLM_MODEL": "",
            },
            "jwt_required": {
                "kind": "env-default", "source": "docker-compose.prod.postgres.yaml",
                "expression": r'UAR_SECURITY__JWT_REQUIRED:\s*"?\$\{UAR_SECURITY__JWT_REQUIRED:-(true|false)\}',
            },
            "external_cache_enabled": {
                "kind": "literal-env", "source": "docker-compose.prod.postgres.yaml",
                "expression": r'UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED:\s*"(true|false)"',
            },
            "substitutions": [
                "Compose image is replaced by the supplied candidate image",
                "Postgres and Redis endpoints are projected to isolated local services",
                "unstructured endpoint remains local and is not contacted by this settings probe",
            ],
        },
        {
            "name": "k8s-base",
            "sources": ["k8s/base/configmap.yaml"],
            "config": None,
            "port": 1906,
            "postgres": "pgvector/pgvector:pg17",
            "redis": None,
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_SERVER__GRPC_PORT": "50051",
                "UAR_SERVER__HOST": "0.0.0.0",
                "UAR_MEMORY__ENABLED": "false",
                "UAR_MEMORY__SURREAL_ENDPOINT": "ws://surreal:8000",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "LLM_MODEL": "",
            },
            "jwt_required": {
                "kind": "literal-env", "source": "k8s/base/configmap.yaml",
                "expression": r'UAR_SECURITY__JWT_REQUIRED:\s*"(true|false)"',
            },
            "external_cache_enabled": {"kind": "runtime-default"},
            "substitutions": [
                "ConfigMap placeholder database URL is projected to isolated local Postgres",
                "Kubernetes Secret JWT and settings admin values are generated for the probe",
            ],
        },
        {
            "name": "helm",
            "sources": ["k8s/helm/uar/values.yaml", "k8s/helm/uar/templates/configmap.yaml"],
            "config": None,
            "port": 1906,
            "postgres": "pgvector/pgvector:pg17",
            "redis": None,
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_SERVER__GRPC_PORT": "50051",
                "UAR_SERVER__HOST": "0.0.0.0",
                "UAR_MEMORY__ENABLED": "false",
                "UAR_MEMORY__SURREAL_ENDPOINT": "ws://surreal:8000",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "LLM_MODEL": "",
            },
            "jwt_required": {
                "kind": "literal-env", "source": "k8s/helm/uar/values.yaml",
                "expression": r'UAR_SECURITY__JWT_REQUIRED:\s*"(true|false)"',
            },
            "external_cache_enabled": {"kind": "runtime-default"},
            "substitutions": [
                "Helm image placeholder is projected to the supplied candidate image",
                "values placeholder database URL is projected to isolated local Postgres",
                "Secret values are generated for the probe",
            ],
        },
        {
            "name": "opentofu",
            "sources": ["k8s/opentofu/configmap.tf", "k8s/opentofu/deployment.tf",
                        "k8s/opentofu/secrets.tf", "k8s/opentofu/variables.tf"],
            "config": None,
            "port": 1906,
            "postgres": "tribehealth/uar-postgres:pg17",
            "postgres_fallback": "pgvector/pgvector:pg17",
            "redis": "redis:7-alpine",
            "environment": {
                "UAR_SERVER__PORT": "1906",
                "UAR_SERVER__GRPC_PORT": "50051",
                "UAR_SERVER__HOST": "0.0.0.0",
                "UAR_PERSISTENCE__PROVIDER": "postgres",
                "UAR_PERSISTENCE__VECTOR_DIMENSION": "1536",
                "UAR_MEMORY__ENABLED": "false",
                "UAR_MEMORY__EMBEDDING_PROVIDER": "local",
                "UAR_MEMORY__SURREAL_ENDPOINT": "ws://surreal:8000",
                "UAR_FILE_PROCESSING__PROVIDER": "unstructured",
                "UAR_UNSTRUCTURED__API_URL": "http://unstructured:8000/general/v0/general",
                "LLM_PROTOCOL": "auto",
                "LLM_MODEL": "",
            },
            "jwt_required": {
                "kind": "literal-env", "source": "k8s/opentofu/configmap.tf",
                "expression": r'UAR_SECURITY__JWT_REQUIRED\s*=\s*"(true|false)"',
            },
            "external_cache_enabled": {
                "kind": "literal-env", "source": "k8s/opentofu/configmap.tf",
                "expression": r'UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED\s*=\s*"(true|false)"',
            },
            "substitutions": [
                "OpenTofu deployment image is projected to the supplied candidate image",
                "Kubernetes Postgres, Redis, and Surreal service DNS names are projected to local runner services",
                "OpenTofu has no settings admin key declaration; the probe adds a generated required local admin key",
                "unstructured endpoint remains local and is not contacted by this settings probe",
            ],
        },
        {
            "name": "example-config",
            "sources": ["example.config.yaml"],
            "config": "example.config.yaml",
            "port": 1906,
            "postgres": None,
            "redis": None,
            "environment": {
                "UAR_PERSISTENCE__PROVIDER": "surreal",
                "UAR_PERSISTENCE__DATABASE_URL": "surrealkv:///var/lib/uar/data/profile-example.db",
                "LLM_MODEL": "",
            },
            "jwt_required": {"kind": "yaml", "source": "example.config.yaml"},
            "external_cache_enabled": {"kind": "yaml", "source": "example.config.yaml"},
            "substitutions": [
                "jwt secret and settings admin key are generated for the probe",
                "relative embedded database and upload paths are projected below writable /var/lib/uar",
            ],
        },
    ]


def _start_postgres(docker, wait_for, network, profile_name, image, password, fallback_image=None):
    name = "uar-profile-pg-" + uuid.uuid4().hex[:12]
    database = "uar_" + profile_name.replace("-", "_")
    selected_image = image
    command = ("run", "--detach", "--rm", "--name", name, "--network", network,
               "--network-alias", "postgres", "--tmpfs", "/var/lib/postgresql/data",
               "--env", "POSTGRES_USER=uar", "--env", "POSTGRES_PASSWORD=" + password,
               "--env", "POSTGRES_DB=" + database)
    try:
        docker(*command, image)
    except Exception as error:
        unavailable = ("pull access denied", "manifest unknown", "not found", "no matching manifest")
        if fallback_image is None or not any(marker in str(error).lower() for marker in unavailable):
            raise
        docker("rm", "--force", name, check=False)
        selected_image = fallback_image
        docker(*command, selected_image)
    wait_for(profile_name + " Postgres health", lambda: docker(
        "exec", name, "pg_isready", "-U", "uar", "-d", database).stdout, timeout=180)
    return name, "postgres://uar:%s@postgres:5432/%s" % (password, database), selected_image


def _start_redis(docker, wait_for, network, profile_name, image):
    name = "uar-profile-redis-" + uuid.uuid4().hex[:12]
    docker("run", "--detach", "--rm", "--name", name, "--network", network,
           "--network-alias", "redis", image)
    wait_for(profile_name + " Redis health", lambda: docker(
        "exec", name, "redis-cli", "ping").stdout, timeout=90)
    return name


def _status_probe(docker, name, port, path):
    probe = (
        "import urllib.error, urllib.request\n"
        "try:\n"
        "    response = urllib.request.urlopen('http://127.0.0.1:%d%s', timeout=5)\n"
        "    print(response.status)\n"
        "except urllib.error.HTTPError as error:\n"
        "    print(error.code)\n"
    ) % (port, path)
    return docker("exec", name, "python3", "-c", probe).stdout.strip()


def _health_probe(docker, name, port):
    return docker("exec", name, "python3", "-c", (
        "import urllib.request; "
        "response = urllib.request.urlopen('http://127.0.0.1:%d/health', timeout=5); "
        "assert response.status == 200" % port)).stdout


def _run_profile(image, profile, evidence, *, docker, require, wait_for, network, surreal_password):
    profile_dir = evidence / "profiles" / profile["name"]
    profile_dir.mkdir(parents=True)
    generated_jwt = secrets.token_urlsafe(32)
    generated_admin = secrets.token_urlsafe(32)
    generated_postgres = secrets.token_urlsafe(24)
    environment = dict(profile["environment"])
    jwt_required = _resolved_jwt_required(profile)
    external_cache_enabled = _resolved_external_cache(profile)
    environment.update({
        "UAR_SECURITY__JWT_SECRET": generated_jwt,
        "UAR_SECURITY__SETTINGS_ADMIN_KEY": generated_admin,
    })
    if profile["jwt_required"]["kind"] != "yaml":
        environment["UAR_SECURITY__JWT_REQUIRED"] = str(jwt_required).lower()
    if profile["external_cache_enabled"]["kind"] != "yaml":
        environment["UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED"] = str(external_cache_enabled).lower()
    if environment.get("UAR_PERSISTENCE__SURREAL_PASS") == "<generated>":
        environment["UAR_PERSISTENCE__SURREAL_PASS"] = surreal_password
    containers = []
    try:
        selected_postgres_image = None
        if profile["postgres"]:
            postgres_name, postgres_url, selected_postgres_image = _start_postgres(
                docker, wait_for, network, profile["name"], profile["postgres"], generated_postgres,
                profile.get("postgres_fallback"))
            containers.append(postgres_name)
            environment["UAR_PERSISTENCE__DATABASE_URL"] = postgres_url
            environment["DATABASE_URL"] = postgres_url
        if profile["redis"]:
            containers.append(_start_redis(
                docker, wait_for, network, profile["name"], profile["redis"]))
            environment["UAR_PERSISTENCE__REDIS_URL"] = "redis://redis:6379"
            environment["REDIS_URL"] = "redis://redis:6379"

        provenance_sources = _provenance_sources(profile)
        source_hashes = {source: _source_hash(source) for source in provenance_sources}
        projection = {
            "profile": profile["name"],
            "candidate_image": image,
            "source_hashes": source_hashes,
            "source_files": provenance_sources,
            "projection": "candidate image direct runtime; no Compose, Kubernetes, OpenTofu, or source build execution",
            "substitutions": profile["substitutions"],
            "jwt_required": jwt_required,
            "jwt_required_derivation": profile["jwt_required"],
            "external_cache_enabled": external_cache_enabled,
            "external_cache_enabled_derivation": profile["external_cache_enabled"],
            "environment": _masked_environment(environment),
            "postgres_image_requested": profile["postgres"],
            "postgres_image_selected": selected_postgres_image,
            "postgres_fallback": profile.get("postgres_fallback"),
            "redis_image": profile["redis"],
            "anonymous_endpoints": ["/api/uar/settings/effective", "/api/agents"],
        }
        if profile["config"]:
            staged = profile_dir / "config.yaml"
            _materialize_config(profile["config"], staged,
                                "/var/lib/uar/uploads/" + profile["name"])
            projection["staged_config"] = str(staged)
        _write_json(profile_dir / "projection.json", projection)

        name = "uar-profile-" + uuid.uuid4().hex[:12]
        command = ["run", "--detach", "--rm", "--name", name, "--network", network]
        if profile["config"]:
            command.extend(["--volume", str(profile_dir / "config.yaml") + ":/opt/uar/profile.yaml:ro"])
        for key, value in sorted(environment.items()):
            command.extend(["--env", key + "=" + value])
        command.append(image)
        command.append("universal-agent-runtime")
        if profile["config"]:
            command.extend(["--config", "/opt/uar/profile.yaml"])
        docker(*command)
        containers.append(name)
        wait_for(profile["name"] + " runtime health",
                 lambda: _health_probe(docker, name, profile["port"]), timeout=180)
        effective_status = _status_probe(docker, name, profile["port"], "/api/uar/settings/effective")
        require(effective_status == "401",
                profile["name"] + " unauthenticated effective-settings status was " + effective_status)
        agents_status = _status_probe(docker, name, profile["port"], "/api/agents")
        require(agents_status == "401",
                profile["name"] + " unauthenticated agent-list status was " + agents_status)
        projection["health"] = "200"
        projection["anonymous_statuses"] = {"effective_settings": 401, "agents": 401}
        _write_json(profile_dir / "projection.json", projection)
        return {
            "name": profile["name"],
            "source_hashes": source_hashes,
            "anonymous_statuses": {"effective_settings": 401, "agents": 401},
            "projection_receipt": str(profile_dir / "projection.json"),
        }
    finally:
        for container in reversed(containers):
            docker("rm", "--force", container, check=False)


def run_shipped_profile_checks(image, evidence, *, docker, require, wait_for, network, surreal_password):
    """Start every shipped runtime projection and require both anonymous 401 responses."""
    results = []
    for profile in _profile_inventory():
        results.append(_run_profile(
            image, profile, evidence, docker=docker, require=require, wait_for=wait_for,
            network=network, surreal_password=surreal_password))
    return {
        "profiles": results,
        "profile_count": len(results),
        "acceptance": "each installed candidate projection returned HTTP 401 for unauthenticated effective-settings and agent-list requests",
    }
