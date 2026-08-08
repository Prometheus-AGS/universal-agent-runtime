# Verification: tailwind4-css-first-tokens

Date: 2026-08-07
Phase change: C-02

## Summary

| Dimension | Status |
|---|---|
| Completeness | 9/9 implementation tasks complete; 4/4 requirements mapped |
| Correctness | 6/6 scenarios have implementation or executable assertion evidence |
| Coherence | Implementation follows the CSS-first staging design; no critical issue found |

## Acceptance evidence

1. `frontend/package.json` pins `tailwindcss` and `@tailwindcss/vite` to `4.3.3` and `tw-animate-css` to `1.4.0`; the legacy animation plugin and direct PostCSS toolchain declarations are removed. Both workspace roots pin Vite 8.1.4, and both maintained lockfiles contain no other Vite 8 resolution.
2. `frontend/vite.config.ts` registers `tailwindcss()`, the repository's tracked `frontend/vite.config.js` build artifact carries the same plugin, and `frontend/src/index.css` imports the CSS-first `frontend/src/shared/theme/tokens.css` source.
3. `tokens.css` defines complete-color surface, text, brand, status, run-phase, typography, radius, non-colliding easing, duration, animation, explicitly frontend-scoped source-scan, theme, and staged semantic-compatibility contracts.
4. `frontend/tailwind.config.ts` and `frontend/postcss.config.js` are deleted. `frontend/components.json` uses the Tailwind 4 empty config path, and both Storybook workflow filters name the token source.
5. No live workflow or top-level frontend configuration still references either deleted file.
6. The implementation deliberately leaves legacy HSL-channel consumers for C-05 and C-14a; it only relabels the old `index.css` values as staged compatibility.

## Scenario mapping

- **Frontend tooling is inspected:** exact dependency assertions, Vite plugin assertions, CSS import assertions, and deleted-file assertions live in `verify-token-foundation.sh`.
- **A downstream surface selects design roles:** the script asserts every canonical color/run-phase/font/radius/easing/duration role in `tokens.css`, compares complete colors with their staged HSL channels within the channels' rounding tolerance, and checks explicit/system light parity.
- **Existing frontend source is compiled during staging:** after lock convergence and review remediation, a final Vite 8.1.4 development compile produced 282,176 bytes of CSS headed by Tailwind 4.3.3 and containing representative `bg-background`, `animate-in`, `slide-in-from-bottom-1`, `rounded-4xl`, and light phase-role values.
- **Reduced-motion user loads the frontend:** the script asserts the preserved duration override and its one-iteration cap, so the infinite shimmer cannot become a 1ms loop.
- **Token-only change is proposed:** the script asserts exactly two `tokens.css` path-filter entries in the Storybook workflow.
- **Component generator reads frontend configuration:** the script parses `components.json` and asserts the empty config path plus `src/index.css` entry.

## Commands run

```text
openspec/changes/tailwind4-css-first-tokens/verify-token-foundation.sh
pnpm -C frontend typecheck
node scripts/check-frontend-boundaries.mjs
openspec validate tailwind4-css-first-tokens --strict
git diff --check
```

All five checks passed. The targeted script reported `Tailwind 4 CSS-first token foundation assertions passed`; TypeScript completed through `tsc -b`; the boundary gate reported zero production violations; strict OpenSpec validation reported the change valid; and the diff whitespace check produced no findings.

The Vite integration probe started the normal development server on `127.0.0.1:4178`, requested the application entry and `src/index.css`, inspected the compiled CSS, and then stopped the server. The first run after changing installed Vite versions exposed stale files in the generated `frontend/node_modules/.vite` optimizer cache; that cache was moved aside recoverably, and the fresh-cache probe completed without pre-transform errors.

## Lint evidence and scope distinction

`pnpm -C frontend lint` was run and did not pass. Its reported errors are in pre-existing generated output under `frontend/test-results/chromatic-archives/**`; it also visits generated `frontend/coverage/**` output. A narrower ESLint invocation over source then hit the repository's pre-existing multi-TSConfig parser ambiguity between `frontend` and `frontend/packages/a2ui-uar`, before reporting on C-02-owned source. C-02 changes CSS, JSON, YAML, package metadata, and one Vite import/plugin registration; TypeScript and the architecture boundary gate pass.

Those observed lint defects are not represented as green and are not repaired in C-02 because this change does not own generated-artifact cleanup or ESLint configuration. Phase exit still requires a green repository-wide lint gate.

## Issues by priority

### CRITICAL

None in the C-02 implementation or OpenSpec contract.

### WARNING

- The repository-wide lint gate remains red on generated artifacts unrelated to C-02. The phase must resolve or exclude those artifacts before final certification.

### SUGGESTION

None.

## Adversarial review round 1

The isolated round-one judge used `k3` against producer `openai/gpt-5` through the local REST gateway (`cross_model_check: verified-distinct`). It returned `PASS` with 0 critical, 6 warning, and 3 suggestion findings; the anti-theater gate passed at score `0.01785714365541935`.

Disposition:

- Confirmed the live accordion is the Base UI wrapper and consumes `--accordion-panel-height`; the Radix-variable warning does not apply.
- Added executable HSL-to-RGB comparison for the shared dark, light, and high-contrast roles. This caught and corrected a high-contrast yellow drift.
- Preserved the existing Inter-resolving `font-sans`/`font-body` stacks and added Roboto only as an optional `font-reading` role.
- Restored the `var(--radius)`-derived `sm` through `4xl` scale, covering five live large-radius call sites.
- Broadened the dangling-reference scan across live frontend/workflow config formats while excluding the imported entity-management documentation example.
- Scoped Tailwind automatic detection to `frontend/` while retaining explicit A2UI source coverage.
- Renamed the project easing to `--ease-emphasis`, avoiding replacement of Tailwind's global `ease-out` role.
- Removed the premature global 3px focus rule; the existing focus contract remains, and C-15 owns 3px certification.
- The Chromatic package/workflow hunks belong to completed C-00 and were present before C-02; C-02 preserves that operator-directed dirty work.

The round-one receipt is stored at `.kbd-orchestrator/phases/uar-uiux-full-migration-2026-08/review/tailwind4-css-first-tokens/findings.json`. A post-remediation review is required before archive.

Round two also passed with no critical findings. It identified that the repository tracks the TypeScript-generated `vite.config.js`, so C-02 now inventories and asserts that compiled artifact alongside its source. It also caught the 1ms infinite-animation risk in the reduced-motion override; the override now caps iteration count at one. Both workspace roots now pin the shared Vite 8 peer graph to 8.1.4. Chromatic ownership remains C-00, and the explicit token path is intentionally retained in both workflow trigger blocks per the binding execution handoff.

The closure review passed with no critical findings and found that the recursive config scan omitted hidden directories and that peer re-resolution had introduced additional Vite 8 versions. The scan now uses `rg --hidden`, and both workspace override maps prevent Vite 8 divergence. Its remaining Chromatic warning concerns completed C-00 state that the current execution handoff explicitly says is configured and must not be redone.

The first final-snapshot review reported two critical lockfile-specifier mismatches. Both were falsified against pnpm 11.15.0: `pnpm install --frozen-lockfile --lockfile-only` passed at the root workspace, and `pnpm -C frontend install --frozen-lockfile --lockfile-only` passed at the frontend workspace. pnpm intentionally records an override-normalized exact importer specifier while retaining the caret in `package.json`; frozen installs accept that state. The same review's light-theme finding was valid, so explicit/system-light now both define all seven run-phase roles and the assertion script checks their equality. Task 4.2 now states the recorded lint condition inline.

The corrected-final review returned `PASS` with 0 critical, 1 warning, and 0 suggestions using judge `k3` against producer `openai/gpt-5` (`cross_model_check: verified-distinct`). Its anti-theater screen passed at score `0.0803571417927742`. The remaining warning noted that the persistent assertion does not scan extensionless/root build entry points. A targeted closure scan across `frontend/Dockerfile`, root `Dockerfile`, `scripts/`, `run-dev.sh`, `start.sh`, and `bootstrap.sh` found no deleted-config references; the persistent assertion remains scoped to the binding handoff's live frontend/workflow configuration surface.

Review receipt SHA-256 values:

- Round 1: `cb15d96f627dce71a4f30c6b7b3aa4476080939961404fe8ac26d8b800e24cb1`
- Round 2: `b8a150688b6204fad1698f462d01d63f80447a5004911bb9766a7f55053e09d8`
- Closure: `5afe86b099e03d23ee8b8d86753e3405fb4d42c7db6fc1aa697f243cffa8e832`
- First final snapshot: `e0b982cfbbc91a00507c29d561b4b2e9d959c995535c8f25cb35d71259a08815`
- Corrected final: `e91daa96f2ca3484a6137255116c76308102581bc229d4a08e6d87a44ef8e9d5`

## Final assessment

The C-02 implementation is complete and its owned contracts are deterministically verified. It is ready for isolated adversarial review, with the unrelated repository-wide lint condition explicitly retained as phase-level evidence rather than hidden.
