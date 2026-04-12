# AGENTS.md — entity-graph-optimize

## When to activate

- Performance complaints (slow tables, janky typing, fan-out renders).
- Suspicion of architectural drift (components importing Zustand graph).
- Memory growth in long-lived SPAs.

## Agent roles

- **auditor.md** — Static compliance with CLAUDE.md, import rules, JSDoc on public hooks.
- **performance-analyzer.md** — Dynamic patterns: selectors, subscriptions, list churn.

## Outputs

- Prioritized issue list (P0–P3).
- Patch suggestions scoped per file.
- Verification commands: `pnpm run typecheck`, manual scenario list.

## Do not

- Recommend bypassing the graph for “speed” without measuring — usually increases inconsistency.
- Add automatic GC inside the library from an app audit without a core RFC (library change is separate).
