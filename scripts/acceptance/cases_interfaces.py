"""U05 mounted interface and actual shipped SDK positive parity.

Each expanded case has three repetitions on both durable backends. The browser
and A2UI families have separate owners; no rendered-UI claim is made here.
"""
from __future__ import annotations

import copy
import json
import secrets
from urllib.parse import quote
import uuid

from core import Blocked, Case, Context, Outcome, Predicate, require
from cases_prompt import configure_provider
from protocol_clients import MountedClient, run_program, verify_run


HTTP_PROFILES = (
    "chat:json", "chat:openai", "chat:agui", "chat:dual", "chat:agui_spec",
    "openai:json", "openai:openai", "openai:agui", "openai:dual", "openai:agui_spec",
    "native:agui", "native:agui_spec", "anthropic:json", "anthropic:stream",
    "a2a_legacy:json", "a2a_v1:json", "a2a_v1:stream", "acp:json", "acp:stream",
)
PROGRAM_PROFILES = ("python_sdk", "typescript_sdk", "rust_sdk", "grpc_legacy", "grpc_v1")


def prepare_agent(ctx: Context, custom: bool) -> tuple[str, str | None]:
    if not custom:
        return "default-agent", None
    response = ctx.request("GET", "/api/agents/default-agent")
    require(response.status == 200, "default artifact unavailable for custom fixture")
    artifact = copy.deepcopy(response.json())
    identifier = "acceptance-interface-" + str(uuid.uuid4())
    marker = secrets.token_hex(12)
    artifact.update(id=identifier, version="1.0.0", extensions={})
    artifact["metadata"].update(title=identifier, description="Owned interface parity fixture")
    artifact["runtime"]["entry"] = "default"
    artifact["prompt"] = {"system": 'Answer using only a JSON object with keys code and marker. '
        f'The private marker is "{marker}". Include it exactly in marker. '
        'Remember the parcel code given by the user and return that code in code. '
        'On follow-up recover the earlier parcel code from the conversation.', "instructions": []}
    provider, model = ctx.fixture["provider"]["qualified_model"].split("/", 1)
    artifact["policy"]["provider"]["default"] = {"provider": provider, "model": model}
    artifact["policy"]["tools"]["allow"] = []
    artifact["memory"]["kb"]["enabled"] = False
    created = ctx.request("POST", "/api/agents", artifact)
    require(created.status == 201 and created.json().get("id") == identifier, "custom parity artifact create failed")
    return identifier, marker


def tasks(marker: str | None) -> tuple[list[str], list[dict]]:
    code = secrets.token_hex(16)
    if marker:
        first = f'The private parcel code is {code}. Remember it and apply your configured output instructions.'
        second = 'Recall the parcel code from the previous turn and apply your configured output instructions.'
        expected = {"code": code, "marker": marker}
    else:
        first = f'Remember the private parcel code {code}. Return only JSON {{"code":"the exact parcel code"}}.'
        second = 'What parcel code did I give you? Return only JSON {"code":"the exact earlier parcel code"}.'
        expected = {"code": code}
    return [first, second], [expected, expected]


def useful(content: str, expected: dict) -> None:
    try:
        answer = json.loads(content)
    except (ValueError, TypeError) as error:
        raise AssertionError("interface answer did not satisfy strict JSON task format") from error
    require(answer == expected, "interface lost the fresh fact, prior context, or saved custom prompt effect")


def http_case(ctx: Context, profile: str, custom: bool) -> Outcome:
    configure_provider(ctx)
    agent, marker = prepare_agent(ctx, custom)
    inputs, expected = tasks(marker)
    client = MountedClient(ctx, profile, agent, ctx.fixture["provider"]["qualified_model"])
    turns = []
    for text, answer in zip(inputs, expected):
        result = client.turn(text)
        useful(result["content"], answer)
        turns.append(result)
    if custom:
        deleted = ctx.request("DELETE", "/api/agents/" + quote(agent, safe=""))
        require(deleted.status == 204, "owned custom interface artifact cleanup failed")
    return Outcome(Predicate(True, "mounted protocol terminals and actual configured dispatch correlated"),
                   Predicate(True, "fresh fact, follow-up context and selected saved prompt agree"),
                   {"profile": profile, "agent_id": agent, "turns": turns})


def program_case(ctx: Context, name: str, mode: str, custom: bool) -> Outcome:
    configure_provider(ctx)
    agent, marker = prepare_agent(ctx, custom)
    inputs, expected = tasks(marker)
    authorization_env = ctx.backend.get("headers_env", {}).get("Authorization")
    if not authorization_env:
        raise Blocked("installed client requires an explicit bearer credential reference")
    receipt = run_program(ctx, name, {"base_url": ctx.backend["base_url"],
        "grpc_endpoint": ctx.backend.get("grpc_endpoint"), "authorization_env": authorization_env,
        "agent_id": agent, "model": ctx.fixture["provider"]["qualified_model"],
        "session_id": str(uuid.uuid4()), "mode": mode, "turns": inputs, "expected": expected,
        "timeout_seconds": ctx.fixture["limits"]["request_seconds"],
        "max_bytes": ctx.fixture["limits"]["response_bytes"]})
    require(receipt.get("client") == name and receipt.get("mode") == mode, "wrong installed client/profile receipt")
    turns = receipt.get("turns", [])
    require(len(turns) == 2, "installed client omitted a conversation turn")
    seen = set()
    for turn, answer in zip(turns, expected):
        require(turn.get("protocol_terminal") == "success", "installed client lacks successful terminal")
        useful(turn.get("content"), answer)
        run_id = turn.get("run_id")
        require(run_id and run_id not in seen, "installed client omitted or duplicated native run identity")
        seen.add(run_id)
        verify_run(ctx, run_id, turn.get("session_id"), agent, ctx.fixture["provider"]["qualified_model"])
    require((turns[0].get("session_id") and turns[0].get("session_id") == turns[1].get("session_id"))
            or (turns[0].get("context_id") and turns[0].get("context_id") == turns[1].get("context_id")),
            "installed client follow-up changed context")
    if custom:
        require(ctx.request("DELETE", "/api/agents/" + quote(agent, safe="")).status == 204,
                "owned custom client artifact cleanup failed")
    return Outcome(Predicate(True, "actual shipped client and real provider receipts correlated"),
                   Predicate(True, "two useful input-dependent rounds with selected prompt effect"),
                   {"client": name, "mode": mode, "agent_id": agent, "receipt": receipt})


def cases() -> list[Case]:
    result = []
    for profile in HTTP_PROFILES:
        for custom in (False, True):
            label = "custom" if custom else "default"
            identifier = "U05." + profile.replace(":", ".") + "." + label
            result.append(Case(identifier, "U05", lambda ctx, p=profile, c=custom: http_case(ctx, p, c)))
    for profile in ("anthropic:json:implicit", "anthropic:stream:implicit"):
        result.append(Case("U05." + profile.replace(":", "."), "U05",
                           lambda ctx, p=profile: http_case(ctx, p, False)))
    for name in PROGRAM_PROFILES:
        for mode in ("json", "stream"):
            for custom in (False, True):
                label = "custom" if custom else "default"
                result.append(Case(f"U05.{name}.{mode}.{label}", "U05",
                    lambda ctx, n=name, m=mode, c=custom: program_case(ctx, n, m, c)))
    return result
