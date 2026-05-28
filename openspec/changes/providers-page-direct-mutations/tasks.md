## 1. Mutations

- [x] 1.1 `configureProvider` → direct service call + post-success `loadProvidersIntoGraph()` refresh. Non-optimistic.
- [x] 1.2 `setDefault` → direct service call + optimistic `upsertEntity("ProviderMeta", "current", { default_id })` with rollback on failure.
- [x] 1.3 `removeProvider` → snapshot + direct service call + optimistic `removeEntity("Provider", id)` with re-upsert on failure.

## 2. Local state replaces store fields

- [x] 2.1 `const [saving, setSaving] = useState(false);`
- [x] 2.2 `const [removing, setRemoving] = useState<string | null>(null);`
- [x] 2.3 `const [error, setError] = useState<string | null>(null);`
- [x] 2.4 `clearError` → `() => setError(null)`.

## 3. Drop the legacy hook import

- [x] 3.1 Removed `useProvidersAdmin` import + call from `providers-page.tsx`.

## 4. Verification

- [x] 4.1 `pnpm --filter ./frontend build` clean — bundle hash `index-DZ4RVqVx.js`.
- [ ] 4.2 Manual: each mutation works — pending browser smoke.
- [ ] 4.3 Manual: `setDefault` rejection rolls back — pending.
- [ ] 4.4 Manual: `removeProvider` rejection re-upserts — pending.
