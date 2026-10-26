#!/usr/bin/env python3
"""Compose a closed fixture from operator bindings and a final byte inventory.

SOURCE AUTHORED ONLY. This module has not been imported or executed. Composition
is a future explicitly approved local action; it never starts services, probes an
executable, resolves credentials, builds an artifact, or grants gate authority.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit


BACKENDS = ("postgres_pgvector", "surreal_disk")
ENV = re.compile(r"[A-Z][A-Z0-9_]*")
DIGEST = re.compile(r"sha256:[0-9a-f]{64}")
ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent.parent
CLOSURES = ("native", "python", "node", "python_sdk", "typescript_sdk", "rust_sdk",
            "grpc", "playwright", "browser", "video_encoder", "classifier_wasm", "local_embedding_assets")


class Incomplete(ValueError):
    """A closed input-contract error; never print input contents or credentials."""


def need(value, reason):
    if not value:
        raise Incomplete(reason)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def absolute(value):
    need(isinstance(value, str) and Path(value).is_absolute() and "REPLACE" not in value,
         "an explicit absolute path is required")
    return str(Path(value))


def env(value):
    need(isinstance(value, str) and ENV.fullmatch(value), "an explicit environment reference is required")
    return value


def private_parent(value):
    path = Path(absolute(value))
    need(path.is_dir() and not path.is_symlink() and path.stat().st_mode & 0o077 == 0,
         "owned state parent must already be an explicit private directory")
    return path


def positive(value, name, integer=False):
    need(type(value) in ((int,) if integer else (int, float)) and math.isfinite(value) and value > 0,
         "positive finite operator allowance required: " + name)
    return value


def port(value):
    positive(value, "port", True)
    need(1024 <= value <= 65535, "owned port must be nonprivileged")
    return value


def reference(value, inventory):
    need(isinstance(value, dict) and set(value) == {"path", "sha256"}, "frozen reference shape differs")
    path = absolute(value["path"])
    need(inventory.get(path) == value["sha256"], "frozen reference is outside the final inventory")
    return dict(value)


def linux_contract(bindings, inventory, paths, artifact, source_identity):
    """Bind already prepared immutable image bytes; never invoke Docker here."""
    platform = bindings.get("runtime_platform", "macos-native")
    need(platform in ("macos-native", "linux-container"), "unsupported runtime platform")
    if platform == "macos-native":
        need(bindings.get("linux_runtime") is None, "native runtime cannot carry Linux launch authority")
        return None
    spec = copy.deepcopy(bindings["linux_runtime"])
    need(spec.get("schema_version") == 1 and spec.get("executable") is True,
         "Linux runtime requires explicit approved executable binding")
    need(spec.get("host_alias") == "host.docker.internal", "Linux host alias must be explicitly approved Colima alias")
    need(spec.get("repo_root") == str(REPO) and spec.get("container_python") == "/usr/local/bin/python3",
         "Linux runtime requires same-path repository and explicit image Python")
    need(spec.get("architecture") in ("arm64", "amd64") and DIGEST.fullmatch(spec.get("image_id", ""))
         and (spec.get("image_source_id") == spec["image_id"] or
              re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:/-]*@sha256:[0-9a-f]{64}", spec.get("image_source_id", ""))),
         "Linux runtime image source, immutable ID and architecture must be explicit")
    absolute(spec["container_binary"])
    client = reference(spec["docker_client"], inventory)
    need(client["path"] == paths["docker"], "Linux and store Docker clients must match")
    receipt_ref = reference(spec["image_receipt"], inventory)
    receipt = read(Path(receipt_ref["path"]))
    need(receipt.get("schema_version") == 1 and receipt.get("status") == "LINUX_IMAGE_PREPARED_NOT_RUNTIME_ACCEPTANCE"
         and receipt.get("source_identity") == source_identity and receipt.get("artifact_identity") == artifact["sha256"]
         and receipt.get("binary_path") == artifact["path"] and receipt.get("docker_client") == client
         and all(receipt.get(key) == spec[key] for key in
                 ("image_id", "image_source_id", "architecture", "container_binary", "container_python")),
         "Linux image receipt does not bind the frozen extracted binary/source/client")
    need(Path(artifact["path"]).read_bytes()[:4] == b"\x7fELF", "Linux installed artifact must be extracted ELF, never Mach-O")
    routes = spec.get("owned_endpoint_routes")
    need(isinstance(routes, list) and routes, "Linux owned endpoint routing inventory is required")
    for row in routes:
        need(isinstance(row, dict) and set(row) == {"scheme", "host", "port"}
             and row["host"] == "127.0.0.1" and row["scheme"] in ("http", "https", "ws", "wss", "postgres", "postgresql"),
             "Linux route must name an exact owned loopback endpoint")
        port(row["port"])
    need(len({(row["scheme"], row["host"], row["port"]) for row in routes}) == len(routes), "duplicate Linux endpoint route")
    roots = spec.get("readonly_resource_roots")
    need(isinstance(roots, list) and len(roots) == len(set(roots)), "Linux readonly resource roots must be explicit")
    for value in roots:
        root = Path(absolute(value))
        need(root.is_dir() and not root.is_symlink() and root != REPO and not REPO.is_relative_to(root)
             and not root.is_relative_to(REPO), "Linux extra readonly roots must be disjoint from repository")
    return spec


def endpoint(value, https=False):
    need(isinstance(value, str), "explicit endpoint required")
    parsed = urlsplit(value)
    need(parsed.hostname and not parsed.username and not parsed.password and not parsed.query and not parsed.fragment,
         "endpoint must not embed credentials or arbitrary query/fragment")
    need(parsed.scheme == "https" if https else parsed.scheme in ("http", "https"), "endpoint scheme unsupported")
    if parsed.scheme == "http":
        need(parsed.hostname == "127.0.0.1" and parsed.port is not None, "HTTP endpoint must be explicit owned IPv4 loopback")
    return value.rstrip("/")


def split_fields(text):
    """Split source DDL fields, preserving nested type/default expressions."""
    start = depth = 0
    quote = None
    result = []
    index = 0
    while index < len(text):
        char = text[index]
        if quote:
            if char == quote:
                if index + 1 < len(text) and text[index + 1] == quote:
                    index += 1
                else:
                    quote = None
        elif char in ("'", '"'):
            quote = char
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == "," and depth == 0:
            result.append(text[start:index].strip())
            start = index + 1
        index += 1
    result.append(text[start:].strip())
    return result


def schema_contract(inventory):
    """Read every actual migration; never query or initialize a datastore."""
    postgres = sorted((REPO / "migrations").glob("*.sql"), key=lambda path: int(path.name.split("_", 1)[0]))
    surreal_hosts = [REPO / "src/uar/persistence/providers" / name for name in
                     ("surreal_host.rs", "ingestion_records/surreal.rs", "accounting/surreal.rs", "checkpoint_records/surreal.rs")]
    surreal = set()
    for host in surreal_hosts:
        need(str(host) in inventory, "actual Surreal migration host source missing from inventory")
        for match in re.finditer(r'include_str!\s*\(\s*"([^"]+\.surql)"\s*\)', host.read_text()):
            surreal.add((host.parent / match.group(1)).resolve())
    surreal = sorted(surreal)
    need(postgres and surreal, "complete migration sources unavailable")
    tables = {}
    versions = set()
    for path in postgres:
        need(str(path) in inventory, "top-level SQL migration missing from final source inventory")
        version = int(path.name.split("_", 1)[0])
        need(version not in versions, "duplicate SQLx migration version")
        versions.add(version)
        text = re.sub(r"--[^\n]*", "", path.read_text())
        for match in re.finditer(r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\(", text, re.I):
            position, depth, quote = match.end(), 1, None
            end = position
            while end < len(text) and depth:
                char = text[end]
                if quote:
                    if char == quote:
                        if end + 1 < len(text) and text[end + 1] == quote:
                            end += 1
                        else:
                            quote = None
                elif char in ("'", '"'):
                    quote = char
                elif char == "(":
                    depth += 1
                elif char == ")":
                    depth -= 1
                end += 1
            need(depth == 0, "migration CREATE TABLE source is not closed")
            columns = tables.setdefault(match.group(1), set())
            for field in split_fields(text[position:end - 1]):
                head = re.match(r'"?([a-zA-Z_][a-zA-Z0-9_]*)"?\s+', field)
                if head and head.group(1).upper() not in ("CONSTRAINT", "PRIMARY", "UNIQUE", "FOREIGN", "CHECK", "EXCLUDE"):
                    columns.add(head.group(1))
        for match in re.finditer(r"ALTER\s+TABLE\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+ADD\s+COLUMN\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z_][a-zA-Z0-9_]*)", text, re.I):
            need(match.group(1) in tables, "column addition has no frozen source table")
            tables[match.group(1)].add(match.group(2))
        for match in re.finditer(r"ALTER\s+TABLE\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+RENAME\s+COLUMN\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+TO\s+([a-zA-Z_][a-zA-Z0-9_]*)", text, re.I):
            columns = tables[match.group(1)]
            columns.discard(match.group(2))
            columns.add(match.group(3))
        for match in re.finditer(r"ALTER\s+TABLE\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+DROP\s+COLUMN\s+(?:IF\s+EXISTS\s+)?([a-zA-Z_][a-zA-Z0-9_]*)", text, re.I):
            tables[match.group(1)].discard(match.group(2))
    stables = {}
    for path in [*surreal, surreal_hosts[-1]]:
        need(str(path) in inventory, "Surreal migration missing from final source inventory")
        for match in re.finditer(r"DEFINE\s+TABLE\s+(?:(?:OVERWRITE|IF\s+NOT\s+EXISTS)\s+)?([a-zA-Z_][a-zA-Z0-9_]*)", path.read_text(), re.I):
            # Empty columns explicitly mean table-presence proof, not invented
            # field constraints for SCHEMALESS/JSON-record migration tables.
            stables[match.group(1)] = []
    # Base agents are concretely created by protected seeding. The old
    # schema.surql is not executed by this host; do not pretend all its legacy
    # tables/field constraints exist before the first user conversation.
    stables["agents"] = []
    required = {"agents", "durable_run_records", "durable_run_events", "accounting_calls",
                "canonical_tool_receipts", "tool_admission_evidence", "principal_knowledge_bases",
                "principal_knowledge_documents", "principal_knowledge_chunks", "principal_knowledge_ingestion",
                "knowledge_ingestion", "agent_threads", "uar_collaboration_state", "agui_runs"}
    need(required.issubset(tables) and required.issubset(stables), "migration source table closure is incomplete")
    return {"postgres_pgvector": {"tables": {name: sorted(columns) for name, columns in sorted(tables.items())},
                                   "migration_sources": [str(path) for path in postgres]},
            "surreal_disk": {"tables": dict(sorted(stables.items())),
                             "migration_sources":[str(path) for path in surreal],
                             "inline_migration_sources":[str(surreal_hosts[-1])]}}


def replace_slots(value, bindings):
    if isinstance(value, dict):
        return {key: replace_slots(item, bindings) for key, item in value.items()}
    if isinstance(value, list):
        return [replace_slots(item, bindings) for item in value]
    if isinstance(value, str) and value.startswith("@binding:") and value.endswith("@"):
        key = value[9:-1]
        need(key in bindings and bindings[key] is not None, "native template binding missing: " + key)
        return copy.deepcopy(bindings[key])
    return value


def protected_overflow_vectors(bindings, providers):
    """Materialize source text only; actual durable refusal remains the oracle."""
    recipe = bindings.get("native_policy_source_recipes", {}).get("protected_overflow")
    if recipe is None:
        return []
    primary = providers[0]
    selected = next(model for model in primary["config"]["models"] if model["id"] == primary["wire_model"])
    context = positive(selected.get("context_window"), "actual primary declared context", True)
    need(recipe["model"] == primary["qualified_model"] and recipe["role"] == "user"
         and recipe["kind"] == "over_limit" and recipe["stream"] is False,
         "protected-overflow recipe differs from selected mounted destination")
    unit = recipe["repeat_unit"]
    need(isinstance(unit, str) and unit.isascii() and len(unit) > 0, "overflow candidate must use bounded ASCII source text")
    multiplier = positive(recipe["repeat_count_per_context_token"], "overflow source multiplier", True)
    need(len(unit) * context * multiplier + 8192 < 10 * 1024 * 1024,
         "protected overflow source text cannot fit the mounted request body")
    request = {"model":primary["qualified_model"], "stream":False,
               "max_completion_tokens":recipe["max_completion_tokens"],
               "messages":[{"role":"user","content":unit * (context * multiplier)}]}
    # Actual mounted source: DefaultBodyLimit::max(10 * 1024 * 1024).
    # Leave room for the case's dynamic session UUID without pretending to know
    # tokenizer exactness or to have observed protected-overflow admission.
    need(len(canonical(request)) + 4096 < 10 * 1024 * 1024,
         "protected overflow candidate exceeds mounted request body bound")
    return [{"kind":"over_limit", "request":request}]


def propose(bindings_path, output):
    """Author only inert proposal files; do not read/hash installed artifacts."""
    supplied = read(bindings_path)
    need(supplied.get("schema_version") == 1, "source proposal requires a version-1 bindings document")
    pending = read(ROOT / "fixtures" / "bindings.example.json")
    def merge(target, source):
        for key, value in source.items():
            if isinstance(value, dict) and isinstance(target.get(key), dict):
                merge(target[key], value)
            else:
                target[key] = copy.deepcopy(value)
    merge(pending, supplied)
    if supplied.get("native_policy_source") is not None:
        policy_path = Path(absolute(supplied["native_policy_source"]))
        need(policy_path == ROOT / "fixtures" / "native-policy.local-draft.json",
             "local proposal policy must name the owned source policy artifact")
        policy = read(policy_path)
        need(policy.get("schema_version") == 1 and policy.get("example") is True
             and policy.get("executable") is False and isinstance(policy.get("native_configuration"), dict),
             "source policy must remain an inert typed native configuration proposal")
        pending["native_configuration"] = copy.deepcopy(policy["native_configuration"])
        merge(pending["native_configuration"], supplied.get("native_configuration", {}))
        pending["native_policy_source_recipes"] = copy.deepcopy(policy.get("source_recipes", {}))
        if "teams" in policy.get("source_recipes", {}):
            merge(pending["teams"], policy["source_recipes"]["teams"])
            merge(pending["teams"], supplied.get("teams", {}))
    if "packet_recipe" in supplied and "model_role_recipe" in supplied:
        recipe = supplied["packet_recipe"]
        roles = supplied["model_role_recipe"]
        wire_rows = []
        providers = []
        for role in ("primary", "alternate", "structured"):
            choice = roles[role]
            alias = choice["wire_model"]
            model_source = next(row for row in supplied["confirmed_model_aliases"] if row["alias"] == alias)
            capabilities = model_source["catalog_capabilities"]
            configured_model = {"id":alias, "enabled":True, "context_window":model_source["context_window"],
                "max_output_tokens":model_source["max_output_tokens"], "supports_tools":capabilities["function_calling"],
                "supports_vision":capabilities["vision"], "supports_structured_output":capabilities["structured_output"],
                "supports_reasoning":capabilities["reasoning"], "supports_streaming":True}
            providers.append({"kind":"genuine", "qualified_model":choice["provider_id"] + "/" + alias,
                "wire_model":alias, "credential_env":roles["ingress_key_env"], "budget_binding_id":choice["budget_binding_id"],
                "config":{"id":choice["provider_id"], "display_name":"Owned " + role, "protocol":"chat",
                          "default_model":alias, "enabled":True, "models":[configured_model]}})
            service = supplied["confirmed_existing_service"]
            wire_rows.append({"id":choice["budget_binding_id"], "kind":"model", "spend_pool":"liter",
                "listen_port":choice["listen_port"], "request_path":"/v1/chat/completions", "model_path":"/model",
                "upstream_url":roles["upstream_url"], "allowed_models":[alias],
                "headers_env":{"Authorization":roles["upstream_key_env"]}, "header_prefixes":roles["header_prefixes"],
                "ingress_authorization_env":roles["ingress_key_env"], "ingress_authorization_prefix":roles["ingress_authorization_prefix"],
                "max_output_tokens_per_attempt":roles["max_output_tokens_per_attempt"],
                "output_token_paths":["/max_tokens","/max_completion_tokens"], "approved_charge_ceiling_microunits":0,
                "max_request_bytes":None, "max_response_bytes":None, "charge_approval":{"path":None,"sha256":None},
                "upstream_authority":{"kind":"existing-service-loopback", "service_id":service["service_id"],
                    "operator_approved":False, "endpoint_url":roles["upstream_url"],
                    **{name:copy.deepcopy(service[name]) for name in ("launchagent","launch_script","service_config")},
                    "approval":{"path":None,"sha256":None}}})
        embedding = copy.deepcopy(supplied["embedding_wire_recipe"])
        embedding.update(max_request_bytes=None, max_response_bytes=None, charge_approval={"path":None,"sha256":None})
        wire_rows.append(embedding)
        for wire in wire_rows:
            wire.update(copy.deepcopy(supplied["binding_request_limits"]))
        pending["providers"] = providers
        pending["budget_proxy"]["bindings"] = wire_rows
        for backend, prefix in (("postgres_pgvector","postgres"),("surreal_disk","surreal")):
            pending["backends"][backend].update(http_port=recipe["ports"][prefix + "_http"],
                                               grpc_port=recipe["ports"][prefix + "_grpc"])
        pending["backends"]["surreal_disk"]["surreal_http_port"] = recipe["ports"]["surreal"]
        pending["capabilities"]["genuine_proxy_port"] = recipe["ports"]["structured_metadata"]
        pending["capabilities"]["structured_binding_model"] = providers[2]["qualified_model"]
        pending["postgres_store"]["image"] = recipe["postgres_image"]
        pending["surreal"]["disk_container"]["image"] = recipe["surreal_image"]
        service = pending["existing_service_observer"]
        service["sources"].update({name:copy.deepcopy(supplied["confirmed_existing_service"][name])
                                   for name in ("launchagent","launch_script","service_config")})
        service["expected_models"] = [{"alias":row["alias"], "provider_model":row["configured_route"],
                                       "upstream_origin":urlsplit(row["configured_upstream"]).scheme + "://" + urlsplit(row["configured_upstream"]).netloc}
                                      for row in supplied["confirmed_model_aliases"]]
    def inert(value, parent=""):
        if isinstance(value, dict):
            for key, item in value.items():
                if key in ("executable", "finalized", "full_closure", "full_closure_approved", "operator_approved"):
                    value[key] = False
                else:
                    if key.lower() == "authorization" and item is not None:
                        if parent == "headers_env":
                            env(item)
                        else:
                            need(parent == "header_prefixes" and item == "Bearer ",
                                 "source proposals cannot copy literal authorization values")
                    if key in ("api_key", "embedding_api_key", "jwt_secret", "settings_admin_key", "openai_api_key", "cohere_api_key", "anthropic_api_key", "mistral_api_key") and item is not None:
                        need(isinstance(item, str) and re.fullmatch(r"@env:[A-Z][A-Z0-9_]*@", item),
                             "source proposals cannot copy inline credential values")
                    inert(item, key)
        elif isinstance(value, list):
            for item in value:
                inert(item, parent)
    inert(pending)
    if not pending["budget_vectors"]:
        pending["budget_vectors"] = protected_overflow_vectors(pending, pending["providers"])
    pending.update(example=True, status="SOURCE_PROPOSAL_PENDING_BUILD_AND_AUTHORITY")
    manifest = read(ROOT / "fixtures" / "scenarios.json")
    count = len(manifest["required_cases"])
    need(count == manifest["source_authored_case_count"] and len(set(manifest["required_cases"])) == count,
         "literal source manifest requires lead reconciliation")
    need(manifest["positive_repetitions"] == 3 and manifest["backends"] == list(BACKENDS),
         "source manifest must preserve three repetitions on both mandatory adapters")
    missing = []
    def slots(value, path=""):
        if value is None or value == []:
            missing.append(path)
        elif isinstance(value, dict):
            for key, item in value.items():
                slots(item, path + "/" + key)
        elif isinstance(value, list):
            for index, item in enumerate(value):
                slots(item, path + "/" + str(index))
    slots(pending)
    need(not output.exists(), "proposal output must be a new exclusive directory")
    output.parent.resolve(strict=True)
    output.mkdir(mode=0o700)
    def emit(name, value):
        with (output / name).open("xb") as handle:
            handle.write(canonical(value) + b"\n")
            handle.flush()
            os.fsync(handle.fileno())
        (output / name).chmod(0o600)
    emit("bindings.pending.json", pending)
    emit("inventory.pending.json", read(ROOT / "fixtures" / "inventory.example.json"))
    recipe = {"schema_version":1, "executable":False, "status":"SOURCE_PROPOSAL_NOT_READY_NOT_GATE_AUTHORITY",
        "input_bindings":str(bindings_path), "unresolved_slots":missing,
        "modules":manifest["modules"], "required_cases":manifest["required_cases"], "case_count":count,
        "planned_rows":count * manifest["positive_repetitions"] * len(manifest["backends"]),
        "dependency_order":[
            {"stage":"freeze_source", "authority":"lead whole-phase inventory; no build or runtime claim"},
            {"stage":"approved_build", "requires":"explicit operator instruction naming actual build argv and unfinished/derived items",
             "proposed_argv":supplied.get("proposed_build_argv"), "produces":"actual artifact and successful source-bound build receipt"},
            {"stage":"prepare_private_inputs", "requires":"separately approved exact private-input preparation and actual source-backed credential refs",
             "program_source":str(ROOT / "prepare_private_inputs.py"), "executable_binding":"installed.python",
             "config_slot":"private_inputs_config", "runnable":False, "produces":"private env values plus nonsecret identity patches/receipt"},
            {"stage":"adopt_resource_authority", "requires":"actual tariff/epoch/prior-consumption/unknown reconciliation and external signed budget authority",
             "self_reference":"signature bytes are not included in their own signed preimage; authority does not hash the derived fixture"},
            {"stage":"deterministic_compose", "requires":"final actual byte inventory, actual build receipt, adopted signature/seed/charge refs; no unresolved slots",
             "argv_template":[pending.get("installed", {}).get("python"),str(ROOT / "compose_fixture.py"),
                 "--bindings",str(output / "bindings.pending.json"),"--inventory",str(output / "inventory.pending.json"),
                 "--output",str(output.parent / (output.name + "-sealed")),"--compose"], "runnable":False},
            {"stage":"external_derived_adoption", "requires":"new derived-input-receipt.json and final exact operator gate instruction",
             "self_reference":"final derived receipt/adoption/signature stays external; do not inject it into the frozen fixture it names"}],
        "templates":[str(ROOT / "fixtures" / name) for name in
            ("postgres-server-template.json","surreal-server-template.json","team-agent.json","team-template.json","team-binding.json")],
        "claims":{"installed_files_read":False,"artifact_built":False,"signatures_adopted":False,
                  "services_started":False,"runtime_verified":False}}
    emit("proposal.json", recipe)
    return {"status":recipe["status"],"proposal":str(output / "proposal.json"),"case_count":count,
            "planned_rows":recipe["planned_rows"],"executable":False}


def compose(bindings_path, inventory_path, output):
    bindings = read(bindings_path)
    final = read(inventory_path)
    need(bindings.get("schema_version") == 1 and bindings.get("executable") is True
         and bindings.get("example") is False, "example bindings cannot compose an executable fixture")
    need(final.get("schema_version") == 1 and final.get("finalized") is True,
         "a lead-finalized complete source and installed-artifact inventory is required")
    need(bindings.get("full_closure_approved") is True and final.get("full_closure") is True,
         "full installed dependency/resource closure must be explicitly approved")
    inventory = {}
    for row in final["files"]:
        path = absolute(row["path"])
        need(path not in inventory and DIGEST.fullmatch(row["sha256"]), "duplicate or invalid frozen inventory entry")
        candidate = Path(path)
        need(candidate.is_file() and not candidate.is_symlink(), "final inventory file is not regular")
        need(sha(candidate.read_bytes()) == row["sha256"], "final inventory bytes changed")
        inventory[path] = row["sha256"]
    for path in (bindings_path, inventory_path):
        need(path.is_absolute() and path.is_file() and not path.is_symlink(), "composition input must be an absolute regular file")
    prepared_reference = reference(bindings["private_inputs"]["inputs"], inventory)
    preparation_receipt_reference = reference(bindings["private_inputs"]["receipt"], inventory)
    preparation_config_reference = reference(bindings["private_inputs_config"], inventory)
    prepared = read(Path(prepared_reference["path"]))
    preparation_receipt = read(Path(preparation_receipt_reference["path"]))
    preparation_config = read(Path(preparation_config_reference["path"]))
    scope_extension = None
    scope_reference = None
    if "composition_scope_approval" in bindings:
        scope_reference = reference(bindings["composition_scope_approval"], inventory)
        scope_path = Path(scope_reference["path"])
        need(scope_path.stat().st_uid == os.getuid() and scope_path.stat().st_mode & 0o077 == 0,
             "composition output extension must be a private owned frozen document")
        scope_extension = read(scope_path)
        need(scope_extension.get("schema_version") == 1 and scope_extension.get("operator_approved") is True
             and scope_extension.get("operation") == "extend-uar-private-composition-outputs"
             and scope_extension.get("private_inputs") == prepared_reference
             and scope_extension.get("private_config") == preparation_config_reference,
             "composition output extension differs from preserved private inputs/config")
        scope_root = Path(absolute(scope_extension["output_root"]))
        need(scope_root.is_dir() and not scope_root.is_symlink() and scope_root.resolve(strict=True) == scope_root,
             "composition output extension requires the exact existing canonical output root")
        scope_outputs = scope_extension.get("outputs")
        need(isinstance(scope_outputs, dict) and set(scope_outputs) == {"macos-native", "linux-container"}
             and len(set(scope_outputs.values())) == 2,
             "composition extension must list both distinct exact platform outputs")
        for value in scope_outputs.values():
            candidate = Path(absolute(value))
            need(candidate.parent == scope_root and candidate.name not in (".", "..")
                 and not candidate.is_symlink() and candidate.resolve() == candidate,
                 "composition extension output must be a canonical direct child of its approved root")
        need(scope_outputs.get(bindings.get("runtime_platform", "macos-native")) == str(output),
             "selected platform output is outside the exact composition extension")
        need(not output.exists() and not output.is_symlink(), "selected platform composition output must be fresh")
    need(prepared.get("status") == "PRIVATE_INPUTS_PREPARED_NOT_ADOPTED"
         and prepared.get("epoch_id") == bindings["budget_epoch_id"] == preparation_receipt.get("epoch_id")
         and preparation_receipt.get("inputs_sha256") == prepared_reference["sha256"]
         and preparation_receipt.get("inputs_path") == prepared_reference["path"]
         and preparation_receipt.get("preparation_config_sha256") == preparation_config_reference["sha256"]
         and (scope_extension is not None or preparation_config.get("composition_output_dir") == str(output)),
         "actual private input/config/receipt identities differ from this composition")
    bindings["native_configuration"]["security"].update(prepared["security_patch"])
    bindings["orchestrator"] = prepared["orchestrator"]
    bindings["teams"]["workspace_id"] = prepared["primary"]["workspace_id"]
    bindings["teams"]["foreign_authorization_env"] = prepared["foreign_authorization_env"]
    bindings["capabilities"]["foreign_authorization_env"] = prepared["foreign_authorization_env"]
    for backend_name, child in (("postgres_pgvector", "postgres"), ("surreal_disk", "surreal")):
        bindings["backends"][backend_name]["state_parent"] = str(Path(prepared["scope_root"]) / child)
        bindings["backends"][backend_name]["headers_env"] = prepared["primary_headers_env"]
        prefix = "POSTGRES" if backend_name == "postgres_pgvector" else "SURREAL"
        derived_refs = {"state_parent_env":"UAR_FIXTURE_" + prefix + "_STATE_PARENT",
                        "runtime_config_env":"UAR_FIXTURE_" + prefix + "_RUNTIME_CONFIG",
                        "startup_config_env":"UAR_FIXTURE_" + prefix + "_STARTUP_CONFIG"}
        need(set(derived_refs.values()).issubset(prepared["environment_names"]), "prepared backend environment references differ from their actual generator")
        bindings["backends"][backend_name].update(derived_refs)
    bindings["budget_ledger_parent"] = str(Path(prepared["scope_root"]) / "packet")
    bindings["budget_state_parent_env"] = "UAR_FIXTURE_PACKET_STATE_PARENT"
    need(bindings["budget_state_parent_env"] in prepared["environment_names"], "prepared packet environment reference missing")
    bindings["backends"]["surreal_disk"]["surreal_http_port"] = prepared["ports"]["surreal"]
    bindings["postgres"].update(prepared["datastores"]["postgres"])
    bindings["surreal"].update(prepared["datastores"]["surreal"])
    for provider in bindings["providers"]:
        need(provider["credential_env"] == prepared["budget_ingress"]["ingress_authorization_env"],
             "provider must reference the raw prepared ingress key, not an upstream credential")
    for proxy in bindings["budget_proxy"]["bindings"]:
        need(all(proxy.get(key) == value for key, value in prepared["budget_ingress"].items()),
             "budget ingress must match the source-traced private input contract")
    closures = final["closures"]
    for name in CLOSURES:
        group = closures.get(name, {})
        need(group.get("complete") is True and isinstance(group.get("files"), list) and group["files"],
             "installed closure is incomplete: " + name)
        need(all(absolute(path) in inventory for path in group["files"]), "closure file absent from inventory")
        closure_reference = reference(group["manifest"], inventory)
        closure = read(Path(closure_reference["path"]))
        need(closure.get("kind") == name and isinstance(closure.get("files"), list)
             and {row["path"]:row["sha256"] for row in closure["files"]}
                 == {path:inventory[path] for path in group["files"]}, "installed closure manifest differs from its complete frozen inventory")
    paths = bindings["installed"]
    local_model_files = {"bg-small-en-v1.5.onnx", "tokenizer.json", "config.json",
                         "special_tokens_map.json", "tokenizer_config.json"}
    local_model_roots = {Path(path).parent for path in closures["local_embedding_assets"]["files"]
                         if Path(path).name == "bg-small-en-v1.5.onnx"}
    need(len(local_model_roots) == 1, "local strategy needs one frozen packaged embedding resource root")
    local_models_dir = next(iter(local_model_roots))
    need(all(str(local_models_dir / name) in closures["local_embedding_assets"]["files"]
             for name in local_model_files), "local strategy embedding resource files are incomplete")
    for key in ("server_binary", "python", "node", "psql", "docker", "python_sdk_entry",
                "typescript_sdk_entry", "rust_sdk_binary", "grpc_test_binary", "browser_executable",
                "playwright_entry", "video_encoder", "classifier_wasm"):
        need(absolute(paths[key]) in inventory, "installed executable/source input is not frozen: " + key)
    need(paths["server_binary"] in closures["native"]["files"] and paths["browser_executable"] in closures["browser"]["files"],
         "installed entry differs from its approved dependency closure")
    artifact = reference(final["installed_artifact"], inventory)
    need(artifact["path"] == paths["server_binary"], "built artifact and installed server identity differ")
    build_reference = reference(final["build_receipt"], inventory)
    for key in ("operator_approval", "operator_signature"):
        reference(bindings[key], inventory)
    sources = sorted(set(final["source_files"]))
    need(sources and len(sources) == len(final["source_files"]) and all(path in inventory for path in sources),
         "final selected source denominator is incomplete")
    derived_paths = {artifact["path"], build_reference["path"], prepared_reference["path"],
                     preparation_receipt_reference["path"], preparation_config_reference["path"],
                     bindings["operator_approval"]["path"], bindings["operator_signature"]["path"],
                     str(bindings_path), str(inventory_path)}
    if bindings.get("runtime_platform") == "linux-container":
        derived_paths.add(reference(bindings["linux_runtime"]["image_receipt"], inventory)["path"])
    if scope_reference is not None:
        derived_paths.add(scope_reference["path"])
    need(not set(sources).intersection(derived_paths),
         "source-code identity must exclude later artifacts, authority and generated-input receipts")
    source_identity = sha(canonical([{"path":path, "sha256":inventory[path]} for path in sources]))
    if scope_extension is not None:
        need(scope_extension.get("source_identity") == source_identity
             and read(Path(bindings["operator_approval"]["path"])).get("composition_scope_approval") == scope_reference,
             "composition extension must match current source and frozen final operator authority")
    linux = linux_contract(bindings, inventory, paths, artifact, source_identity)
    build = read(Path(build_reference["path"]))
    need(build.get("source_identity") == source_identity and build.get("artifact_identity") == artifact["sha256"]
         and build.get("artifact_path") == artifact["path"] and build.get("exit_code") == 0
         and build.get("operator_approved_command") == build.get("command")
         and isinstance(build.get("command"), list) and build["command"],
         "installed artifact requires the actual approved successful build receipt for this final source")
    for path in [ROOT / "compose_fixture.py", ROOT / "core.py", ROOT / "user_outcomes.py", *ROOT.glob("cases_*.py"),
                 *ROOT.glob("observers_*.py"), ROOT / "protocol_clients.py", ROOT / "browser_client.mjs",
                 ROOT / "tools_fixture.py", ROOT / "budget_proxy.py", ROOT / "budget_proxy.README.md", ROOT / "budget_proxy.schema.json",
                 ROOT / "budget_proxy.example.json", ROOT / "packet_processes.py", ROOT / "observers_service.py", ROOT / "observers_service.README.md",
                 ROOT / "prepare_private_inputs.py", ROOT / "prepare_private_inputs.README.md", ROOT / "compose_fixture.README.md",
                 ROOT / "prepare_artifacts.py", ROOT / "prepare_artifacts.README.md",
                 ROOT / "linux_runtime.py", ROOT / "linux_runtime.README.md",
                 ROOT / "failure_sweep.py", ROOT / "failure_sweep.README.md",
                 ROOT / "fixtures" / "bindings.local-draft.json", ROOT / "fixtures" / "private-inputs.local-draft.json",
                 ROOT / "fixtures" / "native-policy.local-draft.json",
                 ROOT / "fixtures" / "native-policy.local-draft.README.md",
                 REPO / "src/server.rs", REPO / "vendor/git/liter-llm/schemas/catalog.json",
                 ROOT / "stores_fixture.py", ROOT / "stores_fixture.README.md",
                 ROOT / "fixtures" / "scenarios.json"]:
        need(str(path) in sources, "acceptance source missing from the final denominator")
    manifest = read(ROOT / "fixtures" / "scenarios.json")
    need(manifest["positive_repetitions"] == 3 and manifest["backends"] == list(BACKENDS)
         and {name.split(".", 1)[0] for name in manifest["required_cases"]} == {f"U{number:02d}" for number in range(1, 26)},
         "literal full manifest must preserve all25 outcome families and both three-repetition adapters")
    case_count = len(manifest["required_cases"])
    planned_rows = case_count * manifest["positive_repetitions"] * len(BACKENDS)
    need(manifest["source_authored_case_count"] == case_count > 0
         and len(set(manifest["required_cases"])) == case_count and manifest["planned_backend_repetition_rows"] == planned_rows,
         "full source manifest changed; reconcile the canonical denominator before composing")
    for module in manifest["modules"]:
        need(str(ROOT / (module + ".py")) in sources, "manifest case implementation missing from the frozen source denominator")
    need(str(ROOT / "observers_skill_retention.py") in sources, "bounded-retention observer missing from frozen source denominator")
    schema = schema_contract(inventory)
    need(all(path in sources for path in schema["postgres_pgvector"]["migration_sources"]),
         "migrations must contribute to source identity")
    provider_rows = copy.deepcopy(bindings["providers"])
    need(len(provider_rows) >= 3, "primary, alternate and structured genuine model bindings are required")
    need(len({row["config"]["id"] for row in provider_rows}) == len(provider_rows), "provider IDs must be unique")
    budget = copy.deepcopy(bindings["budget_proxy"])
    need(budget.get("schema_version") == 1 and budget.get("executable") is True, "approved executable packet budget config required")
    selected_services = bindings.get("selected_embedding_services", ["cohere"])
    need(selected_services in (["cohere"], ["cohere", "openai_embeddings"])
         and prepared.get("selected_embedding_services", ["cohere"]) == selected_services,
         "embedding selection differs from explicitly prepared private credentials")
    approved_pools = {"cohere":{"limit_microunits":10000000},
                      "openai_embeddings":{"limit_microunits":10000000},
                      "liter":{"limit_microunits":0}}
    need(budget.get("spend_pools") == approved_pools,
         "separate operator-approved provider allowances cannot be omitted, pooled or transferred")
    for name in ("approval_packet", "approval_signature", "seed"):
        reference(budget[name], inventory)
        need(budget[name]["path"] not in sources, "resource authority/seed inputs cannot enter their own source-build preimage")
    if scope_reference is not None:
        need(read(Path(budget["approval_packet"]["path"])).get("composition_scope_approval") == scope_reference,
             "signed resource authority must bind the exact composition extension reference")
    seed = read(Path(budget["seed"]["path"]))
    need(seed.get("epoch_id") == budget["epoch_id"] and seed.get("unresolved_reservations") == []
         and seed.get("prior_epoch_id") and seed.get("prior_consumption_evidence"),
         "prior epoch consumption and all unknown reservations must be explicitly reconciled")
    for row in seed["prior_consumption_evidence"]:
        reference(row, inventory)
    for name in ("model_attempts", "embedding_attempts", "generated_tokens", "cost_microunits", "request_bytes", "response_bytes", "max_in_flight", "wall_seconds"):
        positive(budget["limits"][name], "packet budget " + name, True)
    counters = ("model_attempts", "embedding_attempts", "generated_tokens", "cost_microunits", "request_bytes", "response_bytes")
    need(budget["limits"]["cost_microunits"] <= 20000000, "aggregate cost exceeds the two independent allowances")
    pool_consumed = seed.get("spend_pool_consumed_microunits", {})
    need(set(pool_consumed) == set(approved_pools)
         and all(type(pool_consumed[key]) is int and 0 <= pool_consumed[key] <= approved_pools[key]["limit_microunits"]
                 for key in approved_pools)
         and sum(pool_consumed.values()) == seed["consumed"]["cost_microunits"],
         "prior provider-pool consumption must be reconciled without reset or transfer")
    need(set(seed["consumed"]) == set(counters)
         and all(type(seed["consumed"][key]) is int and 0 <= seed["consumed"][key] <= budget["limits"][key] for key in counters),
         "prior consumption differs from the exact approved remaining allowance")
    need(seed["approval_packet_sha256"] == budget["approval_packet"]["sha256"]
         and seed["approval_signature_sha256"] == budget["approval_signature"]["sha256"], "seed approval identities differ")
    positive(seed["epoch_started_unix"], "epoch start", True)
    need(type(seed["deadline_unix"]) is int and 0 < seed["deadline_unix"] - seed["epoch_started_unix"] <= budget["limits"]["wall_seconds"],
         "absolute budget deadline differs from approved epoch duration")
    positive(budget["upstream_timeout_seconds"], "upstream timeout", True)
    positive(budget["ingress_timeout_seconds"], "ingress timeout", True)
    need(budget["ingress_timeout_seconds"] <= budget["upstream_timeout_seconds"] <= budget["limits"]["wall_seconds"],
         "proxy timeout exceeds epoch allowance")
    need(budget["epoch_id"] == bindings["budget_epoch_id"], "explicit budget epoch differs")
    ledger_parent = private_parent(bindings["budget_ledger_parent"])
    budget["ledger_root"] = str(ledger_parent / ("uar_budget_" + hashlib.sha256(budget["epoch_id"].encode()).hexdigest()[:20]))
    service = copy.deepcopy(bindings["existing_service_observer"])
    need(service.get("schema_version") == 1 and service.get("executable") is True,
         "actual existing-service observer binding is required before any positive dispatch")
    for tool in ("launchctl", "ps", "lsof"):
        reference(service["tools"][tool], inventory)
    for source in ("launchagent", "launch_script", "service_config", "executable"):
        reference(service["sources"][source], inventory)
    need(service["expected_max_retries"] == 3 and service["expected_cost_tracking"] is False
         and service["expected_alias_patterns"] == ["openai/*"] and service["expected_models"],
         "configured existing-service inspection expectations differ from the observed source contract")
    need({row["alias"] for row in service["expected_models"]} == {"gpt-5.4", "k3", "MiniMax-M3"}
         and service["expected_argv"] == [service["sources"]["executable"]["path"], "api", "--config", service["sources"]["service_config"]["path"]],
         "existing-service expected models/argv must bind the actual configured service")
    need(service["approved_listener_names"] and set(service["approved_listener_names"]).issubset({"127.0.0.1:4000", "[::1]:4000", "*:4000"}),
         "existing-service listener expectations must name exact approved port4000 sockets")
    for key, variable in bindings["existing_service_environment_env"].items():
        env(key)
        env(variable)
    service["receipt_path"] = str(ledger_parent / ("existing-service-" + hashlib.sha256(
        canonical({"epoch_id":budget["epoch_id"], "composition_output":str(output)})).hexdigest()[:20] + ".json"))
    budget_by_id = {row["id"]: row for row in budget["bindings"]}
    need(len(budget_by_id) == len(budget["bindings"]) and budget_by_id, "budget endpoint bindings must be unique and complete")
    ports = []
    secret_env = set(env(value) for value in bindings["secret_env"])
    for row in budget["bindings"]:
        ports.append(port(row["listen_port"]))
        parsed_upstream = urlsplit(row["upstream_url"])
        if row["kind"] == "model":
            need(row.get("spend_pool") == "liter" and row["approved_charge_ceiling_microunits"] == 0
                 and parsed_upstream.scheme == "http" and parsed_upstream.port == 4000,
                 "the operator selected existing Liter service; model routes must not bypass it")
        else:
            pool = row.get("spend_pool")
            embedding_endpoints = {"cohere":"https://api.cohere.ai/compatibility/v1/embeddings",
                                   "openai_embeddings":"https://api.openai.com/v1/embeddings"}
            need(pool in selected_services and row["upstream_url"] == embedding_endpoints[pool]
                 and row["request_path"] == "/v1/embeddings",
                 "embedding endpoint must match its separately approved selected provider pool")
            need(type(row["approved_charge_ceiling_microunits"]) is int
                 and row["approved_charge_ceiling_microunits"] > 0,
                 "paid embedding requests require an explicit positive full-request ceiling")
        if parsed_upstream.scheme == "https":
            endpoint(row["upstream_url"], True)
            need(row.get("upstream_authority") is None, "HTTPS endpoint must use its exact ordinary endpoint approval")
        else:
            authority = row["upstream_authority"]
            need(parsed_upstream.scheme == "http" and parsed_upstream.hostname in ("localhost", "127.0.0.1")
                 and parsed_upstream.port == 4000 and parsed_upstream.path == ("/v1/chat/completions" if row["kind"] == "model" else "/v1/embeddings")
                 and not parsed_upstream.username and not parsed_upstream.password and not parsed_upstream.query and not parsed_upstream.fragment,
                 "only the explicitly approved existing Liter service may use a local HTTP upstream")
            need(authority.get("kind") == "existing-service-loopback" and authority.get("service_id") == "ai.prometheus.liter-llm-api"
                 and authority.get("operator_approved") is True and authority.get("endpoint_url") == row["upstream_url"],
                 "existing Liter service authority differs")
            for name in ("launchagent", "launch_script", "service_config", "approval"):
                reference(authority[name], inventory)
            need(all(authority[name] == service["sources"][name] for name in ("launchagent", "launch_script", "service_config")),
                 "budget authority and actual service observer must bind the same source bytes")
            expected_authority = {"kind":authority["kind"], "service_id":authority["service_id"], "operator_approved":True,
                "endpoint_url":row["upstream_url"], "epoch_id":budget["epoch_id"], "binding_id":row["id"],
                "header_prefixes":row.get("header_prefixes", {}),
                **{name:authority[name] for name in ("launchagent", "launch_script", "service_config")}}
            need(read(Path(authority["approval"]["path"])) == expected_authority, "exact existing-service approval differs from its frozen provenance")
        reference(row["charge_approval"], inventory)
        charge = read(Path(row["charge_approval"]["path"]))
        need(charge.get("operator_approved") is True
             and charge.get("epoch_id") == budget["epoch_id"] and charge.get("binding_id") == row["id"]
             and charge.get("upstream_url") == row["upstream_url"] and charge.get("allowed_models") == row["allowed_models"]
             and charge.get("max_request_bytes") == row["max_request_bytes"] and charge.get("max_response_bytes") == row["max_response_bytes"]
             and charge.get("charge_ceiling_microunits") == row["approved_charge_ceiling_microunits"]
             and charge.get("spend_pool") == row["spend_pool"]
             and charge.get("spend_pool_limit_microunits") == approved_pools[row["spend_pool"]]["limit_microunits"]
             and isinstance(charge.get("pricing_evidence"), str) and charge["pricing_evidence"], "per-attempt charge approval differs from its exact binding")
        need(type(row["approved_charge_ceiling_microunits"]) is int and row["approved_charge_ceiling_microunits"] >= 0
             and (row["approved_charge_ceiling_microunits"] > 0 or charge.get("zero_charge_explicitly_approved") is True),
             "zero or uncertain per-attempt cost cannot be silently adopted")
        if parsed_upstream.scheme == "http":
            need(charge.get("upstream_authority_kind") == "existing-service-loopback", "existing service charge authority missing")
        prefixes = row.get("header_prefixes", {})
        need(prefixes in ({}, {"Authorization":"Bearer "}) and set(prefixes).issubset(row["headers_env"]),
             "only an explicitly approved bearer prefix may transform a referenced header")
        if prefixes:
            need(charge.get("header_prefixes") == prefixes, "header prefix differs from exact charge approval")
        if parsed_upstream.scheme == "http":
            need(row["headers_env"].get("Authorization") == "LITER_LLM_MASTER_KEY"
                 and prefixes == {"Authorization":"Bearer "}, "existing Liter raw master-key reference requires explicit approved Bearer prefix")
        elif row["kind"] == "embedding":
            credential = "COHERE_API_KEY" if row["spend_pool"] == "cohere" else "OPENAI_API_KEY"
            need(row["headers_env"].get("Authorization") == credential
                 and prefixes == {"Authorization":"Bearer "}, "embedding pool requires its selected raw key reference and approved Bearer prefix")
        need(row.get("ingress_authorization_prefix") == "Bearer ",
             "raw ingress credential references require the explicit protocol Bearer prefix")
        for key in ("max_request_bytes", "max_response_bytes"):
            positive(row[key], key, True)
        if row["kind"] == "model":
            positive(row["max_output_tokens_per_attempt"], "per-model output ceiling", True)
            need(row["max_output_tokens_per_attempt"] >= 4096, "actual summary role needs the admitted 4096-token ceiling")
            positive(charge["full_context_tokens"], "approved full-context exposure", True)
            need(charge["basis"] == "full-context-plus-output-ceiling"
                 and charge["max_output_tokens_per_attempt"] == row["max_output_tokens_per_attempt"]
                 and row["output_token_paths"] and set(row["output_token_paths"]).issubset({"/max_tokens", "/max_completion_tokens"}),
                 "model charge/cap contract differs from actual public request fields")
        else:
            need(charge["basis"] == "embedding-request-ceiling", "embedding charge basis missing")
        secret_env.add(env(row["ingress_authorization_env"]))
        secret_env.update(env(value) for value in row["headers_env"].values())
        need(row["kind"] in ("model", "embedding") and row["allowed_models"], "budget endpoint kind/models missing")
        need(row["request_path"].startswith("/") and row["model_path"] == "/model", "unsupported budget request contract")
    used_bindings = set()
    provider_budget_bindings = {}
    origins = []
    for row in provider_rows:
        config = row["config"]
        need(row.get("kind") == "genuine" and row["qualified_model"] == config["id"] + "/" + config["default_model"],
             "genuine provider identity differs")
        need(config["default_model"] in {"gpt-5.4", "k3", "MiniMax-M3"} and row["wire_model"] == config["default_model"],
             "model binding must use an actually configured existing-service alias")
        need(config.get("enabled") is True and config.get("protocol") == "chat" and "api_key" not in config,
             "full fixture requires explicit enabled preserving chat providers without inline credentials")
        wire = budget_by_id[row.pop("budget_binding_id")]
        need(wire["kind"] == "model" and row["wire_model"] in wire["allowed_models"], "actual serialized model route bypasses approved budget binding")
        need(any(model.get("id") == config["default_model"] and model.get("enabled") is True for model in config["models"]),
             "declared default model must be explicitly enabled in its configured provider")
        need(wire["request_path"] == "/v1/chat/completions", "model forwarder route differs from actual chat transport")
        config["base_url"] = "http://127.0.0.1:" + str(wire["listen_port"]) + "/v1"
        need(row["credential_env"] == wire["ingress_authorization_env"], "model ingress credential reference differs")
        secret_env.add(env(row["credential_env"]))
        used_bindings.add(wire["id"])
        provider_budget_bindings[config["id"]] = wire["id"]
        origins.append({"origin":config["base_url"].removesuffix("/v1"), "provider_ids":[config["id"]],
                        "budget_binding_id":wire["id"], "approved_upstream":wire["upstream_url"]})
    knowledge = copy.deepcopy(bindings["knowledge"])
    spaces = knowledge["embedding_spaces"]
    need(len(spaces) == 3 and all(row.get("kind") == "genuine" and row.get("synthetic") is False for row in spaces),
         "three actual approved embedding spaces are mandatory")
    a, b, c = [row["fingerprint"] for row in spaces]
    for fingerprint in (a, b, c):
        positive(fingerprint["dimension"], "embedding dimension", True)
    need(a != b and a["dimension"] == b["dimension"] and b["dimension"] != c["dimension"], "A/B/C embedding geometry differs")
    for row in spaces:
        wire = budget_by_id[row.pop("budget_binding_id")]
        need(wire["kind"] == "embedding" and row["fingerprint"]["model"] in wire["allowed_models"], "embedding route bypasses packet budget")
        row["endpoint"] = "http://127.0.0.1:" + str(wire["listen_port"]) + wire["request_path"].rsplit("/", 1)[0]
        need(row.get("source_kind") == "approved_hosted_endpoint" and DIGEST.fullmatch(row["model_source_identity"]),
             "hosted embedding source approval identity missing")
        reference(row["source_approval"], inventory)
        used_bindings.add(wire["id"])
    memory = bindings["native_configuration"]["memory"]
    memory_wire = budget_by_id[bindings["memory_budget_binding_id"]]
    need(memory_wire["kind"] == "embedding" and memory["embedding_model"] in memory_wire["allowed_models"]
         and memory.get("embedding_api") == "openai" and memory.get("embedding_provider") == "openai"
         and memory.get("embedding_dim") == 1024,
         "owned memory must use its explicit compatible hosted model, SDK adapter and known store dimension")
    used_bindings.add(memory_wire["id"])
    need(all(row["fingerprint"]["backend"] == "openai" for row in spaces),
         "Cohere compatibility uses the actual OpenAI backend rather than the native Cohere payload")
    need(used_bindings == set(budget_by_id), "every declared budget endpoint must have an explicit consumer")
    limits = bindings["limits"]
    for name in ("wall_seconds", "request_seconds", "command_seconds", "cleanup_seconds", "http_requests", "response_bytes", "total_output_bytes", "output_tokens"):
        positive(limits[name], name, name in ("http_requests", "response_bytes", "total_output_bytes", "output_tokens"))
    need(all(limits["output_tokens"] <= row["max_output_tokens_per_attempt"] for row in budget["bindings"] if row["kind"] == "model"),
         "public/test output cap exceeds an approved model binding ceiling")
    vectors = copy.deepcopy(bindings["budget_vectors"])
    if not vectors and "protected_overflow" in bindings.get("native_policy_source_recipes", {}):
        vectors = protected_overflow_vectors(bindings, provider_rows)
    bindings["budget_vectors"] = vectors
    need(isinstance(vectors, list) and sum(row.get("kind") == "over_limit" for row in vectors) == 1
         and all(row.get("kind") in ("over_limit", "exact_boundary") for row in vectors)
         and sum(row.get("kind") == "exact_boundary" for row in vectors) <= 1,
         "actual protected-overflow vector is required; exact boundary cannot be fabricated")
    for row in vectors:
        request = row["request"]
        need(request.get("model") == provider_rows[0]["qualified_model"] and request.get("stream") is False
             and isinstance(request.get("messages"), list) and request["messages"], "budget vector must exercise the actual primary mounted request path")
        positive(request.get("max_completion_tokens", request.get("max_tokens")), "budget vector output ceiling", True)
    for name in ("index_seconds", "regeneration_seconds", "regeneration_documents"):
        positive(knowledge[name], "knowledge " + name, True)
    need(limits["cleanup_seconds"] > budget["upstream_timeout_seconds"] + budget["ingress_timeout_seconds"],
         "cleanup allowance must cover joined forwarding quiescence")
    for name in ("observer_seconds", "maintenance_seconds", "ready_seconds"):
        positive(bindings[name], name, True)
    need(bindings["observer_seconds"] <= 120 and bindings["ready_seconds"] <= 120,
         "observer/packet startup deadlines exceed the concrete owner contract")
    postgres_store = copy.deepcopy(bindings["postgres_store"])
    need(postgres_store.get("schema_version") == 1 and postgres_store.get("executable") is True,
         "actual owned PostgreSQL provisioning configuration is required")
    postgres_store.update(docker={"path":paths["docker"],"sha256":inventory[paths["docker"]]},
        psql={"path":paths["psql"],"sha256":inventory[paths["psql"]]},
        state_parent_env=bindings["backends"]["postgres_pgvector"]["state_parent_env"],
        host_port=prepared["ports"]["postgres"], **prepared["datastores"]["postgres"])
    ports.append(port(postgres_store["host_port"]))
    need(isinstance(postgres_store["image"], str) and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:/-]*@sha256:[0-9a-f]{64}", postgres_store["image"]),
         "owned PostgreSQL image must be an explicit immutable pgvector-capable binding")
    need(type(postgres_store["container_pg_port"]) is int and 1 <= postgres_store["container_pg_port"] <= 65535,
         "container PostgreSQL port must be explicit")
    mount_target = Path(absolute(postgres_store["mount_target"]))
    need(Path(absolute(postgres_store["pgdata"])).is_relative_to(mount_target), "PostgreSQL data must be inside its declared owned mount")
    pg_limits = postgres_store["limits"]
    for field in ("command_seconds", "ready_seconds", "cleanup_seconds", "stop_seconds", "total_seconds", "output_bytes"):
        positive(pg_limits[field], "PostgreSQL " + field, True)
    need(pg_limits["total_seconds"] <= 110 and bindings["observer_seconds"] >= pg_limits["total_seconds"] + 5
         and limits["command_seconds"] >= bindings["observer_seconds"]
         and limits["cleanup_seconds"] >= bindings["observer_seconds"],
         "outer observation/cleanup allowances must cover the actual bounded PostgreSQL lifecycle owner")
    need(max(pg_limits[field] for field in ("ready_seconds", "cleanup_seconds", "command_seconds")) < pg_limits["total_seconds"]
         and pg_limits["stop_seconds"] < min(pg_limits["cleanup_seconds"], pg_limits["command_seconds"])
         and pg_limits["output_bytes"] <= 1048576, "PostgreSQL subcommand/stop limits exceed their owner budget")
    for key, variable in postgres_store["docker_environment_env"].items():
        env(key)
        env(variable)
    teams = copy.deepcopy(bindings["teams"])
    positive(teams["case_seconds"], "team case deadline", True)
    for name in ("tokens", "costMicrounits", "elapsedSeconds"):
        positive(teams["reservation"][name], "team " + name, True)
    for name in ("maxTokens", "maxCostMicrounits", "maxElapsedSeconds"):
        positive(teams["budget"][name], "team " + name, True)
    need(teams["budget"]["currency"] == "USD" and teams["model_binding"]["credentialRef"] and teams["storage_ref"], "explicit team binding metadata missing")
    need(isinstance(teams.get("workspace_id"), str) and teams["workspace_id"], "team workspace must match the actual prepared authority")
    for name in ("expected_principal_key", "expected_history_owner_key", "expected_subject"):
        need(isinstance(bindings["orchestrator"][name], str) and bindings["orchestrator"][name], "authenticated expected authority identity missing")
    browser = copy.deepcopy(bindings["browser"])
    browser.update(executable=paths["browser_executable"], playwright_entry=paths["playwright_entry"])
    static_directory = Path(absolute(browser["static_directory"]))
    static_files = sorted(path for path in inventory if Path(path).is_relative_to(static_directory))
    need({str(path) for path in static_directory.rglob("*") if path.is_file()} == set(static_files),
         "actual shipped static directory differs from its complete frozen closure")
    need(str(static_directory / "index.html") in static_files and browser["static_files"], "complete installed static closure including index is required")
    for row in browser["static_files"]:
        need(row["url"].startswith("/assets/") and inventory.get(absolute(row["path"])) == row["sha256"], "browser asset URL/identity differs")
    # The final inventory asserts the complete transitive installed closures;
    # entry-file hashes alone are never described as dependency closure.
    clients = {"python_sdk":[paths["python"], paths["python_sdk_entry"]],
               "typescript_sdk":[paths["node"], paths["typescript_sdk_entry"]],
               "rust_sdk":[paths["rust_sdk_binary"]],
               "grpc_legacy":[paths["grpc_test_binary"], "--exact", "installed_grpc_user_outcomes", "--nocapture"],
               "grpc_v1":[paths["grpc_test_binary"], "--exact", "installed_grpc_user_outcomes", "--nocapture"],
               "browser":[paths["node"], str(ROOT / "browser_client.mjs")]}
    need(bindings["native_configuration"].get("memory", {}).get("enabled") is True
         and bindings["native_configuration"]["memory"].get("auto_capture") is True,
         "full memory lifecycle/consent requires enabled owned embedded memory and auto capture")
    need(bindings["native_configuration"]["security"].get("jwt_required") is True
         and bindings["native_configuration"]["security"].get("settings_mutation_auth_required") is True,
         "actual authenticated settings/owner boundary must remain enabled")
    need(bindings["native_configuration"]["sessions"].get("persisted_ttl_secs", 0) > 0,
         "real startup maintenance TTL is required")
    cap = copy.deepcopy(bindings["capabilities"])
    if linux is not None:
        cap["negative_provider_port"] = port(prepared["ports"]["negative_401_peer"])
        ports.append(cap["negative_provider_port"])
        need({"scheme":"http", "host":"127.0.0.1", "port":cap["negative_provider_port"]}
             in linux["owned_endpoint_routes"], "Linux negative provider peer requires an exact prepared endpoint route")
    positive(cap["case_seconds"], "capability case deadline", True)
    cap.update(python_binary=paths["python"], tools_fixture_path=str(ROOT / "tools_fixture.py"))
    env(cap["foreign_authorization_env"])
    env(teams["foreign_authorization_env"])
    secret_env.update((cap["foreign_authorization_env"], teams["foreign_authorization_env"]))
    def configured_secrets(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key in ("api_key", "embedding_api_key", "jwt_secret", "settings_admin_key", "openai_api_key", "cohere_api_key", "anthropic_api_key", "mistral_api_key") and item is not None:
                    need(isinstance(item, str) and re.fullmatch(r"@env:[A-Z][A-Z0-9_]*@", item),
                         "native credentials must be whole-leaf protected environment references")
                configured_secrets(item)
        elif isinstance(value, list):
            for item in value:
                configured_secrets(item)
        elif isinstance(value, str) and value.startswith("@env:") and value.endswith("@"):
            variable = env(value[5:-1])
            if variable not in ("UAR_ACCEPTANCE_MEMORY_PATH", "UAR_ACCEPTANCE_TOOLS_EFFECTS_ROOT", "UAR_ACCEPTANCE_OWNED_STATE_DIR"):
                secret_env.add(variable)
    configured_secrets(bindings["native_configuration"])
    need(cap["structured_binding_model"] in {row["qualified_model"] for row in provider_rows}, "structured model binding is undeclared")
    structured = next(row for row in provider_rows if row["qualified_model"] == cap["structured_binding_model"])
    need(structured is not provider_rows[0], "structured metadata forwarding binding must be distinct from the boot primary")
    cap["genuine_upstream"] = structured["config"]["base_url"]
    ports.append(port(cap["genuine_proxy_port"]))
    structured["config"]["base_url"] = "http://127.0.0.1:" + str(cap["genuine_proxy_port"]) + "/v1"
    # This chained wire proxy needs the approved loopback budget hop contract;
    # no direct HTTPS route may evade the one persistent packet ledger.
    need(bindings.get("structured_proxy_budget_hop_approved") is True, "structured metadata proxy's budget hop requires explicit composition approval")
    need(set(bindings["backends"]) == set(BACKENDS), "both explicit backend bindings are mandatory")
    for binding in bindings["backends"].values():
        need(set(binding["headers_env"]) >= {"Authorization", "x-uar-admin-key"}, "backend authentication references are incomplete")
        secret_env.update(env(variable) for variable in binding["headers_env"].values())
    for key, variable in bindings["server_environment_env"].items():
        need(not key.startswith("UAR_") and key not in ("PORT", "CONFIG_FILE", "DATABASE_URL", "PGOPTIONS", "MCP_CONFIG_PATH", "MCP_CONFIG_DIR"),
             "server environment cannot override the rendered owned configuration")
        env(variable)
    for name in ("postgres-server-template.json", "surreal-server-template.json", "team-agent.json", "team-template.json", "team-binding.json"):
        need(str(ROOT / "fixtures" / name) in sources, "composition template must be frozen source")
    need(not output.exists(), "composition output must be a new exclusive directory")
    output.parent.resolve(strict=True)
    output.mkdir(mode=0o700)
    generated = {}
    def emit(name, value):
        path = output / name
        with path.open("xb") as handle:
            handle.write(canonical(value) + b"\n")
            handle.flush()
            os.fsync(handle.fileno())
        path.chmod(0o600)
        generated[str(path)] = sha(path.read_bytes())
        return str(path)
    budget_path = emit("budget-proxy.json", budget)
    service_path = emit("existing-service-observer.json", service)
    postgres_store_path = emit("postgres-store.json", postgres_store)
    cap["structured_proxy_budget_config"] = budget_path
    cap["structured_proxy_budget_hop_approved"] = True
    for name, source in (("agent_template", "team-agent.json"), ("team_template", "team-template.json"), ("binding_template", "team-binding.json")):
        original = ROOT / "fixtures" / source
        need(str(original) in sources, "team source template missing from source denominator")
        document = read(original)
        if name == "agent_template":
            descriptor = document["sourceDescriptor"]
            document["sourceIdentity"] = {"profile":"urn:prometheus:uar:agent-md:1.1", "id":descriptor["metadata"]["id"],
                "version":descriptor["metadata"]["version"], "digest":sha(canonical(descriptor)), "revision":None}
        document.pop("contentDigest", None)
        document["contentDigest"] = sha(canonical(document))
        teams[name] = emit(source, document)
    backend_files = {name: {key:str(output / (name + "-" + key + ".json")) for key in ("runtime", "startup", "server", "mcp")}
                     for name in BACKENDS}
    configuration_files = sorted([str(bindings_path), str(inventory_path), prepared_reference["path"],
                                 preparation_receipt_reference["path"], preparation_config_reference["path"], budget_path, service_path, postgres_store_path,
                                 *(teams[key] for key in ("agent_template", "team_template", "binding_template")),
                                 *(path for files in backend_files.values() for path in files.values())])
    profiles = []
    parent_paths = []
    runtime_hooks = {name:name for name in ("prepare", "ready", "admission", "inference", "session", "cleanup")}
    if linux is not None:
        from linux_runtime import route_endpoint
        runtime_hooks["runtime_stop"] = "stop-runtime"
    for name in BACKENDS:
        binding = bindings["backends"][name]
        files = backend_files[name]
        http_port, grpc_port = port(binding["http_port"]), port(binding["grpc_port"])
        ports.extend((http_port, grpc_port))
        parent_paths.append(str(private_parent(binding["state_parent"])))
        scope_env = env(binding["state_parent_env"])
        runtime_env, startup_env = env(binding["runtime_config_env"]), env(binding["startup_config_env"])
        environment_env = {scope_env:scope_env, "UAR_ACCEPTANCE_STATE_PARENT":scope_env,
                           "UAR_ACCEPTANCE_RUNTIME_CONFIG":runtime_env, "UAR_ACCEPTANCE_STARTUP_CONFIG":startup_env}
        for variable in secret_env:
            environment_env[variable] = variable
        for variable in bindings["server_environment_env"].values():
            environment_env[variable] = variable
        for key, variable in postgres_store["docker_environment_env"].items():
            environment_env[key] = variable
        postgres = copy.deepcopy(bindings["postgres"])
        postgres.update(psql_binary=paths["psql"], schema_scope="run_sha256_20")
        surreal = copy.deepcopy(bindings["surreal"])
        surreal["database_scope"] = "run_sha256_20"
        disk = surreal["disk_container"]
        disk.update(container_env="UAR_ACCEPTANCE_OWNED_SCOPE", docker_binary=paths["docker"])
        need("@sha256:" in disk["image"] and disk["args"], "Surreal image/daemon argv must be exact digest-pinned inputs")
        if name == "surreal_disk":
            ports.append(port(binding["surreal_http_port"]))
        for variable in (postgres["connection_env"], surreal["url_env"], surreal["username_env"], surreal["password_env"]):
            environment_env[env(variable)] = variable
        startup = {"schema_version":1, "state_dir_env":"UAR_ACCEPTANCE_STATE_PARENT",
                   "command_seconds":bindings["observer_seconds"], "maintenance_seconds":bindings["maintenance_seconds"],
                   "postgres":postgres, "surreal":{key:value for key, value in surreal.items() if key != "disk_container"}}
        slots = copy.deepcopy(bindings["native_configuration"])
        slots.update(http_port=http_port, grpc_port=grpc_port, model=provider_rows[0]["qualified_model"],
                     output_tokens=limits["output_tokens"], provider_config=provider_rows[0]["config"],
                     vector_dimension=a["dimension"])
        server = replace_slots(read(ROOT / "fixtures" / ("postgres-server-template.json" if name == "postgres_pgvector" else "surreal-server-template.json")), slots)
        server["llm"]["model"] = provider_rows[0]["qualified_model"]
        server.setdefault("models", {})["models_dir"] = str(local_models_dir)
        server["llm"]["embedding"]["models_dir"] = str(local_models_dir)
        server["llm"]["base_url"] = provider_rows[0]["config"]["base_url"]
        server["llm"]["api_key_env"] = None
        server["llm"]["api_key"] = "@env:" + provider_rows[0]["credential_env"] + "@"
        server["llm"]["embedding"]["base_url"] = spaces[0]["endpoint"]
        need(len({row["endpoint"] for row in spaces}) == 1, "live embedding model changes require one approved multi-model endpoint binding")
        embedding_wire = next(row for row in budget["bindings"] if row["kind"] == "embedding"
                              and a["model"] in row["allowed_models"]
                              and "http://127.0.0.1:" + str(row["listen_port"]) + row["request_path"].rsplit("/", 1)[0] == spaces[0]["endpoint"])
        server["llm"]["embedding"].update(backend=a["backend"], model=a["model"], vector_dimension=a["dimension"],
            send_dimensions=False, api_key_env=None, api_key="@env:" + embedding_wire["ingress_authorization_env"] + "@")
        server["memory"].update(embedding_base_url="http://127.0.0.1:" + str(memory_wire["listen_port"]) + memory_wire["request_path"].rsplit("/", 1)[0],
            embedding_api_key="@env:" + memory_wire["ingress_authorization_env"] + "@")
        server["memory"].update(namespace="@scope@", database="@scope@", db_path="@env:UAR_ACCEPTANCE_MEMORY_PATH@", surreal_endpoint=None)
        server["native_tools"].update(file_tools_enabled=True, file_allowed_paths=["@env:UAR_ACCEPTANCE_TOOLS_EFFECTS_ROOT@"])
        mcp = {"mcpServers":{cap["mcp_server_name"]:{"command":paths["python"],
               "args":[str(ROOT / "tools_fixture.py"), "mcp", "@env:UAR_ACCEPTANCE_TOOLS_EFFECTS_ROOT@"],
               "env":{}, "sandboxed":False, "preapproved_tools":[]}}}
        if linux is not None:
            mcp["mcpServers"][cap["mcp_server_name"]]["command"] = linux["container_python"]
            accessible = [Path(linux["repo_root"]), *[Path(value) for value in linux["readonly_resource_roots"]]]
            need(all(any(value.is_relative_to(root) for root in accessible)
                     for value in (static_directory, local_models_dir, Path(paths["classifier_wasm"]))),
                 "Linux static, embedding or WASM resources lack approved same-path readonly mounts")
            need(all(not Path(binding["state_parent"]).is_relative_to(root)
                     and not root.is_relative_to(Path(binding["state_parent"])) for root in accessible),
                 "Linux writable owned state must be disjoint from readonly resource roots")
        if name == "postgres_pgvector":
            lifecycle = {"prepare_argv":[paths["python"],str(ROOT / "stores_fixture.py"),"postgres-prepare",postgres_store_path],
                         "cleanup_argv":[paths["python"],str(ROOT / "stores_fixture.py"),"postgres-cleanup",postgres_store_path]}
        else:
            lifecycle = {"prepare_argv":[paths["docker"], "run", "--detach", "--pull=never", "--name", "@scope@",
                         "--cidfile", "@env:UAR_ACCEPTANCE_SURREAL_CID_FILE@",
                         "--label", "@env:UAR_ACCEPTANCE_OWNED_LABEL@",
                         "--label", "@env:UAR_ACCEPTANCE_OWNED_RUN_LABEL@",
                         "--label", "@env:UAR_ACCEPTANCE_OWNED_NONCE_LABEL@",
                         "--label", "@env:UAR_ACCEPTANCE_OWNED_BACKEND_LABEL@",
                         "--volume", "@env:UAR_ACCEPTANCE_SURREAL_VOLUME@",
                         "--publish", "127.0.0.1:" + str(binding["surreal_http_port"]) + ":8000", disk["image"], *disk["args"]],
                         "cleanup_argv":[paths["docker"], "rm", "--force", "@env:UAR_ACCEPTANCE_SURREAL_CONTAINER_ID@"]}
        runtime = {"schema_version":1, "executable":True, "state_dir_env":"UAR_ACCEPTANCE_STATE_PARENT",
                   "command_seconds":bindings["observer_seconds"], "ready_seconds":bindings["ready_seconds"],
                   "server_binary":paths["server_binary"], "server_template":files["server"], "mcp_template":files["mcp"],
                   "static_directory":str(static_directory), "static_files":static_files, "team_execution_max_active":3,
                   "source_files":sources, "configuration_files":configuration_files,
                   "server_environment_env":bindings["server_environment_env"], "approved_genuine_origins":origins,
                   "budget_proxy":{"config_path":budget_path,"provider_bindings":provider_budget_bindings},
                   "schema_contract":schema, "postgres":postgres, "surreal":surreal,
                   "store_lifecycle":{name:lifecycle}}
        if name == "postgres_pgvector":
            runtime["postgres_store"] = {"config_path":postgres_store_path}
        if linux is not None:
            runtime.update(runtime_platform="linux-container", linux_runtime=linux, linux_ports=[http_port, grpc_port])
            runtime["approved_genuine_origins"] = [{**row, "origin":route_endpoint(row["origin"], linux)} for row in origins]
        for key, value in (("startup", startup), ("server", server), ("mcp", mcp), ("runtime", runtime)):
            emit(name + "-" + key + ".json", value)
        observers = {hook:[paths["python"], str(ROOT / "observers_runtime.py"), action] for hook, action in runtime_hooks.items()}
        observers.update({hook:[paths["python"], str(ROOT / "observers_startup.py"), action] for hook, action in
                          (("startup_initial_capture", "capture"), ("startup_initial", "initial"), ("startup_maintenance", "maintenance"), ("startup_maintenance_read", "read"))})
        observers["orchestrator_run"] = [paths["python"], str(ROOT / "observers_orchestrator.py"), "run"]
        observers["skill_retention"] = [paths["python"], str(ROOT / "observers_skill_retention.py"), "run"]
        for hook, action in (("strategy_run", "run"), ("strategy_budget", "budget"), ("strategy_session", "session"),
                             ("strategy_skill_scope", "skill-scope"), ("strategy_seed_discovery", "seed-discovery"), ("strategy_discovery", "discovery")):
            observers[hook] = [paths["python"], str(ROOT / "observers_strategies.py"), action]
        for hook in ("knowledge", "teams"):
            observers[hook] = [paths["python"], str(ROOT / ("observers_" + hook + ".py"))]
        for action in ("prepare_effect", "effect", "run_evidence", "provider_ready", "store_fault", "memory", "image", "isolation", "wire"):
            observers["cap_" + action] = [paths["python"], str(ROOT / "observers_capabilities.py"), action]
        profiles.append({"name":name, "profile_id":binding["profile_id"], "durable":True,
                         "base_url":"http://127.0.0.1:" + str(http_port), "grpc_endpoint":"http://127.0.0.1:" + str(grpc_port),
                         "headers_env":binding["headers_env"], "environment_env":environment_env,
                         "server":{"argv":[paths["python"], str(ROOT / "observers_runtime.py"), "serve"]},
                         "observers":observers, "clients":clients})
        if linux is not None:
            profiles[-1]["runtime_platform"] = "linux-container"
    need(len(set(ports)) == len(ports), "all declared owned listener ports must be distinct")
    need(set(ports).issubset(set(prepared["ports"].values())), "every owned listener must belong to the exact private preparation port inventory")
    need(len(set(parent_paths)) == 2 and all(not Path(a).is_relative_to(Path(b)) for a in parent_paths for b in parent_paths if a != b),
         "backend state parents must be distinct and disjoint")
    files = {**inventory, **generated, str(bindings_path):sha(bindings_path.read_bytes()), str(inventory_path):sha(inventory_path.read_bytes())}
    configuration_revision = sha(canonical([{"path":path, "sha256":files[path]} for path in configuration_files]))
    if linux is not None:
        for row in provider_rows:
            row["config"]["base_url"] = route_endpoint(row["config"]["base_url"], linux)
        for row in spaces:
            row["endpoint"] = route_endpoint(row["endpoint"], linux)
    fixture = {"schema_version":1, "executable":True, "source_identity":source_identity,
               "artifact_identity":artifact["sha256"], "configuration_revision":configuration_revision,
               "schema_revision":sha(canonical(schema)), "positive_repetitions":3, "modules":manifest["modules"],
               "required_cases":manifest["required_cases"], "limits":limits, "provider":provider_rows[0],
               "genuine_bindings":provider_rows[1:], "secret_env":sorted(secret_env), "backends":profiles,
               "knowledge":knowledge, "orchestrator":bindings["orchestrator"], "teams":teams, "browser":browser,
               "capabilities":cap, "strategies":{"wasm_component_path":paths["classifier_wasm"], "budget_vectors":bindings["budget_vectors"]},
               "interfaces":{"acp_path":"/acp"}, "budget_proxy":{"config":budget_path, "epoch_id":budget["epoch_id"]},
               "packet_processes":{"existing_service":{"argv":[paths["python"], str(ROOT / "observers_service.py"), service_path],
                    "environment_env":{"UAR_ACCEPTANCE_STATE_PARENT":bindings["budget_state_parent_env"],
                        **bindings["existing_service_environment_env"]}},
                    "budget_proxy":{"argv":[paths["python"], str(ROOT / "budget_proxy.py"), budget_path],
                    "environment_env":{"UAR_ACCEPTANCE_STATE_PARENT":bindings["budget_state_parent_env"],
                        **{variable:variable for variable in secret_env}},
                    "epoch_id":budget["epoch_id"], "listen_ports":[row["listen_port"] for row in budget["bindings"]],
                    "ready_seconds":bindings["ready_seconds"]}},
               "frozen_files":[{"path":path, "sha256":identity} for path, identity in sorted(files.items())]}
    if linux is not None:
        fixture.update(runtime_platform="linux-container", linux_runtime=linux)
    if "failure_sweep" in bindings:
        sweep = copy.deepcopy(bindings["failure_sweep"])
        need(sweep.get("schema_version") == 1 and isinstance(sweep.get("prerequisites"), dict)
             and set(sweep["prerequisites"]) == set(manifest["required_cases"]),
             "failure sweep requires the full exact frozen case prerequisite map")
        if "correction_authority" in sweep:
            reference(sweep["correction_authority"], inventory)
        fixture["failure_sweep"] = sweep
    emit("fixture.json", fixture)
    # Values are non-secret path bindings only, for explicit adoption by the
    # caller. No env file is sourced or arbitrary executable input interpreted.
    adoption = {"schema_version":1, "status":"COMPOSED_NOT_EXECUTED_NOT_GATE_AUTHORITY",
                "fixture":str(output / "fixture.json"), "case_count":case_count, "planned_rows":planned_rows,
                "private_environment_file":preparation_receipt["env_path"],
                "private_environment_adoption":"load in parent memory only under exact approval; never source or print values",
                "required_environment_values":{}}
    for name in BACKENDS:
        binding = bindings["backends"][name]
        adoption["required_environment_values"].update({binding["state_parent_env"]:binding["state_parent"],
            binding["runtime_config_env"]:backend_files[name]["runtime"], binding["startup_config_env"]:backend_files[name]["startup"]})
    adoption["required_environment_values"][bindings["surreal"]["url_env"]] = "http://127.0.0.1:" + str(bindings["backends"]["surreal_disk"]["surreal_http_port"])
    adoption["required_environment_values"][env(bindings["budget_state_parent_env"])] = str(ledger_parent)
    emit("adoption.json", adoption)
    emit("derived-input-receipt.json", {"schema_version":1, "status":"DERIVED_INPUTS_NOT_RUNTIME_ACCEPTANCE",
        "source_identity":source_identity, "artifact_identity":artifact["sha256"],
        "operator_approval":bindings["operator_approval"], "operator_signature":bindings["operator_signature"],
        "outputs":[{"path":path,"sha256":identity} for path, identity in sorted(generated.items())],
        "self_reference":False, "final_gate_adoption":"external; never insert this receipt or its future signature into fixture inputs"})
    return adoption


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bindings", required=True, type=Path)
    parser.add_argument("--inventory", type=Path)
    parser.add_argument("--output", required=True, type=Path)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--compose", action="store_true")
    mode.add_argument("--propose", action="store_true")
    args = parser.parse_args()
    need(args.output.is_absolute(), "output must be an absolute new directory")
    need(args.bindings.is_absolute(), "bindings must name an absolute source input")
    if args.propose:
        need(args.inventory is None, "source proposal mode does not inspect a final artifact inventory")
        result = propose(args.bindings, args.output)
    else:
        need(args.inventory is not None and args.inventory.is_absolute(), "composition requires the actual absolute final inventory")
        result = compose(args.bindings, args.inventory, args.output)
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(json.dumps({"status":"BLOCKED", "error_type":type(error).__name__,
                          "detail":str(error) if isinstance(error, Incomplete) else "composition failed; input details withheld"}))
        sys.exit(2)
