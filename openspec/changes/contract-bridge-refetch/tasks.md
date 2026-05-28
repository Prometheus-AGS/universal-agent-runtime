## 1. Test

- [ ] 1.1 Author `frontend/src/lib/realtime/__tests__/use-graph-bridge.test.tsx`.
- [ ] 1.2 Single-key fires once on relevant mutation.
- [ ] 1.3 Single-key does NOT fire on unrelated-type mutation.
- [ ] 1.4 Multi-key fires once when any of the watched types mutates.

## 2. Sanity

- [ ] 2.1 Manually break the bridge `useEffect`; confirm the test fails.

## 3. Verification

- [ ] 3.1 `pnpm --filter ./frontend test` green.
