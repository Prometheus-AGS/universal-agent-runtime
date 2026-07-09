# Findings: npm-root-remediation

## Live audit vs. assessment-era snapshot

A fresh `npm audit --json` at execute time found **15 findings** (11
moderate, 4 high) — all with `fixAvailable: true` (a plain boolean, not a
semver-major object), meaning every one resolves within already-declared
`package.json` ranges. No `--force` was needed for any finding.

| Crate | Severity | Advisory | Disposition |
|---|---|---|---|
| `ajv` | moderate | `<6.14.0` | Fixed via `npm audit fix` |
| `brace-expansion` | moderate | `<1.1.13 \|\| >=2.0.0 <2.0.3` | Fixed via `npm audit fix` |
| `js-yaml` | moderate | `4.0.0-4.1.1` | Fixed via `npm audit fix` |
| `uuid` | moderate | `<11.1.1` (missing buffer bounds check) | Fixed via `npm audit fix` |
| `dompurify` | moderate | `<=3.4.10` | Fixed via `npm audit fix` |
| `flatted` | high | `<=3.4.1` | Fixed via `npm audit fix` |
| `lodash-es` | high | `<=4.17.23` | Fixed via `npm audit fix` |
| `minimatch` | high | `<=3.1.3 \|\| 9.0.0-9.0.6` (3 distinct ReDoS GHSA IDs) | Fixed via `npm audit fix` |
| `picomatch` | high | `<=2.3.1 \|\| 4.0.0-4.0.3` (2 distinct GHSA IDs) | Fixed via `npm audit fix` |
| `chevrotain`, `@chevrotain/gast`, `@chevrotain/cst-dts-gen` | moderate | `11.0.0-11.1.0` | Fixed — resolved by the `lodash-es` bump propagating through this chain |
| `langium` | moderate | `2.1.0-4.1.3` | Fixed — same chain (depends on `chevrotain`) |
| `@mermaid-js/parser` | moderate | `<=0.6.3` | Fixed — same chain (depends on `langium`) |
| `mermaid` | moderate | `11.0.0-alpha.1 - 11.14.0` | Fixed — same chain (depends on `@mermaid-js/parser`) |

The `chevrotain`→`langium`→`@mermaid-js/parser`→`mermaid` chain (5 crates,
1 finding count each in the raw audit output) all trace back to a single
vulnerable `lodash-es` resolution — not 5 independent issues. Pulled in
transitively via a dev-tooling dependency (likely `monocart-coverage-reports`,
which renders Mermaid diagrams in HTML coverage reports); not a direct
dependency of the shipped application.

## Verification

- `npm audit fix` (no `--force`): "found 0 vulnerabilities" after.
- `npm audit` re-run: confirmed 0 vulnerabilities.
- `package.json` diff: **empty** — no direct-dependency range edits
  required; `package-lock.json` only (2051 insertions, 1146 deletions).
- Root dev tools sanity-checked post-fix: `npx eslint --version` (9.39.2),
  `npx tsc --version` (5.9.3), `npx playwright --version` (1.57.0),
  `npx prettier --version` (3.7.4), `npx tailwindcss --version` (4.1.18) —
  all respond correctly.
- `bun run build` (== `pnpm -C frontend build`): succeeds, 5719 modules
  transformed, only pre-existing unrelated warnings (Tailwind ambiguous
  class names, PGlite Node-fs externals) — no new errors.
