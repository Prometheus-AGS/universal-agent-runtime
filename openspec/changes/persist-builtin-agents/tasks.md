# Tasks — persist-builtin-agents
# Commit: f2d19dc on branch fix/persist-builtin-agents
# Worktree: ~/.claude/worktrees/persist-builtin-agents

## §1 Persist builtins at startup (P0)
- [x] Add `seed_builtin_agents()` in `src/uar/defaults.rs` (idempotent upsert by id for `default-agent` + `orchestrator-agent`)
- [x] Call it at server startup alongside `ensure_default_knowledge_base`
- [x] Row id preserved on re-seed to avoid FK breakage; idempotent on every boot

## §2 First-class serialized shape (P1)
- [x] Add `resolve_agent_for_run()` public helper in `src/uar/api/discovery.rs` — used by chat handler
- [x] `extractAgentConfig` now includes `agent_id` field — propagates through `AgentConfigContext`
- [/] Top-level `name`/`description` mapping deferred — builtins already render via `metadata?.title ?? id` fallback; no breakage observed. Adding explicit field rename is P3 cleanup

## §3 Robust agent switching (P1)
- [x] `agent-selector.tsx` rendered unconditionally (removed `activeThreadId` gate at `chat-page.tsx:146`); Settings button disabled when no active thread
- [x] `agent_id` + `model` added to `UarChatPayload` and included in chat send body
- [x] Server: `ChatCompletionRequest.agent_id` field added; resolved at priority 1 over session side-channel
- [x] `resolve_agent_for_run()` falls back to `default-agent` on lookup failure — no silent regression
- [/] Agent-config POST errors still swallowed in `agent-selector.tsx:121` — surface-errors deferred to P3 UX polish pass

## §4 Validation (gate)
- [x] `cargo check` clean (SKIP_FRONTEND_BUILD=1)
- [x] `cargo test --lib` — 218/218 passed
- [ ] Fresh DB — both builtins in admin list + chat selector — pending manual run after merge
- [ ] Realtime Agent ChangeSet eviction test — pending merge (no longer a theoretical risk with DB backing)
- [ ] Select orchestrator-agent, send message → verify run uses it — pending merge

## Notes
- Worktree: `~/.claude/worktrees/persist-builtin-agents`, branch `fix/persist-builtin-agents`, commit `f2d19dc`
