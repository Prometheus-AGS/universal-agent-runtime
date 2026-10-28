"""U11 genuine bounded initial skill reattachment, including warm/cold turns.

Authoring only under CLOSED. This case makes no claim about later tool rounds.
"""
from __future__ import annotations

import secrets
from urllib.parse import quote
import uuid

from cases_prompt import answer_json, configure_provider, prompt
from cases_strategies import select, setting
from core import Blocked, Case, Context, Outcome, Predicate, require


BUDGET = {"per_skill_tokens": 1024, "total_tokens": 1800}


def body(slot: str, code: str, head: str, tail: str) -> str:
    # The existing truncator preserves head and tail. Neither sentinel is an
    # expected answer; canonical history must not retain injected body copies.
    start = (f'Private parcel_{slot} fact: {code}. For the joint parcel task return only JSON '
             '{"parcel_a":"private fact from skill a","parcel_b":"private fact from skill b",'
             '"result":user quantity plus 19}. Never output the body sentinels.\n' + head + '\n')
    return start + ("Irrelevant bounded-retention pressure material.\n" * 2400) + tail + '\n'


def private_body(ctx: Context, slot: str, code: str, head: str, tail: str) -> str:
    content = body(slot, code, head, tail)
    # The mounted write API echoes prompt_overlay. Register its exact value
    # before either request or response evidence is recorded.
    ctx.evidence.secrets.append(content)
    ctx.evidence.secrets.sort(key=len, reverse=True)
    return content


def bounded(ctx: Context) -> Outcome:
    if not ctx.backend.get("observers", {}).get("skill_retention"):
        raise Blocked("U11 bounded skill reattachment requires the concrete durable observer")
    configure_provider(ctx)
    join = "parcel" + secrets.token_hex(8)
    ids = ["acceptance-retention-" + slot + "-" + secrets.token_hex(8) for slot in ("a", "b")]
    sentinels = {slot: ["body-head-" + secrets.token_hex(16), "body-tail-" + secrets.token_hex(16)] for slot in ("a", "b")}
    created = []
    restored = []
    config_path = "/api/uar/skills/config"
    before = ctx.request("GET", config_path)
    require(before.status == 200, "mounted matching configuration missing")
    matching = before.json()
    receipts = []
    try:
        for slot, skill_id in zip(("a", "b"), ids):
            response = ctx.request("POST", "/api/uar/skills/", {
                "name": skill_id, "description": f'Joint {join} parcel task skill {slot}.',
                "triggers": {"keywords": [join], "semantic": f'Joint {join} parcels'},
                "prompt_overlay": private_body(ctx, slot, secrets.token_hex(12), *sentinels[slot]), "enabled": True})
            require(response.status == 201 and response.json().get("skill_id") == skill_id,
                    "owned long-body skill creation failed")
            created.append(skill_id)
        for key, value in (("skills.matching.engine", "service"),
                           ("harness.skill_activation_mode", "legacy_overlay"),
                           ("harness.skill_reattachment", BUDGET)):
            restored.append((key, setting(ctx, key, value)))
        require(ctx.request("PUT", config_path, {**matching, "engine": "service", "algorithm": "keyword",
                "threshold": 0.1, "margin_threshold": 0.0, "top_k": 2, "model_name": None}).status == 200,
                "explicit bounded-case matching selection failed")
        old_mode = setting(ctx, "harness.mode", "typed")
        restored.append(("harness.mode", old_mode))
        for mode in ("typed", "legacy"):
            setting(ctx, "harness.mode", mode)
            session = str(uuid.uuid4())
            previous = None
            for stage in ("initial", "warm", "cold"):
                # New private facts are never in user input or previous answers.
                # A follow-up cannot pass merely by repeating an earlier answer.
                codes = {slot: secrets.token_hex(12) for slot in ("a", "b")}
                for slot, skill_id in zip(("a", "b"), ids):
                    updated = ctx.request("PATCH", "/api/uar/skills/" + quote(skill_id, safe=""),
                                          {"prompt_overlay": private_body(ctx, slot, codes[slot], *sentinels[slot])})
                    require(updated.status == 200, "durable current skill-body revision failed")
                if stage == "cold":
                    ctx.restart()
                quantity = secrets.randbelow(800) + 100
                result = prompt(ctx, f'Perform the joint {join} parcel task for quantity {quantity}. '
                    'Use both currently selected skills and their current private facts. Apply their configured JSON instructions.',
                    session_id=session, run_policy=select(ids))
                require(answer_json(result) == {"parcel_a": codes["a"], "parcel_b": codes["b"], "result": quantity + 19},
                        "bounded current skill heads did not determine the useful genuine answer")
                observed = ctx.observe("skill_retention", {"run_id": result["run_id"], "session_id": session,
                    "skill_ids": ids, "sentinels": [marker for pair in sentinels.values() for marker in pair]})
                receipt = observed.get("receipt", {})
                require(observed.get("run_id") == result["run_id"] and observed.get("session_id") == session
                        and observed.get("history_body_free") is True and observed.get("original_hashes_match") is True,
                        "actual durable retention receipt or body-free canonical history evidence missing")
                require(receipt.get("scope") == "initial_assembly" and receipt.get("count_quality") == "approximate"
                        and receipt.get("per_skill_tokens") == BUDGET["per_skill_tokens"]
                        and receipt.get("total_tokens") == BUDGET["total_tokens"]
                        and receipt.get("rendered_tokens", 0) > 0
                        and receipt["rendered_tokens"] <= BUDGET["total_tokens"]
                        and receipt.get("request_tokens", 0) <= receipt.get("request_input_limit", -1),
                        "actual initial helper estimates do not obey their configured total/context bounds")
                fragments = receipt.get("fragments", [])
                require({row.get("skill_id") for row in fragments} == set(ids)
                        and all(row.get("retained") is True and row.get("truncated") is True
                            and row.get("original_rendered_tokens", 0) > BUDGET["per_skill_tokens"]
                            and 0 < row.get("retained_rendered_tokens", 0) <= BUDGET["per_skill_tokens"]
                            and row.get("original_body_sha256") != row.get("retained_body_sha256")
                            and row.get("original_rendered_sha256") != row.get("retained_rendered_sha256")
                            for row in fragments), "pressure did not cause actual bounded truncation of both active skills")
                require(any(row["retained_rendered_tokens"] < BUDGET["per_skill_tokens"] - 32 for row in fragments),
                        "joint total budget did not constrain either rendered attachment")
                if previous:
                    old = {row["skill_id"]: row for row in previous["fragments"]}
                    require(all(row["original_body_sha256"] != old[row["skill_id"]]["original_body_sha256"]
                                and row["retained_body_sha256"] != old[row["skill_id"]]["retained_body_sha256"]
                                for row in fragments), "warm/cold run reattached stale private skill bodies")
                previous = receipt
                receipts.append({"mode": mode, "stage": stage, "run_id": result["run_id"], "receipt": observed})
        ctx.record("bounded-skill-initial-reattachment", receipts)
        return Outcome(Predicate(True, "actual initial rendered-fragment budget/hash evidence on first/warm/cold runs in typed and legacy modes"),
                       Predicate(True, "fresh private head facts survive real bounded attachments; injected bodies stay out of canonical history"),
                       {"scope": "initial_assembly_each_run", "runs": receipts, "later_tool_rounds": "not assessed"})
    finally:
        require(ctx.request("PUT", config_path, matching).status == 200, "owned matching restoration failed")
        for key, value in reversed(restored):
            setting(ctx, key, value)
        for skill_id in created:
            require(ctx.request("DELETE", "/api/uar/skills/" + quote(skill_id, safe="")).status == 204,
                    "owned long skill cleanup failed")


def cases() -> list[Case]:
    return [Case("U11.bounded-skill-reattachment", "U11", bounded, durable=True)]
