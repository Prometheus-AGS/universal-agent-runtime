# Verification

Results are limited to architecture-documentation source under the
documentation profile. The complete production build, rendered Mermaid output,
browser navigation, responsive behavior, accessibility, deployment, runtime
behavior, and every cross-profile readiness claim are intentionally deferred.

| Requirement | Command | Observed result | Limit | Source SHA | Profile |
|---|---|---|---|---|---|
| Complete architecture source | `node scripts/validate-documentation-architecture.mjs` | Exit `0`; `Documentation architecture validation passed (7 guides).` | Manifest, source, provenance, markers, headings, and local files only; no render | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing page fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing architecture page` | Isolated copied source fixture | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing authority fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing source authority` | Isolated copied source fixture; existence only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Invalid profile fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: invalid profile` | Exact manifest vocabulary only: `server-full`, `minimal`, `embedded-mobile` | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing provenance fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing provenance record` | Frontmatter/manifest correspondence in isolated fixture | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing profile limit fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing profile limit` | Required section/source marker only; no runtime comparison | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing trust boundary fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing trust boundary` | Required public concept marker in isolated fixture | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Missing diagram explanation fails closed | `node scripts/test-documentation-architecture.mjs` | `PASS negative control: missing diagram explanation` | Mermaid pages require adjacent prose marker; visual equivalence unrendered | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Complete fixture positive control | `node scripts/test-documentation-architecture.mjs` | `PASS positive control: complete architecture source` after all seven negative controls | Current working-tree source only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Conceptual navigation | Bounded Node check over `docs/publication/architecture.json` and seven guide-local Markdown links | `PASS architecture navigation: intro -> trust-boundary -> execution-lifecycle -> state-and-events -> profiles -> protocols -> delegation; all local architecture links resolve` | Source paths only; Docusaurus/browser routing unexecuted | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Truth and publication safety | `rg` forbidden-pattern scan plus bounded Node provenance/authority/profile audit | `PASS architecture truth/safety audit: 7 guides`; no forbidden raw history, local paths, credentials, absent-version claim, or prospective delivery language | Named patterns and exact current authorities only; not a general secret scanner or runtime proof | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Docusaurus TypeScript | `npm --prefix website run typecheck` | Exit `0`; `tsc` completed without diagnostics | Type/config compile only; no production build | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Brand source non-regression | `node scripts/validate-documentation-brand.mjs` | Exit `0`; `Documentation brand validation passed.` | Previously established branding source only; no browser render | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Publication composition controls | `node scripts/test-documentation-publication.mjs` | Exit `0`; every publication negative and positive control printed `PASS`, including preservation of child-validator failure | Isolated publication fixtures; complete repository publication remains intentionally incomplete | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Strict OpenSpec | `openspec validate document-uar-theory-and-architecture --strict` | Exit `0`; `Change 'document-uar-theory-and-architecture' is valid` | This change bundle only | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Artifact-refiner content gate | Python Draft 7 validation against artifact-refiner `1.4.1` schemas, referenced-file check, five phase checkpoints, convergence-state check, and `state-finalize.sh` | Both schemas valid; one referenced review artifact exists and is non-empty; five constraints satisfied; state finalized with zero blocking violations | Bounded `direct:content` review; no preview, deployed-site, runtime, or cross-profile claim | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Permitted-surface audit | Bounded `git status --porcelain=v1 -z` classifier over tracked and untracked changes | `PASS scoped diff: 18 changed paths`; all within architecture documentation, local validation, OpenSpec, refiner evidence, or active KBD projection surfaces | Current working-tree delta before KBD refresh and change commit | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |
| Canonical KBD transition | `prometheus kbd change transition … in-progress; prometheus kbd change transition … complete; prometheus kbd revise … --exact-next-work "/opsx:new document-inference-skills-knowledge-and-agents"` | Canonical revision `349`; `4/11` changes complete; exact next command names the product-workflow documentation change | Lifecycle position only; no later content or phase-completion claim | `d743b3ff9a2f8cf6aecb791d6e58a50498b4b4cb` | documentation only |

## Deferred evidence

- The production Docusaurus build is not run because the remaining product,
  operations, API, README, decision-history, and testing-history content is not
  complete.
- No rendered Mermaid inspection, browser-route pass, keyboard pass,
  accessibility-tree inspection, automated accessibility scan, responsive
  screenshot, or deployed Pages request is claimed.
- Documentation-source evidence makes no runtime, inference, provider,
  realtime, security, release, or cross-profile readiness claim.
