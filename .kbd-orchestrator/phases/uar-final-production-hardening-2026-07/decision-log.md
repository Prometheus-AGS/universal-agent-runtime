# Decision Log — uar-final-production-hardening-2026-07

### 2026-07-11 — Primary UI stack

Options: React first-party UI vs. HTMX/Web Components primary UI

Decision: **React 19 + TypeScript is the authoritative first-party UI.** HTMX/Web Components are not the primary interface and must not be described as such in present-tense product documentation.

Provenance: operator instruction plus current implementation evidence.

Consequences: retain the React frontend and Tauri webview path; rewrite stale README/package/docs claims; treat A2UI as validated declarative data rendered by React and AG-UI as the event protocol.

### 2026-07-11 — UI verification stack

Options: adopt a new admin/testing framework vs. finish the existing React/Vitest/Testing Library/Playwright stack

Decision: **finish and enforce the existing stack.** MSW is conditional on a one-domain fixture-reuse pilot. AG-UI/A2UI packages/specifications are conformance references before they are dependency candidates.

Provenance: local code/dependency inventory and official documentation research.

### 2026-07-11 — Remaining phase course

Decision: the current remaining five changes are insufficient to certify all advertised surfaces. Preserve the four completed changes, then re-spec/replan the remainder as architecture-boundary, protocol-conformance, surface-certification, documentation-truth, and release-evidence waves described in `analysis.md`.

Provenance: direct code scan found live Component/Hook/Store/Service violations and missing behavior contracts across A2UI Testing, Runtime Console, Providers, Models, Knowledge, Settings, and chat/session UI.
