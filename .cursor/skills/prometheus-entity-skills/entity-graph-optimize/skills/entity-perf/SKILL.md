---
name: entity-perf
description: >
  Slash command /entity-perf — analyze re-render patterns, Zustand selector granularity, duplicate
  fetches (dedupe keys), realtime flush settings, and list virtualization opportunities for
  @prometheus-ags/prometheus-entity-management apps.
---

# /entity-perf

## Invoke

Run **performance-analyzer** agent (`agents/performance-analyzer.md`).

## Steps

1. Capture baseline: React Profiler short session + network waterfall for primary user journey.
2. Identify top components by render time / commit count.
3. Trace each to data source:
   - Library hook vs custom `useStore`
   - `queryKey` stability
   - Realtime volume
4. Apply fixes from `prompts/execute.md` selectively.
5. Re-profile; document delta.

## Common wins

- Replace wide selectors with `useEntity` / narrow `useStore` + `useCallback`
- `useMemo` on `queryKey` and filter objects
- `RealtimeManager({ flushInterval: 16 })`

## References

- `../references/perf-patterns.md`
- Library: `src/engine.ts` (dedupe, subscribers)
