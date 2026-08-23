## 1. Establish the site-local brand platform

- [x] 1.1 Pin the verified local-search and UAR font packages in `website/package.json` and regenerate only `website/package-lock.json`; verify npm resolves the exact requested versions without changing the root or SDK lockfiles.
- [x] 1.2 Configure hashed English local search for docs and pages with blog/Ask-AI disabled, and load all fonts locally; verify the config contains no hosted search, analytics, remote-font, or network-service dependency.

## 2. Apply the shipped UAR identity

- [x] 2.1 Add reviewed static UAR mark, favicon, and light/dark wordmark copies from `frontend/public/brand/`, remove stock Docusaurus assets, and verify canonical/source copies are byte-identical.
- [x] 2.2 Replace stock Infima CSS with the UAR light/dark surface ladders, ember/cyan signals, typography, focus, code, sidebar, search, table, Mermaid, responsive, and reduced-motion rules; verify a source control rejects green stock tokens, gradients, decorative shadows/borders, invisible focus, and missing reduced-motion handling.
- [x] 2.3 Update Docusaurus metadata and Mermaid/theme configuration to use the UAR identity and coordinated modes; verify no stock social card, favicon, tagline, or theme color remains.

## 3. Replace the tutorial experience

- [x] 3.1 Replace the tutorial homepage and sample feature component with semantic UAR product orientation, trust-boundary explanation, product/protocol surfaces, profile limits, and existing-route calls to action; verify heading order, link semantics, asset dimensions, and static module-level composition.
- [x] 3.2 Reconcile navbar, footer, sidebar labels, and API links with UAR product language and current existing routes; verify no navigation item targets a missing document in the pre-content site.
- [x] 3.3 Remove every remaining stock tutorial/sample asset or reference under `website/`; verify the branding validator identifies zero Docusaurus tutorial/crocodile/sample markers.

## 4. Verify and hand off

- [x] 4.1 After tasks 1–3 are code/content complete, run the isolated branding/source controls, typecheck, and dependency-resolution checks locally; record commands, observed outputs, limits, source SHA, and documentation profile in `verification.md` without running the phase-level production build.
- [x] 4.2 Run `openspec validate brand-uar-docusaurus-site --strict`, the available Web Interface Guidelines source audit, and the artifact-refiner gate; correct source findings until all bounded gates pass.
- [x] 4.3 Audit the diff for runtime, React application, provider/model, realtime, vendored, non-website lockfile, and raw `.prometheus` changes; remove anything outside the permitted surface.
- [x] 4.4 Transition the registered KBD change through the canonical runtime and refresh the cross-tool handoff; verify `current-waypoint.json` names the first content change next without editing generated JSON projections manually.
