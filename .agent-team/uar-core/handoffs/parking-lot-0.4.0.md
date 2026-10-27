# Parking Lot 0.4.0 source upgrade

Phase: `uar-post-lifecycle-verification-readiness`, user-outcome implementation.
Verification remains **CLOSED**. This is a source-authoring handoff, not acceptance.

## Official source identity

- Repository: `https://github.com/Prometheus-AGS/prometheus-parking-lot-rs`.
- Exact adopted commit: `f24382acd7ba818204e32f4449e9c9c37c52558d`.
- Commit timestamp: `2026-10-25T05:06:38Z`; subject: `Merge pull request #95 from Prometheus-AGS/codex/release-certification`.
- Official root `[workspace.package]` declares `0.4.0`, edition 2024 and Rust minimum 1.95.
- At retrieval on 2026-10-25, official latest release and tags still exposed only `v0.3.0`, pointing to old pin `4b747350c6883497813bed33623a9937ad7a5894`. Both official `v0.4.0` tag-ref and release-by-tag endpoints returned 404. Thus this is an **untagged source snapshot**, not a claim that 0.4.0 was published or signed.
- Source: <https://github.com/Prometheus-AGS/prometheus-parking-lot-rs/tree/f24382acd7ba818204e32f4449e9c9c37c52558d>.

Source-inspection retrievals used `gh api` for official latest release, tags, exact ref/release, main commit, root Cargo.toml, recursive exact tree and archive. Exact archive request:

```
gh api repos/Prometheus-AGS/prometheus-parking-lot-rs/tarball/f24382acd7ba818204e32f4449e9c9c37c52558d > /tmp/uar-parking-lot-0.4.0.tar.gz
```

Initial sandbox archive request could not connect. Escalated same read succeeded; no publishing or external mutation occurred. Archive members were read directly with Python tarfile; only explicitly selected relative production paths were exported. A compare API request returned a truncated first-300-file list dominated by upstream assurance records; it was not used as complete source-change evidence.

## Changed source

- `vendor/git/prometheus-parking-lot-rs/**`: replaced export with 125 real source/manifest/README/changelog/license/migration files from exact commit archive. All implementation bytes, including inline tests and target-specific sources, are upstream bytes. This includes 0.4 dispatcher/queue ownership behavior and the upstream durable mailbox/migration source changes, not merely edited version labels.
- Eight manifests materialize upstream workspace package/dependency/lint inheritance into standalone packages, retaining sibling relative paths, exact requirements, features, optionality, target conditions and upstream Rust minimum. Removed root workspace and development-only explicit bench/example targets whose files are not exported. The actual current standalone adaptation is preserved. Reintroducing `workspace = "../.."` would regress a previously repaired multiple-workspace-root deployment defect. Root continues excluding facade and all seven nested crates.
- `Cargo.toml`: core direct path dependency now also requires `version = "0.4.0"`. UAR continues to consume only the runtime-free core; it does not restore the facade/Tokio/optional backend dependency graph.
- `Cargo.lock`: only `prometheus-parking-lot-core` local package version changes `0.3.0 -> 0.4.0`. Exact upstream core dependency requirements are unchanged; manual local package-version reconciliation needed no Cargo solver command and adds no unrelated package changes.
- `versions.toml`: added scoped `prometheus_parking_lot = "0.4.0"` and exact commit pin with untagged provenance comment. Other architecture pins preserved.
- `vendor/git/README.md`: records new exact pin, observed tag/release absence, actual standalone adaptation and core-only UAR consumption; corrects prior facade-feature description.
- `docs/releases/v1.3.0-parking-lot.md`: advances dependency source identity to 0.4.0, keeps UAR 1.3.0 version, records source compatibility and separates upstream optional mailbox upgrades from UAR durable ingestion persistence.
- This handoff is newly authored.

Lead-requested acceptance registration, separate from dependency upgrade: `Cargo.toml` now registers `tests/user_outcome_grpc.rs` as `user_outcome_grpc`, required feature `a2a-transport`. Source inspection shows imports are generated A2A clients gated by that feature; the test consumes an externally owned installed host and does not launch server binaries. The feature requirement matches existing generated-client grpc targets. No test executed.

## Integration contract forwarded to lead/source owner

Upstream 0.4 `Dispatched<T>` adds:

```
pub rejected: Vec<(T, SchedulerError)>
```

With UAR, `T` remains `ScheduledTask<IngestionJob>`. The ingestion owner must drain rejected descriptors outside dispatch locks, remove each `descriptor.payload.storage_key()` from the admitted set, and call durable failure settlement with `TaskError::executor(error)`. This prevents rejected work being silently ignored. Lead forwarded this to the current ingestion source writer; this writer made **no edits** to `src/uar/rag/*` or persistence during the upgrade.

Other source-visible changes: `TaskQueue::prune_expired` and `ParkedQueue::prune_expired` return removed task vectors; `Dispatcher::prune_expired` keeps removal count and returns full items in its batch. UAR's wrapper does not implement those traits or invoke pruning. `DeliveryOutcome::Duplicate` now carries original status, and upstream PostgreSQL/redb mailbox formats advance to protocol/schema 3 with explicit evidence requirements. UAR does not consume upstream durable queues/mailboxes, so those source changes do not authorize or require a UAR ingestion-store migration. Existing core executor, metadata, permit, submit, complete and poke interfaces remain the relevant boundary.

## Unverified and ownership release

No Cargo update, metadata, check, test, compiler probe, formatting, lint, build, package check or independent review gate ran. No upstream release evidence is adopted as UAR evidence. Source compatibility, runtime rejection settlement, supported-profile compilation and installed-user outcomes remain unverified. Any full-phase acceptance requires renewed source inventory and exact operator approval.

Root Cargo.toml/Cargo.lock ownership is released to lead. All unrelated dirty files, including new KB/ingestion work and the vendored Liter subtree, were preserved. No live service/model commands, GitHub mutations, pushes or releases were made.
