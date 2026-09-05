# Local release installation — 2026-09-05

## Authorized scope and plan

The operator requested a release build, installation, LaunchAgent restart,
commit, and push. This is a local deployment, not GA publication or renewed
authorization for the previously cancelled certification gates. The three
completed Presentation changes remain active pending separate sync/archive
approval.

1. Build the locked `server-full` release executable and production web assets.
2. Back up the existing executable and static bundle, install the new artifacts,
   and retain the existing LaunchAgent plist, service environment and config.
3. Restart the existing UAR LaunchAgent; check executable identity, liveness,
   readiness and served assets. Do not send inference requests or expose keys.
4. Commit the completed runtime/Presentation source, tests, accepted historical
   archives and execution records; push the current branch to origin without
   force. Leave unrelated skill upgrades and accessibility receipts untouched.

## Baseline

Branch: `feat/context-history-integrity`; starting HEAD `f86267d4`.
Existing UAR PID: 31618; listener: loopback port1906.
The service uses `/Users/gqadonis/Projects/graph-explorer/.uar/config.yaml`,
not the stock installer's config. Therefore the stock installer must not be
used: it would regenerate the plist and merge provider configuration.

The uncomfortable baseline: `/healthz` returned200, but `/readyz` timed out
after10seconds and the configured native database's port28000 `/health` timed
out after8seconds. This predates installation. Do not attribute it to the new
release or treat liveness as readiness. The shared database was not restarted.

## Verification receipts

- `pnpm build`: exit0, 10472modules,17.92seconds. Four existing pinned PGlite0.5.4
  direct-eval warnings retained.
- `node scripts/validate-static-bundle.mjs static`: exit0,
  `Frontend bundle valid (11 referenced assets).`
- `git diff --check`: exit0 before deployment changes.
- `git diff --cached --check`: exit0 after removing whitespace-only defects
  from nine new Markdown review/spec artifacts.
- `gitleaks git --pre-commit --staged --redact --no-banner --no-color`:
  exit0, approximately2.99MB scanned, no leaks found. This is a heuristic scan.
- `pnpm github-actions-policy:validate`: exit0, deployment workflows only.
- `pnpm -C frontend typecheck && pnpm -C frontend lint`: exit0.
- Independent artifact-only review found no blocking scope or evidence issue
  in439staged files; it did not rerun tests or establish deployment success.
- Private rollback backup:
  `/Users/gqadonis/.prometheus/backups/uar/release-20260905.UGe0zD`.
  Existing executable, static bundle, plist, service environment and config
  retained. `cmp` of the backed-up and installed executable exited0.
- `cargo build --release --locked --no-default-features --features server-full
  --bin universal-agent-runtime`: exit0, `Finished release profile [optimized]
  target(s) in 47m 41s`. No Rust warnings were reported.
- The release executable reports `universal-agent-runtime 1.0.0`; `file` reports
  a Mach-O64-bit arm64 executable. Built and installed SHA-256:
  `2e3d3ab62920d43e1c26b78fb77e1cd46c80c9c4e1cfc97d369c73be9e46cd4c`.
- Database health recovered without mutation and returned200 twice before the
  UAR restart. The old process's readiness still timed out.
- Installed the executable and static bundle through a temporary same-filesystem
  staging directory after validation. Retained the previous static directory
  as `static-before-switch` in the rollback backup. Removed only the empty
  staging directory. `cmp` confirmed executable equality.
- Restarted only `com.prometheus.universal-agent-runtime` using the existing
  `packaging/native/macos/control.sh` stop/start operations. New PID18125.
  Existing plist, service environment and custom config match backup byte-for-byte.
- Startup was slow: database sign-in, database initialization, default knowledge
  base and persistence/RAG initialization progressed over several minutes.
  Loopback listeners1906/50051 exist. Health and readiness each returned200,
  but a subsequent readiness check returned408 after30.010136seconds and another
  timed out after10seconds. **Consistent readiness is not established.** The
  pre-install dependency timeout remains relevant; no root cause is asserted.
- A local HTTP byte comparison verified the served index and all11referenced
  assets against the new build. Index SHA-256:
  `2a8cb773797697a3daed2a3803a3f1e8912197c9839035cdc77dfa8de4677a5d`.
- The shared database LaunchAgent was not restarted. Approval was requested
  because that action would interrupt other applications using the database.
- Source commit: `d6f4f862` (`feat(runtime): complete governed execution and
  presentation workflows`),439files. Pre-commit lint17.34s, typecheck18.38s,
  deployment-policy validation0.55s and commit-message validation1.17s passed.
- `git push --set-upstream origin feat/context-history-integrity`: exit0;
  created the remote branch and configured tracking. No force push, PR creation,
  release tag or GA publication. GitHub reported14default-branch dependency
  alerts (13high,1moderate); no advisory remediation is claimed.
- Unrelated Impeccable upgrade files and accessibility receipts remain outside
  the commit. The new release remains installed with liveness observed200 and
  intermittent readiness. No database restart or root-cause repair was attempted.

Prior phase-boundary behavioral evidence is in the Presentation execution log
and the three OpenSpec verification reports. It includes the full server-full
suite,462frontend unit tests, targeted inspector/API checks and actual memory,
SurrealKV and PostgreSQL catalog contracts. This deployment does not rerun or
reinterpret those suites as release certification. Deferred429 coverage,
live-peer/billing, zoom/contrast and credential-rotation limits remain.

No production source, dependency pin, service configuration, credential or
GitHub Actions workflow was edited by this deployment procedure. Restarting the
host runs its normal schema initialization and idempotent seeding against the
existing database; no manual record edits, cleanup or data reset were performed.

## Authorized shared-database restart follow-up

The operator subsequently approved restarting the shared SurrealDB LaunchAgent
to attempt readiness recovery. Stopped UAR first, then booted out SurrealDB.
Both labels were absent and both previous processes had exited before the
database was bootstrapped again. No forced kill, cleanup or configuration edit.

SurrealDB PID29223 opened the existing RocksDB store and started its web server.
`curl --max-time 8 ... http://127.0.0.1:28000/health` returned200 in0.564665s.
The installed3.2.4 CLI completed an authenticated WebSocket `RETURN true;` query
with result `[true]`, exit0 in5705ms. Credentials were read from the existing
plist into child-process environment only; they were not printed or persisted.
Initial pre-listener connection failures are not counted as query passes.

Only after that query passed, restarted UAR as PID34726. Startup completed over
several minutes. Post-start liveness returned200 in0.021005s. A repeated
WebSocket query passed in9109ms, but another query during startup exceeded its
15-second timeout. Post-start readiness exceeded15seconds, then a separate
35-second-bounded probe returned408 in30.022509s. **The restart did not establish
stable readiness.** These observations do not establish the internal cause of
the latency; another restart or a dependency/configuration change is not implied.

Both LaunchAgents remain running. Both plist hashes, UAR service environment,
custom config and installed executable hash match their pre-restart values.
The RocksDB directory retains device16777235/inode898189177; it was not moved,
replaced or deleted. Normal host startup initialization is not a zero-write
claim. No product code, guards, dependencies or tests changed; only this receipt
and the append-only session log were updated. This is bounded operational
evidence, not a sustained-availability or release-certification result.
