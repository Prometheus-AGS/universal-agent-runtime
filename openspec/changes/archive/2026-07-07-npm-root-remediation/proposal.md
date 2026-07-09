## Why

The root `package.json`/`package-lock.json` (npm-managed dev tooling only —
`playwright`, `eslint`, `prettier`, `tailwindcss`, `typescript`,
`monocart-coverage-reports`, `@tauri-apps/cli`; the actual frontend app is
built via `pnpm -C frontend`) carries 15 live `npm audit` findings (11
moderate, 4 high), all with a semver-compatible fix available — including
Dependabot's original alerts plus the assessment's 2 net-new findings
(`ajv`, `brace-expansion`) and extra GHSA IDs on already-flagged packages
(`minimatch`, `picomatch` each have 2-3 distinct advisories).

## What Changes

- Ran `npm audit fix` (no `--force`) — a live re-check confirmed all 15
  findings resolve within existing semver ranges, no major-version/breaking
  bump required for any of them:
  - `ajv` (<6.14.0), `brace-expansion` (<1.1.13 / 2.0.0-2.0.2), `js-yaml`
    (4.0.0-4.1.1), `uuid` (<11.1.1), `dompurify` (≤3.4.10) — direct
    moderate advisories, each with a compatible patched version.
  - `flatted`, `lodash-es`, `minimatch`, `picomatch` — high-severity
    (ReDoS / method injection), each with a compatible patched version.
  - `chevrotain`/`@chevrotain/gast`/`@chevrotain/cst-dts-gen`/`langium`/
    `@mermaid-js/parser`/`mermaid` — a single vulnerable-`lodash-es`
    chain surfaced through Mermaid's parser toolchain (pulled in
    transitively, likely via a docs/diagram-rendering dev dependency);
    resolved by the same `lodash-es` bump propagating through the chain.
- No `package.json` direct-dependency version ranges required editing —
  `npm audit fix` resolved everything within already-declared ranges
  (`package-lock.json`-only diff).

## Capabilities

### New Capabilities

None — see `Modified Capabilities` below.

### Modified Capabilities

- `dependency-security-posture`: adds the "npm Semver-Compatible Fix
  Application" requirement (apply `npm audit fix` without `--force` when
  every finding resolves within existing ranges; evaluate `--force`
  candidates individually rather than blanket-applying). Otherwise no
  other spec-level requirement changes; this is a lockfile-only npm
  remediation, no application source changed.

## Impact

- **Affected code**: `package-lock.json` only (dev-tooling dependencies;
  root `package.json` has no runtime application code — actual frontend
  build/test/lint all delegate to `pnpm -C frontend`).
- **Runtime UX / provider compatibility / realtime state**: none — none of
  the 15 affected packages are runtime dependencies of the shipped
  application.
- **KBD workflow state**: `progress.json` for
  `uar-dependabot-remediation-2026-07` to be updated to DONE for this
  change once verified; this is Round 2's first of 2 changes.
