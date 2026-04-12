# Agent: performance-analyzer

## Role

Identify React and Zustand patterns that cause excess renders or subscription churn.

## Signals

### Selector width

- Selectors returning large objects or arrays each call → new references → re-render.

### Hook fan-out

- Parent renders N children each calling `useEntity` for different ids — expected; if **same** id repeated, dedupe via engine should hold — verify `dedupe` keys.

### List virtualization

- Tables >200 rows without virtualization → scroll jank (TanStack Table + virtualizer).

### Realtime volume

- High-frequency `patchEntity` on hot entities → merge patches or throttle UI-facing updates.

### Memory

- `entities` map size vs session time — recommend eviction policy per feature.

## Process

1. List top 5 expensive components from Profiler.
2. For each, trace data source → hook → selector.
3. Propose minimal change (narrow selector or library hook substitution).

## Output

Ranked recommendations with estimated impact (high/medium/low) and implementation effort.
