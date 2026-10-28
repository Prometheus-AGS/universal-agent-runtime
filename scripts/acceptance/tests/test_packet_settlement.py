"""Authored only. Do not execute while the operator verification gate is CLOSED."""
import json
from pathlib import Path
import sqlite3
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import budget_proxy
from core import Blocked, digest
from packet_processes import PacketProcesses


class PacketSettlementTests(unittest.TestCase):
    def packet(self, directory, state="completed", status=200, usage=None):
        root = Path(directory)
        root.chmod(0o700)
        config_path = root / "config.json"
        config_path.write_text("{}")
        config_path.chmod(0o600)
        counters = dict.fromkeys(budget_proxy.COUNTERS, 0)
        seed = {"consumed": counters, "spend_pool_consumed_microunits": {"liter": 0},
                "deadline_unix": int(time.time()) + 60}
        binding = {"id": "model", "kind": "model", "spend_pool": "liter",
                   "approved_charge_ceiling_microunits": 0, "max_response_bytes": 8}
        config = {"epoch_id": "epoch", "ledger_root": str(root), "bindings": [binding],
                  "limits": dict.fromkeys(budget_proxy.COUNTERS, 100),
                  "spend_pools": {"liter": {"limit_microunits": 0}}}
        ready_path = root / "ready-nonce.json"
        ready = {"pid": 123, "nonce": "nonce", "config_sha256": digest(b"{}"),
                 "epoch": {"epoch_id": "epoch", "input_identity": "identity"},
                 "listeners": [8080], "ready_path": str(ready_path)}
        ready_path.write_text(json.dumps(ready))
        ready_path.chmod(0o600)
        reserved = {**counters, "model_attempts": 1, "generated_tokens": 10,
                    "request_bytes": 3, "response_bytes": 8}
        receipt = {"epoch_id": "epoch", "input_identity": "identity", "binding_id": "model",
                   "kind": "model", "spend_pool": "liter", "http_status": status,
                   "response_bytes": 5, "response_sha256": digest(b"reply"),
                   "reserved_at_unix": 1, "outbound_started_unix": 1,
                   "outbound_finished_unix": 2, "settled_at_unix": 2,
                   "actual_usage": usage if usage is not None else {"output_tokens": 4}}
        meta = {"identity": "identity", "epoch_id": "epoch", "owner": {"pid": 123, "nonce": "nonce"},
                "seed": seed, "blocked": None,
                "totals": {**reserved, "response_bytes": 5 if state == "completed" else 8},
                "spend_pool_totals": {"liter": 0}}
        with sqlite3.connect(root / "ledger.sqlite3") as db:
            db.execute("CREATE TABLE meta (key TEXT PRIMARY KEY,value TEXT)")
            db.executemany("INSERT INTO meta VALUES (?,?)", [(key, json.dumps(value)) for key, value in meta.items()])
            db.execute("CREATE TABLE reservations (request_id TEXT,binding_id TEXT,kind TEXT,state TEXT,reserved TEXT,receipt TEXT)")
            db.execute("INSERT INTO reservations VALUES (?,?,?,?,?,?)",
                       ("request", "model", "model", state, json.dumps(reserved), json.dumps(receipt)))
        (root / "ledger.sqlite3").chmod(0o600)
        packet = PacketProcesses.__new__(PacketProcesses)
        source = Path(budget_proxy.__file__).resolve()
        packet.fixture = {"frozen_files": [{"path": str(path), "sha256": digest(path.read_bytes())}
                                            for path in (config_path, source)]}
        packet.spec = {"argv": ["python", str(source), str(config_path)], "epoch_id": "epoch", "listen_ports": [8080]}
        packet.ready = ready
        packet.process = SimpleNamespace(pid=123, poll=lambda: None)
        packet.ctx = SimpleNamespace(remaining=lambda: 60, record=lambda name, value: None)
        return packet, config, seed

    def test_pending_drain_settles_before_return(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory, state="reserved")

            def complete(_seconds):
                with sqlite3.connect(Path(directory) / "ledger.sqlite3") as db:
                    db.execute("UPDATE reservations SET state='completed'")
                    totals = json.loads(db.execute("SELECT value FROM meta WHERE key='totals'").fetchone()[0])
                    totals["response_bytes"] = 5
                    db.execute("UPDATE meta SET value=? WHERE key='totals'", (json.dumps(totals),))

            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")), \
                    patch("packet_processes.time.sleep", side_effect=complete) as sleep:
                result = packet.settle(lambda: 60)
            self.assertEqual(sleep.call_count, 1)
            self.assertEqual(result["pending"], [])
            self.assertEqual(result["totals_reserved"]["model_attempts"], 1)
            self.assertEqual(result["totals_reserved"]["generated_tokens"], 10)

    def test_unknown_reservation_stops_without_wait_or_retry(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory, state="unknown")
            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")), \
                    patch("packet_processes.time.sleep") as sleep:
                with self.assertRaises(Blocked):
                    packet.settle(lambda: 60)
            sleep.assert_not_called()

    def test_known_controlled_401_keeps_reservation_and_permits_settlement(self):
        for status in (401, 500):
            with self.subTest(status=status), tempfile.TemporaryDirectory() as directory:
                packet, config, seed = self.packet(directory, state="failed", status=status, usage={})
                with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")):
                    result = packet.settle(lambda: 60)
                self.assertEqual(result["pending"], [])
                self.assertEqual(result["totals_reserved"]["response_bytes"], 8)
                self.assertEqual(result["totals_reserved"]["generated_tokens"], 10)

    def test_malformed_terminals_stop_before_continuation(self):
        changes = [(name, value) for name in ("reserved_at_unix", "outbound_started_unix",
                    "outbound_finished_unix", "settled_at_unix")
                   for value in (None, float("nan"), float("inf"), 0, True)]
        changes += [("reserved_at_unix", 3), ("settled_at_unix", 0.5),
                    ("response_sha256", None), ("response_sha256", "sha256:" + "g" * 64),
                    ("response_bytes", 9)]
        for state, status in (("failed", 401), ("failed", 500), ("completed", 200)):
            for field, value in changes:
                with self.subTest(state=state, status=status, field=field, value=value), \
                        tempfile.TemporaryDirectory() as directory:
                    packet, config, seed = self.packet(directory, state=state, status=status)
                    with sqlite3.connect(Path(directory) / "ledger.sqlite3") as db:
                        receipt = json.loads(db.execute("SELECT receipt FROM reservations").fetchone()[0])
                        if field == "response_sha256" and value is None:
                            receipt.pop(field)
                        else:
                            receipt[field] = value
                        db.execute("UPDATE reservations SET receipt=?", (json.dumps(receipt),))
                    with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")), \
                            patch("packet_processes.time.sleep") as sleep:
                        with self.assertRaises(Blocked):
                            packet.settle(lambda: 60)
                    sleep.assert_not_called()

    def test_server_uses_shared_terminal_validator(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory)
            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")), \
                    patch("budget_proxy.terminal_receipt_error", return_value="accounting") as validator:
                with self.assertRaises(Blocked):
                    packet.settle(lambda: 60)
            validator.assert_called_once()
            self.assertEqual(validator.call_args.kwargs["max_output_tokens"], 10)
            self.assertEqual(validator.call_args.kwargs["max_response_bytes"], 8)

    def test_completed_output_cannot_exceed_actual_reservation(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory, usage={"output_tokens": 11})
            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")):
                with self.assertRaises(Blocked):
                    packet.settle(lambda: 60)

    def test_shared_embedding_requires_actual_input_or_total_usage(self):
        receipt = {"epoch_id": "epoch", "input_identity": "identity", "binding_id": "embedding",
                   "kind": "embedding", "http_status": 200, "response_bytes": 5,
                   "response_sha256": digest(b"reply"), "reserved_at_unix": 1,
                   "outbound_started_unix": 2, "outbound_finished_unix": 3, "settled_at_unix": 4}
        bounds = {"epoch_id": "epoch", "input_identity": "identity", "binding_id": "embedding",
                  "kind": "embedding", "max_response_bytes": 8, "max_output_tokens": 0}
        for usage, expected in ((None, "usage"), ({"output_tokens": 4}, "usage"),
                                ({"input_or_total_tokens": 4}, None)):
            with self.subTest(usage=usage):
                receipt["actual_usage"] = usage
                self.assertEqual(budget_proxy.terminal_receipt_error("completed", receipt, **bounds), expected)

    def test_successful_unknown_usage_stops(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory, usage={})
            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")):
                with self.assertRaises(Blocked):
                    packet.settle(lambda: 60)

    def test_exact_attempt_saturation_is_still_known_settlement(self):
        with tempfile.TemporaryDirectory() as directory:
            packet, config, seed = self.packet(directory)
            config["limits"]["model_attempts"] = 1
            config["limits"]["embedding_attempts"] = 0
            with patch("budget_proxy.validate_config", return_value=(config, seed, "identity")):
                result = packet.settle(lambda: 60)
            self.assertEqual(result["totals_reserved"]["model_attempts"], 1)
            self.assertEqual(result["pending"], [])

    def test_pending_timeout_never_returns_settled(self):
        packet = PacketProcesses.__new__(PacketProcesses)
        packet.ctx = SimpleNamespace(remaining=lambda: 60)
        snapshot = {"pending": ["request"], "deadline_unix": time.time() + 1000}
        with patch.object(packet, "_ledger_snapshot", return_value=snapshot), \
                patch("packet_processes.time.monotonic", side_effect=[0, 0, 0, 0, 91]), \
                patch("packet_processes.time.sleep"):
            with self.assertRaises(Blocked):
                packet.settle(lambda: 60)
