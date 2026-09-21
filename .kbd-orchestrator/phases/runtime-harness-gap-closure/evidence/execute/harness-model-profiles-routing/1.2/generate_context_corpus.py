#!/usr/bin/env python3
"""Generate the deterministic context corpus frozen by task 1.2."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import sys
from itertools import product
from typing import Any


GENERATOR_VERSION = "context-corpus-v1"
SEED = 17
STRATEGIES = {
    "SlidingWindow": "sliding-window",
    "KeepFirstLast": "keep-first-last",
    "progressive summarization": "progressive-summarization",
}
PERSISTENCE = ("fresh", "reload", "checkpoint")
ENTRY_POINTS = ("initial", "iteration", "retry", "failover", "graph", "resume")
SIZES = (0, 31_999, 32_000, 32_001, 1_048_576)
PARALLEL_CALLS = (1, 2, 4)
PAYLOADS = (
    {"id": "empty", "kind": "text", "baseUtf8": ""},
    {"id": "raw-json", "kind": "json", "baseUtf8": "{\r\n\"x\":1}"},
    {"id": "unicode", "kind": "text", "baseUtf8": "Unicode:雪🙂é"},
    {"id": "code", "kind": "code", "baseUtf8": "code:print('retain')"},
    {"id": "log", "kind": "log", "baseUtf8": "log:ERROR stable-id-17"},
    {"id": "file", "kind": "file", "baseUtf8": "file:/fixture/report.txt"},
    {
        "id": "media",
        "kind": "media",
        "baseUtf8": "media:sha256:fixture-media-17",
    },
)


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def sha_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha_value(value: Any) -> str:
    return sha_bytes(canonical_bytes(value))


def expand_payload(base_utf8: str, target_size: int) -> bytes:
    """Repeat complete UTF-8 strings, then fill the remainder with ASCII '~'."""
    base = base_utf8.encode("utf-8")
    if target_size == 0:
        return b""
    if not base:
        return b"~" * target_size
    repetitions, remainder = divmod(target_size, len(base))
    return base * repetitions + b"~" * remainder


def wire_payload(kind: str, raw: bytes) -> bytes:
    if kind != "json":
        return raw
    return json.dumps(
        raw.decode("utf-8"), ensure_ascii=False, separators=(",", ":")
    ).encode("utf-8")


def cartesian_cases() -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    strategy_ordinals = {strategy: 0 for strategy in STRATEGIES}
    for payload, (strategy, strategy_slug), persistence, size, parallel in product(
        PAYLOADS, STRATEGIES.items(), PERSISTENCE, SIZES, PARALLEL_CALLS
    ):
        raw = expand_payload(payload["baseUtf8"], size)
        wire = wire_payload(payload["kind"], raw)
        case_id = (
            f"CTX-{payload['id']}-{strategy_slug}-{persistence}-"
            f"{size:07d}-p{parallel}"
        )
        strategy_ordinal = strategy_ordinals[strategy]
        entry_point = ENTRY_POINTS[strategy_ordinal % len(ENTRY_POINTS)]
        strategy_ordinals[strategy] = strategy_ordinal + 1
        call_ids = [f"{case_id}-call-{index:02d}" for index in range(parallel)]
        records = [
            {
                "id": f"{case_id}-payload",
                "role": "user",
                "kind": payload["kind"],
                "protection": "protected",
                "content": {
                    "reconstruction": {
                        "generatorVersion": GENERATOR_VERSION,
                        "baseUtf8": payload["baseUtf8"],
                        "targetRawBytes": size,
                        "expansion": "whole-base-utf8-repetition-then-ascii-tilde-padding",
                        "wireEncoding": (
                            "canonical-json-string-utf8"
                            if payload["kind"] == "json"
                            else "raw-utf8"
                        ),
                    },
                    "rawBytes": len(raw),
                    "rawSha256": sha_bytes(raw),
                    "wireBytes": len(wire),
                    "wireSha256": sha_bytes(wire),
                },
            }
        ]
        for index, call_id in enumerate(call_ids):
            arguments_raw = canonical_bytes(
                {
                    "caseId": case_id,
                    "ordinal": index,
                    "payloadSha256": sha_bytes(raw),
                }
            ).decode("utf-8")
            result_raw = f"fixture-result:{case_id}:{index}:{sha_bytes(wire)}"
            records.extend(
                (
                    {
                        "id": call_id,
                        "role": "assistant",
                        "kind": "tool-call",
                        "protection": "protected",
                        "parallelGroupId": f"{case_id}-parallel-group",
                        "callId": call_id,
                        "toolName": "fixture.echo",
                        "argumentsRaw": arguments_raw,
                        "argumentsRawSha256": sha_bytes(arguments_raw.encode("utf-8")),
                    },
                    {
                        "id": f"{call_id}-result",
                        "role": "tool",
                        "kind": "tool-result",
                        "protection": "protected",
                        "parallelGroupId": f"{case_id}-parallel-group",
                        "callId": call_id,
                        "contentRaw": result_raw,
                        "contentRawSha256": sha_bytes(result_raw.encode("utf-8")),
                    },
                )
            )
        record_ids = [record["id"] for record in records]
        input_contract = {
            "generatorVersion": GENERATOR_VERSION,
            "payloadId": payload["id"],
            "payloadKind": payload["kind"],
            "baseUtf8": payload["baseUtf8"],
            "targetRawBytes": size,
            "expansion": "whole-base-utf8-repetition-then-ascii-tilde-padding",
            "wireEncoding": (
                "canonical-json-string-utf8" if payload["kind"] == "json" else "raw-utf8"
            ),
            "strategy": strategy,
            "persistence": persistence,
            "entryPoint": entry_point,
            "parallelCallIds": call_ids,
            "recordIdsInOrder": record_ids,
            "records": records,
        }
        cases.append(
            {
                "id": case_id,
                "input": input_contract,
                "inputSha256": sha_value(input_contract),
                "rawPayloadBytes": len(raw),
                "rawPayloadSha256": sha_bytes(raw),
                "wirePayloadBytes": len(wire),
                "wirePayloadSha256": sha_bytes(wire),
                "expected": {
                    "outcome": "protected_records_preserved",
                    "recordIdsInOrder": record_ids,
                    "rawPayloadSha256": sha_bytes(raw),
                    "protectedMutationCount": 0,
                    "protectedDropCount": 0,
                    "toolGroupsValid": True,
                    "compressibleKinds": ["host-marked-eligible-prose"],
                },
            }
        )
    return cases


def normal_entry_point_coverage(cases: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for strategy, entry_point in product(STRATEGIES, ENTRY_POINTS):
        case_ids = [
            case["id"]
            for case in cases
            if case["input"]["strategy"] == strategy
            and case["input"]["entryPoint"] == entry_point
        ]
        rows.append(
            {
                "strategy": strategy,
                "entryPoint": entry_point,
                "caseCount": len(case_ids),
                "caseIds": case_ids,
                "caseIdsSha256": sha_value(case_ids),
            }
        )
    return rows


def long_history() -> dict[str, Any]:
    records: list[dict[str, Any]] = []
    for index in range(1_000):
        if index == 10:
            records.append(
                {
                    "id": "history-decision-0010",
                    "role": "assistant",
                    "kind": "durable-decision",
                    "protection": "protected",
                    "text": "launch=2026-10-01",
                }
            )
        if index == 900:
            records.append(
                {
                    "id": "history-pending-0900",
                    "role": "assistant",
                    "kind": "pending-work",
                    "protection": "protected",
                    "text": "verify checksum ABC123",
                }
            )
        records.append(
            {
                "id": f"history-prose-{index:04d}",
                "role": "assistant",
                "kind": "prose",
                "protection": "eligible",
                "text": "Earlier discussion about travel.",
            }
        )
    records.extend(
        (
            {
                "id": "history-user-continue-1",
                "role": "user",
                "kind": "current-input",
                "protection": "protected",
                "text": "continue",
            },
            {
                "id": "history-user-continue-2",
                "role": "user",
                "kind": "current-input",
                "protection": "protected",
                "text": "continue",
            },
        )
    )
    return {
        "id": "LONG-HISTORY-v1",
        "indexSemantics": (
            "zero-based prose index; protected records are inserted immediately before "
            "prose records 10 and 900"
        ),
        "serialization": "canonical-json-utf8-sort-keys-compact-no-trailing-newline",
        "records": records,
        "recordsSha256": sha_value(records),
        "expected": {
            "protectedRecordIdsInOrder": [
                "history-decision-0010",
                "history-pending-0900",
                "history-user-continue-1",
                "history-user-continue-2",
            ],
            "eligibleRecordCount": 1_000,
            "protectedRecordCount": 4,
            "distinctContinueRecords": 2,
            "protectedMutationCount": 0,
            "protectedDropCount": 0,
        },
    }


def failure_fixtures() -> dict[str, dict[str, Any]]:
    protected = {
        "id": "protected-evidence-1",
        "kind": "evidence",
        "protection": "protected",
        "text": "launch=2026-10-01",
    }
    eligible = {
        "id": "eligible-prose-1",
        "kind": "prose",
        "protection": "eligible",
        "text": "Earlier discussion about travel.",
        "tokens": 80,
    }
    call = {
        "id": "assistant-call-record-1",
        "role": "assistant",
        "kind": "tool-call",
        "callId": "call-1",
        "argumentsRaw": "{\"path\":\"/fixture/report.txt\"}",
    }
    result = {
        "id": "tool-result-record-1",
        "role": "tool",
        "kind": "tool-result",
        "callId": "call-1",
        "payload": "checksum=ABC123",
    }
    summary_base = {
        "records": [eligible, protected],
        "budget": {
            "inputAllowance": 100,
            "protectedTokens": 40,
            "eligibleTokens": 80,
            "eligibleBudget": 60,
        },
    }
    return {
        "pending_without_result": {
            "input": {
                "records": [call],
                "hostExecutionState": "pending",
                "durableTerminalReceipt": None,
            },
            "expected": {
                "outcome": "pending_or_recovery_required",
                "dispatch": False,
                "fabricatedResultCount": 0,
                "preservedRecordIds": [call["id"]],
            },
        },
        "verified_cancelled": {
            "input": {
                "records": [call],
                "hostExecutionState": "cancelled",
                "durableTerminalReceipt": {
                    "callId": "call-1",
                    "status": "cancelled",
                    "receiptId": "terminal-receipt-1",
                },
            },
            "expected": {
                "outcome": "typed_cancelled_result_recorded",
                "dispatch": "after_history_validation",
                "preservedRecordIds": [call["id"]],
                "insertedResult": {
                    "callId": "call-1",
                    "status": "cancelled",
                    "provenanceReceiptId": "terminal-receipt-1",
                },
            },
        },
        "orphan": {
            "input": {"records": [result]},
            "expected": {
                "outcome": "invalid_history",
                "dispatch": False,
                "preservedRecordIds": [result["id"]],
            },
        },
        "duplicate_result": {
            "input": {
                "records": [
                    call,
                    result,
                    {**result, "id": "tool-result-record-2"},
                ]
            },
            "expected": {
                "outcome": "invalid_history",
                "dispatch": False,
                "preservedRecordIds": [
                    call["id"],
                    result["id"],
                    "tool-result-record-2",
                ],
            },
        },
        "summary_error": {
            "input": {**summary_base, "summarizer": {"outcome": "error"}},
            "expected": {
                "outcome": "summary_failed",
                "dispatch": False,
                "preservedRecordIds": [eligible["id"], protected["id"]],
            },
        },
        "summary_empty": {
            "input": {**summary_base, "summarizer": {"outcome": "success", "text": ""}},
            "expected": {
                "outcome": "summary_empty",
                "dispatch": False,
                "preservedRecordIds": [eligible["id"], protected["id"]],
            },
        },
        "summary_oversized": {
            "input": {
                **summary_base,
                "summarizer": {"outcome": "success", "text": "oversized", "tokens": 61},
            },
            "expected": {
                "outcome": "summary_oversized",
                "dispatch": False,
                "preservedRecordIds": [eligible["id"], protected["id"]],
            },
        },
        "summary_cancelled": {
            "input": {**summary_base, "summarizer": {"outcome": "cancelled"}},
            "expected": {
                "outcome": "summary_cancelled",
                "dispatch": False,
                "preservedRecordIds": [eligible["id"], protected["id"]],
            },
        },
        "unknown_media_count": {
            "input": {
                "records": [
                    {
                        "id": "media-record-1",
                        "kind": "media",
                        "protection": "protected",
                        "reference": "media:sha256:fixture-media-17",
                    }
                ],
                "count": {"quality": "approximate", "revision": "unknown-media-v1"},
            },
            "expected": {
                "outcome": "unsupported_count",
                "dispatch": False,
                "preservedRecordIds": ["media-record-1"],
            },
        },
        "revoked_before_dispatch": {
            "input": {
                "records": [protected],
                "preparedAuthorization": "allowed",
                "currentAuthorization": "revoked",
                "authorizationRevision": "auth-rev-2",
            },
            "expected": {
                "outcome": "blocked_authorization",
                "dispatch": False,
                "modelPayloadContainsRevokedContent": False,
                "summarizerPayloadContainsRevokedContent": False,
            },
        },
        "smaller_failover": {
            "input": {
                "canonicalSource": "immutable",
                "firstDestination": {"inputAllowance": 1_000},
                "failoverDestination": {
                    "C": 1_000,
                    "L": None,
                    "H": None,
                    "O": 100,
                    "R": 0,
                    "M": 0,
                },
                "F": 901,
            },
            "expected": {
                "outcome": "protected_overflow",
                "dispatch": False,
                "inputAllowance": 900,
                "requiredTokens": 901,
                "canonicalSourceReused": True,
                "protectedMutationCount": 0,
            },
        },
    }


def failure_cases() -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    for (failure_id, fixture), (strategy, strategy_slug), entry_point in product(
        failure_fixtures().items(), STRATEGIES.items(), ENTRY_POINTS
    ):
        case_id = f"FAIL-{failure_id}-{strategy_slug}-{entry_point}"
        case_input = {
            "strategy": strategy,
            "entryPoint": entry_point,
            **fixture["input"],
        }
        cases.append(
            {
                "id": case_id,
                "failure": failure_id,
                "input": case_input,
                "inputSha256": sha_value(case_input),
                "expected": fixture["expected"],
            }
        )
    return cases


BUDGET_FIXTURES = (
    {
        "id": "reserves-and-independent-limit",
        "input": {
            "C": 10_000,
            "L": 6_000,
            "H": 7_000,
            "O": 2_000,
            "R": 0,
            "M": 100,
            "F": 5_000,
            "eligibleTokens": 0,
            "countQuality": "exact",
            "countRevision": "fixture-v1",
        },
        "expected": {"outcome": "fit", "I": 6_000, "B": 1_000},
    },
    {
        "id": "reserve-sum-exceeds-context",
        "input": {
            "C": 1_000,
            "L": None,
            "H": None,
            "O": 1_100,
            "R": 0,
            "M": 0,
            "F": 0,
            "eligibleTokens": 0,
            "countQuality": "exact",
            "countRevision": "fixture-v1",
        },
        "expected": {
            "outcome": "invalid_limits",
            "field": "reserve_sum",
            "value": 1_100,
        },
    },
    {
        "id": "protected-overflow",
        "input": {
            "C": 1_000,
            "L": None,
            "H": None,
            "O": 100,
            "R": 0,
            "M": 0,
            "F": 901,
            "eligibleTokens": 0,
            "countQuality": "exact",
            "countRevision": "fixture-v1",
        },
        "expected": {
            "outcome": "protected_overflow",
            "requiredTokens": 901,
            "inputAllowance": 900,
        },
    },
)


def budget_cases() -> list[dict[str, Any]]:
    cases: list[dict[str, Any]] = []
    for fixture, (strategy, strategy_slug), entry_point in product(
        BUDGET_FIXTURES, STRATEGIES.items(), ENTRY_POINTS
    ):
        case_id = f"BUDGET-{fixture['id']}-{strategy_slug}-{entry_point}"
        case_input = {
            "strategy": strategy,
            "entryPoint": entry_point,
            **fixture["input"],
        }
        cases.append(
            {
                "id": case_id,
                "fixture": fixture["id"],
                "input": case_input,
                "inputSha256": sha_value(case_input),
                "expected": fixture["expected"],
            }
        )
    return cases


def build_manifest() -> dict[str, Any]:
    cartesian = cartesian_cases()
    normal_coverage = normal_entry_point_coverage(cartesian)
    history = long_history()
    failures = failure_cases()
    budgets = budget_cases()
    return {
        "schemaVersion": 1,
        "generatorVersion": GENERATOR_VERSION,
        "seed": SEED,
        "serialization": "utf8-json-sort-keys-indent-2-trailing-newline",
        "generatorContract": {
            "payloadExpansion": (
                "Target zero is empty. Otherwise repeat the complete UTF-8 base byte "
                "sequence, then fill the remainder with ASCII '~'; never split a code point. "
                "An empty base uses only '~'. No separator is inserted."
            ),
            "jsonWrapping": (
                "For payload kind json, wrap the expanded UTF-8 string with Python "
                "json.dumps ensure_ascii=False and separators comma/colon; all other kinds "
                "use the expanded bytes unchanged. Raw byte counts precede wrapping."
            ),
            "caseOrdering": (
                "payload declaration order, strategy declaration order, persistence order, "
                "ascending size, parallel-call order. Within each strategy assign normal "
                "cases to entry points round-robin in declared entry-point order; failures "
                "and budgets use fixture, strategy, entry-point declaration order"
            ),
            "entryPoints": list(ENTRY_POINTS),
            "strategies": list(STRATEGIES),
            "persistence": list(PERSISTENCE),
            "sizes": list(SIZES),
            "parallelCalls": list(PARALLEL_CALLS),
            "payloads": list(PAYLOADS),
        },
        "cartesianCases": cartesian,
        "normalEntryPointCoverage": normal_coverage,
        "longHistory": history,
        "failureCases": failures,
        "budgetCases": budgets,
        "counts": {
            "cartesianCases": len(cartesian),
            "normalEntryPointBindings": len(normal_coverage),
            "longHistoryRecords": len(history["records"]),
            "failureCases": len(failures),
            "budgetCases": len(budgets),
        },
        "aggregateSha256": {
            "cartesianCases": sha_value(cartesian),
            "normalEntryPointCoverage": sha_value(normal_coverage),
            "longHistory": sha_value(history),
            "failureCases": sha_value(failures),
            "budgetCases": sha_value(budgets),
        },
    }


def rendered_manifest() -> bytes:
    return (
        json.dumps(build_manifest(), sort_keys=True, indent=2, ensure_ascii=False) + "\n"
    ).encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--write", type=pathlib.Path)
    group.add_argument("--check", type=pathlib.Path)
    args = parser.parse_args()
    rendered = rendered_manifest()
    if args.write:
        args.write.write_bytes(rendered)
        print(f"WROTE {args.write} sha256={sha_bytes(rendered)}")
        return 0
    if args.check:
        actual = args.check.read_bytes()
        if actual != rendered:
            print(
                f"MISMATCH {args.check} expected={sha_bytes(rendered)} "
                f"actual={sha_bytes(actual)}",
                file=sys.stderr,
            )
            return 1
        manifest = json.loads(actual)
        print(
            f"PASS {args.check} sha256={sha_bytes(actual)} "
            f"cartesian={manifest['counts']['cartesianCases']} "
            f"failures={manifest['counts']['failureCases']} "
            f"budgets={manifest['counts']['budgetCases']}"
        )
        return 0
    sys.stdout.buffer.write(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
