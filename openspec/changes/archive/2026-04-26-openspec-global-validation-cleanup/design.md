## Context

The `runtime-console-validation-hardening` KBD phase is blocked from final closure because the active `implement-opencode-suggestions` OpenSpec change does not pass strict validation. The failing deltas use lowercase `must`/`should` wording in requirement bodies and scenarios, while OpenSpec requires normative `SHALL` or `MUST` language in each requirement delta.

The affected deltas describe useful product requirements for accessibility, frontend resilience, offline access, SSE replay, storage health, Tauri sidecar execution, and tool analytics. The cleanup should preserve that intent and avoid runtime behavior changes.

## Goals

- Restore strict validation for `implement-opencode-suggestions`.
- Keep the product semantics of the affected requirement deltas intact.
- Record a narrow validation-hygiene capability so the KBD phase has inspectable evidence for this closure work.
- Verify active OpenSpec changes after the repair and preserve the results in KBD state.

## Non-Goals

- Do not change frontend runtime behavior, provider routing, API compatibility, memory behavior, or realtime event ingestion.
- Do not introduce new provider credentials or make live model calls.
- Do not archive or sync unrelated OpenSpec changes as part of this cleanup.

## Decisions

- Normalize the failing requirement and scenario text to `SHALL`/`MUST` rather than deleting the deltas, because the active change appears to carry valid product intent.
- Keep this cleanup as a separate OpenSpec change named `openspec-global-validation-cleanup` so the KBD execution log can trace why another active change was touched.
- Add a small `openspec-validation-hygiene` spec for future phase closure work instead of folding this process requirement into a product runtime spec.
- Run targeted validation for `implement-opencode-suggestions` before broader active-change validation, so any remaining failure can be attributed precisely.

## Risks

- Wording repairs could accidentally strengthen a scenario. The cleanup should only convert existing intent from lowercase or advisory language to normative parser-compatible wording.
- Broader active-change validation may still fail if another unrelated change is invalid. In that case, KBD state should record the unrelated blocker rather than claiming full closure.
- Main specs may remain invalid under `openspec validate --all`; this cleanup targets active change validation needed by the phase archive gate.
