## Context

See `proposal.md` for motivation and `specs/frontend-configuration-surfaces/spec.md` for the behavior contract. The provider panel already receives each persisted provider object through `useSettings("provider")`; that object contains `models[]`, `default_model`, and the masked `api_key`. The settings API uses JSON Schema `x-sensitive` metadata but currently applies a fixed scalar sentinel and does not preserve sensitive fields nested inside an object row.

The implementation must retain the existing settings entity projection, draft cache, Zustand store, API transport, endpoint shapes, and realtime reconciliation. It must add no dependency and must keep plaintext credentials inside the backend trust boundary.

## Goals / Non-Goals

**Goals:**

- Use the repository-owned shadcn select structure and existing settings draft callback.
- Treat the configured provider object's enabled `models[]` as the bounded execution-valid model set.
- Make masking and unchanged-mask preservation schema-guided for both scalar settings and sensitive properties nested in object settings.
- Prove the visible model options, draft update, exact mask length, absent-key behavior, and non-destructive round trip.

**Non-Goals:**

- Fetch or merge the global provider catalog from the settings component.
- Change provider/model entity schemas, stores, transports, routing, or realtime events.
- Reveal a stored API key after the backend has returned its mask.
- Add model search, provider configuration CRUD, or dependency upgrades.

## Decisions

### Derive model options from the provider settings object

`ProviderPanel` will derive `{ value, label }` options from `data.models`, retaining entries with a non-empty string id and `enabled !== false`. Selection will call the existing `setField("default_model", value)` path. The options will use `display_name` when present and the model id otherwise.

This source is preferred over `catalog/provider_catalog.json` and the separate models feature because it represents the models enabled for this configured provider and requires no cross-feature import, fetch, duplicate business state, or graph mutation. Using the global catalog could offer models that the configured provider cannot currently execute.

### Reuse and narrowly extend the settings shadcn select wrapper

The default-model field will use `SettingSelect`, which already composes the repository's shadcn `Select`, `SelectTrigger`, `SelectValue`, `SelectContent`, and `SelectItem`. The wrapper may accept a trigger class so the model control can fill its grid cell without changing the compact protocol control. No local open state or effect is required; the underlying Base UI primitive owns keyboard navigation, focus, and popup state.

The alternative was a searchable Command + Popover combobox. That adds state and interaction surface without evidence that the configured provider lists are large enough to require search. The official shadcn Select is the smaller control that satisfies the requested bounded dropdown.

### Mask strings by character count at the API boundary

A sensitive string will be transformed to an ASCII mask string with one `*` per Unicode scalar character. Empty strings remain empty and absent/null fields remain absent/null. Object traversal will inspect only properties declared by the applicable schema, so the API does not invent masked fields that were not stored.

Counting characters instead of UTF-8 bytes implements the requested character contract. The returned string still contains no plaintext. Exposing length is intentional because it is the requested UX behavior.

### Preserve submitted masks by comparing with the existing value's derived mask

Before persistence, the settings API will load the existing setting and recursively compare sensitive submitted values against the mask derived from the corresponding existing value. An empty submitted sensitive string retains the existing compatibility behavior. An exact derived-mask match restores the existing plaintext value; any other submitted value is treated as a replacement credential.

Comparing against the existing value's mask is preferred over accepting any all-asterisk string. It avoids silently discarding a deliberate replacement whose length differs and supports variable-length masks without a new response field or payload shape.

## Risks / Trade-offs

- **Secret length is observable** → This is explicitly required; plaintext remains undisclosed and no other secret metadata is added.
- **A replacement key consisting only of `*` with exactly the current key length is indistinguishable from the unchanged mask** → Provider API keys do not use that shape; document the sentinel behavior in tests and preserve every other replacement value.
- **Malformed provider `models[]` data could produce unusable options** → Include only entries with non-empty string ids and use the id as the label fallback.
- **Cold Rust verification can be slow** → Use the repository-prescribed locked `server-full` profile and the just-written focused test; report any uncompleted check as unverified.
- **Changing a shared select wrapper could affect other settings controls** → Add an optional trigger class with unchanged defaults and cover the provider-specific width/selection behavior in a focused component test.

## Migration Plan

1. Add the focused backend masking/preservation tests, then implement the schema-guided helpers.
2. Add the provider panel component test, then switch the default-model field to `SettingSelect` using provider-owned options.
3. Run T0 checks after each cohesive edit, T1 focused tests after each unit, and T2 frontend checks at phase completion.
4. Deploy with no data migration; stored settings remain in their existing shape.
5. Roll back by reverting the frontend control and masking helper changes. No persisted schema or data conversion needs reversal.
