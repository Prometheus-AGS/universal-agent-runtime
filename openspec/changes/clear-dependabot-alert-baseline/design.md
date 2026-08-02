## Context

The live Dependabot API reports 34 open alerts across seven lockfiles. Two of those locks are duplicate npm fallback graphs: root npm resolution fails with an ESLint peer conflict, and frontend npm resolution fails on the workspace protocol, while the same surfaces have authoritative pnpm workspaces used by builds and releases. The remaining website and TypeScript SDK npm locks are reproducible and CI-authoritative. React Router's high-severity advisory has no patched v7 line, so dependency refresh alone cannot clear it.

## Goals / Non-Goals

**Goals:**

- Reach and verify zero open Dependabot alerts.
- Keep every retained dependency graph reproducible with its declared package manager.
- Preserve frontend routing behavior while moving to the patched React Router line.
- Preserve Tauri's supported desktop feature set while giving its `glib` alert an evidence-backed disposition.

**Non-Goals:**

- Broad dependency modernization unrelated to an open alert.
- UI design, route structure, provider behavior, or realtime-state changes.
- Dismissing actionable alerts or adding permanent allowlist entries for patched dependencies.

## Decisions

1. **Remove non-reproducible duplicate locks.** Root and frontend `package-lock.json` files will be removed because fresh npm resolution fails against their current manifests. Root security audit CI will use pnpm, matching the root `packageManager`, build, release, and workspace commands. Website and TypeScript SDK remain npm-managed because their CI runs `npm ci` successfully.
2. **Migrate to React Router v8 rather than override internals.** Frontend imports will move from `react-router-dom` to `react-router`, with React and React DOM raised to 19.2.8. This follows the official declarative-mode v8 migration and avoids forcing an unsupported v8 core beneath a v7 compatibility package.
3. **Refresh compatible transitives through package managers.** pnpm/npm locks will be updated using their native resolvers. Overrides are added only if every parent range excludes a patched release, and any such override is exact.
4. **Prefer a patched desktop graph; dismiss only when proven unreachable.** The controlling Tauri/Wry stack will be checked first. If current upstream still requires `glib` 0.18, dismissal is permitted only after source search proves neither first-party code nor the resolved Tauri/Wry/Tao/GTK/WebKit/GIO chain calls the vulnerable `VariantStrIter` API. Deleting the desktop lock is not acceptable because it is an independently reproducible workspace.
5. **Use GitHub as the completion authority.** Local audits prove the committed graphs, while completion additionally requires the live Dependabot API to report zero open alerts after the pushed commit is processed.
6. **Remediate newly published RustSec findings found during certification.** The consolidated audit discovered compatible patched releases for `ammonia` and the Wasmtime 46 line. The root Cargo lock will take those patch releases even though GitHub had not yet opened Dependabot alerts for them; retaining known vulnerabilities would contradict the requested clean baseline.

## Risks / Trade-offs

- **React Router v8 baseline raises Node/React requirements** → Pin React 19.2.8 and verify the project and CI use Node 22.22+ or Node 24.
- **Lock refreshes can introduce unrelated semver-compatible drift** → Review package-level diffs and constrain updates to alert-bearing dependency paths where practical.
- **Dependabot processing is asynchronous after push** → Poll the live API and distinguish processing delay from a still-vulnerable committed graph.
- **Tauri's Linux GUI chain constrains `glib` to 0.18** → Record current-version and no-call-site evidence, dismiss specifically as `not_used`, and revisit when Tauri adopts GTK bindings on the patched glib line.

## Migration Plan

1. Update frontend router and React manifests/imports.
2. Regenerate retained lockfiles and remove unreproducible duplicate npm locks.
3. Resolve or disposition the Tauri alert and update dependency-security documentation/CI.
4. Apply compatible Rust patch releases surfaced by the consolidated audit.
5. Run focused type/compiler checks, then consolidated audits and supported-profile validation.
6. Commit and push; wait for Dependabot processing and verify zero open alerts.

Rollback is a normal Git revert of the remediation commit and restored prior locks, but rollback reopens known security alerts and is not an acceptable steady state.

## Open Questions

None. The live alert inventory and observed package-manager failures determine the remediation path.
