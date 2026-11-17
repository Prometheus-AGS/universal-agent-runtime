# Proposal

## Why

An eligible new Mac should use its already available Apple on-device model to help the operator set up stronger models and providers. The current missing-model requirement forbids that first-run path even when it needs neither a download nor credentials.

## What Changes

- Amend the existing provider specification directly and retain this delta as the implementation handoff. This is proposed behavior, not released or locally verified integration.
- Default to available `SystemLanguageModel` only on supported native Apple profiles with no saved model choice, explicit configuration, or bootstrap opt-out. Preserve all existing selections and non-Apple missing-model behavior.
- Describe optional macOS 27 custom `LanguageModel` adapters through MLX, exported Core AI resources, or a flint-loader bridge. Preserve UAR tool authority and flint-loader custom-weight lifecycle authority.
- Require observable durable setup progress, consent for downloads, provider transitions, cloud use and credential handling, cancellation, retry, resume, and manual fallback.
- Keep generation and embedding registry identities and reindex decisions separate.

## Capabilities

### New Capabilities

None; provider selection and setup belong to the existing capability.

### Modified Capabilities

- `provider-model-settings-certification`: eligible Apple bootstrap exception, provider capability admission, setup lifecycle and embedding independence.

## Impact

Future implementation touches native provider registration, the existing liter-llm routing seam, trusted-host setup actions, persistent settings and their entity-backed console projection. It must preserve tool governance, wire contracts and source-owned dependency pins. No new SDK dependency, runtime configuration, model, credential or process is changed by this documentation commit.

The canonical KBD waypoint was `rc5-agui-a2ui-1`, COMPLETE with no active phase, when inspected on 2026-11-15. This proposal does not reopen it or certify RC5 for Apple inference. A future implementation must establish a dedicated phase and full implementation inventory before its VLBV gates. Native task inventory showed UAR owners idle or not loaded; unrelated `.prometheus/knowledge/wiki` edits are excluded.

Implementation tracker: [UAR #519](https://github.com/Prometheus-AGS/universal-agent-runtime/issues/519); no additional issue is created by this change.
