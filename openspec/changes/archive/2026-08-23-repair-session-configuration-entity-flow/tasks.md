## 1. Establish the entity authority

- [x] 1.1 Add the required 3.0.2 facade exports and typed configured Provider, configured Model, AgentSession, and AgentSessionDraft contracts; verify no feature imports the package directly.
- [x] 1.2 Register Provider, Model, and AgentSession transports once at application boot and register the local AgentSessionDraft schema; verify the idempotent registration source and exported introspection contract at Tier 0, deferring boot observation to the phase's post-code-completion functional proof.
- [x] 1.3 Add platform domain hooks for configured models and field-level session draft selection/actions; place each field subscription in its own control component and verify that source boundary at Tier 0, deferring render observation to the phase's post-code-completion functional proof.

## 2. Repair the frontend flow

- [x] 2.1 Add typed GET/POST session configuration transport using the backend's `model` field; verify load and save decode the same AgentSessionConfig shape.
- [x] 2.2 Migrate ModelSelector to the configured-model domain hook and remove its dependency on catalog loading; verify opening it cannot call `/api/models` and includes configured catalog-unknown providers.
- [x] 2.3 Migrate SessionConfigPanel from component-local business state and render-body setters to canonical/draft domain hooks; add serialized field-merge writes and generation-checked save/cancel handling, verify the lifecycle source at Tier 0, and defer runtime observation to the phase's post-code-completion functional proof.
- [x] 2.4 Remove every context/session control that lacks a complete backend/runtime contract, or implement the contract end to end; produce a per-control inventory naming each visible control's typed API, persistence, and runtime owner (or its removal diff) and verify no retained field is serialized under a key Rust ignores.
- [x] 2.5 Apply the design-system-resolved body inset, bottom inset, and group spacing; verify the responsive source utilities at Tier 0 and defer computed styles at 320, 768, 1024, and 1440 pixels to the phase's post-code-completion functional proof.
- [x] 2.6 Retire obsolete panel store/service code after all consumers move; verify a source search finds no duplicate AgentSession or configured Provider/Model cache owned by the chat feature.

## 3. Make the saved policy effective

- [x] 3.1 Reorder chat request resolution so session identity and effective policy precede final provider/model selection while explicit turn model remains highest precedence; verify the completed source and Tier 0 compilation cover one resolution path, then defer behavior claims to the phase's post-code-completion functional HTTP/browser proof.
- [x] 3.2 Make the frontend runtime stop copying the agent default into each turn when a saved session model should apply; verify the request construction and effective-route source compile, then defer emitted-request and genuine-routing agreement to the phase's post-code-completion functional proof.

## 4. Complete the change

- [x] 4.1 After the vertical slice is code-complete, run only required Tier 0 compilation, lint, type, and architecture checks; do not run a unit suite or the installed-browser functional gate until all UAR phase code is complete.
- [x] 4.2 Run `openspec validate repair-session-configuration-entity-flow --strict`, write row-form `verification.md` with source SHA/profile/limits, and commit only this change's implementation, OpenSpec, and KBD artifacts.
