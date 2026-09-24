# UAR core tool policy

This is task-scoped practice, not a permanent tool grant. Operator authorization and active harness controls remain authoritative. A role export does not install tools, enforce ownership, or grant shell, network, filesystem, MCP, credential or cross-repository access.

## Discovery and evidence

Start repository research with the configured Compass graph, then inspect exact source/manifests where coverage is incomplete. Read [repository-map.md](repository-map.md) for provenance, duplicated vendor trees and parse failures. Check freshness before relying on source lines. Graph relationships do not prove successful runtime execution.

The installed executable is `/usr/local/bin/compass`. Its local help verified these CLI forms:

```text
/usr/local/bin/compass query "QUESTION" --graph /Users/gqadonis/Projects/prometheus/universal-agent-runtime/compass-out/graph.json --scope source:src/uar --text-budget 2000
/usr/local/bin/compass serve --graph /Users/gqadonis/Projects/prometheus/universal-agent-runtime/compass-out/graph.json --transport stdio
```

Discovery may return fuzzy candidates, ambiguity, unknown coverage and bounded truncation. Do not promote a suggested candidate to an exact answer. Consult the installed query help for evidence output, CompassQL and source/package/node scopes.

The MCP server entry is `compass`. Session evidence reported an advertised tool list with 18 tools and verified names `graph_stats` and `get_callees`; this count is a snapshot, not a permanent contract. Discover current tools and exact input schemas with the server's `tools/list` before invoking native calls. Do not invent harness prefixes or assume identical exposure across platforms.

If native Compass is unavailable, use the verified local CLI for authorized read-only queries and report the limitation. Do not silently rebuild the graph or expand its Rust-only scope. Reference source is not proof that an installed binary exposes the same interface.

## Scope and authorization

- Default activation: lead, one specialist and verifier. Maximum four active agents including lead. Each implementer receives exact paths; cross-role consultation does not create a second writer.
- Use available native file/search tools or `rg` and Node.js for bounded inspection. Follow repository skill routing before dependency/library/framework work.
- Read schemas and identify effects before tool use. MCP calls that write files, run commands, change services or publish data need actual task authorization and native controls. Availability is not authorization.
- Cross-repository edits require an explicit scope naming the checkout, consumer contract and integration/pin step. Sharing a workspace does not transfer UAR ownership.
- Role prompts, accepted handoffs and exports transfer no native permissions or approvals. Do not bypass a denied action through another transport or process. Existing operator authorization need not be requested again for the same covered action.
- Repository human gates still apply to architecture, promotion, escalations and phase boundaries. Destructive actions, publication and deployment need applicable authorization; team metadata is not an approval receipt.

## Credentials and real boundaries

Never put credential values in prompts, manifests, state, handoffs, exports, logs or review artifacts. Read only necessary configuration fields; do not print whole MCP/environment files. Reference credential sources by name and let destinations resolve them under their own controls. Redact sensitive output before attaching evidence.

Actual UAR trust boundaries include tool execution, filesystem roots, process/network access, sandbox admission, authentication, credential binding and tenant/session memory scopes. Preserve the trusted-host mutation contract in `src/uar/runtime/thread/kernel.rs`. Agent reasoning is not a write grant. Report security changes and trace them to an observed defect or an actual boundary.

## Checks and coordination

Do not build for read-only research or documentation work. For production changes, finish and wire a meaningful capability before its required integration check. One writer builds in a shared target directory. The lead schedules final phase gates; the verifier records exact commands, results, artifact/commit context and unverified claims.

Existing OpenSpec/KBD and team state keep their respective authority. Optional memory tools do not replace inspectable evidence or repository fallback records. Do not add a service, port, dispatch bus or hidden coordination store through role configuration.
