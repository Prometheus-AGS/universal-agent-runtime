ASSESSMENT: fix-provider-default-settings-consistency
Project: universal-agent-runtime
Date: 2026-08-19
Codebase baseline: Commit `231a5f6669b5893931cbfb24d29d1f50b88eed96` starts and serves requests, but the screen-certification profile exposes a settings bootstrap failure and a non-atomic default-provider mutation.
Cross-tool progress: none in this child; the parent `screen-by-screen-validation` change remains in progress with its BDD source/evidence work preserved outside this child's current write scope.

IMPLEMENTATION STATUS
- Memory embedding configuration: PARTIAL — `MemoryConfig` documents and the memory service implements `openai`, `cohere`, and `local`, and `tests/config_integration.rs` parses `embedding_provider: "local"`; the canonical settings schema in `src/uar/settings/manager.rs` permits only `openai` and `cohere`.
- Settings bootstrap: PARTIAL — `SettingsManager::initialize` validates each configured value against its namespace schema. With `memory.embedding_provider=local`, initialization stops before the later LLM settings are seeded. `src/server.rs` logs the error but still exposes the partially initialized `SettingsManager`, so later mutations reach missing rows.
- Default-provider persistence: PARTIAL — `POST /api/uar/providers/{id}/default` calls `ProviderRegistry::set_default` before `SettingsManager::set_default_provider_id`. A persistence failure therefore returns HTTP 500 after the in-memory default has already changed.
- Regression coverage: PARTIAL — configuration parsing covers `local`, and settings persistence covers a successful initialized default-provider write. No test combines `local` with settings bootstrap, and no provider API negative control proves that a failed durable write leaves the registry default unchanged.

CROSS-TOOL PROGRESS
- NONE — child runtime revision 153 records zero changes and zero implementation tasks.
- RELATED BUT NOT EXECUTED HERE — `make-config-authoritative-on-boot`, `optimistic-mutations`, `providers-page-direct-mutations`, and `contract-optimistic-rollback` are incomplete OpenSpec proposals that overlap provider/settings or frontend rollback behavior. This child must not absorb their broader config-authority or UI migration decisions.

SPEC GAP SUMMARY
- `openspec/specs/provider-model-settings-certification/spec.md` requires default selections to round-trip through their owning APIs and determine routed work. The observed HTTP 500 plus changed in-memory default violates that requirement.
- The supported memory configuration and the generated settings schema disagree about `local`. The child needs an OpenSpec delta that makes accepted configuration values consistent across resolved config, bootstrap schema, and settings writes.
- A schema-only repair is insufficient: it would remove the observed trigger but leave the provider handler capable of publishing an in-memory default after any future persistence failure.
- The generated child `scope.json` currently permits only child KBD artifacts. Planning must explicitly widen it to the minimum product, test, OpenSpec, and append-only history paths before Execute.
- `handoff-in.md` still contains generated placeholders. Planning must convert the observed failure and return-to-parent condition into explicit success criteria before Execute.

OBSERVED FAILURE EVIDENCE
- Command: `curl -sS -i -X POST http://127.0.0.1:3102/api/uar/providers/bdd-provider-b/default`
- Output: `HTTP/1.1 500 Internal Server Error` with `{"error":"Failed to persist default provider: Setting 'llm.default_provider' not found"}`.
- Command: `curl -sS http://127.0.0.1:3102/api/uar/providers | jq '{default_id}'`
- Output after the rejected mutation: `{"default_id":"bdd-provider-b"}`.
- Server output before those requests: `Settings bootstrap failed — continuing without persistent settings`; cause chain ended with `Setting 'memory.embedding_provider' data failed JSON Schema validation: "local" is not one of "openai" or "cohere"`.

BUILD HEALTH
- build check: UNKNOWN — no compilation command was run during this fact-finding phase, and no product source was changed.
- focused runtime behavior: FAIL — an isolated current binary returned health HTTP 200 and provider creation HTTP 201, then `POST /api/uar/providers/bdd-provider-b/default` returned HTTP 500 with `Failed to persist default provider: Setting 'llm.default_provider' not found`.
- negative observation: FAIL-CLOSED CONTROL MISSING — after that HTTP 500, `GET /api/uar/providers` reported `default_id: "bdd-provider-b"`; the rejected mutation still changed runtime routing state.
- known violations: the settings schema excludes a supported value; startup continues with a partially initialized manager; provider mutation ordering exposes partially committed state.
- test coverage: PARTIAL — positive paths exist, but the two observed failure scenarios have no regression tests.

CONSTRAINT CHECK
- AGENTS.md violations: NONE introduced by this assessment. The child was created before product execution, the observed defect was reproduced rather than inferred, and no product code was changed.
- child-scope constraint: COMPLIANT for Assess — only child KBD artifacts are being written. Product edits are forbidden until Plan narrows and widens `allowedWritePaths` deliberately.
- constraints.md violations: N/A — `.kbd-orchestrator/constraints.md` does not exist.
- capability-inversion constraint: unaffected — the repair belongs to trusted host settings/provider code, not an agent kernel.

GOAL PROGRESS
- Align the settings schema with the supported local memory embedding provider: NOT MET — the enum remains `openai|cohere` while resolved configuration and runtime support include `local`.
- Make default-provider updates persistence-consistent without partial in-memory mutation: NOT MET — the handler mutates the registry before attempting persistence.
- Add focused regression evidence and return control to screen-by-screen-validation: NOT MET — the child has no OpenSpec change, tests, verification artifact, commit, reflection, or handoff-out yet.

MINIMUM REPAIR BOUNDARY FOR PLAN
- `src/uar/settings/manager.rs`: align the memory settings schema with the already supported `local` value and add a focused bootstrap regression test in the existing settings test surface.
- `src/uar/api/providers.rs`: validate the target provider, require the durable default write to succeed before changing the registry, and add a negative control showing that persistence failure leaves the prior default unchanged.
- OpenSpec: extend the existing provider/settings capability with the two observed failure scenarios; do not implement the broader inactive provider/config proposals.
- Parent return condition: focused Rust checks, Tier 0, strict OpenSpec validation, and independent artifact review pass; then exit the child and resume the already-authored Providers/Auth/MCP browser checks before full screen recertification.

SYCOPHANCY REVIEW
- The optional `sycophancy-correction` tool is not available in this session. Manual review retained the uncomfortable findings: the current startup deliberately continues after a bootstrap error, and a failed API response currently changes live routing state.

ASSESSMENT COMPLETE
