# Issue #426 source implementation inventory — UAR 1.3.0

Gate: **CLOSED**. Source implementation and acceptance authoring are complete for this issue. No compilation, locked resolution, tests, formatting, lint, packaging, customer build or independent review has run. This inventory is not acceptance evidence or permission to execute a gate.

## Exact source identity

Upstream Parking Lot: `4b747350c6883497813bed33623a9937ad7a5894` (manifest 0.3.0), replacing `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`. The companion `issue-426-source-manifest.json` records the actual UAR checkout HEAD, each delivered file's SHA-256 and the removed superseded vendor paths. The manifest is an authored source-identity record, not a source-equivalence or resolver check. It excludes unrelated operator files and regenerable build/cache material. Its recorded production/test/documentation scope is explicit.

## File-by-file delivery inventory

| Files | Delivered source behavior/artifact | Author |
|---|---|---|
| vendor/git/prometheus-parking-lot-rs/** | 125 exact-commit blob exports: facade, all seven workspace production crates, required READMEs/licenses and PG migrations; obsolete 0.1 layout removed. Only root manifest bench/example targets omitted for omitted dev files. | parking_vendor |
| vendor/git/README.md | Exact pin, manifest version, inheritance/license record and reproducible export adaptation. | parking_vendor |
| Cargo.toml | Root UAR1.3.0, explicit facade tokio-runtime with default features disabled, direct same-export core path for dispatcher APIs. Toolchain and versions.toml unchanged. | lead |
| Cargo.lock | Root UAR1.3.0 identity, source-derived facade/core/Tokio/WASM0.3.0 graph entries and direct core edge using already locked registry identities. No Cargo resolver ran. | lead |
| src/server.rs | Existing one shared pool/CPU-count/100 waiting-descriptor configuration clarified; execution ownership remains scope-owned. | lead |
| src/uar/rag/ingestion_admission.rs | Production core-dispatcher adapter, independently bounded parked queue, strict local FIFO, oversized rejection, completion/poke/release wake and closure. | parking_adapter |
| src/uar/rag/ingestion_worker.rs | Migrated API/errors/metadata, descriptor deduplication before claim/source load, durable overflow, retained actual joins and permit settlement, cooperative deadlines/shutdown and admission/mailbox observations. | parking_adapter |
| src/uar/rag/mod.rs | Public production admission module declaration. | parking_adapter |
| tests/ingestion_admission.rs | Six authored integration cases entering production pool/adapter, real embedded store and owned controlled embedding gates. Existing durable/lifecycle suites preserved. | parking_acceptance |
| docs/releases/v1.3.0-parking-lot.md | Units/queue/lifetime/durability contract, exact upstream pin, fixture requirements and explicitly unverified acceptance. | parking_acceptance |
| CHANGELOG.md | Unreleased1.3.0 entry, issue426 scope and unverified status. | lead |
| openspec/changes/update-parking-lot-admission/** | Proposal, design, durable-ingestion spec delta, eight source/authoring tasks completed and two operator-gated tasks pending. | lead |
| Phase issue-426-* artifacts; plan.md appendix; canonical projections; team handoffs/state; .prometheus append-only notes | Approved scope/ownership, canonical registration/completion records, source identity, proposed commands, handoffs and limitations. | lead |

No unrequested production subsystem was changed. The direct core dependency and observation API implement the selected dispatcher boundary; required backend/FFI workspace source does not enable those optional products. No upstream workflows, tooling, standalone test suites or agent state were imported. Pre-existing `.codex/config.toml` edits and unrelated untracked memory files are preserved and excluded from this issue's identity scope.

## Whole active-phase reconciliation

Authoritative phase: `uar-post-lifecycle-verification-readiness`, Execute; plan revision99 adopted the new issue scope. Canonical status at revision286 showed the following six prior changes and every one of their registered tasks COMPLETE: `pl-main-budget-reconciliation`, `pl-main-compatible-budget-errors`, `pl-main-destination-budget-contract`, `pl-main-persistence-discriminators`, `pl-main-boundary-evidence`, `openai-client-tools`. Their existing handoff/implementation inventories and immutable acceptance/publication receipts remain historical; this new source does not inherit their runtime acceptance.

New issue implementation items: 1.1 scope/contract, 2.1 vendor export, 2.2 manifest/lock authoring, 3.1 bounded admission, 3.2 API/lifetime settlement, 3.3 acceptance/docs authoring, 4.1 version bump and 4.2 handoff/inventory. All eight source deliverables are authored and writer handoffs collected. Canonical transitions record this authoring separately from the still-pending 5.1 integration/build and 5.2 independent review tasks. Overall change/phase acceptance remains in progress; no generated counters are overridden. Run-wide outstanding work in other phases is not relabeled as active-phase implementation.

Collected writer handoffs:

- `.agent-team/uar-core/handoffs/issue-426-vendor.md`
- `.agent-team/uar-core/handoffs/issue-426-adapter.md`
- `.agent-team/uar-core/handoffs/issue-426-acceptance.md`
- `.agent-team/uar-core/handoffs/issue-426-lead.md`

No implementation writer remains active. No implementation item is knowingly omitted from this issue's source scope. Canonical task5.1/5.2 and all compatibility/runtime evidence remain pending. The gate stays CLOSED until the operator reviews this inventory, any canonical disagreement is resolved, and exact commands are explicitly approved.

## Authored scenarios and guards

Six new cases cover durable overflow/generation publication, two running slots, canceled/concurrent shutdown waiters before physical engine release, executor error/panic settlement, dropped/timed-out retrieval callers and real adapter oversized rejection/FIFO/queue bound/exceptional grant wake/closure. Their presence is not a passing test result. Existing durable crash/retry/delete fencing and retention/replay coverage remains part of the proposed finite gate.

Every new guard traces to the approved issue or an actual boundary: constructor integer/positive-capacity validation protects explicit capacity configuration; admission closure/spawn refusal protects shutdown races; durable namespace and exact-generation CAS protect persistence ownership; cooperative cancellation/deadlines protect publication; panic containment covers executor, claim and settlement failures; release-hook notification prevents stranded parked work after exceptional grant return. No default timeout, background retry loop, second durable queue, unrelated auth guard or speculative platform rewrite was added.

## Commands and unresolved evidence

Executed command categories: source/config/manifest reads; read-only GitHub issue/tree/blob retrieval; authorized local source/test/doc/memory writes; OpenSpec scaffolding/status/instructions; native team coordination and canonical KBD registration/transitions/status. The exact upstream tree selected125files and direct blob retrieval returned125payloads with no retrieval errors; the vendor writer's export reported125files. This is retrieval/authoring evidence only. Stalled redirected/archive source downloads were interrupted without a usable export; no gate ran accidentally.

**No verification command was run at any tier.** Explicitly not run: Cargo resolver/build/test/check, format/lint, git diff --check, OpenSpec validate, package/security audits, independent Rust/code review, container/customer builds or platform probes. No artifact was produced by a build, and no test PASS is claimed.

The exact proposed finite integration and native/mobile/container build commands are in [issue-426-gates.proposed.md](issue-426-gates.proposed.md). They require explicit operator approval; no readiness/identity/import preflight is added. Existing isolated durable-ingestion fixture settings must be supplied under that command's environment contract. Unsupported/missing collaborators remain BLOCKED/NOT RUN.

The uncomfortable remaining risk is resolver/compiler/runtime failure in source that has only been inspected. In particular, optional backend manifests remain upstream-exact, Cargo.lock was authored without executing Cargo, and the lifetime tests have not established real behavior. A production-compatible release and issue closure cannot be claimed until those gates and independent review pass for this exact source. No push, PR, tag, release or deployment occurred.


## Final canonical authoring receipt

Canonical revision296, plan99: eight issue source/authoring tasks1.1–4.2 COMPLETE; verification5.1 and independent review5.2 remain PENDING. All four local team source-authoring tasks are complete with writer handoff evidence. Overall acceptance remains IN_PROGRESS, gate CLOSED. Source manifest records UAR HEAD `75a2bfdcf467ef2c47b65cbcb045bece4e0e7df3`, 144 delivered source/test/doc files and 22 removed obsolete vendor paths. No command approval is inferred from these records.
