# Verification

Results are limited to Docusaurus branding source under the documentation
profile. The complete production build, rendered visual quality, browser
accessibility, search interaction, generated API references, deployment, and
public routes are intentionally deferred until all phase content is complete.

| Requirement | Command | Observed result | Limit | Source SHA | Profile |
|---|---|---|---|---|---|
| Current brand source | `node scripts/validate-documentation-brand.mjs` | Exit `0`; `Documentation brand validation passed.` | Source/configuration inspection only; no browser render | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Hosted search fails closed | `node scripts/test-documentation-brand.mjs` | `PASS negative control: hosted search rejected` | Isolated copied source fixture | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Remote fonts fail closed | `node scripts/test-documentation-brand.mjs` | `PASS negative control: remote font rejected` | Isolated copied source fixture | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Stock identity fails closed | `node scripts/test-documentation-brand.mjs` | Stock green and stock tutorial mutations each printed `PASS negative control` | Token/marker source control; no pixel comparison | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Flat 2.0 decoration fails closed | `node scripts/test-documentation-brand.mjs` | Gradient, decorative shadow, and decorative border mutations each printed `PASS negative control` | CSS declarations covered by the bounded validator | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Accessibility source fails closed | `node scripts/test-documentation-brand.mjs` | Invisible focus and missing reduced-motion mutations each printed `PASS negative control` | Source semantics only; keyboard/AT/contrast unrendered | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Asset drift fails closed | `node scripts/test-documentation-brand.mjs` | `PASS negative control: asset drift rejected` | Five canonical/copy pairs; social card has no app-source pair | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Missing navigation fails closed | `node scripts/test-documentation-brand.mjs` | `PASS negative control: missing route rejected` | Current internal pre-content routes only; later changes may add routes | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Complete source fixture | `node scripts/test-documentation-brand.mjs` | `PASS positive control: current UAR documentation brand source` after eleven negative controls | Current working tree source only | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Exact site dependencies | `npm --prefix website ls --depth=0 @easyops-cn/docusaurus-search-local @fontsource-variable/geist @fontsource/space-grotesk @fontsource/jetbrains-mono` | Search resolved `0.55.3`; Geist, Space Grotesk, and JetBrains Mono each resolved `5.3.0` | npm reported 20 high-severity transitive advisories during install; no advisory remediation was authorized | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Docusaurus TypeScript | `npm --prefix website run typecheck` | Exit `0`; `tsc` completed without diagnostics | Type/config compile only; no production build | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Current Web Interface Guidelines source audit | Manual source audit of `website/src/pages/index.tsx`, its CSS module, and `custom.css` against the fetched current guideline | Meaningful links, one H1 with ordered H2/H3, labelled sections, explicit image dimensions, visible focus, scroll margins, narrow-table overflow, static module-level data, and reduced-motion handling observed | Visual hierarchy, browser behavior, AT output, and contrast remain unverified until final certification | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Strict OpenSpec | `openspec validate brand-uar-docusaurus-site --strict` | Exit `0`; `Change 'brand-uar-docusaurus-site' is valid` | This change bundle only | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Artifact-refiner constraints and manifest | Python Draft 7 validation against artifact-refiner `1.4.1` schemas plus manifest file-existence check | Both schemas printed `PASS`; `dist/brand-source-review.md` exists and is non-empty; named state converged | Bounded `direct:content` source review; preview explicitly deferred and not recast as visual certification | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Lockfile scope | `git diff --name-only -- '*lock*'` | Only `website/package-lock.json` observed | Working-tree diff before change commit | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Permitted-surface audit | `git diff --name-only -- src frontend vendored vendor sdks Cargo.toml Cargo.lock pnpm-lock.yaml .prometheus` | No output | Current change working-tree delta only; `.refiner` and OpenSpec/KBD evidence are permitted process artifacts | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |
| Canonical KBD transition | `prometheus kbd change transition … in-progress; prometheus kbd change transition … complete; prometheus kbd revise … --exact-next-work "/opsx:new document-uar-theory-and-architecture"` | Canonical revision `346`; `3/11` changes complete; exact next command names the first content change | Lifecycle position only; no architecture-content or phase-completion claim | `8f242f815c9733035ad7fb6ced990cf951119a8a` | documentation only |

## Deferred evidence

- The production Docusaurus build is not run here because required content is
  not complete.
- No browser screenshot, keyboard pass, accessibility-tree inspection,
  automated accessibility scan, local-search interaction, or contrast
  measurement is claimed.
- Real rustdoc/TypeDoc generation, GitHub Pages deployment, and public route
  requests remain owned by `certify-and-publish-uar-docs`.
- The 20 npm high-severity advisories remain unresolved and are not hidden by a
  broadened dependency migration in this branding change.
