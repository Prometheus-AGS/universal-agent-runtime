## 1. OpenSpec Artifacts

- [x] 1.1 Create the cleanup proposal describing the active-change validation blocker.
- [x] 1.2 Create the cleanup design documenting the spec-only repair strategy.
- [x] 1.3 Create the `openspec-validation-hygiene` spec delta.
- [x] 1.4 Create this task checklist for verification and archive handoff.

## 2. Delta Repairs

- [x] 2.1 Normalize `a11y` delta wording to `SHALL`/`MUST`.
- [x] 2.2 Normalize `error-boundaries` delta wording to `SHALL`/`MUST`.
- [x] 2.3 Normalize `offline-mode` delta wording to `SHALL`/`MUST`.
- [x] 2.4 Normalize `sse-replay` delta wording to `SHALL`/`MUST`.
- [x] 2.5 Normalize `storage-health` delta wording to `SHALL`/`MUST`.
- [x] 2.6 Normalize `tauri-packaging` delta wording to `SHALL`/`MUST`.
- [x] 2.7 Normalize `tool-analytics` delta wording to `SHALL`/`MUST`.

## 3. Verification

- [x] 3.1 Run `openspec validate implement-opencode-suggestions --strict`.
- [x] 3.2 Run `openspec validate openspec-global-validation-cleanup --strict`.
- [x] 3.3 Run `openspec validate --changes`.
- [x] 3.4 Record KBD progress, waypoint, and QA evidence.

## 4. Archive

- [x] 4.1 Sync the cleanup spec into `openspec/specs/openspec-validation-hygiene/`.
- [x] 4.2 Archive `openspec-global-validation-cleanup`.
- [x] 4.3 Advance KBD execution to the next pending validation-hardening change.
