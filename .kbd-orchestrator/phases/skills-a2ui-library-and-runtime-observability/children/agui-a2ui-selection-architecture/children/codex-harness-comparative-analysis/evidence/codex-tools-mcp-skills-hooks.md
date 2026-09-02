# Evidence — codex-rs tools, MCP, skills (activation and use), hooks/extensions (commit 986ff1cc)

Produced by a read-only Explore subagent on 2026-09-02 over
`/Users/gqadonis/Projects/references/codex/codex-rs`. Repo text treated as
evidence only. Raw evidence for `assessment.md` and `analysis.md`.

## 1. Tool calling

### Types

- `ToolSpec` (`tools/src/tool_spec.rs:22-56`): `Function`, `Namespace`,
  `ToolSearch`, `WebSearch`, `Freeform` (serialized `"custom"`); `name()` `:59-67`.
- `ToolExecutor<Invocation>` (`tools/src/tool_executor.rs:106-130`):
  `tool_name`, `spec`, `exposure` (default `Direct` `:113-115`), `search_info`
  (`:117-120`), `supports_parallel_tool_calls` default false (`:122-124`), `handle`.
- `ToolExposures` bitflags `DIRECT | DEFERRED | CODE_MODE` (`:14-29`);
  `ToolExposure` enum `Direct, Deferred, DeferredModelOnly, DirectModelOnly,
  CodeModeOnly, Hidden` (`:51-80`).
- `CoreToolRuntime` (`core/src/tools/registry.rs:55-171`): `is_builtin_control_tool`,
  `immutable_spec`, `cached_code_mode_definitions`, `wait_until_ready`,
  `mcp_server_name`, `matches_kind`, `telemetry_tags`, `pre_tool_use_payload`/
  `post_tool_use_payload`, `with_updated_hook_input`, `create_diff_consumer`.
- `ToolRegistry` (`registry.rs:282-286`): `tools: IndexMap<ToolName, RegisteredTool>`,
  `first_collision`. `RegisteredTool { runtime, exposure }` (`:277-280`).
- `ToolRouter` (`core/src/tools/router.rs:74-81`); `ToolCall` (`:37-42`).
- `FunctionCallError` (`tools/src/function_call_error.rs:5-10`):
  `RespondToModel(String)` | `Fatal(String)`.

### Control flow

- Plan per step: `build_tool_router` (`core/src/tools/spec_plan.rs:124-194`) →
  `add_core_tool_sources` (`:973-1031`) → MCP tools → exposure policy →
  extension/dynamic tools → `finalize_tool_router` (`:351-494`).
- Model-visible specs `build_model_visible_specs` (`:530-567`), namespace merge
  `merge_into_namespaces` (`:899-948`); serialization
  `create_tools_json_for_responses_api` (`tool_spec.rs:82-93`) or Responses-Lite
  (`:95-142`).
- Parse: `ToolRouter::build_tool_call` (`router.rs:246-298`) keeps arguments as
  a raw string. Handlers deserialize via `parse_arguments`
  (`core/src/tools/handlers/mod.rs:83-90`) mapping serde errors to
  `RespondToModel("failed to parse function arguments: {err}")`, returned as a
  failed `FunctionCallOutput` with `success: Some(false)`
  (`core/src/tools/parallel.rs:219-243`); turn continues. Typed args reject
  unknown fields (test `handlers/multi_agents_tests.rs:1735`). Hook-facing input
  falls back to `Value::String` on parse failure (`registry.rs:806-812`).
- Dispatch `dispatch_any_with_terminal_outcome` (`registry.rs:491-752`): counter,
  trace, unknown tool → `RespondToModel` (`:814-819`), `matches_kind` mismatch →
  `Fatal` (`:545-561`), PreToolUse hooks (`:563-615`), OTEL-instrumented
  execution (`:642-657`), PostToolUse hooks (`:717-745`).
- Parallelism: one `Arc<RwLock<()>>` per `ToolCallRuntime` (`parallel.rs:42-62`);
  parallel-capable tools take read, others write (`:155-159`). Eligibility
  `exposure != Hidden && supports_parallel_tool_calls()` (`registry.rs:482-485`).
  MCP tools opt in via server setting or `read_only_hint`
  (`handlers/mcp.rs:128-139`; rationale `:129-130`). Cancellation aborts the
  dispatch task and synthesizes "aborted by user after {secs}s"
  (`parallel.rs:180-208`, `:246-263`).

### Approval, exec policy, sandbox

- `SafetyCheck::{AutoApprove, AskUser, Reject}` (`core/src/safety.rs:20-24`);
  `assess_patch_safety` (`:26-98`) auto-approves confined writes only when a
  platform sandbox exists (`:60-89`).
- `ExecApprovalRequirement::{Forbidden, NeedsApproval, Skip { bypass_sandbox }}`
  from `create_exec_approval_requirement_for_parsed_commands`
  (`core/src/exec_policy.rs:337-462`); `bypass_sandbox` only when every segment
  matches an explicit Allow (`:443-454`).
- `load_exec_policy` (`:662-716`) walks config layers collecting Starlark
  `*.rules`, merges enterprise overlay. `prompt_is_rejected_by_policy` (`:216-238`).
- Unmatched commands `render_decision_for_unmatched_command_for_platform`
  (`:770-855`; sandbox-enforces-instead-of-prompt rationale `:828-831`).
- `BANNED_PREFIX_SUGGESTIONS` ~90 interpreter/shell prefixes (`:57-146`).
- `canonicalize_command_for_approval` (`core/src/command_canonicalization.rs:14-38`).
- Network policy decisions (`core/src/network_policy_decision.rs:47-73`);
  `append_network_rule_and_update` (`exec_policy.rs:514-558`).
- `SandboxType::{None, MacosSeatbelt, LinuxSeccomp, WindowsRestrictedToken}`
  (`sandboxing/src/manager.rs:36-53`); `get_platform_sandbox` (`:62-75`);
  seatbelt SBPL policies, landlock, bwrap (WSL1 guard `manager.rs:1-4`), windows.

### Truncation, collisions, filtering

- `TruncationPolicy::{Bytes, Tokens}` (`utils/output-truncation/src/lib.rs`);
  MCP results use server `output_token_limit` else model policy
  (`handlers/mcp.rs:222-226`); caps: MCP namespace description 512 KiB
  (`mcp.rs:49`), agent-plugin namespace 1 000 B (`:48`), per-plugin MCP spec
  8 000 B / 64 000 B total (`core/src/mcp_tool_exposure.rs:19-20`, `:121-141`
  demotes overflow to `Hidden`).
- Collisions: trusted registration errors on duplicates (`registry.rs:319-334`);
  `register_external_with_exposure` skips with warning and records
  `first_collision` (`:353-384`); reserved names `exec_command`,
  `shell_command` (`:359-367`); `error_on_tool_collisions` →
  `CodexErrorDetails::ToolCollision` (`spec_plan.rs:416-470`).
- Per-model gating: search tool (`spec_plan.rs:628-630`), hosted specs
  suppressed for Responses-Lite (`:600-606`), apply_patch (`:1233`),
  experimental tools (`:1161-1178`, `:1189-1197`, `:1238-1245`), image
  generation (`:698-734`).
- Per-server `apply_mcp_tool_exposure_policy` (`spec_plan.rs:196-268`;
  direct→deferred flip when tool search on `:254-266`).

Decisions: single RwLock parallelism with safe default; malformed arguments are
model-visible errors and only harness bugs are `Fatal` (`registry.rs:558`);
deferred exposure keeps the initial list small and `tool_search` is registered
only when deferred searchable tools exist (`mcp_tool_exposure.rs:90-94`;
`spec_plan.rs:371-406`).

## 2. MCP activation and execution

### Types

- `McpConnectionSet` (`codex-mcp/src/connection_manager.rs:186-201`): `servers`,
  `event_stream_connection`, `disabled_servers`, `protocol_mode`,
  `required_servers`, `optional_startup_deadline`, `tool_catalog_revision`,
  `codex_apps_tools_override`, `tool_plugin_provenance`,
  `prefix_mcp_tool_names`, `non_prefixed_mcp_tool_servers`,
  `elicitation_requests`, `trusted_access`.
- `McpServerConnection` (`:84-91`): `identity`, `client`, `startup_timeout`,
  `startup_trigger: Option<watch::Sender<bool>>`, diagnostics guard; `Drop`
  cancels (`:155-159`). `McpServerView` (`:161-168`) with `tool_filter`,
  `tool_timeout`, `catalog_item_limit`.
- `McpHandler` (`core/src/tools/handlers/mcp.rs:51-55`) with immutable shared spec.
- `ElicitationService` (`core/src/elicitation.rs:12-28`), refcounted pause gate.

### Control flow

- Config via policy-aware `ResolvedMcpCatalog` (`spec_plan.rs:158-165`);
  per-server `enabled`, `startup_timeout_sec`, `tool_timeout_sec`,
  `enabled_tools`, `disabled_tools`, `omit_tools_from`
  (`config/src/mcp_requirements_tests.rs:21-27`); transports `Stdio` /
  `StreamableHttp` (`connection_manager.rs:329-335`, `:626-633`).
- Lazy start: `allow_deferred_startup = startup_policy == LazyWhenCached &&
  previous.is_some()` (`:258-259`); defer only when not a selected-plugin server
  and cache holds an allowed model-visible tool (`:559-570`); watch trigger
  (`:571-576`); spawned not joined (`:700-705`); `client()` fires the trigger
  (`:131-136`).
- Reuse `reusable_client` (`:94-129`) requires identical config, completed
  startup, open client, matching OAuth credentials (`:110-117`); pending
  startups reused (`:415-518`).
- Auth: per-server ChatGPT providers (`:346-360`); failure classes
  `ReauthenticationRequired` etc. (`:466-482`, `:624-678`);
  `authentication_failed_servers` (`:795-810`),
  `updated_oauth_credentials_after_auth_failure` (`:812-847`). OAuth in
  `rmcp-client/src/oauth.rs`, `perform_oauth_login.rs`.
- Startup events `McpStartupStatus::{Starting, Ready, Cancelled, Failed}`
  (`:608-618`, `:680-688`); aggregate `McpStartupComplete` (`:724-764`); failed
  startups schedule `reconnect_failed_startup()` (`:694-696`).
- Per-turn projection: `McpHandlerCache::append_mcp_tools`
  (`core/src/mcp_tool_exposure.rs:37-71`, weak-ptr keyed `:50-56`);
  `append_mcp_tools` (`:75-147`) with model-visibility, `AppToolPolicyEvaluator`
  on `destructive_hint`/`open_world_hint` (`:157-185`), byte budget, Deferred
  or Direct registration.
- Invocation `McpHandler::handle_call` (`handlers/mcp.rs:176-249`);
  `wait_until_ready` (`:274-280`) blocks on `wait_for_mcp_server`; `call_tool`
  (`connection_manager.rs:903-964`) validates, `effective_timeout =
  min(server, requested)` (`:947-952`), strips `ENTITLEMENT_CONTEXT_KEY`
  (`:953-956`).
- Timeouts `DEFAULT_STARTUP_TIMEOUT = 30s`, `DEFAULT_TOOL_TIMEOUT = 300s`
  (`codex-mcp/src/rmcp_client.rs:98-99`).
- Elicitation routing (`codex-mcp/src/elicitation.rs`;
  `rmcp-client/src/elicitation_client_service.rs`); core pause gate
  (`core/src/elicitation.rs:6-10`, `register`/`Drop`, `wait_until_clear`).
- Skill MCP dependencies `maybe_prompt_and_install_mcp_dependencies`
  (`core/src/mcp_skill_dependencies.rs:40-83`): first-party only (`:47-51`),
  feature-gated (`:56-58`), `admit_mcp_dependencies` (`:86-113`) re-resolves the
  policy catalog before and after the prompt (`:76-82`), persisted via
  `replace_mcp_servers` (`:148-155`), OAuth with scope retry (`:195-230`).
- Child processes: `rmcp-client/src/stdio_server_launcher.rs`,
  `executor_process_transport.rs`; environment resolved by
  `resolve_server_environment` (`connection_manager.rs:324-325`). No per-child
  seatbelt/landlock wrapper on this path.

Decisions: lazy startup gated on cache completeness; connection identity
encodes credentials; skill dependencies admitted through the same catalog twice.

## 3. Skill activation and management

### Types

- `SkillMetadata` (`skills/src/model.rs:8-20`): `name`, `description`,
  `short_description`, `interface`, `dependencies`, `policy`,
  `path_to_skills_md`, `scope: SkillScope`, `plugin_id`, `remote_plugin_id`.
  `SkillPolicy { allow_implicit_invocation (default true :23-28), products }`
  (`:63-68`); `SkillDependencies`/`SkillToolDependency` (`:81-94`).
- `LoadedSkillRoot` (`skills/src/loading.rs:23-29`); `SkillRootLoader`,
  `SkillRootSnapshotCache` (`:42-46`, `:113-115`).
- `SkillCatalogEntry` (`ext/skills/src/catalog.rs`) with `authority`, `id`,
  `main_prompt`, `prompt_visible`, `enabled`, `canonical_skill_id`,
  `analytics_scope`.
- `SkillsExtensionConfig` (`ext/skills/src/config.rs:5-16`):
  `include_instructions`, `max_context_tokens`, `bundled_skills_enabled`,
  `orchestrator_skills_enabled`, `shadow_selection_enabled`.

### Control flow

- Discovery: embedded system skills installed to `CODEX_HOME/skills/.system`
  with a fingerprint marker (`skills/src/lib.rs:63-141`); roots via
  `load_roots` (`loading.rs:98-115`); `SkillScope::{System, Admin, Repo, User}`
  (`core/src/skills.rs:141-145`).
- Order: System→Admin→Repo→User→None, then name, then path
  (`ext/skills/src/render.rs:48-67`).
- Catalog line `- {name}: {description} ({locator_kind}: {locator})`
  (`render.rs:252-261`) in `AvailableSkillsInstructions` (developer role, kind
  `skills.catalog`, `fragments.rs:10-61`). Full body only for selected skills
  as `SkillInstructions` (user role, `<skill>` markers, `fragments.rs:64-113`).
- Budget `skill_metadata_budget` (`render.rs:127-153`): explicit
  `max_context_tokens` capped `MAX_CONFIGURED_SKILL_METADATA_TOKEN_BUDGET = 10_000`;
  else 2% of context window (`SKILL_METADATA_CONTEXT_WINDOW_PERCENT` `:20`);
  else `DEFAULT_SKILL_METADATA_CHAR_BUDGET = 8_000` (`:17`). Per-description cap
  `MAX_CATALOG_SKILL_DESCRIPTION_CHARS = 1_024` (`:21-22`, `:1156-1174`); body
  `MAX_SKILL_PROMPT_BYTES = 8_000` (`:19`, `:1176-1178`); name 256 B, path 1 024 B (`:28-29`).
- Allocation `allocate_skill_lines` (`:325-366`): full → minimum lines →
  round-robin description chars (`allocate_description_chars` `:408-447`) →
  omission with note (`:1151-1154`); alias plan (`:1048-1074`, `:1111-1126`).
- Explicit activation `collect_explicit_skill_mentions` (`skills/src/selection.rs:42-109`):
  structured `UserInput::Skill` first (`:67-80`), then `$name` tokens
  (`mentions.rs:41`); path mentions first (`:130-162`); plain names only when
  unambiguous and not a connector slug (`:178-190`).
- Injection `TurnInputContributor::contribute` (`ext/skills/src/extension.rs:359-555`):
  catalog, explicit selection (`:400`), catalog fragment (`:422-451`),
  `read_main_prompt` (`:598-622`), truncation, `SkillInstructions` (`:457-524`);
  read failures are warnings (`:514-522`); `SkillsTurnState` (`:543-548`).
- Shadow selector `ShadowSelectionExperiment`
  (`ext/skills/src/shadow_selection_experiment/mod.rs:50-68`): six lexical
  selectors, metrics `codex.skills.shadow_selection.*` (`:42-48`), line 1 says
  temporary; invoked `extension.rs:401-418`; never changes prompt contents
  (`config.rs:14-15`); feature `skill_search` stable default-on
  (`features/src/lib.rs:1454-1459`).
- Implicit activation `detect_implicit_skill_invocation_for_command`
  (`skills/src/invocation.rs:26-43`): runner+script pattern (`:80-83`),
  ancestor walk to `scripts/` (`:111-127`), or SKILL.md read (`:129-144`).
- Telemetry `emit_explicit_skill_invocations` (`core/src/skills.rs:38-119`)
  `codex.skill.injected` with `status, skill, invoke_type, plugin_id,
  model_slug, reasoning_effort`; implicit dedup per turn (`:121-209`, `:24-25`,
  `:138-162`).

## 3b. Skill use after activation

- Body loaded once at turn input (`extension.rs:598-622`, `:468-481`, `:482-499`).
- Persisted as a real conversation item: `build_extension_turn_input_items`
  (`core/src/session/turn.rs:926-976`, `:970-974`) and
  `record_conversation_items` (`turn.rs:289-290`). Later turns do not
  re-inject; re-mention re-injects. Within-turn dedup `InjectedHostSkillPrompts`
  (`extension.rs:526-541`; `turn.rs:904-915`).
- Scripts/resources not inlined. `skills.read` returns `skill_root`
  (`ext/skills/src/tools/read.rs:66`, `:241-248`); `skill://` resource ids
  (`:117-123`); fingerprinted cursor (`tools/mod.rs:342-362`);
  `MAX_SKILL_RESPONSE_BYTES = 512 KiB` (`tools/mod.rs:56`); byte-budget binary
  search (`read.rs:294-331`).
- Resource reads fail closed without a sandbox context (`read.rs:135-146`,
  `:147-154`); handle validation (`tools/mod.rs:55`, `:328-340`); suite
  `ext/skills/tests/executor_file_system_authority.rs`.
- Script execution is ordinary `exec_command` under the full policy stack;
  attribution inferred after the fact (`invocation.rs:26-43`, `:111-127`).
- No ambient active-skill tag on tool calls (`ToolInvocation` in
  `core/src/tools/context.rs` has no skill field; telemetry tags
  `registry.rs:534-544`). Attribution paths: explicit injection
  (`core/src/skills.rs:38-119`), implicit shell detection (`:121-209`,
  `record_plugin_turn_usage` `:168-171`), `skills.read` of a main prompt
  (`read.rs:277-287`). All converge on `codex.skill.injected` and
  `track_skill_invocations` (`tools/mod.rs:157-168`, `:170-188`);
  `ActiveSkillTurnMetrics::record_plugin` (`tools/mod.rs:137-143`).
- Skills cannot restrict or add tools for the turn: contribute only fragments
  (`extension.rs:355-556`); `tools_for_step` registers only `skills.list` and
  `skills.read` (`extension.rs:289-322`, `tools/mod.rs:91-97`, `:70-74`).
  Indirect additive path is the user-gated MCP dependency install
  (`mcp_skill_dependencies.rs:40-83`, `:148-155`).
- Compaction: `SkillInstructions` matched by `is_skill_prompt_fragment`
  (`core/src/context/contextual_user_message.rs:23`; `ext/skills/src/lib.rs:50-54`);
  `trim_pre_turn_context_updates` (`history.rs:600-628`, `:619-623`) drops
  contextual messages wholesale; catalog fragment in
  `CONTEXTUAL_DEVELOPER_PREFIXES` (`core/src/event_mapping.rs:54`); other drop
  sites `history.rs:122, 273, 386, 400, 620`; not counted as a real API message
  (`history.rs:983`). Dropped, never summarized.
- Caches: `SkillsThreadState` (`extension.rs:590-593`);
  `executor_catalog: Arc<OnceCell<SkillCatalog>>` (`tools/mod.rs:202`, `:237-239`);
  `ExecutorReadSnapshot` has no expiry (`read.rs:155-157`).

Tests: `prompt_hidden_skill_can_still_be_invoked`
(`ext/skills/tests/skills_extension.rs:2370-2445`);
`shadow_lru_selector_recovers_a_skill_invoked_on_an_earlier_turn` (`:857`);
`skills_list_only_returns_model_visible_bounded_metadata` (`:1545`);
`root_qualified_locator_selects_only_the_matching_executor_skill` (`:1916`);
`selected_executor_catalog_follows_step_availability_and_reuses_its_cache`
(`:997`); budget tests `:1239, :1312, :1427, :2023, :2254`;
`core/tests/suite/{skills,skill_approval}.rs`; unit `skills/src/selection_tests.rs`,
`invocation_tests.rs`, `ext/skills/src/render_tests.rs`.

Decisions: body is a first-class conversation item with contextual (droppable)
semantics; skills expand context, not authority; attribution reconstructed from
observable events, acknowledged incomplete (`shadow_selection_experiment/mod.rs:92-97`).

## 4. Hooks, plugins, extensions

- Hook events (`hooks/src/lib.rs:23-35`, 12): `PreToolUse`, `PermissionRequest`,
  `PostToolUse`, `PreCompact`, `PostCompact`, `SessionStart`, `SessionEnd`,
  `UserPromptSubmit`, `SubagentStart`, `SubagentStop`, `Stop`, `Interrupt`;
  nine with matchers (`:44-52`).
- `HookResult::{Success, FailedContinue, FailedAbort}` (`hooks/src/types.rs:16-24`);
  `HookPayload` (`:62-70`); `Hook { name, func }` (`:37-41`, `HookFn` `:12`).
- `PreToolUseHookResult::{Blocked, Continue { updated_input }}`
  (`core/src/hook_runtime.rs:77`). Dispatch points `hook_runtime.rs:124, 184,
  246, 285, 376, 455, 486, 528, 565, 593`.
- Tool-call lifecycle in `dispatch_any_with_terminal_outcome`
  (`registry.rs:563-615`, `:670-745`); PostToolUse block rejects the result not
  the execution (`:703`); `PostToolUseFeedbackOutput` (`:220-245`).
- Hook tool naming `HookToolName` (`registry.rs:247-259`); MCP as
  `mcp__{server}__{tool}` (`handlers/mcp.rs:95-117`).
- Hooks call MCP via `CoreHookMcpExecutor::execute`
  (`core/src/hook_mcp_executor.rs:18-57`), `wait_for_server = false` (`:36`).
- Hook infra `hooks/src/{config_rules,declarations,engine/,registry,
  output_spill,schema,legacy_notify}.rs`; `hook_key` (`hooks/src/lib.rs:112-120`).
- Contributor traits (`ext/extension-api/src/contributors.rs`):
  `McpServerContributor` (`:77`), `ContextContributor` (`:92`),
  `ThreadLifecycleContributor` (`:135`), `TurnLifecycleContributor` (`:186`),
  `TurnInputContributor` (`:227`), `ConfigContributor` (`:244`),
  `TokenUsageContributor` (`:261`), `SkillInvocationContributor` (`:281`),
  `ToolContributor` (`:302`), `ToolLifecycleContributor` (`:326`),
  `ApprovalReviewContributor` (`:354`), `TurnItemContributor` (`:380`).
  `ToolCallOutcome::{Completed, Failed, Blocked}` (`registry.rs:580-608`, `:704-709`).
- Plugins `core-plugins/` (~75 modules: manifest, loader, manager, store,
  marketplace, remote, npm, bundles, executor hooks, MCP overlay, app routing,
  skill snapshots, metrics, startup sync). Core `core/src/plugins/{mod,injection,
  mentions,discoverable,render,metrics}.rs`; `build_plugin_injections`
  (`injection.rs:14-60`). Tool suggest gating (`spec_plan.rs:632-637`, `:1217-1231`).
  Connectors/apps `core/src/connectors.rs`, `AppToolPolicyEvaluator`
  (`mcp_tool_exposure.rs:157-185`).

Decisions: stable tool-agnostic hook contract (`registry.rs:95-138`, `:140-143`);
blocking a result separated from blocking execution; hook MCP calls fail fast
on cold servers.

## Files cited

`tools/src/{tool_spec,tool_executor,function_call_error}.rs`,
`core/src/tools/{registry,router,parallel,spec_plan,context}.rs`,
`core/src/tools/handlers/{mod,mcp,multi_agents_tests,extension_tools}.rs`,
`core/src/tools/code_mode/mod.rs`, `code-mode-protocol/src/{lib,description}.rs`,
`utils/output-truncation/src/lib.rs`, `core/src/{safety,exec_policy,
command_canonicalization,network_policy_decision,function_tool,skills,
mention_syntax,mcp_tool_exposure,mcp_skill_dependencies,elicitation,
hook_runtime,hook_mcp_executor,connectors,event_mapping}.rs`,
`core/src/tools/approvals.rs`, `sandboxing/src/manager.rs`, `bwrap/src/main.rs`,
`codex-mcp/src/{connection_manager,rmcp_client,elicitation}.rs`,
`rmcp-client/src/{stdio_server_launcher,executor_process_transport,
elicitation_client_service,oauth,perform_oauth_login}.rs`,
`config/src/mcp_requirements_tests.rs`, `skills/src/{lib,model,loading,
selection,invocation,mentions}.rs`, `ext/skills/src/{extension,render,
fragments,config,catalog,lib}.rs`, `ext/skills/src/tools/{mod,read}.rs`,
`ext/skills/src/shadow_selection_experiment/mod.rs`,
`ext/skills/tests/{skills_extension,executor_file_system_authority}.rs`,
`context-fragments/src/fragment.rs`, `core/src/context/contextual_user_message.rs`,
`core/src/context_manager/history.rs`, `core/src/session/turn.rs`,
`hooks/src/{lib,types}.rs`, `ext/extension-api/src/{contributors,registry}.rs`,
`core/src/plugins/{mod,injection}.rs`, `features/src/lib.rs`, `cli/src/main.rs`,
`app-server/src/extensions.rs`, `mcp-server/src/message_processor.rs`.
