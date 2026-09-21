#!/usr/bin/env python3
"""Verify the frozen routing and compaction acceptance contract."""

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
from collections import Counter
from typing import Any


WORKSPACE = pathlib.Path(__file__).resolve().parents[7]
CONTRACT_PATH = (
    WORKSPACE
    / ".kbd-orchestrator/phases/runtime-harness-gap-closure/acceptance-contract.json"
)
EVIDENCE = (
    WORKSPACE
    / ".kbd-orchestrator/phases/runtime-harness-gap-closure/evidence/execute/"
    "harness-model-profiles-routing/1.2"
)


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical_sha(value: Any) -> str:
    encoded = json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


contract = json.loads(CONTRACT_PATH.read_text())
frozen = json.loads((EVIDENCE / "frozen-baseline.json").read_text())
assert contract["version"] == "harness-gap-eval-v1"
assert contract["status"] == "frozen_inputs_and_thresholds_not_executed"
assert contract["baseline"]["sourceRevision"] == "226d4a0af89811975662cf3f203c699f70e8cebc"
assert contract["baseline"]["measurements"] == "NOT_RUN"
assert frozen["acceptanceContract"]["sha256"] == sha(CONTRACT_PATH)
assert frozen["acceptanceContract"]["version"] == contract["version"]
assert frozen["acceptanceContract"]["status"] == contract["status"]

routing_cases = {case["id"]: case for case in contract["routingCases"]}
task_cases = {case["id"]: case for case in contract["routingTaskCases"]}
coverage = contract["routingConstraintCoverage"]
assert len(routing_cases) == 18
assert len(task_cases) == 8
assert frozen["datasets"]["routingAdmissionCases"] == len(routing_cases)
assert frozen["datasets"]["routingTaskQualityCases"] == len(task_cases)

for category in (
    "explicitModelCases",
    "tenantPolicyCases",
    "sharedCostBudgetCases",
    "sharedLatencyBudgetCases",
    "protectedFitCases",
):
    assert coverage[category]
    assert set(coverage[category]) <= set(routing_cases)
assert set(coverage["qualityCases"]) == set(task_cases)
assert frozen["datasets"]["explicitModelCaseIds"] == coverage["explicitModelCases"]
assert frozen["datasets"]["tenantPolicyCaseIds"] == coverage["tenantPolicyCases"]
assert frozen["datasets"]["sharedCostBudgetCaseIds"] == coverage["sharedCostBudgetCases"]
assert frozen["datasets"]["sharedLatencyBudgetCaseIds"] == coverage["sharedLatencyBudgetCases"]
assert frozen["datasets"]["qualityCaseIds"] == coverage["qualityCases"]

for case_id in coverage["explicitModelCases"]:
    assert routing_cases[case_id]["requirements"]["explicit"]
for case_id in coverage["tenantPolicyCases"]:
    requirements = routing_cases[case_id]["requirements"]
    assert requirements["tenant_id"]
    assert requirements["tenant_allowed"]
for case_id in coverage["sharedCostBudgetCases"]:
    shared = routing_cases[case_id]["requirements"]["shared_budget"]
    assert "max_cost_units" in shared
    assert set(shared["spent"]) == {
        "root",
        "classifier",
        "summaries",
        "retries",
        "children",
    }
    assert "candidate_cost_units" in shared
for case_id in coverage["sharedLatencyBudgetCases"]:
    shared = routing_cases[case_id]["requirements"]["shared_budget"]
    assert "max_latency_ms" in shared
    assert set(shared["elapsed_ms"]) == {
        "root",
        "classifier",
        "summaries",
        "retries",
        "children",
    }
    assert "candidate_latency_ms" in shared

assert routing_cases["R13"]["expected"] == "forbidden"
assert routing_cases["R14"]["expected"] == "B"
assert routing_cases["R15"]["expected"] == "budget"
assert routing_cases["R16"]["expected"] == "B"
assert routing_cases["R17"]["expected"] == "budget"
assert routing_cases["R18"]["expected"] == "A"

thresholds = contract["thresholds"]
for threshold in (
    "route_constraints_pass_rate",
    "route_explicit_model_integrity",
    "route_tenant_policy_integrity",
    "route_shared_budget_accounting_accuracy",
    "route_task_success_delta_min",
    "route_quality_or_cost_or_latency_improvement",
    "context_protected_fact_retention",
    "context_task_success_delta_min",
    "context_total_cost_ratio_max",
    "context_p95_latency_ratio_max",
):
    assert threshold in thresholds
assert thresholds["route_constraints_pass_rate"] == 1
assert thresholds["route_explicit_model_integrity"] == 1
assert thresholds["route_tenant_policy_integrity"] == 1
assert thresholds["route_shared_budget_accounting_accuracy"] == 1
assert thresholds["context_protected_fact_retention"] == 1
assert thresholds["route_task_success_delta_min"] == 0
assert thresholds["context_task_success_delta_min"] == 0
assert thresholds["context_total_cost_ratio_max"] == 1.1
assert thresholds["context_p95_latency_ratio_max"] == 1.1

measurement = contract["measurement"]
assert measurement["repetitions"] == 3
assert all(
    component in measurement["sharedBudget"]
    for component in ("root", "classifier", "summaries", "retries", "children")
)
assert "nearest-rank" in measurement["latency"]
assert "price revision" in measurement["cost"]
assert "BLOCKED" in measurement["liveRuns"]
assert "never divide by zero" in measurement["zeroBaseline"]

context = contract["contextCorpus"]
generator_contract = context["generator"]
generator_path = WORKSPACE / generator_contract["path"]
manifest_path = WORKSPACE / generator_contract["materializedManifestPath"]
assert generator_contract["version"] == "context-corpus-v1"
assert sha(generator_path) == generator_contract["sha256"]
assert sha(manifest_path) == generator_contract["materializedManifestSha256"]
assert frozen["contextCorpus"]["generator"]["sha256"] == sha(generator_path)
assert frozen["contextCorpus"]["manifest"]["sha256"] == sha(manifest_path)

regeneration = subprocess.run(
    [sys.executable, str(generator_path), "--check", str(manifest_path)],
    check=False,
    capture_output=True,
    text=True,
)
assert regeneration.returncode == 0, regeneration.stderr or regeneration.stdout
assert regeneration.stdout.startswith("PASS ")

manifest = json.loads(manifest_path.read_text())
assert manifest["schemaVersion"] == 1
assert manifest["generatorVersion"] == generator_contract["version"]
assert manifest["seed"] == context["seed"] == 17
assert set(context["strategies"]) == {
    "SlidingWindow",
    "KeepFirstLast",
    "progressive summarization",
}
assert set(context["persistence"]) == {"fresh", "reload", "checkpoint"}
assert set(context["entryPoints"]) == {
    "initial",
    "iteration",
    "retry",
    "failover",
    "graph",
    "resume",
}
assert context["payloads"] == manifest["generatorContract"]["payloads"]
assert context["sizeBytes"] == manifest["generatorContract"]["sizes"]
assert context["parallelCalls"] == manifest["generatorContract"]["parallelCalls"]
assert "0x0D 0x0A" in context["generatorAssertions"][0]
raw_json = next(payload for payload in context["payloads"] if payload["id"] == "raw-json")
assert b"\r\n" in raw_json["baseUtf8"].encode("utf-8")

counts = manifest["counts"]
assert counts == {
    "cartesianCases": 945,
    "longHistoryRecords": 1004,
    "failureCases": 198,
    "budgetCases": 54,
    "normalEntryPointBindings": 18,
}
assert context["caseCounts"] == {
    "cartesian": counts["cartesianCases"],
    "failure": counts["failureCases"],
    "budget": counts["budgetCases"],
    "longHistoryRecords": counts["longHistoryRecords"],
    "normalEntryPointBindings": counts["normalEntryPointBindings"],
}
assert frozen["datasets"]["contextCartesianCases"] == counts["cartesianCases"]
assert frozen["datasets"]["contextFailureCases"] == counts["failureCases"]
assert frozen["datasets"]["contextBudgetEntryPointCases"] == counts["budgetCases"]
assert frozen["datasets"]["contextLongHistoryRecords"] == counts["longHistoryRecords"]
assert frozen["datasets"]["contextNormalEntryPointBindings"] == counts["normalEntryPointBindings"]

for section in (
    "cartesianCases",
    "normalEntryPointCoverage",
    "longHistory",
    "failureCases",
    "budgetCases",
):
    assert manifest["aggregateSha256"][section] == canonical_sha(manifest[section])
assert frozen["contextCorpus"]["aggregateSha256"] == manifest["aggregateSha256"]

cartesian_cases = manifest["cartesianCases"]
assert len({case["id"] for case in cartesian_cases}) == len(cartesian_cases)
assert all(case["inputSha256"] == canonical_sha(case["input"]) for case in cartesian_cases)
assert all(case["expected"]["outcome"] == "protected_records_preserved" for case in cartesian_cases)
assert all(case["expected"]["rawPayloadSha256"] == case["rawPayloadSha256"] for case in cartesian_cases)
assert all(case["expected"]["protectedMutationCount"] == 0 for case in cartesian_cases)
assert all(case["expected"]["protectedDropCount"] == 0 for case in cartesian_cases)
assert Counter(case["input"]["payloadId"] for case in cartesian_cases) == Counter(
    {payload["id"]: 135 for payload in context["payloads"]}
)
for case in cartesian_cases:
    case_input = case["input"]
    records = case_input["records"]
    assert [record["id"] for record in records] == case_input["recordIdsInOrder"]
    payload_record = records[0]
    assert payload_record["role"] == "user"
    assert payload_record["kind"] == case_input["payloadKind"]
    assert payload_record["protection"] == "protected"
    assert payload_record["content"]["rawBytes"] == case["rawPayloadBytes"]
    assert payload_record["content"]["rawSha256"] == case["rawPayloadSha256"]
    assert payload_record["content"]["wireBytes"] == case["wirePayloadBytes"]
    assert payload_record["content"]["wireSha256"] == case["wirePayloadSha256"]
    reconstruction = payload_record["content"]["reconstruction"]
    assert reconstruction["baseUtf8"] == case_input["baseUtf8"]
    assert reconstruction["targetRawBytes"] == case_input["targetRawBytes"]
    call_records = records[1::2]
    result_records = records[2::2]
    assert len(call_records) == len(result_records) == len(case_input["parallelCallIds"])
    assert [record["callId"] for record in call_records] == case_input["parallelCallIds"]
    assert [record["callId"] for record in result_records] == case_input["parallelCallIds"]
    for ordinal, (call_record, result_record) in enumerate(zip(call_records, result_records)):
        assert call_record["role"] == "assistant"
        assert call_record["kind"] == "tool-call"
        assert call_record["protection"] == "protected"
        assert call_record["toolName"] == "fixture.echo"
        arguments_sha = hashlib.sha256(
            call_record["argumentsRaw"].encode("utf-8")
        ).hexdigest()
        assert arguments_sha == call_record["argumentsRawSha256"]
        arguments = json.loads(call_record["argumentsRaw"])
        assert arguments == {
            "caseId": case["id"],
            "ordinal": ordinal,
            "payloadSha256": case["rawPayloadSha256"],
        }
        assert result_record["role"] == "tool"
        assert result_record["kind"] == "tool-result"
        assert result_record["protection"] == "protected"
        assert result_record["parallelGroupId"] == call_record["parallelGroupId"]
        assert hashlib.sha256(result_record["contentRaw"].encode("utf-8")).hexdigest() == result_record["contentRawSha256"]

normal_coverage = manifest["normalEntryPointCoverage"]
assert len(normal_coverage) == 18
assert {
    (row["strategy"], row["entryPoint"]) for row in normal_coverage
} == {
    (strategy, entry_point)
    for strategy in context["strategies"]
    for entry_point in context["entryPoints"]
}
for row in normal_coverage:
    expected_ids = [
        case["id"]
        for case in cartesian_cases
        if case["input"]["strategy"] == row["strategy"]
        and case["input"]["entryPoint"] == row["entryPoint"]
    ]
    assert row["caseIds"] == expected_ids
    assert row["caseCount"] == len(expected_ids) > 0
    assert row["caseIdsSha256"] == canonical_sha(expected_ids)

history = manifest["longHistory"]
assert history["id"] == context["longHistory"]["manifestId"]
assert history["recordsSha256"] == canonical_sha(history["records"])
assert history["recordsSha256"] == context["longHistory"]["recordsSha256"]
assert [
    record["id"]
    for record in history["records"]
    if record["protection"] == "protected"
] == history["expected"]["protectedRecordIdsInOrder"]
assert history["expected"]["eligibleRecordCount"] == 1_000
assert history["expected"]["protectedRecordCount"] == 4
assert history["expected"]["distinctContinueRecords"] == 2

failure_cases = manifest["failureCases"]
failure_counts = Counter(case["failure"] for case in failure_cases)
assert failure_counts == Counter({failure: 18 for failure in context["failureCases"]})
assert len({case["id"] for case in failure_cases}) == len(failure_cases)
assert all(case["inputSha256"] == canonical_sha(case["input"]) for case in failure_cases)
expected_failure_outcomes = {
    "pending_without_result": "pending_or_recovery_required",
    "verified_cancelled": "typed_cancelled_result_recorded",
    "orphan": "invalid_history",
    "duplicate_result": "invalid_history",
    "summary_error": "summary_failed",
    "summary_empty": "summary_empty",
    "summary_oversized": "summary_oversized",
    "summary_cancelled": "summary_cancelled",
    "unknown_media_count": "unsupported_count",
    "revoked_before_dispatch": "blocked_authorization",
    "smaller_failover": "protected_overflow",
}
assert all(
    case["expected"]["outcome"] == expected_failure_outcomes[case["failure"]]
    for case in failure_cases
)

budget_cases = manifest["budgetCases"]
assert len({case["id"] for case in budget_cases}) == len(budget_cases)
assert all(case["inputSha256"] == canonical_sha(case["input"]) for case in budget_cases)
assert Counter(case["fixture"] for case in budget_cases) == Counter(
    {fixture["id"]: 18 for fixture in context["budgetCases"]}
)
for case in budget_cases:
    budget_input = case["input"]
    expected = case["expected"]
    reserve_sum = budget_input["O"] + budget_input["R"] + budget_input["M"]
    if reserve_sum > budget_input["C"]:
        assert expected == {
            "outcome": "invalid_limits",
            "field": "reserve_sum",
            "value": reserve_sum,
        }
        continue
    ceilings = [budget_input["C"] - reserve_sum]
    ceilings.extend(
        value for value in (budget_input["L"], budget_input["H"]) if value is not None
    )
    allowance = min(ceilings)
    if budget_input["F"] > allowance:
        assert expected == {
            "outcome": "protected_overflow",
            "requiredTokens": budget_input["F"],
            "inputAllowance": allowance,
        }
    else:
        assert expected["I"] == allowance
        assert expected["B"] == allowance - budget_input["F"]
        assert expected["outcome"] == "fit"

for artifact in contract["baseline"]["resultArtifacts"].values():
    path = WORKSPACE / artifact["path"]
    assert sha(path) == artifact["sha256"]
    assert artifact["boundary"]
for target in contract["baseline"]["verificationTargets"].values():
    path = WORKSPACE / target["path"]
    assert sha(path) == target["sha256"]
assert frozen["fixtureSources"] == {
    target["path"]: target["sha256"]
    for target in contract["baseline"]["verificationTargets"].values()
}

assert (EVIDENCE / "prompt_assembly-initial-failure.exit").read_text().strip() == "101"
assert "queries overflow the depth limit" in (
    EVIDENCE / "prompt_assembly-initial-failure.log"
).read_text()
assert (EVIDENCE / "cargo-check.exit").read_text().strip() == "0"
assert (EVIDENCE / "prompt_assembly.exit").read_text().strip() == "0"
assert (EVIDENCE / "model_path_resiliency.exit").read_text().strip() == "0"
assert "6 passed; 0 failed" in (EVIDENCE / "prompt_assembly.log").read_text()
assert "12 passed; 0 failed" in (EVIDENCE / "model_path_resiliency.log").read_text()

print("PASS frozen routing/compaction acceptance contract")
print(regeneration.stdout.strip())
print("routing_cases=18 task_quality_cases=8")
print("context_cartesian_cases=945 context_failure_cases=198 context_budget_cases=54")
print("context_long_history_records=1004")
print(
    "explicit_model_cases=10 tenant_policy_cases=3 "
    "shared_cost_cases=3 shared_latency_cases=1"
)
print(
    "thresholds=quality,cost,latency,explicit_model,tenant,"
    "shared_budget,protected_retention"
)
print(f"acceptance_contract_sha256={sha(CONTRACT_PATH)}")
print(f"context_corpus_manifest_sha256={sha(manifest_path)}")
print(f"frozen_baseline_sha256={sha(EVIDENCE / 'frozen-baseline.json')}")
