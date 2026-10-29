# Frontend dependency upgrade handoff — 2026-10-28

Writer: uar-console (frontend). Worktree `/Users/gqadonis/.claude/worktrees/main-locked-build`, branch `chore/deps-latest-compatible`. Nothing committed or pushed. **Nothing was built, tested, linted or type-checked** (operator: "No, gate only"). Only `pnpm install --lockfile-only --ignore-scripts`, `pnpm install --frozen-lockfile --ignore-scripts`, `pnpm peers check`, `pnpm outdated` and `npm view` were run. Evidence of those runs is at the end.

Release-age policy: pnpm 11 default (24 h). Versions younger than 24 h at selection (pglite 0.5.13, lucide-react 1.63.0, @hookform/resolvers 6.0.2, @types/node 26.6.6, chromatic 18.13.4, assistant-ui/react 0.15.32, react-resizable-panels 4.15.1) were not taken; the newest eligible release was. No exclusion was added for them.

## 1. PEM 4.3.0

- Submodule `frontend/packages/prometheus-entity-management`: `34a21156` (v4.1.1-6, pkg 4.2.0) -> `ea38b265a8a711555097813421227715755baae1` (tag `v4.3.0`, "chore(release): prepare 4.3.0"). The tag was missing from this worktree's submodule clone, so I ran `git fetch origin tag v4.3.0` (origin = the main checkout's submodule) and `git checkout v4.3.0` inside the submodule. Gitlink shows modified in `git status`; not staged.
- `frontend/package.json`: `@prometheus-ags/prometheus-entity-management` 4.2.0 -> 4.3.0; `@prometheus-ags/entity-graph-core` 3.0.3 -> 4.3.0. The console imports nothing from `entity-graph-core` directly (only 8 symbols from the alias package, all present in 4.3.0's exports; 4.2.0 -> 4.3.0 export lists of core and react are identical, diffed from the published tarballs).
- Lock resolves both from the registry (not `link:`): core 4.3.0, entity-graph-react 4.3.0 (via the alias, whose only dependency is `entity-graph-react ^4.3.0`), alias 4.3.0. This also fixes the old mismatch where entity-graph-react 4.2.0 (peer core ^4.2.0) ran against core 3.0.3.
- `frontend/pnpm-workspace.yaml` `minimumReleaseAgeExclude`: removed `entity-graph-core@3.0.3` and `prometheus-entity-management@3.0.3` (nothing resolves 3.0.3 any more); added `@prometheus-ags/entity-graph-core@4.3.0`, `@prometheus-ags/entity-graph-react@4.3.0` (new: it is the transitive package that holds the code), `@prometheus-ags/prometheus-entity-management@4.3.0`.
- 4.3.0's behavior changes (CHANGELOG) are all in `@prometheus-ags/a2ui-react` (idempotent AG-UI redelivery, enforced resource limits, `replace:true` no longer deletes surfaces). The console does not import that package, so no console code change.
- Auto-installed optional peer `@ag-ui/core` moves 0.0.57 -> 1.0.3 (PEM a2ui-react 4.3.0 accepts `>=0.0.59 <0.1.0 || ^1.0.0`). Console never imports `@ag-ui/*`.

### `@prometheus-ags/a2ui-react` name collision (reported, not resolved)
`frontend/packages/a2ui-react` is the private workspace package `@prometheus-ags/a2ui-react` 0.10.1 (wraps `@a2ui/react` 0.12.0; consumed by `a2ui-uar` as `workspace:*`). npm now also has PEM's `@prometheus-ags/a2ui-react` 4.3.0. Consequences:
- Today: none. The published package is not in the console's dependency graph (alias 4.3.0 -> entity-graph-react 4.3.0 -> no a2ui-react), and the console has zero imports of it.
- If anything ever adds a semver spec such as `@prometheus-ags/a2ui-react@^4.3.0`, pnpm will fetch the registry package (a `^4` range cannot match workspace 0.10.1), and two different packages with one name will exist in one workspace graph. `workspace:*` consumers keep the local one; filters like `pnpm --filter @prometheus-ags/a2ui-react` become ambiguous.
- If the PEM submodule's own `packages/a2ui-react` were ever added to `frontend/pnpm-workspace.yaml` (only core and react are listed), pnpm would reject duplicate workspace names.
- Pairing is compatible if it is ever adopted: PEM a2ui-react 4.3.0 pins `@a2ui/react` 0.12.0, `@a2ui/web_core` 0.12.0, `@a2ui/markdown-it` 0.2.0, `zod` 3.25.76 exactly — the same set the console uses.

## 2. Overrides in `frontend/pnpm-workspace.yaml`

| override | old | new | why |
|---|---|---|---|
| `vite` | `8.1.4` | `8.3.5` | Reason (one Vite 8 peer graph across frontend/A2UI/Storybook/Vitest, openspec 2026-08-07 tailwind4-css-first-tokens) still applies; only the value moves to the newest Vite 8, which every peer range accepts (@vitejs/plugin-react ^8, vite-plugin-svelte ^8, @tailwindcss/vite ^8, storybook react-vite ^8, vitest ^8, esbuild peer ^0.27 \|\| ^0.28). Manifests already said `^8.3.5`. |
| `valibot@<=1.4.1` | `1.4.2` | `1.5.0` | Dependabot floor kept (>=1.4.2). The exact `1.4.2` rewrote every valibot range, so the new `@valibot/to-json-schema` 1.8.0 (peer `^1.5.0`) produced an unmet peer. 1.5.0 is the current valibot, published 2026-09-09. |
| `@a2ui/markdown-it` | (none) | `0.2.0` | `@a2ui/react` 0.12.0 depends on `@a2ui/markdown-it ^0.2.0`; that now resolves to 0.2.2, whose peer is `@a2ui/web_core ^0.14.0` (unmet against held 0.12.0). 0.2.0 is PEM 4.3.0's exact validated pin. |
| `esbuild`, `dompurify`, `js-yaml`, `nanoid` | unchanged | unchanged | Security floors / versions.toml; still applicable. Lock holds dompurify 3.4.14 and 3.4.17, js-yaml 3.15.1 and 4.3.1, nanoid 3.3.20 (>= floors), esbuild 0.28.1. |

versions.toml pins verified in the lock: vitest, @vitest/{browser,browser-playwright,coverage-v8,ui,...} all 4.1.10; zod 3.25.76 (zod 4.6.7 appears only as a nested dependency of @assistant-ui/react, as before); loro-crdt 1.13.9. `a2ui-uar` had `vitest ^4.1.10`, which now floats to 4.1.11/4.1.12; changed to exact `4.1.10`.

## 3. Version tables (only rows that changed, or that are held / not at npm latest)

### frontend/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| @assistant-ui/react | dep | 0.15.30 | 0.15.31 | 0.15.32 | 0.15.32 was <24h old at selection; 0.15.31 chosen (exact pin style kept) |
| @assistant-ui/react-markdown | dep | 0.14.23 | 0.14.24 | 0.14.24 | latest |
| @electric-sql/pglite | dep | ^0.5.4 | ^0.5.12 | 0.5.13 | 0.5.13 is <24h old (release-age gate); 0.5.12 is newest eligible |
| @hookform/resolvers | dep | ^5.9.1 | ^6.0.1 | 6.0.2 | 6.0.2 is <24h old; 6.0.1 newest eligible |
| @prometheus-ags/entity-graph-core | dep | 3.0.3 | 4.3.0 | 4.3.0 | latest |
| @prometheus-ags/prometheus-entity-management | dep | 4.2.0 | 4.3.0 | 4.3.0 | latest |
| @tanstack/react-table | dep | ^8.21.3 | ^9.3.1 | 9.3.1 | latest |
| @tanstack/react-virtual | dep | 3.14.9 | 3.14.23 | 3.14.23 | latest |
| @tauri-apps/api | dep | ^2.12.1 | ^2.12.4 | 2.12.4 | latest |
| @tauri-apps/plugin-shell | dep | ^2.5.0 | ^2.6.0 | 2.6.0 | latest |
| immer | dep | ^11.1.4 | ^11.2.9 | 11.2.9 | latest |
| katex | dep | ^0.19.0 | ^0.19.1 | 0.19.1 | latest |
| loro-crdt | dep | 1.13.9 | 1.13.9 | 1.17.1 | Held: versions.toml pin 1.13.9 (entity package tested peer) |
| lucide-react | dep | ^0.575.0 | ^1.62.0 | 1.63.0 | 1.63.0 is <24h old (release-age gate); 1.62.0 newest eligible |
| mermaid | dep | ^11.17.0 | ^12.2.0 | 12.2.0 | latest |
| react | dep | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| react-dom | dep | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| react-hook-form | dep | ^7.90.0 | ^7.91.0 | 7.91.0 | latest |
| react-resizable-panels | dep | ^4 | ^4 | 4.15.1 | spec ^4 unchanged; lock resolves 4.15.0 (4.15.1 <24h old) |
| react-router | dep | ^8.5.0 | ^8.5.1 | 8.5.1 | latest |
| tailwind-merge | dep | ^2.6.0 | ^3.7.0 | 3.7.0 | latest |
| zod | dep | 3.25.76 | 3.25.76 | 4.6.7 | Held: versions.toml pin 3.25.76 (@a2ui/web_core Zod 3 boundary) |
| zustand | dep | ^5.0.11 | ^5.0.16 | 5.0.16 | latest |
| @chromatic-com/playwright | dev | ^0.14.12 | ^1.0.0 | 1.0.0 | latest |
| @playwright/test | dev | ^1.62.1 | ^1.64.1 | 1.64.1 | latest |
| @storybook/addon-a11y | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| @storybook/addon-docs | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| @storybook/addon-mcp | dev | ^0.7.0 | ^10.6.5 | 10.6.5 | latest |
| @storybook/addon-vitest | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| @storybook/react-vite | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| @testing-library/react | dev | ^16.3.2 | ^16.3.3 | 16.3.3 | latest |
| @testing-library/user-event | dev | ^14.6.1 | ^14.6.8 | 14.6.8 | latest |
| @types/node | dev | ^26.6.3 | ^26.6.5 | 26.6.6 | 26.6.6 is <24h old; 26.6.5 newest eligible |
| @types/react | dev | ^19.2.14 | ^19.3.0 | 19.3.0 | latest |
| @types/react-dom | dev | ^19.2.3 | ^19.3.0 | 19.3.0 | latest |
| @vitejs/plugin-react | dev | ^6.1.1 | ^6.1.2 | 6.1.2 | latest |
| @vitest/browser-playwright | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin |
| @vitest/coverage-v8 | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin |
| @vitest/ui | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin |
| axe-core | dev | 4.13.0 | 4.14.0 | 4.14.0 | latest |
| chromatic | dev | ^18.13.1 | ^18.13.3 | 18.13.4 | 18.13.4 is <24h old; 18.13.3 newest eligible |
| eslint | dev | ^10.7.0 | ^10.13.0 | 10.13.0 | latest |
| eslint-plugin-react-hooks | dev | ^7.0.1 | ^7.1.1 | 7.1.1 | latest |
| eslint-plugin-react-refresh | dev | ^0.5.3 | ^0.5.7 | 0.5.7 | latest |
| eslint-plugin-storybook | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| eslint-plugin-unicorn | dev | 73.0.0 | 79.0.0 | 79.0.0 | latest |
| globals | dev | ^17.9.0 | ^17.13.0 | 17.13.0 | latest |
| happy-dom | dev | ^20.14.5 | ^21.0.4 | 21.0.4 | latest |
| storybook | dev | ^10.6.3 | ^10.6.5 | 10.6.5 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |
| typescript-eslint | dev | ^8.66.0 | ^8.72.1 | 8.72.1 | latest |
| vitest | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin (Vitest set 4.1.10) |

### frontend/packages/a2ui-core/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| @a2ui/web_core | dep | 0.12.0 | 0.12.0 | 0.14.0 | Held 0.12.0: PEM 4.3.0 a2ui-react pins @a2ui/web_core, @a2ui/react exactly 0.12.0 (I19 pairing) |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |

### frontend/packages/a2ui-inspector/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| zustand | dep | ^5.0.11 | ^5.0.16 | 5.0.16 | latest |
| @testing-library/react | dev | ^16.3.2 | ^16.3.3 | 16.3.3 | latest |
| @testing-library/svelte | dev | ^5.3.1 | ^5.4.2 | 5.4.2 | latest |
| @types/react | dev | ^19.2.14 | ^19.3.0 | 19.3.0 | latest |
| @types/react-dom | dev | ^19.2.3 | ^19.3.0 | 19.3.0 | latest |
| @vitejs/plugin-react | dev | ^6.1.1 | ^6.1.2 | 6.1.2 | latest |
| eslint | dev | ^10.7.0 | ^10.13.0 | 10.13.0 | latest |
| eslint-plugin-react-hooks | dev | ^7.0.1 | ^7.1.1 | 7.1.1 | latest |
| globals | dev | ^17.9.0 | ^17.13.0 | 17.13.0 | latest |
| happy-dom | dev | ^20.14.5 | ^21.0.4 | 21.0.4 | latest |
| react | dev | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| react-dom | dev | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| svelte | dev | 5.57.1 | 5.57.3 | 5.57.3 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |
| typescript-eslint | dev | ^8.66.0 | ^8.72.1 | 8.72.1 | latest |
| vitest | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin (Vitest set 4.1.10) |

### frontend/packages/a2ui-lit/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| @types/node | dev | ^26.6.3 | ^26.6.5 | 26.6.6 | 26.6.6 is <24h old; 26.6.5 newest eligible |
| eslint | dev | ^10.7.0 | ^10.13.0 | 10.13.0 | latest |
| globals | dev | ^17.9.0 | ^17.13.0 | 17.13.0 | latest |
| happy-dom | dev | ^20.14.5 | ^21.0.4 | 21.0.4 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |
| typescript-eslint | dev | ^8.66.0 | ^8.72.1 | 8.72.1 | latest |
| vitest | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin (Vitest set 4.1.10) |

### frontend/packages/a2ui-react/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| @a2ui/react | dep | 0.12.0 | 0.12.0 | 0.14.0 | Held 0.12.0: same PEM pairing |
| @types/react | dev | ^19.2.15 | ^19.3.0 | 19.3.0 | latest |
| @types/react-dom | dev | ^19.2.3 | ^19.3.0 | 19.3.0 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |

### frontend/packages/a2ui-svelte/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| svelte | dep | 5.57.1 | 5.57.3 | 5.57.3 | latest |
| @testing-library/svelte | dev | ^5.3.1 | ^5.4.2 | 5.4.2 | latest |
| @types/node | dev | ^26.6.3 | ^26.6.5 | 26.6.6 | 26.6.6 is <24h old; 26.6.5 newest eligible |
| eslint | dev | ^10.7.0 | ^10.13.0 | 10.13.0 | latest |
| globals | dev | ^17.9.0 | ^17.13.0 | 17.13.0 | latest |
| happy-dom | dev | ^20.14.5 | ^21.0.4 | 21.0.4 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |
| typescript-eslint | dev | ^8.66.0 | ^8.72.1 | 8.72.1 | latest |
| vitest | dev | 4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin (Vitest set 4.1.10) |

### frontend/packages/a2ui-uar/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| i18next | dep | 26.5.4 | 26.5.11 | 26.5.11 | latest |
| lucide-react | dep | ^0.575.0 | ^1.62.0 | 1.63.0 | 1.63.0 is <24h old (release-age gate); 1.62.0 newest eligible |
| motion | dep | 13.5.0 | 14.7.1 | 14.7.1 | latest |
| react-aria-components | dep | ^1.21.1 | ^1.22.1 | 1.22.1 | latest |
| tailwind-merge | dep | ^2.6.0 | ^3.7.0 | 3.7.0 | latest |
| zod | dep | 3.25.76 | 3.25.76 | 4.6.7 | Held: versions.toml pin 3.25.76 (@a2ui/web_core Zod 3 boundary) |
| @testing-library/react | dev | ^16.3.2 | ^16.3.3 | 16.3.3 | latest |
| @testing-library/user-event | dev | ^14.6.1 | ^14.6.8 | 14.6.8 | latest |
| @types/react | dev | ^19.2.15 | ^19.3.0 | 19.3.0 | latest |
| @types/react-dom | dev | ^19.2.3 | ^19.3.0 | 19.3.0 | latest |
| @vitejs/plugin-react | dev | ^6.1.1 | ^6.1.2 | 6.1.2 | latest |
| axe-core | dev | 4.13.0 | 4.14.0 | 4.14.0 | latest |
| eslint | dev | ^10.7.0 | ^10.13.0 | 10.13.0 | latest |
| globals | dev | ^17.9.0 | ^17.13.0 | 17.13.0 | latest |
| happy-dom | dev | ^20.14.5 | ^21.0.4 | 21.0.4 | latest |
| react | dev | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| react-dom | dev | ^19.2.8 | ^19.3.1 | 19.3.1 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |
| typescript-eslint | dev | ^8.66.0 | ^8.72.1 | 8.72.1 | latest |
| vitest | dev | ^4.1.10 | 4.1.10 | 5.0.5 | Held: versions.toml pin (Vitest set 4.1.10) |

### frontend/packages/uar-ui/package.json

| package | section | old | new | npm latest | note |
|---|---|---|---|---|---|
| tailwind-merge | dep | ^2.6.0 | ^3.7.0 | 3.7.0 | latest |
| @types/react | dev | ^19.2.14 | ^19.3.0 | 19.3.0 | latest |
| @types/react-dom | dev | ^19.2.3 | ^19.3.0 | 19.3.0 | latest |
| typescript | dev | ^5.9.3 | ^6.0.3 | 7.0.2 | Held below 7: typescript-eslint 8.72.1 peer typescript >=4.8.4 <6.1.0 (also svelte-check ^5||^6); 6.0.3 is newest in range |


## 4. Code migrations

| file | change | breaking change answered | source |
|---|---|---|---|
| `frontend/src/shared/markdown/blocks/mermaid-block.tsx` | `mermaid.initialize` gains `layout: "dagre"`, `look: "classic"` | Mermaid 12.0.0: default layout becomes ELK and the default look/theme change; v11 rendering is restored with exactly these options. Keeps the lazy mermaid chunk free of the new default ELK path. `theme: "base"`, `securityLevel: "strict"`, `secure`, `suppressErrorRendering`, `render()` shape are unchanged. Test uses `expect.objectContaining`, so it still matches. `layout`/`look` verified in installed `mermaid/dist/config.type.d.ts`. | https://github.com/mermaid-js/mermaid/releases/tag/mermaid%4012.0.0 |
| `frontend/tsconfig.json` | remove `baseUrl: "."` (kept `paths` `@/*` -> `./src/*`, which resolves relative to the tsconfig); add `"types": ["*"]`, `"noUncheckedSideEffectImports": false` | TypeScript 6.0: `baseUrl` deprecated (TS5101 error); `types` defaults to `[]`; `noUncheckedSideEffectImports` defaults to true (`src/main.tsx` side-effect imports `@fontsource-variable/*`, which TS cannot resolve to a TS file). `"types": ["*"]` restores the 5.9 behavior (verified in installed `typescript/lib/_tsc.js`, `options.types` with `"*"`). | https://devblogs.microsoft.com/typescript/announcing-typescript-6-0/ |
| `frontend/tsconfig.node.json` | add `"types": ["*"]` | same `types` default (`vite.config.ts`, `build/*.ts` use Node globals) | same |
| `frontend/packages/a2ui-core`, `a2ui-react`, `a2ui-lit`, `a2ui-svelte` `tsconfig.json` | add `"types": ["*"]` | same; these packages relied on `@types/*` auto-inclusion (including `@types/node` found by walking up to `frontend/node_modules/@types`) | same |
| `frontend/packages/a2ui-uar`, `a2ui-inspector` `tsconfig.json` | add `"types": ["*"]` and `"noUncheckedSideEffectImports": false` | same, plus `import "../styles.css"` side-effect imports | same |
| `uar-ui/tsconfig.json` | unchanged | already lists `types` explicitly; no `baseUrl` | — |

Upgrades checked and needing **no** code change:
- `lucide-react` 0.575 -> 1.62: v1 removes brand icons (Chromium, Codepen, Codesandbox, Dribbble, Facebook, Figma, Framer, Github, Gitlab, Instagram, LinkedIn, Pocket, RailSymbol, Slack; https://lucide.dev/guide/react/migration). Every one of the 334 distinct named imports across `src` and `packages/*/src` was checked against the 1.62.0 `.d.ts` export list; all exist (`LucideIcon` type also present).
- `tailwind-merge` 2 -> 3: v3 targets Tailwind 4 (the console is on 4.3.3). The three `cn()` helpers only call `twMerge(clsx(...))`; no custom config.
- `motion` 13.5 -> 14.7.1: 14.0.0 only removed internal APIs (https://github.com/motiondivision/motion/blob/main/CHANGELOG.md); code uses `AnimatePresence`, `MotionConfig`, `motion` from `motion/react` only.
- `happy-dom` 20 -> 21: 21.0.0 disables CSS-file and iframe page loading by default (SSRF hardening; https://github.com/capricorn86/happy-dom/releases/tag/v21.0.0). No test depends on loaded stylesheets/iframes (no happy-dom settings in any vitest config; the iframe test in `markdown-bubble.test.tsx` asserts sanitizer removal).
- `@chromatic-com/playwright` 0.14 -> 1.0.0: 0.15.0 made it ESM-only (Node >= 22), 1.0.0 is the stable release (https://github.com/chromaui/chromatic-e2e/releases). `frontend/package.json` is `"type": "module"`; Node 24 in use.
- `eslint-plugin-unicorn` 73 -> 79: the config uses only `unicorn/filename-case` with `{case: "kebabCase"}`; schema unchanged in 79 (read `rules/filename-case.js`). No preset is enabled, so new rules/breaking option changes (76, 77, 79) do not apply.
- `@storybook/addon-mcp` 0.7.0 -> 10.6.5: the addon now ships from the Storybook monorepo with Storybook's version number; peers `storybook ^10.6.5`, `@storybook/addon-vitest ^10.6.5` are met. `.storybook/main.ts` addon string unchanged.
- `@hookform/resolvers` 5 -> 6 and `@tanstack/react-table` 8 -> 9: **neither is imported anywhere in `frontend/src`, packages or e2e** (react-hook-form is used by `src/components/ui/form.tsx`; resolvers and react-table are dead dependencies). Versions bumped to the latest; no migration needed. Consider removing them in a separate change.
- `react` 19.3.1, `@types/react(-dom)` 19.3.0, `react-router` 8.5.1: within every peer range (PEM `>=19 <20`, react-router `>=19.2.7`).
- `@electric-sql/pglite` ^0.5.4 -> ^0.5.12: see risks.

## 5. Majors / newer versions reported but not taken

| package | newest | held at | reason |
|---|---|---|---|
| typescript | 7.0.2 | 6.0.3 | typescript-eslint 8.72.1 peer `>=4.8.4 <6.1.0`; svelte-check 4.7.7 peer `^5 \|\| ^6`. TS 7 is excluded by peers. |
| vitest, @vitest/browser-playwright, @vitest/coverage-v8, @vitest/ui | 5.0.5 | 4.1.10 | versions.toml pin (whole set). `@storybook/addon-vitest` 10.6.5 accepts 4 or 5, so only the pin blocks it. |
| zod | 4.6.7 | 3.25.76 | versions.toml pin; `@a2ui/web_core` 0.12 Zod 3 schema objects (PEM a2ui-react also pins zod 3.25.76). |
| loro-crdt | 1.17.1 | 1.13.9 | versions.toml pin; PEM peer is `>=1.0.0`, so the pin, not PEM, holds it. |
| @a2ui/web_core, @a2ui/react | 0.14.0 | 0.12.0 | PEM 4.3.0 `a2ui-react` pins both exactly to 0.12.0 (scenario I19); 0.14.0 `@a2ui/react` also needs `@a2ui/markdown-it` 0.2.2. Not attempted. |
| @assistant-ui/react, pglite, lucide-react, @hookform/resolvers, @types/node, chromatic, react-resizable-panels | 1 patch newer each | see table | younger than 24 h; will be taken by a plain `pnpm update` tomorrow. |

## 6. Files changed (all under `frontend/` except the submodule pointer being inside it)

- `frontend/package.json`, `frontend/pnpm-workspace.yaml`, `frontend/pnpm-lock.yaml` (fully regenerated; old lock backed up outside the repo)
- `frontend/packages/{a2ui-core,a2ui-inspector,a2ui-lit,a2ui-react,a2ui-svelte,a2ui-uar,uar-ui}/package.json`
- `frontend/tsconfig.json`, `frontend/tsconfig.node.json`, `frontend/packages/{a2ui-core,a2ui-inspector,a2ui-lit,a2ui-react,a2ui-svelte,a2ui-uar}/tsconfig.json`
- `frontend/src/shared/markdown/blocks/mermaid-block.tsx`
- submodule gitlink `frontend/packages/prometheus-entity-management` -> `ea38b265a8a711555097813421227715755baae1`
- Not touched: `versions.toml`, vendored code, submodule contents, Rust files, root `pnpm-workspace.yaml` / `pnpm-lock.yaml`.

## 7. Needs the lead (outside my scope)

1. **Root workspace is now out of sync.** `/pnpm-workspace.yaml` includes `frontend`, `frontend/packages/*` and the PEM packages, so root `pnpm-lock.yaml` carries the frontend importers. Per gotchas.md (2026-08-24) both lock scopes must be reconciled. Root `pnpm-workspace.yaml` still has `vite: 8.1.4`, `valibot@<=1.4.1: 1.4.2`, no `@a2ui/markdown-it` override, and the 3.0.3 `minimumReleaseAgeExclude` entries. Apply the same four edits from section 2, then `pnpm install --lockfile-only --ignore-scripts` at the repo root and verify `--frozen-lockfile`. I did not touch root files.
2. Gate must run `pnpm exec playwright install` for the new Playwright 1.64.1 browser builds before e2e/a11y/performance gates.

## 8. Risks and unverified (everything is unbuilt)

- No `tsc`, `vitest`, `eslint`, Storybook, `vite build` or Playwright was run. Every statement above is from changelogs, registry metadata and installed package sources/types.
- TypeScript 6.0 is the riskiest change: tsconfig edits are parity-preserving (`types: ["*"]`), but TS 6 also changed inference/lib typings that only `tsc -b` can reveal. Fast revert: `typescript` back to `^5.9.3` in all 8 manifests, restore `baseUrl` and drop the `types`/`noUncheckedSideEffectImports` lines.
- `eslint-plugin-react-hooks` 7.0.1 -> 7.1.1 (root) and `eslint` 10.13 / `typescript-eslint` 8.72.1 may report new violations.
- `@electric-sql/pglite` 0.5.4 -> 0.5.12: 0.5.9 moved to PostgreSQL 18.6 and 0.5.12 changed initdb to `TZ=UTC`. The committed `src/platform/pglite/pglite-seed-v*.tar.gz` was built with the old runtime. `pglite:seed:check` hashes migrations, not the PGlite version, but loading the old data directory under the new runtime is unverified; regenerate the seed (`pnpm pglite:seed`) if the gate shows a restore problem.
- `vite.config.ts` chunk rule `vendor-entities` matches `node_modules/@prometheus-ags/(prometheus-entity-management|entity-graph-core)/` only. The entity React code lives in `entity-graph-react` (since 4.2.0 aliasing), which that rule does not match. This predates my change (same with 4.2.0); I left it. No budget script references `vendor-entities`.
- The PEM workspace importers inside the submodule (entity-graph-react) still resolve React 19.2.8 (their own `devDependencies` pin), so the lock holds React 19.2.8 and 19.3.1. The console resolves 19.3.1 only; submodule code is out of scope.
- Peer check clean but `pnpm install` printed deprecation warnings for `@opentelemetry/sdk-trace-base@2.12.0`, `@testing-library/jest-dom@6.10.0` (transitive) and `katex@0.18.11` (mermaid 12 depends on katex ^0.18, the console on ^0.19; two katex copies are expected).

## 9. Commands run and results

- `pnpm install --lockfile-only --ignore-scripts` (frontend, lock deleted first): resolved 1368 packages, "Done in 12.1s".
- `pnpm peers check`: first run found 2 unmet peers (`valibot`, `@a2ui/web_core` via `@a2ui/markdown-it@0.2.2`); after the two overrides: "No peer dependency issues found".
- `pnpm install --frozen-lockfile --ignore-scripts` (frontend): passed, "Done in 28.1s" (this is the frozen install that origin/main fails).
- `pnpm outdated -r`: remaining entries are only the holds in section 5 and submodule-owned packages.
