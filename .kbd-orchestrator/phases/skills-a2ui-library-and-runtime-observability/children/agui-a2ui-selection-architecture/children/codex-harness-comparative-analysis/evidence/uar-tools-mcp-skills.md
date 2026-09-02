# Evidence — UAR tools, MCP, skills (commit dce44e78)

Produced by a read-only Explore subagent on 2026-09-02. File:line citations are
against commit `dce44e78`. This is raw evidence for `assessment.md`; the
assessment's judgments live there, not here.

## 1. Tool calling

### Key types

- `NativeSkill` trait — `src/uar/runtime/native_skill.rs:49-61`. Methods: `name`,
  `description`, `parameters_schema`, `async execute(args: Value)`. No
  effect/risk/approval field and no output-limit field on the trait.
- `NativeSkillRegistry` — `native_skill.rs:66-68`. `provider_tool_name()`
  (`:87-97`, sanitizes to `[A-Za-z0-9_-]`), `get()` (`:122-130`), `filtered()`
  (`:161-171`), `openai_tools_json()` (`:176-197`).
- `McpRegistry` — `src/mcp/registry.rs:227-235`: `services`, `shutting_down`,
  `server_config`, `tool_index: HashMap<namespaced_name, (server, raw_tool)>`,
  `tools: Vec<(String, Tool)>`, `native_tools`.
- `ToolGovernanceDecision` — `src/uar/governance/engine.rs:17-21`:
  `Allow | Deny | RequireApproval`.
- `ToolApprovalResult` — `src/llm/orchestrator.rs:78+`:
  `Approved | Rejected{reason} | GovernanceBypassed`.

### Tool list construction

- Built once, before the loop, at `orchestrator.rs:498-512`. MCP tools first
  (`:498`), native skills folded in at `:503-507`.
- Collision handling is native-wins: `:505` removes the MCP entry before `:506`
  pushes the native one. Exists only on this path.
- Sorted by name at `:508-512`.
- `tools.clone()` passed per iteration at `:601`; the list is frozen for the
  whole run. A skill activating mid-run cannot add tools.

### Argument parsing / validation

- Parallel path `orchestrator.rs:985-986` and sequential path `:1036-1037`:
  `serde_json::from_str(...).unwrap_or_else(|_| Value::Object(empty))`.
- No JSON Schema validation of tool arguments anywhere. `parameters_schema()` is
  only serialized into the prompt (`native_skill.rs:186`).
- `jsonschema` is used only for persistence entity schemas:
  `src/uar/persistence/providers/surreal.rs:1463`, `postgres.rs:1291`,
  `memory.rs:60`.

### Effect / approval classification

- `tool_requires_approval` — `src/uar/runtime/manager.rs:366-370`: substring
  scan for `delete|remove|write|drop|truncate|destroy`. Doc comment `:363-365`
  calls it a heuristic to be replaced by Cedar.
- `is_parallel_safe_tool` — `orchestrator.rs:138-148`: prefix allowlist
  `get_ list_ read_ search_ query_ fetch_ lookup_ status_ health_`.
- `is_code_execution_tool` — `orchestrator.rs:115-134`: exact names plus
  `starts_with("execute_")` / `ends_with("_repl")`.

### Approval flow

Gate closure `manager.rs:1712-1826`:
- `governance_bypass_decision` (`:372-377`) returns `GovernanceBypassed` when
  the runtime gate is disabled.
- `ToolApprovalPolicy::Deny` → `Rejected` + `ToolCallDenied` (`:1735-1749`).
- `heuristic_flag = policy==Ask || tool_requires_approval(name)` (`:1750-1751`).
- Cedar via `engine.tool_decision(agent_id, tool_name, heuristic_flag)`
  (`:1752-1755`); no engine → heuristic alone (`:1756-1757`).
- `RequireApproval` → `ToolCallApprovalRequired` (`:1780-1789`), oneshot,
  300s timeout auto-reject (`:1797`, `:1814-1822`).
- `engine.tool_decision` (`engine.rs:169-182`): Cedar deny wins; Cedar allow +
  heuristic flag → `RequireApproval`. `is_tool_allowed` fails closed on
  request-construction error (`engine.rs:156-164`).

### Parallel execution

`orchestrator.rs:974-1030`. Requires all of: `parallel_tool_calls == Some(true)`,
more than one call, no approval gate, no sandbox runner, every name passing
`is_parallel_safe_tool`. `.buffered(8)` (`:1008`) preserves order. Otherwise
sequential (`:1034`).

### Sandbox

`src/sandbox/mod.rs:1-6`: microsandbox/libkrun runner removed. Remaining:
`wasmtime_runner.rs`, `remote_runner.rs`. Routing `orchestrator.rs:1081-1094`.
Code extraction `:177-210` reads `code|command|script`; if absent, falls through
to unsandboxed execution (`:1179-1192`).

### Output limits

None. No truncation in `orchestrator.rs` or `registry.rs`. Whole results pushed
to history at `:1019-1023` and `:1272-1276`.

### Guardrails

`src/uar/guardrails.rs:62-87` `screen_input` only. One call site,
`src/server.rs:4702`, on user input. Tool arguments and results are never
screened.

## 2. MCP

### Configuration

`src/mcp/config.rs:5-31`: `McpConfig { mcp_servers }`;
`McpServerEntry::Stdio { command, args, env, sandboxed }` (`:13-21`) or
`RemoteHttp` (`:23`). Sources: global (`manager.rs:1556`), skill-contributed
(`manager.rs:1456-1469`), run-policy narrowing (`:1560-1582`).

### Startup — eager

`from_config` (`registry.rs:281-366`) connects every server eagerly, then
`list_tools`. Per-server failure isolation with `continue` (`:294-345`),
documented at `:368-373`. Timeouts: connect 20s (`:36`), list_tools 10s (`:43`),
tool call 30s (`:51`). Namespacing `:352`: `sanitize_tool_name("{server}__{tool}")`.

### Per-run registry construction

- `manager.rs:1448-1470`: for each matched skill with `mcp_config`,
  `McpRegistry::from_config(config).await` — fresh connections per skill, per run.
- `manager.rs:1556-1559`: `final_mcp = (*self.global_mcp).clone()` then `.merge()`.
- `manager.rs:1582-1583`: `.filtered(server_filter, tool_filter)`.

### Reconnect

`registry.rs:64-161`: slot carries `reconnect_entry`, `generation`,
`reconnects_in_flight`, `shutting_down`. `begin_reconnect` (`:115-123`) refuses
while shutting down; `install_reconnected_service` (`:126-138`) discards stale
results when `generation` moved.

### Metrics

`set_mcp_server_status` (`:300`, `:310`, `:315`); `mcp_tool_calls_total`
(`:957`), `mcp_tool_duration_ms` (`:960`), `record_tool_call` (`:964`).

### stdio sandboxing — not implemented

`Stdio.sandboxed` (`config.rs:21`) only stored and echoed: `registry.rs:1133`,
`:1226`, `:1234`, `api/mcp_admin.rs:47,66,148,157`. Spawn (`registry.rs:423-428`)
is a plain `Command`. Unchecked task: `microsandbox-mcp-isolation/tasks.md:116`.

## 3. Skills

### Discovery and precedence

`src/uar/runtime/skills/builtin_loader.rs:7-22`: primary root
`UAR_BUILTIN_SKILLS_DIR` (`:193-199`), extra roots `UAR_EXTRA_BUILTIN_SKILL_DIRS`
(`:208-222`). Collision policy precedence-wins (`:19-22`), allowlist
`scripts/skill-collision-allowlist.json` (`:224-242`). User skills from
`~/.agents/skills` (`storage/filesystem.rs:544-548`).

### Enablement scopes

`src/uar/domain/skills.rs:144-178` `enabled_for_with_agent_fallback`:
tombstoned → false; Conversation; Agent; legacy agent fallback; Global.
Filtered at `service.rs:608-618`. Effective-policy intersection is a separate
filter at `manager.rs:1434-1435`.

### Matching

Service path (`service.rs:625-666`) by `SkillMatchingAlgorithm`:
- `Keyword` → `keyword_match` (`:627`)
- `Embedding`/`LocalEmbedding` → `registry.find_matches` (`:631-636`)
- `Llm` → not implemented, falls back to keyword (`:638-643`)
- `Hybrid` → union, dedup, `truncate(top_k)` (`:644-665`)

`keyword_match` (`:693-731`): `_threshold: f32` never read. Scoring keyword +1.0,
title +0.5, description +0.3 (`:707-720`); `score > 0.0` survives (`:721`).

Legacy classifier path (`manager.rs:1358-1397`): `should_accept` → all scored
skills (`:1371-1379`); `out_of_scope` → empty (`:1380-1382`); else all scored
skills anyway (`:1383-1394`), log `:1387`. Classifier error → tag ∪ vector
(`:1398-1430`).

### Injection

`manager.rs:1448-1454`: full `prompt_overlay` (the SKILL.md body,
`storage/filesystem.rs:138`, `:583-586`) appended verbatim with a
`[SKILL: <title>]` header. No catalog mode, no token budget.

### max_active — never enforced

`SkillPolicy { prefer, max_active }` at `src/uar/domain/artifact.rs:96-104`,
default 3 (`defaults.rs:50`), populated at `compiler/to_artifact.rs:133`. No
reads under `src/uar/runtime/`, `src/uar/orchestrator/`, `src/llm/`.

### Explicit activation

No model-facing activation tool. Operator-side only: agent skill bindings
(`api/skills.rs:193`, `:651`, `:659`) and run-policy `skills.ids`.

### Telemetry

`record_skill_activation` per matched skill (`service.rs:686-688`).
`correlate_skill_activation_outcomes` (`manager.rs:140-151`, called `:2419`).

### WASM skill runtime — orphaned

`WasmSkillRuntime` (`skills/wasm_runtime.rs:94-296`) referenced nowhere outside
its module and one test. `wasm-component-skill-runtime/tasks.md:4,12,22`.

## 4. Claim verification

- (a) "Native and MCP registries are combined late." FALSE. Combined at
  `orchestrator.rs:498-512` before the loop (`:556`) and frozen. The real
  problem is the opposite: the tool set cannot change once a run starts.
- (b) "Every selected full overlay injected; keyword threshold ignored; LLM
  matching falls back." TRUE on all three (`manager.rs:1448-1454`,
  `service.rs:698`, `service.rs:638-643`, `manager.rs:1383-1394`).
- (c) "Skill MCP registries are recreated and merged during each run." TRUE
  (`manager.rs:1457`, `:1556-1559`).
- (d) "Name-prefix inference of tool effects." TRUE (`orchestrator.rs:138-148`,
  `manager.rs:366-370`, `orchestrator.rs:132-133`).
- (e) "jsonschema already pinned." PARTIAL. `Cargo.toml:418`
  `jsonschema = { version = "0.49.4", default-features = false }` (caret, not
  exact). Absent from `versions.toml`. Unused on the tool path.

## 5. Skill use after activation

- Overlay built once pre-run (`manager.rs:1448-1454`), pushed as one `System`
  message (`:1472-1477`), history appended (`:1478`). The loop
  (`orchestrator.rs:556`) only appends; the overlay persists for all iterations
  and is never re-evaluated against later turns.
- Skill-declared MCP servers are the only mechanism that works
  (`:1457` → `:1558` → `:1582`). Skills cannot contribute native tools
  (`native_skills/mod.rs:29-139` config-driven). WASM components not callable.
  Scripts: `service.rs:7` advertises sandboxed execution; no `execute_script`
  path exists. `preferred_tools` parsed/persisted (`storage/filesystem.rs:139`,
  `service.rs:527-528`, `api/skills.rs:113,129`) but never consulted.
- Matching runs once against the first user input (`:1334`, `:1360`).
- Attribution: `skill_servers` map (`manager.rs:1446`, `:1459-1462`) captured
  before merge because the merged registry loses provenance (`:1439-1445`).
  `correlate_skill_activation_outcomes` is server-granular. Overlay-only skills
  are excluded by design (`:1441-1445`, `:136-139`). Scope cuts documented at
  `service.rs:668-678` and `skill-activation-metrics/tasks.md:22-27`.
- A skill can only widen the tool set (via MCP). Narrowing is operator-side
  (`manager.rs:1560-1582`), applied after the merge. The approval gate never
  consults skill identity.
- Trimming: the System message carrying the overlay passes through
  `trim_with_summarization` (`:1516-1523`) and `context_manager.apply`
  (`:1535-1536`) with no pinning signal. `context_limit` defaults to 8_192 for
  uncataloged models (`:1534`). `SkillActivated` emitted after trimming
  (`:1541-1550`).
- Tests: `tests/skill_scoped_governance.rs:471` covers activation → overlay in
  request (`:551-561`); `:352` and `:159` cover scoping;
  `tests/wasm_skill_execution.rs:41,85` bypass `RunManager`;
  `manager.rs:2900-2940` unit-test correlation with hand-built maps. No test
  drives activation → skill MCP tool → model invocation → `used=true`.

## 6. Defects observed

1. No tool-argument validation (`orchestrator.rs:985-986`, `:1036-1037`).
2. No tool-output limits (`:1019-1023`, `:1272-1276`).
3. Name-prefix effect inference (`orchestrator.rs:138-148`, `manager.rs:366-370`).
4. `McpRegistry::merge` does not dedup `tools` (`registry.rs:733-734`).
5. `max_active` and `prefer` dead config (`artifact.rs:96-105`).
6. Keyword threshold ignored (`service.rs:698`).
7. Below-threshold classification injects everything (`manager.rs:1383-1394`).
8. Skill overlay unbudgeted and unprotected from trimming
   (`manager.rs:1448-1454` vs `:1516-1536`; `:1541-1550`).
9. `preferred_tools` inert.
10. `Stdio.sandboxed` inert (`config.rs:21`; `registry.rs:423-428`).
11. `ToolNormalizerDriver` dead code; `build_driver` (`orchestrator.rs:47-76`)
    never constructs it, so `tool_extractor.rs`, `tool_normalizer.rs`,
    `xml_tool_injector.rs` are unreachable.
12. `WasmSkillRuntime` orphaned.
13. Guardrails cover user input only (`server.rs:4702`).
14. Skill MCP servers re-spawn every run (`manager.rs:1457`).
15. Approval timeout hardcoded 300s (`manager.rs:1797`).
16. Activation outcome excludes overlay-only skills (`manager.rs:1441-1445`).

## 7. Open OpenSpec changes touching these areas

| Change | Unchecked / total |
|---|---|
| `wasm-component-skill-runtime` | 6 / 15 |
| `migrate-skills-page-direct-and-redesign` | 10 / 10 |
| `add-skill-kind-and-origin` | 3 / 11 |
| `add-skill-system-submodule` | 3 / 16 |
| `builtin-skills-ui-affordance` | 3 / 5 |
| `skill-activation-metrics` | 3 / 7 |
| `microsandbox-mcp-isolation` | 1 / 80 |
| `mount-governance-guardrails` | 1 / 21 |
| `guardrail-pii-block` | 1 / 9 |
| `tool-loop-cedar-gating` | 1 / 14 |
| `add-preferred-model-to-skill-form-fixtures` | 1 / 5 |

Complete but unarchived: `ch08-activation-outcome-correlation` (10/10),
`wire-mcp-server-provisioning` (8/8), `rmcp-pin-bump` (17/17),
`wasmtime-disposition` (10/10). `governance-tool-approval-reconciliation` has no
`tasks.md`.
