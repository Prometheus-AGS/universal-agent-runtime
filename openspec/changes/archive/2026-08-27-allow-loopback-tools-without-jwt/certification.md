# Certification Receipts

Date: 2026-08-27–28
Operator authorization: Tier 3 milestone, release installation, commit, push,
and PR creation were explicitly authorized.

## Candidate identity

| Candidate | Source commit | Build profile | SHA-256 |
| --- | --- | --- | --- |
| Forward gate | `171cbf8531534c7c56fd72aea2a9c815172e85dd` | default `cargo build --release` | `e5e1690de5e92a9c3b49f3ab15820cd073c1b78e86b6f082f2f8170ca3881f14` |
| Rollback gate | `0f97859f56bf9f097ba8ecc78b24daff6612145a` | default `cargo build --release` | `d3975af6e8fb068404a990e0a598f7241a1d5a5ddea0ff3affe7c5c959f8a0ca` |
| Forward install | `171cbf8531534c7c56fd72aea2a9c815172e85dd` | locked release, `server-full` | `b6fe01c4f3e68e02ce5967da48d70d980880e01261a7c9d64bf8619e89450de2` |
| Rollback install | `0f97859f56bf9f097ba8ecc78b24daff6612145a` | locked release, `server-full` | `4ff9e1157a139a30c7cc988e56afbe82e07907bf746293ae38ba32e05c5cbdcd` |

Both install candidates are ARM64 Mach-O executables. The forward installed
digest matched the retained candidate byte-for-byte.

## Local verification

| Gate | Observed result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS, exit 0 |
| `cargo check --locked --no-default-features --features server-full` | PASS, exit 0, zero compiler warnings |
| `cargo test --locked --no-default-features --features server-full` | PASS: library 670 passed/1 ignored; BDD 9/9 scenarios and 49/49 steps; integration 93 passed/1 ignored; settings 47; skill-pack 1; UAR integration 9; doctests 17 passed/17 ignored |
| `pnpm build` | PASS, 8,329 modules; only existing PGlite direct-eval warnings |
| `pnpm test` | PASS, 80 files and 406 tests |
| `pnpm typecheck` | PASS, exit 0 |
| `pnpm lint` | PASS, exit 0 |
| `pnpm github-actions-policy:validate` | PASS; no non-deployment testing was added to GitHub Actions |
| `openspec validate allow-loopback-tools-without-jwt --strict` | PASS |
| Focused Governance Playwright | PASS, 5/5: 320 px, 200% zoom, keyboard focus, Required/locked, Unknown/Refresh, authoritative Off |
| `pnpm support-matrix:validate` | PASS: 23 features, 9 provider tiers, 10 disabled direct dependencies |
| `pnpm release-local-contracts:validate` | PASS: supply chain, provenance, manifest, packaging, installed candidate, MCP boundary, and six negative controls |
| Exact `cargo build --release` | PASS for forward and rollback after the disabled-telemetry facade was aligned |

The deterministic `web_search` integration regression passed inside the full
Rust suite: eligible anonymous Off executes without denial or approval events;
enabled or ineligible postures preserve the existing gates. Direct HTTP Cedar
coverage additionally proves that a request carrying `X-Agent-Id` bypasses
Cedar only for direct configured-tool execution while Off, receives HTTP 403
for that tool while On, and remains governed for non-tool actor creation while
Off.

## Rollback and ingress matrix

- Rollback, loopback, no JWT: effective On, mutation unavailable, reason
  `persistence_unavailable`; Off returned HTTP 500 with `governance mutation is
  unavailable` and status stayed On.
- Forward, loopback, no JWT: default Off; live On advanced revision 10→11 and
  Off advanced 11→12; inactive-warning count stayed exactly one.
- Forward, `0.0.0.0`, no JWT: Required/On, `may_disable=false`, reasons
  `configured_host_not_allowed` and `bound_ingress_not_loopback`; no inactive
  warning.
- Forward, loopback, JWT required: unauthenticated status returned 401; an
  authenticated test JWT observed Required/On, `may_disable=false`, reason
  `jwt_required`; no inactive warning.
- Shared-database downgrade: a seed-owned forward Off default booted as rollback
  On. The rollback mutation was unavailable; returning forward retained the
  normalized On until the seed-owned preference was restored through the
  forward API. The final isolated status was Off at revision 11. A separate
  focused regression starts from API-owned false, proves rollback preserves the
  row while enforcing On, and proves forward restart returns effective Off.

## Installed release

- Installer: `packaging/native/macos/install.sh --binary
  target/release/universal-agent-runtime --static-dir static`.
- LaunchAgent: `com.prometheus.universal-agent-runtime`.
- Program: `/Users/gqadonis/.uar/bin/universal-agent-runtime`.
- Installed SHA-256:
  `b6fe01c4f3e68e02ce5967da48d70d980880e01261a7c9d64bf8619e89450de2`.
- Health: `ok`. Readiness: ready, six MCP tools.
- Installed boot: `586bdaff-c660-43f2-a9d4-9c2e86119593`; authoritative
  state Off, revision 12, mutable, configured `127.0.0.1`, JWT disabled.
- One configured tool execution: `native__memory_list` returned
  `success=true` in 0 ms while governance was Off; the inactive warning count
  for that boot remained exactly one.
- One non-tool control request to actor creation with `X-Agent-Id` returned
  HTTP 403 and `GOVERNANCE_DENIED` while governance was Off, proving the bypass
  is scoped to direct configured-tool execution.

The installed MCP registry exposes six native memory tools and no search MCP.
Accordingly, live configured-tool execution is proven with `memory_list`; the
search-specific governance behavior is proven by the deterministic
`web_search` integration regression. No live search integration is claimed.
The machine-readable installed execution receipt is
`evidence/installed-tool-execution.json`; the release and posture matrix is
`evidence/release-matrix.json`.

## Recoverability

Backup directory:
`/Users/gqadonis/.prometheus/backups/uar/governance-release-20260828T.HtRDLE`

- Prior installed binary SHA-256:
  `a7aefee1d23be3b0f65a08d07fcbfb9f8a8d50746035f08cc724543acb8ff42f`.
- Complete governance row export SHA-256:
  `5154022f9c6b7a1891b742b51952d3167c6a98b6164320fadbcd6ff716873a3f`.
- Forward and rollback `server-full` candidates are retained in the same
  directory as `forward-171cbf85-server-full-universal-agent-runtime` and
  `rollback-0f97859f-server-full-universal-agent-runtime`. The older
  unqualified candidate filenames are superseded and SHALL NOT be selected.

## Remaining risk

The rollback candidate can normalize a seed-owned Off default to On, but it
preserves an API-owned Off row. Recovery must inspect the durable ownership
marker before writing; a concurrent unexpected row change is a stop condition.
The installed runtime has no configured search MCP, so a live third-party
search call remains outside this environment's evidence.

## Independent review correction

The first isolated artifact review failed the candidate. It found an ownership
overclaim in rollback documentation, an implicit settings-notification race,
missing direct-HTTP Cedar bypass coverage, stale KBD progress, and no durable
installed-tool receipt. The corrected candidate preserves API-owned `false`
rows while distinguishing seed-owned normalization, suppresses the implicit DB
governance notification and publishes one explicit post-runtime event, records
non-transactional notification failure without rolling back the commit, shares
the coherent gate with HTTP Cedar, and adds machine-readable receipts. The
corrected source was rebuilt and all Tier 0–3 gates above were rerun.
The second isolated review then found that the HTTP bypass was mounted too
broadly and that the recovery directory still contained only superseded
candidates. The bypass is now path-scoped to `/api/tools/*/execute`, non-tool
actions retain Cedar enforcement, final commit-qualified binaries are retained,
and the required gate was restarted again from the final source.
