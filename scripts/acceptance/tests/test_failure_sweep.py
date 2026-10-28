"""Authored behavior tests; execution requires the operator-open verification gate."""
import unittest

from core import Blocked
from failure_sweep import aggregate, dependency_ids, sweep_contract


class SweepContractTests(unittest.TestCase):
    def fixture(self):
        names = [f"case-{number}" for number in range(97)]
        return {"required_cases": names, "failure_sweep": {
            "schema_version": 1, "prerequisites": {name: [] for name in names}}}

    def test_every_case_retains_explicit_prerequisites(self):
        fixture = self.fixture()
        del fixture["failure_sweep"]["prerequisites"]["case-1"]
        with self.assertRaises(Blocked):
            sweep_contract(fixture, [])

    def test_future_dependency_is_rejected_before_dispatch(self):
        fixture = self.fixture()
        fixture["failure_sweep"]["prerequisites"]["case-0"] = ["case-1"]
        with self.assertRaises(Blocked):
            sweep_contract(fixture, [])

    def test_dependency_is_backend_and_repetition_specific(self):
        row = {"backend": "surreal_disk", "case_id": "B", "attempt": 3}
        self.assertEqual(dependency_ids(row, {"B": ["A"]}),
                         ["surreal_disk:A:3"])

    def test_subset_cannot_shrink_full_denominator(self):
        with self.assertRaises(Blocked):
            sweep_contract(self.fixture(), ["case-0"])

    def test_mixed_source_success_does_not_pass(self):
        rows = [{"status": "PASS", "source_identity": "old", "artifact_identity": "a"},
                {"status": "PASS", "source_identity": "new", "artifact_identity": "b"}]
        self.assertEqual(aggregate(rows)["status"], "BLOCKED")

    def test_unresolved_failure_is_visible_with_later_pass(self):
        rows = [{"status": "FAIL", "source_identity": "s", "artifact_identity": "a"},
                {"status": "PASS", "source_identity": "s", "artifact_identity": "a"}]
        self.assertEqual(aggregate(rows)["status"], "FAIL")

    def test_dependency_block_is_not_pass(self):
        self.assertEqual(aggregate([{"status": "BLOCKED"}])["status"], "BLOCKED")


class CollectionLifecycleTests(unittest.TestCase):
    """Controller policy tests; fakes never stand in for production acceptance."""

    def run_collection(self, *, fail=False, cleanup_failure=False, dependent=False, correction=False,
                       borrowed=False, shared=None, settlement_failure=False, events=None,
                       context_failure=False):
        from types import SimpleNamespace
        from unittest.mock import patch
        from core import Case, Failed, Outcome, Predicate
        from failure_sweep import execute_collection

        fixture = SweepContractTests().fixture()
        fixture.update(source_identity="source", artifact_identity="artifact", limits={
            "wall_seconds": 10000, "http_requests": 100000, "total_output_bytes": 100000})
        fixture["backends"] = [{"name": "postgres_pgvector"}, {"name": "surreal_disk"}]
        if dependent:
            fixture["failure_sweep"]["prerequisites"]["case-1"] = ["case-0"]
        calls = []
        events = events if events is not None else []

        class FakeContext:
            def __init__(self, fixture, backend, run_id, evidence):
                if context_failure:
                    raise Blocked("context construction failed before dispatch")
                self.fixture = fixture
                self.backend, self.run_id = backend, run_id
                self.environment = {}
                self.requests = self.output_bytes = 0

            def cleanup(self):
                events.append("cleanup")
                self.requests += 1
                self.output_bytes += 2
                if cleanup_failure:
                    raise Blocked("cleanup ownership unknown")

        class FakePacket:
            def __init__(self, *args):
                self.process = SimpleNamespace(pid=123)
                self.ready = {"nonce": "nonce", "epoch": {"input_identity": "identity"}}

            def start(self):
                if borrowed:
                    raise AssertionError("borrowed packet owner was restarted")

            def alive(self):
                pass

            def settle(self, remaining):
                remaining()
                events.append("settle")
                if calls and settlement_failure:
                    raise Blocked("forwarding accounting unknown")

            def stop(self):
                if borrowed:
                    raise AssertionError("borrowed packet owner was stopped")

        class MemoryEvidence:
            def __init__(self):
                self.receipts = []
                self.secrets = []

            def write(self, name, value):
                events.append(name)
                self.receipts.append({"sha256": "receipt"})
                return str(len(self.receipts))

        def case_run(ctx):
            events.append("dispatch")
            calls.append((ctx.backend["name"], ctx.case_id, ctx.attempt))
            if (fail and ctx.fixture["source_identity"] == "source" and ctx.case_id == "case-0"
                    and ctx.attempt == 1 and ctx.backend["name"] == "postgres_pgvector"):
                raise Failed("observed failed predicate")
            return Outcome(Predicate(True, "protocol"), Predicate(True, "useful"))

        definitions = [Case(name, f"U{number % 25 + 1:02d}", case_run)
                       for number, name in enumerate(fixture["required_cases"])]
        refreshed = {**fixture, "source_identity": "corrected", "artifact_identity": "refreshed",
                     "_fixture_digest": "new-fixture"}
        authority = {"max_corrections": 1} if correction else None
        with patch("failure_sweep.Context", FakeContext), patch("failure_sweep.PacketProcesses", FakePacket), \
                patch("failure_sweep.correction_authority", return_value=authority), \
                patch("failure_sweep.await_correction", return_value={"host": "receipt"}), \
                patch("failure_sweep.refreshed_fixture", return_value=(refreshed, [])):
            result = execute_collection(fixture, MemoryEvidence(), "sweep", [],
                                        lambda fixture: definitions, lambda ctx: None, None,
                                        packet_owner=FakePacket() if borrowed else None,
                                        shared_state=shared)
        return result, calls

    def test_failure_continues_independent_items_without_same_source_retry(self):
        (code, summary), calls = self.run_collection(fail=True)
        self.assertEqual(code, 1)
        self.assertEqual(len(summary["inventory"]), 582)
        self.assertEqual(summary["counts"]["PASS"], 581)
        self.assertEqual(len(calls), 582)
        self.assertEqual(calls.count(("postgres_pgvector", "case-0", 1)), 1)

    def test_failed_prerequisite_blocks_only_matching_backend_repetition(self):
        (code, summary), calls = self.run_collection(fail=True, dependent=True)
        self.assertEqual(code, 1)
        self.assertEqual(summary["counts"]["BLOCKED"], 1)
        self.assertNotIn(("postgres_pgvector", "case-1", 1), calls)
        self.assertIn(("surreal_disk", "case-1", 1), calls)
        self.assertIn(("postgres_pgvector", "case-1", 2), calls)

    def test_unknown_cleanup_stops_independent_continuation(self):
        (code, summary), calls = self.run_collection(cleanup_failure=True)
        self.assertEqual(code, 2)
        self.assertTrue(summary["cleanup_unconfirmed"])
        self.assertEqual(len(calls), 1)
        self.assertEqual(summary["counts"]["NOT_RUN"], 581)

    def test_cleanup_and_ledger_settlement_precede_next_dispatch(self):
        events = []
        (code, summary), calls = self.run_collection(events=events)
        self.assertEqual(code, 0)
        first = events.index("dispatch")
        second = events.index("dispatch", first + 1)
        between = events[first + 1:second]
        self.assertLess(between.index("cleanup"), between.index("settle"))
        self.assertEqual(between.count("settle"), 2)
        self.assertLess(events.index("settle"), first)

    def test_unknown_settlement_retains_failure_and_cleanup_then_stops_before_correction(self):
        events = []
        (code, summary), calls = self.run_collection(
            fail=True, correction=True, settlement_failure=True, events=events)
        self.assertEqual(code, 2)
        self.assertTrue(summary["safety_stop"])
        self.assertEqual(len(calls), 1)
        self.assertEqual(summary["corrections_consumed"], 0)
        self.assertEqual(len(summary["failures"]), 1)
        self.assertTrue(summary["results"][0]["cleanup_confirmed"])
        self.assertEqual(summary["results"][0]["error"], "observed failed predicate")
        self.assertLess(events.index("sweep-failure"), events.index("sweep-settlement-failure"))
        self.assertEqual(summary["counts"]["NOT_RUN"], 581)

    def test_no_context_is_not_reported_as_confirmed_cleanup(self):
        (code, summary), calls = self.run_collection(context_failure=True)
        self.assertEqual(code, 2)
        self.assertEqual(calls, [])
        self.assertFalse(summary["results"][0]["cleanup_confirmed"])
        self.assertTrue(summary["cleanup_unconfirmed"])

    def test_one_authorized_refresh_reruns_only_the_failed_item(self):
        (code, summary), calls = self.run_collection(fail=True, correction=True)
        self.assertEqual(code, 0)
        self.assertEqual(summary["corrections_consumed"], 1)
        self.assertEqual(calls.count(("postgres_pgvector", "case-0", 1)), 2)
        self.assertEqual(len(calls), 583)
        self.assertEqual(len(summary["failures"]), 1)
        self.assertEqual(summary["counts"]["PASS"], 582)

    def test_borrowed_owner_preserves_counters_without_start_or_stop(self):
        import time
        shared = {"started": time.monotonic(), "requests": 21, "output_bytes": 31,
                  "corrections": 2, "max_corrections": 12}
        (code, summary), calls = self.run_collection(borrowed=True, shared=shared)
        self.assertEqual(code, 0)
        self.assertTrue(summary["borrowed_packet_owner"])
        self.assertEqual(shared["requests"], 21 + 582)
        self.assertEqual(shared["output_bytes"], 31 + 582 * 2)
        self.assertEqual(shared["corrections"], 2)

    def test_exhausted_shared_correction_cap_cannot_reset_for_next_platform(self):
        import time
        shared = {"started": time.monotonic(), "requests": 21, "output_bytes": 31,
                  "corrections": 12, "max_corrections": 12}
        (code, summary), calls = self.run_collection(fail=True, correction=True, borrowed=True, shared=shared)
        self.assertEqual(code, 1)
        self.assertEqual(summary["corrections_consumed"], 0)
        self.assertEqual(shared["corrections"], 12)
        self.assertEqual(len(calls), 582)


class CorrectionReceiptTests(unittest.TestCase):
    def test_receipt_binds_observed_failure_and_sweep(self):
        import json
        from pathlib import Path
        import tempfile
        from core import digest
        from failure_sweep import await_correction, item_id

        class Evidence:
            def write(self, name, value):
                pass

        row = {"backend": "surreal_disk", "case_id": "U01.configured-prompt", "attempt": 1,
               "sweep_run_id": "observed-sweep"}
        failure = {"path": "00001-failed.json", "sha256": "observed-failure-hash"}
        fixture = {"source_identity": "before-source", "artifact_identity": "before-artifact"}
        with tempfile.TemporaryDirectory() as directory:
            authority = {"receipt_directory": directory, "authority_id": "adopted-authority", "wait_seconds": 1}
            receipt = {"schema_version": 1, "authority_id": "adopted-authority", "run_id": "other-sweep",
                "item_id": item_id(row), "correction_number": 1, "failed_receipt": failure,
                "old_source_identity": "before-source", "old_artifact_identity": "before-artifact",
                "targeted_correction_completed": True, "artifact_refresh_completed": True,
                "description": "One bounded source correction and refreshed artifact"}
            receipt["seal"] = digest(json.dumps(receipt, sort_keys=True, separators=(",", ":"), allow_nan=False).encode())
            path = Path(directory) / (digest(item_id(row).encode()).split(":")[1] + ".json")
            path.write_text(json.dumps(receipt))
            path.chmod(0o400)
            with self.assertRaises(Blocked):
                await_correction(authority, row, failure, fixture, Evidence(), lambda: 1)

    def test_without_authority_no_wait_or_retry_is_requested(self):
        from failure_sweep import await_correction
        self.assertIsNone(await_correction(None, {}, {}, {}, None, None))


if __name__ == "__main__":
    unittest.main()
