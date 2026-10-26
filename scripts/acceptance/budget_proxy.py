"""Exclusive epoch-owned forwarding and conservative packet reservations.

No process, ledger or network operation occurs on import. This program requires
an exact operator-approved executable configuration; example inputs never run.
Reservations are conservative approved exposure, NOT verified actual spend.
"""
from __future__ import annotations

from contextlib import contextmanager
import fcntl
import hashlib
import hmac
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import math
import os
from pathlib import Path
import re
import signal
import socket
import sqlite3
import sys
import threading
import time
import urllib.error
import urllib.request
from urllib.parse import urlsplit
import uuid


COUNTERS = ("model_attempts", "embedding_attempts", "generated_tokens", "cost_microunits", "request_bytes", "response_bytes")
SPEND_POOL_MAXIMA = {"cohere": 10_000_000, "openai_embeddings": 10_000_000, "liter": 0}
MAX_CONFIG_BYTES = 1024 * 1024


class Refusal(RuntimeError):
    """Closed diagnostic; never contains caller/upstream content or credentials."""


def need(condition, code):
    if not condition:
        raise Refusal(code)


def integer(value, code, positive=False):
    need(type(value) is int and (value > 0 if positive else value >= 0), code)
    return value


def sha(raw):
    return "sha256:" + hashlib.sha256(raw).hexdigest()


def terminal_receipt_error(state, receipt, *, epoch_id, input_identity, binding_id,
                           kind, max_response_bytes, max_output_tokens):
    """Pure admission check; return the caller's accounting/usage error category.

    Bounds come from the caller's approved binding and actual reservation. This
    never settles a ledger, releases a reservation, or establishes useful success.
    """
    if not isinstance(receipt, dict):
        return "accounting"
    status, size = receipt.get("http_status"), receipt.get("response_bytes")
    stamps = [receipt.get(name) for name in ("reserved_at_unix",
        "outbound_started_unix", "outbound_finished_unix", "settled_at_unix")]
    response_hash = receipt.get("response_sha256")
    if not (state in ("completed", "failed") and kind in ("model", "embedding")
            and receipt.get("epoch_id") == epoch_id
            and receipt.get("input_identity") == input_identity
            and receipt.get("binding_id") == binding_id and receipt.get("kind") == kind
            and type(status) is int and 100 <= status <= 599
            and (200 <= status < 300) == (state == "completed")
            and type(max_response_bytes) is int and max_response_bytes >= 0
            and type(size) is int and 0 <= size <= max_response_bytes
            and isinstance(response_hash, str)
            and re.fullmatch(r"sha256:[0-9a-f]{64}", response_hash)
            and all((type(value) is int and value > 0)
                    or (type(value) is float and math.isfinite(value) and value > 0)
                    for value in stamps)):
        return "accounting"
    if stamps != sorted(stamps):
        return "accounting"
    if state == "completed":
        usage = receipt.get("actual_usage")
        key = "output_tokens" if kind == "model" else "input_or_total_tokens"
        value = usage.get(key) if isinstance(usage, dict) else None
        if not (type(value) is int and value >= 0
                and (kind != "model" or (type(max_output_tokens) is int
                     and 0 <= value <= max_output_tokens))):
            return "usage"
    return None


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()


def private_file(raw_path, maximum=MAX_CONFIG_BYTES):
    path = Path(raw_path)
    need(path.is_absolute() and path.is_file() and not path.is_symlink(), "input_file_unavailable")
    need(path.stat().st_mode & 0o077 == 0 and path.stat().st_size <= maximum, "input_file_not_private_or_bounded")
    return path.read_bytes()


def approved_input(reference):
    need(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "approval_reference_invalid")
    raw = private_file(reference["path"])
    need(sha(raw) == reference["sha256"], "approval_input_digest_changed")
    return raw


def bound_source(reference):
    """Hash existing service source files without logging their contents."""
    need(isinstance(reference, dict) and set(reference) == {"path", "sha256"}, "service_source_reference_invalid")
    path = Path(reference["path"])
    need(path.is_absolute() and path.is_file() and not path.is_symlink()
         and path.stat().st_size <= MAX_CONFIG_BYTES, "service_source_unavailable_or_unbounded")
    need(sha(path.read_bytes()) == reference["sha256"], "service_source_digest_changed")


def upstream_authority(config, binding):
    endpoint = urlsplit(binding["upstream_url"])
    need(bool(endpoint.hostname) and endpoint.username is None and endpoint.password is None
         and not endpoint.query and not endpoint.fragment, "upstream_endpoint_invalid")
    authority = binding.get("upstream_authority")
    if endpoint.scheme == "https":
        need(authority is None, "https_authority_must_use_approved_endpoint_binding")
        return "https-endpoint"
    expected_path = "/v1/chat/completions" if binding.get("kind") == "model" else "/v1/embeddings"
    need(endpoint.scheme == "http" and endpoint.hostname in ("127.0.0.1", "localhost")
         and endpoint.port == 4000 and endpoint.path == expected_path, "unapproved_cleartext_upstream")
    need(isinstance(authority, dict) and authority.get("kind") == "existing-service-loopback"
         and authority.get("service_id") == "ai.prometheus.liter-llm-api"
         and authority.get("operator_approved") is True
         and authority.get("endpoint_url") == binding["upstream_url"], "configured_service_authority_missing")
    for name in ("launchagent", "launch_script", "service_config"):
        bound_source(authority[name])
    approved = json.loads(approved_input(authority["approval"]))
    expected = {"kind": authority["kind"], "service_id": authority["service_id"], "operator_approved": True,
                "endpoint_url": binding["upstream_url"], "epoch_id": config["epoch_id"], "binding_id": binding["id"],
                "header_prefixes": binding.get("header_prefixes", {}),
                **{name: authority[name] for name in ("launchagent", "launch_script", "service_config")}}
    need(approved == expected, "configured_service_approval_scope_differs")
    return "existing-service-loopback"


def secret(name):
    need(isinstance(name, str) and bool(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name)), "secret_reference_invalid")
    value = os.environ.get(name)
    need(isinstance(value, str) and bool(value) and "\r" not in value and "\n" not in value,
         "named_secret_unavailable")
    return value


def pointer(value, path):
    need(isinstance(path, str) and path.startswith("/") and "~" not in path, "token_path_invalid")
    for piece in path[1:].split("/"):
        if not isinstance(value, dict) or piece not in value:
            return None
        value = value[piece]
    return value


def validate_config(raw):
    config = json.loads(raw)
    need(config.get("schema_version") == 1 and config.get("executable") is True, "budget_config_not_executable")
    need(isinstance(config.get("epoch_id"), str) and 0 < len(config["epoch_id"]) <= 128, "epoch_id_invalid")
    approval = approved_input(config["approval_packet"])
    signature = approved_input(config["approval_signature"])
    seed_raw = approved_input(config["seed"])
    seed = json.loads(seed_raw)
    # This checks exact adopted inputs and binding, not cryptographic validity.
    # Signature verification/adoption occurs in the separately named packet.
    need(seed.get("epoch_id") == config["epoch_id"]
         and seed.get("approval_packet_sha256") == sha(approval)
         and seed.get("approval_signature_sha256") == sha(signature), "seed_approval_binding_differs")
    need(isinstance(seed.get("prior_epoch_id"), str) and bool(seed["prior_epoch_id"])
         and isinstance(seed.get("prior_consumption_evidence"), list) and seed["prior_consumption_evidence"],
         "prior_consumption_reconciliation_missing")
    for reference in seed["prior_consumption_evidence"]:
        approved_input(reference)
    need(isinstance(seed.get("unresolved_reservations"), list), "prior_unknown_inventory_missing")
    limits = config["limits"]
    usage = seed["consumed"]
    need(set(usage) == set(COUNTERS), "seed_counters_incomplete")
    for field in COUNTERS:
        integer(limits.get(field), "limit_invalid")
        integer(usage[field], "seed_counter_invalid")
        need(usage[field] <= limits[field], "seed_already_exceeds_approved_limit")
    pools = config.get("spend_pools")
    prior_pools = seed.get("spend_pool_consumed_microunits")
    need(isinstance(pools, dict) and set(pools) == set(SPEND_POOL_MAXIMA)
         and isinstance(prior_pools, dict) and set(prior_pools) == set(SPEND_POOL_MAXIMA),
         "explicit_spend_pools_and_prior_allocations_required")
    for name, maximum in SPEND_POOL_MAXIMA.items():
        need(isinstance(pools[name], dict) and set(pools[name]) == {"limit_microunits"}, "spend_pool_limit_shape_invalid")
        cap = integer(pools[name]["limit_microunits"], "spend_pool_limit_invalid")
        prior = integer(prior_pools[name], "prior_spend_pool_allocation_invalid")
        need(prior <= cap <= maximum, "provider_spend_pool_limit_or_prior_exceeded")
    need(limits["cost_microunits"] <= 20_000_000
         and limits["cost_microunits"] <= sum(pool["limit_microunits"] for pool in pools.values())
         and sum(prior_pools.values()) == usage["cost_microunits"], "global_and_provider_spend_allocations_differ")
    integer(limits.get("max_in_flight"), "concurrency_limit_invalid", True)
    integer(limits.get("wall_seconds"), "wall_limit_invalid", True)
    integer(seed.get("epoch_started_unix"), "epoch_start_invalid", True)
    need(type(seed.get("deadline_unix")) is int
         and seed["epoch_started_unix"] < seed["deadline_unix"]
         and seed["deadline_unix"] - seed["epoch_started_unix"] <= limits["wall_seconds"], "absolute_epoch_deadline_invalid")
    integer(config.get("upstream_timeout_seconds"), "upstream_timeout_invalid", True)
    integer(config.get("ingress_timeout_seconds"), "ingress_timeout_invalid", True)
    need(config["upstream_timeout_seconds"] <= limits["wall_seconds"]
         and config["ingress_timeout_seconds"] <= config["upstream_timeout_seconds"], "timeout_exceeds_epoch_bound")
    need(isinstance(config.get("bindings"), list) and 0 < len(config["bindings"]) <= 64, "approved_bindings_missing_or_unbounded")
    ids, ports = set(), set()
    for binding in config["bindings"]:
        need(isinstance(binding.get("id"), str) and bool(binding["id"]) and binding["id"] not in ids, "binding_identity_invalid")
        ids.add(binding["id"])
        need(binding.get("kind") in ("model", "embedding"), "binding_kind_invalid")
        port = integer(binding.get("listen_port"), "binding_port_invalid", True)
        need(1024 <= port <= 65535 and port not in ports, "binding_listener_not_exclusive")
        ports.add(port)
        authority_kind = upstream_authority(config, binding)
        pool = binding.get("spend_pool")
        need(pool in SPEND_POOL_MAXIMA, "binding_spend_pool_missing")
        endpoint = urlsplit(binding["upstream_url"])
        if pool == "liter":
            need(binding["kind"] == "model" and authority_kind == "existing-service-loopback",
                 "zero_incremental_liter_pool_requires_selected_model_service")
        elif pool == "cohere":
            need(binding["kind"] == "embedding" and endpoint.scheme == "https"
                 and endpoint.hostname == "api.cohere.ai" and endpoint.port in (None, 443)
                 and endpoint.path == "/compatibility/v1/embeddings", "cohere_pool_endpoint_mismatch")
        else:
            need(binding["kind"] == "embedding" and endpoint.scheme == "https"
                 and endpoint.hostname == "api.openai.com" and endpoint.port in (None, 443)
                 and endpoint.path == "/v1/embeddings", "openai_embedding_pool_endpoint_mismatch")
        need(binding.get("request_path", "").startswith("/") and "?" not in binding["request_path"]
             and "#" not in binding["request_path"] and not binding["request_path"].startswith("//"),
             "binding_request_path_invalid")
        need(isinstance(binding.get("allowed_models"), list) and binding["allowed_models"]
             and all(isinstance(model, str) and model for model in binding["allowed_models"])
             and len(set(binding["allowed_models"])) == len(binding["allowed_models"]), "named_model_allowlist_invalid")
        need(binding.get("model_path") == "/model", "unsupported_model_selector")
        integer(binding.get("max_request_bytes"), "request_byte_bound_invalid", True)
        integer(binding.get("max_response_bytes"), "response_byte_bound_invalid", True)
        need(binding["max_request_bytes"] <= limits["request_bytes"]
             and binding["max_response_bytes"] <= limits["response_bytes"], "per_attempt_bytes_exceed_packet")
        ceiling = integer(binding.get("approved_charge_ceiling_microunits"), "approved_charge_ceiling_missing")
        need(ceiling == 0 if pool == "liter" else ceiling > 0, "charge_ceiling_does_not_match_provider_pool")
        charge = json.loads(approved_input(binding["charge_approval"]))
        need(charge.get("spend_pool") == pool
             and charge.get("spend_pool_limit_microunits") == pools[pool]["limit_microunits"],
             "charge_approval_spend_pool_differs")
        prefixes = binding.get("header_prefixes", {})
        need(isinstance(prefixes, dict) and all(name == "Authorization" and prefix == "Bearer "
             and name in binding.get("headers_env", {}) for name, prefix in prefixes.items()),
             "unapproved_header_prefix")
        if prefixes:
            need(charge.get("header_prefixes") == prefixes, "header_prefix_charge_scope_differs")
        if authority_kind == "existing-service-loopback":
            need(charge.get("upstream_authority_kind") == authority_kind, "configured_service_charge_approval_missing")
        need(charge.get("epoch_id") == config["epoch_id"] and charge.get("binding_id") == binding["id"]
             and charge.get("upstream_url") == binding["upstream_url"]
             and charge.get("allowed_models") == binding["allowed_models"]
             and charge.get("charge_ceiling_microunits") == ceiling
             and charge.get("max_request_bytes") == binding["max_request_bytes"]
             and charge.get("max_response_bytes") == binding["max_response_bytes"], "charge_approval_scope_differs")
        need(charge.get("operator_approved") is True
             and charge.get("basis") == ("full-context-plus-output-ceiling" if binding["kind"] == "model" else "embedding-request-ceiling")
             and isinstance(charge.get("pricing_evidence"), str) and bool(charge["pricing_evidence"]), "charge_approval_basis_missing")
        need(ceiling > 0 or charge.get("zero_charge_explicitly_approved") is True, "zero_charge_not_explicitly_approved")
        if binding["kind"] == "model":
            integer(binding.get("max_output_tokens_per_attempt"), "output_token_bound_missing", True)
            need(charge.get("max_output_tokens_per_attempt") == binding["max_output_tokens_per_attempt"]
                 and type(charge.get("full_context_tokens")) is int and charge["full_context_tokens"] > 0,
                 "full_context_charge_approval_missing")
            need(isinstance(binding.get("output_token_paths"), list) and binding["output_token_paths"], "output_token_paths_missing")
            for path in binding["output_token_paths"]:
                pointer({}, path)
        need(isinstance(binding.get("headers_env"), dict) and binding["headers_env"], "upstream_headers_references_missing")
        for name, reference in binding["headers_env"].items():
            need(re.fullmatch(r"[A-Za-z0-9-]+", name) and name.lower() not in ("host", "content-length", "transfer-encoding"),
                 "upstream_header_invalid")
            secret(reference)
        need(binding.get("ingress_authorization_prefix", "") in ("", "Bearer "), "ingress_prefix_invalid")
        secret(binding["ingress_authorization_env"])
    identity = sha(raw + b"\0" + seed_raw + b"\0" + approval + b"\0" + signature)
    return config, seed, identity


class Ledger:
    """One daemon owns one epoch. A stale in-flight reservation blocks restart."""
    def __init__(self, config, seed, identity):
        self.config, self.seed, self.identity = config, seed, identity
        self.mutex = threading.RLock()
        self.accepting = True
        self.nonce = uuid.uuid4().hex
        root = Path(config["ledger_root"])
        expected = "uar_budget_" + hashlib.sha256(config["epoch_id"].encode()).hexdigest()[:20]
        need(root.is_absolute() and root.name == expected and not root.is_symlink()
             and root.parent.is_dir() and not root.parent.is_symlink()
             and root.parent.stat().st_mode & 0o077 == 0, "ledger_parent_not_exact_private_epoch_scope")
        root.mkdir(mode=0o700, exist_ok=True)
        need(root.is_dir() and root.stat().st_mode & 0o077 == 0, "ledger_root_not_private")
        self.root = root
        lock_path = root / "owner.lock"
        self.lock = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
        try:
            fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            os.close(self.lock)
            raise Refusal("budget_epoch_already_has_live_owner") from error
        path = root / "ledger.sqlite3"
        need(not path.is_symlink(), "ledger_path_is_symlink")
        exists = path.exists()
        self.db = sqlite3.connect(str(path), isolation_level=None, check_same_thread=False, timeout=10)
        os.chmod(path, 0o600)
        self.db.execute("PRAGMA synchronous=FULL")
        if not exists:
            with self.transaction():
                self.db.execute("CREATE TABLE meta (key TEXT PRIMARY KEY,value TEXT NOT NULL)")
                self.db.execute("CREATE TABLE reservations (request_id TEXT PRIMARY KEY,binding_id TEXT NOT NULL,kind TEXT NOT NULL,"
                                "state TEXT NOT NULL,reserved TEXT NOT NULL,receipt TEXT NOT NULL)")
                self.put("identity", identity)
                self.put("epoch_id", config["epoch_id"])
                self.put("seed", seed)
                self.put("totals", dict(seed["consumed"]))
                self.put("spend_pool_totals", dict(seed["spend_pool_consumed_microunits"]))
                self.put("blocked", "prior_unknown_reservations" if seed["unresolved_reservations"] else None)
        else:
            need(self.get("identity") == identity and self.get("epoch_id") == config["epoch_id"],
                 "existing_epoch_inputs_changed_no_reset_permitted")
            with self.transaction():
                stale = self.db.execute("SELECT request_id FROM reservations WHERE state='reserved'").fetchall()
                if stale:
                    self.db.execute("UPDATE reservations SET state='unknown' WHERE state='reserved'")
                    self.put("blocked", "prior_process_left_inflight_reservation")
        pool_totals = self.get("spend_pool_totals")
        need(isinstance(pool_totals, dict) and set(pool_totals) == set(SPEND_POOL_MAXIMA)
             and all(type(value) is int and 0 <= value <= config["spend_pools"][name]["limit_microunits"]
                     for name, value in pool_totals.items())
             and sum(pool_totals.values()) == self.get("totals")["cost_microunits"],
             "existing_provider_spend_accounting_missing_or_inconsistent_no_reset")
        self.put("owner", {"pid": os.getpid(), "nonce": self.nonce})

    @contextmanager
    def transaction(self):
        with self.mutex:
            self.db.execute("BEGIN IMMEDIATE")
            try:
                yield
                self.db.execute("COMMIT")
            except BaseException:
                self.db.execute("ROLLBACK")
                raise

    def get(self, key):
        row = self.db.execute("SELECT value FROM meta WHERE key=?", (key,)).fetchone()
        need(row is not None, "ledger_metadata_missing")
        return json.loads(row[0])

    def put(self, key, value):
        self.db.execute("INSERT INTO meta(key,value) VALUES (?,?) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
                        (key, canonical(value).decode()))

    def remaining_seconds(self):
        now = time.time()
        need(now >= self.seed["epoch_started_unix"], "epoch_not_started")
        remaining = self.seed["deadline_unix"] - now
        need(remaining > 0, "epoch_wall_allowance_exhausted")
        return remaining

    def reserve(self, binding, body, request_bytes, request_wire_sha256):
        need(isinstance(body, dict) and pointer(body, binding["model_path"]) in binding["allowed_models"], "model_not_approved")
        if binding["spend_pool"] == "cohere":
            need(set(body).issubset({"model", "input", "encoding_format"}), "cohere_text_request_fields_not_approved")
            inputs = body.get("input")
            texts = [inputs] if isinstance(inputs, str) else inputs
            need(isinstance(texts, list) and 0 < len(texts) <= 96
                 and all(isinstance(text, str) and bool(text) for text in texts),
                 "cohere_text_batch_not_approved")
        output = 0
        if binding["kind"] == "model":
            outputs = [pointer(body, path) for path in binding["output_token_paths"] if pointer(body, path) is not None]
            need(outputs and all(type(value) is int and value > 0 for value in outputs), "explicit_output_ceiling_missing")
            output = max(outputs)
            need(output <= binding["max_output_tokens_per_attempt"], "per_attempt_output_ceiling_exceeded")
            need(type(body.get("n", 1)) is int and body.get("n", 1) == 1 and not body.get("best_of"), "multiple_generation_not_approved")
        delta = dict.fromkeys(COUNTERS, 0)
        delta["model_attempts" if binding["kind"] == "model" else "embedding_attempts"] = 1
        delta["generated_tokens"] = output
        delta["cost_microunits"] = binding["approved_charge_ceiling_microunits"]
        delta["request_bytes"] = request_bytes
        delta["response_bytes"] = binding["max_response_bytes"]
        with self.transaction():
            need(self.accepting, "budget_admission_closed")
            self.remaining_seconds()
            need(self.get("blocked") is None, "epoch_has_unknown_accounting")
            totals = self.get("totals")
            need(all(totals[key] + delta[key] <= self.config["limits"][key] for key in COUNTERS), "packet_allowance_exhausted")
            pool, pool_totals = binding["spend_pool"], self.get("spend_pool_totals")
            need(pool_totals[pool] + delta["cost_microunits"] <= self.config["spend_pools"][pool]["limit_microunits"],
                 "provider_spend_pool_exhausted")
            request_id = uuid.uuid4().hex
            self.put("totals", {key: totals[key] + delta[key] for key in COUNTERS})
            pool_totals[pool] += delta["cost_microunits"]
            self.put("spend_pool_totals", pool_totals)
            receipt = {"epoch_id": self.config["epoch_id"], "owner_nonce": self.nonce,
                       "owner_pid": os.getpid(), "input_identity": self.identity,
                       "binding_id": binding["id"], "kind": binding["kind"],
                       "spend_pool": pool, "reserved_charge_microunits": delta["cost_microunits"],
                       "spend_pool_total_after_microunits": pool_totals[pool],
                       "request_path": binding["request_path"], "upstream_url": binding["upstream_url"],
                       "upstream_authority_kind": "existing-service-loopback" if binding.get("upstream_authority") else "https-endpoint",
                       "upstream_authority": binding.get("upstream_authority"),
                       "request_wire_sha256": request_wire_sha256,
                       "request_semantic_sha256": sha(canonical(body)), "model": body["model"],
                       "reserved_at_unix": time.time(), "actual_usage": None}
            self.db.execute("INSERT INTO reservations VALUES (?,?,?,?,?,?)",
                            (request_id, binding["id"], binding["kind"], "reserved", canonical(delta).decode(), canonical(receipt).decode()))
            return request_id

    def outbound(self, request_id):
        with self.transaction():
            row = self.db.execute("SELECT state,receipt FROM reservations WHERE request_id=?", (request_id,)).fetchone()
            need(row and row[0] == "reserved", "outbound_reservation_missing")
            receipt = json.loads(row[1])
            need("outbound_started_unix" not in receipt, "attempt_would_forward_twice")
            receipt["outbound_started_unix"] = time.time()
            self.db.execute("UPDATE reservations SET receipt=? WHERE request_id=?", (canonical(receipt).decode(), request_id))

    def settle(self, request_id, state, receipt):
        with self.transaction():
            row = self.db.execute("SELECT state,reserved,receipt FROM reservations WHERE request_id=?", (request_id,)).fetchone()
            need(row is not None and row[0] == "reserved", "reservation_settlement_conflict")
            reserved, prior = json.loads(row[1]), json.loads(row[2])
            prior.update(receipt)
            prior["settled_at_unix"] = time.time()
            if state == "unknown":
                self.put("blocked", "request_outcome_or_usage_unknown")
            elif state == "completed":
                # Counts/output/cost ceilings remain reserved. Only known unused
                # response bytes are released; no tariff/actual-spend inference.
                actual = receipt["response_bytes"]
                need(0 <= actual <= reserved["response_bytes"], "actual_response_exceeded_reserved_bytes")
                totals = self.get("totals")
                totals["response_bytes"] -= reserved["response_bytes"] - actual
                self.put("totals", totals)
            self.db.execute("UPDATE reservations SET state=?,receipt=? WHERE request_id=?",
                            (state, canonical(prior).decode(), request_id))

    def snapshot(self):
        with self.mutex:
            return {"epoch_id": self.config["epoch_id"], "input_identity": self.identity,
                    "totals_reserved": self.get("totals"), "blocked": self.get("blocked"),
                    "spend_pool_reserved_microunits": self.get("spend_pool_totals"),
                    "spend_pool_limits_microunits": {name: value["limit_microunits"] for name, value in self.config["spend_pools"].items()},
                    "meaning": "operator-approved conservative reservations, not verified actual spend"}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def owned_hop(config_path, upstream_base):
    """Read the running epoch's owned ready receipt; never start a listener."""
    raw = private_file(config_path)
    config, _seed, identity = validate_config(raw)
    bindings = [binding for binding in config["bindings"]
                if binding["kind"] == "model" and binding["request_path"] == "/v1/chat/completions"
                and upstream_base == "http://127.0.0.1:" + str(binding["listen_port"]) + "/v1"]
    need(len(bindings) == 1, "metadata_hop_not_exact_approved_budget_binding")
    root = Path(config["ledger_root"])
    need(root.is_dir() and not root.is_symlink() and root.stat().st_mode & 0o077 == 0,
         "budget_hop_ledger_not_private")
    path = root / "ledger.sqlite3"
    need(path.is_file() and not path.is_symlink(), "budget_hop_ledger_unavailable")
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True) as db:
        meta = {key: json.loads(value) for key, value in db.execute("SELECT key,value FROM meta")}
    need(meta.get("identity") == identity and meta.get("epoch_id") == config["epoch_id"]
         and meta.get("blocked") is None, "budget_hop_identity_or_accounting_differs")
    owner = meta["owner"]
    need(type(owner.get("pid")) is int and owner["pid"] > 0
         and isinstance(owner.get("nonce"), str) and re.fullmatch(r"[a-f0-9]{32}", owner["nonce"]),
         "budget_hop_owner_invalid")
    ready = json.loads(private_file(str(root / ("ready-" + owner["nonce"] + ".json"))))
    need(ready.get("pid") == owner["pid"] and ready.get("nonce") == owner["nonce"]
         and ready.get("config_sha256") == sha(raw)
         and ready.get("epoch", {}).get("input_identity") == identity
         and bindings[0]["listen_port"] in ready.get("listeners", []), "budget_hop_ready_receipt_differs")
    return config, bindings[0], ready


def reported_usage(raw, content_type):
    """Read supported usage envelopes; absent usage is explicitly unknown."""
    candidates = []
    if "text/event-stream" in content_type:
        for line in raw.splitlines():
            if not line.startswith(b"data:"):
                continue
            data = line[5:].strip()
            if data == b"[DONE]":
                continue
            try:
                value = json.loads(data)
            except (ValueError, UnicodeDecodeError):
                continue
            if isinstance(value, dict) and isinstance(value.get("usage"), dict):
                candidates.append(value["usage"])
            elif isinstance(value, dict) and isinstance(value.get("message"), dict) and isinstance(value["message"].get("usage"), dict):
                candidates.append(value["message"]["usage"])
    else:
        try:
            value = json.loads(raw)
            if isinstance(value, dict) and isinstance(value.get("usage"), dict):
                candidates.append(value["usage"])
            if isinstance(value, dict) and isinstance(value.get("meta"), dict):
                # Cohere's native Embed response reports actual input usage here.
                # https://docs.cohere.com/v1/reference/embed
                billed = value["meta"].get("billed_units")
                if isinstance(billed, dict) and type(billed.get("input_tokens")) is int:
                    candidates.append({"input_tokens": billed["input_tokens"]})
        except (ValueError, UnicodeDecodeError):
            pass
    output = [value for usage in candidates for key in ("completion_tokens", "output_tokens")
              if type(value := usage.get(key)) is int and value >= 0]
    inputs = [value for usage in candidates for key in ("prompt_tokens", "input_tokens", "total_tokens")
              if type(value := usage.get(key)) is int and value >= 0]
    return {"output_tokens": max(output) if output else None,
            "input_or_total_tokens": max(inputs) if inputs else None,
            "usage_observed": bool(candidates)}


def handler_class(ledger, binding, capacity):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect)
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.0"
        def log_message(self, *_):
            pass
        def refusal(self, status, code):
            raw = canonical({"error": {"code": code, "message": "packet forwarding refused; inspect the private epoch ledger"}})
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)
        def do_POST(self):
            if not capacity.acquire(blocking=False):
                self.refusal(429, "packet_concurrency_exhausted")
                return
            request_id = None
            response_started = False
            response_bytes = 0
            digest = hashlib.sha256()
            try:
                need(self.path == binding["request_path"], "unapproved_request_path")
                incoming = self.headers.get("Authorization", "")
                expected = binding.get("ingress_authorization_prefix", "") + secret(binding["ingress_authorization_env"])
                need(hmac.compare_digest(incoming, expected), "proxy_ingress_unauthorized")
                need(self.headers.get("Transfer-Encoding") is None, "chunked_ingress_not_approved")
                size = int(self.headers.get("Content-Length", "0"))
                need(0 < size <= binding["max_request_bytes"], "request_bytes_exceed_binding")
                self.connection.settimeout(min(ledger.config["ingress_timeout_seconds"], ledger.remaining_seconds()))
                raw = self.rfile.read(size)
                need(len(raw) == size, "request_body_incomplete")
                body = json.loads(raw)
                request_id = ledger.reserve(binding, body, len(raw), sha(raw))
                headers = {name: binding.get("header_prefixes", {}).get(name, "") + secret(reference)
                           for name, reference in binding["headers_env"].items()}
                headers["Content-Type"] = "application/json"
                request = urllib.request.Request(binding["upstream_url"], data=raw, headers=headers, method="POST")
                timeout = min(ledger.config["upstream_timeout_seconds"], ledger.remaining_seconds())
                ledger.outbound(request_id)
                try:
                    response = opener.open(request, timeout=timeout)
                except urllib.error.HTTPError as error:
                    response = error
                status = response.status
                content_type = response.headers.get("Content-Type", "application/json")
                # A canceled downstream waiter does not establish the upstream
                # outcome. Drain this already dispatched request under the same
                # byte/time reservation so authoritative usage can still settle.
                downstream_disconnected = False
                response_started = True
                try:
                    self.send_response(status)
                    self.send_header("Content-Type", content_type)
                    self.end_headers()
                except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError, socket.timeout):
                    downstream_disconnected = True
                    self.close_connection = True
                chunks = []
                with response:
                    while True:
                        ledger.remaining_seconds()
                        remaining_bytes = binding["max_response_bytes"] - response_bytes
                        need(remaining_bytes > 0, "response_byte_allowance_fully_consumed_without_confirmed_end")
                        chunk = response.read1(min(16384, remaining_bytes))
                        if not chunk:
                            break
                        response_bytes += len(chunk)
                        need(response_bytes <= binding["max_response_bytes"], "response_bytes_exceed_reserved_allowance")
                        chunks.append(chunk)
                        digest.update(chunk)
                        if not downstream_disconnected:
                            try:
                                self.wfile.write(chunk)
                                self.wfile.flush()
                            except (BrokenPipeError, ConnectionResetError, ConnectionAbortedError, socket.timeout):
                                downstream_disconnected = True
                                self.close_connection = True
                raw_response = b"".join(chunks)
                usage = reported_usage(raw_response, content_type)
                successful = 200 <= status < 300
                if successful and binding["kind"] == "model":
                    need(usage["output_tokens"] is not None, "successful_generation_usage_missing")
                    declared = max(pointer(body, path) or 0 for path in binding["output_token_paths"])
                    need(usage["output_tokens"] <= declared, "upstream_generated_above_declared_ceiling")
                if successful and binding["kind"] == "embedding":
                    need(usage["input_or_total_tokens"] is not None, "successful_embedding_usage_missing")
                ledger.settle(request_id, "completed" if successful else "failed", {
                    "http_status": status, "response_bytes": response_bytes, "response_sha256": "sha256:" + digest.hexdigest(),
                    "outbound_finished_unix": time.time(),
                    "downstream_disconnected": downstream_disconnected,
                    "actual_usage": usage, "actual_charge": "unverified; full approved ceiling retained"})
                request_id = None
            except BaseException as error:
                if request_id is not None:
                    try:
                        ledger.settle(request_id, "unknown", {"failure": str(error) if isinstance(error, Refusal) else type(error).__name__,
                            "response_bytes_observed": response_bytes, "response_sha256_partial": "sha256:" + digest.hexdigest(),
                            "actual_charge": "unknown; full approved reservation retained"})
                    except BaseException:
                        with ledger.mutex:
                            ledger.accepting = False
                if not response_started:
                    try:
                        self.refusal(403 if isinstance(error, Refusal) and str(error) == "proxy_ingress_unauthorized" else 429,
                                     str(error) if isinstance(error, Refusal) else "proxy_request_unconfirmed")
                    except (OSError, socket.error):
                        pass
                # Once streaming began, close it without a fake success/error
                # content fragment. Durable UNKNOWN remains authoritative.
                self.close_connection = True
            finally:
                capacity.release()
    return Handler


def main():
    need(len(sys.argv) == 2, "expected_one_exact_approved_config_path")
    raw = private_file(sys.argv[1])
    config, seed, identity = validate_config(raw)
    ledger = Ledger(config, seed, identity)
    ledger.remaining_seconds()
    need(ledger.get("blocked") is None, "epoch_blocked_before_listener_start")
    capacity = threading.BoundedSemaphore(config["limits"]["max_in_flight"])
    servers, threads = [], []
    stopped = threading.Event()
    def stop(_signum=None, _frame=None):
        ledger.accepting = False
        stopped.set()
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    # A trickling or blocked upstream cannot extend the absolute packet wall
    # allowance. Hard deadline exhaustion terminates this owner and leaves any
    # in-flight durable reservation unreconciled; the packet must fail closed.
    def exhausted():
        ledger.accepting = False
        os._exit(75)
    deadline_guard = threading.Timer(ledger.remaining_seconds(), exhausted)
    deadline_guard.daemon = True
    deadline_guard.start()
    try:
        # Bind every frozen endpoint before serving any charged request.
        for binding in config["bindings"]:
            server = ThreadingHTTPServer(("127.0.0.1", binding["listen_port"]), handler_class(ledger, binding, capacity))
            server.daemon_threads = False
            servers.append(server)
        for server in servers:
            thread = threading.Thread(target=server.serve_forever, kwargs={"poll_interval": 0.1})
            thread.start()
            threads.append(thread)
        ready_path = ledger.root / ("ready-" + ledger.nonce + ".json")
        receipt = {"pid": os.getpid(), "nonce": ledger.nonce, "config_sha256": sha(raw), "ready_path": str(ready_path),
                   "listeners": [row["listen_port"] for row in config["bindings"]],
                   "epoch": ledger.snapshot()}
        with os.fdopen(os.open(ready_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600), "w") as stream:
            json.dump(receipt, stream, sort_keys=True)
            stream.flush()
            os.fsync(stream.fileno())
        print(json.dumps(receipt, sort_keys=True), flush=True)
        while not stopped.wait(timeout=min(1, max(0.01, seed["deadline_unix"] - time.time()))):
            if time.time() >= seed["deadline_unix"]:
                stop()
    finally:
        ledger.accepting = False
        for index, server in enumerate(servers):
            if index < len(threads):
                server.shutdown()
            server.server_close()  # joins all non-daemon request workers
        for thread in threads:
            thread.join()
        deadline_guard.cancel()
        print(json.dumps({"terminal": ledger.snapshot()}, sort_keys=True), flush=True)
        ledger.db.close()
        fcntl.flock(ledger.lock, fcntl.LOCK_UN)
        os.close(ledger.lock)


if __name__ == "__main__":
    try:
        main()
    except BaseException as error:
        print(json.dumps({"error": "budget_forwarder_refused_or_unconfirmed", "code": str(error) if isinstance(error, Refusal)
                          else type(error).__name__}), flush=True)
        sys.exit(2)
