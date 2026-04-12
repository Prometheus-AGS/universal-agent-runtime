# Reflect — Measure improvement

## Before/after

- Capture React Profiler flamegraph for representative interaction (sort column, open sheet, type in inline edit).
- Record network panel duplicate requests (should collapse with dedupe keys).

## Regression checks

- Cross-view update still works (edit in sheet → table row updates).
- CRUD cascade still refreshes related lists when FK changes.
- Realtime updates still arrive (subscription smoke test).

## Sign-off

- [ ] No new `useGraphStore` imports in components
- [ ] Typecheck green
- [ ] Documented any intentional `flushInterval: 0`

## Next

`persist.md`
