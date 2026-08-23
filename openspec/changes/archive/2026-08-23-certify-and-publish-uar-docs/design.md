## Context

Changes 1–10 established one npm Docusaurus build, one Pages publisher, source
classification, complete product routes, branding, current guides, README
authority, and reviewed history. Production build and browser evidence were
deliberately deferred until this point so expensive checks would run against the
complete content rather than partial sites.

## Goals / Non-Goals

**Goals:** build and stage the exact portal locally; validate all classified
content and routes; inspect rendered behavior at four theme/viewport
combinations; run accessibility checks; publish the same build contract through
Pages; validate live deep links; set repository metadata; retain final evidence.

**Non-Goals:** product/runtime testing, real-model inference, load/soak testing,
runtime release certification, dependency upgrades, or redesign after a passing
artifact. Any rendered defect is fixed at its documentation source and the full
artifact is rebuilt once.

## Decisions

### 1. Use one local artifact sequence

Run npm frozen install, Docusaurus build, UAR library Rustdoc, TypeDoc, staging,
and composed publication validation in that order. The Rust reference targets
the public `universal-agent-runtime` library under `server-full`; workspace-only
utility binaries are not part of that public API. Generated references are real
artifacts; missing output fails rather than producing placeholders. Staging adds
only a stable `/docs/api/rust/` landing redirect to the generated UAR crate page.
Rustdoc source pages and only those source anchors that expose machine-local
build-output paths are removed during staging; the sanitizer then verifies that
no macOS, Linux, or Windows home path remains.

### 2. Use Playwright locally as the DevTools fallback

The Chrome DevTools MCP required by the selected browser skill is unavailable in
this harness. The checked-in Playwright/Chromium and axe-core dependencies provide
the same required observations: DOM, console, network, screenshots, interaction,
responsive layout, keyboard focus, Mermaid rendering, search, and WCAG analysis.
No browser content is treated as instruction.

### 3. Make live validation data-driven

`scripts/validate-deployed-documentation.mjs` reads every required route from the
frozen product route manifest and adds root, documentation, history, and generated
reference routes. The deploy job checks out only the source needed by this
deployment validator after Pages publishes. This remains deployment validation,
not routine product testing.

### 4. Bind repository metadata to observed deployment

The homepage field is set only after the workflow reports success and the live
validator observes the intended root and deep routes. If Pages permissions or
repository settings block deployment, report the URL/homepage claim unverified
rather than substituting a local URL.

## Risks / Trade-offs

- GitHub Pages can be temporarily unavailable after deployment; bounded retries
  distinguish propagation delay from a missing route.
- The first full-workspace attempt exposed an incompatible `rmcp` import in the
  internal `mcp-server-fetch` binary. Publishing the public UAR library reference
  avoids representing an internal utility as API surface while leaving that
  observed product-code defect explicit and untouched.
- Axe can report third-party Docusaurus/plugin markup. Serious violations still
  block publication; fixes remain in owned theme/content where possible.
- A workflow-dispatched branch deployment can be replaced later by main. The PR
  and homepage retain the canonical stable URL, while merge controls the next
  deployment from main.

## Migration Plan

1. Add final validators and deployment-route integration.
2. Run isolated validator controls, then the single complete local artifact build.
3. Serve and inspect the artifact locally; fix and rebuild if necessary.
4. Strict-validate every phase OpenSpec change and write final evidence.
5. Commit, push, dispatch Pages, observe the live routes, set homepage, reflect,
   update KBD, and open the PR.
