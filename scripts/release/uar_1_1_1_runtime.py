"""Phase-specific installed-runtime integration. Importing performs no I/O.

Every passed row follows assertions over actual public admission, provider wire
traffic and (where required) durable host records. Source supplements are
reported separately by the lead and never credited to a published image.
"""
import hashlib
import importlib.util
import json
import re
import sys
import threading
import time
import uuid
from pathlib import Path

TEXT = "runtime-control:λ雪😀"
MODEL = "claude-haiku-4-5-20251001"
REQUIRED_SCENARIOS = {
    "binding": ("binding-qualified-durable-control", "binding-unattested-refusal",
                "binding-16000-catalog-reservation-refusal", "binding-v1-messages-policy",
                "binding-shared-alias-and-credential-free", "binding-fallback-policy", "binding-timeout-inheritance", "binding-team-source-policy", "binding-summary-capture-policy", "binding-invalid-budget-refusal"),
    "streams": ("streams-native-partition-matrix", "streams-sse-fields-fixed-partitions", "streams-eof-matrix", "streams-invalid-matrix",
                "streams-tool-terminal-control", "streams-cancel-cleanup", "streams-graph-inherited-control"),
    "mcp-outcomes": ("mcp-application-error", "mcp-explicit-success-and-omitted-status",
                     "mcp-protocol-and-timeout", "mcp-uncertain-effect-preserved"),
    "combined": ("combined-failed-tool-incomplete-stream", "combined-valid-control"),
}


def _artifact(name, allowed_tools=None, budgets=None, provider="anthropic", model=MODEL):
    return {"version":"1.0.0", "kind":"agent", "id":name,
        "metadata":{"title":name,"description":"UAR 1.1.1 runtime acceptance","tags":["release"]},
        "runtime":{"entry":"agent","protocols":{}},
        "policy":{"provider":{"default":{"provider":provider,"model":model},"fallbacks":[]},
                  "tools":{"allow":list(allowed_tools or []),"deny":[],"max_concurrent":1,"execution_mode":"direct"},
                  "skills":{"prefer":[],"max_active":1}},
        "schemas":{"inputs":None,"outputs":None,"state":None},
        "prompt":{"system":"Return the controlled provider response.","instructions":[]},
        "memory":{"conversation":{"enabled":False},"kb":{"enabled":False,"knowledge_bases":[],"citation_required":False}},
        "tools":{"bundles":[]},"ui":{"forms":{"enabled":False},"artifacts":{"enabled":False,"preferred_types":[]}},
        "extensions":{"budgets":budgets} if budgets is not None else {}}


def _mode(path, mode="complete", **settings):
    pending = path.with_suffix(".pending")
    pending.write_text(json.dumps(dict(mode=mode, **settings)))
    pending.replace(path)


def _rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


def _start_provider(context):
    root = Path(context["resource_root"])
    port_file = root / context["resources"].unique_name("provider-port")
    trace = root / context["resources"].unique_name("provider-trace")
    mode = root / context["resources"].unique_name("provider-mode")
    _mode(mode)
    context["resources"].start_process([sys.executable, str(Path(__file__).resolve().parents[2] / "tests/fixtures/uar_1_1_1_provider.py"),
        "--port-file", str(port_file), "--trace", str(trace), "--mode-file", str(mode)],
        cwd=Path(__file__).resolve().parents[2], label="uar111-provider")
    context["wait_for"]("provider ready", lambda: port_file.exists() and port_file.read_text().strip(), timeout=30)
    return mode, trace, context["resources"].relay(int(port_file.read_text()))


def _config(context, relay, backend="local", provider="anthropic", model=MODEL):
    config = context["base_config"](backend=backend)
    config["llm"] = {"model":provider + "/" + model,"base_url":relay + "/v1","api_key":"runtime-fixture-key", "protocol":"chat","timeout_secs":15,"max_retries":0}
    config["providers"] = [{"id":provider,"base_url":relay + "/v1","api_key":"runtime-fixture-key","protocol":"chat","default_model":model,"enabled":True,
        "models":[{"id":model,"enabled":True,"supports_tools":True,"supports_streaming":True}]}]
    return config


def _runtime(context, relay, backend="local", config=None, env=None, mounts=None, verify=True):
    config = config or _config(context, relay, backend)
    runtime = context["new_runtime"]("runtime", config, env=dict({"ANTHROPIC_NATIVE_DRIVER":"true"}, **(env or {})), mounts=mounts).start()
    if verify:
        # Explicit admin verification is setup, never evidence of run-source authorization.
        admin = runtime.http(context["jwt"]("phase-host", ["admin"], host=True), context["admin_key"])
        for provider in config["providers"]:
            result = admin.request("POST", "/api/uar/providers/" + provider["id"] + "/test", {"model":provider["default_model"]}, expected=200)
            context["require"](result["ok"] and result["received_text"], "provider model verification failed")
    return runtime


def _client(runtime, context):
    return runtime.http(context["jwt"]("runtime-owner", ["user"]))


def _admit(runtime, context, mode, allowed_tools=None, budgets=None, artifact=None):
    client = _client(runtime, context)
    body = {"artifact":artifact or _artifact("uar111-" + uuid.uuid4().hex, allowed_tools, budgets),"input":"controlled " + mode}
    return client, client.request("POST", "/api/uar/runs", body, expected=200)


def _finish(client, admitted, context):
    run_id = admitted["run_id"]
    stream = client.stream(admitted["stream_url"], timeout=240, total_timeout=240)
    status = context["wait_for"]("run settled", lambda: _terminal(client, run_id), timeout=30)
    events = client.request("GET", "/api/uar/runs/" + run_id + "/events", expected=200)
    # The AG-UI endpoint defaults an omitted cursor to the last event, then
    # waits for future broadcast traffic even after a terminal run. Replay from
    # zero so the terminal event remains in the finite response.
    spec = client.stream(admitted["stream_url"] + "?stream_mode=agui_spec&last_event_id=0", timeout=240, total_timeout=240)
    return {"run_id":run_id,"stream":stream,"agui":spec,"status":status["status"],"events":events["events"]}


def _finish_after_approval(client, admitted, context):
    run_id = admitted["run_id"]
    observed = {}

    def observe():
        try:
            observed["stream"] = client.stream(admitted["stream_url"], timeout=240, total_timeout=240)
        except BaseException as error:
            observed["error"] = error

    observer = threading.Thread(target=observe)
    observer.start()
    pending = context["wait_for"]("graph tool approval", lambda: client.request(
        "GET", "/api/uar/runs/" + run_id + "/tool-approval/pending", expected=200)["pending"], timeout=30)
    approval_id = pending.get("approval_id") or pending.get("approvalId")
    context["require"](approval_id, "graph approval did not expose its approval id")
    resolved = client.request("POST", "/api/uar/runs/" + run_id + "/tool-approval",
        {"approved":True,"approval_id":approval_id}, expected=200)
    context["require"](resolved["resolved"] and resolved["decision"] == "allow", "graph approval was not resolved")
    observer.join()
    if "error" in observed:
        raise observed["error"]
    status = context["wait_for"]("run settled", lambda: _terminal(client, run_id), timeout=30)
    events = client.request("GET", "/api/uar/runs/" + run_id + "/events", expected=200)
    spec = client.stream(admitted["stream_url"] + "?stream_mode=agui_spec&last_event_id=0", timeout=240, total_timeout=240)
    return {"run_id":run_id,"stream":observed["stream"],"agui":spec,"status":status["status"],"events":events["events"]}


def _terminal(client, run_id):
    row = client.request("GET", "/api/uar/runs/" + run_id, expected=200)
    return row if row["status"] in {"done", "error", "cancelled"} else None


def _run(runtime, context, mode, allowed_tools=None, budgets=None, artifact=None):
    client, admitted = _admit(runtime, context, mode, allowed_tools, budgets, artifact)
    return _finish(client, admitted, context)


def _events(row, name):
    return [event["data"] for event in row["events"] if event["eventName"] == name]


def _assert_terminal(context, row, success):
    require = context["require"]
    require(row["status"] == ("done" if success else "error"), "unexpected terminal status: " + str(row))
    errors = _events(row, "agui.error")
    require(not errors if success else bool(errors), "error projection disagrees with settlement")
    require("RUN_FINISHED" in row["agui"] if success else "RUN_ERROR" in row["agui"], "AG-UI terminal disagrees with settlement")
    if success:
        require(TEXT in "".join(event["delta"]["text"] for event in _events(row, "agui.message.delta")), "complete stream lost text/UTF-8")


def _scenario(identifier, observations):
    return {"id":identifier,"status":"passed","observations":observations}


def _save(context, name, value):
    path = Path(context["evidence_root"]) / (name + ".json")
    path.write_text(context["resources"].redact(json.dumps(value, ensure_ascii=False, indent=2)))
    path.chmod(0o600)
    return str(path)


def _mcp_runtime(context, relay):
    root = Path(context["resource_root"])
    peer_root = root / context["resources"].unique_name("mcp")
    peer_root.mkdir(mode=0o777)
    peer_root.chmod(0o777)  # disposable trace directory writable by the installed image's user
    mcp = root / context["resources"].unique_name("mcp-config")
    mcp.write_text(json.dumps({"mcpServers":{"outcomes":{"command":"/usr/bin/python3",
        "args":["/fixture/truthful_tool_outcome_mcp.py","/peer/effects.jsonl"],"env":{},"sandboxed":False,"preapproved_tools":["effect"]}}}))
    config = _config(context, relay, "surreal")
    config["server"]["mcp_shared_transports_enabled"] = True
    runtime = _runtime(context, relay, config=config, env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED":"1", "MCP_CONFIG_PATH":"/opt/uar/mcp.json"},
        mounts=[(Path(__file__).resolve().parents[2] / "tests/fixtures", "/fixture", "ro"),(peer_root,"/peer"),(mcp,"/opt/uar/mcp.json","ro")])
    catalog = runtime.http(context["jwt"]("phase-host", ["admin"], host=True), context["admin_key"]).request("GET", "/api/uar/mcp/servers")
    context["require"](any(server["name"] == "outcomes" and server["connected"] is True
        and "outcomes__effect" in server["tools"] for server in catalog["servers"]), "configured MCP outcome tool not connected at startup")
    return runtime, peer_root / "effects.jsonl"


def _receipts(context, runtime, run_id):
    database = runtime.config["persistence"]["surreal_db"]
    results = context["resources"].surreal_query(database, "SELECT VALUE data FROM canonical_tool_receipts WHERE run_id = " + json.dumps(run_id) + ";")
    context["require"](results and results[0]["status"] == "OK", "durable receipt query failed")
    return [json.loads(data) for data in results[0]["result"]]


def _receipt_assert(context, runtime, row, outcome):
    receipts = _receipts(context, runtime, row["run_id"])
    require = context["require"]
    require(len(receipts) == 1, "model-selected tool must persist exactly one canonical receipt")
    receipt = receipts[0]
    require(receipt["source"] == "mcp" and receipt["outcome"] == outcome, "canonical source/outcome mismatch")
    result = _events(row, "agui.tool_result")
    require(len(result) == 1 and result[0]["success"] == (outcome == "succeeded"), "ToolEnd success disagrees with typed receipt")
    if outcome in {"succeeded", "application_failed"}:
        represented = receipt["typed_value"] if outcome == "succeeded" else receipt["typed_value"]["result"]
        require(represented["structuredContent"]["nested"]["control"] == 42, "structured MCP content lost")
        require(represented["content"][0]["text"] == "fixture-response-control:λ", "MCP text lost")
        require(represented["content"][1]["type"] == "image" and represented["content"][2]["type"] == "resource", "MCP rich payload lost")
        require(represented["_meta"]["fixture"] == "metadata-control", "MCP metadata lost")
    public = _client(runtime, context).request("GET", "/api/uar/runs/" + row["run_id"] + "/tool-outcomes")
    require(len(public["receipts"]) == 1 and public["receipts"][0]["outcome"] == outcome, "public outcome disagrees with durable receipt")
    return receipts


def _stream_scenarios(context):
    mode, trace, relay = _start_provider(context)
    runtime, effects = _mcp_runtime(context, relay)
    observations = []
    try:
        # Load only the fixture's byte encoder; invocation is in the completed-phase gate.
        path = Path(__file__).resolve().parents[2] / "tests/fixtures/uar_1_1_1_provider.py"
        spec = importlib.util.spec_from_file_location("uar111_wire_fixture", path)
        peer = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(peer)
        matrix = []
        baseline_signature = None
        for newline in ("\n", "\r\n", "\r"):
            for bom in (False, True):
                wire = peer.anthropic_stream("complete", newline=newline, bom=bom)
                for split in [None, "bytewise", *range(1, len(wire))]:
                    _mode(mode, newline=newline, bom=bom, split=split)
                    before = trace.stat().st_size
                    row = _run(runtime, context, "complete")
                    _assert_terminal(context, row, True)
                    signature = [event["eventName"] for event in row["events"] if event["eventName"] not in {"agui.mcp.state"}]
                    if baseline_signature is None:
                        baseline_signature = signature
                    context["require"](signature == baseline_signature,"partition changed normalized event sequence")
                    with trace.open("rb") as recorded:
                        recorded.seek(before)
                        requests = [json.loads(line) for line in recorded if line.strip()]
                    context["require"](len(requests) == 1 and requests[0]["path"].endswith("/v1/messages"), "case did not select native Anthropic")
                    context["require"](requests[0]["sha256"] == hashlib.sha256(wire).hexdigest(), "fixture wire differed from partition matrix")
                    matrix.append({"newline":repr(newline),"bom":bom,"split":split,"run":row["run_id"],"status":row["status"],"wire":requests[0]["sha256"]})
        observations.append(_scenario("streams-native-partition-matrix", {"cases":len(matrix),"receipt":_save(context,"partitions",matrix)}))
        fields = []
        for newline in ("\n","\r\n","\r"):
            for fixed in (1,2,7,31):
                _mode(mode,"sse_fields",newline=newline,bom=True,fixed=fixed)
                row = _run(runtime,context,"sse-fields")
                _assert_terminal(context,row,True)
                fields.append({"run":row["run_id"],"newline":repr(newline),"fixed":fixed})
        observations.append(_scenario("streams-sse-fields-fixed-partitions",{"receipt":_save(context,"sse-fields",fields),"cases":len(fields)}))
        failures = {}
        for value in ("empty","metadata_only","text_only","partial_tool","complete_tool_eof","missing_terminal"):
            _mode(mode, value)
            before = len(_rows(effects))
            row = _run(runtime, context, value, ["outcomes__effect"])
            _assert_terminal(context, row, False)
            context["require"](len(_rows(effects)) == before and not _events(row,"agui.tool_result"), "unterminated stream executed pending tool")
            failures[value] = row
        observations.append(_scenario("streams-eof-matrix", {"receipt":_save(context,"eof",failures),"cases":len(failures)}))
        failures = {}
        for value in ("known_malformed","known_wrong_shape","overflow","partial_bom","invalid_utf8","after_terminal"):
            _mode(mode,value,split="bytewise" if value == "partial_bom" else None)
            row = _run(runtime,context,value)
            _assert_terminal(context,row,False)
            failures[value] = row
        _mode(mode,"unknown_event",split="bytewise")
        unknown = _run(runtime,context,"unknown_event")
        _assert_terminal(context,unknown,True)
        failures["unknown_event_positive"] = unknown
        observations.append(_scenario("streams-invalid-matrix", {"receipt":_save(context,"invalid",failures)}))
        _mode(mode,"tool",split="bytewise")
        before = len(_rows(effects))
        row = _run(runtime,context,"tool",["outcomes__effect"])
        _assert_terminal(context,row,True)
        _receipt_assert(context,runtime,row,"succeeded")
        context["require"](len(_rows(effects)) == before + 1,"terminal tool loop effect count mismatch")
        observations.append(_scenario("streams-tool-terminal-control", {"receipt":_save(context,"tool-loop",row)}))
        _mode(mode,"cancel")
        before = len(_rows(trace))
        client, admitted = _admit(runtime,context,"cancel")
        context["wait_for"]("cancel provider request",lambda:len(_rows(trace)) > before,timeout=30)
        client.request("POST","/api/uar/runs/" + admitted["run_id"] + "/cancel",{},expected=(200,204))
        cancelled = _finish(client,admitted,context)
        context["require"](cancelled["status"] == "cancelled" and bool(_events(cancelled,"agui.cancelled")), "cancellation lost precedence")
        _mode(mode)
        cleanup = _run(runtime,context,"after-cancel")
        _assert_terminal(context,cleanup,True)
        observations.append(_scenario("streams-cancel-cleanup", {"cancel":cancelled["run_id"],"next":cleanup["run_id"]}))
        _mode(mode,"graph")
        graph = _artifact("orchestrator-agent",["spawn_agent","wait_agents"])
        client, admitted = _admit(runtime,context,"graph",artifact=graph)
        row = _finish_after_approval(client, admitted, context)
        _assert_terminal(context,row,True)
        context["require"](bool(_events(row,"agui.subagent.started")),"graph never delegated through host child path")
        observations.append(_scenario("streams-graph-inherited-control", {"receipt":_save(context,"graph",row)}))
        return observations
    finally:
        runtime.stop()


def _mcp_scenarios(context):
    mode, trace, relay = _start_provider(context)
    observations = {}
    for value in ("application","success","omitted","protocol","timeout","unknown"):
        # Fresh MCP connection makes each independent invocation's start state explicit.
        _mode(mode)
        runtime, effects = _mcp_runtime(context,relay)
        try:
            _mode(mode,value)
            before = len(_rows(trace))
            row = _run(runtime,context,value,["outcomes__effect"])
            expected = "succeeded" if value in {"success","omitted"} else "application_failed" if value == "application" else "outcome_unknown"
            original = _receipt_assert(context,runtime,row,expected)
            context["require"](len(_rows(effects)) == 1,"MCP invocation replayed automatically")
            if expected == "outcome_unknown":
                _assert_terminal(context,row,False)
                context["require"](len(_rows(trace)) - before == 1,"uncertain effect triggered a model replay")
            else:
                _assert_terminal(context,row,True)
            _mode(mode)
            runtime.restart()
            context["require"](_receipts(context,runtime,row["run_id"]) == original,"restart changed original typed receipt")
            _mode(mode,"success")
            later = _run(runtime,context,"success",["outcomes__effect"])
            _assert_terminal(context,later,True)
            _receipt_assert(context,runtime,later,"succeeded")
            context["require"](len(_rows(effects)) == 2 and _receipts(context,runtime,row["run_id"]) == original,"later recovery rewrote or replayed original effect")
            observations[value] = {"original":row,"receipt":original,"later":later["run_id"],"effects":len(_rows(effects))}
        finally:
            runtime.stop()
    path = _save(context,"mcp-outcomes",observations)
    return [_scenario("mcp-application-error", {"receipt":path,"case":"application"}),
            _scenario("mcp-explicit-success-and-omitted-status", {"receipt":path,"cases":["success","omitted"]}),
            _scenario("mcp-protocol-and-timeout", {"receipt":path,"cases":["protocol","timeout"]}),
            _scenario("mcp-uncertain-effect-preserved", {"receipt":path,"case":"unknown","automatic_replays":0})]


def _binding_refusal(context, runtime, trace, category):
    before = len(_rows(trace))
    row = _run(runtime,context,"binding-refusal",budgets={"max_tokens_per_turn":32})
    _assert_terminal(context,row,False)
    errors = _events(row,"agui.error")
    error = next((event for event in errors if event.get("code") == "orchestrator_start_failed"), None)
    context["require"](error is not None and re.search(r"diagnostic reference [0-9a-f-]{36}",error["message"]), "safe binding code/reference absent")
    context["require"](category in runtime.logs(),"trusted safe failure category missing")
    context["require"](len(_rows(trace)) == before,"binding refusal contacted provider")
    raw_log_secret_absent = "runtime-fixture-key" not in runtime.logs(redact=False)
    context["require"]("runtime-fixture-key" not in row["stream"] and raw_log_secret_absent,"credential leaked through binding error")
    return {"run":row["run_id"],"category":category,"provider_calls":0,"public_error":error,"raw_log_secret_absent":raw_log_secret_absent}


def _binding_scenarios(context):
    mode, trace, relay = _start_provider(context)
    observations = []
    runtime = _runtime(context,relay,backend="surreal")
    try:
        # Persist through the real API so startup inspects a bounded saved agent.
        bounded = _artifact("uar111-saved-bounded-" + uuid.uuid4().hex, budgets={"max_tokens_per_turn":16000})
        saved = _client(runtime,context).request("POST","/api/agents",bounded,expected=201)
        context["require"](saved["extensions"]["budgets"] == bounded["extensions"]["budgets"],"agent save changed its budget")
        before = len(_rows(trace))
        prior_warnings = runtime.logs().count("budget_durable_storage_unavailable")
        runtime.restart()
        restored = _client(runtime,context).request("GET","/api/agents/"+bounded["id"],expected=200)
        context["require"](restored["extensions"]["budgets"] == {"max_tokens_per_turn":16000},"unattested restart changed persisted ceiling")
        logs = re.sub(r"\x1b\[[0-9;]*m", "", runtime.logs())
        warnings = [line for line in logs.splitlines() if "budget_durable_storage_unavailable" in line]
        warning = warnings[prior_warnings] if len(warnings) > prior_warnings else ""
        context["require"]("WARN" in warning and re.search(r'bounded_agent_count["\s:=]+[1-9][0-9]*',warning),"bounded persisted agent lacked safe startup warning")
        context["require"](len(_rows(trace)) == before,"startup warning contacted provider")
        result = _binding_refusal(context,runtime,trace,"remote_surreal_durability_not_attested")
        result["startup_warning"] = warning
        result["persisted_agent"] = saved["id"]
        observations.append(_scenario("binding-unattested-refusal",result))
    finally:
        runtime.stop()
    runtime = _runtime(context,relay,backend="surreal",env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED":"1"})
    try:
        row = _run(runtime,context,"qualified",budgets={"max_tokens_per_turn":1000000})
        _assert_terminal(context,row,True)
        database = runtime.config["persistence"]["surreal_db"]
        sql = "SELECT VALUE data FROM accounting_calls WHERE run_id = " + json.dumps(row["run_id"]) + ";"
        accounting = context["resources"].surreal_query(database, sql)
        context["require"](accounting[0]["status"] == "OK" and len(accounting[0]["result"]) == 1, "qualified run lacked durable accounting call")
        runtime.restart()
        restored_accounting = context["resources"].surreal_query(database, sql)
        context["require"](restored_accounting[0]["status"] == "OK" and len(restored_accounting[0]["result"]) == 1,
                           "restart lost durable accounting call")
        context["require"](restored_accounting[0]["result"] == accounting[0]["result"],
                           "restart changed durable accounting call/liability")
        later = _run(runtime,context,"qualified-restart",budgets={"max_tokens_per_turn":1000000})
        _assert_terminal(context,later,True)
        observations.append(_scenario("binding-qualified-durable-control", {"before":row["run_id"],"after_restart":later["run_id"],"accounting_receipt":_save(context,"durable-accounting",accounting)}))
    finally:
        runtime.stop()
    observations.append(_scenario("binding-16000-catalog-reservation-refusal",_catalog_budget_scenario(context,mode,relay,trace)))
    observations.append(_scenario("binding-v1-messages-policy",_messages_policy_scenario(context,mode,relay,trace)))
    observations.append(_scenario("binding-shared-alias-and-credential-free",_source_policy_scenarios(context,mode,relay,trace)))
    observations.append(_scenario("binding-fallback-policy",_fallback_scenario(context,mode,relay,trace)))
    observations.append(_scenario("binding-timeout-inheritance",_timeout_scenario(context,mode,relay,trace)))
    observations.append(_scenario("binding-team-source-policy",_team_policy_scenario(context,mode,relay,trace)))
    observations.append(_scenario("binding-summary-capture-policy",_summary_policy_scenario(context,mode,relay,trace)))
    _mode(mode)
    runtime = _runtime(context,relay)
    try:
        before = len(_rows(trace))
        row = _run(runtime,context,"invalid-budget",budgets={"max_cost_per_session_usd":-1})
        _assert_terminal(context,row,False)
        errors = _events(row,"agui.error")
        context["require"](any(error["code"] == "orchestrator_start_failed" and "budget_admission" in error["message"] and "diagnostic reference" in error["message"] for error in errors),"malformed budget lacked safe diagnostic")
        context["require"](len(_rows(trace)) == before and "budget_admission_failed" in runtime.logs(),"malformed budget had effects or wrong trusted category")
        observations.append(_scenario("binding-invalid-budget-refusal",{"run":row["run_id"],"provider_calls":0}))
    finally:
        runtime.stop()
    return observations


def _accounting_snapshot(context, runtime):
    database = runtime.config["persistence"]["surreal_db"]
    results = context["resources"].surreal_query(database,
        "SELECT VALUE data FROM accounting_scopes; SELECT VALUE data FROM accounting_calls;")
    context["require"](len(results) == 2 and all(row["status"] == "OK" for row in results),"accounting snapshot query failed")
    scopes, calls = ([json.loads(value) for value in row["result"]] for row in results)
    return {"scopes":sorted(scopes,key=lambda row: json.dumps(row["identity"],sort_keys=True)),
            "calls":sorted(calls,key=lambda row: json.dumps(row["admission"]["identity"],sort_keys=True))}


def _catalog_budget_scenario(context, mode, relay, trace):
    _mode(mode,"catalog_budget")
    config = _config(context,relay,"surreal",provider="openai",model="gpt-4o-mini")
    runtime = _runtime(context,relay,config=config,env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED":"1"})
    require = context["require"]
    try:
        client = _client(runtime,context)
        catalog = client.request("GET","/api/models",expected=200)["openai"]["models"]["gpt-4o-mini"]
        # No configured metadata overrides: these are the pinned global catalog
        # limits consumed by begin_call when there is no final-wire contract.
        require(catalog["limit"]["context"] == 128000 and catalog["limit"]["output"] == 16384,
                "fixture catalog bound changed; update the explicit reservation evidence")
        require(catalog["cost"]["input"] > 0 and catalog["cost"]["output"] > 0,"fixture model is not priced")
        bound = catalog["limit"]["context"] + catalog["limit"]["output"]
        require(bound == 144384 and bound > 16000,"catalog bound does not exercise the reported refusal")
        adequate = _artifact("uar111-budget-positive-"+uuid.uuid4().hex,budgets={"max_tokens_per_turn":1000000},provider="openai",model="gpt-4o-mini")
        positive_before = len(_rows(trace))
        positive = _run(runtime,context,"adequate-catalog-budget",artifact=adequate)
        _assert_terminal(context,positive,True)
        require(len(_rows(trace)) == positive_before+1,"adequate ceiling did not perform exactly one provider call")
        before = _accounting_snapshot(context,runtime)
        positive_calls = [row for row in before["calls"] if row["admission"]["identity"]["run_id"] == positive["run_id"]]
        require(len(positive_calls) == 1 and positive_calls[0]["admission"]["reserved_tokens"] == bound,
                "positive control did not reserve the actual full catalog bound")
        require(positive_calls[0]["state"] == "settled" and positive_calls[0]["observed_tokens"] == 2,
                "positive control did not durably settle actual terminal usage")
        bounded = _artifact("uar111-budget-16000-"+uuid.uuid4().hex,budgets={"max_tokens_per_turn":16000},provider="openai",model="gpt-4o-mini")
        saved = client.request("POST","/api/agents",bounded,expected=201)
        require(saved["extensions"]["budgets"] == {"max_tokens_per_turn":16000},"saved customer ceiling changed")
        contacts = len(_rows(trace))
        refused = _run(runtime,context,"catalog-budget-refusal",artifact=bounded)
        _assert_terminal(context,refused,False)
        errors = _events(refused,"agui.error")
        require(any(error.get("code") == "provider_error" and error.get("message") == "budget_admission_failed" for error in errors),
                "actual per-call refusal lacked safe public category")
        require(len(_rows(trace)) == contacts,"refused catalog reservation contacted provider")
        logs = re.sub(r"\x1b\[[0-9;]*m", "", runtime.logs())
        diagnostic = next((line for line in logs.splitlines() if "model_call_admission" in line and "budget_admission_failed" in line and refused["run_id"] in line), "")
        require("ERROR" in diagnostic and "accounting_budget_exhausted" in diagnostic and "catalog_context_and_output" in diagnostic,
                "per-call refusal lacked typed safe trusted diagnostic")
        require(re.search(r'diagnostic_reference["\s:=]+[0-9a-f-]{36}',diagnostic)
                and re.search(r'reserved_tokens["\s:=]+(?:Some\()?144384',diagnostic)
                and re.search(r'configured_turn_token_limit["\s:=]+(?:Some\()?16000',diagnostic),
                "per-call diagnostic omitted reference or exact reservation/ceiling")
        raw_log_secret_absent = "runtime-fixture-key" not in runtime.logs(redact=False)
        require(raw_log_secret_absent and "runtime-fixture-key" not in refused["stream"],"budget diagnostic leaked fixture key")
        after = _accounting_snapshot(context,runtime)
        require(after["calls"] == before["calls"],"refused reservation created a call or rewrote prior liability")
        require(all(row in after["scopes"] for row in before["scopes"]),"refused reservation changed prior scope balances/limits")
        run_scopes = [row for row in after["scopes"] if row["identity"]["scope"] == "run" and row["identity"]["scope_id"] == refused["run_id"]]
        require(len(run_scopes) == 1 and run_scopes[0]["limits"]["tokens"] == 16000,"refusal did not retain exact durable run ceiling")
        scope = run_scopes[0]
        require(all(scope[key] == 0 for key in ("settled_cost_usd","settled_tokens","reserved_cost_usd","reserved_tokens","unknown_calls","pending_calls")),
                "refused per-call reservation changed liability")
        runtime.restart()
        require(_accounting_snapshot(context,runtime) == after,"restart changed refused budget or prior liability")
        restored = _client(runtime,context).request("GET","/api/agents/"+bounded["id"],expected=200)
        require(restored["extensions"]["budgets"] == {"max_tokens_per_turn":16000},"restart removed or weakened saved ceiling")
        require(len(_rows(trace)) == contacts,"restart replayed the refused model request")
        return {"positive_run":positive["run_id"],"refused_run":refused["run_id"],"model":"openai/gpt-4o-mini",
                "catalog_reservation_tokens":bound,"max_tokens_per_turn":16000,"refused_provider_calls":0,"raw_log_secret_absent":raw_log_secret_absent,
                "receipt":_save(context,"catalog-budget-16000",{"catalog":catalog,"public_errors":errors,"trusted_diagnostic":diagnostic,
                    "before":before,"after":after,"restored_budgets":restored["extensions"]["budgets"]})}
    finally:
        runtime.stop()


def _grant(provider):
    return {"issuer":"phase-issuer","tenant_id":"phase-tenant","provider_id":provider,"source":"config"}


def _messages_policy_scenario(context, mode, relay, trace):
    results = {}
    for allowed in (False,True):
        _mode(mode)
        base = _config(context,relay)
        base["security"]["provider_credentials"] = {"mode":"shared","shared_fallbacks":[_grant("anthropic")] if allowed else []}
        runtime = _runtime(context,relay,config=base)
        try:
            before = len(_rows(trace))
            body = {"model":"anthropic/" + MODEL,"max_tokens":32,"stream":True,"messages":[{"role":"user","content":"phase fixture"}]}
            result = _client(runtime,context).request("POST","/v1/messages",body,expected=200 if allowed else 403)
            delta = len(_rows(trace)) - before
            context["require"](delta == int(allowed),"direct messages policy provider-call count mismatch")
            if allowed:
                context["require"](TEXT in result and "message_stop" in result,"allowed direct messages response incomplete")
            else:
                context["require"](result["error"]["type"] == "invalid_request_error"
                                   and result["error"]["message"] == "credential_source_denied",
                                   "wrong direct messages policy refusal")
            results[str(allowed)] = {"provider_calls":delta,"response_complete":allowed}
        finally:
            runtime.stop()
    return results


def _source_policy_scenarios(context,mode,relay,trace):
    results = {}
    for case, provider, key, grants in (
        ("alias-denied","fixture-alias","runtime-fixture-key",[_grant("openai")]),
        ("alias-allowed","fixture-alias","runtime-fixture-key",[_grant("fixture-alias")]),
        ("credential-free","fixture-alias","",[])):
        _mode(mode)
        config = _config(context,relay,provider=provider,model="gpt-4o-mini")
        config["llm"]["api_key"] = key
        config["providers"][0]["api_key"] = key
        config["security"]["provider_credentials"] = {"mode":"shared","shared_fallbacks":grants}
        runtime = _runtime(context,relay,config=config,env={"OPENAI_API_KEY":"fixture-ambient-must-not-be-used"},verify=bool(key))
        try:
            before = len(_rows(trace))
            row = _run(runtime,context,case,artifact=_artifact("uar111-" + uuid.uuid4().hex,provider=provider,model="gpt-4o-mini"))
            allowed = case != "alias-denied"
            _assert_terminal(context,row,allowed)
            requests = _rows(trace)[before:]
            context["require"](len(requests) == int(allowed),"alias grant matched wrong provider identity")
            if case == "credential-free":
                context["require"](all(request["credential_source"] == "empty" for request in requests),"credential-free capture picked up ambient key")
            results[case] = {"run":row["run_id"],"provider_calls":len(requests)}
        finally:
            runtime.stop()
    return results


def _fallback_scenario(context,mode,relay,trace):
    other_mode, other_trace, other_relay = _start_provider(context)
    results = {}
    for case, enabled, allowed in (("enabled-denied",True,False),("enabled-allowed",True,True),("disabled-denied-inert",False,False)):
        _mode(mode)
        _mode(other_mode)
        config = _config(context,relay,provider="openai",model="gpt-4o-mini")
        config["resilience"] = {"retries_enabled":False}
        config["providers"].append({"id":"fixture-fallback","base_url":other_relay + "/v1","api_key":"runtime-fixture-key","protocol":"chat","default_model":"gpt-4o-mini","models":[{"id":"gpt-4o-mini","enabled":True}]})
        config["failover"] = {"enabled":enabled,"fallback_models":[{"model":"fixture-fallback/gpt-4o-mini","base_url":other_relay + "/v1","api_key":"runtime-fixture-key"}]}
        config["security"]["provider_credentials"] = {"mode":"shared","shared_fallbacks":[_grant("openai")] + ([_grant("fixture-fallback")] if allowed else [])}
        runtime = _runtime(context,relay,config=config)
        try:
            _mode(mode,http_status=503 if enabled else 200)
            before, fallback_before = len(_rows(trace)),len(_rows(other_trace))
            row = _run(runtime,context,"fallback",artifact=_artifact("uar111-" + uuid.uuid4().hex,provider="openai",model="gpt-4o-mini"))
            _assert_terminal(context,row,allowed or not enabled)
            primary_requests, fallback_requests = _rows(trace)[before:],_rows(other_trace)[fallback_before:]
            context["require"](len(fallback_requests) == int(enabled and allowed),"fallback source authority mismatch")
            context["require"](len(primary_requests) == int(allowed or not enabled),"enabled denial or disabled inert fallback changed primary dispatch")
            if fallback_requests:
                context["require"](primary_requests[0]["time"] < fallback_requests[0]["time"],"fallback ran before the failing primary")
            results[case] = {"run":row["run_id"],"failover_enabled":enabled,"fallback_source_granted":allowed,
                "primary_calls":len(primary_requests),"fallback_calls":len(fallback_requests)}
        finally:
            runtime.stop()
    _mode(mode)
    return results


def _timeout_scenario(context,mode,relay,trace):
    _mode(mode)
    config = _config(context,relay,provider="openai",model="gpt-4o-mini")
    config["llm"].update(timeout_secs=2,max_retries=0)
    runtime = _runtime(context,relay,config=config)
    try:
        _mode(mode,delay=6)
        before,start = len(_rows(trace)),time.monotonic()
        row = _run(runtime,context,"timeout-inheritance",artifact=_artifact("uar111-" + uuid.uuid4().hex,provider="openai",model="gpt-4o-mini"))
        elapsed = time.monotonic()-start
        _assert_terminal(context,row,False)
        context["require"](elapsed < 6 and len(_rows(trace))-before == 1,"catalog route lost timeout/max_retries inheritance")
        return {"run":row["run_id"],"configured_timeout":2,"elapsed":elapsed,"provider_calls":1}
    finally:
        runtime.stop()
        _mode(mode)


def _combined_scenarios(context):
    mode,trace,relay = _start_provider(context)
    runtime,effects = _mcp_runtime(context,relay)
    try:
        _mode(mode,"combined")
        failed = _run(runtime,context,"combined",["outcomes__effect"])
        _assert_terminal(context,failed,False)
        original = _receipt_assert(context,runtime,failed,"application_failed")
        context["require"](len(_rows(effects)) == 1,"combined failure replayed MCP effect")
        _mode(mode,"success")
        later = _run(runtime,context,"success",["outcomes__effect"])
        _assert_terminal(context,later,True)
        _receipt_assert(context,runtime,later,"succeeded")
        _mode(mode)
        runtime.restart()
        context["require"](_receipts(context,runtime,failed["run_id"]) == original and len(_rows(effects)) == 2,"combined recovery rewrote original receipt or replayed effect")
        path = _save(context,"combined",{"failed":failed,"original_receipt":original,"later":later})
        return [_scenario("combined-failed-tool-incomplete-stream",{"receipt":path,"outcome":"application_failed","status":failed["status"]}),
                _scenario("combined-valid-control",{"receipt":path,"status":later["status"]})]
    finally:
        runtime.stop()


def run_group(context):
    runners = {"binding":_binding_scenarios,"streams":_stream_scenarios,"mcp-outcomes":_mcp_scenarios,"combined":_combined_scenarios}
    if context["group"] not in runners:
        raise RuntimeError("unsupported runtime group")
    return {"status":"passed","scenarios":runners[context["group"]](context)}


def _summary_policy_scenario(context,mode,relay,trace):
    results = {}
    for allowed in (False,True):
        _mode(mode)
        config = _config(context,relay)
        config["security"]["provider_credentials"] = {"mode":"shared","shared_fallbacks":[_grant("anthropic")] if allowed else []}
        runtime = _runtime(context,relay,config=config)
        try:
            artifact = _artifact("summary-capture-" + uuid.uuid4().hex)
            artifact["extensions"]["uar.run_policy"] = {"version":1,"context_strategy":{"type":"summarize","threshold":0,"summary_max_tokens":64,"model":"ungranted-helper/model"}}
            before = len(_rows(trace))
            row = _run(runtime,context,"summary-capture",artifact=artifact)
            _assert_terminal(context,row,allowed)
            requests = _rows(trace)[before:]
            context["require"](len(requests) == int(allowed),"summary strategy bypassed capture or constructed an extra helper")
            context["require"](all(request["request"]["model"] == MODEL for request in requests),"summary selection escaped captured primary")
            results[str(allowed)] = {"run":row["run_id"],"provider_calls":len(requests),"summary_inference":False}
        finally:
            runtime.stop()
    results["limit"] = "Production has no authorized prose marks or summary final-wire budget contract; this exercises capture and absence of ungranted helper I/O, not summary inference."
    return results


def _digest(value):
    return "sha256:" + hashlib.sha256(value).hexdigest()


def _seal(document):
    document.pop("contentDigest",None)
    document["contentDigest"] = _digest(json.dumps(document,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode())
    return document


def _ref(document):
    return {"id":document["id"],"version":document["version"],"digest":document["contentDigest"]}


def _team_policy_scenario(context,mode,relay,trace):
    results = {}
    fallback_mode, fallback_trace, fallback_relay = _start_provider(context)
    fixtures = Path(__file__).resolve().parents[2] / "tests/fixtures/collaboration"
    def template(name):
        return json.loads((fixtures / name).read_text())
    for case, allowed, with_fallback in (("primary-denied",False,False),("primary-allowed",True,False),
                                        ("distinct-fallback-denied",False,True),("distinct-fallback-allowed",True,True)):
        _mode(mode)
        _mode(fallback_mode,"catalog_budget")
        config = _config(context,relay,"surreal",provider="openai",model="gpt-4o-mini")
        grants = [_grant("openai")] if allowed or with_fallback else []
        if with_fallback:
            fallback_provider = _config(context,fallback_relay,provider="azure",model="gpt-4o-mini")["providers"][0]
            config["providers"].append(fallback_provider)
            if allowed:
                grants.append(_grant("azure"))
        config["security"]["provider_credentials"] = {"mode":"shared","shared_fallbacks":grants}
        runtime = _runtime(context,relay,config=config,env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED":"1"})
        try:
            client = runtime.http(context["jwt"]("runtime-owner",["admin"]),context["admin_key"])
            workspace = "phase-team-" + uuid.uuid4().hex
            headers = {"x-uar-workspace-id":workspace}
            def request(method,path,body=None,expected=200):
                return client.request(method,path,body,expected=expected,headers=headers)
            prefix = "/api/v1/collaboration"
            capabilities = request("GET",prefix + "/capabilities")
            context["require"](capabilities["collaboration"]["activation"]["teamExecution"],"team execution is unavailable")
            suffix = uuid.uuid4().hex
            agent = template("agent-definition.json")
            agent["id"] = "urn:uar:phase111:agent:" + suffix
            agent["skills"] = []
            agent["sourceDescriptor"]["skills"] = []
            agent["legacySections"]["skills"] = ""
            agent["input"] = agent["output"] = {}
            agent["context"] = {"mode":"none","artifacts":[],"history":"none","memoryScopes":[]}
            if with_fallback:
                agent["models"].append({"role":"fallback","capabilities":["text","tool-calling"],"preferredAliases":["distinct-fallback"]})
            agent["renameMapping"]["targetId"] = agent["id"]
            _seal(agent)
            team = template("team-definition.json")
            team["id"] = "urn:uar:phase111:team:" + suffix
            team["members"][0]["definition"] = _ref(agent)
            team["input"] = team["output"] = {}
            # Actual settings-only profile has no final-wire ceiling; reserve well
            # above the pinned gpt-4o-mini full request+output capacity.
            team["budget"]["maxTokens"] = 1000000
            _seal(team)
            workflow = template("workflow-definition.json")
            files, inventory = {},[]
            for name,document in (("agent-definition.json",agent),("team-definition.json",team),("workflow-definition.json",workflow)):
                source = json.dumps(document,ensure_ascii=False,indent=2)
                files[name] = source
                inventory.append({"path":name,"kind":document["kind"],"definition":_ref(document),"byteDigest":_digest(source.encode())})
            manifest = template("package-manifest.json")
            manifest["id"] = "urn:uar:phase111:package:" + suffix
            manifest["entrypoints"] = [_ref(team)]
            manifest["files"] = inventory
            manifest["lock"] = [{"requestedBy":team["id"],"reference":_ref(agent),"resolvedPath":"agent-definition.json"},
                                {"requestedBy":team["id"],"reference":_ref(workflow),"resolvedPath":"workflow-definition.json"}]
            _seal(manifest)
            request("POST",prefix + "/packages:install",{"commandId":"package-"+suffix,"manifest":json.dumps(manifest),"files":files},201)
            binding = template("deployment-binding.json")
            binding.update(id="urn:uar:phase111:binding:"+suffix,package=_ref(manifest),ownerId=capabilities["bindingOwnerId"],workspaceId=workspace,runtimeInstanceId=capabilities["instance"]["id"])
            binding["modelBindings"][0].update(modelId="gpt-4o-mini",profile={"id":"uar.openai-compatible-chat.settings-v1","revision":1},settingsRevision=1)
            if with_fallback:
                binding["modelBindings"].append({"requestedAlias":"distinct-fallback","providerId":"azure","modelId":"gpt-4o-mini",
                    "credentialRef":"protected-credential://phase111/fallback","profile":{"id":"uar.openai-compatible-chat.settings-v1","revision":1},"settingsRevision":1})
            binding["skillBindings"] = []
            binding["representationGrantRefs"] = []
            binding["effectiveBudget"]["maxTokens"] = 1000000
            _seal(binding)
            installed = request("POST",prefix+"/deployment-bindings",{"commandId":"binding-"+suffix,"expectedRevision":0,"binding":binding},201)
            context["require"](installed["preflight"]["activationSupported"],"team binding never reached executable state")
            instance = request("POST",prefix+"/team-instances",{"commandId":"team-"+suffix,"deploymentBindingId":binding["id"],"teamDefinition":_ref(team),"input":{},"memberSlots":[{"role":"reviewer","count":1}]},201)
            path = prefix+"/team-instances/"+instance["id"]
            instance = request("POST",path+"/tasks",{"commandId":"task-"+suffix,"taskId":"capture","expectedTeamRevision":instance["revision"],"title":"capture","role":"reviewer","input":{},"outputContract":{},"dependsOn":[]},201)
            def task_revision(current):
                return next(task["revision"] for task in current["tasks"] if task["id"] == "capture")
            member = instance["members"][0]["id"]
            instance = request("POST",path+"/tasks/capture/claim",{"commandId":"claim-"+suffix,"expectedTeamRevision":instance["revision"],"expectedTaskRevision":task_revision(instance),"memberId":member})
            _mode(mode,"catalog_budget",http_status=503 if with_fallback else 200)
            before, fallback_before = len(_rows(trace)),len(_rows(fallback_trace))
            admitted = request("POST",path+"/tasks/capture/admit",{"commandId":"admit-"+suffix,"expectedTeamRevision":instance["revision"],"expectedTaskRevision":task_revision(instance),"memberId":member,"reservation":{"tokens":1000000,"costMicrounits":1000000,"elapsedSeconds":120},"contextArtifactIds":[]})
            run_id = admitted["runId"]
            def registered():
                status,_ = client.request("GET","/api/uar/runs/"+run_id,expected=(200,404),return_status=True)
                return status == 200
            context["wait_for"]("team run registration",registered,timeout=60)
            row = _finish(client,{"run_id":run_id,"stream_url":"/api/uar/runs/"+run_id+"/stream"},context)
            _assert_terminal(context,row,allowed)
            primary_requests, fallback_requests = _rows(trace)[before:],_rows(fallback_trace)[fallback_before:]
            context["require"](len(primary_requests) == int(allowed),"team source policy did not govern actual primary capture")
            context["require"](len(fallback_requests) == int(allowed and with_fallback),"team distinct fallback source policy did not govern capture")
            if allowed and with_fallback:
                context["require"](primary_requests[0]["time"] < fallback_requests[0]["time"],"team fallback was not dispatched after primary failure")
                context["require"](all(request["path"].endswith("/chat/completions") and request["request"]["model"] == "gpt-4o-mini"
                    for request in primary_requests+fallback_requests),"team requests did not use admitted endpoint profiles")
            raw_log_secret_absent = "runtime-fixture-key" not in runtime.logs(redact=False)
            context["require"](raw_log_secret_absent,"team provider diagnostic leaked fixture key")
            if not allowed:
                context["require"](any(error["code"] == "TEAM_PROVIDER_REQUEST_REJECTED" and "diagnostic reference" in error["message"] for error in _events(row,"agui.error")),"team refusal lost established public contract")
            execution = request("GET",path+"/execution")
            context["require"](any(attempt["runId"] == run_id for attempt in execution["attempts"]),"team attempt not retained durably")
            results[case] = {"run":run_id,"primary_provider":"openai","fallback_provider":"azure" if with_fallback else None,
                "primary_calls":len(primary_requests),"fallback_calls":len(fallback_requests),"raw_log_secret_absent":raw_log_secret_absent,
                "receipt":_save(context,"team-"+case,{"run":row,"execution":execution})}
        finally:
            runtime.stop()
    return results
