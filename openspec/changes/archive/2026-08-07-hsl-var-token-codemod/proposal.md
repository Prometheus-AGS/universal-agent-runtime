# Change: Replace non-admin HSL channel call sites with semantic tokens

## Why

The Tailwind 4 token foundation is live, but 29 measured non-admin call sites
still wrap legacy HSL-channel variables instead of consuming the stable
`--color-*` design-token contract. Converting this bounded set now completes
the Wave 2 mechanical sweep without duplicating the 307 admin-page occurrences
owned by C-14a.

## What Changes

- Replace the 29 measured non-admin `hsl(var(--x))` call sites in the shared
  stylesheet, assistant thread, admin shared components, and KnowMe logo with
  semantic `--color-*` token references.
- Add scoped semantic aliases for the admin-terminal channel variables used by
  shared admin components and preserve alpha treatments with semantic-color
  mixing.
- Refresh the exact Flat 2.0 allowlist entries whose source strings change;
  the underlying border findings remain deferred rather than being hidden.
- Add a deterministic scope gate that rejects any remaining legacy HSL-channel
  call site in the six-file C-05 set while proving `admin/pages/` stays deferred.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-design-system`: require migrated non-admin surfaces to consume
  semantic color tokens while preserving the explicitly staged admin-page
  boundary.

## Impact

- **Runtime UX:** colors, alpha, contrast, layout, focus, motion, and behavior
  remain visually equivalent; only token ownership changes.
- **Provider compatibility:** no provider API, model routing, or dependency
  changes.
- **Realtime state:** no store, service, event, or entity-graph changes.
- **Workflow:** C-05 must be verified, recorded through canonical KBD state,
  and archived before the next plan change begins.
