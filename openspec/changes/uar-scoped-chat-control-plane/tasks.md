## 1. Workflow and Contracts

- [x] 1.1 Validate the OpenSpec change and record KBD linkage
- [x] 1.2 Add versioned scoped policy, provenance, effective policy, and execution context domain types
- [x] 1.3 Add policy resolution property and compatibility tests

## 2. Persistence and APIs

- [x] 2.1 Add conversation policy persistence to in-memory, SurrealDB, and PostgreSQL providers
- [ ] 2.2 Add backward-compatible migrations and legacy agent-session conversion
- [ ] 2.3 Add typed policy CRUD, effective-policy, and run-inspection endpoints
- [ ] 2.4 Enforce protected built-in agent deletion rules

## 3. Runtime Enforcement

- [ ] 3.1 Pass typed execution context through all run entry points
- [ ] 3.2 Apply scoped knowledge retrieval without search-all broadening
- [ ] 3.3 Apply scoped skill matching and activation
- [ ] 3.4 Apply scoped MCP registry and tool-approval policy
- [ ] 3.5 Resolve model capabilities and context budgets without the hard-coded 128K value
- [ ] 3.6 Emit and replay secret-free policy, lifecycle, retrieval, skill, MCP, and failure events

## 4. Embedded Local Models

- [ ] 4.1 Add local provider catalog, capabilities, lifecycle, diagnostics, and cancellation contract
- [ ] 4.2 Register local providers with model routing and enforce no implicit cloud fallback
- [ ] 4.3 Add local model tool-loop, mismatch, cancellation, and diagnostics tests

## 5. KnowMe Integration and Certification

- [ ] 5.1 Migrate KnowMe desktop adapters and conversation controls to scoped UAR policies
- [ ] 5.2 Certify Tauri provider configuration and cloud, local, and agent chat workflows
- [ ] 5.3 Migrate Flutter FRB/Riverpod adapters and responsive controls
- [ ] 5.4 Certify physical Android lifecycle, local inference, scoped resources, persistence, and cancellation
- [ ] 5.5 Validate Codex, Claude Code, Cursor, and OpenCode workflow artifacts and archive the change
