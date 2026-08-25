ASSESSMENT: fix-runtime-settings-namespace-routes
Project: universal-agent-runtime
Date: 2026-08-25
Codebase baseline: The review branch preserves four local KBD/UI commits, is four commits behind origin/main, and reproduces a frontend-only namespace-to-route mismatch against an intact backend/provider configuration.
Cross-tool progress: none in this fresh run

IMPLEMENTATION STATUS
- Canonical backend settings routes: DONE — `src/uar/api/settings.rs` registers `/providers` and hyphenated namespace routes including `/context-management`.
- Settings write route conversion: DONE — `putSettingsNamespace()` already calls `namespaceToSlug(namespace)`.
- Settings read route conversion: MISSING — `fetchSettingsNamespace()` interpolates its argument directly, producing singular and underscored GET paths.
- Focused transport regression coverage: MISSING — existing settings tests exercise hooks/page behavior but no focused API test asserts the three required read URLs or existing non-2xx propagation.
- Installed-service browser coverage: PARTIAL — installed-session Playwright infrastructure exists, but there is no port-1906 settings namespace route test.
- Terminal-run continuity: DONE upstream, NOT PINNED here — `codex/kbd-run-rollover` is pushed at `f1e58b25b0a9926c24d1bb0ddb6c0678d16c6f49`; the installed CLI emitted revision 651 and the new UAR waypoint is fresh and conflict-free.

CROSS-TOOL PROGRESS
- NONE — the new phase began with 0/0 completion and no changes, blockers, decisions, or claims.

SPEC GAP SUMMARY
- `frontend-configuration-surfaces` preserves request compatibility but does not yet state that settings reads must canonicalize internal namespace keys to backend URL slugs.
- `kbd-phase-inventory-governance` covers terminal inventory but not the required successor-run boundary after terminal lifecycle state.
- The UAR submodule pin remains at `c25561548aeb9ca656fdb942ab34378beedc2fe2`; it must move to the pushed rollover commit during Execute, never through a detached edit.

BUILD HEALTH
- build check: UNKNOWN — phase verification commands have not run against the pending origin/main merge and implementation.
- known violations: the observed GET path mismatch; no unrelated source violation was inferred.
- test coverage: PARTIAL — page/hook coverage exists, while the transport and installed-service regressions requested by this phase are absent.

CONSTRAINT CHECK
- AGENTS.md violations: NONE in the fresh successor state; the former phase remains immutable audit rather than current work.
- constraints.md violations: N/A — no phase-specific constraints file is present.
- preserved operator files: untracked `versions.toml` and the former phase `prior-context.md` remain untouched.

GOAL PROGRESS
- Use canonical backend slugs for settings namespace reads: NOT MET — the GET wrapper still interpolates the namespace key.
- Preserve terminal-run continuity through supported KBD rollover: PARTIAL — upstream implementation and installed runtime are verified; the UAR submodule pin/spec delta remain.
- Rebuild, install, and verify the LaunchAgent service: NOT MET — no UAR source change or installed UAR rebuild has occurred in this phase.

ASSESSMENT COMPLETE
