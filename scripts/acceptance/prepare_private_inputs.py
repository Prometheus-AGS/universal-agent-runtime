"""Prepare only fresh owned fixture credentials and private input artifacts.

No I/O on import. Later named approval must explicitly authorize credential
creation and temporary private copies of explicitly selected existing key values.
No shell, source/eval, process, service, database, network or model operation.
"""
from __future__ import annotations

import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import secrets
import sys
import time
from urllib.parse import quote


EXISTING_NAMES = ("COHERE_API_KEY", "LITER_LLM_MASTER_KEY")
EFFECTS = ["create-exclusive-private-scope-and-state-parents", "generate-owned-hs256-jwt-and-admin-secrets",
           "mint-two-owned-fixture-user-tokens", "generate-shared-owned-ingress-raw-key",
           "generate-owned-postgres-and-surreal-credential-inputs", "privately-copy-two-selected-existing-provider-keys"]
SOURCE_NAMES = ("claims", "verifier", "presentation_owner", "user_settings", "collaboration_scope", "session_storage")
SOURCE_PATHS = {"claims": "src/uar/security/claims.rs", "verifier": "src/uar/security/verifier/mod.rs",
                "presentation_owner": "src/uar/runtime/actor/messages.rs", "user_settings": "src/uar/api/user_settings.rs",
                "collaboration_scope": "src/uar/api/collaboration.rs", "session_storage": "src/uar/persistence/mod.rs"}


class Refusal(RuntimeError):
    pass


def need(condition, code):
    if not condition:
        raise Refusal(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def private_json(path):
    path = Path(path)
    need(path.is_absolute() and path.is_file() and not path.is_symlink() and path.stat().st_uid == os.getuid()
         and path.stat().st_mode & 0o077 == 0 and path.stat().st_size <= 1048576, "private_input_unavailable")
    raw = path.read_bytes()
    return json.loads(raw), raw


def bound_reference(ref):
    need(isinstance(ref, dict) and set(ref) == {"path", "sha256"}, "frozen_reference_invalid")
    path = Path(ref["path"])
    need(path.is_absolute() and path.is_file() and not path.is_symlink()
         and path.stat().st_size <= 1048576, "frozen_source_unavailable")
    need(sha(path.read_bytes()) == ref["sha256"], "frozen_source_changed")


def literal_key(name, source):
    existing = os.environ.get(name)
    if existing is not None:
        need(0 < len(existing) <= 16384 and not any(char in existing for char in "\r\n\0"), "selected_environment_key_invalid")
        return existing, {"name": name, "source": "existing-environment"}
    expected = Path.home() / (".bash_profile" if name in ("COHERE_API_KEY", "OPENAI_API_KEY") else ".prometheus/kbd/secrets.env")
    path = Path(source)
    need(path == expected and path.is_file() and not path.is_symlink() and path.stat().st_size <= 1048576,
         "selected_literal_source_unavailable")
    selected = []
    prefix = re.compile(r"^\s*(?:export\s+)?" + re.escape(name) + r"\s*=")
    literal = re.compile(r"^\s*(?:export\s+)?" + re.escape(name)
                         + r"\s*=\s*('([^'\r\n]*)'|\"([^\"\\$`\r\n]*)\"|([A-Za-z0-9_./:+,=@%\-]+))\s*(?:#.*)?$")
    with path.open(encoding="utf-8") as stream:
        for line in stream:
            if not prefix.match(line):
                continue
            match = literal.fullmatch(line.rstrip("\r\n"))
            need(match is not None, "selected_key_assignment_is_not_literal")
            selected.append(next(value for value in match.groups()[1:] if value is not None))
    need(len(selected) == 1 and 0 < len(selected[0]) <= 16384 and "\0" not in selected[0],
         "selected_literal_assignment_missing_or_ambiguous")
    return selected[0], {"name": name, "source": "selected-literal-assignment", "path": str(path)}


def utf8len(value):
    return len(value.encode("utf-8"))


def identity(issuer, tenant, subject, workspace):
    # claims::CredentialPrincipal::storage_key / session_owner_key.
    credential = f"v1:k:issuer:i:{utf8len(issuer)}:{issuer}:t:{utf8len(tenant)}:{tenant}:s:{utf8len(subject)}:{subject}"
    # ActorOwner::presentation_owner_key / user_settings::principal_storage_key.
    presentation = f"v1:t:{utf8len(tenant)}:{tenant}:s:{utf8len(subject)}:{subject}"
    return {"subject": subject, "tenant_id": tenant, "issuer": issuer, "workspace_id": workspace,
            "credential_principal_key": credential, "session_owner_key": credential,
            "history_owner_key": presentation, "collaboration_owner_key": presentation,
            "workspace_header": {"x-uar-workspace-id": workspace}}


def jwt(secret, claims):
    def encoded(raw):
        return base64.urlsafe_b64encode(raw).rstrip(b"=")
    unsigned = encoded(canonical({"alg": "HS256", "typ": "JWT"})) + b"." + encoded(canonical(claims))
    signature = hmac.new(secret.encode(), unsigned, hashlib.sha256).digest()
    return (unsigned + b"." + encoded(signature)).decode()


def write_private(path, value):
    raw = canonical(value) + b"\n"
    need(len(raw) <= 1048576, "private_artifact_exceeds_bound")
    with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())


def prepare(config, config_digest):
    need(config.get("schema_version") == 1 and config.get("executable") is True, "private_preparation_not_executable")
    epoch = config["epoch_id"]
    need(isinstance(epoch, str) and re.fullmatch(r"[A-Za-z0-9._:-]{1,128}", epoch), "epoch_identity_invalid")
    suffix = hashlib.sha256(epoch.encode()).hexdigest()[:20]
    parent = Path(config["scope_parent"])
    need(parent.is_absolute() and parent.is_dir() and not parent.is_symlink() and parent.stat().st_uid == os.getuid()
         and parent.stat().st_mode & 0o077 == 0,
         "existing_private_parent_required")
    root = parent / ("uar_fixture_inputs_" + suffix)
    need(not root.exists() and not root.is_symlink(), "private_scope_already_exists_no_reset")
    sources = config["source_contracts"]
    need(set(sources) == set(SOURCE_NAMES), "authentication_source_contracts_incomplete")
    repo = Path(__file__).resolve().parents[2]
    for name, reference in sources.items():
        need(Path(reference["path"]) == repo / SOURCE_PATHS[name], "authentication_source_path_differs")
        bound_reference(reference)
    approval_ref = config["preparation_approval"]
    selected_services = config.get("selected_embedding_services", ["cohere"])
    need(selected_services in (["cohere"], ["cohere", "openai_embeddings"]), "explicit_embedding_service_selection_invalid")
    existing_names = (*EXISTING_NAMES, *(("OPENAI_API_KEY",) if "openai_embeddings" in selected_services else ()))
    effects = [*EFFECTS, *(["privately-copy-explicitly-selected-openai-embedding-key"] if "openai_embeddings" in selected_services else [])]
    bound_reference(approval_ref)
    approval, _raw = private_json(approval_ref["path"])
    projection = {key: value for key, value in config.items() if key != "preparation_approval"}
    need(approval == {"operation": "prepare-uar-acceptance-private-inputs", "epoch_id": epoch,
                      "scope_root": str(root), "preparation_projection_sha256": sha(canonical(projection)),
                      "effects": effects, "operator_approved": True}, "credential_generation_approval_scope_differs")
    validity = config["validity"]
    start, end = validity["not_before_unix"], validity["expires_unix"]
    need(type(start) is int and type(end) is int and 0 < end - start <= 86400
         and start <= int(time.time()) < end, "fixture_token_validity_not_current_or_bounded")
    ports = config["ports"]
    need(isinstance(ports, dict) and ports and all(re.fullmatch(r"[a-z][a-z0-9_]*", name) for name in ports)
         and all(type(value) is int and 1024 <= value <= 65535 for value in ports.values())
         and len(set(ports.values())) == len(ports), "explicit_distinct_nonprivileged_ports_required")
    need("postgres" in ports and "surreal" in ports, "owned_datastore_ports_missing")
    need(set(config["existing_key_sources"]) == set(existing_names), "selected_existing_key_sources_incomplete")
    environment, existing_receipts = {}, []
    for name in existing_names:
        environment[name], receipt = literal_key(name, config["existing_key_sources"][name])
        existing_receipts.append(receipt)
    composition = Path(config["composition_output_dir"])
    need(composition.is_absolute() and not composition.exists() and not composition.is_symlink(),
         "future_composition_output_must_be_new_absolute_path")
    for reference in config["pending_operator_inputs"]:
        need(isinstance(reference, str) and bool(reference), "pending_operator_input_name_invalid")
    jwt_secret, admin_key = secrets.token_urlsafe(48), secrets.token_urlsafe(48)
    ingress_key = secrets.token_urlsafe(48)
    issuer, tenant, audience = "urn:uar:acceptance:" + suffix, "uar-fixture-tenant-" + suffix, "uar-acceptance-" + suffix
    workspace = "uar-fixture-workspace-" + suffix
    primary = identity(issuer, tenant, "uar-fixture-primary-" + suffix, workspace)
    foreign = identity(issuer, tenant, "uar-fixture-foreign-" + suffix, workspace)
    users = []
    for who in (primary, foreign):
        claims = {"sub": who["subject"], "name": None, "roles": ["user"], "tenant_id": tenant,
                  "iss": issuer, "aud": audience, "iat": start, "nbf": start, "exp": end}
        users.append((claims, "Bearer " + jwt(jwt_secret, claims)))
    username = "uar_fixture_" + suffix
    pg_password, surreal_password = secrets.token_urlsafe(36), secrets.token_urlsafe(36)
    database = "uar_fixture_" + suffix
    connection = f"postgresql://{quote(username, safe='')}:{quote(pg_password, safe='')}@127.0.0.1:{ports['postgres']}/{database}"
    environment.update(UAR_FIXTURE_JWT_SECRET=jwt_secret, UAR_FIXTURE_ADMIN_KEY=admin_key,
        UAR_FIXTURE_PRIMARY_AUTHORIZATION=users[0][1], UAR_FIXTURE_FOREIGN_AUTHORIZATION=users[1][1],
        UAR_FIXTURE_INGRESS_RAW_KEY=ingress_key, UAR_FIXTURE_PG_USERNAME=username,
        UAR_FIXTURE_PG_PASSWORD=pg_password, UAR_FIXTURE_PG_DATABASE=database, UAR_FIXTURE_PG_CONNECTION=connection,
        UAR_FIXTURE_SURREAL_USERNAME=username, UAR_FIXTURE_SURREAL_PASSWORD=surreal_password,
        UAR_FIXTURE_SURREAL_URL=f"http://127.0.0.1:{ports['surreal']}")
    root.mkdir(mode=0o700)
    for name, key in (("postgres", "UAR_FIXTURE_POSTGRES_STATE_PARENT"), ("surreal", "UAR_FIXTURE_SURREAL_STATE_PARENT"),
                      ("packet", "UAR_FIXTURE_PACKET_STATE_PARENT")):
        directory = root / name
        directory.mkdir(mode=0o700)
        environment[key] = str(directory)
    for prefix, backend in (("POSTGRES", "postgres_pgvector"), ("SURREAL", "surreal_disk")):
        for role in ("runtime", "startup"):
            environment[f"UAR_FIXTURE_{prefix}_{role.upper()}_CONFIG"] = str(composition / (backend + "-" + role + ".json"))
    security = {"deployment_profile": "remote", "jwt_required": True, "jwt_algorithm": "HS256",
                "jwt_secret": "@env:UAR_FIXTURE_JWT_SECRET@", "jwks_url": None, "jwt_issuer": issuer,
                "jwt_audience": audience, "jwt_validate_nbf": True, "settings_mutation_auth_required": True,
                "settings_admin_key": "@env:UAR_FIXTURE_ADMIN_KEY@",
                "workspace_authorities": [{"issuer": issuer, "subject": user["subject"], "tenant_id": tenant,
                                            "workspace_id": workspace} for user in (primary, foreign)]}
    inputs = {"schema_version": 1, "status": "PRIVATE_INPUTS_PREPARED_NOT_ADOPTED", "epoch_id": epoch,
              "selected_embedding_services": selected_services,
              "scope_root": str(root), "ports": ports, "security_patch": security,
              "primary": primary, "foreign": foreign, "claims": {"primary": users[0][0], "foreign": users[1][0]},
              "orchestrator": {"expected_principal_key": primary["credential_principal_key"],
                               "expected_history_owner_key": primary["history_owner_key"], "expected_subject": primary["subject"]},
              "primary_headers_env": {"Authorization": "UAR_FIXTURE_PRIMARY_AUTHORIZATION", "x-uar-admin-key": "UAR_FIXTURE_ADMIN_KEY"},
              "foreign_authorization_env": "UAR_FIXTURE_FOREIGN_AUTHORIZATION",
              "budget_ingress": {"ingress_authorization_env": "UAR_FIXTURE_INGRESS_RAW_KEY", "ingress_authorization_prefix": "Bearer "},
              "datastores": {"postgres": {"username_env": "UAR_FIXTURE_PG_USERNAME", "password_env": "UAR_FIXTURE_PG_PASSWORD",
                              "database_env": "UAR_FIXTURE_PG_DATABASE", "connection_env": "UAR_FIXTURE_PG_CONNECTION"},
                             "surreal": {"username_env": "UAR_FIXTURE_SURREAL_USERNAME", "password_env": "UAR_FIXTURE_SURREAL_PASSWORD",
                                         "url_env": "UAR_FIXTURE_SURREAL_URL"}},
              "environment_names": sorted(environment), "pending_operator_inputs": config["pending_operator_inputs"],
              "limitations": ["No datastore role/database or service has been created", "No tariff/charge/adoption/signature/seed inputs were generated",
                              "Existing high-water consumption and unknown reservations remain external reconciliation obligations",
                              "Predicted owner keys are source-derived, not runtime authentication receipts"]}
    write_private(root / "env.json", environment)
    write_private(root / "inputs.json", inputs)
    receipt = {"schema_version": "uar.private-fixture-inputs.v1", "status": "prepared_not_adopted", "epoch_id": epoch,
               "preparation_config_sha256": config_digest, "scope_root": str(root),
               "env_path": str(root / "env.json"), "inputs_path": str(root / "inputs.json"),
               "environment_names": sorted(environment), "selected_existing_sources": existing_receipts,
               "inputs_sha256": sha(canonical(inputs) + b"\n"), "effects": effects,
               "selected_embedding_services": selected_services,
               "source_contracts": sources, "pending_operator_inputs": config["pending_operator_inputs"]}
    write_private(root / "receipt.json", receipt)
    directory_fd = os.open(root, os.O_RDONLY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)
    return receipt


def main():
    need(len(sys.argv) == 2, "expected_one_private_preparation_config")
    config, raw = private_json(sys.argv[1])
    receipt = prepare(config, sha(raw))
    print(json.dumps(receipt, sort_keys=True), flush=True)


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"error": "private_fixture_preparation_unconfirmed", "code": str(error) if isinstance(error, Refusal)
                          else type(error).__name__}), flush=True)
        sys.exit(2)
