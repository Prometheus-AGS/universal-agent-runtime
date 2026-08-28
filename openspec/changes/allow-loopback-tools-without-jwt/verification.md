# OpenSpec Verification: allow-loopback-tools-without-jwt

Date: 2026-08-27–28
Source candidate: `171cbf8531534c7c56fd72aea2a9c815172e85dd`
Rollback candidate: `0f97859f56bf9f097ba8ecc78b24daff6612145a`

## Result

The implementation satisfies the change's 10 added requirements and 55 scenarios. Local verification passed after the final production correction. The final isolated artifact critic returned PASS after the production, recovery, evidence, and KBD consistency corrections.

This verification does not mark publication or archive complete. Those OpenSpec tasks remain pending until the branches are pushed, the forward PR exists, and this change is archived.

## Requirement Evidence

| Delta requirement | Scenarios | Evidence | Result |
| --- | ---: | --- | --- |
| Optional JWT permits an exact local governance-optional posture | 11 | `runtime_control` unit coverage for exact configured literals, JWT-required, non-local and IPv6 posture, ingress declaration/sealing, and pre-finalization admission; full Rust suite; live loopback, non-local, and JWT matrix in `certification.md` and `evidence/release-matrix.json` | PASS |
| Eligible local governance is persisted and defaults Off | 6 | Governance persistence integration suite (8/8 focused and 47/47 full settings tests); restart/downgrade receipts in `rollback.md`; installed Off revision in both evidence JSON files | PASS |
| Governance-disabled local runs bypass governance decisions | 4 | deterministic `web_search` integration regression; middleware regressions `governance_off_bypasses_direct_tool_http_cedar`, `governance_on_preserves_direct_tool_http_cedar`, and `governance_off_preserves_cedar_for_non_tool_actions`; installed `native__memory_list` receipt plus actor-creation denial control | PASS |
| Inactive governance posts one warning per process | 4 | `inactive_warning_is_emitted_once_per_process`; installed warning cardinality of exactly one in `evidence/release-matrix.json`; live restart scope receipt | PASS |
| Governance status is one coherent runtime authority | 5 | runtime snapshot validation/revision tests; boot-instance replacement and stale-revision frontend tests; status API integration coverage; live revision 10→11→12 receipt | PASS |
| Preference resolution and mutation fail closed | 4 | `persistence_failure_never_finalizes_off_or_warns`, ineligible failure coverage, focused persistence integration tests, rollback mutation-unavailable live receipt | PASS |
| Governance mutations are serialized and authoritatively confirmed | 6 | mutation mutex/order integration coverage, post-runtime notification and delivery-failure tests, partial/rejection/change-elsewhere frontend store tests, live authoritative revision transitions | PASS |
| Governance settings distinguish durable, draft, and saving state | 8 | settings store and hooks test suites for partial, rejection, remote revision, restarted boot, and confirmation timeout; authoritative-save Playwright scenario | PASS |
| Governance master and dependent policies are accessible | 4 | component/unit accessibility coverage; Playwright keyboard, locked Required, Unknown/Refresh, focus visibility, and 200% zoom scenarios | PASS |
| Inactive warning has stable operational and visual semantics | 3 | runtime structured-warning field test and once-only receipt; Playwright 320 CSS px light/dark contrast and authoritative-Off warning scenarios | PASS |

## Observed Gates

- `cargo fmt --all -- --check`: exit 0.
- `cargo check --locked --no-default-features --features server-full`: exit 0, zero compiler warnings.
- `cargo test --locked --no-default-features --features server-full`: library 670 passed/1 ignored; BDD 9/9 scenarios and 49/49 steps; integration 93 passed/1 ignored; settings 47; skill-pack 1; UAR integration 9; doctests 17 passed/17 ignored.
- `pnpm build`: exit 0, 8,329 modules; existing PGlite direct-eval warnings only.
- `pnpm test`: 80 files and 406 tests passed.
- `pnpm typecheck`, `pnpm lint`, and `pnpm github-actions-policy:validate`: exit 0.
- Focused Governance Playwright: 5/5 passed.
- `pnpm support-matrix:validate`: 23 features, 9 provider tiers, 10 disabled direct dependencies.
- `pnpm release-local-contracts:validate`: all contracts and six negative controls passed.
- `openspec validate allow-loopback-tools-without-jwt --strict --no-interactive`: passed.
- Forward and rollback release builds and SHA-256 identities match `certification.md`.
- Installed health, readiness, status, toggling, warning cardinality, tool execution, and non-tool denial match the machine-readable evidence.

## Independent Review

The review sequence was FAIL, FAIL, PASS. The first two reviews exposed concrete authority-boundary, notification-order, rollback-ownership, evidence-retention, bypass-scope, and recovery-artifact defects. Each production finding was corrected and the complete local gate restarted. The final reviewer found only stale KBD claims; after those were corrected, the same isolated reviewer returned PASS.

## Remaining Limits

- No third-party search MCP is installed, so no live external search call is claimed. Search-specific behavior is deterministic integration evidence; installed live execution uses a configured native tool.
- Non-loopback and JWT live receipts were captured on the immediately preceding source. The final delta only narrowed direct HTTP bypass to the tool-execution route, and exact-final-source regression coverage includes eligible/ineligible authority plus tool/non-tool middleware behavior.
- Existing PGlite direct-eval build warnings remain unrelated to this change.
