## 1. Settings API Secret Handling

- [x] 1.1 Replace fixed sensitive-value masking with schema-guided character-count masking, preserve absent and empty values, recursively restore unchanged nested masks before persistence, and verify the focused settings API unit tests pass under the prescribed locked `server-full` profile.

## 2. Provider Default-Model Control

- [x] 2.1 Add focused ProviderPanel coverage, derive options from every enabled provider-owned model with display-name fallback, render the default model through the existing shadcn `SettingSelect`, and verify the component test plus TypeScript and lint gates pass.

## 3. Phase Validation

- [x] 3.1 Run settings structure validation and strict OpenSpec validation, and verify both commands pass without changing endpoint, payload, entity, store, or realtime contracts.
- [x] 3.2 Run the phase-completion frontend build/full test suite and Rust format/full locked `server-full` test suite, recording actual output and any unverified claim.

## 4. Agent UI Design Guidance

- [x] 4.1 Record the operator-requested UI skill precedence and independent review standard in the durable project instructions so `AGENTS.md` and its `CLAUDE.md` symlink direct agents through Impeccable, Anthropic `frontend-design`, and UI/UX Pro Max in that order, with dual-agent Impeccable critique and fresh-context adversarial review as the normal UI quality gate.
