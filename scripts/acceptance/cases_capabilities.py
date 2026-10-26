"""U22/U23/U25 real capability witnesses; no execution on import.

Every positive inference is genuine. Controlled transport failures and actual
owned-file MCP tools are negative/effect fixtures, never assistant stubs.
"""
from __future__ import annotations

import copy
import json
import os
from pathlib import Path
import secrets
import time
from urllib.parse import quote
import uuid

from core import Blocked, Case, Outcome, Predicate, require
from cases_prompt import answer_json, configure_provider, prompt
from protocol_clients import verify_run
from cases_strategies import setting, observed


def runtime_endpoint(ctx, endpoint):
    """Route only explicitly declared host listeners into the Linux runtime."""
    if ctx.fixture.get("runtime_platform") == "linux-container":
        from linux_runtime import route_endpoint
        return route_endpoint(endpoint, ctx.fixture["linux_runtime"])
    return endpoint


def observe(ctx, action, values):
    result = ctx.observe("cap_" + action, values)
    require(result.get("run_id") == ctx.run_id and result.get("backend") == ctx.backend["name"],
            "capability observer is not bound to this invocation/adapter")
    return result


def foreign_headers(ctx):
    name = ctx.fixture.get("capabilities", {}).get("foreign_authorization_env")
    if not name or not os.environ.get(name):
        raise Blocked("actual second credential principal is required")
    return {"Authorization": os.environ[name]}


def agent(ctx, tools, instructions, memory=False):
    original = ctx.request("GET", "/api/agents/default-agent")
    require(original.status == 200, "native agent template unavailable")
    value = copy.deepcopy(original.json())
    name = "acceptance-capability-" + str(uuid.uuid4())
    value.update(id=name, version="1.0.0", extensions={})
    value["metadata"].update(title=name, description="Run-owned capability predicate")
    value["runtime"]["entry"] = "default"
    value["prompt"] = {"system": instructions, "instructions": []}
    value["memory"]["conversation"]["enabled"] = memory
    value["memory"]["kb"]["enabled"] = False
    value["policy"]["tools"]["allow"] = tools
    value["policy"]["tools"]["deny"] = []
    require(ctx.request("POST", "/api/agents", value).status == 201, "owned agent create failed")
    return name


def events(ctx, run_id):
    result = ctx.request("GET", "/api/uar/runs/" + run_id + "/events")
    require(result.status == 200 and result.json().get("gapReason") is None, "actual public event history is incomplete")
    return result.json()["events"]


def answer(events_value):
    content = "".join(event["data"]["delta"]["text"] for event in events_value
                      if event["eventName"] == "agui.message.delta")
    return answer_json({"content": content})


def paired(proof, allowed):
    require(proof["status"] == "done", "positive capability task did not finish successfully")
    for call in proof["calls"]:
        require(call["name"] in allowed, "model invoked an undeclared tool")
        matches = [row for row in proof["results"] if row["id"] == call["id"] and row["position"] > call["position"]]
        require(len(matches) == 1, "real tool call/result pairing was lost or duplicated")
        require(any(row["call_id"] == call["id"] and row["completeness"]["status"] == "complete"
                    for row in proof["canonical_tools"]), "paired result lacks complete canonical receipt")


def tool_turn(ctx, agent_id, session, text, allowed, decision=True, mcp=None, strategy=None, hard_deny=False):
    policy = {"version": 1, "agent_id": agent_id, "skills": {"mode": "none", "ids": [], "denied_ids": []},
              "tools": {"mode": "selected", "ids": allowed, "denied_ids": []}, "tool_approval": "deny" if hard_deny else "ask",
              "memory_enabled": False}
    if strategy is not None:
        policy["context_strategy"] = strategy
    require(ctx.request("PUT", "/api/uar/conversations/" + session + "/policy", policy).status == 200,
            "exact conversation tool policy did not persist")
    payload = {"agent_id": agent_id, "model": ctx.fixture["provider"]["qualified_model"],
               "session_id": session, "input": text}
    if mcp:
        payload["mcp_servers"] = [{"name": mcp}]
    started = ctx.request("POST", "/api/uar/runs", payload)
    require(started.status == 200, "native governed run admission failed")
    run_id = started.json()["run_id"]
    decisions, deadline = [], time.monotonic() + min(ctx.remaining(), ctx.fixture["capabilities"]["case_seconds"])
    while time.monotonic() < deadline:
        status = ctx.request("GET", "/api/uar/runs/" + run_id)
        require(status.status == 200, "owned capability run disappeared")
        if status.json()["status"] in ("done", "error", "cancelled"):
            break
        pending = ctx.request("GET", "/api/uar/runs/" + run_id + "/tool-approval/pending")
        require(pending.status == 200, "actual approval snapshot unavailable")
        challenge = pending.json().get("pending")
        if challenge:
            require(not hard_deny, "hard-denied tool requested an approval bypass")
            require(challenge.get("name") in allowed and challenge.get("approvalId"),
                    "tool approval requested a foreign capability or lacked challenge identity")
            require(challenge["approvalId"] not in decisions and len(decisions) < 4,
                    "duplicate or excessive real approval request")
            resolved = ctx.request("POST", "/api/uar/runs/" + run_id + "/tool-approval",
                                   {"approved": decision, "approval_id": challenge["approvalId"]})
            require(resolved.status == 200 and resolved.json().get("resolved") is True,
                    "fixture host decision did not resolve the exact challenge")
            decisions.append(challenge["approvalId"])
        time.sleep(min(0.1, ctx.remaining()))
    else:
        raise Blocked("actual governed tool task exceeded finite control allowance")
    require(hard_deny or decisions, "approval/denial case never reached an actual tool challenge")
    return run_id, status.json(), decisions


def tools(ctx, native):
    configure_provider(ctx)
    config = ctx.fixture["capabilities"]
    catalog = ctx.request("GET", "/api/tools")
    require(catalog.status == 200, "tool inventory is unavailable")
    if native:
        allowed = ["file_write", "file_read"]
        builtin = {row["name"] for row in catalog.json().get("built_in_tools", [])}
        require(set(allowed) <= builtin, "actual native filesystem tools are disabled")
        mcp = None
    else:
        mcp = config["mcp_server_name"]
        rows = [row for row in catalog.json().get("tools", [])
                if row.get("server") == mcp and row.get("name") == "owned_record"]
        require(len(rows) == 1 and rows[0]["source"] == "mcp_server", "actual stdio file peer was not discovered")
        allowed = [rows[0]["namespaced_name"]]
    instructions = ('Use exactly the requested real tools, never simulate a result. After append and read, '
                    'return only JSON with fact copied from the first JSON line and result equal to number+23. '
                    'If host denies, return only {"denied":true}; never assert an effect without a successful tool result.')
    selected = agent(ctx, allowed, instructions)
    receipts = []
    for permitted in (True, False, "hard_deny"):
        name, marker = "tool-" + secrets.token_hex(10), secrets.token_hex(12)
        owned = observe(ctx, "prepare_effect", {"name": name})
        if native:
            text = (f'Call file_write with path={json.dumps(owned["path"])}, content="{marker}\\n", append=true exactly once. '
                    f'Then call file_read on the same path and use its actual result. Do not access another path.')
        else:
            text = (f'Call {allowed[0]} with name="{name}", operation="append", marker="{marker}" exactly once; '
                    f'then call it with operation="read" and the same name/marker. Use the actual read result.')
        session = str(uuid.uuid4())
        run_id, status, decisions = tool_turn(ctx, selected, session, text, allowed, permitted is True, mcp,
                                               hard_deny=permitted == "hard_deny")
        effect = observe(ctx, "effect", {"name": name, "marker": marker})
        if permitted is True:
            require(status["status"] == "done", "approved real tool task failed")
            verify_run(ctx, run_id, session, selected, ctx.fixture["provider"]["qualified_model"])
            proof = observe(ctx, "run_evidence", {"run_id": run_id})
            paired(proof, allowed)
            require(answer(proof["events"]["events"]) == {"fact": owned["fact"], "result": owned["number"] + 23},
                    "real next-round answer did not consume the private tool result")
            require(effect["exactly_one_append"] is True and effect["marker_occurrences"] == 1,
                    "approved tool did not perform exactly one separately observed file effect")
            if native:
                writes = [row for row in proof["calls"] if row["name"] == "file_write"]
                reads = [row for row in proof["calls"] if row["name"] == "file_read"]
                require(len(writes) == len(reads) == 1 and writes[0]["arguments"]["path"] == owned["path"]
                        and reads[0]["arguments"]["path"] == owned["path"], "native tool escaped its exact requested record")
            else:
                committed = [row for row in effect["mcp_receipts"] if row.get("effect_committed")]
                require(len(committed) == 1 and committed[0]["operation"] == "append"
                        and len([row for row in effect["mcp_receipts"] if row["operation"] == "read"]) == 1,
                        "actual MCP effect/read receipts were missing or duplicated")
        else:
            require(effect["unchanged"] is True and effect["marker_occurrences"] == 0
                    and not effect["mcp_receipts"], "host-denied tool executed or changed the owned record")
            if permitted == "hard_deny":
                require(status.get("effective_run_policy", {}).get("tool_approval") == "deny" and not decisions,
                        "actual hard-deny policy was not enforced without a human approval bypass")
            if status["status"] == "done":
                require(answer(events(ctx, run_id)) == {"denied": True}, "denied tool falsely reported success")
            else:
                require(status["status"] in ("error", "cancelled"), "denial had no honest terminal")
        receipts.append({"run_id": run_id, "permitted": permitted, "effect": effect, "decisions": decisions})
    require(ctx.request("DELETE", "/api/agents/" + selected).status == 204, "owned capability agent deletion failed")
    return Outcome(Predicate(True, "real discovered tool and exact host approval/denial challenges exercised"),
                   Predicate(True, "authorized effect occurred once; result was consumed; denied effect never occurred"),
                   {"kind": "native" if native else "mcp-discovered", "runs": receipts})


def isolation(ctx):
    configure_provider(ctx)
    secret = secrets.token_hex(16)
    first = prompt(ctx, 'Remember the private vault code ' + secret + '. Return only JSON {"stored":true}.')
    require(answer_json(first) == {"stored": True}, "owned private history was not planted")
    headers = foreign_headers(ctx)
    for suffix, method in (("", "GET"), ("/events", "GET"), ("/cancel", "POST")):
        refused = ctx.request(method, "/api/uar/runs/" + first["run_id"] + suffix, headers=headers)
        require(refused.status in (403, 404), "another principal could read or cancel an owned run")
    other = ctx.request("POST", "/v1/chat/completions", {"agent_id": "default-agent", "session_id": first["session_id"],
        "model": ctx.fixture["provider"]["qualified_model"], "stream": False, "memory_enabled": False,
        "messages": [{"role": "user", "content": 'What private vault code was supplied earlier? '
                      'If absent return only JSON {"code":"UNKNOWN"}; do not guess.'}]}, headers=headers)
    require(other.status == 200 and other.json()["choices"][0]["finish_reason"] == "stop", "foreign partition control inference failed")
    content = other.json()["choices"][0]["message"]["content"]
    require(answer_json({"content": content}) == {"code": "UNKNOWN"} and secret not in content,
            "continuing the same external session ID exposed another credential principal's history")
    receipt = observe(ctx, "isolation", {"session_id": first["session_id"], "secret": secret})
    require(receipt["owner_count"] == 2 and sum(row["private_marker_present"] for row in receipt["records"]) == 1,
            "same session ID was not physically partitioned into two durable owner records")
    ctx.restart()
    recalled = prompt(ctx, 'What was my private vault code? Return only JSON {"code":"the exact code"}.', first["session_id"])
    require(answer_json(recalled) == {"code": secret}, "owner's useful private continuation was lost")
    return Outcome(Predicate(True, "foreign read/events/cancel refused; equal public session IDs persisted under distinct owners"),
                   Predicate(True, "foreign continuation lacked the fact; true owner retained it after cold restart"), receipt)


def failure_honesty(ctx):
    configure_provider(ctx)
    owned = observe(ctx, "prepare_effect", {"name": "fault-" + secrets.token_hex(10)})
    config = ctx.fixture["capabilities"]
    binary, program = config["python_binary"], config["tools_fixture_path"]
    require(all(str(Path(value).resolve()) in ctx.fixture["_frozen_paths"] for value in (binary, program)),
            "negative provider peer executable/source was not frozen")
    command = [binary, program, "provider-fault", owned["root"]]
    if config.get("negative_provider_port") is not None:
        port = config["negative_provider_port"]
        require(type(port) is int and 1024 <= port <= 65535, "invalid frozen negative provider port")
        command.append(str(port))
    process = ctx.owned_process(command)
    deadline = time.monotonic() + min(ctx.remaining(), 10)
    ready_path = Path(owned["root"]) / ("provider-fault-" + str(process.pid) + ".json")
    while not ready_path.exists() and time.monotonic() < deadline:
        time.sleep(0.05)
    ready = observe(ctx, "provider_ready", {"pid": process.pid})
    bad = copy.deepcopy(ctx.fixture["provider"]["config"])
    bad.update(id="acceptance-denied-" + secrets.token_hex(8), base_url=runtime_endpoint(ctx, ready["origin"] + "/v1"), api_key="run-owned-invalid-credential")
    created = ctx.request("POST", "/api/uar/providers/", bad)
    require(created.status == 201, "bounded negative provider configuration rejected")
    model = bad["id"] + "/" + bad["default_model"]
    route_input = {**config["routing_requirements"], "preferred_provider": bad["id"]}
    before_route = ctx.request("POST", "/api/uar/route", route_input)
    require(before_route.status == 200 and before_route.json().get("model") == model,
            "owned fault provider was not actually eligible before its failure")
    failed = ctx.request("POST", "/v1/chat/completions", {"agent_id": "default-agent", "model": model,
        "session_id": str(uuid.uuid4()), "stream": False, "memory_enabled": False,
        "messages": [{"role": "user", "content": "Return a real answer only if the configured provider succeeds."}]})
    body = failed.json()
    successful = (failed.status == 200 and any(row.get("finish_reason") == "stop" for row in body.get("choices", [])))
    require(not successful and bool(body.get("error") or body.get("choices")), "provider denial became an apparent successful task")
    after = observe(ctx, "provider_ready", {"pid": process.pid})
    require(after["provider_denial_calls"] - ready["provider_denial_calls"] == 1,
            "negative control must reach exactly one real401 request, without an automatic retry")
    health = ctx.request("GET", "/api/uar/providers/health")
    status = health.json().get("providers", {}).get(bad["id"], {}) if health.status == 200 else {}
    require(status.get("healthy") is False and status.get("consecutive_errors", 0) >= 1
            and status.get("cooldown_remaining_secs") is not None,
            "actual provider error did not establish its configured cooldown")
    cooling_route = ctx.request("POST", "/api/uar/route", route_input)
    genuine = {row["qualified_model"] for row in [ctx.fixture["provider"], *ctx.fixture.get("genuine_bindings", [])]}
    require(cooling_route.status == 200 and cooling_route.json().get("model") in genuine,
            "cooling provider remained eligible or routing chose an unapproved destination")
    routed = prompt(ctx, 'Return only JSON {"healthy_destination":true}.', model=cooling_route.json()["model"])
    require(answer_json(routed) == {"healthy_destination": True}, "healthy routed destination did not complete useful work")
    deadline = time.monotonic() + min(ctx.remaining(), config["case_seconds"])
    recovered = None
    while time.monotonic() < deadline:
        health = ctx.request("GET", "/api/uar/providers/health")
        recovered = health.json().get("providers", {}).get(bad["id"], {}) if health.status == 200 else {}
        if recovered.get("healthy") is True:
            break
        time.sleep(0.1)
    require(recovered and recovered.get("healthy") is True,
            "actual cooldown eligibility did not recover within the frozen allowance")
    recovered_route = ctx.request("POST", "/api/uar/route", route_input)
    require(recovered_route.status == 200 and recovered_route.json().get("model") == model,
            "expired cooldown did not restore the provider's actual route eligibility")
    ctx.record("provider-cooldown", {"provider_id": bad["id"], "cooling": status, "recovered": recovered,
               "healthy_run_id": routed["run_id"],
               "meaning": "observed error exclusion and expiry eligibility; fault upstream remains a deliberate401"})
    ctx.stop_process(process)
    require(ctx.request("DELETE", "/api/uar/providers/" + bad["id"]).status in (200, 204), "negative provider cleanup failed")
    name = "acceptance-" + ctx.run_id + "-write-refusal-" + secrets.token_hex(5)
    installed = observe(ctx, "store_fault", {"name": name, "operation": "install"})
    require(installed["records"] == [], "fault target KB already existed")
    try:
        refused = ctx.request("POST", "/api/uar/knowledge-bases/", {"name": name})
        require(refused.status in (500, 503) and refused.body, "real DB write refusal was ignored or reported as success")
        actual = observe(ctx, "store_fault", {"name": name, "operation": "read"})
        require(actual["records"] == [], "failed DB mutation committed the supposedly refused record")
    finally:
        observe(ctx, "store_fault", {"name": name, "operation": "remove"})
    number = secrets.randbelow(900) + 100
    result = prompt(ctx, f'Return only JSON {{"sum":{number}+23}}.')
    require(answer_json(result) == {"sum": number + 23}, "genuine inference did not recover after bounded faults")
    return Outcome(Predicate(True, "real provider401 and actual scoped DB write failure surfaced without success terminals"),
                   Predicate(True, "failed durable mutation left no row and genuine inference remained useful"))


def unknown_effect(ctx):
    configure_provider(ctx)
    server = ctx.fixture["capabilities"]["mcp_server_name"]
    catalog = ctx.request("GET", "/api/tools")
    require(catalog.status == 200, "actual MCP inventory unavailable")
    tools = [row["namespaced_name"] for row in catalog.json()["tools"]
             if row.get("server") == server and row.get("name") == "owned_record"]
    require(len(tools) == 1, "unknown-effect fixture peer was not actually discovered")
    selected = agent(ctx, tools, "Perform only the exact requested operation once. Never retry after a transport loss or claim success.")
    name, marker = "unknown-" + secrets.token_hex(10), secrets.token_hex(12)
    observe(ctx, "prepare_effect", {"name": name})
    session = str(uuid.uuid4())
    run_id, status, _ = tool_turn(ctx, selected, session,
        f'Call {tools[0]} exactly once with name="{name}", operation="commit_disconnect", marker="{marker}". '
        'A response loss leaves the effect uncertain; never retry.', tools, mcp=server)
    require(status["status"] == "error", "unknown committed effect produced a successful task terminal")
    before = observe(ctx, "effect", {"name": name, "marker": marker})
    require(before["exactly_one_append"] and before["marker_occurrences"] == 1
            and len(before["mcp_receipts"]) == 1 and before["mcp_receipts"][0]["operation"] == "commit_disconnect",
            "real committed-then-disconnected effect was not independently established")
    proof = observe(ctx, "run_evidence", {"run_id": run_id})
    unknown = [row for row in proof["canonical_tools"] if row["outcome"] == "outcome_unknown"]
    require(len(unknown) == 1 and len(proof["calls"]) == 1
            and any(row.get("state") == "outcome_unknown" for row in proof["tool_admission"]),
            "unknown tool effect was hidden, re-executed or falsely reconciled")
    ctx.restart()
    after = observe(ctx, "effect", {"name": name, "marker": marker})
    durable = observe(ctx, "run_evidence", {"run_id": run_id})
    require(after["sha256"] == before["sha256"] and after["marker_occurrences"] == 1
            and len([row for row in durable["canonical_tools"] if row["outcome"] == "outcome_unknown"]) == 1,
            "restart retried the uncertain mutation or manufactured a successful tool result")
    require(ctx.request("DELETE", "/api/agents/" + selected).status == 204, "owned unknown-effect agent cleanup failed")
    return Outcome(Predicate(True, "real file commit followed by actual MCP process exit produced a failed task"),
                   Predicate(True, "durable unknown outcome survived restart with one effect and no automatic retry"),
                   {"run_id": run_id, "effect_sha256": before["sha256"]})


def complete(ctx, text, *, memory=False, session=None, model=None, response_format=None, image=None, policy=None):
    session = session or str(uuid.uuid4())
    message = text if image is None else [{"type": "text", "text": text}, {"type": "image_url", "image_url": {"url": image}}]
    request = {"agent_id": "default-agent", "session_id": session, "model": model or ctx.fixture["provider"]["qualified_model"],
               "stream": False, "memory_enabled": memory, "messages": [{"role": "user", "content": message}],
               "max_completion_tokens": ctx.fixture["limits"]["output_tokens"]}
    if response_format is not None:
        request["response_format"] = response_format
    if policy is not None:
        request["run_policy"] = policy
    response = ctx.request("POST", "/v1/chat/completions", request)
    require(response.status == 200 and response.json()["choices"][0]["finish_reason"] == "stop", "capability inference did not succeed")
    body = response.json()
    run_id = body["id"].removeprefix("chatcmpl-")
    verify_run(ctx, run_id, session, "default-agent", request["model"])
    return {"content": body["choices"][0]["message"]["content"], "run_id": run_id, "session_id": session}


def memory_lifecycle(ctx):
    configure_provider(ctx)
    subject, code, replacement = "Parcel-" + secrets.token_hex(8), secrets.token_hex(16), secrets.token_hex(16)
    content = f"The private delivery code for {subject} is {code}."
    saved = ctx.request("POST", "/api/memory", {"content": content, "categories": ["acceptance-private-code"]})
    require(saved.status == 200 and saved.json().get("status") == "success" and saved.json().get("id"), "actual owned memory save failed")
    memory_id = saved.json()["id"]
    before = observe(ctx, "memory", {"id": memory_id})
    require(before["record_status"] == 200 and before["owner_id"].startswith("v1:k:"), "memory ownership was not bound to credential principal")
    def recall(expected):
        result = complete(ctx, 'Recall the private delivery code for ' + subject + '. Return only JSON {"code":"exact code"}.', memory=True)
        require(answer_json(result) == {"code": expected}, "real memory recall did not produce the planted private fact")
        hits = [item for event in events(ctx, result["run_id"]) if event["eventName"] == "agui.memory.recall"
                for item in event["data"]["items"]]
        require(any(expected in item.get("value", "") and item.get("source") == "memory_context" for item in hits),
                "private answer lacks actual pre-call scoped memory-recall provenance")
    recall(code)
    denied = ctx.request("GET", "/api/memory/records/" + quote(memory_id, safe=""), headers=foreign_headers(ctx))
    require(denied.status == 404, "foreign owner read a planted memory")
    ctx.restart()
    after = observe(ctx, "memory", {"id": memory_id})
    require(before["storage_identity"] == after["storage_identity"] and before["record_sha256"] == after["record_sha256"]
            and before["server_pid"] != after["server_pid"], "memory did not persist unchanged across actual process restart")
    recall(code)
    changed = ctx.request("PATCH", "/api/memory/records/" + quote(memory_id, safe=""),
                          {"content": f"The private delivery code for {subject} is {replacement}."})
    require(changed.status == 200, "owned memory update failed")
    recall(replacement)
    history = ctx.request("GET", "/api/memory/records/" + quote(memory_id, safe="") + "/history")
    require(history.status == 200 and history.json().get("history"), "memory update has no actual history")
    require(ctx.request("DELETE", "/api/memory/records/" + quote(memory_id, safe="")).status == 200, "memory delete failed")
    removed = observe(ctx, "memory", {"id": memory_id})
    require(removed["record_status"] == 404 and removed["control_status"] == 200
            and removed["control"].get("deleted") is True, "memory deletion lacks durable unqueryable tombstone")
    unknown = complete(ctx, 'Recall the private delivery code for ' + subject + '. If absent return only JSON {"code":"UNKNOWN"}.', memory=True)
    require(answer_json(unknown) == {"code": "UNKNOWN"}, "deleted memory remained recallable in a fresh session")
    ctx.restart()
    purged = observe(ctx, "memory", {"id": memory_id})
    require(purged["record_status"] == 404 and purged["control"].get("purged") is True,
            "deleted memory returned or its physical purge remained unfinished after restart")
    return Outcome(Predicate(True, "actual scoped memory record/update/history/delete observed with disk and restart identity"),
                   Predicate(True, "genuine recall used current private fact; foreign and deleted facts stayed unavailable"),
                   {"memory_id": memory_id, "storage_identity": before["storage_identity"]})


def formats_vision(ctx):
    configure_provider(ctx)
    image = observe(ctx, "image", {"name": "vision-" + secrets.token_hex(8)})
    visual = complete(ctx, 'Return only JSON {"colors":["left block color","middle block color","right block color"]}. '
                      'Use lower-case red, green, blue or yellow from the actual image.', image=image["data_url"])
    require(answer_json(visual) == {"colors": image["colors"]}, "real image observation did not distinguish actual random colored panels")
    nonce, number = secrets.token_hex(8), secrets.randbelow(900) + 100
    schema = {"type": "object", "properties": {"nonce": {"type": "string", "enum": [nonce]},
              "sum": {"type": "integer", "enum": [number + 23]}}, "required": ["nonce", "sum"], "additionalProperties": False}
    binding_model = ctx.fixture["capabilities"]["structured_binding_model"]
    binding = next((row for row in [ctx.fixture["provider"], *ctx.fixture.get("genuine_bindings", [])]
                    if row["qualified_model"] == binding_model and row.get("kind") == "genuine"), None)
    if binding is None:
        raise Blocked("strict schema requires the actual supported genuine preserving adapter binding")
    cap = ctx.fixture["capabilities"]
    port = cap["genuine_proxy_port"]
    require(type(port) is int and 1024 <= port <= 65535
            and binding["config"]["base_url"].rstrip("/") == runtime_endpoint(ctx, "http://127.0.0.1:" + str(port) + "/v1"),
            "wire binding must use the exact frozen owned forwarding listener")
    root = Path(image["path"]).parent
    receipt = root / ("genuine-proxy-" + str(port) + ".json")
    if not receipt.exists():
        require(all(str(Path(value).resolve()) in ctx.fixture["_frozen_paths"]
                    for value in (cap["python_binary"], cap["tools_fixture_path"])), "genuine forwarder source is not frozen")
        process = ctx.owned_process([cap["python_binary"], cap["tools_fixture_path"], "genuine-proxy", str(root),
                                     str(port), cap["genuine_upstream"], cap["structured_proxy_budget_config"]])
        deadline = time.monotonic() + min(ctx.remaining(), 10)
        while not receipt.exists() and time.monotonic() < deadline:
            require(process.poll() is None, "genuine forwarding listener exited before binding")
            time.sleep(0.05)
    require(receipt.is_file(), "owned genuine forwarder did not publish its actual bind receipt")
    listener = json.loads(receipt.read_text())
    require(listener["upstream"] == cap["genuine_upstream"] and listener["port"] == port
            and any(process.pid == listener["pid"] and process.poll() is None for process in ctx.processes),
            "genuine forwarding receipt does not describe an actual invocation-owned process")
    configure_provider(ctx, binding)
    requested = {"type": "json_schema", "json_schema": {"name": "OwnedSchema_" + nonce, "strict": True, "schema": schema}}
    structured = complete(ctx, f'Add 23 to {number}. Explain the calculation in ordinary prose.',
                          response_format=requested, model=binding_model)
    require(answer_json(structured) == {"nonce": nonce, "sum": number + 23},
            "advertised structured format was dropped, ignored or violated")
    wire = observe(ctx, "wire", {"response_format": requested, "wire_model": binding["wire_model"]})
    require(wire.get("actual_genuine_wire"), "schema result lacks actual genuine provider format/cap wire evidence")
    # Nonce appears solely in response_format. Correct output is therefore a
    # discriminating format-forwarding witness, not a request to emit JSON.
    return Outcome(Predicate(True, "genuine advertised vision and strict-schema paths executed"),
                   Predicate(True, "actual random pixels and schema-only nonce determined exact useful outputs"),
                   {"vision_run": visual["run_id"], "structured_run": structured["run_id"], "image_sha256": image["sha256"]})


def routing(ctx):
    configure_provider(ctx)
    primary = ctx.fixture["provider"]
    alternate = next((row for row in ctx.fixture.get("genuine_bindings", [])
                      if row["config"]["id"] != primary["config"]["id"] and row.get("kind") == "genuine"), None)
    if alternate is None:
        raise Blocked("routing witness needs a second distinct genuine enabled provider")
    configure_provider(ctx, alternate)
    requirements = ctx.fixture["capabilities"]["routing_requirements"]
    require(requirements.get("needs_tools") is True and requirements.get("needs_vision") is True
            and requirements.get("needs_structured_output") is True, "routing capability predicate was weakened")
    route = ctx.request("POST", "/api/uar/route", {**requirements, "preferred_provider": primary["config"]["id"]})
    require(route.status == 200 and route.json().get("model") == primary["qualified_model"], "preferred eligible genuine model was not selected")
    disabled = {**primary["config"], "enabled": False}
    require(ctx.request("PUT", "/api/uar/providers/" + primary["config"]["id"], disabled).status == 200, "owned provider disable failed")
    try:
        fallback = ctx.request("POST", "/api/uar/route", {**requirements, "preferred_provider": primary["config"]["id"]})
        eligible = [row for row in ctx.fixture.get("genuine_bindings", [])
                    if row.get("kind") == "genuine" and row["config"]["id"] != primary["config"]["id"]]
        destination = next((row for row in eligible if row["qualified_model"] == fallback.json().get("model")), None) if fallback.status == 200 else None
        require(destination is not None,
                "disabled provider or undeclared destination remained eligible")
        number = secrets.randbelow(900) + 100
        result = complete(ctx, f'Return only JSON {{"sum":{number}+23}}.', model=destination["qualified_model"])
        require(answer_json(result) == {"sum": number + 23}, "actual routed alternate model was not useful")
        impossible = ctx.request("POST", "/api/uar/route", {**requirements, "min_context": 2 ** 60})
        require(impossible.status == 404, "unsatisfiable capability route fabricated an eligible destination")
    finally:
        require(ctx.request("PUT", "/api/uar/providers/" + primary["config"]["id"], primary["config"]).status == 200,
                "owned routing provider restoration failed")
    return Outcome(Predicate(True, "actual catalog capability/preference/disabled filtering observed"),
                   Predicate(True, "selected genuine fallback answered correctly; impossible requirement refused"))


def strategy_disagreement(ctx):
    configure_provider(ctx)
    codes = [secrets.token_hex(12), secrets.token_hex(12)]
    skills = []
    for index, (description, keywords) in enumerate((
        ("Calculate savings-account interest, financial credit and bank account fees.", ["bank"]),
        ("Safely moor a canoe alongside a river shore, stream bank and flowing water.", ["canoe", "stream", "shore"]))):
        name = "acceptance-disagreement-" + secrets.token_hex(8)
        response = ctx.request("POST", "/api/uar/skills/", {"name": name, "description": description,
            "triggers": {"keywords": keywords, "semantic": description}, "enabled": True,
            "prompt_overlay": 'Return only JSON {"selected":"' + codes[index] + '"}.'})
        require(response.status == 201, "discriminating skill creation failed")
        skills.append(name)
    original = ctx.request("GET", "/api/uar/skills/config")
    require(original.status == 200, "matching config unavailable")
    original = original.json()
    old_engine = setting(ctx, "skills.matching.engine", "service")
    query = "At the bank, where should a paddler tie up a small boat beside flowing water?"
    winners = []
    try:
        for method, expected in (("keyword", 0), ("embedding", 1)):
            config = {**original, "engine": "service", "algorithm": method, "threshold": 0.0,
                      "margin_threshold": 0.0, "top_k": 1}
            require(ctx.request("PUT", "/api/uar/skills/config", config).status == 200, "disagreement method rejected")
            result = prompt(ctx, query + " Use the selected skill and return its specified JSON.",
                run_policy={"version": 1, "skills": {"mode": "auto", "ids": skills, "denied_ids": []}})
            proof = observed(ctx, result)
            matching = proof["skill_matching"]
            require(matching["effective_method"] == "skill_service." + method, "matching method was substituted")
            accepted = [row["skill_id"] for row in matching["candidates"] if row.get("accepted")]
            require(accepted == [skills[expected]] and answer_json(result) == {"selected": codes[expected]},
                    "lexical/semantic discriminator did not change the scored winner and consumed private body")
            winners.append(accepted[0])
    finally:
        require(ctx.request("PUT", "/api/uar/skills/config", original).status == 200, "matching config restoration failed")
        setting(ctx, "skills.matching.engine", old_engine)
    require(winners[0] != winners[1], "lexical and semantic controls selected the same winner")
    return Outcome(Predicate(True, "two source-backed matching methods ran on an actual lexical/semantic disagreement"),
                   Predicate(True, "different ranked winners produced different private skill facts"), {"winners": winners})


def context_threshold_restart(ctx):
    configure_provider(ctx)
    runs = []
    for strategy in ({"type": "summarize", "threshold": 20, "summary_max_tokens": 256, "model": None},
                     {"type": "hierarchical", "short_term_turns": 20,
                      "mid_term_summary_tokens": 256, "long_term_facts_tokens": 128},
                     {"type": "sliding_window", "max_messages": 8},
                     {"type": "truncate_middle", "keep_first": 2, "keep_last": 6}, {"type": "none"}, {"type": "auto"}):
        code = secrets.token_hex(16)
        policy = {"version": 1, "skills": {"mode": "none", "ids": [], "denied_ids": []}, "context_strategy": strategy}
        first = prompt(ctx, 'Remember the private route code ' + code + '. Return only JSON {"stored":true}.', run_policy=policy)
        require(answer_json(first) == {"stored": True}, "below-threshold fact seed failed")
        recalled = prompt(ctx, 'Return only JSON {"code":"my private route code"}.', first["session_id"], run_policy=policy)
        require(answer_json(recalled) == {"code": code}, "below-threshold strategy lost the fact")
        receipt = observed(ctx, recalled)["context_reduction"]
        require(receipt["changed"] is False and receipt["source_sha256"] == receipt["prepared_sha256"]
                and not receipt.get("tier_calls"), "below-threshold context was summarized or rewritten")
        ctx.restart()
        cold = prompt(ctx, 'Return only JSON {"code":"my private route code"}.', first["session_id"], run_policy=policy)
        require(answer_json(cold) == {"code": code}, "configured context strategy failed genuine cold recall")
        runs.append({"type": strategy["type"], "run_id": cold["run_id"]})
    return Outcome(Predicate(True, "all six real context strategies observed below threshold and after true process restart"),
                   Predicate(True, "unchanged short history retained the planted private fact"), {"runs": runs})


def memory_consent(ctx):
    configure_provider(ctx)
    before = ctx.request("GET", "/api/memory/policy")
    require(before.status == 200 and before.json().get("host_capture_enabled") is True,
            "automatic capture fixture requires the actual opted-in host capability")
    before = before.json()
    enabled = ctx.request("PUT", "/api/memory/policy", {"enabled": True, "retention": {"kind": "days", "days": 7}})
    require(enabled.status == 200 and enabled.json()["enabled"] is True, "explicit owner consent did not persist")
    capture, excluded = secrets.token_hex(16), secrets.token_hex(16)
    try:
        saved = complete(ctx, 'Remember this private consent fixture fact ' + capture
                         + '. Return only JSON {"acknowledged":true}.', memory=True)
        require(answer_json(saved) == {"acknowledged": True}, "capture seed was not a useful genuine turn")
        deadline = time.monotonic() + min(ctx.remaining(), ctx.fixture["capabilities"]["case_seconds"])
        while time.monotonic() < deadline:
            records = ctx.request("GET", "/api/memory/records")
            require(records.status == 200, "actual captured memory read unavailable")
            matches = [row for row in records.json()["memories"] if capture in row.get("content", "")]
            if matches:
                break
            time.sleep(0.1)
        else:
            raise Blocked("actual owner-consented capture did not commit within allowance")
        disabled = ctx.request("PUT", "/api/memory/policy", {"enabled": False, "retention": None})
        require(disabled.status == 200 and disabled.json()["enabled"] is False
                and disabled.json()["consent_epoch"] > enabled.json()["consent_epoch"], "opt-out did not revoke capture epoch")
        ignored = complete(ctx, 'Private opt-out fact ' + excluded + '. Return only JSON {"acknowledged":true}.', memory=True)
        require(answer_json(ignored) == {"acknowledged": True}, "opt-out control inference failed")
        # A true stop/join/restart is the boundary; elapsed polling alone cannot
        # prove that an old capture queue will never publish later.
        ctx.restart()
        records = ctx.request("GET", "/api/memory/records")
        policy = ctx.request("GET", "/api/memory/policy")
        require(records.status == policy.status == 200 and policy.json()["enabled"] is False
                and not any(excluded in row.get("content", "") for row in records.json()["memories"]),
                "opted-out content was captured or consent was re-enabled after restart")
        require(any(capture in row.get("content", "") for row in records.json()["memories"]),
                "opt-out erased an already committed retained fact")
    finally:
        require(ctx.request("PUT", "/api/memory/policy", {"enabled": before["enabled"],
                    "retention": before["retention"]}).status == 200, "owned capture-policy restoration failed")
    return Outcome(Predicate(True, "actual capture, explicit seven-day owner consent and opt-out epoch exercised"),
                   Predicate(True, "committed fact survived; opted-out fact never appeared after joined restart"))


def activation_limit(ctx):
    configure_provider(ctx)
    old_mode = setting(ctx, "harness.skill_activation_mode", "catalog")
    codes, names = [secrets.token_hex(12), secrets.token_hex(12)], []
    try:
        for index in range(2):
            name = "acceptance-limit-" + secrets.token_hex(8)
            made = ctx.request("POST", "/api/uar/skills/", {"name": name, "description": "Bounded attachment count control " + name,
                "enabled": True, "triggers": {"keywords": [name], "semantic": name},
                "prompt_overlay": 'Return only JSON {"selected":"' + codes[index] + '"}.'})
            require(made.status == 201, "activation-limit skill creation failed")
            names.append(name)
        selected = agent(ctx, ["activate_skill"], "Follow the attached skill's exact private output format.")
        saved = ctx.request("GET", "/api/agents/" + selected)
        require(saved.status == 200, "count-limited agent unavailable")
        artifact = saved.json()
        artifact["policy"]["skills"]["max_active"] = 1
        artifact["extensions"]["uar.run_policy"] = {"version": 1,
            "skills": {"mode": "selected", "ids": names, "denied_ids": []}}
        require(ctx.request("PUT", "/api/agents/" + selected, artifact).status == 200,
                "actual max_active count limit did not persist")
        session = str(uuid.uuid4())
        started = ctx.request("POST", "/api/uar/runs", {"agent_id": selected, "session_id": session,
            "model": ctx.fixture["provider"]["qualified_model"], "input": "Use the admitted first skill and return its exact JSON.",
            "skill_attachments": names})
        require(started.status == 200, "count-limit journey was not admitted")
        failures = started.json().get("activation_failures", [])
        require(any(row.get("code") == "limit_reached" and row.get("skill_id") == names[1]
                    and row.get("limit") == 1 for row in failures), "second attachment lacks actual typed max_active refusal")
        run_id = started.json()["run_id"]
        deadline = time.monotonic() + min(ctx.remaining(), ctx.fixture["capabilities"]["case_seconds"])
        while time.monotonic() < deadline:
            current = ctx.request("GET", "/api/uar/runs/" + run_id)
            require(current.status == 200, "count-limit run disappeared")
            if current.json()["status"] in ("done", "error", "cancelled"):
                break
            time.sleep(0.1)
        require(current.json()["status"] == "done", "first admitted skill did not remain useful")
        verify_run(ctx, run_id, session, selected, ctx.fixture["provider"]["qualified_model"])
        proof = observe(ctx, "run_evidence", {"run_id": run_id})
        require(proof["activated_skills"] == [names[0]]
                and answer(proof["events"]["events"]) == {"selected": codes[0]}
                and codes[1] not in json.dumps(proof["events"]), "refused skill body affected model-visible output")
        require(ctx.request("DELETE", "/api/agents/" + selected).status == 204, "count-limit agent cleanup failed")
    finally:
        setting(ctx, "harness.skill_activation_mode", old_mode)
    return Outcome(Predicate(True, "two real attachments reached typed max_active=1 admission"),
                   Predicate(True, "first skill body remained useful; refused second body never activated"),
                   {"failure_kind": "count_limit", "token_pressure_contract": "bounded reattachment, not activation denial"})


def protected_multimodal(ctx):
    configure_provider(ctx)
    image = observe(ctx, "image", {"name": "protected-" + secrets.token_hex(8)})
    strategy = {"type": "sliding_window", "max_messages": 4}
    first = complete(ctx, 'Remember the colors left to right. Return only JSON {"colors":["left","middle","right"]}.',
                     image=image["data_url"], policy={"version": 1, "context_strategy": strategy})
    require(answer_json(first) == {"colors": image["colors"]}, "protected multimodal seed did not observe the actual image")
    name, marker = "protected-tool-" + secrets.token_hex(8), secrets.token_hex(12)
    owned = observe(ctx, "prepare_effect", {"name": name})
    selected = agent(ctx, ["file_read"], 'Read the exact requested file with file_read. '
                     'Return only JSON {"fact":"first line fact","result":first line number+23}.')
    run_id, status, _ = tool_turn(ctx, selected, first["session_id"],
        'Read the actual file at ' + json.dumps(owned["path"]) + '. Do not access another file.',
        ["file_read"], strategy=strategy)
    require(status["status"] == "done", "protected real tool turn failed")
    verify_run(ctx, run_id, first["session_id"], selected, ctx.fixture["provider"]["qualified_model"])
    seeded = observe(ctx, "run_evidence", {"run_id": run_id})
    paired(seeded, ["file_read"])
    require(answer(seeded["events"]["events"]) == {"fact": owned["fact"], "result": owned["number"] + 23},
            "protected tool result was not consumed in its actual next model round")
    policy = {"version": 1, "skills": {"mode": "none", "ids": [], "denied_ids": []}, "context_strategy": strategy}
    for index in range(4):
        neutral = complete(ctx, 'Neutral record ' + str(index) + '. Return only JSON {"logged":true}.',
                           session=first["session_id"], policy=policy)
        require(answer_json(neutral) == {"logged": True}, "multimodal pressure control inference failed")
    final = complete(ctx, 'Use the original image and prior real file result. Return only JSON '
                     '{"colors":["left","middle","right"],"fact":"file fact","result":file number+23}.',
                     session=first["session_id"], policy=policy)
    require(answer_json(final) == {"colors": image["colors"], "fact": owned["fact"], "result": owned["number"] + 23},
            "context pressure lost protected multimodal/tool facts")
    proof = observe(ctx, "run_evidence", {"run_id": final["run_id"]})
    reduction = proof.get("context_reduction") or {}
    require(reduction.get("changed") is True and reduction.get("protected_groups_preserved") is True
            and proof["image_messages"] > 0, "protected multimodal/tool preparation receipt is missing")
    require(observe(ctx, "effect", {"name": name, "marker": marker})["unchanged"] is True,
            "read-only protected context fixture mutated its source")
    require(ctx.request("DELETE", "/api/agents/" + selected).status == 204, "protected context agent cleanup failed")
    return Outcome(Predicate(True, "real image and genuine tool call/result reached context preparation under pressure"),
                   Predicate(True, "actual facts stayed useful with protected-group and canonical-image receipts"))


def cases():
    return [Case("U22.native-tool-approval", "U22", lambda ctx: tools(ctx, True), durable=True),
            Case("U22.mcp-tool-approval", "U22", lambda ctx: tools(ctx, False), durable=True),
            Case("U23.owner-isolation", "U23", isolation, durable=True),
            Case("U23.failure-honesty", "U23", failure_honesty, durable=True),
            Case("U23.unknown-tool-effect", "U23", unknown_effect, durable=True),
            Case("U25.memory-lifecycle", "U25", memory_lifecycle, durable=True),
            Case("U25.memory-consent", "U25", memory_consent, durable=True),
            Case("U25.formats-vision", "U25", formats_vision),
            Case("U25.routing-capabilities", "U25", routing),
            Case("U25.lexical-semantic-disagreement", "U25", strategy_disagreement),
            Case("U25.context-threshold-restart", "U25", context_threshold_restart, durable=True),
            Case("U22.skill-activation-count-limit", "U22", activation_limit),
            Case("U25.protected-multimodal-context", "U25", protected_multimodal, durable=True)]
