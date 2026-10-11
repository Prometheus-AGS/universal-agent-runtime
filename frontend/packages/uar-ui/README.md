# @prometheus-ags/uar-ui

Presentational component library for the UAR operator shell. It holds the restyled
primitives and the composition patterns (master–detail, wizard, inline confirm, …) that
the screens in `docs/design/` share. Plan and scope: `docs/design/react-implementation-plan.md`.

## Rules (enforced, not advisory)

| Rule | Enforced by |
|---|---|
| No `fetch`, services/API modules, Zustand, entity graph, Tauri or `@/` app imports; relative imports stay inside `src/` | `pnpm frontend:boundaries` (`library-*` rules) |
| Flat 2.0: no borders, rings, layout shadows, blur, gradients or `outline` variants | `pnpm -C frontend lint`, `pnpm frontend:style-gate` |
| kebab-case filenames, React hooks rules | `pnpm -C frontend lint` |
| Types | `pnpm -C frontend typecheck` (runs `tsc -p packages/uar-ui/tsconfig.json`) |
| Behaviour tests | `pnpm -C frontend test --project uar-ui` |
| Stories and a11y | Storybook (`pnpm -C frontend storybook`), stories under `src/**/*.stories.tsx` |

Styling uses Tailwind utilities and the app's tokens (`--color-*`, `--font-*`); the
package ships no tokens or global CSS of its own. Status is never shown by colour alone.

## Contents

- `primitives/status-chip.tsx` — `StatusChip` (tones: neutral, ok, warn, err, live, accent).
