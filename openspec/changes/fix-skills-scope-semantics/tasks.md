## 1. Backend scopes
- [ ] 1.1 Persist per-agent skill state; replace inverted in-memory allowlist semantics.
- [ ] 1.2 Wire per-conversation disables from session agent-config into run-loop matching (verify O1 end-to-end).
- [ ] 1.3 Merge persisted enabled-state over builtin re-registration at startup.
## 2. API/UI
- [ ] 2.1 Serialize origin in SkillResponse; gate delete/edit for pack+builtin origins in UI and API.
## 3. Proof
- [ ] 3.1 Rust integration test: classify -> overlay injection -> SkillActivated; scope-matrix tests; note LLM matcher stub (M4) fixed or explicitly deferred.
