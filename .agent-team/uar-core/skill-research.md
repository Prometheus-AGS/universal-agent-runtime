# UAR team skills research

Date: 2026-09-24. Research only; no project changes or downloaded-code execution.

## Position and evidence

Read UAR AGENTS.md, waypoint and versions.toml. Restored runtime-harness-gap-closure, ready, revision 2585; current recorded change harness-model-profiles-routing (4/8 tasks in openspec list). This research does not advance that phase. UAR pins SurrealDB 3.2.4, Surreal Memory 432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8 and Liter c5c6caac617eb931cd5009146a70831422ec236c. Cargo.toml shows cedar-policy 4.9.1 and a2a-transport/server-full features.

Inventory roots inspected: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills, .claude/skills, skills, and /Users/gqadonis/.codex/skills. Firecrawl developer_search (skills=only) and web search found candidate sources; exact upstream commits and trees were fetched read-only from the GitHub REST API, and candidate Markdown was fetched from commit-addressed raw GitHub URLs. No installer, candidate code, Cargo, or product tests ran.

## Installed skills useful for the team

| Capability | Actual installed skills | Location / caveat |
|---|---|---|
| Rust runtime baseline | prometheus-rust-workspace, rust-best-practices, actor-model | Global Codex; general and actor guidance already present |
| Tokio, cancellation, concurrency | rust-async-patterns | Global Codex; CancellationToken, JoinSet, bounded concurrency and select guidance |
| Axum routes and trusted ingress | axum-patterns, axum-agent-gateway | Global Codex; gateway skill explicitly names Cedar, identity, tenancy, idempotency and cancellation |
| MCP execution and transport | rust-mcp-server-generator, mcp-builder, agent-runtime-security | Global Codex; existing server changes must not scaffold a replacement |
| AG-UI / A2UI | agui-event-contract, a2ui-surface-contract | Global Codex; correlation, replay, resume, explicit terminal outcomes |
| Cedar / capabilities | agent-runtime-security, axum-agent-gateway | Global Codex; security sequence covered, no focused Cedar syntax skill found in inspected roots |
| Memory | kbd-memory-recall, entity-realtime-surreal-live | Global Codex; process memory and LIVE-to-entity adapter, not broad SurrealQL |
| React entity architecture | prometheus-entity-skills and nested entity-graph CRUD/setup/realtime/optimize skills | Project .claude/skills plus global; UAR AGENTS.md is stricter than generic examples |
| UI review | impeccable, frontend-design, ui-ux-pro-max, web-design-guidelines | Project .agents/skills; precedence explicitly defined in AGENTS.md |
| Verification | bdd-cucumber-rs, bdd-cucumber-js, bdd-video-proof, hybrid-runtime-verification, adversarial-review | Global; bdd-testing is a deprecated redirect, not a substantive skill |
| Spec workflow | openspec-propose, openspec-apply-change, openspec-verify-change and companions | Project .agents/.claude skills |

The inspected roots do not contain vercel-react-best-practices, vercel-composition-patterns, surrealql, or a2a-protocol. UAR nevertheless requires both Vercel skills by name. Imported skills elsewhere in runtime submodules were outside the requested inventory scope; do not call this proof of machine-wide absence. rust-router is also absent from the inspected roots, while the Rust routing skill treats it as optional when unavailable.

## Recommended missing candidates (three)

### 1. Vercel React Best Practices

Primary source: https://github.com/vercel-labs/agent-skills/tree/063bee94c3f4df8453406c830b0a7df0f2860278/skills/react-best-practices

Pinned commit: 063bee94c3f4df8453406c830b0a7df0f2860278. Exact subtree: skills/react-best-practices/. Skill name: vercel-react-best-practices. License: MIT declared in SKILL.md frontmatter; repository API returns license=null and no root LICENSE was found in the tree. Preserve the attribution and frontmatter; this is weaker license packaging than a root license file, not evidence of an absent license declaration.

Why useful: explicitly required by UAR rules and supplies focused subscription, render, async and bundle rules complementary to the entity architecture skills. Do not install unrelated deployment skills from the same repository.

Audit: entrypoint and selected rules read; all files in both Vercel subtrees fetched and pattern-scanned (90 Markdown/JSON files, 280592 characters total; no executable files). No instruction-override or exfiltration pattern found by that limited scan. An unpinned executable example exists: rules/rendering-svg-precision.md:27 invokes npx svgo. Treat this as example text, not automatic permission to download/run packages. The general client SWR, Next.js, module-cache, direct-import examples are not UAR architecture authority. In particular, bundle-barrel-imports must never bypass the required @/platform/entities public domain-hook boundary.

Recommendation: install project-local with upstream files intact plus an explicit adjacent provenance/precedence note. Activate only relevant rules; pinned project versions and observed defects govern applicability.

### 2. Vercel Composition Patterns

Primary source: https://github.com/vercel-labs/agent-skills/tree/063bee94c3f4df8453406c830b0a7df0f2860278/skills/composition-patterns

Same pinned commit and MIT frontmatter caveat. Exact subtree: skills/composition-patterns/. Skill name: vercel-composition-patterns.

Why useful: another explicit UAR rule requirement, with narrow component composition and provider-interface guidance; visual Impeccable guidance and entity persistence skills do not duplicate it.

Audit: SKILL.md and state-lift-state rule read, entire subtree included in the document-only scan above. State-provider examples use useState; they are permissible for UI-local mechanics only. They must not relocate Provider, Model, AgentSession or other business entities into component/context state. React 19-only guidance must be checked against actual frontend pins. No executable assets in subtree.

Recommendation: install project-local, same precedence note as candidate 1.

### 3. Official SurrealQL

Primary source: https://github.com/surrealdb/agent-skills/tree/e5eadc5901337f20ad06ab5acf48d8f83bbfc961/skills/surrealql
Official publisher confirmation: https://surrealdb.com/docs/build/ai-agents/agent-skills

Pinned commit: e5eadc5901337f20ad06ab5acf48d8f83bbfc961. Exact subtree: skills/surrealql/. Include repository LICENSE (MIT, Copyright 2026 SurrealDB Ltd.). Skill version: 0.2.0. Four Markdown files: SKILL.md and references/querying.md, schema.md, values.md.

Why useful: concrete schema, records, graph traversal, transactions, NONE/null and query guidance missing from the existing memory-recall and LIVE adapter skills. It does not replace UAR host-owned persistence or tenant/authz boundaries.

Audit: all four Markdown files read. No bundled scripts. Material conflicts: entrypoint says always use latest stable; UAR pins 3.2.4. It suggests curl for remote version discovery and unpinned npx @surrealdb/surql-fmt (including --write). Do not run these automatically. Schema examples include full select/create permissions, an OWNER user with literal placeholder password, and destructive REMOVE/DELETE examples. These are illustrative syntax, not secure production defaults or authorization to execute mutations. Review query features against 3.2.4 and locally installed tools at the completed change boundary.

Recommendation: install reference-only at project level with MIT license and a precedence note binding to versions.toml, no package execution or schema mutation authorized by loading the skill.

## Fourth candidate considered, deferred

A2A Protocol Reference, exact subtree .claude/skills/a2a-protocol/ in https://github.com/mckinsey/agents-at-scale-ark/tree/cf2985d11e0b8fbaefebfc86b0f5315c7ccce036/.claude/skills/a2a-protocol . Repository license Apache-2.0; reference frontmatter attributes official A2A source under Apache-2.0. Forty-six Markdown files, 428335 characters, no executable files in the inspected subtree. Entrypoint allowed-tools is Read,Grep.

It would fill a real A2A protocol-reference gap, but do not install unchanged: its own entrypoint at lines 69 and 148 says /.well-known/agent.json, while its v0.3/spec-05-agent-card.md:28 and topic-agent-discovery.md:34 specify /.well-known/agent-card.json. This is a reproduced internal documentation inconsistency, not a speculative concern. It also defaults to v0.3 and claims SDK support currency without a time/version binding. Existing UAR contracts and selected protocol version must win. Use primary official protocol docs on demand until a corrected, version-bound skill is available.

Cedar search returned Amazon Bedrock AgentCore-oriented skills. Their cloud/runtime assumptions do not fit UAR's local Cedar governed host and would add more conflict than coverage. No Cedar-specific download recommended from this search. Generic Rust/Axum/MCP skills were rejected as redundant with already installed skills.

## Independent team candidate: five roles

This proposal was made without reading other agents' or the parent agent's candidate designs. Roles are a reusable roster, not a requirement to launch five agents concurrently.

1. Runtime and contract lead: owns scope, accepted OpenSpec boundaries, versions, data-flow and contract decisions, work partitioning, and final integration. Uses Rust workspace routing, OpenSpec and api-and-interface-design. Decides which specialist is needed; does not self-certify final evidence.
2. Runtime orchestration and protocols engineer: owns the affected src/llm/orchestrator.rs, src/uar/orchestrator/ and task lifecycle plus src/uar/api/a2a/ contracts for the assigned slice. Uses rust-best-practices, rust-async-patterns, actor-model, rust-mcp-server-generator, agui-event-contract. Explicitly traces cancellation from ingress through child runs, provider request and terminal receipt.
3. Trust and persistence engineer: owns assigned src/uar/governance/, host tool-admission paths, src/uar/memory/ and persistence boundary changes. Uses agent-runtime-security, axum-agent-gateway, SurrealQL and memory skills. Verifies immutable verified identity, narrowed capabilities, policy at resume, and durable receipts. Splitting this into separate security/database writers is unnecessary unless both have independent production changes.
4. Entity UI engineer: owns frontend/src/platform/entities/, typed AG-UI projections and affected frontend features. Uses project entity skills, Vercel candidates and the mandated Impeccable/UI routing. Business state stays in graph; components consume domain hooks. UI critique is a separately isolated review invocation, not approval by its author.
5. Independent integration verifier / artifact critic: read-only over implementation until it reports a concrete repro; owns evidence artifacts and completed-boundary integration scenarios. Uses BDD skills, AG-UI contracts, adversarial review and relevant runtime verification. Receives artifact + acceptance criteria + contract facts, not generation rationale. May block claims; does not mutate product code or silently broaden gates.

Handoff: contract owner provides exact owned files, inputs/outputs, accepted scenarios and exclusions. Implementers return changed paths and unresolved contract facts. Verifier exercises one end-to-end path through HTTP/authz → governed runtime → adapter/persistence → event stream → entity projection, including cancellation/reconnect/tenant denial only when within the accepted change. One build writer per target directory. No GitHub Actions product testing.

Simpler alternative: three roles (runtime lead/engineer; entity-and-persistence engineer for truly bounded full-stack slices; independent verifier). Prefer this for small changes and serial shared-file work. Drawback: one implementer spans security and persistence, so an isolated trust-boundary review must still be scheduled for security-sensitive changes.

Concrete failure modes to design against:

- Protocol owner emits terminal success while cancellation/persistence owner leaves an unresolved child or receipt; test the shared production lifecycle and require explicit event/receipt contracts.
- Security split lets transport metadata become identity or a resumed checkpoint reuse stale policy; trusted host ownership and fresh policy resolution are contract facts, not reviewer preferences.
- UI engineer adopts Vercel state-lifting/SWR literally and duplicates graph entities; platform public-hook boundary and field-level subscribers must be an acceptance criterion.
- Parallel agents each compile the same target or overwrite a shared orchestrator file; lead grants a single writer and bounded file ownership before dispatch.
- Verifier receives the author's narrative and replays only happy-path tests; isolated artifact-only review plus exact phase-boundary receipts reduce confirmation bias.
- Five-role overhead on a one-file fix wastes time; choose three roles or one implementer plus independent critic, loading specialists only for real affected boundaries.

The uncomfortable thing: installing these skills does not certify their syntax against UAR's pins, and a larger team can increase interface mistakes. The team should be selected from the task's affected boundaries, with existing UAR contracts taking precedence over external examples.

## Completion accounting

Only /tmp/uar-team-skill-research.md was written. No project source, native export, dependencies or skill installations changed. Read-only checks: local inventories, openspec list, upstream commit/tree/license lookup, entrypoint/relevant-reference review and static text scanning. One initial Node research command failed with ERR_AMBIGUOUS_MODULE_SYNTAX; rerun with --input-type=module succeeded. No guards/code added. Product functionality, full semantic audit of every external reference, and skill effectiveness remain unverified because this was a research task, not implementation or certification.

## Exact existing project paths for role configuration

- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/prometheus-entity-skills/SKILL.md — not found at this path
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/prometheus-entity-skills/entity-graph-optimize/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/prometheus-entity-skills/entity-graph-realtime/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/prometheus-entity-skills/entity-graph-crud/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.claude/skills/kbd-process-orchestrator/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/impeccable/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/frontend-design/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/ui-ux-pro-max/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/web-design-guidelines/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/openspec-propose/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/openspec-apply-change/SKILL.md — exists
- /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.agents/skills/openspec-verify-change/SKILL.md — exists

Rust, Axum, cancellation, MCP, runtime security, AG-UI and BDD skills named above are global Codex skills; do not export fictitious project-local paths for them. For each, the verified convention is /Users/gqadonis/.codex/skills/<listed-skill-name>/SKILL.md. Global skill resolution is a portability dependency until the parent explicitly vendors or exports those skills.
