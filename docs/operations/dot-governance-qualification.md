# Installed governance qualification

`scripts/dot/governance_journey.py` is source-only acceptance authoring. It runs
through the phase runner after the full implementation boundary; it has not run.

Each profile provides `governance.mandatory_host`, a normal `Host` configuration
with JWT required. The driver starts it, observes mandatory governance, tries to
disable governance through the real revisioned settings API, and requires both
rejection and unchanged effective enforcement. A shared deployment never has to
enable anonymous access to satisfy the local-only Off scenario.

The local profile additionally provides `governance.local` with `host`,
`provider_port`, `artifact`, `run_credentials`, `effect_trace`, and
`isolated_working_directory: true`. The host configuration deliberately selects
loopback-only ingress and `jwt_required: false`; its private settings API still
uses the configured operator key. Use a separate disposable data directory and
working directory outside this checkout. The driver creates only one new Cedar
file under that working directory, refuses to overwrite it, and removes it after
stopping its owned runtime. Preserve the installation's other required assets.

The artifact routes to the controlled provider and the actual stdio effect peer.
The driver observes anonymous native-system-info execution despite an automatic
Cedar denial while governance is Off, then observes the same Cedar denial after
turning governance On. It observes explicit Ask/Deny using native echo, Required
approval/denial and stale-ID refusal using the external effect peer, tool
selection remaining restrictive, and exactly one structured warning per process
despite repeated transitions. A joined restart checks saved Off and the new
process's own warning. Settings are restored before shutdown where the runtime
remains available; failed cleanup does not constitute successful acceptance.

The approved parent plan preserves explicit Ask/Deny and current host authority
under Off. The corresponding OpenSpec delta and warning now say that explicitly.
No tool admission semantics were changed by this reconciliation.

The uncomfortable distinction: every installed MCP descriptor is Required,
including a tool named `web_search` with `readOnlyHint`. Those annotations are
untrusted. This runner cannot demonstrate non-Required web search by silently
weakening that boundary. It uses the existing non-Required native system-info
tool to exercise the automatic bypass and reports that precise scope. A positive
claim about a future non-Required native web-search implementation would require
that real implementation and its own observed dispatch. A2A/AG-UI approval
projection and competing decisions have separate concrete protocol journeys.
