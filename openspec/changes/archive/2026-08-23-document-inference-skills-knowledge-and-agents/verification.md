# Verification

Results are limited to source documentation under the documentation profile.
No runtime behavior, fresh inference, cross-profile equivalence, rendered site,
accessibility, deployment, or public-route claim is made by this change.

| Requirement | Command | Observed result | Limit | Source SHA | Profile |
|---|---|---|---|---|---|
| Seven-guide authority manifest | `node scripts/validate-documentation-product-workflows.mjs` | Exit `0`; `Documentation product-workflow validation passed (7 guides).` | Source, manifest, link, marker, and sanitization validation only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing guide fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: missing guide rejected` | Isolated copied-source mutation | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Unclassified authority fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: unclassified record rejected` | Isolated manifest mutation | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing packaged surface fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: missing packaged surface rejected` | Isolated copied-source mutation | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing state/profile limit fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: missing profile or state limit rejected` | Isolated copied-source mutation | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Synthetic evidence fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: synthetic-as-genuine inference rejected` | Text classifier for the public evidence contract; no inference was run | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing skill safety fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: missing skill safety rejected` | Isolated copied-source mutation | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Knowledge/memory conflation fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: knowledge-memory conflation rejected` | Terminology/source-boundary control only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing diagram explanation fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: missing diagram prose rejected` | Mermaid source and adjacent prose only; no render | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Unsafe private excerpt fails closed | `npm run docs:product-workflows:controls` | `PASS negative control: unsafe private excerpt rejected` | Public-source sanitization patterns; not a secret scanner | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Complete product-workflow fixture | `npm run docs:product-workflows:controls` | `PASS positive control: current product-workflow documentation` after nine negative controls | Current working-tree documentation source | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Docusaurus TypeScript | `npm --prefix website run typecheck` | Exit `0`; `tsc` completed without diagnostics | Type/config compile only; no production build | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Existing architecture controls | `npm run docs:architecture:controls` | Exit `0`; all architecture negative controls and the complete source fixture passed | Source controls only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Existing brand controls | `npm run docs:brand:controls` | Exit `0`; all brand negative controls and the complete source fixture passed | Source controls only; no rendered visual claim | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Composed publication controls | `npm run docs:publication:controls` | Exit `0`; all composed negative controls and the complete source fixture passed | Fixture composition only; the incomplete phase tree is not publication-ready yet | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Public-content safety audit | Node source audit over the seven guides and compatibility page | `PASS no machine-local paths, credentials, private keys, raw event/session payloads, raw .prometheus paths, or versions.toml claims` | Bounded string/pattern audit plus manual source review | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Strict OpenSpec | `openspec validate document-inference-skills-knowledge-and-agents --strict` | Exit `0`; `Change 'document-inference-skills-knowledge-and-agents' is valid` | This change bundle only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Artifact-refiner content gate | Draft 7 validation of the named artifact-refiner constraints and manifest, referenced-file inspection, and active/history final-state comparison | Five of five constraints satisfied; zero blockers; both schemas passed; active and archived state converged with five checkpoints | Bounded `direct:content` review; not a browser, runtime, or fresh-inference certification | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Permitted-surface audit | `git status --short` plus explicit status queries for runtime, React, vendored, lockfile, route/navigation, README, `.prometheus`, and workflow paths | Only product-workflow docs, validators, OpenSpec, named refiner evidence, and root script registration were present; all prohibited-path queries and lockfile query produced no entries | Working-tree delta before KBD handoff and commit | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Canonical KBD transition | `prometheus kbd change transition … in-progress; prometheus kbd change transition … complete; prometheus kbd revise … --exact-next-work "/opsx:new document-security-tenancy-governance-and-operations"` | Revisions `350`–`352`; `83/97` implementation tasks complete; exact next command names the security and operations documentation change | Control plane was unreachable, so the canonical runtime committed locally and refreshed generated projections; lifecycle evidence only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |

## Deferred evidence

- The full Docusaurus production build, Mermaid rendering, browser navigation,
  local-search interaction, keyboard inspection, accessibility-tree review,
  automated accessibility scan, and contrast measurement remain owned by
  `certify-and-publish-uar-docs` after every content slice is complete.
- GitHub Pages deployment, deployed-route validation, and the repository
  homepage link remain unverified until the final publication change.
- No runtime operation or fresh real-model inference was run. Public evidence
  summaries are bounded syntheses of the retained `server-full` evidence at
  source SHA `d41bf7c3a447869896664d44ac0563e1b4a1d9f3`; they transfer to no other
  checkout, provider, model, or profile.
- `server-full`, `minimal`, and `embedded-mobile` behavior are documented
  separately. This documentation-profile result is not a cross-profile claim.
