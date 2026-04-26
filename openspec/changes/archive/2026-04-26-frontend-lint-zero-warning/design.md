## Context

The frontend lint gate currently fails after the runtime console refactor. The failures are concentrated in three categories:

- unused variables and props in e2e/provider/chat surfaces;
- React hook violations from synchronous state updates inside effects;
- `react-refresh/only-export-components` warnings from files that export both components and reusable constants or helpers.

The gate matters because the runtime console depends on predictable React state transitions for live AG-UI/A2UI, provider status, and entity graph updates. Lint failures in selector, chat, and provider surfaces make later visual and realtime validation less trustworthy.

## Goals / Non-Goals

**Goals:**

- Make `bun run lint` exit successfully with no errors and no warnings.
- Preserve `bun run typecheck` success.
- Fix React hook lint issues by using derived state, initial state, async callbacks, or event handlers rather than disabling rules.
- Remove stale unused variables/props in tests and UI components.
- Keep frontend layering intact: components call hooks, hooks expose store state/actions, stores call services, and services own HTTP/SSE I/O.
- Keep runtime console behavior stable so subsequent visual and realtime tests validate the same product surface.
- Update KBD progress after the lint gate is implemented and verified.

**Non-Goals:**

- This change does not add new runtime console pages or provider capabilities.
- This change does not introduce new frontend dependencies.
- This change does not rewrite the chat or provider data model.
- This change does not resolve unrelated OpenSpec validation failures, Surreal Memory workflow mirror tests, or Moonshot credential status.

## Decisions

### Decision 1: Fix lint at source instead of weakening ESLint globally

The ESLint configuration should remain strict. Unused variables should be removed or used in assertions, React effect patterns should be refactored, and Fast Refresh warnings should be resolved by moving non-component exports into separate modules or documenting narrow local exceptions only if the component library pattern requires them.

Alternatives considered:

- Disable `react-hooks/set-state-in-effect`: rejected because the runtime console needs reliable live updates and this rule points at real cascade risks.
- Lower `react-refresh/only-export-components` to off globally: rejected because global suppression would hide future hot-reload boundary problems across the frontend.

### Decision 2: Prefer derived state over effect-synchronized state

Where state can be derived from props, route state, or fetched data, compute it with render-time expressions or `useMemo` rather than synchronizing it in an effect. This applies directly to agent capability lists and thread-dependent reset behavior.

Alternatives considered:

- Keep effect-synchronized state and add lint comments: rejected unless the component truly needs local editable state independent from props.
- Move everything into Zustand stores: rejected for this change because it would expand a lint fix into a data architecture refactor.

### Decision 3: Keep async fetch effects but avoid synchronous prefetch state churn where practical

For components that fetch models or agents on mount, loading state should be initialized to the loading value or updated inside the async workflow rather than forcing synchronous effect state updates solely to reset status. Error state should be set from async outcomes or derived from request state.

Alternatives considered:

- Adopt React Query in these components now: rejected because it is larger than the lint closure slice and could alter UX behavior.
- Move fetches to services/stores now: rejected unless a specific lint fix requires it; layering cleanup can be handled in a later hardening slice.

### Decision 4: Treat warnings as gate failures for this phase

The phase goal is zero-warning lint. Warnings must be resolved or converted into documented accepted exceptions in the ESLint config with a clear reason. The default path is to remove warnings.

Alternatives considered:

- Accept warnings if errors are fixed: rejected because the KBD assessment explicitly identified the repo's zero-warning policy as violated.

## Risks / Trade-offs

- [Risk] Refactoring effect-synchronized state may subtly change component behavior. -> Mitigation: keep edits minimal, run `bun run typecheck`, `bun run lint`, and targeted chat/provider smoke checks where feasible.
- [Risk] Splitting constants/helpers out of component files can create import churn. -> Mitigation: move only the exports that trigger Fast Refresh warnings and keep file names local to the existing component directories.
- [Risk] Some shadcn-style UI files intentionally export component variants from the same file. -> Mitigation: prefer established local patterns, but if a warning must remain by design, document a narrow rule exception rather than disabling the rule broadly.
- [Risk] Fixing lint could tempt broader chat/provider refactors. -> Mitigation: defer feature changes to the later runtime console visual and replay-test changes.

## Migration Plan

1. Re-run `bun run lint` and record the current failure list.
2. Fix unused variables and stale props first.
3. Refactor React effect state synchronization in the smallest safe way.
4. Resolve Fast Refresh warnings by moving non-component exports or adding narrow, justified configuration exceptions only where necessary.
5. Run `bun run lint` until it exits with zero errors and zero warnings.
6. Run `bun run typecheck` to verify no TypeScript regressions.
7. Update `.kbd-orchestrator/phases/runtime-console-validation-hardening/progress.json` from `proposal_complete` to the appropriate implementation/verification status.

Rollback strategy: revert the focused frontend lint edits and KBD progress update. No database, API, or backend migrations are involved.

## Open Questions

- Should shared shadcn-style variant exports be split into separate utility files now, or should the ESLint config allow known UI-library constant exports beyond `allowConstantExport`?
- Should `frontend/src/entities/sync.ts` remain as entity infrastructure with direct `fetch`/`EventSource`, or should that layering concern be addressed in a later hardening change?
