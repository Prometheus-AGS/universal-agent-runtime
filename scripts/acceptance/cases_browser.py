"""Installed browser U08/U24 cases; deferred until exact whole-phase approval."""
from __future__ import annotations

from pathlib import Path
import secrets
import uuid

from core import Blocked, Case, Context, Outcome, Predicate, require
from cases_agents import agent_path, custom_artifact, read_agent
from cases_prompt import configure_provider
from protocol_clients import run_program, verify_run


def profile(ctx: Context) -> dict:
    value = ctx.fixture.get("browser", {})
    frozen = {str(Path(item["path"]).resolve()): item["sha256"] for item in ctx.fixture["frozen_files"]}
    for key in ("executable", "playwright_entry"):
        candidate = value.get(key)
        if not isinstance(candidate, str) or not Path(candidate).is_absolute() or not Path(candidate).is_file():
            raise Blocked("installed browser profile lacks frozen " + key)
        if str(Path(candidate).resolve()) not in frozen:
            raise Blocked("installed browser profile input was not frozen: " + key)
    static = value.get("static_files")
    if not isinstance(static, list) or not static:
        raise Blocked("installed browser requires the actual frozen shipped static asset manifest")
    for row in static:
        if (not isinstance(row, dict) or not str(row.get("url", "")).startswith("/assets/")
                or frozen.get(str(Path(row.get("path", "")).resolve())) != row.get("sha256")):
            raise Blocked("installed static asset is absent from the exact frozen inventory")
    if not ctx.backend.get("clients", {}).get("browser"):
        raise Blocked("concrete installed browser client argv missing")
    if not ctx.backend.get("headers_env", {}).get("Authorization"):
        raise Blocked("browser profile needs explicit frozen owner authorization")
    if ctx.fixture.get("provider", {}).get("kind") != "genuine":
        raise Blocked("browser outcomes require genuine configured model inference")
    return {"browser_executable": value["executable"], "browser_sha256": frozen[str(Path(value["executable"]).resolve())],
            "static_assets": {row["url"]: row["sha256"] for row in static}}


def prepared_agent(ctx: Context, *, a2ui: bool = False, long_name: bool = False) -> dict:
    identifier = "acceptance-browser-" + uuid.uuid4().hex
    marker = secrets.token_hex(16)
    artifact = custom_artifact(ctx, identifier, marker)
    title = ("Orchard invoice specialist for unusually long international shipment descriptions " * 3
             if long_name else "Browser acceptance " + identifier)
    artifact["metadata"].update(title=title, description="Exclusive orchard invoice specialist for parcel arithmetic and prior parcel code.",
                                tags=["orchard", "invoice", "parcel", "acceptance"])
    provider, model = ctx.fixture["provider"]["qualified_model"].split("/", 1)
    artifact["policy"]["provider"]["default"] = {"provider": provider, "model": model}
    artifact["policy"]["skills"]["prefer"] = []
    if a2ui:
        artifact["policy"]["tools"]["allow"] = ["a2ui_render"]
        artifact["prompt"] = {"system": (
            'Use the real a2ui_render tool exactly once on the initial request to make a standard interactive parcel calculator. '
            'Use canonical A2UI v0.9 createSurface/updateComponents/updateDataModel messages and the '
            'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json catalog. '
            'Create a Column root with a Text heading "Parcel calculator", a TextField labeled "Parcel number" '
            'whose value binds /parcel, and a Button labeled "Calculate parcel" with action event name calculate_parcel. '
            'Initialize parcel to an empty string. Render through the tool, not a fenced code block or text approximation. '
            'When a2ui.user_action arrives, read the fresh parcel number from the submitted context/data model; '
            'do not render a new surface. Return only JSON with exactly marker and result, with your private marker "'
            + marker + '" and result equal to that fresh number plus 19.'), "instructions": []}
    else:
        artifact["policy"]["tools"]["allow"] = []
        artifact["prompt"] = {"system": (
            'Return only JSON with exactly code, marker, result. Your private marker is "' + marker
            + '". Put it in marker. Remember the user parcel code and quantity across dialogue turns. '
            'Set code to the supplied parcel code and result to quantity plus 19. On follow-up use the earlier values.'),
            "instructions": []}
    require(ctx.request("POST", "/api/agents", artifact).status == 201, "browser custom agent creation failed")
    saved = read_agent(ctx, identifier)
    return {"id": identifier, "title": saved["metadata"]["title"], "marker": marker}


def inputs(ctx: Context, mode: str, agent: dict, browser: dict) -> dict:
    return {**browser, "base_url": ctx.backend["base_url"], "mode": mode,
            "agent_id": agent["id"], "agent_title": agent["title"],
            "model": ctx.fixture["provider"]["qualified_model"],
            "provider_title": ctx.fixture["provider"]["config"]["display_name"], "headers_env": ctx.backend["headers_env"],
            "artifact_dir": str(ctx.evidence.root / ("browser-" + uuid.uuid4().hex)),
            "timeout_seconds": ctx.fixture["limits"]["request_seconds"],
            "max_bytes": ctx.fixture["limits"]["response_bytes"]}


def artifacts(receipt: dict) -> None:
    rows = receipt.get("artifacts", [])
    require(any(row.get("kind") == "video" and row.get("bytes", 0) > 0 for row in rows)
            and any(row.get("kind") == "screenshot" and row.get("bytes", 0) > 0 for row in rows)
            and all(row.get("sha256", "").startswith("sha256:") for row in rows),
            "installed UI execution lacks recorded video/screenshots with actual byte identities")
    require(receipt.get("static_assets") and all(row.get("matched") is True for row in receipt["static_assets"]),
            "browser did not load the frozen shipped UI assets")


def shipped_chat(ctx: Context, kind: str) -> Outcome:
    browser = profile(ctx)
    configure_provider(ctx)
    custom = None
    try:
        if kind == "default":
            default = read_agent(ctx, "default-agent")
            selected = {"id": "default-agent", "title": default["metadata"]["title"]}
        else:
            custom = prepared_agent(ctx, long_name=kind == "orchestrator")
            selected = custom if kind == "custom" else {
                "id": "orchestrator-agent", "title": read_agent(ctx, "orchestrator-agent")["metadata"]["title"]}
        code, quantity = secrets.token_hex(16), secrets.randbelow(800) + 100
        expected = {"code": code, "result": quantity + 19}
        if custom:
            expected["marker"] = custom["marker"]
        first = (f'For an orchard invoice, remember parcel code {code} and quantity {quantity}. '
                 'Apply your configured output instructions. ')
        second = 'For the same orchard invoice, recall my earlier parcel code and quantity and apply your configured output instructions.'
        if kind == "default":
            first += 'Return only JSON with keys code and result, result is quantity plus 19.'
            second += ' Return only JSON with keys code and result, result is earlier quantity plus 19.'
        request = {**inputs(ctx, "chat", selected, browser), "turns": [first, second], "expected": [expected, expected]}
        if kind == "orchestrator":
            request.update(selected_agent_id=custom["id"], selected_agent_title=custom["title"], reconnect=True)
        receipt = run_program(ctx, "browser", request)
        require(receipt.get("mode") == "chat" and receipt.get("saved_history_reload") is True,
                "shipped browser did not preserve rendered history through reload")
        turns = receipt.get("turns", [])
        require(len(turns) == 2 and turns[0].get("run_id") != turns[1].get("run_id")
                and turns[0].get("session_id") == turns[1].get("session_id") and turns[0].get("session_id"),
                "browser conversation lost actual native turn/session identities")
        for turn in turns:
            require(turn.get("rendered") is True and turn.get("protocol_terminal") == "success",
                    "browser rendered answer or terminal missing")
            verify_run(ctx, turn["run_id"], turn["session_id"], selected["id"], ctx.fixture["provider"]["qualified_model"])
        if kind == "orchestrator":
            selection = receipt.get("selection", {})
            require(selection.get("agent", {}).get("id") == custom["id"]
                    and selection.get("selectedThreadId") and selection.get("selectionMethod") == "intent"
                    and len(receipt.get("themes", [])) == 3,
                    "visible registered selection/themes not correlated to backend intent event")
            require(any(row.get("id") == custom["id"] and row.get("while_running") is True
                        and row.get("focus_inside") is False for row in receipt.get("selection_observations", [])),
                    "selection notice was absent during generation or stole keyboard focus")
            posts = [row for row in receipt["requests"] if row["path"] == "/api/chat/completion" and row["method"] == "POST"]
            require(len(posts) == 2, "UI network reconnect generated duplicate chat POST work")
        artifacts(receipt)
        ctx.record("shipped-browser", {"profile": kind, "receipt": receipt})
        return Outcome(Predicate(True, "installed shipped UI drove genuine correlated model turns and saved-history reload"),
                       Predicate(True, "rendered fresh parcel arithmetic/history and selected saved prompt effect"),
                       {"profile": kind, "receipt": receipt})
    finally:
        if custom:
            require(ctx.request("DELETE", agent_path(custom["id"])).status == 204, "owned browser agent cleanup failed")


def a2ui_action(ctx: Context) -> Outcome:
    browser = profile(ctx)
    configure_provider(ctx)
    agent = prepared_agent(ctx, a2ui=True)
    try:
        number = secrets.randbelow(800) + 100
        request = {**inputs(ctx, "a2ui", agent, browser),
            "turns": ["Create the interactive parcel calculator using your configured surface instructions."],
            "surface_heading": "Parcel calculator", "fresh_input": str(number),
            "expected": [{"marker": agent["marker"], "result": number + 19}]}
        receipt = run_program(ctx, "browser", request)
        action = receipt.get("action", {})
        require(receipt.get("mode") == "a2ui" and action.get("fresh_input_sent") is True
                and action.get("action") == "calculate_parcel"
                and action.get("source_run_id") != action.get("continuation_run_id"),
                "browser standard A2UI action lacks fresh-input/actual-continuation evidence")
        source = verify_run(ctx, action["source_run_id"], receipt["turns"][0]["session_id"], agent["id"],
                            ctx.fixture["provider"]["qualified_model"])
        continued = verify_run(ctx, action["continuation_run_id"], source["inspection"]["conversation_id"], agent["id"],
                               ctx.fixture["provider"]["qualified_model"])
        require(continued["inspection"]["agent_revision"] == source["inspection"]["agent_revision"],
                "standard A2UI continuation changed selected saved agent definition")
        artifacts(receipt)
        ctx.record("standard-a2ui-user-action", receipt)
        return Outcome(Predicate(True, "real native A2UI tool, rendered standard control and mounted user action continued inference"),
                       Predicate(True, "fresh field input changed genuine continued answer using saved private prompt"),
                       {"receipt": receipt})
    finally:
        require(ctx.request("DELETE", agent_path(agent["id"])).status == 204, "owned A2UI agent cleanup failed")


def cases() -> list[Case]:
    return [Case("U08.installed-standard-a2ui-action", "U08", a2ui_action, durable=True),
            *[Case("U24.shipped-ui." + kind, "U24", lambda ctx, kind=kind: shipped_chat(ctx, kind), durable=True)
              for kind in ("default", "custom", "orchestrator")]]
