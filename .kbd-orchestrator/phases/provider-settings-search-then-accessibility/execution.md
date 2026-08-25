# EXECUTION: provider-settings-search-then-accessibility

Project: universal-agent-runtime
Date: 2026-08-25
Selected backend: openspec
Dispatched to: Codex self-execution with two read-only critique subagents and fresh-context adversarial review
Backend rationale: Both ordered UI changes have complete, strictly valid OpenSpec contracts and focused local verification surfaces. OpenSpec preserves the B→A dependency and KBD remains the canonical progress ledger.
Backend entrypoint: apply `provider-model-search`, verify it, then apply `provider-settings-accessibility-dirty-state`
OpenSpec available: YES
Source plan: `.kbd-orchestrator/phases/provider-settings-search-then-accessibility/plan.md`

## EXECUTION SCOPE

- `provider-model-search`: adaptive searchable bounded model picker for inventories of eight or more.
- `provider-settings-accessibility-dirty-state`: programmatic labels, live state, dirty-draft protection, and responsive provider cards.

## DISPATCH CONTRACTS

- `provider-model-search` → Codex self-execution through its OpenSpec tasks. Isolated design critics remain read-only and do not share generation history.
- `provider-settings-accessibility-dirty-state` → Codex self-execution only after the first change passes its focused and completion gates. The final diff goes to fresh-context adversarial review.

## APPROVAL GATES

- The user explicitly fixed execution order as B then A.
- No destructive, external, or persistence-changing action is authorized or required.

## FALLBACK CONDITIONS

- If the installed Base UI Combobox cannot preserve bounded selection and tested keyboard behavior, stop rather than adding an unverified custom picker or dependency.
- If dirty-state protection requires changing persistence or introducing a discard workflow, stop; the approved scope is the existing draft cache plus honest refresh/unload protection.

## VERIFICATION REQUIREMENTS

- Focused provider panel and affected shared primitive Vitest files after each edit.
- Frontend TypeScript, lint, and settings structure gates after each change.
- Strict OpenSpec validation for each change.
- Frontend build and full frontend tests before beginning A and again at phase completion.
- One post-edit Impeccable detector and fresh-context adversarial review before closeout.

## PROGRESS LEDGER

- [COMPLETE] `provider-model-search` — Codex
- [COMPLETE] `provider-settings-accessibility-dirty-state` — Codex

## OBSERVED VERIFICATION

- Focused provider and command tests: 2 files, 15 tests passed.
- TypeScript, lint, settings structure gate, production build, and both strict OpenSpec validations passed.
- Impeccable post-edit detector returned `[]`.
- Final full frontend suite: 339 passed, 12 unrelated baseline failures (2 stale provider-store mocks and 10 A2UI Storybook schema failures).
- First accessibility adversarial pass blocked on this task checkbox and identified two concrete refinements: rejected-save handling and collision-free control IDs. Both were implemented and covered before the required rerun.
- The rerun passed with no critical findings. Its in-scope UI warning and suggestions were applied: dirty drafts remain saveable during background refresh, empty disabled inventories are not marked invalid, and busy Refresh states have descriptions.

## OUTPUTS

- Updated provider settings panel and backward-compatible settings primitives.
- Focused regression evidence.
- Two verified and archived OpenSpec changes.
- Impeccable critique/detector and adversarial-review artifacts.

## BLOCKERS

- None.

## REFLECTION HANDOFF

Reflect must compare the shipped 7/8 picker threshold, actual accessibility associations, refresh/unload protection, responsive behavior, and observed verification output against both OpenSpec task lists. It must lead with any delta and disclose any full-suite baseline failure.

## EXECUTION READY
