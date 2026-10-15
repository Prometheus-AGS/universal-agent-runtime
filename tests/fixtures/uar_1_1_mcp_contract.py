"""Real-runtime MCP/AG-UI qualification, authored before the full-phase gate.

The release runner owns the digest-pinned runtime, its durable database, the
registered A/B agents, destination policy, and provider/peer processes. Configure
the provider with scripts/dot/provider.py and the destination with
dot19_mcp_peer.py. This helper uses the production administration HTTP boundary
for restriction changes and never supplies an executor or writes the database.

Import run_profile with the runner's pinned AG-UI schema validator, restart_runtime
callback, admin_request(method, path, body, expected_status) callback and original
non-secret destination registration payload. The runner owns process lifecycle.
No credentials or private input are written to the returned evidence.
"""

import ipaddress
import json
import time
from pathlib import Path
import urllib.error
import urllib.parse
import urllib.request
import uuid


def peer_rows(path):
    path = Path(path)
    if not path.exists():
        return []
    # The real peer fsyncs each complete JSON line. Never parse a partial write.
    return [json.loads(line) for line in path.read_text().splitlines(keepends=True)
            if line.endswith("\n")]


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def post(base_url, headers, path, body, expected=200, stream=False, method="POST"):
    # Pace negative probes below the shipped 10 requests/second admission limit.
    time.sleep(0.2)
    request_headers = dict(headers, **{"Content-Type": "application/json"})
    if stream:
        request_headers["Accept"] = "text/event-stream"
    request = urllib.request.Request(base_url.rstrip("/") + path,
                                     data=None if body is None else json.dumps(body).encode(),
                                     headers=request_headers, method=method)
    try:
        response = urllib.request.urlopen(request, timeout=120)
    except urllib.error.HTTPError as error:
        with error:
            response_body = error.read().decode()
        require(error.code == expected,
                "unexpected runtime rejection for " + path + ": expected " + str(expected) +
                ", received " + str(error.code) + ": " + response_body)
        return json.loads(response_body)
    with response:
        require(response.status == expected, "unexpected runtime response status")
        if not stream:
            return json.loads(response.read())
        require(response.headers.get_content_type() == "text/event-stream", "missing SSE content type")
        events = []
        data = []
        for raw in response:
            line = raw.decode().rstrip("\r\n")
            if line.startswith("data:"):
                data.append(line[5:].lstrip())
            elif not line and data:
                events.append(json.loads("\n".join(data)))
                data = []
        require(not data, "SSE ended inside an event")
        return events


def agui_path(agent):
    return "/api/uar/ag-ui/agents/" + urllib.parse.quote(agent, safe="")


def validate_segment(events, request, validate_event, allow_error=False):
    require(events, "runtime emitted no AG-UI events")
    for event in events:
        validate_event(event)
    require(events[0]["type"] == "RUN_STARTED", "segment did not start")
    require(not any(event["type"] in ("RUN_FINISHED", "RUN_ERROR") for event in events[:-1]),
            "event followed a terminal event")
    terminal = events[-1]
    require(terminal["type"] in (("RUN_FINISHED", "RUN_ERROR") if allow_error else ("RUN_FINISHED",)),
            "runtime ended with an error or incomplete stream: " + json.dumps(terminal))
    if terminal["type"] == "RUN_FINISHED":
        require(terminal["threadId"] == request["threadId"] and terminal["runId"] == request["runId"],
                "segment identity changed")
    return terminal


def initial_request(server, grant=None):
    return {"threadId": "mcp-contract-" + uuid.uuid4().hex, "runId": uuid.uuid4().hex,
            "protocolVersion": "1.0", "messages": [{"id": uuid.uuid4().hex, "role": "user",
            "content": json.dumps({"dot_action": "tool", "tool": server + "__elicit",
                                   "arguments": {"mode": "form"}})}],
            "tools": [], "context": [],
            "forwardedProps": {"uar": {} if grant is None else {"mcp_servers": [grant]}}}


def continuation(request, entries):
    return {"threadId": request["threadId"], "runId": uuid.uuid4().hex,
            "parentRunId": request["runId"], "protocolVersion": "1.0", "messages": [],
            "tools": [], "context": [], "resume": entries,
            "forwardedProps": request["forwardedProps"]}


def pending_contract(base_url, headers, agent, request, terminal, validate_event, peer_trace):
    """Rejected partial/foreign answers must not consume the durable interrupt."""
    before = peer_rows(peer_trace)
    rejected = []
    for entries in ([], [{"interruptId": "foreign-" + uuid.uuid4().hex,
                          "status": "cancelled"}]):
        invalid_coverage = continuation(request, entries)
        validate_event(invalid_coverage)
        error = post(base_url, headers, agui_path(agent), invalid_coverage, expected=409)
        require(error.get("code") == "resume_must_cover_all_open_interrupts", "wrong pending coverage denial")
        rejected.append(error["code"])
    path = agui_path(agent) + "/threads/" + urllib.parse.quote(request["threadId"], safe="")
    path += "/runs/" + urllib.parse.quote(request["runId"], safe="") + "/stream"
    replay = post(base_url, headers, path, None, stream=True, method="GET")
    replay_terminal = validate_segment(replay, request, validate_event)
    require(replay_terminal.get("outcome") == terminal.get("outcome"), "pending interrupt changed on reconnect")
    require(peer_rows(peer_trace) == before, "invalid resume or reconnect touched MCP peer")
    return {"rejected_codes": rejected, "replay": replay}


def require_no_tool_activity(peer_trace, before, stage):
    after = peer_rows(peer_trace)
    require(after[:len(before)] == before, "peer trace was replaced")
    require(not any(row["event"] in ("tool_call", "elicitation_request", "elicitation_response", "effect")
                    for row in after[len(before):]), "MCP tool activity " + stage)


def deny_tool_approval(base_url, headers, agent, grant, peer_trace, validate_event):
    before = peer_rows(peer_trace)
    request = initial_request(grant["name"], grant)
    validate_event(request)
    events = post(base_url, headers, agui_path(agent), request, stream=True)
    terminal = validate_segment(events, request, validate_event)
    outcome = terminal.get("outcome", {})
    require(outcome.get("type") == "interrupt", "denial scenario did not pause for approval")
    interrupts = outcome["interrupts"]
    require(len(interrupts) == 1 and interrupts[0]["reason"] == "tool_approval",
            "denial scenario did not expose exactly one tool approval")
    require_no_tool_activity(peer_trace, before, "before approval denial")
    pending = pending_contract(base_url, headers, agent, request, terminal, validate_event, peer_trace)
    resumed = continuation(request, [{"interruptId": interrupts[0]["id"],
                                      "status": "resolved", "payload": {"approved": False}}])
    validate_event(resumed)
    resumed_events = post(base_url, headers, agui_path(agent), resumed, stream=True)
    final = validate_segment(resumed_events, resumed, validate_event)
    # This is a live continuation: rejection becomes a tool error for the
    # controlled provider, which completes the run without invoking the tool.
    require(final.get("outcome", {}).get("type") == "success",
            "warm approval denial did not finish or repeated an interrupt")
    require_no_tool_activity(peer_trace, before, "after approval denial")
    path = agui_path(agent) + "/threads/" + urllib.parse.quote(resumed["threadId"], safe="")
    path += "/runs/" + urllib.parse.quote(resumed["runId"], safe="") + "/stream"
    replay = post(base_url, headers, path, None, stream=True, method="GET")
    replay_terminal = validate_segment(replay, resumed, validate_event)
    require(replay_terminal == final, "denied continuation changed its terminal state on reconnect")
    require_no_tool_activity(peer_trace, before, "after denied continuation reconnect")
    return {"approved": False, "segments": [events, resumed_events],
            "pending_interrupt_contract": pending, "terminal_replay": replay,
            "tool_calls_before_approval": 0, "tool_calls_after_denial": 0,
            "elicitation_requests": 0, "elicitation_responses": 0, "independent_effects": 0}


def restriction_transition(base_url, headers, agent_b, grant, destination, admin_request,
                           peer_trace, validate_event, ambient):
    unrestricted = json.loads(json.dumps(destination))
    unrestricted["grant_policy"].pop("allowed_agent_ids", None)
    path = "/api/uar/mcp/servers/" + urllib.parse.quote(grant["name"], safe="")
    admin_request("PUT", path, unrestricted, 200)
    try:
        request = initial_request(grant["name"], None if ambient else grant)
        validate_event(request)
        events = post(base_url, headers, agui_path(agent_b), request, stream=True)
        terminal = validate_segment(events, request, validate_event)
        outcome = terminal.get("outcome", {})
        require(outcome.get("type") == "interrupt", "capture did not reach approval before restriction")
        interrupts = outcome["interrupts"]
        require(len(interrupts) == 1 and interrupts[0]["reason"] == "tool_approval",
                "capture reached an unexpected interrupt")
        admin_request("PUT", path, destination, 200)
        before = peer_rows(peer_trace)
        resumed = continuation(request, [{"interruptId": interrupts[0]["id"],
                                         "status": "resolved", "payload": {"approved": True}}])
        validate_event(resumed)
        resumed_events = post(base_url, headers, agui_path(agent_b), resumed, stream=True)
        final = validate_segment(resumed_events, resumed, validate_event, allow_error=True)
        require(final.get("outcome", {}).get("type") != "interrupt", "revoked capture reached another tool interrupt")
        observed = peer_rows(peer_trace)[len(before):]
        require(not any(row["event"] in ("initialize", "tool_call", "elicitation_request", "effect")
                        for row in observed), "revoked capture initialized or invoked the destination")
        return {"capture": "ambient" if ambient else "unbound_grant", "before": events,
                "continuation": resumed_events, "peer_initializations_and_calls_after_restriction": 0}
    finally:
        admin_request("PUT", path, destination, 200)


def run_profile(base_url, headers, agent_a, agent_b, grant, peer_trace,
                shared_transports_enabled, validate_event, *, restart_runtime,
                admin_request, destination):
    """Exercise an externally owned production runtime; return secret-free evidence.

    The caller must retain the actual runtime configuration and artifact identity
    proving shared_transports_enabled. The label here is not a runtime probe.
    validate_event must raise if a frame violates the pinned full event schema.
    """
    require(type(shared_transports_enabled) is bool, "sharing mode must be explicit")
    require(agent_a != agent_b, "agent isolation requires two distinct registered agents")
    require("grant" in grant and "url" not in grant, "request must use registered destination grant")
    require(bool(grant["grant"].get("headers", {}).get("Authorization")),
            "qualification requires an observable request-time Authorization credential")
    require(destination["grant_policy"].get("allowed_agent_ids") == [agent_a], "destination must exclusively bind A")
    artifact = admin_request("GET", "/api/agents/" + urllib.parse.quote(agent_a, safe=""), None, 200)
    baseline = peer_rows(peer_trace)
    denials = []
    denial_cases = [
            ("same_owner_agent_b", {"agent_id": agent_b}, [grant]),
            ("inline_agent_a", {"artifact": artifact}, [grant]),
            ("direct_endpoint_alias", {"agent_id": agent_b},
             [{"name": grant["name"] + "-alias", "url": destination["url"]}]),
            ("direct_endpoint_fragment_alias", {"agent_id": agent_b},
             [{"name": grant["name"] + "-fragment-alias",
               "url": urllib.parse.urldefrag(destination["url"])[0] + "#agent-binding-bypass"}])]
    endpoint = urllib.parse.urlsplit(destination["url"])
    alternate_path = endpoint.path.rstrip("/") + "/uar-binding-alias"
    alternate_url = urllib.parse.urlunsplit((endpoint.scheme, endpoint.netloc, alternate_path,
                                            endpoint.query, endpoint.fragment))
    denial_cases.append(("direct_origin_alternate_path", {"agent_id": agent_b},
        [{"name": grant["name"] + "-alternate-path", "url": alternate_url}]))
    alias_query = endpoint.query + ("&" if endpoint.query else "") + "uar_binding_alias=1"
    for fragment in ("", "agent-binding-bypass"):
        label = "direct_endpoint_query_alias" + ("_fragment" if fragment else "")
        alias = urllib.parse.urlunsplit((endpoint.scheme, endpoint.netloc, endpoint.path, alias_query, fragment))
        denial_cases.append((label, {"agent_id": agent_b},
            [{"name": grant["name"] + "-" + label, "url": alias}]))
    try:
        loopback_peer = ipaddress.ip_address(endpoint.hostname).is_loopback
    except ValueError:
        loopback_peer = endpoint.hostname == "localhost"
    if loopback_peer:
        # A's successful tool calls below are the reachability control. These
        # variants are restriction-only probes and must never contact the peer.
        for index, host in enumerate(("localhost", "127.0.0.1", "[::1]")):
            authority = host + (":" + str(endpoint.port) if endpoint.port is not None else "")
            for query in (endpoint.query, alias_query):
                for fragment in ("", "agent-binding-bypass"):
                    alias = urllib.parse.urlunsplit((endpoint.scheme, authority, endpoint.path, query, fragment))
                    label = "loopback_host_alias_%d%s%s" % (index,
                        "_query" if query == alias_query else "", "_fragment" if fragment else "")
                    denial_cases.append((label, {"agent_id": agent_b},
                        [{"name": grant["name"] + "-" + label, "url": alias}]))
    for label, selection, servers in denial_cases:
        denial = post(base_url, headers, "/api/uar/runs", dict(selection,
            input="must not initialize the protected destination", mcp_servers=servers), expected=422)
        require("run_mcp_agent_forbidden" in json.dumps(denial),
                "wrong agent denial code for " + label + ": " + json.dumps(denial))
        require(peer_rows(peer_trace) == baseline, "unauthorized selection reached MCP peer")
        denials.append({"scenario": label, "status": 422, "code": "run_mcp_agent_forbidden", "peer_requests": 0})
    restart_runtime()
    require(peer_rows(peer_trace) == baseline, "restart initialized a restricted destination")
    denial = post(base_url, headers, "/api/uar/runs", {
        "agent_id": agent_b, "input": "restriction must survive restart", "mcp_servers": [grant]}, expected=422)
    require("run_mcp_agent_forbidden" in json.dumps(denial), "restart lost agent restriction")
    require(peer_rows(peer_trace) == baseline, "wrong agent reached peer after restart")
    approval_denial = deny_tool_approval(base_url, headers, agent_a, grant, peer_trace, validate_event)
    results = []
    for action in ("accept", "decline", "cancel"):
        before = peer_rows(peer_trace)
        thread = "mcp-contract-" + uuid.uuid4().hex
        wire_run = uuid.uuid4().hex
        request = {
            "threadId": thread, "runId": wire_run, "protocolVersion": "1.0",
            "messages": [{"id": uuid.uuid4().hex, "role": "user", "content": json.dumps({
                "dot_action": "tool", "tool": grant["name"] + "__elicit",
                "arguments": {"mode": "form"}})}],
            "tools": [], "context": [], "forwardedProps": {"uar": {"mcp_servers": [grant]}},
        }
        segments = []
        reasons = []
        pending_evidence = []
        for _ in range(4):
            validate_event(request)
            events = post(base_url, headers, "/api/uar/ag-ui/agents/" + urllib.parse.quote(agent_a, safe=""),
                          request, stream=True)
            require(events, "runtime emitted no AG-UI events")
            for event in events:
                validate_event(event)
            require(events[0]["type"] == "RUN_STARTED", "segment did not start")
            require(not any(event["type"] in ("RUN_FINISHED", "RUN_ERROR") for event in events[:-1]),
                    "event followed a terminal event")
            terminal = events[-1]
            require(terminal["type"] == "RUN_FINISHED", "runtime ended with an error or incomplete stream: " + json.dumps(terminal))
            require(terminal["threadId"] == thread and terminal["runId"] == request["runId"],
                    "segment identity changed")
            segments.append(events)
            outcome = terminal.get("outcome", {"type": "success"})
            if outcome["type"] == "success":
                break
            require(outcome["type"] == "interrupt", "unexpected terminal disposition")
            pending_evidence.append(pending_contract(base_url, headers, agent_a, request, terminal,
                                                     validate_event, peer_trace))
            entries = []
            for interrupt in outcome["interrupts"]:
                reason = interrupt["reason"]
                require(reason in ("tool_approval", "mcp_elicitation"), "unexpected interrupt reason")
                reasons.append(reason)
                if reason == "tool_approval":
                    require_no_tool_activity(peer_trace, before, "before affirmative approval")
                    payload = {"approved": True}
                else:
                    payload = {"action": action}
                    if action == "accept":
                        payload["content"] = {"value": "accepted"}
                entries.append({"interruptId": interrupt["id"],
                                "status": "cancelled" if reason == "mcp_elicitation" and action == "cancel" else "resolved",
                                "payload": payload})
            request = {"threadId": thread, "runId": uuid.uuid4().hex,
                       "parentRunId": request["runId"], "protocolVersion": "1.0", "messages": [],
                       "tools": [], "context": [], "resume": entries,
                       "forwardedProps": {"uar": {"mcp_servers": [grant]}}}
        else:
            raise AssertionError("interaction did not finish within its expected segments")
        require(reasons == ["tool_approval", "mcp_elicitation"], "missing or repeated host interrupts")
        after = peer_rows(peer_trace)
        require(after[:len(before)] == before, "peer trace was replaced")
        observed = after[len(before):]
        initializations = [row for row in observed if row["event"] == "initialize"]
        require(initializations, "no real initialization was observed")
        require(all(row.get("credential_matches") is True for row in initializations),
                "initialize did not receive the exact request credential")
        expected = {"form": {"schemaValidation": True}, "url": {}}
        require(all(row.get("session_id") for row in initializations),
                "initialization omitted its HTTP session identity")
        initialized_sessions = {row["session_id"]: row for row in initializations}
        require(len(initialized_sessions) == len(initializations), "initializations reused an HTTP session identity")
        calls = [row for row in observed if row["event"] == "tool_call"]
        require(len(calls) == 1, "tool call was replayed")
        invoking_sessions = {row.get("session_id") for row in calls}
        require(invoking_sessions <= initialized_sessions.keys(),
                "tool invocation has no matching initialized HTTP session")
        require(all(initialized_sessions[session_id]["params"].get("capabilities", {}).get("elicitation") == expected
                    for session_id in invoking_sessions), "invoking client omitted the pinned elicitation capability")
        discovery_sessions = set(initialized_sessions) - invoking_sessions
        require(all(initialized_sessions[session_id]["params"].get("capabilities", {}) == {}
                    for session_id in discovery_sessions), "unexpected capability on discovery-only session")
        require(all(any(row["event"] == "tools_list" and row.get("session_id") == session_id
                        for row in observed) for session_id in discovery_sessions),
                "non-invoking initialization has no catalog discovery evidence")
        answers = [row for row in observed if row["event"] == "elicitation_response"]
        expected_answer = {"action": action}
        if action == "accept":
            expected_answer["content"] = {"value": "accepted"}
        require(len(answers) == 1 and answers[0]["message"].get("result") == expected_answer,
                "peer did not receive the exact decision once")
        require(answers[0].get("session_id") in invoking_sessions,
                "elicitation decision arrived on a different HTTP session")
        require(all(row.get("credential_matches") is True for row in observed if row["event"] == "tool_call"),
                "tool invocation did not receive the exact request credential")
        require(not any(row["event"] == "effect" for row in observed), "elicitation performed an independent effect")
        results.append({"action": action, "segments": segments, "initialize": initializations,
                        "invocation_sessions": [{"session_id": row["session_id"],
                            "request_id": row["request_id"],
                            "capabilities": initialized_sessions[row["session_id"]]["params"]["capabilities"]}
                            for row in calls],
                        "discovery_only_sessions": sorted(discovery_sessions),
                        "pending_interrupt_contract": pending_evidence,
                        "tool_calls_before_approval": 0,
                        "request_credential_observed": True,
                        "peer_response": answers, "tool_calls": 1, "independent_effects": 0})
    transitions = [restriction_transition(base_url, headers, agent_b, grant, destination, admin_request,
                                          peer_trace, validate_event, ambient=False)]
    if shared_transports_enabled:
        transitions.append(restriction_transition(base_url, headers, agent_b, grant, destination, admin_request,
                                                  peer_trace, validate_event, ambient=True))
    result = {"shared_transports_enabled": shared_transports_enabled,
            "sharing_mode_evidence": "caller must retain actual runtime configuration",
            "wrong_agent": {"status": 422, "code": "run_mcp_agent_forbidden", "peer_requests": 0},
            "selection_denials": denials, "restriction_survives_restart": True,
            "tool_approval_denial": approval_denial,
            "loopback_aliases_exercised": loopback_peer,
            "restriction_transitions": transitions, "actions": results}
    encoded = json.dumps(result)
    for value in grant["grant"].get("headers", {}).values():
        require(not value or value not in encoded, "outbound credential leaked into evidence")
        if value.startswith("Bearer "):
            require(value[7:] not in encoded, "outbound bearer leaked into evidence")
    return result
