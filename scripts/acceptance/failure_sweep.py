"""Opt-in serial collection. No commands or resources are opened on import.

Only the trusted host can perform a correction. This controller consumes a
bounded receipt; it never generates a repair or dispatches a repair command.
"""
from __future__ import annotations

import json
import math
import os
from pathlib import Path
import time
import uuid

from core import Blocked, Context, Failed, Outcome, digest
from packet_processes import PacketProcesses


def item_id(row):
    return f"{row['backend']}:{row['case_id']}:{row['attempt']}"


def dependency_ids(row, prerequisites):
    return [f"{row['backend']}:{name}:{row['attempt']}"
            for name in prerequisites[row["case_id"]]]


def sweep_contract(fixture, selected):
    names = fixture.get("required_cases", [])
    if selected or len(names) != 97 or len(set(names)) != 97:
        raise Blocked("collection requires all 97 declared cases; --case cannot shrink its denominator")
    spec = fixture.get("failure_sweep", {})
    prerequisites = spec.get("prerequisites")
    if spec.get("schema_version") != 1 or not isinstance(prerequisites, dict) or set(prerequisites) != set(names):
        raise Blocked("collection requires an explicit prerequisite list for every case")
    preceding = set()
    for name in names:
        needed = prerequisites[name]
        if (not isinstance(needed, list) or any(not isinstance(value, str) for value in needed)
                or len(set(needed)) != len(needed) or not set(needed).issubset(preceding)):
            raise Blocked("prerequisites must be unique earlier case IDs in serial manifest order")
        preceding.add(name)
    return spec


def aggregate(rows):
    counts = {name: sum(row["status"] == name for row in rows)
              for name in ("PASS", "FAIL", "BLOCKED", "NOT_RUN")}
    identities = {(row.get("source_identity"), row.get("artifact_identity"))
                  for row in rows if row["status"] == "PASS"}
    mixed = len(identities) > 1
    status = ("FAIL" if counts["FAIL"] else "BLOCKED"
              if mixed or counts["BLOCKED"] or counts["NOT_RUN"] or not rows else "PASS")
    return {"status": status, "counts": counts, "mixed_source": mixed,
            "coherent_full_rerun_required": mixed}


def private_json(reference, maximum=1024 * 1024):
    path = Path(reference["path"])
    if (not path.is_absolute() or path.is_symlink() or not path.is_file()
            or path.stat().st_uid != os.getuid()
            or path.stat().st_mode & 0o077 or path.stat().st_size > maximum):
        raise Blocked("correction input must be a bounded private absolute regular file")
    raw = path.read_bytes()
    if digest(raw) != reference["sha256"]:
        raise Blocked("correction input seal differs from the approved reference")
    return json.loads(raw)


def correction_authority(fixture, spec):
    reference = spec.get("correction_authority")
    if reference is None:
        return None
    frozen = {item["path"]: item["sha256"] for item in fixture["frozen_files"]}
    if frozen.get(reference.get("path")) != reference.get("sha256"):
        raise Blocked("correction authority must be a frozen operator approval input")
    authority = private_json(reference)
    wait = authority.get("wait_seconds")
    maximum = authority.get("max_corrections")
    root = Path(authority.get("receipt_directory", ""))
    refreshable = authority.get("refreshable_paths")
    if (authority.get("schema_version") != 1 or authority.get("operator_approved") is not True
            or not isinstance(authority.get("authority_id"), str) or not authority["authority_id"]
            or authority.get("source_identity") != fixture["source_identity"]
            or authority.get("artifact_identity") != fixture["artifact_identity"]
            or authority.get("epoch_id") != fixture["packet_processes"]["budget_proxy"]["epoch_id"]
            or type(maximum) is not int or not 1 <= maximum <= 582
            or type(wait) not in (int, float) or not math.isfinite(wait) or not 0 < wait <= 3600
            or not isinstance(refreshable, list) or not refreshable
            or any(not isinstance(value, str) or value not in frozen for value in refreshable)
            or len(set(refreshable)) != len(refreshable)
            or not root.is_absolute() or root.is_symlink() or not root.is_dir()
            or root.stat().st_uid != os.getuid()
            or root.stat().st_mode & 0o077):
        raise Blocked("explicit bounded correction approval or private host receipt directory is invalid")
    return authority


def await_correction(authority, row, failure, fixture, evidence, remaining):
    """Wait within the live epoch for a host-produced, self-sealed receipt.

    Directory ownership and the separately frozen operator authority admit the
    trusted writer. A self seal is tamper evidence, never itself authorization.
    """
    if authority is None:
        return None
    token = digest(item_id(row).encode()).split(":", 1)[1]
    path = Path(authority["receipt_directory"]) / (token + ".json")
    deadline = time.monotonic() + min(authority["wait_seconds"], remaining())
    evidence.write("correction-request", {"item_id": item_id(row), "failure": failure,
        "receipt_path": str(path), "authority_id": authority["authority_id"],
        "deadline_remaining_seconds": deadline - time.monotonic()})
    while time.monotonic() < deadline:
        remaining()
        if path.exists():
            if (path.is_symlink() or not path.is_file() or path.stat().st_uid != os.getuid()
                    or path.stat().st_mode & 0o277 or path.stat().st_size > 1024 * 1024):
                raise Blocked("host correction receipt is not sealed/private/bounded")
            raw = path.read_bytes()
            receipt = json.loads(raw)
            seal = receipt.pop("seal", None)
            canonical = json.dumps(receipt, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
            if (seal != digest(canonical) or receipt.get("schema_version") != 1
                    or receipt.get("authority_id") != authority["authority_id"]
                    or receipt.get("run_id") != row["sweep_run_id"]
                    or receipt.get("item_id") != item_id(row) or receipt.get("correction_number") != 1
                    or receipt.get("failed_receipt") != failure
                    or receipt.get("old_source_identity") != fixture["source_identity"]
                    or receipt.get("old_artifact_identity") != fixture["artifact_identity"]
                    or receipt.get("targeted_correction_completed") is not True
                    or receipt.get("artifact_refresh_completed") is not True
                    or not isinstance(receipt.get("description"), str) or not receipt["description"]):
                raise Blocked("host correction receipt does not authorize this exact failed item")
            evidence.write("consumed-correction", {"receipt": receipt, "seal": seal, "sha256": digest(raw)})
            return receipt
        time.sleep(min(0.1, max(0, deadline - time.monotonic())))
    evidence.write("correction-unavailable", {"item_id": item_id(row), "failure": failure})
    return None


def refreshed_fixture(old, receipt, loader):
    reference = receipt["refreshed_fixture"]
    # Read the declared seal before loading any revised process or case inputs.
    private_json(reference, 64 * 1024 * 1024)
    new, secrets = loader(Path(reference["path"]))
    mutable = {"source_identity", "artifact_identity", "configuration_revision", "frozen_files"}
    old_contract = {key: value for key, value in old.items() if not key.startswith("_") and key not in mutable}
    new_contract = {key: value for key, value in new.items() if not key.startswith("_") and key not in mutable}
    if (old_contract != new_contract or new["source_identity"] == old["source_identity"]
            or new["artifact_identity"] == old["artifact_identity"]
            or receipt.get("new_source_identity") != new["source_identity"]
            or receipt.get("new_artifact_identity") != new["artifact_identity"]):
        raise Blocked("correction must refresh source and artifact without changing approved commands or allowances")
    before = {item["path"]: item["sha256"] for item in old["frozen_files"]}
    after = {item["path"]: item["sha256"] for item in new["frozen_files"]}
    if set(before) != set(after):
        raise Blocked("correction cannot replace the approved input path inventory")
    authority_reference = old["failure_sweep"]["correction_authority"]
    authority = private_json(authority_reference)
    refreshable = set(authority["refreshable_paths"])
    immutable = {authority_reference["path"]}
    for spec in old["packet_processes"].values():
        immutable.update(value for value in spec.get("argv", []) if value in before)
    for path, seal in before.items():
        # Already imported code and the live packet owner must stay immutable.
        if after[path] != seal and (path not in refreshable or path in immutable
                                   or Path(path).suffix in (".py", ".mjs")):
            raise Blocked("correction changed an unapproved input or live controller/helper; a fresh packet is required")
    return new, secrets


def execute_collection(fixture, evidence, run_id, selected, load_cases, admission, load_fixture,
                       packet_owner=None, shared_state=None):
    rows = []
    summary = {"run_id": run_id, "mode": "collect_results", "phase_acceptance": False,
               "scope": "declared_manifest", "inventory": rows, "results": [], "failures": [],
               "status": "BLOCKED", "denominator": 582}
    packet = packet_owner
    owns_packet = packet_owner is None
    started = time.monotonic()
    state = shared_state if shared_state is not None else {
        "started": started, "requests": 0, "output_bytes": 0,
        "corrections": 0, "max_corrections": 582}
    requests = output = corrections = 0
    spec = None
    active = fixture
    unavailable_backends = {}

    def remaining():
        seconds = fixture["limits"]["wall_seconds"] - (time.monotonic() - started)
        if seconds <= 0 or requests >= fixture["limits"]["http_requests"] or output >= fixture["limits"]["total_output_bytes"]:
            raise Blocked("shared collection allowance exhausted; no independent continuation")
        if packet is not None:
            packet.alive()
        return seconds

    def log_result(row, result):
        if "revision_attempt" in result:
            for key in ("error", "error_type", "reason", "protocol", "useful", "facts",
                        "cleanup_error", "isolated_run_id"):
                row.pop(key, None)
        row.update(result)
        snapshot = dict(row)
        summary["results"].append(snapshot)
        name = evidence.write("sweep-item-terminal", snapshot)
        return {"path": name, "sha256": evidence.receipts[-1]["sha256"]}

    def attempt(row, revision):
        nonlocal requests, output
        remaining()
        packet.settle(remaining)
        ctx = None
        result = {"status": "BLOCKED", "source_identity": active["source_identity"],
                  "artifact_identity": active["artifact_identity"], "revision_attempt": revision,
                  "cleanup_confirmed": False, "admission_status": "BLOCKED"}
        evidence.write("sweep-attempt-start", {"item_id": item_id(row), **result})
        try:
            backend = next(value for value in active["backends"] if value["name"] == row["backend"])
            ctx = Context(active, backend, str(uuid.uuid4()), evidence)
            ctx.environment["UAR_ACCEPTANCE_BUDGET_PID"] = str(packet.process.pid)
            ctx.environment["UAR_ACCEPTANCE_BUDGET_NONCE"] = packet.ready["nonce"]
            ctx.environment["UAR_ACCEPTANCE_BUDGET_INPUT_IDENTITY"] = packet.ready["epoch"]["input_identity"]
            ctx.started, ctx.requests, ctx.output_bytes = started, requests, output
            result["isolated_run_id"] = ctx.run_id
            admission(ctx)
            result["admission_status"] = "PASS"
            ctx.case_id, ctx.attempt = row["case_id"], row["attempt"]
            outcome = available[row["case_id"]].run(ctx)
            if not isinstance(outcome, Outcome):
                raise Blocked("implemented case returned no outcome predicates")
            passed = outcome.protocol.passed is True and outcome.useful.passed is True
            result.update(status="PASS" if passed else "FAIL", protocol=vars(outcome.protocol),
                          useful=vars(outcome.useful), facts=outcome.facts)
        except Exception as error:
            result.update(status="BLOCKED" if isinstance(error, Blocked) else "FAIL",
                          error_type=type(error).__name__, error=str(error))
        finally:
            if ctx is not None:
                try:
                    ctx.cleanup()
                    result["cleanup_confirmed"] = True
                except BaseException as error:
                    result.update(status="BLOCKED", cleanup_error=type(error).__name__)
                requests, output = ctx.requests, ctx.output_bytes
                state["requests"], state["output_bytes"] = requests, output
        result["prerequisite_outcomes"] = list(row["prerequisite_outcomes"])
        result["prerequisite_outcomes"].append({"item_id": "admission:" + row["backend"],
                                               "status": result["admission_status"]})
        if result["admission_status"] != "PASS":
            result["status"] = "BLOCKED"
        receipt = log_result(row, result)
        if row["status"] != "PASS":
            failure = {"item_id": row["item_id"], "revision_attempt": revision, "receipt": receipt}
            summary["failures"].append(failure)
            evidence.write("sweep-failure", failure)
        if not result["cleanup_confirmed"]:
            summary["cleanup_unconfirmed"] = True
            raise Blocked("owned cleanup unconfirmed; no further correction or independent dispatch")
        try:
            packet.settle(remaining)
        except Exception as error:
            evidence.write("sweep-settlement-failure", {"item_id": row["item_id"],
                "revision_attempt": revision, "terminal_receipt": receipt,
                "cleanup_confirmed": True, "error_type": type(error).__name__})
            raise Blocked("shared forwarding settlement unconfirmed after owned cleanup; no continuation") from error
        remaining()
        return receipt

    try:
        if packet_owner is not None and shared_state is None:
            raise Blocked("a borrowed packet owner requires its shared counters and start time")
        if (not isinstance(state, dict)
                or type(state.get("started")) not in (int, float) or not math.isfinite(state["started"])
                or any(type(state.get(key)) is not int or state[key] < 0
                       for key in ("requests", "output_bytes", "corrections", "max_corrections"))
                or state["corrections"] > state["max_corrections"]):
            raise Blocked("shared packet counters, correction cap or monotonic start are invalid")
        started = state["started"]
        requests, output = state["requests"], state["output_bytes"]
        spec = sweep_contract(fixture, selected)
        available = {case.case_id: case for case in load_cases(fixture)}
        if not set(fixture["required_cases"]).issubset(available):
            raise Blocked("collection manifest includes unimplemented cases")
        if {available[name].family for name in fixture["required_cases"]} != {f"U{number:02d}" for number in range(1, 26)}:
            raise Blocked("collection must retain all 25 outcome families")
        for backend in fixture["backends"]:
            for name in fixture["required_cases"]:
                for repetition in range(1, 4):
                    row = {"backend": backend["name"], "case_id": name, "family": available[name].family,
                           "attempt": repetition, "status": "NOT_RUN", "sweep_run_id": run_id}
                    row["item_id"] = item_id(row)
                    row["dependency_ids"] = dependency_ids(row, spec["prerequisites"])
                    rows.append(row)
        evidence.write("instantiated-case-manifest", rows)
        authority = correction_authority(fixture, spec)
        if owns_packet:
            packet = PacketProcesses(fixture, evidence, run_id, started)
            packet.start()
        indexed = {row["item_id"]: row for row in rows}
        for row in rows:
            remaining()
            dependencies = [indexed[name] for name in row["dependency_ids"]]
            row["prerequisite_outcomes"] = [{"item_id": value["item_id"], "status": value["status"],
                "source_identity": value.get("source_identity"), "artifact_identity": value.get("artifact_identity")}
                for value in dependencies]
            row["prerequisite_outcomes"].append({"item_id": "packet", "status": "PASS"})
            if row["backend"] in unavailable_backends:
                row["prerequisite_outcomes"].append(unavailable_backends[row["backend"]])
                log_result(row, {"status": "BLOCKED", "reason": "backend_admission_unavailable"})
                continue
            if any(value["status"] != "PASS" or value.get("source_identity") != active["source_identity"]
                   or value.get("artifact_identity") != active["artifact_identity"] for value in dependencies):
                log_result(row, {"status": "BLOCKED", "reason": "prerequisite_not_passed_on_current_source"})
                continue
            failure = attempt(row, 0)
            if row["admission_status"] != "PASS":
                unavailable_backends[row["backend"]] = {
                    "item_id": "admission:" + row["backend"], "status": "BLOCKED", "receipt": failure}
            if row["status"] == "PASS":
                continue
            # A missing prerequisite is not a product failure and receives no repair slot.
            if (row["status"] == "FAIL" and authority is not None
                    and corrections < authority["max_corrections"]
                    and state["corrections"] < state["max_corrections"]):
                try:
                    receipt = await_correction(authority, row, failure, active, evidence, remaining)
                    if receipt is not None:
                        corrections += 1
                        state["corrections"] += 1
                        active, secrets = refreshed_fixture(active, receipt, load_fixture)
                        evidence.secrets = sorted(set(evidence.secrets + secrets), key=len, reverse=True)
                        evidence.write("sweep-artifact-refresh", {"item_id": row["item_id"],
                            "source_identity": active["source_identity"], "artifact_identity": active["artifact_identity"],
                            "fixture_digest": active["_fixture_digest"], "correction_number": corrections})
                        attempt(row, 1)
                except (Blocked, OSError, ValueError, KeyError, TypeError) as error:
                    # A malformed authority/refresh can leave the old frozen source invalid.
                    # Stop the affected packet rather than dispatching unadmitted inputs.
                    evidence.write("correction-rejected", {"item_id": row["item_id"], "error_type": type(error).__name__})
                    raise Blocked("correction authority or refreshed frozen inputs could not be admitted") from error
            if row["status"] != "PASS":
                evidence.write("sweep-unresolved", {"item_id": row["item_id"], "status": row["status"],
                    "reason": "no_authorized_correction_or_same_item_rerun_unresolved", "receipt": failure})
    except BaseException as error:
        summary.update(error_type=type(error).__name__, error=str(error), safety_stop=True)
        evidence.write("sweep-safety-stop", {"error_type": type(error).__name__, "error": str(error)})
    finally:
        if owns_packet and packet is not None:
            try:
                packet.stop()
            except BaseException as error:
                summary.update(cleanup_unconfirmed=True, safety_stop=True)
                evidence.write("packet-cleanup-failure", {"type": type(error).__name__})
    summary.update(aggregate(rows))
    summary["corrections_consumed"] = corrections
    summary["shared_corrections_consumed"] = state.get("corrections") if isinstance(state, dict) else None
    summary["borrowed_packet_owner"] = not owns_packet
    summary["final_source_identity"] = active["source_identity"]
    summary["final_artifact_identity"] = active["artifact_identity"]
    summary["final_fixture_path"] = active.get("_fixture_path")
    summary["final_fixture_digest"] = active.get("_fixture_digest")
    if summary.get("safety_stop") or len(rows) != 582:
        summary["status"] = "BLOCKED"
    evidence.write("sweep-aggregate", summary)
    return {"PASS": 0, "FAIL": 1, "BLOCKED": 2}[summary["status"]], summary
