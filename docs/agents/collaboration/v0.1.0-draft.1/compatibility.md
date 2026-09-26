# Compatibility and migration contract

## Current-source evidence vs required behavior

The table describes static inspection at UAR a54dd9591dfeaff8694252b69d2d40f0f28e2250, not installed acceptance. The complete descriptor IR is retained by compilation; the to_artifact projection used by compile-and-register is narrower. Copying data into an extension proves storage only, not execution.

| Original section/field | Current projection observed | Collaboration requirement / I1 disposition |
|---|---|---|
| Metadata id/version/name | Artifact ID derived from heading slug, version copied, selected display metadata copied | Preserve original ID/version/provenance and explicit rename mapping; do not create duplicate identities on rename |
| Identity name/role/persona/system_prompt/instructions | Synthesized/explicit prompt and instructions | Preserve complete sourceDescriptor and record prompt transformation |
| UI forms/actions/artifacts | Enabled booleans/preferred artifact types; input/output/state defaults | Retain UI schemas/action refs/catalog version; unsupported required action refused |
| Capabilities | Stashed under extensions | Preserve declarations and map actual admission requirements |
| Skills id | Copied to policy.skills.prefer | Preference is not an allow/deny policy |
| Skills version/required/config | Not retained by that policy or section stash | Preserve and bind exact dependency/config; reject missing required skill |
| Tools allow/deny/bundles | Selected policy/bundle conversion; empty allow inherits defaults | Preserve declared empty/inherit distinctions and exact native fields; compare effective policy |
| MCP Servers | Stashed; bundles reference servers | Preserve endpoint/auth references without exporting credentials; resolve current allowed tools |
| Knowledge sources | IDs only in memory.kb | Preserve source options, scope and retrieval semantics |
| Memory Model | Conversation enabled copied | Preserve remaining isolation/retrieval requirements or diagnose unsupported |
| A2A Contracts | Stashed | Preserve versioned contract; no conformance inference |
| Governance | Stashed | Current Cedar/host composition still enforced; no role-granted permission |
| Budgets & Constraints | Stashed | Bind aggregate reservations/ceilings; do not equate storage with enforcement |
| Execution Model | Stashed | Match admitted kernel profile; unsupported requirements block activation |
| Observability | Stashed | Bind required audit facilities or refuse required profile |
| Deployment Profiles | Stashed; first provider used as default | Preserve profile selection and model/connection identity in binding |
| model_requirements | Missing from projection stash | Preserve full v2 value and enforce/diagnose field support |
| prompt_dialect | Missing from projection stash | Same; defaults cannot silently replace requested dialect |
| rag_configuration | Missing from projection stash | Same; retain retrieval configuration/scope |
| context_strategy | Missing from projection stash | Same; enforce actual authorized context selection |
| api_harness | Missing from projection stash | Same; explicit compatible harness/profile required |

Canonical AgentDefinition sourceDescriptor retains the complete imported IR; legacySections retains original authoring when distinct. Every conversion produces a JSON-pointer diagnostic with source revision, target profile, disposition (exact, translated, optional-unsupported, required-unsupported), reason and effective binding reference. Required-unsupported prevents activation. Optional fields survive export unchanged. Unknown extension preservation cannot preserve credential values: redact private authority content and declare that export needs rebinding.

## Cross-harness declaration matrix

The existing mini native-export notes inspected on2026-09-26 list SEVEN CLI targets and TWO service targets. Earlier planning shorthand “eight harnesses” is not an inventory. This draft covers the actual nine targets without claiming native UAR team support in them. All rows below are source-documentation evidence; installed adapter conformance remains pending.

| Target | Existing export surface | Collaboration adapter obligation |
|---|---|---|
| Codex | .codex/agents TOML plus project config | Preserve instructions/options; native threads are not portable UAR tasks/approvals |
| Claude Code | Agent Markdown, settings, optional plugin/marketplace | Subagent installation does not create an experimental running team |
| Copilot | .github/agents custom agent Markdown | Agent selection and Fleet behavior are distinct from UAR task lifecycle |
| Kimi Code | .kimi-code agents plus optional plugin/marketplace | Current target differs from historical kimi-cli research snapshot; do not assume identical model/nesting behavior |
| MiniMax | Active user-data agent directory | No unverified custom-agent exec selector; preserve destination diagnostics |
| OpenCode | Agent Markdown and config | Preserve prompt/permission merge semantics; JS/TS plugins are separate |
| DeepSeek Harness | Cordis persona and experimental composition profiles | Configuration is not member creation, model isolation or durable UAR ownership |
| UAR | AgentArtifact registration plans | Existing exported agents do not imply persistent team API support |
| BossFang | Agent TOML, optional Hand/workflow registration | Its workflow retains ownership; UAR delegation is a distinct future route |

Every new adapter exports its source version, native CLI/schema provenance, supported/unsupported capabilities and field-loss report. Required UAR-only semantics MUST refuse native execution rather than reduce the team to prompts silently. Opaque native options remain native; import cannot turn them into UAR grants. Full/mini parity covers portable authoring and runtime assets, not interchangeable native sessions.

## Upgrade and rollback

I1 adds immutable canonical documents alongside legacy artifacts and records migration source hash/target digest. Existing deployments remain pinned. Publish provider support before requiring it in consumer bindings. Preserve original records so a failed conversion can be inspected and repaired without rewriting user definitions.

I2 introduces durable records only after storage capability proof and a migration/backup plan. Downgrade must refuse bindings/tasks requiring unknown semantics; it cannot silently resume them with an old single-agent runner. Quiesce or drain before storage downgrade and retain uncertain-effect records. Do not promise rollback of already executed external effects. Schema/version switches require explicit operator decisions and versioned migration receipts.
