# Deterministic verification — provider settings panel width responsiveness

Generated: 2026-08-27T07:53:29Z
Updated: 2026-08-27T08:47:07Z

## Accepted implementation

- `ai-settings-panels.tsx` adds `@container/provider-panel` to the incumbent provider-list body.
- Its incumbent field grid replaces only `lg:grid-cols-2` with `@xl/provider-panel:grid-cols-2`.
- No markup, copy, dependency, state, effect, subscription, provider data, service, store, transport, realtime, unload, or inline width behavior was added.

## Observed validation

- `pnpm typecheck`: exit 0.
- `pnpm lint`: exit 0.
- `pnpm -C frontend settings:structure`: exit 0; 11 modules, largest panel 599/600 lines, 29 keys preserved.
- Focused provider-panel Vitest: exit 0; 1 file and 11 tests passed.
- Strict OpenSpec validation: exit 0; change valid with a `frontend-configuration-surfaces` delta.
- `pnpm build`: exit 0; Vite transformed 8,396 modules and built in 2.76s, with four non-fatal PGlite direct-`eval` warnings.
- `pnpm test`: exit 1; 73/76 files and 350/362 tests passed. The 12 failures are confined to an omitted `updateProvider` mock and invalid A2UI/ChoicePicker story schemas outside this change.
- Old viewport-layout negative control: exit 1; the first boundary assertion received two tracks where one was required.
- Final strengthened authorized Tier 3 production-bundle Playwright: exit 0; 1/1 passed in 4.3s after scrollbar-safe measurement, strict exact-boundary coverage, stable grid targeting, per-state geometry/containment/focus coverage, reverse dirty-state crossing, fail-closed settings and persistence routing, pre/post-focus style comparison, Tab reachability from the document entry point, keyboard operation of every visible control, and viewport-contained geometry for both keyboard-opened portaled listboxes.
- `git diff --check` on the change-owned source, tests, OpenSpec, and KBD evidence: exit 0.

## Certified browser behavior

- Exactly one grid track at the measured 36rem provider-panel boundary minus one pixel.
- Exactly two tracks at the boundary, the boundary plus one pixel, and after restoring unconstrained desktop width.
- Page, provider card, and all six control boxes remain within their expected horizontal bounds.
- Mechanical focus order is Enable, Base URL, Protocol, API Key, reveal, and Default Model, with visible focus inside the scroll viewport; each control is also operated by keyboard.
- The dirty Base URL and Modified state survive both width transitions.
- No durable settings write is accepted.

## Independent review iteration 1

- Judge: `k3`; producer: `gpt-5`; isolation: REST gateway; cross-model check: verified distinct.
- Verdict: PASS; 0 critical, 4 warnings, 2 suggestions.
- Anti-theater gate: PASS at 0.017857.
- All six findings were addressed. Corrected-candidate re-review remains pending.

## Independent review iteration 2

- Judge: `k3`; producer: `gpt-5`; isolation: REST gateway; cross-model check: verified distinct.
- Verdict: PASS; 0 critical, 2 warnings, 2 suggestions.
- Anti-theater gate: PASS at 0.080357.
- All four findings were addressed. Final corrected-candidate re-review remains pending.

## Independent review iteration 3

- Judge: `k3`; producer: `gpt-5`; isolation: REST gateway; cross-model check: verified distinct.
- Verdict: PASS; 0 critical, 1 warning, 2 suggestions.
- Anti-theater gate: PASS at 0.0.
- All three findings were addressed. The final packet includes the registry delta and the two added browser proofs.

## Independent review iteration 4

- Judge: `k3`; producer: `gpt-5`; isolation: REST gateway; cross-model check: verified distinct.
- Verdict: PASS; 0 critical, 4 warnings, 1 suggestion.
- Anti-theater gate: PASS at 0.0.
- All five findings were addressed: persistence writes fail closed, the exact threshold is strict, all six controls are keyboard-operated, body `tabindex` is restored, and registry finalization is deferred until convergence as allowed by the established in-flight shape.

## Independent review iteration 5

- Judge: `k3`; producer: `gpt-5`; isolation: REST gateway; cross-model check: verified distinct.
- Verdict: PASS; 0 critical, 2 warnings, 2 suggestions.
- Anti-theater gate: PASS at 0.0.
- The containment warning is closed by the installed `SelectPrimitive.Portal` implementation plus passing popup viewport-geometry assertions. The artifact was finalized in iteration 5, so no sixth refinement cycle was started.
- The exact-boundary float and preview-status findings were suggestions. The current measured 16px root produces an exact integer 36rem boundary, and the manifest records that preview is not required with zero preview runs.

## Known limitations

- The first default-development-server run failed before rendering because Vite's optimizer could not resolve the entity package's optional `loro-crdt` peer from the application root. No dependency or Vite configuration workaround was added; certification used the passing Tier 2 production bundle.
- The full frontend suite remains non-green on 12 unrelated worktree-baseline failures. No baseline repair is part of this change.
- The installed artifact-refiner adapter omits the canonical prompts and schemas it references. Validation uses the installed `/refine-validate` contract, the repository's established persisted format, and the archived canonical schema assets as an explicitly disclosed fallback.
