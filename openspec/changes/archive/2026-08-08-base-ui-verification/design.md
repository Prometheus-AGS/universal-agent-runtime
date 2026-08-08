## Context

The May verification proposal predates the C-10 shell, C-14 feature migration, and C-14c direct dependency retirement. The application shell command palette already uses Base UI Autocomplete, but the stable `components/ui/command.tsx` facade remains live in agent, model, skill, tool, and knowledge-base selectors. Installed `cmdk@1.1.1` directly depends on four Radix packages. Assistant UI 0.14.26 separately depends on `radix-ui`; registry metadata for current 0.15.10 still does so. Prometheus Entity Management 3.0.0-alpha.0 has no Radix dependency.

The active frontend uses `components.json` style `base-vega`, React 19, Base UI 1.6.0, and stable local wrapper imports. The full frontend/E2E matrix is the final C-14 wave gate. Unrelated dirty submodule, backend, and license paths are protected.

## Goals / Non-Goals

**Goals:**

- Remove the live `cmdk` dependency and its Radix ownership.
- Preserve the complete local `Command*` export surface and all current callers.
- Preserve filtering, pointer selection, Enter selection, focus, and repeated multi-add behavior.
- Document accepted Assistant UI transitives and prove PEM has none.
- Run the full frontend and browser verification matrix assigned to C-14d.

**Non-Goals:**

- Redesigning selectors or the established C-10 shell command palette.
- Upgrading Assistant UI while the current release line still declares Radix and no observed compatibility defect requires an upgrade.
- Removing Radix packages owned by supported third-party dependencies such as Assistant UI or `vaul`.
- Changing backend, provider, entity, persistence, or realtime contracts.

## Decisions

### Use Base UI Autocomplete behind the stable Command facade

Current Base UI guidance distinguishes a filterable selection control (`Combobox`) from a search/action widget (`Autocomplete`). These command wrappers activate actions and, in agent editing, remain open for repeated additions. Persisting a combobox selection into the query would change behavior. The adapter therefore uses an inline, always-open Autocomplete inside the existing Popover/Dialog hosts and translates Base UI item clicks—including Enter on a highlighted item—to the existing `onSelect` callback.

The original proposal's Combobox implementation detail is superseded; the operator decision requires Base UI ownership, not selection-form semantics.

### Preserve the wrapper API rather than rewrite consumers

`Command`, `CommandDialog`, `CommandInput`, `CommandList`, `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandSeparator`, and `CommandShortcut` remain exported. Group headings become Base UI GroupLabel parts and cmdk-specific state selectors become Base UI `data-highlighted` selectors. Existing feature imports do not change.

### Accept only demonstrated third-party Radix ownership

Assistant UI 0.14.26 and the current registry release 0.15.10 declare Radix dependencies, so an upgrade would not remove that ownership. PEM declares none. Root and nested `pnpm why` receipts document the retained graph; application source and direct dependency manifests remain Radix-free.

## Risks / Trade-offs

- [Static item filtering differs from cmdk] → Add focused facade tests for query filtering, empty state, pointer activation, and keyboard activation, then exercise real selectors in Playwright.
- [Selection fills the query and blocks repeated additions] → Use Autocomplete's no-selection model rather than Combobox.
- [External wrapper API drifts] → Retain every authored export and type the compatibility-only `onSelect` prop explicitly.
- [Third-party Radix is mistaken for application regression] → Retain exact installed/current registry dependency evidence and ownership notes.

## Migration Plan

1. Capture protected, dependency, export, caller, and backup-path baselines.
2. Add the Base UI adapter tests, replace the cmdk implementation, and remove cmdk through pnpm.
3. Run focused checks, then the C-14 wave full frontend/browser/build matrix.
4. Retain receipts, complete isolated review, transition canonical KBD, and archive.

Rollback is the ordinary source and lockfile diff; no durable data or external contract changes.

## Open Questions

None. Installed and current registry metadata settle the conditional dependency decisions.
