# UAR lifecycle command proposal — verification CLOSED

Latest operator closeout, 2026-10-21: testing was explicitly stopped. The new embedding attempt was canceled during compilation (Cargo SIGTERM/-15, wrapper143); no new success or test failure is claimed. Other targets never started. All further verification is CLOSED. Remaining work is tracked in https://github.com/Prometheus-AGS/universal-agent-runtime/issues/421. Earlier proposal/inventory text below records its historical state, not current authorization.

Renewed 2026-10-21 for UAR only. Boss pins, launchers, builds and packaging
are outside this packet and are not UAR prerequisites. This replaces the earlier
proposal; historical results remain in their existing receipts.

Execute phase: `uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Canonical waypoint3198 / plan88. See
`lifecycle-uar-approval-inventory-2026-10-21.md` and its saved-source JSON.
All recorded UAR production implementation rows are complete. Verification and
release evidence are incomplete. Presenting these commands authorizes none.

## Ready for approval: three existing local UAR integration commands

Run only explicitly approved commands, serially in this order, from
`/Volumes/T7/Projects/prometheus/universal-agent-runtime`.
Required process environment: `PATH=/Users/gqadonis/.cargo/bin:$PATH`,
the previously recorded Cargo path repair. There is no separate tool/version,
source-identity, readiness, Docker or other preflight command in this packet.

### 1. Real local embedding failover

```sh
UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR=/Volumes/T7/uar-local-embedding-minilm-1110a243fdf4 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL=sentence-transformers/all-MiniLM-L6-v2 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION=384 cargo test --locked --no-default-features --features server-full --test embedding_lifecycle_failover -- --test-threads=1
```

The existing immutable MiniLM preparation passed. Do not rerun its new-directory
preparation command. This target previously passed before the budget API edit.
It exercises real local inference, generations1→2→3→4→5, durable principal-bound
ingestion/query, a distinct equal-dimension model, incompatible-space refusal,
state preservation and fixture cleanup. It cannot pass by skipping absent assets.

### 2. Owned resource lifecycle

```sh
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend,test-probes --test lifecycle_owned_resources -- --test-threads=1
```

Five existing cases cover retained blocking work after waiter cancellation,
shared unbounded supervisor shutdown after canceled/concurrent waiters,
dependency ordering plus owned-engine close/reopen, borrowed-client survival,
and retained close failure.

### 3. Sidecar launch and security

```sh
cargo test --locked --no-default-features --features server-full,postgres-backend,in-memory-backend,test-probes --test sidecar_launch_security -- --test-threads=1
```

Uses real UAR binaries and parent-owned hermetic fixtures, private HOME/work/data/
logs, loopback sockets and a local LLM stub; no hosted credential is required by
source. Cargo supplies the sidecar, main and environment-probe binaries.
macOS observations require lsof and Unix standalone stop uses /bin/kill.
Linux branches use /proc; Windows branches use PowerShell and named pipes.
Native execution certifies only that host's branches.

## Prerequisite and coverage limits

Ready means existing source/receipts reveal no unresolved design or foreign-repo
prerequisite. It is not a fresh environment-health claim: no availability probes
were run. All targets require the toolchain and writable temporary storage;
embedding requires the recorded local model assets and native ONNX execution;
sidecar requires process/socket permissions and platform observation tools.
An unavailable prerequisite must be reported as such, never converted to a pass.
No extra preflight is implied.

These three targets do not directly call `shutdown_with_budget`. They cannot
establish budget expiry/acknowledgement races, repeated shutdown IDs,
`SnapshotPending`, post-budget completion, or the embedded budget wrapper.
Existing unbounded shutdown reaches the shared retained finalization path.
No new test or gate is added.

They also do not establish the full frozen embedding contract, process-restart
settings hydration, authenticated settings mutations, separate MemoryService
embedding behavior, full-server watcher teardown, all database transports or
migration lost-commit outcomes. The historical503 body and individual c9
sidecar traces remain missing.

Retain actual exits, sanitized per-case errors and cleanup outcomes. Stop on
failure. Any implementation repair closes verification again and requires a
renewed inventory and instruction. No earlier pass certifies later source.

## Not ready: existing full release acceptance command

```sh
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Volumes/T7/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/function-first-local.json
```

Keep closed. The manifest lacks current-source image binding and fresh qualified
same-image supervisor/guest and fixture preparation. Independent completed-artifact
review is pending. No preparation commands are invented here. Boss is not a
prerequisite. No existing exact Windows packet is ready in this proposal.

## Separate existing Docker delivery command — held for approval

```sh
/opt/homebrew/bin/gh workflow run deploy.yml --repo Prometheus-AGS/universal-agent-runtime --ref codex/uar-1-1-1-implementation
```

This existing deployment command can be approved separately; it is not one of
the three local verification commands. No dispatch occurs now. It selects the
branch at dispatch; any later receipt must record the actual selected source
and immutable image. Dispatch alone is not success.

Earlier successful run39532243053 built source
`32cb3bcfc9691212ec0b0359ce22474968bc6850` and published
`ghcr.io/prometheus-ags/universal-agent-runtime@sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8`.
It predates published embedding repair d35234ba and budget API0f926ed6.
It remains an older usable image, not evidence for current source.

No tests, command flags, gates, workflows, lint/typecheck commands,
source-identity checks or hidden preflights have been added. Verification CLOSED.
