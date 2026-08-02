## 1. Inventory and Workflow Bootstrap

- [x] 1.1 Capture the live Dependabot alert inventory with package, manifest, severity, vulnerable range, and patched version
- [x] 1.2 Confirm the authoritative package manager and reproducibility of every alert-bearing lockfile

## 2. Frontend Direct Dependency Remediation

- [x] 2.1 Migrate frontend React Router imports and manifest dependency to `react-router` 8.3.0
- [x] 2.2 Raise the frontend React and React DOM baseline to 19.2.8

## 3. JavaScript Dependency Graph Remediation

- [x] 3.1 Refresh root and frontend pnpm locks to patched transitive dependency versions
- [x] 3.2 Remove non-reproducible root/frontend npm fallback locks and move the root audit CI job to pnpm
- [x] 3.3 Refresh the website and TypeScript SDK npm locks to patched transitive dependency versions

## 4. Desktop Dependency Graph Remediation

- [x] 4.1 Verify current Tauri/Wry constraints and prove the vulnerable `glib::VariantStrIter` API has no first-party or resolved dependency call sites
- [x] 4.2 Record the `glib` disposition and dismiss Dependabot alert #140 as `not_used`
- [x] 4.3 Update the root and Tauri Cargo locks to patched `ammonia` 4.1.4 and Wasmtime 46.0.2 releases found during consolidated certification

## 5. Documentation and Validation

- [x] 5.1 Update dependency-management documentation with the authoritative lockfile and remediation baseline
- [x] 5.2 Validate OpenSpec artifacts and run focused frontend/Rust checks
- [x] 5.3 Run consolidated npm, pnpm, Cargo, and supported-profile security validation
- [x] 5.4 Commit and push the remediation, then verify the live Dependabot API reports zero open alerts
