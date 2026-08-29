# Release and local deployment evidence

## Accepted source commits

- Liter 1.18.2: `c5c6caac617eb931cd5009146a70831422ec236c`
- Surreal Memory: `432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8`
- Prometheus Skill System: `ad5c82c6c16145637c589a3ddfa06e0f20d603e7`
- Surreal Memory remote merge: `adc105d9cb33deda37c99b60d5671e3c0e0b50fe`
- Prometheus Skill System remote merge: `c0270899bed6aa3a7823f3b7a8c854df72f21993`

## Tier 3 source artifacts

All four Mach-O binaries carried linker-generated ad-hoc signatures and passed
`codesign --verify --deep --strict` before installation.

| Artifact | SHA-256 | Signing state |
|---|---|---|
| `target/release/universal-agent-runtime` | `b5c401e634eb7e110afa37be2843ccc02d47787e169f3c6d37a7ed7a752a1f3e` | ad hoc, strict verification passed |
| `target/release/liter-llm` | `2ca89b43b946dc5f6c2706307d42d702bdb20a16c8d58ea747e7595c31c40432` | ad hoc, strict verification passed |
| `target/release/surreal-memory-server` | `a5efa4e61707a756290d5561f00f5cff496a2914555bf5754871babd4b1ed499` | ad hoc, strict verification passed |
| `executors/mlx/.build/release/surreal-memory-mlx-executor` | `dd36733da5fd11e7bdb833afd82331cbee47cee6c2613bbcb5bd8687cf4aae27` | ad hoc, strict verification passed |

Build commands:

```text
cargo build --locked --release --no-default-features --features server-full
cargo build --locked --release -p liter-llm-cli
cargo build --locked --release -p surreal-memory-server
swift build --package-path executors/mlx -c release --disable-sandbox
```

All commands exited successfully. The Swift build emitted four upstream MLX
Metal `constexpr if is a C++17 extension` warnings and completed successfully.

The final corrected offline source archive has SHA-256
`aa0af789e7df47723398b6517b8519bd055907b04452bc7957b39153525c8d50`.
A fresh extraction built with a disposable empty `CARGO_HOME`,
`CARGO_NET_OFFLINE=true`, `--locked`, `--offline`, and the `minimal` feature.
The resulting debug acceptance binary had SHA-256
`316a07ccd38557f3374539c623e49157d505e77f8f7b30d89569b502fe566748`.
An earlier extraction reached final compilation but exhausted the volume; it
did not produce an artifact. Both disposable extractions were removed after
the successful rerun.

## Pre-deployment rollback capture

Rollback artifacts were copied to
`/Users/gqadonis/.prometheus/backups/refresh-liter-surreal-dependencies-20260828T1135Z`
before any installed binary was replaced. The capture includes all three
LaunchAgent property lists and the installed UAR, Liter, Surreal Memory server,
and MLX executor binaries from both installed locations.

| Installed artifact | Pre-deployment SHA-256 |
|---|---|
| `/Users/gqadonis/.uar/bin/universal-agent-runtime` | `b6fe01c4f3e68e02ce5967da48d70d980880e01261a7c9d64bf8619e89450de2` |
| `/Users/gqadonis/.local/bin/liter-llm` | `db0cbb11cae16650db1680d24718a7944892514313c9b0c1c0641fceba0002dc` |
| `/usr/local/bin/surreal-memory-server` | `c99f78c9373c935c5d0ae1192a0d470771596e5af876631b5a79fe4b1c586061` |
| `/Users/gqadonis/.local/bin/surreal-memory-server` | `c99f78c9373c935c5d0ae1192a0d470771596e5af876631b5a79fe4b1c586061` |
| `/usr/local/bin/surreal-memory-mlx-executor` | `31fa8dd1c86b4bae8fa2fde2130f48800674cd92f08bd2ba4868e2062bad617d` |
| `/Users/gqadonis/.local/bin/surreal-memory-mlx-executor` | `31fa8dd1c86b4bae8fa2fde2130f48800674cd92f08bd2ba4868e2062bad617d` |

The loaded pre-deployment process identities were SurrealDB PID 924,
Surreal Memory PID 56182, and UAR PID 45385. SurrealDB reported
`3.2.4+20260803.93ab219 for macos on aarch64` and listened only on
`127.0.0.1:28000`; Surreal Memory listened on port 23001; UAR listened on
loopback port 1906. All captured installed binaries had ad-hoc signatures.

## Deployment results

Liter 1.18.2 was installed at `/Users/gqadonis/.local/bin/liter-llm`.
Surreal Memory was installed at both `/usr/local/bin/surreal-memory-server`
and `/Users/gqadonis/.local/bin/surreal-memory-server`; the MLX executor was
installed at the corresponding two locations. Every installed SHA-256 matched
its Tier 3 source artifact, and all five installed files passed strict code
signature verification.

`liter-llm --version` returned `liter-llm 1.18.2`. A disposable trusted-local
MCP configuration was used to send an MCP `initialize` request over stdio. The
installed server returned protocol version `2024-11-05`, server name
`liter-llm`, and server version `1.18.2`; the disposable configuration was then
removed. Starting stdio without an authentication binding was also observed to
fail closed with the expected requirement for `mcp.stdio_key_id` or
`mcp.stdio_trust_local = true`.

## Final integrated runtime candidate

The integrated UAR candidate at
`target/release/universal-agent-runtime` had SHA-256
`d8ebe7a7120e32b07f946c59986987deb0d0d0a6f065ca85760b8b1719bc5a1a`.
It was installed through `packaging/native/macos/install.sh`; the installed
`/Users/gqadonis/.uar/bin/universal-agent-runtime` had the same SHA-256 and
passed `codesign --verify --strict --verbose=2`. Its static entrypoint served
`assets/index-DO4ve2ts.js`.

A dependency-ordered restart produced SurrealDB PID 5272 at
`/opt/homebrew/bin/surreal`, Surreal Memory PID 5292 at
`/usr/local/bin/surreal-memory-server`, and UAR PID 5337 at
`/Users/gqadonis/.uar/bin/universal-agent-runtime`. The post-restart receipts
were:

- an authenticated SurrealDB query returned status `OK` and
  `{ "certified": true }`;
- Surreal Memory `/health` returned version 1.7.0 and status `ok`; the earlier
  certification record survived its controlled restart;
- UAR `/healthz` returned `ok`, `/readyz` returned `ready` with six MCP tools,
  and the installed static entrypoint was reachable;
- startup reconciliation discovered 1,038 standard agent skills and reported
  all 1,038 unchanged on the controlled restart.

The post-offset SurrealDB, Surreal Memory, and UAR stderr segments contained no
`error`, `fatal`, or `panic` match. The UAR operational log did record degraded
optional configuration: internal OpenAI-backed memory lacked a key, Tavily had
an unresolved key placeholder, `npx` was absent for the optional time server,
and an empty configuration-skill source was refused for tombstoning. Because
the milestone asks for clean-log evidence, task 6.4 remains open even though
the required services are healthy and ready.

Two isolated artifact critics passed after the standard-skills implementation
was corrected to allow a plugin selector symlink in an alias target's ancestor
path while continuing to reject a symlink as the final manifest target.

Commit `cc780302e374a6cdb7fa809e2a026d6b109898c5` reproduced the certified UAR
SHA-256 in an exact-commit locked release invocation. The macOS installer then
installed that same digest and relaunched the UAR LaunchAgent. The branch was
pushed without force and opened as
`https://github.com/Prometheus-AGS/universal-agent-runtime/pull/275`.

PR #275 merged to `main` as `c5f83b13cf2c27d4f211a33e2b4ee6ecd48c0a06`.
After GitHub refreshed the default-branch dependency graph, alerts #199, #200,
#204, #205, #208, #213, #214, and #216 were `fixed`. The tracked
`scripts/security-audit-local.sh` guard was confirmed on `origin/main` before
alerts #210 and #211 were dismissed as `tolerable_risk`. Both dismissal
comments identify repository security maintainers as owner, set review date
2026-11-24, and require reopening for untrusted image ingestion or a compatible
fixed release. A final authoritative `state=open` query returned zero alerts.

Final cleanup retained one previously untracked Governance Playwright receipt in
the archived governance change, then removed four legacy worktrees. The two
merged remote feature branches were deleted; the rollback remote branch was
retained as the recovery source. Local state now contains only the root
worktree and local `main`; recursive submodules are clean and reachable. The
generated UAR `target` directory and obsolete offline archive were removed.

Clean-log closure and OpenSpec archive remain pending.
