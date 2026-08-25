# KBD registered-runtime initialization evidence

Date: 2026-08-25
Issue: Prometheus-AGS/universal-agent-runtime#265

## Delivered source

- Upstream repository: `Prometheus-AGS/prometheus-skill-system`
- Review branch: `codex/fix-kbd-uninitialized-runtime`
- Exact commit: `602750ec61bc4674b51231fb36f3bfee3af42b7e`
- Upstream review: https://github.com/Prometheus-AGS/prometheus-skill-system/pull/68
- UAR gitlink `crates/prometheus-skill-system` resolves to the exact commit above.

The upstream change keeps registration and read-only status non-mutating. The
first typed mutation initializes either revision-zero representation, imports
compatible legacy phase state, then applies the mutation. It preserves the
existing run on later mutations and returns a non-zero process status for a
rejected command.

## Local verification

- Strict upstream OpenSpec validation: passed, 7/7 tasks complete.
- Focused `kbd-runtime` projection/import regression: 1 passed.
- Upstream `prometheus-cli` suite: 28 passed before the final scenario; the complete focused process target passed 3/3 afterward.
- Affected Rust formatting: passed.
- Affected production-target Clippy with `-D warnings`: passed.
- Debug and release `prometheus-cli` builds: passed.
- Protected-test verification from committed Git state: passed with zero protected changes.

The broad `kbd-runtime` suite reported 66 passed, 1 failed, and 6 ignored. The
same repository-ledger fixture test fails on clean upstream `origin/main` at
`1308e4b7a5d023e50bc0676ce497003b0bf7597b`, where 30 aggregate-only completion
ledgers trigger the projection-ahead safety guard. The fix preserves that guard
and narrows the mismatch set to five genuinely ambiguous ledgers; it does not
discard projection-only completion evidence.

## Installed CLI

- Installed path: `/Users/gqadonis/.local/bin/prometheus`
- Installed signed SHA-256: `1f3d8d5a35c7012bdc43a89ef87cae0002808abc9b4a7af5933cd742c659fc70`
- `codesign --verify --verbose=2`: passed.
- Installed-binary isolated proof: 3/3 passed through `PROMETHEUS_CLI_TEST_BINARY`.
- Rollback binary: `/Users/gqadonis/.local/bin/backups/prometheus-20260825T105512Z`
- Rollback SHA-256: `72fb22a2472a68f05596854b0d0e8dc97798ec9a932d59cdee699d1e2f277b3c`

`ai.prometheus.sovereign-sync` remained running as PID 26260 with run count 5
before and after installation. The daemon code and binary are unchanged, so it
was not restarted.

## Harness portability

`diff --exclude=.DS_Store -qr` confirmed byte-equivalent KBD orchestrator
payloads for Codex, Claude Code, Cursor, and OpenCode. Their top-level `SKILL.md`
files share SHA-256
`db5205b1c82fb723249d86f8a423d91e9ca4fc4f2a8257acb21313796b88484b`.
The shared host mutation binary resolves to the installed path above.

## Scope confirmation

No UAR backend, frontend, provider, persistence, configuration, payload, or
LaunchAgent runtime binary changed. The untracked `versions.toml` and former
phase `prior-context.md` were not staged or rewritten.
