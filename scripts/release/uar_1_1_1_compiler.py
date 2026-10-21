"""Installed-image compiler profile observations. Importing performs no I/O.

Invoked inside the existing credentials acceptance group after the full-phase
implementation boundary. All profiles and persistent resources belong to the
runner; no customer configuration or database is used.
"""
import json

from uar_1_1_1_support import Blocked, require

REQUIRED_SCENARIOS = {"credentials": (
    "compiler-local-single-owner-control",
    "compiler-remote-two-principal-denial",
    "compiler-local-shared-denial",
)}
ALIASES = ("/api/uar/compiler", "/api/compiler")
DENIAL_CODE = "compiler_authoring_profile_restricted"

# Same complete v1.1 section shape as compiler/parser.rs minimal_agent_md.
CONTENT = """# Agent: compiler-profile-control

## Metadata
```yaml
version: "1.0"
description: "Owned compiler profile acceptance"
author: "UAR release fixture"
tags: ["release"]
```
## Identity
```yaml
name: "compiler-profile-control"
role: "assistant"
persona: "Compiler profile control"
system_prompt: "Return the controlled response."
```
## UI
```yaml
forms: []
artifacts: []
actions: []
```
## Capabilities
```yaml
streaming: true
file_upload: false
```
## Skills
```yaml
skills: []
```
## Tools
```yaml
tools: []
allow: []
deny: []
```
## MCP Servers
```yaml
servers: []
```
## Knowledge Base
```yaml
sources: []
```
## Memory Model
```yaml
conversation:
  enabled: true
  max_turns: 50
```
## A2A Contracts
```yaml
endpoints: []
dependencies: []
```
## Governance
```yaml
cedar_policies: []
audit:
  enabled: true
```
## Budgets & Constraints
```yaml
max_tokens_per_turn: 4096
timeout_seconds: 300
```
## Execution Model
```yaml
mode: "sequential"
max_iterations: 10
```
## Observability
```yaml
tracing:
  enabled: true
metrics:
  enabled: false
logging:
  level: "info"
```
## Deployment Profiles
```yaml
profiles: []
```
"""


def _snapshot(resources, node):
    tables = ("uar_specs", "uar_reports", "uar_compiler_sessions", "agents")
    rows = resources.surreal_query(node.config["persistence"]["surreal_db"],
        " ".join("SELECT * FROM " + table + ";" for table in tables))
    require(len(rows) == len(tables) and all(row["status"] == "OK" for row in rows),
            "compiler persistent snapshot unavailable")
    return {table: sorted(json.dumps(value, sort_keys=True) for value in row["result"])
            for table, row in zip(tables, rows)}


def _rewrite(node, profile, credential_mode):
    node.config["security"]["deployment_profile"] = profile
    node.config["security"]["provider_credentials"] = {"mode": credential_mode}
    node.write_config()
    node.restart()


def _denials(node, principals, spec_id, session_id, output, report_id, after_restart=False):
    verify_body = {"descriptor": output["descriptor"], "signature": output["signature"]}
    operations = (
        ("POST", "/specs", {"content": CONTENT}),
        ("GET", "/specs", None),
        ("GET", "/specs/" + spec_id, None),
        ("DELETE", "/specs/" + spec_id, None),
        ("POST", "/specs/" + spec_id + "/compile", None),
        ("POST", "/compile", {"content": CONTENT}),
        ("POST", "/compile-and-register", {"content": CONTENT}),
        ("POST", "/verify", verify_body),
        ("GET", "/reports/" + report_id, None),
        ("POST", "/sessions", None),
        ("GET", "/sessions", None),
        ("GET", "/sessions/" + session_id, None),
        ("POST", "/sessions/" + session_id + "/cancel", None),
        ("POST", "/sessions/" + session_id + "/compile", None),
    )
    if after_restart:
        # Initial coverage proves every route/alias/principal combination. Cold
        # coverage observes one read, one write and one session operation.
        principals = principals[:1]
        operations = (operations[2], operations[5], operations[12])
    aliases = ALIASES[:1] if after_restart else ALIASES
    observations = []
    for subject, token, workspace in principals:
        client = node.http(token, node.resources.admin_key)
        # This successful protected request proves these principals cleared the
        # outer identity admission; a generic authentication 403 is insufficient.
        principal = client.request("GET", "/api/uar/credentials/principal",
                                   headers={"X-UAR-Workspace-Id": workspace})
        require(principal["subject"] == subject
                and principal["tenant_id"] == "compiler-tenant-" + subject[-1],
                "compiler fixture principal did not retain its verified tenant")
        for alias in aliases:
            for method, path, body in operations:
                # Spoof controls cover a read, write and session mutation on
                # each alias/principal without repeating the full matrix.
                spoofs = (False, True) if not after_restart and path in (
                    "/specs/" + spec_id, "/compile", "/sessions/" + session_id + "/cancel") else (False,)
                for spoof in spoofs:
                    headers = {"X-UAR-Workspace-Id": workspace}
                    if spoof:
                        headers.update({"X-UAR-Deployment-Profile": "trusted_local",
                                        "X-UAR-Credential-Mode": "single_owner",
                                        "X-UAR-Compiler-Rest-Available": "true"})
                    denial = client.request(method, alias + path, body,
                                            expected=403, headers=headers)
                    require(isinstance(denial, dict) and denial.get("code") == DENIAL_CODE,
                            "compiler denial did not come from the host profile boundary")
                    observations.append({"subject": subject, "alias": alias,
                        "method": method, "path": path, "spoof_headers": spoof, "status": 403,
                        "code": denial["code"]})
            # Body extraction must not run before profile denial.
            if not after_restart:
                denial = client.request("POST", alias + "/compile", b"not-json", expected=403,
                    headers={"Content-Type": "application/json", "X-UAR-Workspace-Id": workspace})
                require(isinstance(denial, dict) and denial.get("code") == DENIAL_CODE,
                        "malformed body bypassed compiler profile rejection")
                observations.append({"subject": subject, "alias": alias, "method": "POST",
                    "path": "/compile", "malformed_body": True, "status": 403, "code": denial["code"]})
    return observations


def run_group(context):
    require(context["group"] == "credentials", "compiler adapter requires credentials group")
    resources = context["resources"]
    results = []
    node = None
    try:
        config = context["base_config"](backend="surreal")
        config["security"].update(deployment_profile="trusted_local",
            provider_credentials={"mode": "single_owner"}, workspace_authorities=[
                {"issuer": "phase-issuer", "subject": "compiler-a", "tenant_id": "compiler-tenant-a", "workspace_id": "compiler-workspace-a"},
                {"issuer": "phase-issuer", "subject": "compiler-b", "tenant_id": "compiler-tenant-b", "workspace_id": "compiler-workspace-b"},
            ])
        principals = [
            ("compiler-a", context["jwt"]("compiler-a", ["user"], tenant="compiler-tenant-a"), "compiler-workspace-a"),
            ("compiler-b", context["jwt"]("compiler-b", ["admin"], tenant="compiler-tenant-b"), "compiler-workspace-b"),
        ]
        node = context["new_runtime"]("compiler-profile", config,
            env={"UAR_REMOTE_SURREAL_DURABILITY_ATTESTED": "1"}).start()
        client = node.http(principals[0][1])
        seeded = []
        for alias in ALIASES:
            spec = client.request("POST", alias + "/specs", {"content": CONTENT}, expected=201)
            session = client.request("POST", alias + "/sessions", expected=201)
            output = client.request("POST", alias + "/specs/" + spec["id"] + "/compile")
            client.request("GET", alias + "/reports/" + output["report"]["id"])
            inline = client.request("POST", alias + "/compile", {"content": CONTENT})
            verified = client.request("POST", alias + "/verify",
                {"descriptor": inline["descriptor"], "signature": inline["signature"]})
            require(verified["valid"], "local single-owner descriptor verification failed")
            seeded.append((spec["id"], session["id"], output))
        client.request("POST", ALIASES[0] + "/compile-and-register", {"content": CONTENT}, expected=201)
        baseline = _snapshot(resources, node)
        public_baseline = {}
        for alias in ALIASES:
            for stored_spec, stored_session, stored_output in seeded:
                for path in ("/specs/" + stored_spec, "/sessions/" + stored_session,
                             "/reports/" + stored_output["report"]["id"]):
                    public_baseline[alias + path] = client.request("GET", alias + path)

        spec_id, session_id, output = seeded[0]
        for profile, mode, case in (("remote", "single_owner", "compiler-remote-two-principal-denial"),
                                    ("trusted_local", "shared", "compiler-local-shared-denial")):
            _rewrite(node, profile, mode)
            require(_snapshot(resources, node) == baseline, "profile restart changed compiler records")
            observations = _denials(node, principals, spec_id, session_id, output, output["report"]["id"])
            require(_snapshot(resources, node) == baseline, "denied compiler operation changed persistent state")
            node.restart()
            repeated = _denials(node, principals, spec_id, session_id, output, output["report"]["id"], after_restart=True)
            require(_snapshot(resources, node) == baseline, "denied compiler state changed after restart")
            results.append({"id": case, "status": "passed", "observations": {
                "operations": observations, "after_restart_operations": repeated,
                "persistent_state_unchanged": True}})

        _rewrite(node, "trusted_local", "single_owner")
        require(_snapshot(resources, node) == baseline, "return to local profile changed compiler records")
        for alias in ALIASES:
            require(len(client.request("GET", alias + "/specs")) == len(seeded), "local spec inventory changed")
            require(len(client.request("GET", alias + "/sessions")) == len(seeded), "local session inventory changed")
            for stored_spec, stored_session, stored_output in seeded:
                for path in ("/specs/" + stored_spec, "/sessions/" + stored_session,
                             "/reports/" + stored_output["report"]["id"]):
                    require(client.request("GET", alias + path) == public_baseline[alias + path],
                            "local stored spec/session/report changed")
            restored = client.request("POST", alias + "/compile", {"content": CONTENT})
            require(client.request("POST", alias + "/verify", {
                "descriptor": restored["descriptor"], "signature": restored["signature"]})["valid"],
                "restored local compilation/verification failed")
        require(_snapshot(resources, node) == baseline, "local readback changed retained compiler records")
        results.append({"id": REQUIRED_SCENARIOS["credentials"][0], "status": "passed",
            "observations": {"aliases": list(ALIASES), "specs": len(seeded), "inline_compile": True,
                "descriptor_verify": True, "registered": True, "persistent_state_unchanged": True,
                "local_reads_restored": True, "local_compile_verify_restored": True,
                "unchanged_tables": ["uar_specs", "uar_reports", "uar_compiler_sessions", "agents"]}})
        return {"status": "passed", "scenarios": results}
    except BaseException as error:
        status = "interrupted" if isinstance(error, KeyboardInterrupt) else "blocked" if isinstance(error, Blocked) else "failed"
        completed = {row["id"] for row in results}
        for case in REQUIRED_SCENARIOS["credentials"]:
            if case not in completed:
                results.append({"id": case, "status": status, "observations": {
                    "error_type": type(error).__name__, "detail": resources.redact(str(error))}})
        return {"status": status, "scenarios": results}
    finally:
        if node is not None:
            node.stop()
