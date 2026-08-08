# Third-party Radix ownership

Last verified: 2026-08-08

UAR application source and direct `@radix-ui/*` declarations are retired. A small Radix graph remains through supported third-party packages:

- `@assistant-ui/react@0.14.26` depends on `radix-ui@^1.6.0` and focused Radix helper packages. Its chat primitives remain the binding owner of thread, composer, streaming, and message interaction behavior.
- `vaul@1.1.2` depends on `@radix-ui/react-dialog@^1.1.6` for the retained local Drawer wrapper.

`@prometheus-ags/prometheus-entity-management@3.0.0-alpha.0` does not declare a Radix dependency.

The package registry reported `@assistant-ui/react@0.15.10` as current during this audit, and that release still declares `radix-ui` plus focused Radix helpers. Assistant UI's component registry can generate Base UI-flavored application components when `components.json` uses a `base-*` style, but that does not remove the installed runtime package's internal dependencies. UAR therefore retains the pinned 0.14.26 runtime until a separately verified upgrade is required.

Reproduce the installed ownership graph with:

```bash
pnpm --filter uar-frontend why @radix-ui/react-dialog @radix-ui/react-tooltip
pnpm -C frontend why @radix-ui/react-dialog @radix-ui/react-tooltip
```

Application code must continue importing local wrappers or Assistant UI's supported public primitives; it must not import these transitive Radix packages directly.
