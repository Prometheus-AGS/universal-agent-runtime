"""Durable agent/configuration outcomes through mounted management and chat APIs.

Source: agent_store.rs protection; discovery.rs persisted reads/session config;
defaults.rs insert-only seeds; server.rs settings hydration; manager.rs retention.
Execution is exclusively part of the separately approved whole-phase packet.
"""
from __future__ import annotations

import copy
import secrets
from urllib.parse import quote
import uuid

from core import Blocked, Case, Context, Outcome, Predicate, require
from cases_prompt import answer_json, configure_provider, prompt


PROTECTED = ("default-agent", "orchestrator-agent")


def agent_path(agent_id: str) -> str:
    return "/api/agents/" + quote(agent_id, safe="")


def read_agent(ctx: Context, agent_id: str) -> dict:
    response = ctx.request("GET", agent_path(agent_id))
    require(response.status == 200, "persisted agent read failed")
    value = response.json()
    require(value.get("id") == agent_id, "persisted agent identity differs")
    return value


def custom_artifact(ctx: Context, agent_id: str, marker: str) -> dict:
    artifact = copy.deepcopy(read_agent(ctx, "default-agent"))
    artifact.update(id=agent_id, version="1.0.0", extensions={})
    artifact["metadata"].update(title="Acceptance " + agent_id,
                                description="Run-owned durable prompt effect", tags=["acceptance"])
    artifact["runtime"]["entry"] = "default"
    artifact["prompt"] = {"system": 'Return only JSON with keys marker and result. '
                          f'The private marker is "{marker}". '
                          'Copy it into marker. Set result to the user number plus 19.',
                          "instructions": []}
    artifact["memory"]["kb"]["enabled"] = False
    artifact["policy"]["tools"]["allow"] = []
    return artifact


def arithmetic(ctx: Context, agent_id: str = "default-agent", **selection) -> dict:
    number = secrets.randbelow(800) + 100
    result = prompt(ctx, f'Return only JSON {{"result":number plus 19}}. '
                    f'The number is {number}.', agent_id=agent_id, **selection)
    require(answer_json(result) == {"result": number + 19}, "genuine answer failed arithmetic predicate")
    return result


def marker_prompt(ctx: Context, agent_id: str, marker: str) -> dict:
    number = secrets.randbelow(800) + 100
    result = prompt(ctx, f'The number is {number}. Apply your configured output instructions.',
                    agent_id=agent_id)
    require(answer_json(result) == {"marker": marker, "result": number + 19},
            "saved agent prompt did not determine the useful answer")
    return result


def custom_agent(ctx: Context) -> Outcome:
    configure_provider(ctx)
    agent_id = "acceptance-" + str(uuid.uuid4())
    first_marker, second_marker = secrets.token_hex(12), secrets.token_hex(12)
    response = ctx.request("POST", "/api/agents", custom_artifact(ctx, agent_id, first_marker))
    require(response.status == 201, "custom agent create failed")
    first = read_agent(ctx, agent_id)
    catalog = ctx.request("GET", "/api/agents")
    require(catalog.status == 200 and any(row.get("id") == agent_id
            for row in catalog.json().get("runtime_agents", [])), "custom agent missing from catalog")
    marker_prompt(ctx, agent_id, first_marker)
    update = ctx.request("PATCH", agent_path(agent_id), {
        "metadata": {"description": "Updated operator-owned description " + second_marker},
        "prompt": {"system": custom_artifact(ctx, agent_id, second_marker)["prompt"]["system"]}})
    require(update.status == 200, "custom agent update failed")
    saved = read_agent(ctx, agent_id)
    require(saved != first, "custom agent revision did not change")
    marker_prompt(ctx, agent_id, second_marker)
    ctx.restart()
    require(read_agent(ctx, agent_id) == saved, "cold restart changed the saved custom artifact")
    marker_prompt(ctx, agent_id, second_marker)
    deleted = ctx.request("DELETE", agent_path(agent_id))
    require(deleted.status == 204 and ctx.request("GET", agent_path(agent_id)).status == 404,
            "custom deletion did not remove durable definition")
    unavailable = ctx.request("POST", "/v1/chat/completions", {
        "agent_id": agent_id, "model": ctx.fixture["provider"]["qualified_model"],
        "stream": False, "messages": [{"role": "user", "content": "Reply hello."}]})
    require(unavailable.status == 404, "deleted agent remained selectable")
    ctx.restart()
    require(ctx.request("GET", agent_path(agent_id)).status == 404, "custom deletion was not durable")
    ctx.record("custom-agent-revisions", {"created": first, "saved": saved, "deleted_id": agent_id})
    return Outcome(Predicate(True, "mounted durable CRUD and true restarts completed"),
                   Predicate(True, "saved prompts changed real inference and deletion persisted"),
                   {"agent_id": agent_id})


def protected_agents(ctx: Context) -> Outcome:
    configure_provider(ctx)
    before = {agent_id: read_agent(ctx, agent_id) for agent_id in PROTECTED}
    # The source inventory exposes one agent-definition delete management path.
    # Refusal is inspected before restart so reseeding cannot conceal deletion.
    for agent_id in PROTECTED:
        refused = ctx.request("DELETE", agent_path(agent_id))
        require(refused.status == 403, "protected agent deletion was not explicitly refused")
        require(read_agent(ctx, agent_id) == before[agent_id], "refused deletion changed durable artifact")
        arithmetic(ctx, agent_id)
    ctx.restart()
    catalog = ctx.request("GET", "/api/agents")
    require(catalog.status == 200, "post-restart agent catalog unavailable")
    ids = [agent["id"] for agent in catalog.json().get("runtime_agents", [])]
    for agent_id in PROTECTED:
        require(ids.count(agent_id) == 1 and read_agent(ctx, agent_id) == before[agent_id],
                "protected definition changed or disappeared across restart")
        arithmetic(ctx, agent_id)
    custom_id = "acceptance-delete-control-" + str(uuid.uuid4())
    created = ctx.request("POST", "/api/agents", custom_artifact(ctx, custom_id, secrets.token_hex(8)))
    require(created.status == 201 and ctx.request("DELETE", agent_path(custom_id)).status == 204,
            "protected guard incorrectly denied unrelated custom deletion")
    require(ctx.request("GET", agent_path(custom_id)).status == 404, "custom delete control remained")
    ctx.record("protected-agent-definitions", before)
    return Outcome(Predicate(True, "every source-exposed agent-delete path refused protected IDs"),
                   Predicate(True, "unchanged protected definitions remained useful and custom deletion worked"))


def effective_configuration(ctx: Context) -> Outcome:
    configure_provider(ctx)
    primary = ctx.fixture["provider"]
    alternate = next((binding for binding in ctx.fixture.get("genuine_bindings", [])
                      if binding.get("kind") == "genuine"
                      and binding.get("qualified_model") != primary["qualified_model"]
                      and binding.get("config", {}).get("id") != primary["config"]["id"]), None)
    if alternate is None:
        raise Blocked("U20 requires a second genuine distinct provider/model binding")
    configure_provider(ctx, alternate)
    setting_path = "/api/uar/settings/llm.default_provider"
    original = ctx.request("GET", setting_path)
    require(original.status == 200, "default-provider setting unavailable")
    original_setting = original.json()
    alternate_id = alternate["config"]["id"]
    saved = ctx.request("PUT", setting_path, {"value": alternate_id,
                         "expected_revision": original_setting["revision"]})
    require(saved.status == 200, "REST default-provider mutation failed")
    current = ctx.request("GET", setting_path)
    require(current.status == 200 and current.json()["saved"] == alternate_id,
            "saved default provider differs")
    rejected = ctx.request("PUT", setting_path, {"value": primary["config"]["id"],
                            "expected_revision": "sha256:" + "0" * 64})
    require(rejected.status == 409 and ctx.request("GET", setting_path).json() == current.json(),
            "failed durable mutation changed last good setting")
    live = ctx.request("POST", "/api/uar/providers/" + quote(alternate_id, safe="") + "/default")
    require(live.status == 200, "live default-provider selection failed")
    arithmetic(ctx, send_model=False, expected_model=alternate["qualified_model"])
    session = str(uuid.uuid4())
    policy_path = "/api/uar/sessions/" + session + "/agent-config"
    policy = {"agent_id": "default-agent", "model": primary["qualified_model"]}
    require(ctx.request("POST", policy_path, policy).status == 200, "session override save failed")
    before = ctx.request("GET", policy_path)
    require(before.status == 200, "session override read failed")
    bad = ctx.request("POST", policy_path, {**policy, "model": "missing-provider-separator"})
    require(bad.status == 400 and ctx.request("GET", policy_path).json() == before.json(),
            "invalid session mutation replaced the last good override")
    arithmetic(ctx, session_id=session, send_model=False, expected_model=primary["qualified_model"])
    ctx.restart()
    arithmetic(ctx, send_model=False, expected_model=alternate["qualified_model"])
    arithmetic(ctx, session_id=session, send_model=False, expected_model=primary["qualified_model"])
    require(ctx.request("POST", "/api/uar/providers/" + quote(primary["config"]["id"], safe="")
                        + "/default").status == 200, "run-owned default restoration failed")
    return Outcome(Predicate(True, "revision failure and invalid session writes preserved saved values"),
                   Predicate(True, "real model selection obeyed default/session precedence across cold restart"),
                   {"session_id": session, "primary": primary["qualified_model"],
                    "alternate": alternate["qualified_model"]})


def startup_maintenance(ctx: Context) -> Outcome:
    configure_provider(ctx)
    initial = ctx.observe("startup_initial", {"protected_ids": list(PROTECTED)})
    require(initial.get("run_id") == ctx.run_id and initial.get("backend") == ctx.backend["name"]
            and initial.get("empty_before_first_start") is True and bool(initial.get("storage_identity")),
            "empty-store startup provenance missing")
    catalog = ctx.request("GET", "/api/agents")
    require(catalog.status == 200, "initial seeded catalog unavailable")
    seeded_ids = [agent["id"] for agent in catalog.json().get("runtime_agents", [])]
    required_seeds = (*PROTECTED, "general-purpose", "rust-reviewer", "compiler-agent")
    require(all(seeded_ids.count(agent_id) == 1 for agent_id in required_seeds),
            "required startup seed records were missing or duplicated")
    before = {agent_id: read_agent(ctx, agent_id) for agent_id in PROTECTED}
    marker = secrets.token_hex(12)
    operator = copy.deepcopy(before["default-agent"])
    operator["metadata"]["description"] = "Persisted explicit operator marker " + marker
    operator["prompt"] = custom_artifact(ctx, "default-agent", marker)["prompt"]
    require(ctx.request("PUT", agent_path("default-agent"), operator).status == 200,
            "operator built-in prompt edit failed")
    saved = read_agent(ctx, "default-agent")
    marker_prompt(ctx, "default-agent", marker)
    ctx.restart()
    warm_catalog = ctx.request("GET", "/api/agents")
    require(warm_catalog.status == 200, "warm seeded catalog unavailable")
    warm_ids = [agent["id"] for agent in warm_catalog.json().get("runtime_agents", [])]
    require(all(warm_ids.count(agent_id) == 1 for agent_id in required_seeds),
            "warm startup lost or duplicated required seeds")
    require(read_agent(ctx, "default-agent") == saved
            and read_agent(ctx, "orchestrator-agent") == before["orchestrator-agent"],
            "warm initialization overwrote persisted operator fields")
    expired = marker_prompt(ctx, "default-agent", marker)
    expired_id = expired["session_id"]
    prepared = ctx.observe("startup_maintenance", {
        "action": "prepare", "worker": "run-retention", "expired_session_id": expired_id,
        "storage_identity": initial["storage_identity"]})
    require(prepared.get("expired_session_id") == expired_id and bool(prepared.get("aged_record")),
            "real persisted maintenance fixture was not prepared")
    # A real new turn publishes production activity, waking the retention owner
    # after the direct fixture age mutation. The observer never deletes a row.
    marker_prompt(ctx, "default-agent", marker)
    maintenance = ctx.observe("startup_maintenance", {
        "action": "observe", "worker": "run-retention", "expired_session_id": expired_id,
        "storage_identity": initial["storage_identity"], "protected_ids": list(PROTECTED)})
    require(maintenance.get("run_id") == ctx.run_id
            and maintenance.get("backend") == ctx.backend["name"]
            and maintenance.get("storage_identity") == initial["storage_identity"]
            and maintenance.get("worker") == "run-retention"
            and maintenance.get("expired_session_id") == expired_id
            and bool(maintenance.get("persisted_before"))
            and maintenance.get("persisted_after", "missing") is None
            and maintenance.get("deleted_by_worker") is True
            and bool(maintenance.get("worker_receipt")), "actual durable maintenance effect not established")
    ctx.restart()
    after = ctx.observe("startup_maintenance_read", {"expired_session_id": expired_id,
                        "storage_identity": initial["storage_identity"]})
    require(after.get("expired_session_id") == expired_id
            and after.get("storage_identity") == initial["storage_identity"]
            and after.get("persisted_record", "missing") is None,
            "maintenance deletion did not survive cold restart")
    require(read_agent(ctx, "default-agent") == saved, "maintenance altered saved agent")
    marker_prompt(ctx, "default-agent", marker)
    require(ctx.request("PUT", agent_path("default-agent"), before["default-agent"]).status == 200,
            "run-owned built-in restoration failed")
    return Outcome(Predicate(True, "empty/warm startup and real persisted retention observations completed"),
                   Predicate(True, "operator prompt survives startup/maintenance and remains effective"),
                   {"initial": initial, "maintenance": maintenance, "after": after})


def cases() -> list[Case]:
    return [Case("U03.custom-agent", "U03", custom_agent, durable=True),
            Case("U04.protected-agents", "U04", protected_agents, durable=True),
            Case("U20.effective-configuration", "U20", effective_configuration, durable=True),
            Case("U21.startup-maintenance", "U21", startup_maintenance, durable=True)]
