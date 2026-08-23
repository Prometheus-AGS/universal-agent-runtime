## 1. Make artifact assembly deterministic

- [x] 1.1 Replace the Docusaurus build's pnpm subcommand with the pinned npm script chain; verify the package contract contains no pnpm/yarn/bun invocation and the website lockfile remains unchanged.
- [x] 1.2 Add a dependency-free reference staging command that requires real rustdoc and TypeDoc entrypoints, replaces only their declared build subtrees, and copies the complete artifacts; verify isolated missing-Rust, missing-TypeScript, and successful-staging controls produce the expected nonzero/nonzero/zero exits.
- [x] 1.3 Correct the portal API-reference guide to document the real npm commands and omit the nonexistent hosted Python reference; verify every advertised hosted reference has a corresponding generated artifact in the staging contract.

## 2. Establish one Pages owner

- [x] 2.1 Rewrite `.github/workflows/docs.yml` to check out required submodules, use the pinned npm installs and real generation commands, assemble one Docusaurus artifact, and validate the deployed root plus narrative/Rust/TypeScript routes; verify the workflow contains only deployment execution and deployed-artifact validation steps.
- [x] 2.2 Delete `.github/workflows/typescript-sdk-docs.yml`; verify repository workflow discovery reports `docs.yml` as the sole Pages publisher.
- [x] 2.3 Extend the isolated workflow/publication controls for npm-only build commands, required generated outputs, no placeholder fallback, representative deployed-route checks, and single publisher; verify each corrupted fixture exits nonzero and the complete fixture exits zero.

## 3. Verify and hand off

- [x] 3.1 After tasks 1–2 are code/content complete, run the staging controls and GitHub Actions policy validator locally; record command, observed output, limit, source SHA, and documentation profile in `verification.md` without running the phase-level Docusaurus build.
- [x] 3.2 Run `openspec validate repair-single-pages-portal --strict` and the artifact-refiner gate; correct the change artifacts until both pass.
- [x] 3.3 Audit the diff for runtime, React application, provider/model, realtime, vendored, lockfile, and raw `.prometheus` changes; remove anything outside the permitted surface.
- [x] 3.4 Transition the registered KBD change through the canonical runtime and refresh the cross-tool handoff; verify `current-waypoint.json` names `brand-uar-docusaurus-site` next without editing generated JSON projections manually.
