## Context

Wave 1 established CSS-first semantic colors in
`frontend/src/shared/theme/tokens.css`, while legacy HSL-channel call sites were
explicitly deferred. The C-05 plan estimated 30 non-admin occurrences; a fresh
literal census finds 29 across the six named files: 14 in `index.css`, three
in `enhanced-thread.tsx`, six in `error-bar.tsx`, and two each in
`loading-cursor.tsx`, `empty-frame.tsx`, and `KnowMeLogo.tsx`. The 307
occurrences under `frontend/src/admin/pages/` remain assigned to C-14a.

## Goals / Non-Goals

**Goals:**

- Make the measured C-05 call sites consume semantic `--color-*` values.
- Preserve every rendered color and alpha treatment across dark, light,
  high-contrast, and admin-terminal themes.
- Keep the migration boundary mechanically inspectable and keep the Flat 2.0
  baseline exact.

**Non-Goals:**

- Changing colors, layout, typography, motion, focus, component behavior, or
  runtime data flow.
- Editing `frontend/src/admin/pages/`, whose 307 occurrences belong to C-14a.
- Removing legacy HSL channel definitions before all deferred consumers move.

## Decisions

### Semantic values are the consumer contract

Opaque call sites use `var(--color-<role>)`. Admin shared components receive
complete-color semantic aliases inside the same terminal-theme selector that
owns their current channel variables. The legacy channels remain alongside
those aliases as compatibility input for C-14a.

Alternative: convert channel variables themselves to complete colors. Rejected
because it would break the deferred `hsl(var(--x))` admin-page consumers.

### Alpha is expressed from semantic colors

Call sites that previously used HSL alpha syntax use `color-mix(in srgb, ...,
transparent)` with the same percentage. This keeps the source color semantic
and the effective opacity equivalent without reconstructing a channel tuple.

Alternative: create one token per opacity. Rejected because these are local
alpha treatments rather than stable design roles.

### Scope is an explicit file set

A deterministic repository check owns the six C-05 files and rejects a
remaining `hsl(var(` or `hsla(var(` sequence regardless of case. A negative
fixture proves the case-insensitive branch. The gate recursively counts and reports
`frontend/src/admin/pages/` for migration visibility but does not pin that
future C-14a-owned set to a permanent CI constant.

The same check extracts every migrated `var(--color-*)` reference and requires
a matching definition in the shared token source or scoped stylesheet, so a
semantic-token typo cannot silently pass the syntax-only census.

The Flat 2.0 allowlist remains exact. The two error-bar diagnostics persist
because their prohibited borders are outside this token-only change, so their
snapshot text is refreshed rather than removed.

## Risks / Trade-offs

- **Risk:** Tailwind arbitrary-value parsing could reject a `color-mix` class.
  **Mitigation:** type/lint plus the CSS-first development compiler are cheap
  gates for this completed unit.
- **Risk:** A broad replacement could consume C-14a scope. **Mitigation:** edit
  only the seven measured files and assert deferred admin-page occurrences
  remain.
- **Risk:** Semantic aliases could be unavailable outside the terminal theme.
  **Mitigation:** only admin shared components consume those aliases, and the
  complete-color aliases are declared directly on the terminal-theme root.

## Migration Plan

1. Add the required admin semantic aliases.
2. Replace the 29 measured call sites without changing surrounding structure.
3. Refresh the two changed Flat 2.0 baseline records.
4. Add and run the scoped migration check, frontend cheap gates, and strict
   OpenSpec validation.
5. Record C-05 in canonical KBD state and archive the change.

Rollback is the inverse token substitution plus removal of the scoped aliases
and gate wiring; no persisted state or API contract is involved.

## Open Questions

None. The measured 29/30 drift is resolved by the current source census, and
the admin-page exclusion is binding in the phase plan.
