## 1. Establish the entity authority

- [ ] 1.1 Add the required 3.0.2 facade exports and typed configured Provider, configured Model, AgentSession, and AgentSessionDraft contracts; verify no feature imports the package directly.
- [ ] 1.2 Register Provider, Model, and AgentSession transports once at application boot and register the local AgentSessionDraft schema; verify boot introspection reports each remote entity type exactly once.
- [ ] 1.3 Add platform domain hooks for configured models and field-level session draft selection/actions; verify a changed field notifies its subscriber without changing unrelated field snapshots.

## 2. Repair the frontend flow

- [ ] 2.1 Add typed GET/POST session configuration transport using the backend's `model` field; verify load and save decode the same AgentSessionConfig shape.
- [ ] 2.2 Migrate ModelSelector to the configured-model domain hook and remove its dependency on catalog loading; verify opening it cannot call `/api/models` and includes configured catalog-unknown providers.
- [ ] 2.3 Migrate SessionConfigPanel from component-local business state and render-body setters to canonical/draft domain hooks; verify open initializes a draft, save replaces canonical state then removes the draft, and cancel removes only the draft.
- [ ] 2.4 Remove every context/session control that lacks a complete backend/runtime contract, or implement the contract end to end; produce a per-control inventory naming each visible control's typed API, persistence, and runtime owner (or its removal diff) and verify no retained field is serialized under a key Rust ignores.
- [ ] 2.5 Apply the design-system-resolved body inset, bottom inset, and group spacing; verify computed styles at the established 320, 768, 1024, and 1440 pixel certification widths meet the session-configuration spacing scenarios.
- [ ] 2.6 Retire obsolete panel store/service code after all consumers move; verify a source search finds no duplicate AgentSession or configured Provider/Model cache owned by the chat feature.

## 3. Make the saved policy effective

- [ ] 3.1 Reorder chat request resolution so session identity and effective policy precede final provider/model selection while explicit turn model remains highest precedence; verify the completed source and Tier 0 compilation cover one resolution path, then defer behavior claims to the phase's post-code-completion functional HTTP/browser proof.
- [ ] 3.2 Make the frontend runtime stop copying the agent default into each turn when a saved session model should apply; verify the emitted request and effective backend route agree.

## 4. Complete the change

- [ ] 4.1 After the vertical slice is code-complete, run only required Tier 0 compilation, lint, type, and architecture checks; do not run a unit suite or the installed-browser functional gate until all UAR phase code is complete.
- [ ] 4.2 Run `openspec validate repair-session-configuration-entity-flow --strict`, write row-form `verification.md` with source SHA/profile/limits, and commit only this change's implementation, OpenSpec, and KBD artifacts.
