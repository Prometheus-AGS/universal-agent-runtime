# Specify — Optimization engagement

## Capture context

1. **App profile**
   - Framework (Next, Vite, RN webview, etc.)
   - Average session length and data volume (rows, entities).

2. **Pain**
   - CPU (jank), network (duplicate fetches), memory (tab crash), or correctness (stale UI).

3. **Instrumentation**
   - React DevTools Profiler availability
   - Whether Web Vitals / custom metrics exist

4. **Scope**
   - Whole repo vs one feature folder
   - Time budget (audit-only vs audit + fixes)

## Output

```yaml
targets:
  - path: examples/vite-app/src
    reason: primary demo
pain: performance | memory | architecture
constraints:
  - no breaking API changes
```

## Next

`plan.md`
