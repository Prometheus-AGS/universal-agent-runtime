## Why

The operator needs one project-specific development team that can be used from Codex, Claude Code, Kimi Code, OpenCode, or MiniMax CLI. UAR's runtime, policy, protocol, persistence, and operations-console boundaries require explicit ownership and independent verification; a generic roster would hide those boundaries and the responsibility split with sibling repositories.

## What Changes

- Create a portable UAR-only team definition, grounded in the Rust-only Compass graph, source inspection, dependency manifests, and the existing KBD position.
- Compare independently proposed team structures and choose a small default working group with specialists activated only for relevant work.
- Install native role definitions for the five requested coding harnesses, preserving existing project configuration and documenting native differences.
- Bind actual project skills and researched, provenance-recorded skill additions to responsibilities; use Firecrawl to discover missing skills.
- Record concrete model choices where native support and discovery permit them, distinguish configured availability from successful inference, and preserve unknown prices/capabilities.
- Define inspectable cross-project handoffs without granting this team write ownership of sibling repositories or starting a new orchestration daemon.
- Run isolated adversarial review and sycophancy correction on the design and final artifacts; validate actual export and native discovery paths where available.

## Capabilities

### New Capabilities

- `project-development-team`: Project-local role ownership, skill/model/tool bindings, five native coding-harness installations, cross-project handoff contracts, and explicit verification receipts.

### Modified Capabilities

None.

## Impact

Project tooling and documentation only: `.agent-team/uar-core/`, selected project-level skill directories, native agent/configuration files, and this OpenSpec change. Existing agent definitions, MCP entries, user configuration, and uncommitted work must be preserved. No application runtime, dependency pin, runtime UX, provider wire contract, or realtime entity behavior changes. Those surfaces inform the roles and their verification responsibilities.

The authoritative KBD phase remains `runtime-harness-gap-closure`; this work does not rewrite its generated position/progress state. Native files describe available agents; they do not register agents into a live UAR service or start team execution. Global skill/CLI installation and other repositories' team creation are out of scope.
