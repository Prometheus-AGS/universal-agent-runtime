"""Mounted U09–13 strategy outcomes; execution requires the approved packet.

Every result uses an owned runtime, genuine model calls and actual durable run
receipts. Missing resources/evidence block the case instead of implying support.
"""
from __future__ import annotations

import json
import secrets
from urllib.parse import quote
import uuid

from cases_prompt import answer_json, configure_provider, prompt
from core import Blocked, Case, Context, Outcome, Predicate, require


SERVICE = ("keyword", "embedding", "local_embedding", "hybrid", "llm")
CLASSIFIER = ("rules", "tfidf", "hybrid", "wasm", "localembedding", "llm")


def setting(ctx, key, value):
    path = "/api/uar/settings/" + quote(key, safe="")
    before = ctx.request("GET", path)
    require(before.status == 200, "strategy setting is not mounted: " + key)
    row = before.json()
    saved = ctx.request("PUT", path, {"value": value, "expected_revision": row["revision"]})
    require(saved.status == 200, "strategy setting was not committed: " + key)
    readback = ctx.request("GET", path)
    require(readback.status == 200 and readback.json()["saved"] == value,
            "strategy setting readback differs: " + key)
    return row["saved"]


def skill(ctx, word, value):
    name = "acceptance-" + secrets.token_hex(10)
    body = {"name": name, "description": "Answer " + word + " parcel queries using the private parcel code.",
            "triggers": {"keywords": [word], "semantic": "Answer " + word + " parcel queries"},
            "prompt_overlay": 'For ' + word + ' parcel queries return only JSON {"parcel":"' + value + '"}.',
            "enabled": True}
    response = ctx.request("POST", "/api/uar/skills/", body)
    require(response.status == 201, "owned skill creation failed")
    created = response.json()
    require(created.get("skill_id") == name, "created skill identity differs")
    return name


def select(ids, mode="selected"):
    return {"version": 1, "skills": {"mode": mode, "ids": ids, "denied_ids": []}}


def observed(ctx, result):
    receipt = ctx.observe("strategy_run", {"run_id": result["run_id"]})
    require(receipt.get("run_id") == result["run_id"]
            and receipt.get("session_id") == result["session_id"]
            and receipt.get("status") == "done" and bool(receipt.get("store_identity")),
            "strategy receipt is not bound to the actual completed durable run")
    return receipt


def discovery(ctx: Context) -> Outcome:
    configure_provider(ctx)
    seeded = ctx.observe("strategy_seed_discovery", {})
    require(len(seeded.get("expected_ids", [])) == 3 and len(seeded.get("files", [])) == 3,
            "exact owned standard/nested/symlink discovery fixtures missing")
    ctx.restart()
    found = ctx.observe("strategy_discovery", seeded)
    require({row["skill_id"] for row in found.get("skills", [])} == set(seeded["expected_ids"]),
            "startup discovery differs from owned manifests")
    # Durable scoped enablement must survive cold restart and remain owner-local.
    name = seeded["expected_ids"][0]
    session = str(uuid.uuid4())
    path = "/api/uar/skills/" + quote(name, safe="") + "/toggle"
    change = ctx.request("POST", path, {"enabled": False,
                         "scope": {"kind": "conversation", "id": session}})
    require(change.status == 200, "conversation skill disable failed")
    ctx.restart()
    row = ctx.request("GET", "/api/uar/skills/" + quote(name, safe=""))
    scope = ctx.observe("strategy_skill_scope", {"skill_id": name, "session_id": session})
    require(row.status == 200 and scope.get("skill_id") == name and scope.get("session_id") == session
            and scope.get("enabled") is False and bool(scope.get("store_identity")),
            "scoped skill disable did not survive cold restart")
    title = quote(row.json()["title"], safe="")
    disabled = ctx.request("GET", "/api/uar/skills/match?q=" + title + "&conversation_id=" + session)
    fresh = ctx.request("GET", "/api/uar/skills/match?q=" + title + "&conversation_id=" + str(uuid.uuid4()))
    require(disabled.status == fresh.status == 200
            and not any(item.get("skill_id") == name for item in disabled.json())
            and any(item.get("skill_id") == name for item in fresh.json()),
            "disabled conversation leaked into selection or another conversation lost the global skill")
    return Outcome(Predicate(True, "actual startup registry and durable scope readback observed"),
                   Predicate(True, "top-level nested and one-hop linked skills discovered once; nonmanifest excluded"),
                   {"skills": seeded["expected_ids"], "session_id": session})


def matching(ctx: Context) -> Outcome:
    configure_provider(ctx)
    word, code, decoy = "parcel" + secrets.token_hex(8), secrets.token_hex(12), secrets.token_hex(12)
    intended = skill(ctx, word, code)
    other = skill(ctx, "unrelated-weather-" + secrets.token_hex(8), decoy)
    settings = []
    config_path = "/api/uar/skills/config"
    original = ctx.request("GET", config_path)
    require(original.status == 200, "skill matching config unavailable")
    original = original.json()
    receipts = []
    try:
        for engine, methods in (("service", SERVICE), ("intent_classifier", CLASSIFIER)):
            old = setting(ctx, "skills.matching.engine", engine)
            settings.append(("skills.matching.engine", old))
            for method in methods:
                config = {**original, "engine": engine, "algorithm": method if engine == "service" else "keyword",
                          "threshold": 0.1, "margin_threshold": 0.0, "top_k": 2,
                          "model_name": ctx.fixture["provider"]["qualified_model"] if method == "llm" else None}
                require(ctx.request("PUT", config_path, config).status == 200, "matching configuration rejected")
                if engine == "intent_classifier":
                    old_backend = setting(ctx, "intent_classifier.backend", method)
                    settings.append(("intent_classifier.backend", old_backend))
                    if method == "wasm":
                        component = ctx.fixture.get("strategies", {}).get("wasm_component_path")
                        if not component or component not in ctx.fixture.get("_frozen_paths", []):
                            raise Blocked("WASM positive outcome requires the frozen actual classifier component")
                        settings.append(("intent_classifier.wasm_component_path",
                                         setting(ctx, "intent_classifier.wasm_component_path", component)))
                # Auto selection must rank and consume the intended body; selected
                # would bypass precisely the matching effect under observation.
                result = prompt(ctx, "Answer the " + word + " parcel query using its skill instructions. Return only JSON.",
                                run_policy=select([intended, other], "auto"))
                receipt = observed(ctx, result)
                match = receipt.get("skill_matching", {})
                prefix = "skill_service." if engine == "service" else "legacy_classifier."
                require(match.get("requested_method") == prefix + method
                        and match.get("effective_method") == prefix + method,
                        "requested matching backend did not actually run")
                candidates = match.get("candidates", [])
                require(any(row.get("skill_id") == intended and row.get("accepted") is True
                            and isinstance(row.get("score"), (int, float)) for row in candidates),
                        "intended skill lacks scored accepted candidate evidence")
                require(answer_json(result) == {"parcel": code} and decoy not in result["content"],
                        "selected matching result did not produce the intended private skill outcome")
                receipts.append({"run_id": result["run_id"], "method": prefix + method})
    finally:
        require(ctx.request("PUT", config_path, original).status == 200, "matching config restoration failed")
        for key, old in reversed(settings):
            setting(ctx, key, old)
    return Outcome(Predicate(True, "all eleven explicit service/classifier methods observed without substitution"),
                   Predicate(True, "each accepted intended candidate produced its private body result"), {"runs": receipts})


def activation(ctx: Context) -> Outcome:
    configure_provider(ctx)
    word, code = "parcel" + secrets.token_hex(8), secrets.token_hex(12)
    intended = skill(ctx, word, code)
    confounder = skill(ctx, "weather" + secrets.token_hex(8), secrets.token_hex(12))
    rows = []
    for presentation in ("legacy_overlay", "catalog"):
        old = setting(ctx, "harness.skill_activation_mode", presentation)
        try:
            for mode in ("inherit", "auto", "all", "none", "selected"):
                policy = select([intended] if mode == "selected" else [], mode)
                if mode != "selected":
                    policy["skills"]["denied_ids"] = [confounder]
                question = "Answer the " + word + " parcel query using the eligible skill. "
                question += 'Activate the skill if needed. Return only JSON {"parcel":"code"}.' if mode != "none" else 'Return only JSON {"parcel":"unavailable"}.'
                result = prompt(ctx, question, run_policy=policy)
                proof = observed(ctx, result)
                activated = proof.get("activated_skills")
                require(isinstance(activated, list), "actual activation receipt missing")
                require(confounder not in activated, "denied confounder activated")
                if mode == "none":
                    require(not activated and answer_json(result) == {"parcel": "unavailable"},
                            "none policy activated a skill or leaked its fact")
                else:
                    require(intended in activated and answer_json(result) == {"parcel": code},
                            "eligible skill body not actually activated and consumed")
                rows.append({"run_id": result["run_id"], "mode": mode, "presentation": presentation})
        finally:
            setting(ctx, "harness.skill_activation_mode", old)
    return Outcome(Predicate(True, "actual activation and denied identities observed under five policies and two presentations"),
                   Predicate(True, "private skill fact consumed only with effective activation"), {"runs": rows})


def context_strategies(ctx: Context) -> Outcome:
    configure_provider(ctx)
    strategies = [{"type": "none"}, {"type": "sliding_window", "max_messages": 4},
                  {"type": "truncate_middle", "keep_first": 2, "keep_last": 4},
                  {"type": "summarize", "threshold": 4, "summary_max_tokens": 512, "model": None},
                  {"type": "hierarchical", "short_term_turns": 2,
                   "mid_term_summary_tokens": 512, "long_term_facts_tokens": 256}, {"type": "auto"}]
    runs = []
    for strategy in strategies:
        fact, recent = secrets.token_hex(10), secrets.token_hex(10)
        policy = {**select([], "none"), "context_strategy": strategy}
        first = prompt(ctx, 'Remember ancient parcel code ' + fact + '. Return only JSON {"stored":true}.', run_policy=policy)
        require(answer_json(first) == {"stored": True}, "old fact seed was not acknowledged")
        session = first["session_id"]
        for index in range(6):
            turn = prompt(ctx, 'Record neutral log entry ' + str(index) + '. Return only JSON {"logged":true}.',
                          session, run_policy=policy)
            require(answer_json(turn) == {"logged": True}, "history seed turn failed")
        result = prompt(ctx, 'Recent code is ' + recent + '. Return only JSON {"recent":"the recent code"}.',
                        session, run_policy=policy)
        require(answer_json(result) == {"recent": recent}, "recent fact lost through context preparation")
        receipt = observed(ctx, result).get("context_reduction", {})
        require(receipt.get("schema_version") == 1 and bool(receipt.get("source_sha256"))
                and bool(receipt.get("final_sha256")) and receipt.get("source_messages", 0) >= 12
                and receipt.get("protected_groups_preserved") is True,
                "actual structural/token preparation receipt missing")
        require(receipt.get("final_tokens", -1) >= 0
                and receipt["final_tokens"] <= receipt.get("input_allowance", -1),
                "final context exceeds the observed input allowance")
        kind = strategy["type"]
        if kind in ("sliding_window", "truncate_middle", "summarize", "hierarchical"):
            require(receipt.get("changed") is True and receipt["prepared_sha256"] != receipt["source_sha256"],
                    "configured strategy did not change over-threshold history")
        if kind in ("summarize", "hierarchical"):
            tiers = receipt.get("tier_calls", [])
            required = {"summary"} if kind == "summarize" else {"long_term_facts", "mid_term_summary"}
            require(receipt.get("summary_generated") is True
                    and required.issubset({call.get("tier") for call in tiers if call.get("completed") is True}),
                    "actual successful summary tiers missing")
            require(all(call.get("input_sha256") and call.get("output_sha256") for call in tiers),
                    "summary tier input/output receipts missing")
            recalled = prompt(ctx, 'Return only JSON {"ancient":"the ancient parcel code"}.', session, run_policy=policy)
            require(answer_json(recalled) == {"ancient": fact}, "governed summaries did not retain the old fact")
        canonical = ctx.observe("strategy_session", {"session_id": session, "markers": [fact, recent]})
        require(all(canonical.get("marker_present", {}).get(marker) is True for marker in (fact, recent)),
                "context preparation destructively rewrote canonical history")
        runs.append({"run_id": result["run_id"], "strategy": kind})
    return Outcome(Predicate(True, "six actual structural strategies observed with bounded final context"),
                   Predicate(True, "recent facts remain useful; real summary tiers retain old facts and canonical history"), {"runs": runs})


def whole_budget(ctx: Context) -> Outcome:
    configure_provider(ctx)
    fact = secrets.token_hex(10)
    result = prompt(ctx, 'Return only JSON {"fact":"' + fact + '"}.',
                    run_policy={**select([], "none"), "context_strategy": {"type": "none"}})
    require(answer_json(result) == {"fact": fact}, "below-limit request not useful")
    receipt = observed(ctx, result)
    budget = receipt.get("request_budget")
    require(isinstance(budget, dict) and budget.get("count_quality") in ("approximate", "exact", "validated_upper_bound"),
            "ordinary request lacks its actual budget counting quality")
    require(budget.get("representation") == "final_provider_request"
            and budget.get("serialized_request") is True
            and budget.get("serialized_bytes", 0) > 0 and bool(budget.get("serialized_sha256")),
            "ordinary receipt was not counted from the actual serialized provider request")
    require(budget.get("counted_input_tokens", -1) >= 0
            and budget["counted_input_tokens"] <= budget.get("input_allowance", -1)
            and budget.get("reserved_output_tokens", 0) > 0
            and budget.get("tools_included") is True and budget.get("instructions_included") is True,
            "whole-request counting evidence missing")
    vectors = ctx.fixture.get("strategies", {}).get("budget_vectors", [])
    overflow = [row for row in vectors if row.get("kind") == "over_limit"]
    if len(overflow) != 1:
        raise Blocked("U13 needs one frozen protected-overflow production request")
    # A genuine exact-boundary journey is required only for a destination which
    # actually advertises a proven exact contract. Approximate ordinary requests
    # never borrow the supplementary synthetic COUNT proof.
    boundary = [row for row in vectors if row.get("kind") == "exact_boundary"]
    if budget["count_quality"] == "exact" and len(boundary) != 1:
        raise Blocked("advertised exact destination needs a frozen exact-boundary production request")
    selected = overflow + (boundary if budget["count_quality"] == "exact" else [])
    for vector in selected:
        payload = dict(vector["request"])
        # Only the correlation identifier is dynamic; preserve the frozen body.
        payload["session_id"] = str(uuid.uuid4())
        require(payload.get("model") == ctx.fixture["provider"]["qualified_model"]
                and payload.get("stream") is False, "budget vector destination or transport differs")
        response = ctx.request("POST", "/v1/chat/completions", payload)
        proof = ctx.observe("strategy_budget", {"kind": vector["kind"], "response": response.json(),
                            "status": response.status, "session_id": payload.get("session_id"),
                            "expected_json": vector.get("expected_json")})
        require(proof.get("kind") == vector["kind"]
                and proof.get("count_quality") in ("approximate", "exact", "validated_upper_bound"),
                "budget vector lacks the actual boundary counting quality")
        if vector["kind"] == "exact_boundary":
            require(proof["count_quality"] == "exact" and response.status == 200
                    and proof.get("counted_input_tokens") == proof.get("input_allowance")
                    and proof.get("useful_result") is True and proof.get("settled_calls", 0) > 0,
                    "exact-boundary useful admission not established")
        else:
            # The mounted adapter currently maps runtime errors to HTTP 500.
            # Only correlated durable preparation refusal proves this case.
            require(response.status in (400, 413, 422, 500) and proof.get("refused_before_dispatch") is True
                    and proof.get("provider_calls") == 0 and proof.get("host_effects") == 0,
                    "over-limit request dispatched or had unproven refusal")
    return Outcome(Predicate(True, "ordinary whole-request allowance and pre-dispatch overflow refusal observed"),
                   Predicate(True, "bounded request useful; protected overflow has zero dispatch/effect"),
                   {"run_id": result["run_id"], "count_quality": budget["count_quality"],
                    "genuine_exact_boundary": "observed" if boundary and budget["count_quality"] == "exact" else "unsupported",
                    "supplementary_exact_COUNT": "separate retained obligation"})


def cases() -> list[Case]:
    return [Case("U09.skill-discovery-scope", "U09", discovery, durable=True),
            Case("U10.skill-matching-strategies", "U10", matching),
            Case("U11.skill-activation-governance", "U11", activation),
            Case("U12.context-strategy-effects", "U12", context_strategies, durable=True),
            Case("U13.whole-request-budget", "U13", whole_budget)]
