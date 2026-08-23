## 1. Encode the architecture rules

- [x] 1.1 Add the React/entity-state standing rule outside managed regions in both `AGENTS.md` and `CLAUDE.md` and reconcile `.claude/rules/typescript.md` into explicit entity-backed and transient-Zustand paths; verify it names Vercel React Best Practices, Composition Patterns, the applicable Entity Management skill, platform domain hooks, narrow selectors, and the per-row mutation prohibition without permitting components to import stores/services or hooks to fetch directly.
- [x] 1.2 Extend the existing frontend boundary checker with deterministic rules for render-body setters, per-row feature graph writes, facade bypass, and named duplicate entity caches; verify each rule reports a stable identifier and location.
- [x] 1.3 Add one failing negative fixture per forbidden pattern and allowed fixtures for UI-local widget state and event-driven domain actions; run the negative runner and observe every forbidden fixture fail for its intended reason before checking the repaired source passes.

## 2. Add the bounded functional proof

- [x] 2.1 After all UAR phase implementation is code-complete, build and install the production UI/service once and prepare one focused Playwright scenario against `http://localhost:1906`; verify no mock model or service replaces the installed path.
- [x] 2.2 Run the short local functional HTTP/browser scenario and verify explicit-turn, saved-session, and agent-default model precedence; open responsiveness within two seconds; absence of `/api/models`; graph publications no greater than configured providers plus configured models plus six; configured-model visibility; save/reopen; cancel isolation; genuine inference through the saved model; and computed spacing at 320, 768, 1024, and 1440 pixels.
- [x] 2.3 Capture the browser console/network trace and matching UAR server-log interval under `.prometheus`; verify no credential value is recorded and any failed assertion blocks completion.

## 3. Complete the change

- [x] 3.1 Run required frontend Tier 0 checks and `openspec validate prevent-session-configuration-regressions --strict`; verify no product test was added to or run in GitHub Actions.
- [x] 3.2 Write row-form `verification.md` with commands, observed output, source SHA, installed profile, host-local timing limit, and unverified limits, then commit only this change's guardrails, functional evidence, OpenSpec, and KBD artifacts.
