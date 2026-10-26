"""Authored policy regressions only; never executed under the CLOSED gate."""
import unittest
from copy import deepcopy
from unittest.mock import MagicMock, patch

from ios.runner import collection_status, item_manifest, ledger_snapshot, sealed_file, settled_ledger


class IOSFailurePolicyTests(unittest.TestCase):
    def settlement_snapshot(self, state="failed", status=500):
        return {"epoch_id":"epoch", "input_identity":"input", "binding_id":"binding",
                "model_output_cap":100, "model_response_cap":20,
                "totals":{"model_attempts":1,"cost_microunits":1000},
                "reservations":{"request":{
                    "reserved":{"generated_tokens":15,"response_bytes":20},
                    "state":state, "receipt":{"epoch_id":"epoch", "input_identity":"input",
                        "binding_id":"binding", "kind":"model", "http_status":status,
                        "reserved_at_unix":1.0, "outbound_started_unix":2.0,
                        "outbound_finished_unix":3.0, "settled_at_unix":4.0,
                        "response_bytes":10, "response_sha256":"sha256:"+"a"*64,
                        "actual_usage":{"output_tokens":None},
                        "actual_charge":"unverified; full approved ceiling retained"}}}}

    def settle(self, after):
        before = dict(after,reservations={})
        with patch("ios.runner.ledger_snapshot",return_value=after), patch("ios.runner.DEADLINE",1000), \
                patch("ios.runner.time.monotonic",return_value=10):
            return settled_ledger({},before)

    def test_known_500_and_401_settle_without_releasing_reservations(self):
        for status in (500,401):
            with self.subTest(status=status):
                after = self.settlement_snapshot(status=status)
                original = deepcopy(after)
                self.assertIs(self.settle(after),after)
                self.assertEqual(after,original)
                self.assertEqual(after["reservations"]["request"]["state"],"failed")

    def test_malformed_failed_terminal_is_rejected(self):
        changes = {"http_status":200,"binding_id":"other","epoch_id":"other",
                   "input_identity":"other","outbound_finished_unix":None,
                   "settled_at_unix":1.5,"outbound_started_unix":float("nan"),
                   "response_bytes":-1,"response_sha256":"incomplete"}
        for key,value in changes.items():
            with self.subTest(field=key):
                after = self.settlement_snapshot()
                after["reservations"]["request"]["receipt"][key] = value
                with self.assertRaises(RuntimeError):
                    self.settle(after)
        changes = [(name, value) for name in ("reserved_at_unix", "outbound_started_unix",
                    "outbound_finished_unix", "settled_at_unix")
                   for value in (None, float("nan"), float("inf"), 0, True)]
        changes += [("reserved_at_unix", 5), ("settled_at_unix", 1.5),
                    ("response_sha256", None), ("response_sha256", "sha256:" + "g" * 64)]
        for state,status in (("failed",401),("failed",500),("completed",200)):
            for field,value in changes:
                with self.subTest(state=state,status=status,field=field,value=value):
                    after = self.settlement_snapshot(state=state,status=status)
                    receipt = after["reservations"]["request"]["receipt"]
                    receipt["actual_usage"] = {"output_tokens":10}
                    if field == "response_sha256" and value is None:
                        receipt.pop(field)
                    else:
                        receipt[field] = value
                    with self.assertRaises(RuntimeError):
                        self.settle(after)

    def test_ios_uses_shared_terminal_validator(self):
        after = self.settlement_snapshot()
        with patch("budget_proxy.terminal_receipt_error",return_value="accounting") as validator:
            with self.assertRaisesRegex(RuntimeError,"new forwarding accounting unknown"):
                self.settle(after)
        validator.assert_called_once()
        self.assertEqual(validator.call_args.kwargs["max_output_tokens"],15)
        self.assertEqual(validator.call_args.kwargs["max_response_bytes"],20)

    def test_actual_reservation_bounds_are_stricter_than_binding_ceiling(self):
        after = self.settlement_snapshot(state="completed",status=200)
        receipt = after["reservations"]["request"]["receipt"]
        receipt["actual_usage"] = {"output_tokens":16}
        with self.assertRaisesRegex(RuntimeError,"new successful forwarding usage unknown"):
            self.settle(after)
        receipt["actual_usage"] = {"output_tokens":15}
        self.assertIs(self.settle(after),after)
        for state,status in (("completed",200),("failed",401),("failed",500)):
            with self.subTest(state=state,status=status):
                after = self.settlement_snapshot(state=state,status=status)
                after["reservations"]["request"]["reserved"]["response_bytes"] = 9
                after["reservations"]["request"]["receipt"]["actual_usage"] = {"output_tokens":10}
                with self.assertRaisesRegex(RuntimeError,"new forwarding accounting unknown"):
                    self.settle(after)

    def test_unknown_reservation_blocks_settlement(self):
        with self.assertRaises(RuntimeError):
            self.settle(self.settlement_snapshot(state="unknown"))

    def test_pending_reservation_waits_then_settles_known_failure(self):
        after = self.settlement_snapshot()
        pending = self.settlement_snapshot(state="reserved")
        with patch("ios.runner.ledger_snapshot",side_effect=[pending,after]), \
                patch("ios.runner.DEADLINE",1000), patch("ios.runner.time.monotonic",return_value=10), \
                patch("ios.runner.time.sleep") as sleep:
            self.assertIs(settled_ledger({},dict(after,reservations={})),after)
            sleep.assert_called_once_with(0.25)

    def test_pending_reservation_still_obeys_original_deadline(self):
        pending = self.settlement_snapshot(state="reserved")
        with patch("ios.runner.ledger_snapshot",return_value=pending), \
                patch("ios.runner.DEADLINE",11), patch("ios.runner.time.monotonic",side_effect=[10,10,11]), \
                patch("ios.runner.time.sleep") as sleep:
            with self.assertRaises(RuntimeError):
                settled_ledger({},dict(pending,reservations={}))
            sleep.assert_not_called()

    def test_late_completed_or_empty_snapshot_cannot_settle(self):
        for overall,late in ((1000,100),(11,11)):
            for empty in (False,True):
                with self.subTest(overall=overall,empty=empty):
                    after = self.settlement_snapshot(state="completed",status=200)
                    after["reservations"]["request"]["receipt"]["actual_usage"] = {"output_tokens":10}
                    if empty:
                        after["reservations"] = {}
                    with patch("ios.runner.ledger_snapshot",return_value=after) as snapshot, \
                            patch("ios.runner.DEADLINE",overall), \
                            patch("ios.runner.time.monotonic",side_effect=[10,10,late]):
                        with self.assertRaisesRegex(RuntimeError,"forwarding settlement unknown"):
                            settled_ledger({},dict(after,reservations={}))
                    snapshot.assert_called_once_with({},remaining=min(overall-10,90))

    def test_deadline_expiring_during_terminal_validation_cannot_settle(self):
        after = self.settlement_snapshot()
        with patch("ios.runner.ledger_snapshot",return_value=after), \
                patch("ios.runner.DEADLINE",1000), \
                patch("ios.runner.time.monotonic",side_effect=[10,10,10,100]):
            with self.assertRaisesRegex(RuntimeError,"forwarding settlement unknown"):
                settled_ledger({},dict(after,reservations={}))

    def test_pending_poll_is_clipped_to_remaining_allowance(self):
        pending = self.settlement_snapshot(state="reserved")
        with patch("ios.runner.ledger_snapshot",return_value=pending), \
                patch("ios.runner.DEADLINE",11), \
                patch("ios.runner.time.monotonic",side_effect=[10,10,10.9,10.9,11]), \
                patch("ios.runner.time.sleep") as sleep:
            with self.assertRaisesRegex(RuntimeError,"forwarding settlement unknown"):
                settled_ledger({},dict(pending,reservations={}))
            self.assertEqual(sleep.call_count,1)
            self.assertAlmostEqual(sleep.call_args.args[0],0.1)

    def test_snapshot_clips_sqlite_wait_and_closes_read_connection(self):
        approved = {"ledger_root":"/private/ledger", "limits":{"attempts":1},
                    "seed":{"path":"/private/seed.json"}, "epoch_id":"epoch"}
        binding = {"max_output_tokens_per_attempt":100,"max_response_bytes":20,"id":"binding"}
        for remaining,overall in ((0.05,1000),(None,10.05)):
            for failed in (False,True):
                with self.subTest(remaining=remaining,overall=overall,failed=failed):
                    db = MagicMock()
                    db.execute.side_effect = ([[],RuntimeError("locked")] if failed else
                        [[],[("blocked","null"),("totals",'{"attempts":0}')],[],[]])
                    with patch("budget_proxy.owned_hop",return_value=(approved,binding,
                                {"epoch":{"input_identity":"input"}})), \
                            patch("budget_proxy.COUNTERS",("attempts",)), \
                            patch("ios.runner.sqlite3.connect",return_value=db) as connect, \
                            patch("ios.runner.read",return_value={"deadline_unix":1000}), \
                            patch("ios.runner.DEADLINE",overall), \
                            patch("ios.runner.time.monotonic",return_value=10):
                        if failed:
                            with self.assertRaisesRegex(RuntimeError,"locked"):
                                ledger_snapshot({"budget_proxy_config":"config","base_url":"url"},remaining)
                        else:
                            ledger_snapshot({"budget_proxy_config":"config","base_url":"url"},remaining)
                        self.assertGreater(connect.call_args.kwargs["timeout"],0)
                        self.assertLessEqual(connect.call_args.kwargs["timeout"],0.051)
                        self.assertTrue(connect.call_args.kwargs["uri"])
                        self.assertTrue(connect.call_args.args[0].endswith("?mode=ro"))
                        for call in db.execute.call_args_list:
                            sql = call.args[0]
                            if sql.startswith("PRAGMA busy_timeout="):
                                self.assertLessEqual(int(sql.split("=")[1]),50)
                        progress = db.set_progress_handler.call_args.args[0]
                        self.assertEqual(progress(),0)
                        with patch("ios.runner.time.monotonic",return_value=11):
                            self.assertEqual(progress(),1)
                    db.close.assert_called_once_with()

    def test_default_snapshot_cannot_connect_after_overall_deadline(self):
        with patch("budget_proxy.owned_hop",return_value=({"ledger_root":"/private/ledger"},{},{})), \
                patch("ios.runner.sqlite3.connect") as connect, \
                patch("ios.runner.DEADLINE",10), patch("ios.runner.time.monotonic",return_value=10):
            with self.assertRaisesRegex(RuntimeError,"ledger snapshot allowance exhausted"):
                ledger_snapshot({"budget_proxy_config":"config","base_url":"url"})
            connect.assert_not_called()

    def test_completed_requires_success_http_and_actual_output_usage(self):
        after = self.settlement_snapshot(state="completed",status=200)
        with self.assertRaises(RuntimeError):
            self.settle(after)
        after["reservations"]["request"]["receipt"]["actual_usage"] = {"output_tokens":10}
        self.assertIs(self.settle(after),after)
        after["reservations"]["request"]["receipt"]["http_status"] = 500
        with self.assertRaises(RuntimeError):
            self.settle(after)

    def test_known_failure_settlement_does_not_promote_failed_item(self):
        rows = item_manifest("run")
        for row in rows:
            row.update(status="PASS",source_identity="source",artifact_identity="artifact")
        rows[0]["status"] = "FAIL"
        self.settle(self.settlement_snapshot())
        self.assertEqual(rows[0]["status"],"FAIL")
        self.assertEqual(collection_status(rows),"FAIL")

    def test_six_stable_items_retain_repetition_specific_dependency(self):
        rows = item_manifest("run")
        self.assertEqual([row["item_id"] for row in rows], [
            "ios:seed:1", "ios:restore:1", "ios:seed:2", "ios:restore:2",
            "ios:seed:3", "ios:restore:3"])
        self.assertEqual(rows[3]["dependency_ids"], ["ios:seed:2"])
        self.assertEqual(rows[4]["dependency_ids"], [])

    def test_unresolved_seed_cannot_aggregate_as_success(self):
        rows = item_manifest("run")
        for row in rows:
            row.update(status="PASS", source_identity="source", artifact_identity="artifact")
        rows[0]["status"] = "FAIL"
        rows[1]["status"] = "BLOCKED"
        self.assertEqual(collection_status(rows), "FAIL")

    def test_corrected_mixed_sources_stay_blocked(self):
        rows = item_manifest("run")
        for row in rows:
            row.update(status="PASS", source_identity="old", artifact_identity="old-artifact")
        rows[-1].update(source_identity="new", artifact_identity="new-artifact")
        self.assertEqual(collection_status(rows), "BLOCKED")

    def test_full_coherent_six_item_result_passes(self):
        rows = item_manifest("run")
        for row in rows:
            row.update(status="PASS", source_identity="source", artifact_identity="artifact")
        self.assertEqual(collection_status(rows), "PASS")
        self.assertEqual(collection_status(rows[:-1]), "BLOCKED")

    def test_changed_frozen_correction_input_is_rejected(self):
        with patch("ios.runner.Path") as path_type:
            path = path_type.return_value
            path.is_absolute.return_value = True
            path.resolve.return_value = path
            path.is_file.return_value = True
            path.stat.return_value.st_size = 3
            path.read_bytes.return_value = b"new"
            with self.assertRaises(RuntimeError):
                sealed_file({"path":"/private/controller.py", "sha256":"old"})
