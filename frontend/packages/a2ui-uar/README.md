# `@prometheus-ags/a2ui-uar`

> **Current authority:** [A2UI product guide](/docs/product/a2ui). This private
> workspace package is the first-party React renderer; package-local checks do
> not transfer a claim to another runtime profile.

This is the A2UI renderer UAR product code imports. It builds on
[`@prometheus-ags/a2ui-core`](../a2ui-core), uses the UAR design tokens, and
keeps model-authored UI inside a validated declarative component catalog.
`@prometheus-ags/a2ui-react` is reference-only.

## Host surface

```tsx
import { UarSurface } from "@prometheus-ags/a2ui-uar";
import "@prometheus-ags/a2ui-uar/styles.css";

<UarSurface
  surface={surface}
  theme="dark"
  locale="en"
  direction="auto"
  onRetry={() => reloadSurface()}
/>
```

Supported host options include light, dark, and high-contrast themes; bundled
renderer strings for English, Spanish, Japanese, and Chinese; explicit or
locale-derived text direction; bounded transitions that honor reduced motion;
and surface-local retry/error handling. Agent-authored text is not translated.

## Catalog boundary

The protocol catalog contains `Text`, `Button`, `TextField`, `CheckBox`,
`ChoicePicker`, `Row`, `Column`, `Card`, and `Divider`. The UAR entity extension
catalog adds `EntityCard`, `EntityDiff`, `EntityStream`, `EntityApproval`,
`EntityToolProvider`, `EntityChat`, and `EntityCopilot`. Unknown types fail
closed rather than becoming HTML or JavaScript.

The package uses `web_core`'s `MessageProcessor`, `Catalog`, and
`GenericBinder` to resolve declarative data and actions. The React layer renders
the resulting models; it does not evaluate arbitrary markup.

## Local package checks

Run these from the repository root after package work is complete:

```bash
pnpm --filter @prometheus-ags/a2ui-uar typecheck
pnpm --filter @prometheus-ags/a2ui-uar lint
pnpm --filter @prometheus-ags/a2ui-uar test
pnpm --filter @prometheus-ags/a2ui-uar perf
```

The performance command is a package diagnostic. Product latency and profile
claims require the separately defined browser/runtime evidence. GitHub Actions
are deployment-only and do not run these checks.
