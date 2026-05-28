## 1. Test scaffolding

- [ ] 1.1 Decide: extract `setDefault` to an exported helper (preferred) vs. drive through the rendered component.
- [ ] 1.2 If extracting, add the export to `providers-page.tsx` (named, side-effect-free).

## 2. Test body

- [ ] 2.1 `vi.mock("@/services/providers-api", () => ({ setDefaultProvider: vi.fn().mockRejectedValue(new Error("forced")) }))`.
- [ ] 2.2 Seed graph with two providers + `ProviderMeta:current`.
- [ ] 2.3 Invoke `setDefault("p2")`.
- [ ] 2.4 `await waitFor(() => expect(setDefaultProvider).toHaveBeenCalled())`.
- [ ] 2.5 Assert `entities["ProviderMeta"]["current"].default_id === "p1"`.

## 3. Sanity

- [ ] 3.1 Manually remove the rollback path; confirm the test fails.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend test` green.
