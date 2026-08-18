## 1. Backend scopes
- [x] 1.1 Confirm the delivered durable per-agent state replaced the restart-lossy inverted allowlist.
- [x] 1.2 Prove persisted session agent-config excludes a skill before run-loop overlay and activation emission (O1).
- [x] 1.3 Confirm persisted enabled-state wins over built-in re-registration after a cold restart.
## 2. API/UI
- [x] 2.1 Retain serialized origin and UI gating; reject built-in/pack edits and deletes in the API while leaving disable available.
## 3. Proof
- [x] 3.1 Observe classify -> overlay injection -> SkillActivated, the session-policy negative path, and the scope matrix; explicitly defer the unrelated LLM matcher implementation (M4).
