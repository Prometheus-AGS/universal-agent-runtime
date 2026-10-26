"""Materialize one explicitly approved, inert supporting fixture; no I/O on import."""
from __future__ import annotations

import base64
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import stat
import sys
import time
from urllib.parse import quote


SCENARIOS = ["LIAB-01", "LIAB-02", "ERROR-01", "ERROR-02", "COUNT-01",
             "HISTORY-01", "KB-01", "CAS-01", "BTHREE-01", "IDENTITY-01"]
REQUIRED_SOURCE = ["scripts/post_lifecycle/main_boundary.py", "scripts/post_lifecycle/controlled_peer.py",
                   "tests/post_lifecycle_main.rs", "tests/support/post_lifecycle_count.rs", "tests/support/post_lifecycle_store.rs",
                   "scripts/acceptance/prepare_supporting_inputs.py"]
BASELINE = {"PATH", "LANG", "LC_ALL", "CARGO_HOME", "RUSTUP_HOME"}
PRIVATE_KEYS = {"UAR_FIXTURE_JWT_SECRET", "UAR_FIXTURE_ADMIN_KEY",
                "UAR_FIXTURE_SURREAL_USERNAME", "UAR_FIXTURE_SURREAL_PASSWORD"}
EFFECTS = ["read-selected-existing-private-fixture-inputs", "mint-three-supporting-only-hs256-tokens",
           "create-exclusive-private-versioned-supporting-scope", "materialize-inert-supporting-metadata"]


class Refusal(RuntimeError):
    pass


def need(condition, code):
    if not condition:
        raise Refusal(code)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def digest(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def hex_digest(value):
    need(isinstance(value, str) and re.fullmatch(r"(?:sha256:)?[0-9a-f]{64}", value), "invalid_sha256")
    return value.removeprefix("sha256:")


def read_regular(path, private=False):
    path = Path(path)
    need(path.is_absolute() and path == path.resolve(), "input_path_must_be_absolute_without_symlinks")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        need(stat.S_ISREG(metadata.st_mode) and metadata.st_size <= 134217728, "regular_bounded_input_required")
        if private:
            need(metadata.st_uid == os.getuid() and metadata.st_mode & 0o077 == 0, "private_input_permissions")
        return stream.read(134217729)


def bound(ref, private=False):
    need(isinstance(ref, dict) and set(ref) == {"path", "sha256"}, "invalid_frozen_reference")
    expected = hex_digest(ref["sha256"])
    raw = read_regular(ref["path"], private)
    need(digest(raw)[7:] == expected, "frozen_input_changed")
    return raw


def file_digest(path):
    path = Path(path)
    need(path.is_absolute() and path == path.resolve(), "bound_path_must_be_absolute_without_symlinks")
    descriptor = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(descriptor, "rb") as stream:
        metadata = os.fstat(stream.fileno())
        need(stat.S_ISREG(metadata.st_mode) and metadata.st_size <= 17179869184, "bound_regular_file_required")
        result = hashlib.sha256()
        while chunk := stream.read(1048576):
            result.update(chunk)
        return result.hexdigest()


def write_new(path, raw):
    with os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), "wb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())
    return {"path": str(path), "sha256": digest(raw)}


def write_json(path, value):
    return write_new(path, canonical(value) + b"\n")


def relocate(value, old_root, root, version):
    if isinstance(value, str):
        return value.replace(old_root, str(root)).replace("uar-user-outcomes-007", "uar-user-outcomes-" + version).replace(
            "uar_user_outcomes_007", "uar_user_outcomes_" + version).replace("uar-supporting-owner-007", "uar-supporting-owner-" + version).replace(
            "uar-supporting-other-007", "uar-supporting-other-" + version).replace("uar-supporting-host-007", "uar-supporting-host-" + version).replace(
            "uar_post_lifecycle_user_outcomes_007", "uar_post_lifecycle_user_outcomes_" + version)
    if isinstance(value, list):
        return [relocate(item, old_root, root, version) for item in value]
    if isinstance(value, dict):
        return {key: relocate(item, old_root, root, version) for key, item in value.items()}
    return value


def token(secret, claims):
    def encoded(raw):
        return base64.urlsafe_b64encode(raw).rstrip(b"=")
    unsigned = encoded(canonical({"alg": "HS256", "typ": "JWT"})) + b"." + encoded(canonical(claims))
    return "Bearer " + (unsigned + b"." + encoded(hmac.new(secret.encode(), unsigned, hashlib.sha256).digest())).decode()


def source_projection(inventory, repo):
    regular, unresolved, seen = [], [], set()
    for row in inventory["files"]:
        need(isinstance(row, dict) and isinstance(row.get("path"), str), "invalid_source_row")
        path = Path(row["path"])
        need(path.is_absolute() and path.is_relative_to(repo) and str(path) not in seen, "source_path_invalid_or_duplicate")
        seen.add(str(path))
        kind = row.get("kind")
        need(kind in {"regular", "symlink", "missing"}, "unsupported_source_row_kind")
        if kind != "regular":
            unresolved.append(row)
            continue
        expected = hex_digest(row.get("sha256"))
        need(file_digest(path) == expected, "regular_source_changed")
        regular.append({"path": str(path), "sha256": "sha256:" + expected})
    regular.sort(key=lambda row: row["path"])
    need(digest(canonical(regular)) == "sha256:" + hex_digest(inventory["source_identity"]), "source_projection_identity_differs")
    required = {str(repo / name) for name in REQUIRED_SOURCE}
    need(required <= {row["path"] for row in regular}, "required_executable_source_missing_or_nonregular")
    return regular, unresolved


def prepare(config, config_digest):
    need(config.get("schema_version") == "uar.supporting-input-preparation.v1" and config.get("executable") is True,
         "explicit_executable_preparation_required")
    repo = Path(__file__).resolve().parents[2]
    version = config["output_version"]
    need(isinstance(version, str) and re.fullmatch(r"[0-9]{3}", version) and int(version) > 7, "fresh_version_after_partial007_required")
    root = Path(config["output_root"])
    need(root == repo / "target" / ("uar-user-outcomes-" + version) / "supporting", "versioned_output_path_differs")
    parent = root.parent
    need(parent.is_dir() and parent == parent.resolve() and parent.stat().st_uid == os.getuid()
         and parent.stat().st_mode & 0o077 == 0, "existing_owned_private_parent_required")
    need(not root.exists() and not root.is_symlink(), "output_exists_no_overwrite_or_retry")
    projection = {key: value for key, value in config.items() if key != "preparation_approval"}
    approval = json.loads(bound(config["preparation_approval"], private=True))
    need(approval == {"operation": "prepare-uar-supporting-inputs", "operator_approved": True,
                      "preparation_projection_sha256": digest(canonical(projection)), "output_root": str(root),
                      "effects": EFFECTS}, "preparation_approval_scope_differs")
    plan = json.loads(bound(config["supporting_plan"]))
    packet = json.loads(bound(config["preparation_packet"]))
    inventory = json.loads(bound(config["source_inventory"]))
    need(inventory["root"] == str(repo), "source_checkout_differs")
    regular, unresolved = source_projection(inventory, repo)
    # Nullable symlink/deletion rows never enter boundInputs or digest parsing.
    python_ref = config["python"]
    need(set(python_ref) == {"path", "sha256"} and file_digest(python_ref["path"]) == hex_digest(python_ref["sha256"]),
         "selected_python_binding_changed")
    need(os.stat(python_ref["path"]).st_mode & 0o111, "selected_python_not_executable")
    installed = config["installed_runtime_inputs"]
    need(isinstance(installed, list) and installed, "frozen_python_runtime_closure_required")
    for ref in installed:
        bound(ref)
    baseline = config["toolchain_environment"]
    need(isinstance(baseline, dict) and set(baseline) <= BASELINE
         and {"PATH", "CARGO_HOME", "RUSTUP_HOME"} <= set(baseline), "explicit_toolchain_baseline_required")
    need(all(isinstance(value, str) and value and not any(char in value for char in "\r\n\0")
             for value in baseline.values()), "invalid_toolchain_baseline")
    drafts = {}
    for name in ("manifest", "config", "store"):
        ref = config["templates"][name]
        need(ref["path"] == plan["inert_drafts"][name], "template_not_selected_by_recipe")
        drafts[name] = relocate(json.loads(bound(ref)), plan["paths"]["root"], root, version)
    manifest, runtime, compose = drafts["manifest"], drafts["config"], drafts["store"]
    need(manifest["scenarioIds"] == SCENARIOS, "scenario_recipe_differs")
    need(all(manifest.get(name) is False for name in ("executable", "implementationComplete", "readyToRun", "cleanupAuthorized")),
         "template_must_be_inert")
    need(manifest["verificationApprovalRef"] is None and manifest["storeFixtureApprovalRef"] is None,
         "verification_activation_not_preparation")
    document_bytes = []
    need(len(plan["document_materialization"]) == 5 and set(config["documents"]) == set("abcde"), "five_documents_required")
    for letter, recipe in zip("abcde", plan["document_materialization"]):
        ref = config["documents"][letter]
        need(ref["path"] == recipe["source"], "document_recipe_source_differs")
        raw = bound(ref)
        replacement = recipe.get("exact_marker_replacement")
        if replacement:
            before = replacement["from"].encode()
            need(before in raw, "document_marker_missing")
            after = relocate(replacement["to"], plan["paths"]["root"], root, version).encode()
            raw = raw.replace(before, after)
        else:
            need(b"UAR_HOLD:kb-race:" in raw, "document_hold_marker_missing")
        document_bytes.append((letter, raw))
    validity = config["jwt_validity"]
    need(validity == packet["jwt_validity"], "validity_not_selected_by_preparation_packet")
    start, end = validity["not_before_unix"], validity["expires_unix"]
    need(type(start) is int and type(end) is int and 0 < end - start <= 86400
         and start <= int(time.time()) < end, "approved_validity_expired_or_unbounded")
    # Only these four values are selected from the existing private env object.
    env_path = Path(config["private_environment_path"])
    inputs_ref, private_receipt_ref = config["private_inputs"], config["private_receipt"]
    need(env_path.name == "env.json" and Path(inputs_ref["path"]).parent == env_path.parent
         and Path(private_receipt_ref["path"]).parent == env_path.parent, "private_input_scope_differs")
    inputs = json.loads(bound(inputs_ref, private=True))
    private_receipt = json.loads(bound(private_receipt_ref, private=True))
    need(private_receipt["env_path"] == str(env_path) and private_receipt["inputs_path"] == inputs_ref["path"]
         and private_receipt["inputs_sha256"] == "sha256:" + hex_digest(inputs_ref["sha256"]), "private_receipt_scope_differs")
    need(inputs["epoch_id"] == config["epoch_id"] == "uar-user-outcomes-original-20261024", "original_epoch_differs")
    raw_environment = json.loads(read_regular(env_path, private=True))
    selected = {name: raw_environment[name] for name in PRIVATE_KEYS}
    del raw_environment
    need(all(isinstance(value, str) and value and not any(char in value for char in "\r\n\0")
             for value in selected.values()), "selected_fixture_credential_invalid")
    base = inputs["claims"]["primary"]
    need(all(isinstance(base.get(name), str) and base[name] for name in ("iss", "aud", "tenant_id")), "primary_claims_invalid")
    jwt_recipe = relocate(plan["supporting_jwt_recipe"], plan["paths"]["root"], root, version)
    claims, environment = {}, dict(baseline)
    aliases = {"owner": "UAR_BOUNDARY_OWNER_AUTHORIZATION", "other": "UAR_BOUNDARY_OTHER_OWNER_AUTHORIZATION",
               "host": "UAR_BOUNDARY_EXISTING_HOST_AUTHORIZATION"}
    for role in ("owner", "other", "host"):
        claims[role] = {name: base[name] for name in ("iss", "aud", "tenant_id")}
        claims[role].update(sub=jwt_recipe["subjects"][role], roles=jwt_recipe["roles"][role], name=None,
                            iat=start, nbf=start, exp=end)
    security = runtime["security"]
    security.update(jwt_secret=selected["UAR_FIXTURE_JWT_SECRET"], jwt_issuer=base["iss"], jwt_audience=base["aud"],
                    settings_admin_key=selected["UAR_FIXTURE_ADMIN_KEY"])
    for authority in security["workspace_authorities"] + security["trusted_host_principals"]:
        authority.update(issuer=base["iss"], tenant_id=base["tenant_id"])
    username, password = selected["UAR_FIXTURE_SURREAL_USERNAME"], selected["UAR_FIXTURE_SURREAL_PASSWORD"]
    runtime["persistence"].update(surreal_user=username, surreal_pass=password)
    compose["services"]["store"]["command"][2:4] = ["--user=" + username, "--pass=" + password]
    environment.update(UAR_BOUNDARY_EXISTING_WORKSPACE=jwt_recipe["workspace_id"],
        UAR_BOUNDARY_SETTINGS_ADMIN_KEY=selected["UAR_FIXTURE_ADMIN_KEY"], UAR_BOUNDARY_STORE_USER=username,
        UAR_BOUNDARY_STORE_PASSWORD=password, UAR_API_KEY_TEST_SURREAL_USER=username, UAR_API_KEY_TEST_SURREAL_PASS=password,
        UAR_API_KEY_TEST_SURREAL_URL="ws://127.0.0.1:49312", UAR_REMOTE_SURREAL_DURABILITY_ATTESTED="1",
        RUST_LOG="info", UAR_LOCAL_ONLY="true", HOME=str(root / "home"), TMPDIR=str(root / "tmp"),
        CARGO_TARGET_DIR="target/uar-post-lifecycle-main")
    owner = claims["owner"]["sub"]
    issuer, tenant = base["iss"], base["tenant_id"]
    owner_key = f"v1:k:issuer:i:{len(issuer.encode())}:{issuer}:t:{len(tenant.encode())}:{tenant}:s:{len(owner.encode())}:{owner}"
    variables = manifest["fixtureVariables"]
    variables.update(ownerStorageKey=owner_key, ownerStorageKeyUrlEncoded=quote(owner_key, safe=""),
                     historyOwnerKey=f"v1:t:{len(tenant.encode())}:{tenant}:s:{len(owner.encode())}:{owner}")
    manifest["sourceInventoryDigest"] = hex_digest(inventory["source_identity"])
    manifest["contractDigest"] = hex_digest(config["contract_digest"])
    manifest["processes"]["peer"]["argv"][0] = python_ref["path"]
    peer = next(row for row in regular if row["path"] == str(repo / REQUIRED_SOURCE[1]))
    manifest["processes"]["peer"]["sha256"] = hex_digest(peer["sha256"])
    manifest["proposalMetadata"] = {"gate": "CLOSED", "sourceInventory": config["source_inventory"],
        "globalEpoch": config["epoch_id"], "preparationPacket": config["preparation_packet"],
        "status": "MATERIALIZED_INERT_PENDING_AUTHORITY_AND_EXACT_FINAL_V"}
    # Exclusive mkdir is the one-writer claim. A partial scope is retained on failure.
    root.mkdir(mode=0o700)
    for name in ("documents", "store", "home", "tmp", "ingest", "models", "reviews", "evidence"):
        (root / name).mkdir(mode=0o700)
    for role, alias in aliases.items():
        environment[alias] = token(selected["UAR_FIXTURE_JWT_SECRET"], claims[role])
    produced = []
    for letter, raw in document_bytes:
        ref = write_new(root / "documents" / (letter + ".txt"), raw)
        variables["document_" + letter + "_path"] = ref["path"]
        variables["document_" + letter + "_sha256"] = hex_digest(ref["sha256"])
        produced.append(ref)
    produced.append(write_json(root / "local-main.yaml", runtime))
    produced.append(write_json(root / "local-store.yaml", compose))
    # Do not hash or publish token strings or individual key values, or bind env.json.
    write_json(root / "supporting.env.json", environment)
    bindings = regular + [config["source_inventory"], config["private_inputs"], config["private_receipt"],
                           config["supporting_plan"], config["preparation_packet"], python_ref] + installed + produced
    unique = {}
    for ref in bindings:
        normalized = {"path": ref["path"], "sha256": hex_digest(ref["sha256"])}
        need(normalized["path"] not in unique or unique[normalized["path"]] == normalized, "conflicting_bound_input")
        unique[normalized["path"]] = normalized
    manifest["boundInputs"] = sorted(unique.values(), key=lambda row: row["path"])
    manifest_ref = write_json(root / "boundary-manifest.json", manifest)
    receipt = {"schema_version": "uar.supporting-inputs.v1", "status": "MATERIALIZED_NOT_ACTIVATED",
        "verification_gate": "CLOSED", "preparation_config_sha256": config_digest,
        "output_root": str(root), "source_inventory": config["source_inventory"],
        "source_identity": inventory["source_identity"], "source_regular_count": len(regular),
        "nonregular_source_entries": unresolved, "required_source_paths": REQUIRED_SOURCE,
        "supporting_plan": config["supporting_plan"], "preparation_packet": config["preparation_packet"],
        "templates": config["templates"], "document_sources": config["documents"],
        "preparation_approval": config["preparation_approval"], "private_inputs": inputs_ref,
        "private_receipt": private_receipt_ref, "private_environment_path": str(env_path),
        "python": python_ref, "installed_runtime_inputs": installed, "epoch_id": config["epoch_id"],
        "contract_digest": config["contract_digest"], "jwt_validity": validity, "claims": claims,
        "supporting_tokens_minted": 3, "product_tokens_modified": 0, "provider_keys_copied": 0,
        "environment_path": str(root / "supporting.env.json"), "environment_names": sorted(environment),
        "artifacts": produced + [manifest_ref], "effects": EFFECTS,
        "unverified": ["runtime authentication", "installed runtime closure compatibility", "port availability",
                       "service startup", "final authority adoption", "whole-phase acceptance"],
        "next_boundary": "All metadata exists; lead may propose authority signing/composition only with separate explicit instruction"}
    write_json(root / "inputs-receipt.json", receipt)
    directory_fd = os.open(root, os.O_RDONLY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)
    return {"status": receipt["status"], "receipt": str(root / "inputs-receipt.json"), "gate": "CLOSED"}


def main():
    need(len(sys.argv) == 2, "expected_one_private_preparation_config")
    raw = read_regular(sys.argv[1], private=True)
    result = prepare(json.loads(raw), digest(raw))
    print(json.dumps(result, sort_keys=True), flush=True)


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"status": "SUPPORTING_PREPARATION_UNCONFIRMED", "code": str(error)
                          if isinstance(error, Refusal) else type(error).__name__, "gate": "CLOSED"}), flush=True)
        sys.exit(2)
