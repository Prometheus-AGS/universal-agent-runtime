"""U01 genuine configured inference and U02 warm/cold session continuation.

Mounted contracts: src/server.rs ChatCompletionRequest/api_chat_completion;
src/uar/api/routes.rs RunInspection; src/uar/persistence/accounting.rs.
"""
from __future__ import annotations

import json
import os
import secrets
from urllib.parse import quote
import uuid

from core import Blocked, Case, Context, Outcome, Predicate, require


def configure_provider(ctx: Context, binding: dict | None = None) -> dict:
    binding = binding or ctx.fixture["provider"]
    config = dict(binding["config"])
    variable = binding.get("credential_env")
    if variable:
        credential = os.environ.get(variable)
        if not credential:
            raise Blocked(f"missing genuine provider credential reference: {variable}")
        config["api_key"] = credential
    path = "/api/uar/providers/" + quote(config["id"], safe="")
    existing = ctx.request("GET", path)
    require(existing.status in (200, 404), "provider lookup failed")
    if existing.status == 404:
        response = ctx.request("POST", "/api/uar/providers/", config)
        require(response.status == 201, "provider configuration was not accepted")
    saved = ctx.request("GET", "/api/uar/providers/" + quote(config["id"], safe=""))
    require(saved.status == 200, "configured provider could not be read")
    value = saved.json()
    require(value.get("id") == config["id"] and value.get("enabled") is True,
            "saved provider identity/enabled state differs")
    require(value.get("default_model") == config["default_model"], "saved provider model differs")
    require(value.get("base_url") == config["base_url"], "saved provider endpoint differs")
    require(not value.get("api_key"), "provider read leaked credential material")
    model_id = binding["qualified_model"].split("/", 1)[1]
    verified = ctx.request("POST", path + "/test", {"model": model_id})
    require(verified.status == 200, "real provider verification request failed")
    proof = verified.json()
    require(proof.get("ok") is True and proof.get("received_text") is True
            and proof.get("provider_id") == config["id"] and proof.get("model_id") == model_id,
            "provider verification did not observe the declared binding")
    return value


def prompt(ctx: Context, text: str, session_id: str | None = None,
           agent_id: str = "default-agent", model: str | None = None,
           send_model: bool = True, expected_model: str | None = None,
           run_policy: dict | None = None, allowed_models: list[str] | None = None) -> dict:
    """Require success, persisted effective binding, and real settled inference.

    This helper does not grade the answer. Callers must apply their own fixed
    useful-result predicate; a nonempty response is never sufficient.
    """
    selected = model or ctx.fixture["provider"]["qualified_model"]
    expected = expected_model or selected
    bindings = [ctx.fixture["provider"], *ctx.fixture.get("genuine_bindings", [])]
    declared = next((item for item in bindings if item["qualified_model"] == expected), None)
    if declared is None or declared.get("kind") != "genuine":
        raise Blocked("prompt requires a declared genuine provider/model binding")
    session_id = session_id or str(uuid.uuid4())
    payload = {"agent_id": agent_id, "session_id": session_id, "stream": False,
               "memory_enabled": False, "max_completion_tokens": ctx.fixture["limits"]["output_tokens"],
               "messages": [{"role": "user", "content": text}]}
    if send_model:
        payload["model"] = selected
    if run_policy is not None:
        payload["run_policy"] = run_policy
    response = ctx.request("POST", "/v1/chat/completions", payload)
    require(response.status == 200, "configured prompt did not return HTTP success")
    body = response.json()
    require(body.get("object") == "chat.completion", "wrong completion envelope")
    require(body.get("session_id") == session_id, "completion changed session identity")
    require(body.get("model") == expected, "completion reports the wrong model")
    choices = body.get("choices", [])
    require(len(choices) == 1 and choices[0].get("finish_reason") == "stop",
            "positive case lacks a successful completion terminal")
    message = choices[0].get("message", {})
    require(message.get("role") == "assistant" and isinstance(message.get("content"), str),
            "completion lacks assistant content")
    completion_id = body.get("id", "")
    require(completion_id.startswith("chatcmpl-"), "completion lacks native run correlation")
    run_id = completion_id.removeprefix("chatcmpl-")
    inspection_response = ctx.request("GET", "/api/uar/runs/" + quote(run_id, safe=""))
    require(inspection_response.status == 200, "native run inspection unavailable")
    inspection = inspection_response.json()
    provider_id, model_id = expected.split("/", 1)
    require(inspection.get("run_id") == run_id and inspection.get("conversation_id") == session_id,
            "native run/session identity mismatch")
    require(inspection.get("agent_id") == agent_id and inspection.get("status") == "done",
            "native run agent or successful terminal mismatch")
    require(inspection.get("effective_model") == {"provider_id": provider_id, "model_id": model_id},
            "actual effective model differs from requested binding")
    require(bool(inspection.get("agent_revision")), "effective agent revision missing")
    calls_response = ctx.request("GET", "/api/uar/accounting/calls/" + quote(run_id, safe=""))
    require(calls_response.status == 200, "durable provider accounting unavailable")
    calls = calls_response.json()
    require(isinstance(calls, list) and bool(calls), "no observed provider invocation")
    permitted_models = {expected, *(allowed_models or [])}
    model_calls: dict[str, list[str]] = {}
    for call in calls:
        admission = call.get("admission", {})
        identity = admission.get("identity", {})
        require(identity.get("run_id") == run_id and bool(identity.get("owner"))
                and bool(identity.get("call_id")), "provider call identity missing or foreign")
        actual_model = admission.get("model")
        require(actual_model in permitted_models and call.get("state") == "settled",
                "wrong or unsettled provider invocation")
        require(call.get("observed_tokens", 0) > 0 and bool(call.get("reconciliation_evidence")),
                "provider invocation lacks terminal usage evidence")
        model_calls.setdefault(actual_model, []).append(identity["call_id"])
    require(expected in model_calls, "no observed primary-model invocation")
    # Accounting alone cannot distinguish a controlled peer from genuine inference.
    # The frozen observer must read actual configured endpoint/provenance, bound
    # to this run; a fixture's declaration of kind=genuine is not proof.
    provenance_by_model = {}
    for actual_model, call_ids in model_calls.items():
        binding = next((item for item in bindings if item["qualified_model"] == actual_model), None)
        require(binding is not None and binding.get("kind") == "genuine",
                "observed auxiliary model has no frozen genuine binding")
        proof = ctx.observe("inference", {"run_id": run_id, "session_id": session_id,
            "qualified_model": actual_model, "primary_qualified_model": expected,
            "role": "primary" if actual_model == expected else "auxiliary", "call_ids": call_ids})
        require(proof.get("run_id") == run_id and proof.get("session_id") == session_id
                and proof.get("qualified_model") == actual_model,
                "inference provenance is not bound to this run")
        require(proof.get("kind") == "genuine" and proof.get("synthetic") is False
                and proof.get("base_url") == binding["config"]["base_url"]
                and bool(proof.get("configuration_digest"))
                and bool(proof.get("provider_receipt")), "genuine inference provenance missing")
        provenance_by_model[actual_model] = proof
    provenance = provenance_by_model[expected]
    return {"content": message["content"], "run_id": run_id, "session_id": session_id,
            "inspection": inspection, "accounting": calls, "provenance": provenance,
            "provenance_by_model": provenance_by_model}


def answer_json(result: dict) -> dict:
    try:
        answer = json.loads(result["content"])
    except (ValueError, TypeError) as error:
        require(False, "model did not return the specified JSON object")
        raise error
    require(isinstance(answer, dict), "model answer is not a JSON object")
    return answer


def configured_prompt(ctx: Context) -> Outcome:
    configure_provider(ctx)
    fact = secrets.token_hex(10)
    number = secrets.randbelow(900) + 100
    expected = {"fact": fact, "result": number + 17}
    result = prompt(ctx, f'Return only a JSON object with keys fact and result. '
                    f'The fact is "{fact}". The number is {number}. '
                    'Copy the fact exactly; result must be the number plus 17.')
    passed = answer_json(result) == expected
    ctx.record("useful-result", {"expected": expected, "actual": answer_json(result)})
    return Outcome(Predicate(True, "HTTP/native terminal and genuine provider binding observed"),
                   Predicate(passed, "fresh fact and independently computed transformation match"),
                   {"session_id": result["session_id"], "run_id": result["run_id"]})


def continuation(ctx: Context) -> Outcome:
    configure_provider(ctx)
    fact = secrets.token_hex(16)
    first = prompt(ctx, f'Remember the private parcel code {fact}. '
                   'Return only the JSON object {"remembered":true}.')
    require(answer_json(first) == {"remembered": True}, "first turn did not acknowledge the fact")
    session = first["session_id"]
    warm = prompt(ctx, 'What parcel code did I give you? Return only JSON {"code":"the code"}.', session)
    require(answer_json(warm) == {"code": fact}, "warm continuation forgot the fresh fact")
    before = ctx.observe("session", {"session_id": session, "phase": "before_restart"})
    require(before.get("session_id") == session and before.get("backend") == ctx.backend["name"]
            and before.get("durable") is True and bool(before.get("canonical_messages")),
            "durable pre-restart canonical session receipt missing")
    ctx.restart()
    known_runs = {first["run_id"], warm["run_id"]}
    catalog = ctx.request("GET", "/api/uar/runs")
    require(catalog.status == 200 and isinstance(catalog.json(), list)
            and known_runs.issubset({row.get("run_id") for row in catalog.json()}),
            "cold restart lost the owner's known completed run list")
    for known in known_runs:
        saved = ctx.request("GET", "/api/uar/runs/" + quote(known, safe=""))
        require(saved.status == 200 and saved.json().get("run_id") == known
                and saved.json().get("conversation_id") == session and saved.json().get("status") == "done",
                "cold GET and run catalog disagree about a genuine completed turn")
    cold = prompt(ctx, 'Recall the parcel code from our earlier conversation. '
                  'Return only JSON {"code":"the code"}.', session)
    after = ctx.observe("session", {"session_id": session, "phase": "after_restart"})
    require(after.get("session_id") == session and after.get("durable") is True,
            "durable post-restart session receipt missing")
    old = before["canonical_messages"]
    new = after.get("canonical_messages", [])
    require(len(new) > len(old) and new[:len(old)] == old,
            "cold continuation changed canonical turn order or lost prior turns")
    require(before.get("runtime_epoch") and after.get("runtime_epoch")
            and before["runtime_epoch"] != after["runtime_epoch"],
            "durability observation did not cross a real runtime epoch")
    return Outcome(Predicate(True, "warm and cold native protocol terminals succeeded"),
                   Predicate(answer_json(cold) == {"code": fact}, "cold answer retains private fact"),
                   {"session_id": session, "before": before, "after": after})


def cases() -> list[Case]:
    return [Case("U01.configured-prompt", "U01", configured_prompt),
            Case("U02.session-continuation", "U02", continuation, durable=True)]
