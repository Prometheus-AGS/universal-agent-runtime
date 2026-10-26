"""U06/U07 mounted intent delegation and official AG-UI replay outcomes.

Authored source only. Import has no runtime effects; the frozen whole-phase
packet owns execution, genuine inference, mutations and cleanup.
"""
from __future__ import annotations

import json
import re
import secrets
from urllib.parse import quote
import uuid

from core import Blocked, Case, Context, Outcome, Predicate, require
from cases_agents import agent_path, custom_artifact, read_agent
from cases_prompt import configure_provider
from protocol_clients import sse, verify_run


ROOT = "orchestrator-agent"


def prerequisites(ctx: Context) -> None:
    if ctx.backend["name"] not in ("postgres_pgvector", "surreal_disk"):
        raise Blocked("U06/U07 require an actual supported durable backend")
    authority = ctx.fixture.get("orchestrator", {})
    if not all(isinstance(authority.get(key), str) and authority[key] for key in (
            "expected_principal_key", "expected_history_owner_key", "expected_subject")):
        raise Blocked("U06/U07 require frozen expected authority identities for exact durable correlation")
    if ctx.fixture.get("provider", {}).get("kind") != "genuine":
        raise Blocked("U06/U07 require configured genuine provider inference")
    if not ctx.backend.get("observers", {}).get("orchestrator_run"):
        raise Blocked("U06/U07 require the concrete owned durable orchestrator observer")


def specialist(ctx: Context, domain: str, increment: int) -> dict:
    agent_id = "acceptance-routing-" + uuid.uuid4().hex
    marker = secrets.token_hex(16)
    artifact = custom_artifact(ctx, agent_id, marker)
    artifact["metadata"].update(title=domain + " specialist", description=(
        "Exclusively handles " + domain + " computations and remembers the carry code from earlier user dialogue."),
        tags=["acceptance", domain])
    artifact["prompt"] = {"system": (
        'Return only JSON with exactly marker, result, carry. Your private marker is "' + marker
        + '". Put that marker in marker. For the user quantity, set result to quantity plus '
        + str(increment) + '. Copy the carry code supplied in an earlier or current user turn into carry. '
        'Use the supplied dialogue history. Never invent a missing carry code.'), "instructions": []}
    provider, model = ctx.fixture["provider"]["qualified_model"].split("/", 1)
    artifact["policy"]["provider"]["default"] = {"provider": provider, "model": model}
    require(ctx.request("POST", "/api/agents", artifact).status == 201, "routing specialist creation failed")
    saved = read_agent(ctx, agent_id)
    return {"id": agent_id, "title": saved["metadata"]["title"], "domain": domain,
            "increment": increment, "marker": marker}


def parse_answer(content: str) -> dict:
    # The graph intentionally prefixes the selected registered ID before JSON.
    position = content.find("{")
    require(position >= 0, "selected child returned no structured useful result")
    value, end = json.JSONDecoder().raw_decode(content[position:])
    require(not content[position + end:].strip() and isinstance(value, dict),
            "child result has trailing/unstructured answer content")
    return value


def frames(response) -> list[dict]:
    sse(response)  # Reuse mounted SSE status/content-type/JSON validation.
    result = []
    for block in re.split(r"\r?\n\r?\n", response.body.decode("utf-8")):
        lines = block.splitlines()
        raw = "\n".join(line[5:].lstrip() for line in lines if line.startswith("data:"))
        if not raw:
            continue
        value = json.loads(raw)
        require(isinstance(value, dict), "official AG-UI frame is not an object")
        if re.search("thinking|reasoning|thought", str(value.get("type", "")), re.I):
            continue
        cursor = next((line[3:].strip() for line in lines if line.startswith("id:")), None)
        require(isinstance(cursor, str) and bool(cursor), "official AG-UI frame lacks replay identity")
        result.append({"id": cursor, "data": value})
    require(result and len({row["id"] for row in result}) == len(result), "wire event IDs absent or duplicated")
    return result


def selected_frame(events: list[dict], thread_id: str, wire_id: str, expected: dict) -> tuple[int, dict]:
    require(not any(row["data"].get("type") in ("RUN_ERROR", "RUN_CANCELLED") for row in events),
            "positive orchestrator request emitted a failure terminal")
    matches = [(index, row) for index, row in enumerate(events)
               if row["data"].get("type") == "CUSTOM" and row["data"].get("name") == "uar.agent.selected"]
    require(len(matches) == 1, "exactly one actual child selection is required")
    index, row = matches[0]
    event, value = row["data"], row["data"]["value"]
    require(event.get("threadId") == thread_id and event.get("runId") == wire_id
            and event.get("agentId") == ROOT and value.get("sourceAgentId") == ROOT,
            "selection is not correlated to the root wire request")
    require(value.get("agent") == {"id": expected["id"], "title": expected["title"]}
            and value.get("selectionMethod") == "intent" and bool(value.get("selectedThreadId"))
            and re.fullmatch(r"[0-9a-f]{64}", value.get("rosterRevision", "")),
            "intent did not select the actual current registered specialist/thread")
    terminals = [i for i, event in enumerate(events) if event["data"].get("type") == "RUN_FINISHED"]
    require(len(terminals) == 1 and index < terminals[0], "selection did not precede successful parent completion")
    starts = [row["data"] for row in events if row["data"].get("type") == "RUN_STARTED"]
    terminal = events[terminals[0]]["data"]
    require(len(starts) == 1 and starts[0].get("threadId") == thread_id and starts[0].get("runId") == wire_id
            and starts[0].get("protocolVersion") == "1.0" and terminal.get("threadId") == thread_id
            and terminal.get("runId") == wire_id and terminal.get("outcome", {}).get("type") == "success",
            "official start/finish identity or success outcome differs")
    content_positions = [i for i, event in enumerate(events)
                         if event["data"].get("type") == "TEXT_MESSAGE_CONTENT"]
    require(content_positions and index < content_positions[0], "parent answer was published before selection")
    return index, row


def turn(ctx: Context, thread_id: str, expected: dict, quantity: int, carry: str,
         *, previous: str | None = None, seed: bool = False, replay: bool = False) -> dict:
    wire_id = str(uuid.uuid4())
    text = (f'Please perform the {expected["domain"]} computation for quantity {quantity}. '
            + (f'The carry code for this conversation is {carry}. ' if seed else
               'Use the carry code I gave earlier in this conversation; do not substitute another code. ')
            + 'Return the configured specialist result.')
    # No specialist ID, private marker, or arithmetic rule appears in routing input.
    body = {"threadId": thread_id, "runId": wire_id, "protocolVersion": "1.0",
            "messages": [{"id": str(uuid.uuid4()), "role": "user", "content": text}],
            "tools": [], "context": [], "forwardedProps": {"uar": {
                "model": ctx.fixture["provider"]["qualified_model"]}}}
    if previous is not None:
        body["parentRunId"] = previous
    path = "/api/uar/ag-ui/agents/" + ROOT
    events = frames(ctx.request("POST", path, body))
    index, selection = selected_frame(events, thread_id, wire_id, expected)
    content = "".join(row["data"].get("delta", "") for row in events
                      if row["data"].get("type") == "TEXT_MESSAGE_CONTENT")
    expected_json = {"marker": expected["marker"], "result": quantity + expected["increment"], "carry": carry}
    require(parse_answer(content) == expected_json, "actual child private instructions/history did not determine answer")
    values = {"thread_id": thread_id, "wire_run_id": wire_id, "agent_id": ROOT,
              "selected_thread_id": selection["data"]["value"]["selectedThreadId"],
              "selected_agent_id": expected["id"], "expected_result": expected_json,
              "carry": carry, "selection": selection["data"]["value"]}
    observed = ctx.observe("orchestrator_run", values)
    require(observed.get("wire_run_id") == wire_id and observed.get("child_useful") is True
            and observed.get("carry_in_child_user_history") is True
            and observed.get("root_native_run_id") != observed.get("child_run_id")
            and observed.get("accounting_attribution") == "shared_root_group",
            "actual durable root/child result, fork context or accounting binding is missing")
    verify_run(ctx, observed["root_native_run_id"], observed["session_id"], ROOT,
               ctx.fixture["provider"]["qualified_model"])
    if replay:
        if seed:
            ctx.restart()
        stream_path = path + "/threads/" + quote(thread_id, safe="") + "/runs/" + quote(wire_id, safe="") + "/stream"
        after = selection["id"]
        resumed = frames(ctx.request("GET", stream_path, headers={"Last-Event-ID": after}))
        expected_suffix = events[index + 1:]
        require(resumed == expected_suffix, "reconnect did not replay the exact suffix after selection")
        # Retrying the same official POST is attachment to its stored identity.
        retried = frames(ctx.request("POST", path, body, {"Last-Event-ID": after}))
        require(retried == expected_suffix, "identical POST retry changed the retained event suffix")
        again = ctx.observe("orchestrator_run", values)
        require(again == observed, "reconnect/retry changed durable native/thread/call identities or generated new work")
        ctx.record("orchestrator-replay", {"wire_run_id": wire_id, "after": after,
                   "suffix_ids": [row["id"] for row in resumed], "duplicate_generation": False,
                   "root_call_ids": observed["root_call_ids"], "cold_restart": seed,
                   "observation": "retained-cursor replay"})
    ctx.record("orchestrator-turn", {"wire_thread_id": thread_id, "wire_run_id": wire_id,
               "selection": selection, "answer": expected_json, "durable": observed,
               "frame_ids": [row["id"] for row in events]})
    return {"wire_id": wire_id, "roster_revision": selection["data"]["value"]["rosterRevision"],
            "observation": observed}


def live_roster(ctx: Context) -> Outcome:
    prerequisites(ctx)
    configure_provider(ctx)
    created = []
    try:
        orchard = specialist(ctx, "orchard invoice", 37)
        created.append(orchard["id"])
        estuary = specialist(ctx, "estuary survey", 61)
        created.append(estuary["id"])
        thread, carry = str(uuid.uuid4()), secrets.token_hex(12)
        first = turn(ctx, thread, orchard, secrets.randbelow(800) + 100, carry, seed=True)
        second = turn(ctx, thread, estuary, secrets.randbelow(800) + 100, carry, previous=first["wire_id"])
        require(second["roster_revision"] == first["roster_revision"], "unchanged eligible roster hash drifted")
        # Update current catalog metadata and private prompt without naming IDs to the router.
        updated = read_agent(ctx, orchard["id"])
        orchard["title"] = "Updated orchard invoice specialist"
        updated["metadata"]["title"] = orchard["title"]
        updated["metadata"]["description"] = "Updated exclusive orchard invoice arithmetic specialist."
        new_marker = secrets.token_hex(16)
        updated["prompt"]["system"] = updated["prompt"]["system"].replace(orchard["marker"], new_marker).replace(
            "quantity plus 37", "quantity plus 75")
        orchard.update(marker=new_marker, increment=75)
        require(ctx.request("PUT", agent_path(orchard["id"]), updated).status == 200, "live specialist update failed")
        third = turn(ctx, thread, orchard, secrets.randbelow(800) + 100, carry, previous=second["wire_id"])
        require(third["roster_revision"] != second["roster_revision"], "updated current roster retained stale hash")
        replacement = specialist(ctx, "volcano ledger", 83)
        created.append(replacement["id"])
        fourth = turn(ctx, thread, replacement, secrets.randbelow(800) + 100, carry, previous=third["wire_id"])
        require(fourth["roster_revision"] != third["roster_revision"], "added specialist used stale roster hash")
        require(ctx.request("DELETE", agent_path(orchard["id"])).status == 204, "custom specialist deletion failed")
        created.remove(orchard["id"])
        fifth = turn(ctx, thread, estuary, secrets.randbelow(800) + 100, carry, previous=fourth["wire_id"])
        require(fifth["roster_revision"] != fourth["roster_revision"], "deleted specialist remained in roster hash")
        require(ctx.request("GET", agent_path(orchard["id"])).status == 404, "deleted custom roster entry remains")
        ctx.restart()
        sixth = turn(ctx, thread, replacement, secrets.randbelow(800) + 100, carry, previous=fifth["wire_id"])
        require(sixth["roster_revision"] == fifth["roster_revision"], "cold restart changed unchanged roster identity")
        return Outcome(Predicate(True, "actual intent selections track update/add/delete and cold durable roster"),
                       Predicate(True, "selected real children apply private arithmetic and inherited fresh carry fact"),
                       {"wire_thread_id": thread, "turns": 6})
    finally:
        for agent_id in created:
            require(ctx.request("DELETE", agent_path(agent_id)).status == 204, "owned specialist cleanup failed")


def official_replay(ctx: Context) -> Outcome:
    prerequisites(ctx)
    configure_provider(ctx)
    created = []
    try:
        orchard = specialist(ctx, "orchard invoice", 37)
        created.append(orchard["id"])
        estuary = specialist(ctx, "estuary survey", 61)
        created.append(estuary["id"])
        thread, carry = str(uuid.uuid4()), secrets.token_hex(12)
        first = turn(ctx, thread, orchard, secrets.randbelow(800) + 100, carry, seed=True, replay=True)
        second = turn(ctx, thread, estuary, secrets.randbelow(800) + 100, carry,
                      previous=first["wire_id"], replay=True)
        require(first["wire_id"] != second["wire_id"]
                and first["observation"]["root_native_run_id"] != second["observation"]["root_native_run_id"]
                and first["observation"]["session_id"] == second["observation"]["session_id"],
                "fresh wire turns did not preserve conversation while creating distinct native executions")
        return Outcome(Predicate(True, "official POST, Last-Event-ID GET and idempotent POST replay match durable identities"),
                       Predicate(True, "two intent-selected children return useful private results without replay generation"),
                       {"wire_thread_id": thread, "turns": 2, "ui_rendering": "not assessed by wire case"})
    finally:
        for agent_id in created:
            require(ctx.request("DELETE", agent_path(agent_id)).status == 204, "owned specialist cleanup failed")


def cases() -> list[Case]:
    return [Case("U06.live-roster-intent", "U06", live_roster, durable=True),
            Case("U07.official-agui-selection-replay", "U07", official_replay, durable=True)]
