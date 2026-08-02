## Why

GitHub currently reports 34 open Dependabot alerts across the root, frontend, website, TypeScript SDK, and Tauri dependency graphs, including one critical and multiple high-severity findings. The repository needs a reproducible zero-alert baseline so its existing Dependabot CI gate can distinguish newly introduced risk from accumulated stale locks and vulnerable dependency lines.

## What Changes

- Upgrade the frontend from the vulnerable React Router 7 line to React Router 8.3.0 and its required React 19.2.8 baseline.
- Refresh authoritative pnpm and npm lockfiles so transitive packages resolve to patched versions.
- Refresh the root Cargo lock to patched `ammonia` and Wasmtime releases discovered by the completion audit.
- Remove root and frontend npm fallback lockfiles when their current manifests cannot be reproduced with npm, and make the root security audit use the authoritative pnpm graph.
- Resolve the Tauri `glib` alert through a patched dependency when compatible, or dismiss it as not used only if current upstream constraints and source-level reachability evidence prove the vulnerable API is unreachable.
- Update dependency-management documentation to record the new package-manager and zero-alert baseline.
- Verify local ecosystem audits and GitHub's live Dependabot alert API report no unresolved findings.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dependency-security-posture`: Require one authoritative, reproducible dependency graph per project surface and a verified zero-open-alert baseline after remediation.

## Impact

- Affected manifests and locks: root/frontend pnpm graphs, website and TypeScript SDK npm graphs, and the root and Tauri Cargo locks; the Tauri `glib` alert receives an evidence-backed disposition while compatible Rust dependencies in that lock are patched.
- Affected source: frontend router imports only; route behavior and runtime UX remain unchanged.
- Affected CI: the root JavaScript security-audit job moves from the obsolete npm graph to pnpm.
- Provider compatibility and realtime entity state are unaffected; this change does not modify provider routing, network contracts, stores, services, or event propagation.
- KBD workflow state does not require a phase transition; this operator-directed remediation is tracked by this OpenSpec change and the active task state.
