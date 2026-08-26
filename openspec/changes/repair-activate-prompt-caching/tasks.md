## 1. Control-plane contracts

- [x] 1.1 Register and seed the admin-protected `prompt_caching.enabled=false` namespace and verify focused settings manager/API tests cover seed, round-trip, and authorization.
- [x] 1.2 Add nullable prompt-caching fields to Rust and TypeScript session policy contracts and verify legacy JSON without the field deserializes as Inherit.
- [x] 1.3 Implement four-state JWT user updates, tenant-plus-subject principal keys, and write-through persistence for Memory/Postgres/Surreal; verify update-state, reload, and two-principal isolation tests.
- [x] 1.4 Implement owner-safe session-effective prompt-caching and exact empty 204 agent-config absence behavior; verify precedence, source, cross-owner indistinguishability, first save, and subsequent GET tests.

## 2. Provider execution

- [x] 2.1 Inventory all production policy-bearing `LlmRequest` constructors and document internal calls that do not inherit user policy; verify the inventory against a source scan.
- [x] 2.2 Route effective prompt caching through one request-strategy seam for initial chat, tool-loop, compatibility, and failover paths; verify focused policy and dispatch tests.
- [x] 2.3 Select native Anthropic under its feature gate, remove unconditional cache strategy, and preserve liter-llm fallback; verify stub upstream bodies contain cache controls only when On and feature-gate parity tests pass.
- [x] 2.4 Prove OpenAI dispatch and bodies are unchanged by the UAR toggle and map mocked provider cache usage into deterministic metrics with focused tests.

## 3. Frontend controls

- [x] 3.1 Refine the Prompt Caching panel to render only authoritative values, block on initial error with Retry, preserve dirty drafts, expose complete status/accessibility, and remove unsupported controls; verify focused React loading, recovery, dirty-state, and accessibility tests.
- [x] 3.2 Extend the session-configuration entity domain with persisted Inherit/On/Off and authoritative effective-source state; verify focused domain, transport, and React persistence tests.
- [x] 3.3 Accept empty 204 and legacy 404 as absent agent configuration while preserving other errors; verify complete first-save and subsequent-load frontend tests.

## 4. Documentation and history

- [x] 4.1 Add the prompt-caching provider guide and cross-links from provider configuration, cost, observability, and troubleshooting; verify Docusaurus lint, typecheck, and build succeed.
- [x] 4.2 Append the decision, root cause, verification evidence, and session summary to tracked `.prometheus` history and verify prompt snapshots remain the only excluded knowledge content.

## 5. Integrated verification and deployment

- [x] 5.1 Run strict OpenSpec validation, Rust Tier 0/Tier 1/Tier 2, TypeScript Tier 0/Tier 1/Tier 2, and scoped architecture checks locally and record the observed outputs.
- [ ] 5.2 Run extension-free Playwright against the installed service with no prompt-caching or agent-config 404, page error, or app-origin console error; separately load the repository MV3 extension and verify connected/disconnected messaging.
- [x] 5.3 Build through the existing macOS packaging path, record the release SHA-256, install and confirm hash equality, restart the LaunchAgent, and verify its executable, PID, state/logs, health, APIs, and port 1906 browser behavior.
- [x] 5.4 Run ordered adversarial UI/code review, address material findings, and rerun affected checks.
- [x] 5.5 Commit and push all in-scope code, docs, OpenSpec artifacts, repository-local skills, and tracked `.prometheus` history; verify the checkout ends on `main` with no temporary worktrees or worktree branches.
