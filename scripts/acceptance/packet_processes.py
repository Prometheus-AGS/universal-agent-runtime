"""Invocation-owned packet forwarding process; no work occurs on import.

The exact approved fixture names every command, input and environment reference.
This owner outlives backend cleanup so cold restarts cannot reset reservations.
"""
from __future__ import annotations

from contextlib import closing
import json
import os
from pathlib import Path
import sqlite3
import time

from core import Blocked, Context, Failed, digest, require


class PacketProcesses:
    def __init__(self, fixture, evidence, run_id, started):
        self.fixture = fixture
        self.spec = fixture.get("packet_processes", {}).get("budget_proxy")
        if not isinstance(self.spec, dict):
            raise Blocked("full packet lacks its actual shared forwarding admission owner")
        self.service = fixture.get("packet_processes", {}).get("existing_service")
        if not isinstance(self.service, dict):
            raise Blocked("configured Liter identity observation is missing from the exact packet")
        profile = {"name": "packet", "profile_id": "shared-forwarding-epoch",
                   "environment_env": {**self.spec["environment_env"], **self.service.get("environment_env", {})},
                   "headers_env": {}, "observers": {"existing_service": self.service["argv"]}}
        self.ctx = Context(fixture, profile, run_id, evidence)
        self.ctx.started = started
        self.process = None
        self.ready = None
        self.stopped = False

    def start(self):
        service_config_path = Path(self.service["argv"][2])
        service_config_raw = service_config_path.read_bytes()
        service_config = json.loads(service_config_raw)
        receipt = self.ctx.observe("existing_service")
        receipt_path = Path(service_config["receipt_path"])
        require(receipt.get("schema_version") == "uar.configured-service-identity.v1"
                and receipt.get("status") == "observed" and receipt.get("service_id") == "ai.prometheus.liter-llm-api"
                and receipt.get("config_sha256") == digest(service_config_raw)
                and type(receipt.get("pid")) is int and receipt["pid"] > 1
                and receipt.get("expected_command_matched") is True
                and receipt_path.is_file() and not receipt_path.is_symlink()
                and json.loads(receipt_path.read_bytes()) == receipt,
                "configured Liter live identity is unconfirmed; no positive traffic is admitted")
        self.ctx.record("configured-service-identity", receipt)
        config_path = Path(self.spec["argv"][2])
        config_raw = config_path.read_bytes()
        config = json.loads(config_raw)
        self.process = self.ctx.owned_process(self.spec["argv"])
        deadline = time.monotonic() + min(self.ctx.remaining(), self.spec["ready_seconds"])
        output, _ = self.process._acceptance_output
        while time.monotonic() < deadline:
            self.ctx.remaining()
            require(self.process.poll() is None, "shared forwarding process exited before binding its endpoints")
            output.seek(0)
            raw = output.read(self.fixture["limits"]["response_bytes"] + 1)
            if b"\n" in raw:
                try:
                    receipt = json.loads(raw.split(b"\n", 1)[0])
                except (ValueError, UnicodeDecodeError) as error:
                    raise Failed("shared forwarding startup receipt is malformed") from error
                require(receipt.get("pid") == self.process.pid and receipt.get("nonce")
                        and receipt.get("config_sha256") == digest(config_raw)
                        and receipt.get("epoch", {}).get("epoch_id") == self.spec["epoch_id"]
                        and receipt.get("epoch", {}).get("blocked") is None
                        and sorted(receipt.get("listeners", [])) == sorted(self.spec["listen_ports"]),
                        "shared forwarding receipt differs from the exact approved packet")
                root = Path(config["ledger_root"])
                ready_path = root / ("ready-" + receipt["nonce"] + ".json")
                require(receipt.get("ready_path") == str(ready_path)
                        and ready_path.is_file() and not ready_path.is_symlink()
                        and json.loads(ready_path.read_bytes()) == receipt,
                        "shared forwarding owned readiness file differs from its process receipt")
                self.ctx.record("forwarding-ready", receipt)
                self.ready = receipt
                return
            time.sleep(0.05)
        raise Blocked("shared forwarding endpoints did not publish an owned readiness receipt within allowance")

    def alive(self):
        self.ctx.remaining()
        require(self.process is not None and self.process.poll() is None,
                "shared forwarding owner stopped; no new case may dispatch")

    def _ledger_snapshot(self, seconds):
        """Read one consistent, read-only view of this exact live packet."""
        import budget_proxy

        deadline = time.monotonic() + seconds
        self.alive()
        frozen = {row["path"]: row["sha256"] for row in self.fixture["frozen_files"]}
        config_path = Path(self.spec["argv"][2])
        for path in (config_path, Path(budget_proxy.__file__).resolve()):
            require(path.is_file() and not path.is_symlink()
                    and frozen.get(str(path)) == digest(path.read_bytes()),
                    "shared forwarding config or source differs from frozen packet")
        raw = budget_proxy.private_file(str(config_path))
        config, seed, identity = budget_proxy.validate_config(raw)
        require(self.ready is not None and self.ready["pid"] == self.process.pid
                and self.ready["config_sha256"] == digest(raw)
                and config["epoch_id"] == self.spec["epoch_id"] == self.ready["epoch"]["epoch_id"]
                and identity == self.ready["epoch"]["input_identity"]
                and sorted(self.ready["listeners"]) == sorted(self.spec["listen_ports"]),
                "shared forwarding live source/config/epoch binding changed")
        root = Path(config["ledger_root"])
        require(root.is_dir() and not root.is_symlink() and root.stat().st_mode & 0o077 == 0
                and root.stat().st_uid == os.getuid(),
                "shared forwarding ledger root is not private")
        ready_path = root / ("ready-" + self.ready["nonce"] + ".json")
        require(self.ready["ready_path"] == str(ready_path)
                and json.loads(budget_proxy.private_file(str(ready_path))) == self.ready,
                "shared forwarding owned readiness receipt changed")
        path = root / "ledger.sqlite3"
        require(path.is_file() and not path.is_symlink() and path.stat().st_uid == os.getuid()
                and path.stat().st_mode & 0o077 == 0, "shared forwarding ledger unavailable or not private")
        timeout = max(0, min(deadline - time.monotonic(), self.ctx.remaining(),
                             seed["deadline_unix"] - time.time()))
        require(timeout > 0, "shared forwarding settlement wall allowance exhausted")
        with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=timeout)) as db:
            db.execute("BEGIN")
            meta = {key: json.loads(value) for key, value in db.execute("SELECT key,value FROM meta")}
            maximum = config["limits"]["model_attempts"] + config["limits"]["embedding_attempts"]
            rows = list(db.execute("SELECT request_id,binding_id,kind,state,reserved,receipt FROM reservations LIMIT ?",
                                   (maximum + 1,)))
        require(len(rows) <= maximum, "shared forwarding reservation inventory exceeds approved attempt cap")
        require(meta.get("identity") == identity and meta.get("epoch_id") == config["epoch_id"]
                and meta.get("owner") == {"pid": self.process.pid, "nonce": self.ready["nonce"]}
                and meta.get("seed") == seed and meta.get("blocked") is None,
                "shared forwarding ownership or accounting is unknown")
        totals = dict(seed["consumed"])
        pools = dict(seed["spend_pool_consumed_microunits"])
        bindings = {binding["id"]: binding for binding in config["bindings"]}
        pending = []
        for request_id, binding_id, kind, state, reserved_raw, receipt_raw in rows:
            reserved, receipt = json.loads(reserved_raw), json.loads(receipt_raw)
            binding = bindings.get(binding_id)
            require(binding is not None and kind == binding["kind"]
                    and state in ("reserved", "completed", "failed")
                    and receipt.get("epoch_id") == config["epoch_id"]
                    and receipt.get("input_identity") == identity
                    and receipt.get("binding_id") == binding_id and receipt.get("kind") == kind
                    and receipt.get("spend_pool") == binding["spend_pool"]
                    and set(reserved) == set(budget_proxy.COUNTERS)
                    and all(type(value) is int and value >= 0 for value in reserved.values())
                    and reserved["cost_microunits"] == binding["approved_charge_ceiling_microunits"]
                    and reserved["response_bytes"] == binding["max_response_bytes"],
                    "shared forwarding reservation is unknown or inconsistent")
            if state == "reserved":
                pending.append(request_id)
            else:
                error = budget_proxy.terminal_receipt_error(state, receipt,
                    epoch_id=config["epoch_id"], input_identity=identity,
                    binding_id=binding_id, kind=kind,
                    max_response_bytes=reserved["response_bytes"],
                    max_output_tokens=reserved["generated_tokens"])
                require(error != "accounting",
                        "shared forwarding terminal accounting is inconsistent")
                # Known non-success HTTP responses retain their full reservation.
                # The proxy requires actual token usage only for successful calls.
                if state == "completed":
                    require(error != "usage",
                            "shared forwarding successful usage is unknown or above reservation")
                    reserved["response_bytes"] = receipt["response_bytes"]
            for key in budget_proxy.COUNTERS:
                totals[key] += reserved[key]
            pools[binding["spend_pool"]] += reserved["cost_microunits"]
        require(meta.get("totals") == totals and meta.get("spend_pool_totals") == pools
                and all(totals[key] <= config["limits"][key] for key in budget_proxy.COUNTERS)
                and all(pools[key] <= config["spend_pools"][key]["limit_microunits"] for key in pools),
                "shared forwarding cumulative allowance is inconsistent or exceeded")
        return {"epoch_id": config["epoch_id"], "input_identity": identity,
                "pending": pending, "reservation_count": len(rows), "totals_reserved": totals,
                "spend_pool_reserved_microunits": pools, "deadline_unix": seed["deadline_unix"]}

    def settle(self, remaining):
        """Wait for upstream draining, without refund, retry, or new dispatch."""
        deadline = time.monotonic() + min(90, remaining(), self.ctx.remaining())
        try:
            while True:
                seconds = min(deadline - time.monotonic(), remaining(), self.ctx.remaining())
                require(seconds > 0, "shared forwarding settlement unknown; no continuation")
                snapshot = self._ledger_snapshot(seconds)
                require(time.monotonic() < deadline and time.time() < snapshot["deadline_unix"],
                        "shared forwarding settlement allowance exhausted")
                if not snapshot["pending"]:
                    self.ctx.record("forwarding-settled", snapshot)
                    return snapshot
                time.sleep(min(0.25, max(0, deadline - time.monotonic())))
        except Exception as error:
            raise Blocked("shared forwarding settlement unconfirmed; no correction or independent dispatch") from error

    def stop(self):
        if self.stopped:
            return
        self.stopped = True
        if self.process is None:
            for process in list(self.ctx.processes):
                self.ctx.stop_process(process)
                self.ctx.release_process(process)
            return
        self.ctx.stop_process(self.process)
        output, errors = self.process._acceptance_output
        try:
            output.seek(0)
            raw = output.read(self.fixture["limits"]["response_bytes"] + 1)
            require(len(raw) <= self.fixture["limits"]["response_bytes"], "packet forwarding receipt exceeded bound")
            rows = [json.loads(line) for line in raw.splitlines() if line.strip()]
            terminals = [row["terminal"] for row in rows if isinstance(row, dict) and "terminal" in row]
            require(self.process.returncode == 0 and len(terminals) == 1
                    and terminals[0].get("epoch_id") == self.spec["epoch_id"],
                    "shared forwarding cleanup lacks joined successful terminal ledger receipt")
            self.ctx.record("forwarding-terminal", terminals[0])
            require(terminals[0].get("blocked") is None, "shared forwarding allowance exhausted or became unknown")
        finally:
            output.close()
            errors.close()
