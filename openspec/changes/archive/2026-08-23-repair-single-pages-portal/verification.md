# Verification

Results are limited to the documentation build/publisher source contract. The
full Docusaurus build, generated workspace references, GitHub deployment, and
public routes are intentionally deferred until all phase content is complete.

| Requirement | Command | Observed result | Limit | Source SHA | Profile |
|---|---|---|---|---|---|
| Missing Rust output fails closed | `node scripts/test-documentation-staging.mjs` | `PASS negative control: missing Rust reference` | Isolated temporary fixture; no real rustdoc generated | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Missing TypeScript output fails closed | `node scripts/test-documentation-staging.mjs` | `PASS negative control: missing TypeScript reference` | Isolated temporary fixture; no real TypeDoc generated | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Complete staging copies both trees | `node scripts/test-documentation-staging.mjs` | `PASS positive control: complete reference staging` | Synthetic HTML fixture verifies staging mechanics only | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Workflow/package negative controls | `node scripts/test-documentation-publication.mjs` | npm mismatch, missing staging, placeholder fallback, missing deployed TypeScript route, missing publisher, and competing publisher each printed `PASS negative control`; complete fixture passed | Isolated workflow fixtures only | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Sole Pages publisher | `node scripts/validate-github-actions-policy.mjs` | Exit `0`; `GitHub Actions policy validation passed (deployment workflows only; Pages publisher: docs.yml).` | Source policy only; workflow not dispatched | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Workflow syntax | `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/docs.yml")'` | Exit `0`; `PASS: docs.yml parses as YAML` | YAML syntax only; GitHub expression/runtime semantics unexecuted | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Frozen documentation lockfiles | `test -z "$(git diff -- website/package-lock.json sdks/typescript/package-lock.json)"` | Exit `0`; both lockfiles unchanged | Hash/diff evidence only; no install run in this change | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Strict OpenSpec | `openspec validate repair-single-pages-portal --strict` | Exit `0`; `Change 'repair-single-pages-portal' is valid` | This change bundle only | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Artifact-refiner constraints | `validate-constraints.sh` from artifact-refiner `1.4.1` against `repair-single-pages-portal` | Exit `0`; `Constraints structure valid` | Named PMPO review only | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Artifact-refiner manifest | `validate-manifest.sh` from artifact-refiner `1.4.1` against `repair-single-pages-portal` | Exit `0`; manifest and referenced-file checks passed | No browser preview required; no deployment claim | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Permitted-surface audit | `git diff --name-only` plus untracked-file audit; reject `src`, `frontend/src`, `vendor`, documentation lockfiles, and `.prometheus` | No prohibited or frozen-lockfile path observed | Current change working-tree delta only | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |
| Canonical KBD transition | `prometheus kbd change transition … in-progress; prometheus kbd change transition … complete; prometheus kbd revise … --exact-next-work "/opsx:new brand-uar-docusaurus-site"` | Canonical revision `343`; `2/11` changes complete; next command names the branding change | Lifecycle position only; no branding or phase-completion claim | `9274dc2d3e07beb3613ec924a4ceea4cb37c2a70` | documentation only |

## Deferred evidence

- The production Docusaurus build is not run here because branding and required
  page content are not complete.
- Real rustdoc and TypeDoc generation are not run here; the final local gate and
  deployment workflow own those artifacts.
- No GitHub workflow was dispatched and no public route was requested.
